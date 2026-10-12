use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::time::Duration;
use windows_sys::Win32::Security::WinTrust::*;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

// The update repository is pinned server-side. The frontend passes a repo only
// to select the download URL, but verification always targets this constant.
pub const UPDATE_REPO: &str = "chamarawickramarathne-spec/Data-tracker";

// Maps the running process architecture to the release asset suffix.
fn arch_suffix() -> &'static str {
    match std::env::consts::ARCH {
        "x86" => "x86",
        _ => "x64",
    }
}

// Release asset name for this architecture, e.g. "data-tracker-x64.exe".
fn update_exe_name() -> String {
    format!("data-tracker-{}.exe", arch_suffix())
}
const GENERIC_VERIFY_V2: windows_sys::core::GUID =
    windows_sys::core::GUID::from_u128(0x00aac56b_cd44_11d0_8cc2_00c04fc295ee);

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
pub async fn apply_update(
    app: tauri::AppHandle,
    version: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || run_apply_update(&version))
        .await
        .map_err(|e| format!("Update apply failed: {e}"))??;

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        handle.exit(0);
    });

    Ok(())
}

fn run_apply_update(version: &str) -> Result<(), String> {
    // The repo used for the download is always the pinned constant.
    verify_version_format(version)?;

    let exe = std::env::current_exe().map_err(|e| format!("Failed to locate app: {e}"))?;
    let app_dir = exe.parent().ok_or("Failed to resolve app directory")?.to_path_buf();
    let exe_path = exe.as_path();
    let exe_name = exe_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("data-tracker.exe");
    let staged = app_dir.join(format!("{}.new", exe_name));
    let script = app_dir.join("apply_update.cmd");
    let asset_name = update_exe_name();

    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        UPDATE_REPO, version, asset_name
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
    std::fs::write(&staged, &bytes).map_err(|e| format!("Failed to stage update: {e}"))?;

    // Fetch the expected SHA-256 checksum from the release body and verify it.
    let expected_sha = fetch_release_checksum(version, arch_suffix())?;
    verify_sha256(&bytes, &expected_sha)?;

    // Verify the Authenticode signature of the downloaded executable.
    verify_authenticode(&staged)?;

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

fn verify_version_format(version: &str) -> Result<(), String> {
    if parse_semver(version).is_none() {
        return Err("Invalid version format refused".to_string());
    }
    if version.contains('/') || version.contains('\\') || version.contains("..") {
        return Err("Version contains invalid characters".to_string());
    }
    Ok(())
}

fn fetch_release_checksum(version: &str, arch: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent("DataTracker-update-check")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    // Primary source: a machine-readable SHA256SUMS.txt published as a release
    // asset (sha256sum format). This is unambiguous and immune to HTML markup.
    let sums_url = format!(
        "https://github.com/{}/releases/download/v{}/SHA256SUMS.txt",
        UPDATE_REPO, version
    );
    if let Ok(resp) = client.get(&sums_url).send() {
        if resp.status().is_success() {
            if let Ok(text) = resp.text() {
                if let Some(hex) = parse_checksum_file(&text, &update_exe_name(), arch) {
                    return Ok(hex);
                }
                log::warn!(
                    "SHA256SUMS.txt present but contained no checksum for {}",
                    update_exe_name()
                );
            }
        }
    }

    // Fallback: scan the rendered release page. The body text is wrapped in
    // HTML (<p>/<br>/</p>), so match an architecture-specific label first
    // (SHA256-x64: / SHA256-x86:) then a plain SHA256: line, and take the
    // first complete 64-character hex digest that follows.
    let page_url = format!("https://github.com/{}/releases/tag/v{}", UPDATE_REPO, version);
    let response = client
        .get(&page_url)
        .send()
        .map_err(|e| format!("Failed to fetch release checksum: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("Failed to fetch release checksum: HTTP {}", response.status()));
    }
    let body = response
        .text()
        .map_err(|e| format!("Failed to read release body: {e}"))?;

    let arch_prefix = format!("SHA256-{}:", arch);
    if let Some(hex) = extract_hex_after(&body, &arch_prefix) {
        return Ok(hex);
    }
    if let Some(hex) = extract_hex_after(&body, "SHA256:") {
        return Ok(hex);
    }
    Err("Release has no valid SHA256 checksum to verify against".to_string())
}

// Parses a sha256sum-format file ("<hex>  <filename>") and returns the digest
// for the requested asset. A legacy "data-tracker.exe" line is accepted as an
// x64 fallback for releases published before architecture-specific payloads.
fn parse_checksum_file(text: &str, asset_name: &str, arch: &str) -> Option<String> {
    let mut legacy: Option<String> = None;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (hex, name) = match (parts.next(), parts.next()) {
            (Some(h), Some(n)) => (h, n.trim_start_matches('*')),
            _ => continue,
        };
        let hex = match normalize_hex(hex) {
            Some(h) => h,
            None => continue,
        };
        if name == asset_name {
            return Some(hex);
        }
        if arch == "x64" && name == "data-tracker.exe" && legacy.is_none() {
            legacy = Some(hex);
        }
    }
    legacy
}

// Finds the first occurrence of `prefix` followed by a complete 64-character
// hex digest, tolerating whitespace and surrounding HTML between them.
fn extract_hex_after(body: &str, prefix: &str) -> Option<String> {
    let mut search_from = 0usize;
    while let Some(pos) = body[search_from..].find(prefix) {
        let start = search_from + pos + prefix.len();
        let rest = body[start..].trim_start();
        let hex: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        if hex.len() == 64 {
            return Some(hex.to_ascii_lowercase());
        }
        search_from = start;
    }
    None
}

// Accepts a trimmed 64-character hex digest and returns it lowercased.
fn normalize_hex(hex: &str) -> Option<String> {
    let hex = hex.trim();
    if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}

fn verify_sha256(data: &[u8], expected_hex: &str) -> Result<(), String> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual = hasher.finalize();
    let actual_hex = actual
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    if actual_hex != expected_hex.to_ascii_lowercase() {
        return Err("Downloaded update failed SHA-256 verification; update aborted".to_string());
    }
    Ok(())
}

