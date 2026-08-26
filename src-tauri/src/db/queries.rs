use crate::db::{UserSettings};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsageRow {
    pub date: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
    pub peak_upload_speed: u64,
    pub peak_download_speed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyUsageRow {
    pub year: i32,
    pub month: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageSummaryRow {
    pub app_name: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCalendarRow {
    pub date: String,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyBreakdownRow {
    pub hour: u32,
    pub download_bytes: u64,
    pub upload_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBreakdownRow {
    pub day: u32,
    pub download_bytes: u64,
    pub upload_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppHourlyBreakdownRow {
    pub hour: u32,
    pub download_bytes: u64,
    pub upload_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDailyBreakdownRow {
    pub day: u32,
    pub download_bytes: u64,
    pub upload_bytes: u64,
    pub total_bytes: u64,
}

pub fn get_settings(conn: &Connection) -> Result<UserSettings> {
    conn.query_row(
        "SELECT daily_limit_bytes, monthly_limit_bytes, warning_threshold_pct,
                danger_threshold_pct, notifications_enabled, sound_alerts_enabled,
                auto_start_enabled, minimize_to_tray, theme, data_retention_days,
                selected_adapter, daily_summary_enabled, daily_summary_time
         FROM user_settings WHERE id = 'default'",
        [],
        |row| {
            Ok(UserSettings {
                daily_limit_bytes: row.get(0)?,
                monthly_limit_bytes: row.get(1)?,
                warning_threshold_pct: row.get(2)?,
                danger_threshold_pct: row.get(3)?,
                notifications_enabled: row.get::<_, i32>(4)? != 0,
                sound_alerts_enabled: row.get::<_, i32>(5)? != 0,
                auto_start_enabled: row.get::<_, i32>(6)? != 0,
                minimize_to_tray: row.get::<_, i32>(7)? != 0,
                theme: row.get(8)?,
                data_retention_days: row.get(9)?,
                selected_adapter: row.get(10)?,
                daily_summary_enabled: row.get::<_, i32>(11)? != 0,
                daily_summary_time: row.get(12)?,
            })
        },
    )
}

pub fn update_settings(
    conn: &Connection,
    daily_limit_bytes: Option<i64>,
    monthly_limit_bytes: Option<i64>,
    warning_threshold_pct: Option<i32>,
    danger_threshold_pct: Option<i32>,
    notifications_enabled: Option<bool>,
    sound_alerts_enabled: Option<bool>,
    auto_start_enabled: Option<bool>,
    minimize_to_tray: Option<bool>,
    theme: Option<String>,
    data_retention_days: Option<i32>,
    selected_adapter: Option<String>,
    daily_summary_enabled: Option<bool>,
    daily_summary_time: Option<String>,
) -> Result<UserSettings> {
    conn.execute(
        "UPDATE user_settings SET
            daily_limit_bytes = COALESCE(?1, daily_limit_bytes),
            monthly_limit_bytes = COALESCE(?2, monthly_limit_bytes),
            warning_threshold_pct = COALESCE(?3, warning_threshold_pct),
            danger_threshold_pct = COALESCE(?4, danger_threshold_pct),
            notifications_enabled = COALESCE(?5, notifications_enabled),
            sound_alerts_enabled = COALESCE(?6, sound_alerts_enabled),
            auto_start_enabled = COALESCE(?7, auto_start_enabled),
            minimize_to_tray = COALESCE(?8, minimize_to_tray),
            theme = COALESCE(?9, theme),
            data_retention_days = COALESCE(?10, data_retention_days),
            selected_adapter = COALESCE(?11, selected_adapter),
            daily_summary_enabled = COALESCE(?12, daily_summary_enabled),
            daily_summary_time = COALESCE(?13, daily_summary_time),
            updated_at = datetime('now')
        WHERE id = 'default'",
        params![
            daily_limit_bytes,
            monthly_limit_bytes,
            warning_threshold_pct,
            danger_threshold_pct,
            notifications_enabled.map(|v| v as i32),
            sound_alerts_enabled.map(|v| v as i32),
            auto_start_enabled.map(|v| v as i32),
            minimize_to_tray.map(|v| v as i32),
            theme,
            data_retention_days,
            selected_adapter,
            daily_summary_enabled.map(|v| v as i32),
            daily_summary_time,
        ],
    )?;

    get_settings(conn)
}

pub fn upsert_daily_usage(
    conn: &Connection,
    date: &str,
    adapter_name: &str,
    upload_bytes: u64,
    download_bytes: u64,
    upload_speed: u64,
    download_speed: u64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO daily_usage (date, adapter_name, upload_bytes, download_bytes,
         total_bytes, peak_upload_speed, peak_download_speed)
         VALUES (?1, ?2, ?3, ?4, ?3 + ?4, ?5, ?6)
         ON CONFLICT(date, adapter_name) DO UPDATE SET
            upload_bytes = upload_bytes + excluded.upload_bytes,
            download_bytes = download_bytes + excluded.download_bytes,
            total_bytes = total_bytes + excluded.total_bytes,
            peak_upload_speed = MAX(peak_upload_speed, excluded.peak_upload_speed),
            peak_download_speed = MAX(peak_download_speed, excluded.peak_download_speed)",
        params![date, adapter_name, upload_bytes, download_bytes, upload_speed, download_speed],
    )?;
    Ok(())
}

pub fn upsert_monthly_usage(
    conn: &Connection,
    year: i32,
    month: u32,
    adapter_name: &str,
    upload_bytes: u64,
    download_bytes: u64,
    upload_speed: u64,
    download_speed: u64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO monthly_usage (year, month, adapter_name, upload_bytes, download_bytes,
         total_bytes, peak_upload_speed, peak_download_speed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?4 + ?5, ?6, ?7)
         ON CONFLICT(year, month, adapter_name) DO UPDATE SET
            upload_bytes = upload_bytes + excluded.upload_bytes,
            download_bytes = download_bytes + excluded.download_bytes,
            total_bytes = total_bytes + excluded.total_bytes,
            peak_upload_speed = MAX(peak_upload_speed, excluded.peak_upload_speed),
            peak_download_speed = MAX(peak_download_speed, excluded.peak_download_speed)",
        params![year, month as i32, adapter_name, upload_bytes, download_bytes, upload_speed, download_speed],
    )?;
    Ok(())
}

pub fn insert_snapshot(
    conn: &Connection,
    timestamp: &str,
    adapter_name: &str,
    upload_bytes: u64,
    download_bytes: u64,
    upload_speed: u64,
    download_speed: u64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO usage_snapshots (timestamp, adapter_name, upload_bytes, download_bytes,
         total_bytes, upload_speed_bps, download_speed_bps) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![timestamp, adapter_name, upload_bytes, download_bytes,
                upload_bytes + download_bytes, upload_speed, download_speed],
    )?;
    Ok(())
}

pub fn insert_app_usage(
    conn: &Connection,
    timestamp: &str,
    app_name: &str,
    app_path: &str,
    upload_bytes: u64,
    download_bytes: u64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO app_usage_records (timestamp, app_name, app_path, upload_bytes, download_bytes, total_bytes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![timestamp, app_name, app_path, upload_bytes, download_bytes, upload_bytes + download_bytes],
    )?;
    Ok(())
}

pub fn get_daily_usage(conn: &Connection, date: &str) -> Result<DailyUsageRow> {
    conn.query_row(
        "SELECT COALESCE(date, ?1) as date,
                COALESCE(SUM(upload_bytes), 0), COALESCE(SUM(download_bytes), 0),
                COALESCE(SUM(total_bytes), 0), COALESCE(MAX(peak_upload_speed), 0),
                COALESCE(MAX(peak_download_speed), 0)
         FROM daily_usage WHERE date = ?1",
        params![date],
        |row| {
            Ok(DailyUsageRow {
                date: row.get(0)?,
                upload_bytes: row.get(1)?,
                download_bytes: row.get(2)?,
                total_bytes: row.get(3)?,
                peak_upload_speed: row.get(4)?,
                peak_download_speed: row.get(5)?,
            })
        },
    )
}

pub fn get_monthly_usage(conn: &Connection, year: i32, month: u32) -> Result<MonthlyUsageRow> {
    conn.query_row(
        "SELECT ?1 as year, ?2 as month,
                COALESCE(SUM(upload_bytes), 0), COALESCE(SUM(download_bytes), 0),
                COALESCE(SUM(total_bytes), 0)
         FROM monthly_usage WHERE year = ?1 AND month = ?2",
        params![year, month as i32],
        |row| {
            Ok(MonthlyUsageRow {
                year: row.get(0)?,
                month: row.get::<_, i32>(1)? as u32,
                upload_bytes: row.get(2)?,
                download_bytes: row.get(3)?,
                total_bytes: row.get(4)?,
            })
        },
    )
}

pub fn get_daily_app_breakdown(conn: &Connection, date: &str) -> Result<Vec<AppUsageSummaryRow>> {
    let mut stmt = conn.prepare(
        "SELECT app_name, upload_bytes, download_bytes, total_bytes,
                CASE WHEN (SELECT SUM(total_bytes) FROM daily_app_usage WHERE date = ?1) > 0
                     THEN CAST(total_bytes AS REAL) / (SELECT SUM(total_bytes) FROM daily_app_usage WHERE date = ?1) * 100
                     ELSE 0 END as percentage
         FROM daily_app_usage WHERE date = ?1
         ORDER BY total_bytes DESC",
    )?;

    let rows = stmt.query_map(params![date], |row| {
        Ok(AppUsageSummaryRow {
            app_name: row.get(0)?,
            upload_bytes: row.get(1)?,
            download_bytes: row.get(2)?,
            total_bytes: row.get(3)?,
            percentage: row.get(4)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_monthly_app_breakdown(conn: &Connection, year: i32, month: u32) -> Result<Vec<AppUsageSummaryRow>> {
    let mut stmt = conn.prepare(
        "SELECT app_name, upload_bytes, download_bytes, total_bytes,
                CASE WHEN (SELECT SUM(total_bytes) FROM monthly_app_usage WHERE year = ?1 AND month = ?2) > 0
                     THEN CAST(total_bytes AS REAL) / (SELECT SUM(total_bytes) FROM monthly_app_usage WHERE year = ?1 AND month = ?2) * 100
                     ELSE 0 END as percentage
         FROM monthly_app_usage WHERE year = ?1 AND month = ?2
         ORDER BY total_bytes DESC",
    )?;

    let rows = stmt.query_map(params![year, month as i32], |row| {
        Ok(AppUsageSummaryRow {
            app_name: row.get(0)?,
            upload_bytes: row.get(1)?,
            download_bytes: row.get(2)?,
            total_bytes: row.get(3)?,
            percentage: row.get(4)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_usage_history(conn: &Connection, start_date: &str, end_date: &str) -> Result<Vec<DailyCalendarRow>> {
    let mut stmt = conn.prepare(
        "SELECT date, COALESCE(SUM(total_bytes), 0)
         FROM daily_usage WHERE date BETWEEN ?1 AND ?2
         GROUP BY date ORDER BY date",
    )?;

    let rows = stmt.query_map(params![start_date, end_date], |row| {
        Ok(DailyCalendarRow {
            date: row.get(0)?,
            total_bytes: row.get(1)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_hourly_breakdown(conn: &Connection, date: &str) -> Result<Vec<HourlyBreakdownRow>> {
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour,
                COALESCE(SUM(download_bytes), 0),
                COALESCE(SUM(upload_bytes), 0)
         FROM usage_snapshots WHERE date(timestamp) = ?1
         GROUP BY hour ORDER BY hour",
    )?;

    let rows = stmt.query_map(params![date], |row| {
        Ok(HourlyBreakdownRow {
            hour: row.get(0)?,
            download_bytes: row.get(1)?,
            upload_bytes: row.get(2)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_daily_breakdown_for_month(conn: &Connection, year: i32, month: u32) -> Result<Vec<DailyBreakdownRow>> {
    let start_date = format!("{}-{:02}-01", year, month);
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let end_date = format!("{}-{:02}-01", next_year, next_month);

    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%d', date) AS INTEGER) as day,
                COALESCE(SUM(download_bytes), 0),
                COALESCE(SUM(upload_bytes), 0),
                COALESCE(SUM(total_bytes), 0)
         FROM daily_usage WHERE date >= ?1 AND date < ?2
         GROUP BY date ORDER BY day",
    )?;

    let rows = stmt.query_map(params![start_date, end_date], |row| {
        Ok(DailyBreakdownRow {
            day: row.get(0)?,
            download_bytes: row.get(1)?,
            upload_bytes: row.get(2)?,
            total_bytes: row.get(3)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn upsert_app_daily_usage(
    conn: &Connection,
    date: &str,
    app_name: &str,
    upload_bytes: u64,
    download_bytes: u64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO daily_app_usage (date, app_name, upload_bytes, download_bytes, total_bytes)
         VALUES (?1, ?2, ?3, ?4, ?3 + ?4)
         ON CONFLICT(date, app_name) DO UPDATE SET
            upload_bytes = upload_bytes + excluded.upload_bytes,
            download_bytes = download_bytes + excluded.download_bytes,
            total_bytes = total_bytes + excluded.total_bytes",
        params![date, app_name, upload_bytes, download_bytes],
    )?;
    Ok(())
}

pub fn upsert_app_monthly_usage(
    conn: &Connection,
    year: i32,
    month: u32,
    app_name: &str,
    upload_bytes: u64,
    download_bytes: u64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO monthly_app_usage (year, month, app_name, upload_bytes, download_bytes, total_bytes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?4 + ?5)
         ON CONFLICT(year, month, app_name) DO UPDATE SET
            upload_bytes = upload_bytes + excluded.upload_bytes,
            download_bytes = download_bytes + excluded.download_bytes,
            total_bytes = total_bytes + excluded.total_bytes",
        params![year, month as i32, app_name, upload_bytes, download_bytes],
    )?;
    Ok(())
}

pub fn get_app_hourly_breakdown(
    conn: &Connection,
    app_name: &str,
    date: &str,
) -> Result<Vec<AppHourlyBreakdownRow>> {
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour,
                COALESCE(SUM(download_bytes), 0),
                COALESCE(SUM(upload_bytes), 0),
                COALESCE(SUM(total_bytes), 0)
         FROM app_usage_records WHERE app_name = ?1 AND date(timestamp) = ?2
         GROUP BY hour ORDER BY hour",
    )?;

    let rows = stmt.query_map(params![app_name, date], |row| {
        Ok(AppHourlyBreakdownRow {
            hour: row.get(0)?,
            download_bytes: row.get(1)?,
            upload_bytes: row.get(2)?,
            total_bytes: row.get(3)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_app_daily_breakdown_month(
    conn: &Connection,
    app_name: &str,
    year: i32,
    month: u32,
) -> Result<Vec<AppDailyBreakdownRow>> {
    let start_date = format!("{}-{:02}-01", year, month);
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let end_date = format!("{}-{:02}-01", next_year, next_month);

    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%d', date) AS INTEGER) as day,
                COALESCE(SUM(download_bytes), 0),
                COALESCE(SUM(upload_bytes), 0),
                COALESCE(SUM(total_bytes), 0)
         FROM daily_app_usage WHERE app_name = ?1 AND date >= ?2 AND date < ?3
         GROUP BY date ORDER BY day",
    )?;

    let rows = stmt.query_map(params![app_name, start_date, end_date], |row| {
        Ok(AppDailyBreakdownRow {
            day: row.get(0)?,
            download_bytes: row.get(1)?,
            upload_bytes: row.get(2)?,
            total_bytes: row.get(3)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakHourCell {
    pub day_of_week: u32,
    pub hour: u32,
    pub total_bytes: u64,
}

pub fn get_peak_hours_data(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<PeakHourCell>> {
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%w', timestamp) AS INTEGER) as dow,
                CAST(strftime('%H', timestamp) AS INTEGER) as hour,
                COALESCE(SUM(total_bytes), 0)
         FROM usage_snapshots
         WHERE date(timestamp) >= ?1 AND date(timestamp) < ?2
         GROUP BY dow, hour
         ORDER BY dow, hour",
    )?;

    let rows = stmt.query_map(params![start_date, end_date], |row| {
        Ok(PeakHourCell {
            day_of_week: row.get(0)?,
            hour: row.get(1)?,
            total_bytes: row.get(2)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageForecastRow {
    pub total_bytes: u64,
    pub hours_active: f64,
}

pub fn get_today_usage_rate(conn: &Connection, date: &str) -> Result<UsageForecastRow> {
    let row = conn.query_row(
        "SELECT COALESCE(SUM(total_bytes), 0),
                MAX(CAST(strftime('%H', timestamp) AS REAL) + CAST(strftime('%M', timestamp) AS REAL) / 60.0)
                    - MIN(CAST(strftime('%H', timestamp) AS REAL) + CAST(strftime('%M', timestamp) AS REAL) / 60.0)
         FROM usage_snapshots WHERE date(timestamp) = ?1",
        params![date],
        |row| {
            Ok(UsageForecastRow {
                total_bytes: row.get(0)?,
                hours_active: row.get::<_, Option<f64>>(1)?.unwrap_or(0.0).max(1.0),
            })
        },
    )?;
    Ok(row)
}

pub fn get_monthly_usage_rate(conn: &Connection, year: i32, month: u32) -> Result<UsageForecastRow> {
    let start_date = format!("{}-{:02}-01", year, month);
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let end_date = format!("{}-{:02}-01", next_year, next_month);

    let row = conn.query_row(
        "SELECT COALESCE(SUM(total_bytes), 0),
                MAX(CAST(strftime('%d', date) AS REAL)) - MIN(CAST(strftime('%d', date) AS REAL)) + 1
         FROM daily_usage WHERE date >= ?1 AND date < ?2",
        params![start_date, end_date],
        |row| {
            Ok(UsageForecastRow {
                total_bytes: row.get(0)?,
                hours_active: row.get::<_, Option<f64>>(1)?.unwrap_or(1.0).max(1.0),
            })
        },
    )?;
    Ok(row)
}
