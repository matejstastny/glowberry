use serde::Serialize;
use tauri::State;

use crate::error::LanternError;
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
) -> Result<(), LanternError> {
    let mut settings = Settings::load(&state.config_dir);
    settings.data_dir = path;
    settings
        .save(&state.config_dir)
        .map_err(|e| LanternError::Other(format!("Failed to save settings: {e}")))
}
