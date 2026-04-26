use tauri::{AppHandle, State};

use crate::error::GlowberryError;
use crate::instance::manager::Instance;
use crate::minecraft::install;
use crate::state::AppState;

/// Install (or update in place) the Starlight modpack from a GitHub release URL.
#[tauri::command]
pub async fn install_starlight(
    app: AppHandle,
    state: State<'_, AppState>,
    mrpack_url: String,
    mrpack_name: String,
    mrpack_size: u64,
    version_tag: String,
) -> Result<Instance, GlowberryError> {
    // Find existing Starlight instance to reuse its ID (in-place update)
    let existing_id = {
        let instances = state.instances.lock().unwrap();
        instances
            .list()
            .unwrap_or_default()
            .into_iter()
            .find(|i| {
                i.modpack
                    .as_ref()
                    .is_some_and(|m| m.project_slug == "starlightmodpack")
            })
            .map(|i| i.id)
    };

    install::install_from_github(
        app,
        &state,
        mrpack_url,
        mrpack_name,
        mrpack_size,
        version_tag,
        existing_id,
    )
    .await
}
