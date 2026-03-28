mod auth;
mod commands;
mod download;
mod error;
mod instance;
mod minecraft;
mod modrinth;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = directories::ProjectDirs::from("com", "lantern", "Lantern")
        .expect("Failed to determine data directory")
        .data_dir()
        .to_path_buf();

    let app_state = AppState::new(data_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::auth::start_login,
            commands::auth::get_auth_status,
            commands::auth::try_restore_session,
            commands::auth::logout,
            commands::instances::list_instances,
            commands::instances::get_instance,
            commands::instances::delete_instance,
            commands::modpacks::search_modpacks,
            commands::modpacks::get_project,
            commands::modpacks::list_versions,
            commands::file_locks::list_instance_files,
            commands::file_locks::set_file_lock,
            commands::file_locks::get_locked_files,
            commands::install::install_modpack,
            commands::launch::launch_instance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lantern");
}
