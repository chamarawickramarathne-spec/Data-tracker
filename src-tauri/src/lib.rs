mod commands;
mod db;
mod monitor;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize the database
            let db_path = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir")
                .join("datatracker.db");

            std::fs::create_dir_all(db_path.parent().unwrap()).ok();

            let db = db::init_database(&db_path).expect("failed to init database");

            // Apply autostart setting from database
            {
                use tauri_plugin_autostart::ManagerExt;
                if let Ok(settings) = crate::db::queries::get_settings(&db) {
                    let autolaunch = app.autolaunch();
                    if settings.auto_start_enabled {
                        let _ = autolaunch.enable();
                    } else {
                        let _ = autolaunch.disable();
                    }
                }
            }

            app.manage(db::DbState(std::sync::Mutex::new(db)));

            app.manage(commands::network::LiveSpeed(std::sync::Mutex::new(
                commands::network::NetworkSpeed {
                    download_speed: 0,
                    upload_speed: 0,
                    total_download: 0,
                    total_upload: 0,
                    adapter_name: String::new(),
                },
            )));

            // Start network monitor in background
            let monitor_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                monitor::start_monitoring(monitor_handle).await;
            });

            // Setup system tray
            setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::network::get_current_speed,
            commands::network::get_adapters,
            commands::network::set_adapter,
            commands::network::get_app_usage_realtime,
            commands::usage::get_daily_usage,
            commands::usage::get_monthly_usage,
            commands::usage::get_daily_app_breakdown,
            commands::usage::get_monthly_app_breakdown,
            commands::usage::get_usage_history,
            commands::usage::get_hourly_breakdown,
            commands::usage::get_daily_breakdown,
            commands::usage::get_app_hourly_breakdown,
            commands::usage::get_app_daily_breakdown_month,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::theme::get_system_theme,
            commands::update::check_for_updates,
            commands::update::apply_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};

    let show_item = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app).item(&show_item).item(&quit_item).build()?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Data Tracker")
        .on_tray_icon_event(|tray_icon, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray_icon.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
