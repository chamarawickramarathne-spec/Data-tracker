use rusqlite::{params, Connection, DatabaseName};
use std::path::{Path, PathBuf};

const SPIKE_THRESHOLD: i64 = 1_i64 << 40;
const ROLLUP_SPIKE_THRESHOLD: i64 = 1_i64 << 50;
const FILTER_SQL: &str = "adapter_name LIKE '%-QoS Packet Scheduler-%' \
    OR adapter_name LIKE '%-WFP %' \
    OR adapter_name LIKE '%-LightWeight Filter-%' \
    OR adapter_name LIKE '%-Endpoint-%'";

#[derive(Debug, Default)]
pub struct RepairReport {
    pub repaired: bool,
    pub backup_path: Option<PathBuf>,
    pub deleted_snapshots: usize,
    pub deleted_app_records: usize,
    pub reattributed_snapshots: usize,
    pub deduped_snapshots: usize,
    pub rebuilt_daily_rows: usize,
    pub rebuilt_monthly_rows: usize,
    pub rebuilt_daily_app_rows: usize,
    pub rebuilt_monthly_app_rows: usize,
}

pub fn repair_history(conn: &mut Connection, db_path: &Path) -> Result<RepairReport, String> {
    if !needs_repair(conn).map_err(|e| format!("repair detection failed: {e}"))? {
        return Ok(RepairReport::default());
    }

    let backup_path = backup_path_for(db_path);
    conn.backup(DatabaseName::Main, &backup_path, None)
        .map_err(|e| format!("database backup failed: {e}"))?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("repair transaction failed: {e}"))?;

    let deleted_snapshots = tx
        .execute(
            "DELETE FROM usage_snapshots WHERE total_bytes > ?1 OR total_bytes < 0",
            params![SPIKE_THRESHOLD],
        )
        .map_err(|e| format!("snapshot cleanup failed: {e}"))?;
    let deleted_app_records = tx
        .execute(
            "DELETE FROM app_usage_records WHERE total_bytes > ?1 OR total_bytes < 0",
            params![SPIKE_THRESHOLD],
        )
        .map_err(|e| format!("app cleanup failed: {e}"))?;

    let mut reattributed_snapshots = 0usize;
    let names = {
        let mut stmt = tx
            .prepare("SELECT DISTINCT adapter_name FROM usage_snapshots")
            .map_err(|e| format!("adapter query failed: {e}"))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("adapter query failed: {e}"))?;
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    };

    for name in names {
        if let Some(base) = physical_base(&name) {
            reattributed_snapshots += tx
                .execute(
                    "UPDATE usage_snapshots SET adapter_name = ?1 WHERE adapter_name = ?2",
                    params![base, name],
                )
                .map_err(|e| format!("adapter repair failed: {e}"))?;
        }
    }

    let deduped_snapshots = tx
        .execute(
            "DELETE FROM usage_snapshots WHERE id IN (
                SELECT id FROM (
                    SELECT id,
                           ROW_NUMBER() OVER (
                               PARTITION BY timestamp, adapter_name
                               ORDER BY total_bytes DESC, id DESC
                           ) AS row_num
                    FROM usage_snapshots
                ) WHERE row_num > 1
            )",
            [],
        )
        .map_err(|e| format!("snapshot deduplication failed: {e}"))?;

    for table in [
        "daily_usage",
        "monthly_usage",
        "daily_app_usage",
        "monthly_app_usage",
    ] {
        tx.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|e| format!("{table} reset failed: {e}"))?;
    }

    let rebuilt_daily_rows = tx
        .execute(
            "INSERT INTO daily_usage (
                date, adapter_name, upload_bytes, download_bytes, total_bytes,
                peak_upload_speed, peak_download_speed
             )
             SELECT date(timestamp), adapter_name,
                    SUM(upload_bytes), SUM(download_bytes), SUM(total_bytes),
                    MAX(upload_speed_bps), MAX(download_speed_bps)
             FROM usage_snapshots
             GROUP BY date(timestamp), adapter_name",
            [],
        )
        .map_err(|e| format!("daily usage rebuild failed: {e}"))?;
    let rebuilt_monthly_rows = tx
        .execute(
            "INSERT INTO monthly_usage (
                year, month, adapter_name, upload_bytes, download_bytes, total_bytes,
                peak_upload_speed, peak_download_speed
             )
             SELECT CAST(strftime('%Y', date) AS INTEGER),
                    CAST(strftime('%m', date) AS INTEGER), adapter_name,
                    SUM(upload_bytes), SUM(download_bytes), SUM(total_bytes),
                    MAX(peak_upload_speed), MAX(peak_download_speed)
             FROM daily_usage
             GROUP BY strftime('%Y-%m', date), adapter_name",
            [],
        )
        .map_err(|e| format!("monthly usage rebuild failed: {e}"))?;
    let rebuilt_daily_app_rows = tx
        .execute(
            "INSERT INTO daily_app_usage (
                date, app_name, upload_bytes, download_bytes, total_bytes
             )
             SELECT date(timestamp), app_name,
                    SUM(upload_bytes), SUM(download_bytes), SUM(total_bytes)
             FROM app_usage_records
             GROUP BY date(timestamp), app_name",
            [],
        )
        .map_err(|e| format!("daily app usage rebuild failed: {e}"))?;
    let rebuilt_monthly_app_rows = tx
        .execute(
            "INSERT INTO monthly_app_usage (
                year, month, app_name, upload_bytes, download_bytes, total_bytes
             )
             SELECT CAST(strftime('%Y', date) AS INTEGER),
                    CAST(strftime('%m', date) AS INTEGER), app_name,
                    SUM(upload_bytes), SUM(download_bytes), SUM(total_bytes)
             FROM daily_app_usage
             GROUP BY strftime('%Y-%m', date), app_name",
            [],
        )
        .map_err(|e| format!("monthly app usage rebuild failed: {e}"))?;

    tx.commit()
        .map_err(|e| format!("repair commit failed: {e}"))?;

    Ok(RepairReport {
        repaired: true,
        backup_path: Some(backup_path),
        deleted_snapshots,
        deleted_app_records,
        reattributed_snapshots,
        deduped_snapshots,
        rebuilt_daily_rows,
        rebuilt_monthly_rows,
        rebuilt_daily_app_rows,
        rebuilt_monthly_app_rows,
    })
}

