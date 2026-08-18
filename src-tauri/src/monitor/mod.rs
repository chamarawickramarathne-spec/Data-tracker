pub mod adapter;
pub mod aggregator;
pub mod app_usage;
pub mod alerts;

use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;

pub async fn start_monitoring(app: AppHandle) {
    log::info!("Starting network monitoring engine");

    let alerts_handle = app.clone();
    tokio::spawn(async move {
        crate::monitor::alerts::run(alerts_handle).await;
    });

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
        let app_usage = crate::monitor::app_usage::AppUsageTracker::new();

        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    app_usage.capture();
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

                    let adapter_total = session_in + session_out;
                    let app = monitor_handle.clone();
                    let adapter = last_adapter_name.clone();
                    let up = session_out;
                    let down = session_in;
                    let peak_up = last_upload_speed;
                    let peak_down = last_download_speed;
                    let mut app_samples = app_usage.flush();

                    session_in = 0;
                    session_out = 0;

                    let raw_app_total: u64 = app_samples.iter().map(|s| s.upload_bytes + s.download_bytes).sum();
                    if adapter_total > 0 && raw_app_total > adapter_total {
                        let scale = adapter_total as f64 / raw_app_total as f64;
                        log::warn!(
                            "App data ({}) exceeds adapter ({}), scaling by {:.4}",
                            crate::monitor::aggregator::format_bytes(raw_app_total),
                            crate::monitor::aggregator::format_bytes(adapter_total),
                            scale,
                        );
                        for s in &mut app_samples {
                            s.upload_bytes = (s.upload_bytes as f64 * scale) as u64;
                            s.download_bytes = (s.download_bytes as f64 * scale) as u64;
                        }
                    }

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

                        for sample in app_samples {
                            let _ = crate::db::queries::insert_app_usage(
                                &conn, &timestamp, &sample.app_name, "", sample.upload_bytes, sample.download_bytes,
                            );
                            let _ = crate::db::queries::upsert_app_daily_usage(
                                &conn, &date, &sample.app_name, sample.upload_bytes, sample.download_bytes,
                            );
                            let _ = crate::db::queries::upsert_app_monthly_usage(
                                &conn, year, month, &sample.app_name, sample.upload_bytes, sample.download_bytes,
                            );
                        }
                    });
                }
            }
        }
    });
}
