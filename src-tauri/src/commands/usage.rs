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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageForecast {
    pub daily_limit_bytes: i64,
    pub daily_used_bytes: u64,
    pub daily_rate_per_hour: f64,
    pub daily_hours_remaining: Option<f64>,
    pub daily_estimated_hit: Option<String>,
    pub monthly_limit_bytes: i64,
    pub monthly_used_bytes: u64,
    pub monthly_rate_per_day: f64,
    pub monthly_days_remaining: Option<f64>,
    pub monthly_estimated_hit: Option<String>,
}

#[tauri::command]
pub fn get_usage_forecast(
    db: State<'_, DbState>,
) -> Result<UsageForecast, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let settings = crate::db::queries::get_settings(&conn).map_err(|e| e.to_string())?;

    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2026);
    let month = now.format("%m").to_string().parse::<u32>().unwrap_or(1);

    let daily = crate::db::queries::get_today_usage_rate(&conn, &today)
        .unwrap_or(crate::db::queries::UsageForecastRow { total_bytes: 0, hours_active: 1.0 });
    let monthly = crate::db::queries::get_monthly_usage_rate(&conn, year, month)
        .unwrap_or(crate::db::queries::UsageForecastRow { total_bytes: 0, hours_active: 1.0 });

    let daily_rate = if daily.hours_active > 0.0 { daily.total_bytes as f64 / daily.hours_active } else { 0.0 };
    let monthly_rate = if monthly.hours_active > 0.0 { monthly.total_bytes as f64 / monthly.hours_active } else { 0.0 };

    let (daily_hours_remaining, daily_estimated_hit) = if settings.daily_limit_bytes > 0 && daily_rate > 0.0 {
        let remaining = (settings.daily_limit_bytes as u64).saturating_sub(daily.total_bytes) as f64;
        let hours = remaining / daily_rate;
        let hit_time = now + chrono::Duration::milliseconds((hours * 3600.0 * 1000.0) as i64);
        (Some(hours), Some(hit_time.format("%H:%M").to_string()))
    } else {
        (None, None)
    };

    let (monthly_days_remaining, monthly_estimated_hit) = if settings.monthly_limit_bytes > 0 && monthly_rate > 0.0 {
        let remaining = (settings.monthly_limit_bytes as u64).saturating_sub(monthly.total_bytes) as f64;
        let days = remaining / monthly_rate;
        let hit_time = now + chrono::Duration::milliseconds((days * 24.0 * 3600.0 * 1000.0) as i64);
        (Some(days), Some(hit_time.format("%b %d, %Y").to_string()))
    } else {
        (None, None)
    };

    Ok(UsageForecast {
        daily_limit_bytes: settings.daily_limit_bytes,
        daily_used_bytes: daily.total_bytes,
        daily_rate_per_hour: daily_rate,
        daily_hours_remaining,
        daily_estimated_hit,
        monthly_limit_bytes: settings.monthly_limit_bytes,
        monthly_used_bytes: monthly.total_bytes,
        monthly_rate_per_day: monthly_rate,
        monthly_days_remaining,
        monthly_estimated_hit,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeakHourEntry {
    pub day_of_week: u32,
    pub hour: u32,
    pub total_bytes: u64,
}

#[tauri::command]
pub fn get_peak_hours_heatmap(
    db: State<'_, DbState>,
    year: i32,
    month: u32,
) -> Result<Vec<PeakHourEntry>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let start_date = format!("{}-{:02}-01", year, month);
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let end_date = format!("{}-{:02}-01", next_year, next_month);

    crate::db::queries::get_peak_hours_data(&conn, &start_date, &end_date)
        .map(|rows| rows.into_iter().map(|r| PeakHourEntry {
            day_of_week: r.day_of_week,
            hour: r.hour,
            total_bytes: r.total_bytes,
        }).collect())
        .map_err(|e| e.to_string())
}
