use tauri::State;

use crate::error::GlowberryError;
use crate::instance::file_lock::{self, FileEntry};
use crate::state::AppState;

#[tauri::command]
pub fn list_instance_files(
    state: State<'_, AppState>,
    instance_id: String,
    search: Option<String>,
) -> Result<Vec<FileEntry>, GlowberryError> {
    let manager = state.instances.lock().unwrap();
    let instance = manager.get(&instance_id)?;
    let mc_dir = manager.minecraft_dir(&instance_id);
    file_lock::list_files(&mc_dir, &instance, search.as_deref())
}

#[tauri::command]
pub fn set_file_lock(
    state: State<'_, AppState>,
    instance_id: String,
    path: String,
    locked: bool,
) -> Result<(), GlowberryError> {
    let manager = state.instances.lock().unwrap();
    let mut instance = manager.get(&instance_id)?;

    if locked {
        instance.locked_files.insert(path);
    } else {
        instance.locked_files.remove(&path);
    }

    manager.save(&instance)?;
    Ok(())
}

#[tauri::command]
pub fn get_locked_files(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<String>, GlowberryError> {
    let manager = state.instances.lock().unwrap();
    let instance = manager.get(&instance_id)?;
    Ok(instance.locked_files.into_iter().collect())
}
