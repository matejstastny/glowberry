use tauri::State;

use crate::error::GlowberryError;
use crate::instance::manager::Instance;
use crate::modrinth::mrpack::extract_overrides;
use crate::state::AppState;

/// Return sorted preset names available for an instance.
#[tauri::command]
pub async fn list_presets(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<String>, GlowberryError> {
    let presets_dir = state
        .data_dir
        .join("instances")
        .join(&instance_id)
        .join("presets");

    if !presets_dir.exists() {
        return Ok(vec![]);
    }

    let mut names = Vec::new();
    for entry in std::fs::read_dir(&presets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("mrpack") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Apply a preset's overrides to the instance game directory and persist the
/// active preset on the instance. Saves/ and servers.dat are never touched.
#[tauri::command]
pub async fn switch_preset(
    state: State<'_, AppState>,
    instance_id: String,
    preset_name: String,
) -> Result<Instance, GlowberryError> {
    let instance_dir = state.data_dir.join("instances").join(&instance_id);
    let preset_path = instance_dir
        .join("presets")
        .join(format!("{preset_name}.mrpack"));

    if !preset_path.exists() {
        return Err(GlowberryError::Other(format!(
            "Preset '{preset_name}' not found"
        )));
    }

    let game_dir = instance_dir.join("game");
    tokio::fs::create_dir_all(&game_dir).await?;

    let mut locked = std::collections::HashSet::new();
    locked.insert("saves/".to_string());
    locked.insert("servers.dat".to_string());

    let preset_path_clone = preset_path.clone();
    let game_dir_clone = game_dir.clone();
    tokio::task::spawn_blocking(move || {
        extract_overrides(&preset_path_clone, &game_dir_clone, &locked)
    })
    .await
    .map_err(|e| GlowberryError::Other(format!("Extract task failed: {e}")))??;

    let mut instance = {
        let instances = state.instances.lock().unwrap();
        instances.get(&instance_id)?
    };
    instance.active_preset = Some(preset_name);
    {
        let instances = state.instances.lock().unwrap();
        instances.save(&instance)?;
    }

    Ok(instance)
}
