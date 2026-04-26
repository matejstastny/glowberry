use serde::Serialize;
use tauri::State;

use crate::error::GlowberryError;
use crate::settings::Settings;
use crate::state::AppState;

#[derive(Serialize)]
pub struct SettingsInfo {
    pub data_dir: String,
    pub default_data_dir: String,
    pub data_dir_override: Option<String>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> SettingsInfo {
    let settings = Settings::load(&state.config_dir);
    SettingsInfo {
        data_dir: state.data_dir.to_string_lossy().to_string(),
        default_data_dir: state.default_data_dir.to_string_lossy().to_string(),
        data_dir_override: settings.data_dir,
    }
}

#[tauri::command]
pub fn set_data_dir(
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<(), GlowberryError> {
    let mut settings = Settings::load(&state.config_dir);
    settings.data_dir = path;
    settings
        .save(&state.config_dir)
        .map_err(|e| GlowberryError::Other(format!("Failed to save settings: {e}")))
}

#[tauri::command]
pub fn show_main_window(window: tauri::Window) -> Result<(), String> {
    if window.is_minimized().map_err(|e| e.to_string())? {
        window.unminimize().map_err(|e| e.to_string())?;
    }
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}