fn needs_repair(conn: &Connection) -> rusqlite::Result<bool> {
    let sql = format!(
        "SELECT EXISTS(SELECT 1 FROM usage_snapshots WHERE total_bytes > ?1 OR total_bytes < 0)
             OR EXISTS(SELECT 1 FROM app_usage_records WHERE total_bytes > ?1 OR total_bytes < 0)
             OR EXISTS(SELECT 1 FROM usage_snapshots WHERE {FILTER_SQL})
             OR EXISTS(SELECT 1 FROM daily_usage WHERE {FILTER_SQL})
             OR EXISTS(SELECT 1 FROM monthly_usage WHERE {FILTER_SQL})
             OR EXISTS(SELECT 1 FROM daily_usage WHERE total_bytes > ?2 OR total_bytes < 0)
             OR EXISTS(SELECT 1 FROM monthly_usage WHERE total_bytes > ?2 OR total_bytes < 0)
             OR EXISTS(SELECT 1 FROM daily_app_usage WHERE total_bytes > ?1 OR total_bytes < 0)
             OR EXISTS(SELECT 1 FROM monthly_app_usage WHERE total_bytes > ?1 OR total_bytes < 0)"
    );
    conn.query_row(
        &sql,
        params![SPIKE_THRESHOLD, ROLLUP_SPIKE_THRESHOLD],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value != 0)
}

fn physical_base(name: &str) -> Option<String> {
    for marker in [
        "-QoS Packet Scheduler-",
        "-WFP ",
        "-LightWeight Filter-",
        "-Endpoint-",
    ] {
        if let Some(index) = name.find(marker) {
            let base = name[..index].trim();
            if !base.is_empty() && base != name {
                return Some(base.to_string());
            }
        }
    }
    None
}

