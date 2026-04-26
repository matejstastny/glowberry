use tauri::State;

use crate::error::GlowberryError;
use crate::modrinth::api::ModrinthApi;
use crate::modrinth::types::Version;
use crate::state::AppState;

#[tauri::command]
pub async fn list_versions(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<Version>, GlowberryError> {
    let api = ModrinthApi::new(state.http_client.clone());
    api.list_versions(&project_id).await
}
