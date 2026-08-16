pub mod queries;

use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

#[derive(Debug, Clone)]
pub struct UserSettings {
    pub daily_limit_bytes: i64,
    pub monthly_limit_bytes: i64,
    pub warning_threshold_pct: i32,
    pub danger_threshold_pct: i32,
    pub notifications_enabled: bool,
    pub sound_alerts_enabled: bool,
    pub auto_start_enabled: bool,
    pub minimize_to_tray: bool,
    pub theme: String,
    pub data_retention_days: i32,
    pub selected_adapter: String,
    pub daily_summary_enabled: bool,
    pub daily_summary_time: String,
}

pub fn init_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS user_settings (
            id TEXT PRIMARY KEY DEFAULT 'default',
            daily_limit_bytes INTEGER NOT NULL DEFAULT 0,
            monthly_limit_bytes INTEGER NOT NULL DEFAULT 0,
            warning_threshold_pct INTEGER NOT NULL DEFAULT 80,
            danger_threshold_pct INTEGER NOT NULL DEFAULT 95,
            notifications_enabled INTEGER NOT NULL DEFAULT 1,
            sound_alerts_enabled INTEGER NOT NULL DEFAULT 0,
            auto_start_enabled INTEGER NOT NULL DEFAULT 1,
            minimize_to_tray INTEGER NOT NULL DEFAULT 1,
            theme TEXT NOT NULL DEFAULT 'auto',
            data_retention_days INTEGER NOT NULL DEFAULT 90,
            selected_adapter TEXT NOT NULL DEFAULT '',
            daily_summary_enabled INTEGER NOT NULL DEFAULT 0,
            daily_summary_time TEXT NOT NULL DEFAULT '20:00',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS usage_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            adapter_name TEXT NOT NULL DEFAULT '',
            upload_bytes INTEGER NOT NULL DEFAULT 0,
            download_bytes INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            upload_speed_bps INTEGER NOT NULL DEFAULT 0,
            download_speed_bps INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS app_usage_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            app_name TEXT NOT NULL,
            app_path TEXT NOT NULL DEFAULT '',
            upload_bytes INTEGER NOT NULL DEFAULT 0,
            download_bytes INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS daily_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            adapter_name TEXT NOT NULL DEFAULT '',
            upload_bytes INTEGER NOT NULL DEFAULT 0,
            download_bytes INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            peak_upload_speed INTEGER NOT NULL DEFAULT 0,
            peak_download_speed INTEGER NOT NULL DEFAULT 0,
            UNIQUE(date, adapter_name)
        );

        CREATE TABLE IF NOT EXISTS daily_app_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            app_name TEXT NOT NULL,
            upload_bytes INTEGER NOT NULL DEFAULT 0,
            download_bytes INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            UNIQUE(date, app_name)
        );

        CREATE TABLE IF NOT EXISTS monthly_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            year INTEGER NOT NULL,
            month INTEGER NOT NULL,
            adapter_name TEXT NOT NULL DEFAULT '',
            upload_bytes INTEGER NOT NULL DEFAULT 0,
            download_bytes INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            peak_upload_speed INTEGER NOT NULL DEFAULT 0,
            peak_download_speed INTEGER NOT NULL DEFAULT 0,
            UNIQUE(year, month, adapter_name)
        );

        CREATE TABLE IF NOT EXISTS monthly_app_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            year INTEGER NOT NULL,
            month INTEGER NOT NULL,
            app_name TEXT NOT NULL,
            upload_bytes INTEGER NOT NULL DEFAULT 0,
            download_bytes INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            UNIQUE(year, month, app_name)
        );

        CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON usage_snapshots(timestamp);
        CREATE INDEX IF NOT EXISTS idx_app_usage_timestamp ON app_usage_records(timestamp);
        CREATE INDEX IF NOT EXISTS idx_daily_usage_date ON daily_usage(date);
        CREATE INDEX IF NOT EXISTS idx_daily_app_usage_date ON daily_app_usage(date);
        CREATE INDEX IF NOT EXISTS idx_monthly_usage_period ON monthly_usage(year, month);
        CREATE INDEX IF NOT EXISTS idx_monthly_app_usage_period ON monthly_app_usage(year, month);
        ",
    )?;

    // Insert default settings if not exists
    conn.execute(
        "INSERT OR IGNORE INTO user_settings (id) VALUES ('default')",
        [],
    )?;

    // Migrations for databases created before these columns existed
    for migration in [
        "ALTER TABLE user_settings ADD COLUMN daily_summary_enabled INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE user_settings ADD COLUMN daily_summary_time TEXT NOT NULL DEFAULT '20:00'",
    ] {
        let _ = conn.execute(migration, []);
    }

    Ok(conn)
}
