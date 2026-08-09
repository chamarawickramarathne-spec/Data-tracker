use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
use crate::db::DbState;

pub struct LiveSpeed(pub Mutex<NetworkSpeed>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSpeed {
    pub download_speed: u64,
    pub upload_speed: u64,
    pub total_download: u64,
    pub total_upload: u64,
    pub adapter_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAdapter {
    pub name: String,
    pub description: String,
    pub is_connected: bool,
    pub speed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNetworkUsage {
    pub pid: u32,
    pub app_name: String,
    pub app_path: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
}

#[tauri::command]
pub fn get_current_speed(state: State<'_, LiveSpeed>) -> Result<NetworkSpeed, String> {
    let speed = state.0.lock().map_err(|e| e.to_string())?;
    Ok(speed.clone())
}

#[tauri::command]
pub fn get_adapters() -> Result<Vec<NetworkAdapter>, String> {
    crate::monitor::adapter::get_network_adapters()
        .map(|adapters| adapters.into_iter().map(|a| NetworkAdapter {
            name: a.name,
            description: a.description,
            is_connected: a.is_connected,
            speed: a.speed,
        }).collect())
}

#[tauri::command]
pub fn set_adapter(_db: State<'_, DbState>, _name: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn get_app_usage_realtime() -> Result<Vec<AppNetworkUsage>, String> {
    Ok(Vec::new())
}
