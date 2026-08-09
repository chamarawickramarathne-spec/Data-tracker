use chrono::Utc;

pub struct AggregatedData {
    pub timestamp: String,
    pub adapter_name: String,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub total_download: u64,
    pub total_upload: u64,
}

pub fn aggregate_snapshot(
    download_speed: u64,
    upload_speed: u64,
    total_download: u64,
    total_upload: u64,
    adapter_name: &str,
) -> AggregatedData {
    AggregatedData {
        timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        adapter_name: adapter_name.to_string(),
        upload_speed,
        download_speed,
        total_upload,
        total_download,
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}
