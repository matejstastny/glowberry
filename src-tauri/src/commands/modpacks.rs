use tauri::State;

use crate::error::LanternError;
use crate::modrinth::api::ModrinthApi;
use crate::modrinth::types::*;
use crate::state::AppState;

#[tauri::command]
pub async fn search_modpacks(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<SearchResult, LanternError> {
    let api = ModrinthApi::new(state.http_client.clone());
    api.search_modpacks(&query, limit.unwrap_or(20), offset.unwrap_or(0))
        .await
}

#[tauri::command]
pub async fn get_project(
    state: State<'_, AppState>,
    id_or_slug: String,
) -> Result<Project, LanternError> {
    let api = ModrinthApi::new(state.http_client.clone());
    api.get_project(&id_or_slug).await
}

#[tauri::command]
pub async fn list_versions(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<Version>, LanternError> {
    let api = ModrinthApi::new(state.http_client.clone());
    api.list_versions(&project_id).await
}
