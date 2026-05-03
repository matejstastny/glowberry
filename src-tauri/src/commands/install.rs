use tauri::{AppHandle, State};

use crate::error::GlowberryError;
use crate::instance::manager::Instance;
use crate::minecraft::install;
use crate::state::AppState;

/// Install (or update in place) the Starlight modpack from a GitHub release asset.
#[tauri::command]
pub async fn install_starlight(
    app: AppHandle,
    state: State<'_, AppState>,
    asset_url: String,
    asset_name: String,
    asset_size: u64,
    version_tag: String,
) -> Result<Instance, GlowberryError> {
    let (existing_id, existing_active_preset) = {
        let instances = state.instances.lock().unwrap();
        let existing = instances
            .list()
            .unwrap_or_default()
            .into_iter()
            .find(|i| {
                i.modpack
                    .as_ref()
                    .is_some_and(|m| m.project_slug == "starlightmodpack")
            });
        (
            existing.as_ref().map(|i| i.id.clone()),
            existing.and_then(|i| i.active_preset),
        )
    };

    install::install_from_github(
        app,
        &state,
        asset_url,
        asset_name,
        asset_size,
        version_tag,
        existing_id,
        existing_active_preset,
    )
    .await
}
