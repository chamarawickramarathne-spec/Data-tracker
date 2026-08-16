use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub date: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
    pub peak_upload_speed: u64,
    pub peak_download_speed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUsageSummary {
    pub app_name: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyUsage {
    pub year: i32,
    pub month: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyCalendarEntry {
    pub date: String,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyBreakdown {
    pub hour: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyBreakdownEntry {
    pub day: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppHourlyBreakdownEntry {
    pub hour: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDailyBreakdownEntry {
    pub day: u32,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
}

#[tauri::command]
pub fn get_daily_usage(
    db: State<'_, DbState>,
    date: String,
) -> Result<DailyUsage, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_daily_usage(&conn, &date)
        .map(|row| DailyUsage {
            date: row.date,
            upload_bytes: row.upload_bytes,
            download_bytes: row.download_bytes,
            total_bytes: row.total_bytes,
            peak_upload_speed: row.peak_upload_speed,
            peak_download_speed: row.peak_download_speed,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_monthly_usage(
    db: State<'_, DbState>,
    year: i32,
    month: u32,
) -> Result<MonthlyUsage, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_monthly_usage(&conn, year, month)
        .map(|row| MonthlyUsage {
            year: row.year,
            month: row.month,
            upload_bytes: row.upload_bytes,
            download_bytes: row.download_bytes,
            total_bytes: row.total_bytes,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_daily_app_breakdown(
    db: State<'_, DbState>,
    date: String,
) -> Result<Vec<AppUsageSummary>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_daily_app_breakdown(&conn, &date)
        .map(|rows| rows.into_iter().map(|r| AppUsageSummary {
            app_name: r.app_name,
            upload_bytes: r.upload_bytes,
            download_bytes: r.download_bytes,
            total_bytes: r.total_bytes,
            percentage: r.percentage,
        }).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_monthly_app_breakdown(
    db: State<'_, DbState>,
    year: i32,
    month: u32,
) -> Result<Vec<AppUsageSummary>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_monthly_app_breakdown(&conn, year, month)
        .map(|rows| rows.into_iter().map(|r| AppUsageSummary {
            app_name: r.app_name,
            upload_bytes: r.upload_bytes,
            download_bytes: r.download_bytes,
            total_bytes: r.total_bytes,
            percentage: r.percentage,
        }).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_usage_history(
    db: State<'_, DbState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyCalendarEntry>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_usage_history(&conn, &start_date, &end_date)
        .map(|rows| rows.into_iter().map(|r| DailyCalendarEntry {
            date: r.date,
            total_bytes: r.total_bytes,
        }).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_hourly_breakdown(
    db: State<'_, DbState>,
    date: String,
) -> Result<Vec<HourlyBreakdown>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_hourly_breakdown(&conn, &date)
        .map(|rows| rows.into_iter().map(|r| HourlyBreakdown {
            hour: r.hour,
            upload_bytes: r.upload_bytes,
            download_bytes: r.download_bytes,
        }).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_daily_breakdown(
    db: State<'_, DbState>,
    year: i32,
    month: u32,
) -> Result<Vec<DailyBreakdownEntry>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_daily_breakdown_for_month(&conn, year, month)
        .map(|rows| rows.into_iter().map(|r| DailyBreakdownEntry {
            day: r.day,
            upload_bytes: r.upload_bytes,
            download_bytes: r.download_bytes,
            total_bytes: r.total_bytes,
        }).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_hourly_breakdown(
    db: State<'_, DbState>,
    app_name: String,
    date: String,
) -> Result<Vec<AppHourlyBreakdownEntry>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_app_hourly_breakdown(&conn, &app_name, &date)
        .map(|rows| rows.into_iter().map(|r| AppHourlyBreakdownEntry {
            hour: r.hour,
            upload_bytes: r.upload_bytes,
            download_bytes: r.download_bytes,
            total_bytes: r.total_bytes,
        }).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_daily_breakdown_month(
    db: State<'_, DbState>,
    app_name: String,
    year: i32,
    month: u32,
) -> Result<Vec<AppDailyBreakdownEntry>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_app_daily_breakdown_month(&conn, &app_name, year, month)
        .map(|rows| rows.into_iter().map(|r| AppDailyBreakdownEntry {
            day: r.day,
            upload_bytes: r.upload_bytes,
            download_bytes: r.download_bytes,
            total_bytes: r.total_bytes,
        }).collect())
        .map_err(|e| e.to_string())
}
