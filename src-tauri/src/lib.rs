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
    let dirs = directories::ProjectDirs::from("com", "glowberry", "Glowberry")
        .expect("Failed to determine data directory");
    let data_dir = dirs.data_dir().to_path_buf();
    eprintln!("[init] Data directory: {}", data_dir.display());

    let app_state = AppState::new(data_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::start_login,
            commands::auth::cancel_login,
            commands::auth::get_auth_status,
            commands::auth::try_restore_session,
            commands::auth::logout,
            // Instances
            commands::instances::list_instances,
            commands::instances::set_instance_memory,
            // Install
            commands::install::install_starlight,
            // GitHub update check
            commands::github::check_starlight_update,
            // Launch
            commands::launch::launch_instance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Glowberry");
}
