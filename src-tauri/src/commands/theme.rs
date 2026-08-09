use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub theme: String,
    pub is_dark: bool,
}

#[tauri::command]
pub fn get_system_theme() -> Result<ThemeInfo, String> {
    let mode = dark_light::detect();
    let is_dark = match mode {
        dark_light::Mode::Dark => true,
        dark_light::Mode::Light => false,
        dark_light::Mode::Default => false,
    };

    Ok(ThemeInfo {
        theme: if is_dark { "dark".to_string() } else { "light".to_string() },
        is_dark,
    })
}
