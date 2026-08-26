use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedTestResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub latency_ms: f64,
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedTestProgress {
    pub phase: String,
    pub progress: f64,
}

#[tauri::command]
pub async fn run_speed_test(
    app: tauri::AppHandle,
) -> Result<SpeedTestResult, String> {
    use futures_util::StreamExt;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("DataTracker/1.1.0")
        .build()
        .map_err(|e| e.to_string())?;

    // Latency test
    let mut latencies = Vec::new();
    for _ in 0..5 {
        let start = std::time::Instant::now();
        let _ = client
            .get("https://speed.cloudflare.com/__down?bytes=0")
            .send()
            .await;
        latencies.push(start.elapsed().as_millis() as f64);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let latency_ms = latencies.get(latencies.len() / 2).copied().unwrap_or(0.0);

    let _ = app.emit("speedtest-progress", SpeedTestProgress { phase: "download".into(), progress: 0.0 });

    // Download test - 25MB
    let dl_start = std::time::Instant::now();
    let dl_response = client
        .get("https://speed.cloudflare.com/__down?bytes=26214400")
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;
    let mut dl_stream = dl_response.bytes_stream();
    let mut dl_bytes: u64 = 0;
    while let Some(chunk) = dl_stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream error: {e}"))?;
        dl_bytes += chunk.len() as u64;
        let progress = (dl_bytes as f64 / 26_214_400.0 * 100.0).min(100.0);
        let _ = app.emit("speedtest-progress", SpeedTestProgress { phase: "download".into(), progress });
    }
    let dl_elapsed = dl_start.elapsed().as_secs_f64();
    let download_mbps = if dl_elapsed > 0.0 { (dl_bytes as f64 * 8.0) / (dl_elapsed * 1_000_000.0) } else { 0.0 };

    let _ = app.emit("speedtest-progress", SpeedTestProgress { phase: "upload".into(), progress: 0.0 });

    // Upload test - 25MB
    let upload_data = vec![0u8; 26_214_400];
    let ul_start = std::time::Instant::now();
    let ul_response = client
        .post("https://speed.cloudflare.com/__up")
        .body(upload_data)
        .send()
        .await
        .map_err(|e| format!("Upload failed: {e}"))?;
    let _ = ul_response.bytes().await;
    let ul_elapsed = ul_start.elapsed().as_secs_f64();
    let upload_mbps = if ul_elapsed > 0.0 { (26_214_400.0 * 8.0) / (ul_elapsed * 1_000_000.0) } else { 0.0 };

    let _ = app.emit("speedtest-progress", SpeedTestProgress { phase: "done".into(), progress: 100.0 });

    Ok(SpeedTestResult {
        download_mbps: (download_mbps * 100.0).round() / 100.0,
        upload_mbps: (upload_mbps * 100.0).round() / 100.0,
        latency_ms: (latency_ms * 10.0).round() / 10.0,
        server: "Cloudflare".to_string(),
    })
}
