use tauri::State;
use tauri_plugin_opener::OpenerExt;

use crate::error::GlowberryError;
use crate::state::AppState;

/// Open the Glowberry data folder in the system file manager.
#[tauri::command]
pub async fn open_data_folder(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), GlowberryError> {
    let url = url::Url::from_file_path(&state.data_dir)
        .map_err(|_| GlowberryError::Other("Invalid data directory path".into()))?;
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|e| GlowberryError::Other(e.to_string()))?;
    Ok(())
}
