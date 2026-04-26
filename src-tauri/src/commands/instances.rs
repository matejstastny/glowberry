use tauri::State;

use crate::error::GlowberryError;
use crate::instance::manager::Instance;
use crate::state::AppState;

#[tauri::command]
pub fn list_instances(state: State<'_, AppState>) -> Result<Vec<Instance>, GlowberryError> {
    let manager = state.instances.lock().unwrap();
    manager.list()
}

#[tauri::command]
pub fn set_instance_memory(
    state: State<'_, AppState>,
    id: String,
    memory_mb: u32,
) -> Result<(), GlowberryError> {
    let manager = state.instances.lock().unwrap();
    let mut instance = manager.get(&id)?;
    instance.memory_mb = memory_mb;
    manager.save(&instance)
}
