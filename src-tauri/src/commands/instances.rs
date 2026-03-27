use tauri::State;

use crate::error::LanternError;
use crate::instance::manager::Instance;
use crate::state::AppState;

#[tauri::command]
pub fn list_instances(state: State<'_, AppState>) -> Result<Vec<Instance>, LanternError> {
    let manager = state.instances.lock().unwrap();
    manager.list()
}

#[tauri::command]
pub fn get_instance(state: State<'_, AppState>, id: String) -> Result<Instance, LanternError> {
    let manager = state.instances.lock().unwrap();
    manager.get(&id)
}

#[tauri::command]
pub fn delete_instance(state: State<'_, AppState>, id: String) -> Result<(), LanternError> {
    let manager = state.instances.lock().unwrap();
    manager.delete(&id)
}
