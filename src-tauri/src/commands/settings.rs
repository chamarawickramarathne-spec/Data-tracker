use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::State;
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResponse {
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
}

#[tauri::command]
pub fn get_settings(db: State<'_, DbState>) -> Result<SettingsResponse, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let settings = crate::db::queries::get_settings(&conn).map_err(|e| e.to_string())?;
    Ok(SettingsResponse {
        daily_limit_bytes: settings.daily_limit_bytes,
        monthly_limit_bytes: settings.monthly_limit_bytes,
        warning_threshold_pct: settings.warning_threshold_pct,
        danger_threshold_pct: settings.danger_threshold_pct,
        notifications_enabled: settings.notifications_enabled,
        sound_alerts_enabled: settings.sound_alerts_enabled,
        auto_start_enabled: settings.auto_start_enabled,
        minimize_to_tray: settings.minimize_to_tray,
        theme: settings.theme,
        data_retention_days: settings.data_retention_days,
        selected_adapter: settings.selected_adapter,
    })
}

#[tauri::command(rename_args = "camelCase")]
pub fn update_settings(
    app: AppHandle,
    db: State<'_, DbState>,
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
) -> Result<SettingsResponse, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let settings = crate::db::queries::update_settings(
        &conn,
        daily_limit_bytes,
        monthly_limit_bytes,
        warning_threshold_pct,
        danger_threshold_pct,
        notifications_enabled,
        sound_alerts_enabled,
        auto_start_enabled,
        minimize_to_tray,
        theme,
        data_retention_days,
        selected_adapter,
    )
    .map_err(|e| e.to_string())?;

    if let Some(enabled) = auto_start_enabled {
        let autolaunch = app.autolaunch();
        let _ = if enabled {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
    }

    Ok(SettingsResponse {
        daily_limit_bytes: settings.daily_limit_bytes,
        monthly_limit_bytes: settings.monthly_limit_bytes,
        warning_threshold_pct: settings.warning_threshold_pct,
        danger_threshold_pct: settings.danger_threshold_pct,
        notifications_enabled: settings.notifications_enabled,
        sound_alerts_enabled: settings.sound_alerts_enabled,
        auto_start_enabled: settings.auto_start_enabled,
        minimize_to_tray: settings.minimize_to_tray,
        theme: settings.theme,
        data_retention_days: settings.data_retention_days,
        selected_adapter: settings.selected_adapter,
    })
}
