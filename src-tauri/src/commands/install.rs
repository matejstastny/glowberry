use tauri::{AppHandle, State};

use crate::error::GlowberryError;
use crate::instance::manager::Instance;
use crate::minecraft::install;
use crate::state::AppState;

#[tauri::command]
pub async fn install_modpack(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    version_id: String,
) -> Result<Instance, GlowberryError> {
    install::install_modpack(app, &state, project_id, version_id).await
}
