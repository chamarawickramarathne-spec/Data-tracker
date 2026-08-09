pub mod adapter;
pub mod aggregator;

use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;

pub async fn start_monitoring(app: AppHandle) {
    log::info!("Starting network monitoring engine");

    let monitor_handle = app.clone();
    tokio::spawn(async move {
        let mut tick_interval = tokio::time::interval(std::time::Duration::from_secs(3));
        let mut save_interval = tokio::time::interval(std::time::Duration::from_secs(60));
        save_interval.tick().await;

        let mut prev_in: Option<u64> = None;
        let mut prev_out: Option<u64> = None;
        let mut total_in: u64 = 0;
        let mut total_out: u64 = 0;
        let mut session_in: u64 = 0;
        let mut session_out: u64 = 0;
        let mut last_adapter_name = String::new();
        let mut last_download_speed: u64 = 0;
        let mut last_upload_speed: u64 = 0;

        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    if let Ok(stats) = adapter::get_adapter_stats() {
                        let current_in = stats.bytes_received;
                        let current_out = stats.bytes_sent;

                        let download_speed = prev_in.map(|p| current_in.saturating_sub(p)).unwrap_or(0);
                        let upload_speed = prev_out.map(|p| current_out.saturating_sub(p)).unwrap_or(0);

                        if prev_in.is_some() {
                            total_in += download_speed;
                            total_out += upload_speed;
                            session_in += download_speed;
                            session_out += upload_speed;
                        }

                        prev_in = Some(current_in);
                        prev_out = Some(current_out);
                        last_download_speed = download_speed;
                        last_upload_speed = upload_speed;
                        last_adapter_name = stats.name.clone();

                        if let Some(state) = monitor_handle.try_state::<crate::commands::network::LiveSpeed>() {
                            let mut speed = state.0.lock().unwrap();
                            speed.download_speed = download_speed;
                            speed.upload_speed = upload_speed;
                            speed.total_download = total_in;
                            speed.total_upload = total_out;
                            speed.adapter_name = stats.name;
                        }

                        let _ = monitor_handle.emit("network-speed", serde_json::json!({
                            "downloadSpeed": download_speed,
                            "uploadSpeed": upload_speed,
                            "totalDownload": total_in,
                            "totalUpload": total_out,
                            "adapterName": last_adapter_name,
                        }));
                    }
                }
                _ = save_interval.tick() => {
                    if session_in == 0 && session_out == 0 {
                        continue;
                    }

                    let app = monitor_handle.clone();
                    let adapter = last_adapter_name.clone();
                    let up = session_out;
                    let down = session_in;
                    let peak_up = last_upload_speed;
                    let peak_down = last_download_speed;

                    session_in = 0;
                    session_out = 0;

                    tokio::task::spawn_blocking(move || {
                        let Some(state) = app.try_state::<crate::db::DbState>() else { return };
                        let Ok(conn) = state.0.lock() else { return };

                        let now = chrono::Local::now();
                        let date = now.format("%Y-%m-%d").to_string();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let year: i32 = now.format("%Y").to_string().parse().unwrap_or(2026);
                        let month: u32 = now.format("%m").to_string().parse().unwrap_or(1);

                        let _ = crate::db::queries::insert_snapshot(
                            &conn, &timestamp, &adapter, up, down, peak_up, peak_down,
                        );

                        let _ = crate::db::queries::upsert_daily_usage(
                            &conn, &date, &adapter, up, down, peak_up, peak_down,
                        );

                        let _ = crate::db::queries::upsert_monthly_usage(
                            &conn, year, month, &adapter, up, down, peak_up, peak_down,
                        );
                    });
                }
            }
        }
    });
}
