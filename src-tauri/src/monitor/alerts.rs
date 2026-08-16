use std::sync::Mutex;

use chrono::{Datelike, Timelike};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use windows_sys::Win32::System::Diagnostics::Debug::MessageBeep;
use windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONASTERISK;

enum Level {
    Warning,
    Danger,
    Limit,
}

struct AlertState {
    day_key: String,
    day_warning_sent: bool,
    day_danger_sent: bool,
    day_limit_sent: bool,
    month_key: String,
    month_warning_sent: bool,
    month_danger_sent: bool,
    month_limit_sent: bool,
    summary_date: String,
}

impl AlertState {
    fn new() -> Self {
        Self {
            day_key: String::new(),
            day_warning_sent: false,
            day_danger_sent: false,
            day_limit_sent: false,
            month_key: String::new(),
            month_warning_sent: false,
            month_danger_sent: false,
            month_limit_sent: false,
            summary_date: String::new(),
        }
    }
}

pub async fn run(app: AppHandle) {
    let state = Mutex::new(AlertState::new());
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        check_limits(&app, &state);
        check_daily_summary(&app, &state);
    }
}

fn check_limits(app: &AppHandle, state: &Mutex<AlertState>) {
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let month_key = now.format("%Y-%m").to_string();
    let year = now.year();
    let month = now.month();

    let Some(db) = app.try_state::<crate::db::DbState>() else { return };
    let Ok(conn) = db.0.lock() else { return };
    let Ok(settings) = crate::db::queries::get_settings(&conn) else { return };
    if !settings.notifications_enabled {
        return;
    }

    let Ok(day) = crate::db::queries::get_daily_usage(&conn, &today) else { return };
    let Ok(month_usage) = crate::db::queries::get_monthly_usage(&conn, year, month) else { return };

    let mut alert_sent = false;
    {
        let mut s = state.lock().unwrap();

        if s.day_key != today {
            s.day_key = today.clone();
            s.day_warning_sent = false;
            s.day_danger_sent = false;
            s.day_limit_sent = false;
        }
        if settings.daily_limit_bytes > 0 {
            if let Some(level) = crossed(
                day.total_bytes,
                settings.daily_limit_bytes as u64,
                settings.warning_threshold_pct,
                settings.danger_threshold_pct,
            ) {
                let flag = match level {
                    Level::Warning => &mut s.day_warning_sent,
                    Level::Danger => &mut s.day_danger_sent,
                    Level::Limit => &mut s.day_limit_sent,
                };
                if !*flag {
                    *flag = true;
                    alert_sent = true;
                    let _ = send_alert(
                        app,
                        "Daily data limit reached",
                        format!(
                            "{} of {} used today ({:.0}% of your daily limit)",
                            crate::monitor::aggregator::format_bytes(day.total_bytes),
                            crate::monitor::aggregator::format_bytes(settings.daily_limit_bytes as u64),
                            day.total_bytes as f64 / settings.daily_limit_bytes as f64 * 100.0,
                        ),
                    );
                }
            }
        }

        if s.month_key != month_key {
            s.month_key = month_key.clone();
            s.month_warning_sent = false;
            s.month_danger_sent = false;
            s.month_limit_sent = false;
        }
        if settings.monthly_limit_bytes > 0 {
            if let Some(level) = crossed(
                month_usage.total_bytes,
                settings.monthly_limit_bytes as u64,
                settings.warning_threshold_pct,
                settings.danger_threshold_pct,
            ) {
                let flag = match level {
                    Level::Warning => &mut s.month_warning_sent,
                    Level::Danger => &mut s.month_danger_sent,
                    Level::Limit => &mut s.month_limit_sent,
                };
                if !*flag {
                    *flag = true;
                    alert_sent = true;
                    let _ = send_alert(
                        app,
                        "Monthly data limit reached",
                        format!(
                            "{} of {} used this month ({:.0}% of your monthly limit)",
                            crate::monitor::aggregator::format_bytes(month_usage.total_bytes),
                            crate::monitor::aggregator::format_bytes(settings.monthly_limit_bytes as u64),
                            month_usage.total_bytes as f64 / settings.monthly_limit_bytes as f64 * 100.0,
                        ),
                    );
                }
            }
        }
    }

    if alert_sent && settings.sound_alerts_enabled {
        play_sound();
    }
}

fn check_daily_summary(app: &AppHandle, state: &Mutex<AlertState>) {
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();

    let Some(db) = app.try_state::<crate::db::DbState>() else { return };
    let Ok(conn) = db.0.lock() else { return };
    let Ok(settings) = crate::db::queries::get_settings(&conn) else { return };
    if !settings.notifications_enabled || !settings.daily_summary_enabled {
        return;
    }

    let parts: Vec<&str> = settings.daily_summary_time.split(':').collect();
    if parts.len() != 2 {
        return;
    }
    let Ok(hour) = parts[0].trim().parse::<u32>() else { return };
    let Ok(minute) = parts[1].trim().parse::<u32>() else { return };
    let now_minutes = now.hour() * 60 + now.minute();
    if now_minutes < hour * 60 + minute {
        return;
    }

    {
        let mut s = state.lock().unwrap();
        if s.summary_date == today {
            return;
        }
        s.summary_date = today.clone();
    }

    let Ok(day) = crate::db::queries::get_daily_usage(&conn, &today) else { return };
    let _ = send_alert(
        app,
        "Daily usage summary",
        format!(
            "Downloaded {} · Uploaded {} · Total {} today",
            crate::monitor::aggregator::format_bytes(day.download_bytes),
            crate::monitor::aggregator::format_bytes(day.upload_bytes),
            crate::monitor::aggregator::format_bytes(day.total_bytes),
        ),
    );
    if settings.sound_alerts_enabled {
        play_sound();
    }
}

fn crossed(total: u64, limit: u64, warning_pct: i32, danger_pct: i32) -> Option<Level> {
    if total >= limit {
        Some(Level::Limit)
    } else if total >= limit * danger_pct as u64 / 100 {
        Some(Level::Danger)
    } else if total >= limit * warning_pct as u64 / 100 {
        Some(Level::Warning)
    } else {
        None
    }
}

fn send_alert(app: &AppHandle, title: &str, body: String) -> Result<(), tauri_plugin_notification::Error> {
    app.notification().builder().title(title).body(&body).show()
}

fn play_sound() {
    unsafe {
        MessageBeep(MB_ICONASTERISK);
    }
}