fn backup_path_for(db_path: &Path) -> PathBuf {
    let file_name = db_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("datatracker.db");
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let mut path = db_path.with_file_name(format!("{file_name}.bak-{timestamp}"));
    let mut suffix = 1;
    while path.exists() {
        path = db_path.with_file_name(format!("{file_name}.bak-{timestamp}-{suffix}"));
        suffix += 1;
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn repairs_spikes_filters_and_rollups() {
        let path = temp_db_path();
        let mut conn = super::super::init_database(&path).unwrap();

        conn.execute_batch(
            "INSERT INTO usage_snapshots
                (timestamp, adapter_name, upload_bytes, download_bytes, total_bytes)
             VALUES
                ('2026-11-01 17:09:21', 'Ethernet-QoS Packet Scheduler-0000', 43825725950298, 0, 43825725950298),
                ('2026-11-01 17:11:21', 'Ethernet-QoS Packet Scheduler-0000', 43825725794941, 0, 43825725794941),
                ('2026-11-01 10:00:00', 'Ethernet', 1000000, 2000000, 3000000),
                ('2026-11-01 11:00:00', 'Ethernet-QoS Packet Scheduler-0000', 1000, 2000, 3000),
                ('2026-10-15 10:00:00', 'Ethernet-QoS Packet Scheduler-0000', 1000, 2000, 3000);
             INSERT INTO app_usage_records
                (timestamp, app_name, upload_bytes, download_bytes, total_bytes)
             VALUES
                ('2026-11-01 17:09:21', 'chrome.exe', 1500000000000, 0, 1500000000000),
                ('2026-11-01 10:00:00', 'chrome.exe', 500, 700, 1200),
                ('2026-10-15 10:00:00', 'firefox.exe', 100, 200, 300);
             INSERT INTO daily_usage
                (date, adapter_name, upload_bytes, download_bytes, total_bytes)
             VALUES
                ('2026-11-01', 'Ethernet-QoS Packet Scheduler-0000', 43825725950298, 0, 43825725950298);
             INSERT INTO monthly_usage
                (year, month, adapter_name, upload_bytes, download_bytes, total_bytes)
             VALUES
                (2026, 11, 'Ethernet-QoS Packet Scheduler-0000', 43825725950298, 0, 43825725950298);
             INSERT INTO daily_app_usage
                (date, app_name, upload_bytes, download_bytes, total_bytes)
             VALUES
                ('2026-11-01', 'chrome.exe', 1500000000000, 0, 1500000000000);
             INSERT INTO monthly_app_usage
                (year, month, app_name, upload_bytes, download_bytes, total_bytes)
             VALUES
                (2026, 11, 'chrome.exe', 1500000000000, 0, 1500000000000);",
        )
        .unwrap();

        let report = repair_history(&mut conn, &path).unwrap();
        assert!(report.repaired);
        assert_eq!(report.deleted_snapshots, 2);
        assert_eq!(report.deleted_app_records, 1);
        assert_eq!(report.reattributed_snapshots, 2);
        assert_eq!(report.deduped_snapshots, 0);
        assert!(report.backup_path.as_ref().unwrap().exists());

        assert_eq!(scalar(&conn, "SELECT COUNT(*) FROM usage_snapshots"), 3);
        assert_eq!(
            scalar(&conn, "SELECT SUM(total_bytes) FROM usage_snapshots"),
            3_006_000
        );
        assert_eq!(
            scalar(
                &conn,
                &format!("SELECT COUNT(*) FROM usage_snapshots WHERE {FILTER_SQL}")
            ),
            0
        );
        assert_eq!(
            scalar(
                &conn,
                "SELECT total_bytes FROM daily_usage WHERE date = '2026-11-01'"
            ),
            3_003_000
        );
        assert_eq!(
            scalar(
                &conn,
                "SELECT total_bytes FROM monthly_usage WHERE year = 2026 AND month = 11"
            ),
            3_003_000
        );
        assert_eq!(
            scalar(
                &conn,
                "SELECT total_bytes FROM daily_app_usage WHERE app_name = 'chrome.exe'"
            ),
            1_200
        );
        assert_eq!(
            scalar(
                &conn,
                "SELECT total_bytes FROM monthly_app_usage WHERE app_name = 'chrome.exe'"
            ),
            1_200
        );
        assert_eq!(
            scalar(
                &conn,
                "SELECT total_bytes FROM daily_app_usage WHERE app_name = 'firefox.exe'"
            ),
            300
        );

        let second = repair_history(&mut conn, &path).unwrap();
        assert!(!second.repaired);

        let backup = report.backup_path.unwrap();
        drop(conn);
        cleanup(&path);
        cleanup(&backup);
    }

    #[test]
    fn physical_base_strips_filter_suffixes() {
        assert_eq!(
            physical_base("Ethernet-QoS Packet Scheduler-0000"),
            Some("Ethernet".to_string())
        );
        assert_eq!(
            physical_base("Ethernet-WFP 802.3 MAC Layer LightWeight Filter-0000"),
            Some("Ethernet".to_string())
        );
        assert_eq!(physical_base("Ethernet"), None);
    }

    fn scalar(conn: &Connection, sql: &str) -> i64 {
        conn.query_row(sql, [], |row| row.get(0)).unwrap()
    }

    fn temp_db_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "datatracker-repair-test-{}-{nanos}.db",
            std::process::id()
        ))
    }

    fn cleanup(path: &Path) {
        let base = path.to_string_lossy();
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{base}{suffix}"));
        }
    }
}
