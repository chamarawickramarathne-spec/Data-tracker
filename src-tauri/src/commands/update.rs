use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::time::Duration;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub const UPDATE_REPO: &str = "chamarawickramarathne-spec/Data-tracker";
pub const UPDATE_EXE_NAME: &str = "data-tracker.exe";

#[derive(Serialize)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: Option<String>,
}

fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn version_gt(a: &str, b: &str) -> bool {
    match (parse_semver(a), parse_semver(b)) {
        (Some(x), Some(y)) => x > y,
        _ => false,
    }
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    tauri::async_runtime::spawn_blocking(run_check_for_updates)
        .await
        .map_err(|e| format!("Update check failed: {e}"))?
}

fn run_check_for_updates() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::blocking::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent("DataTracker-update-check")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let url = format!("https://github.com/{}/releases/latest", UPDATE_REPO);
    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to contact update server: {e}"))?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!("Update server returned HTTP {status} (no releases yet?)"));
    }

    let final_url = response.url().clone();
    let tag = final_url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .ok_or_else(|| "Update server returned an unexpected URL".to_string())?;
    let latest = tag.strip_prefix('v').unwrap_or(tag).to_string();
    let latest = if version_gt(&latest, &current) { Some(latest) } else { None };

    Ok(UpdateInfo { current, latest })
}

#[tauri::command]
pub async fn apply_update(repo: String, version: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || run_apply_update(&repo, &version))
        .await
        .map_err(|e| format!("Update apply failed: {e}"))?
}

fn run_apply_update(repo: &str, version: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to locate app: {e}"))?;
    let app_dir = exe.parent().ok_or("Failed to resolve app directory")?.to_path_buf();
    let exe_path = exe.as_path();
    let exe_name = exe_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(UPDATE_EXE_NAME);
    let staged = app_dir.join(format!("{}.new", exe_name));
    let script = app_dir.join("apply_update.cmd");

    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        repo, version, UPDATE_EXE_NAME
    );

    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent("DataTracker-update-check")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?
        .get(&url)
        .send()
        .map_err(|e| format!("Download failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .map_err(|e| format!("Download failed: {e}"))?;
    std::fs::write(&staged, bytes).map_err(|e| format!("Failed to stage update: {e}"))?;

    let quoted_exe = format!("\"{}\"", exe_path.display());
    let quoted_staged = format!("\"{}\"", staged.display());
    let quoted_script = format!("\"{}\"", script.display());
    let lines = format!(
        "@echo off\r\n\
         timeout /t 2 /nobreak >nul\r\n\
         :wait\r\n\
         del /q {quoted_exe} >nul 2>&1\r\n\
         if exist {quoted_exe} ( timeout /t 1 /nobreak >nul & goto wait )\r\n\
         move /y {quoted_staged} {quoted_exe} >nul\r\n\
         start \"\" {quoted_exe}\r\n\
         del /q {quoted_script} >nul 2>&1\r\n"
    );
    std::fs::write(&script, lines).map_err(|e| format!("Failed to write update script: {e}"))?;

    let launch = Command::new("cmd")
        .args(["/c", &script.to_string_lossy()])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Failed to launch updater: {e}"))?;
    drop(launch);

    Ok(())
}