fn verify_authenticode(path: &std::path::Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: wide.as_ptr(),
        hFile: std::ptr::null_mut(),
        pgKnownSubject: std::ptr::null_mut(),
    };

    let mut data = WINTRUST_DATA {
        cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
        pPolicyCallbackData: std::ptr::null_mut(),
        pSIPClientData: std::ptr::null_mut(),
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 { pFile: &mut file_info },
        dwStateAction: WTD_STATEACTION_VERIFY,
        hWVTStateData: std::ptr::null_mut(),
        pwszURLReference: std::ptr::null_mut(),
        dwProvFlags: WTD_REVOCATION_CHECK_NONE | WTD_SAFER_FLAG,
        dwUIContext: WTD_UICONTEXT_EXECUTE,
        pSignatureSettings: std::ptr::null_mut(),
    };

    let mut action = GENERIC_VERIFY_V2;

    let result = unsafe {
        WinVerifyTrust(
            std::ptr::null_mut(),
            &mut action,
            &mut data as *mut WINTRUST_DATA as *mut c_void,
        )
    };

    // Always release the trust state handle before branching on the result.
    if data.hWVTStateData != std::ptr::null_mut() {
        unsafe {
            data.dwStateAction = WTD_STATEACTION_CLOSE;
            WinVerifyTrust(
                std::ptr::null_mut(),
                &mut action,
                &mut data as *mut WINTRUST_DATA as *mut c_void,
            );
        }
    }

    if result == 0 {
        return Ok(());
    }

    // This project ships unsigned executables (no code-signing certificate).
    // A missing signature is tolerated so that SHA-256-verified updates can
    // still be applied. A signature that is present but invalid or untrusted
    // is always rejected.
    if is_unsigned(result) {
        log::warn!(
            "Downloaded update is not Authenticode-signed (0x{:08X}); relying on SHA-256 verification",
            result as u32
        );
        return Ok(());
    }

    Err(format!(
        "Downloaded update failed Authenticode signature verification (0x{:08X}); update aborted",
        result as u32
    ))
}

// WinVerifyTrust error codes that indicate no signature is present at all.
fn is_unsigned(result: i32) -> bool {
    const TRUST_E_NOSIGNATURE: i32 = 0x800B0100u32 as i32;
    const TRUST_E_SUBJECT_FORM_UNKNOWN: i32 = 0x800B0003u32 as i32;
    const TRUST_E_PROVIDER_UNKNOWN: i32 = 0x800B0001u32 as i32;
    result == TRUST_E_NOSIGNATURE
        || result == TRUST_E_SUBJECT_FORM_UNKNOWN
        || result == TRUST_E_PROVIDER_UNKNOWN
}
