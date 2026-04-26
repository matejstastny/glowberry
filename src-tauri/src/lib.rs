mod auth;
mod commands;
mod download;
mod error;
mod instance;
mod minecraft;
mod modrinth;
mod settings;
mod state;

use settings::Settings;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let dirs = directories::ProjectDirs::from("com", "glowberry", "Glowberry")
        .expect("Failed to determine data directory");
    let config_dir = dirs.config_dir().to_path_buf();
    let default_data_dir = dirs.data_dir().to_path_buf();

    let settings = Settings::load(&config_dir);
    let data_dir = settings.resolve_data_dir(&default_data_dir);
    eprintln!("[init] Data directory: {}", data_dir.display());

    let app_state = AppState::new(data_dir, config_dir, default_data_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::instances::get_instance,
            commands::instances::delete_instance,
            commands::instances::set_instance_memory,
            // Install
            commands::install::install_modpack,
            commands::install::install_starlight,
            // GitHub update check
            commands::github::check_starlight_update,
            // Launch
            commands::launch::launch_instance,
            // Settings
            commands::settings::get_settings,
            commands::settings::set_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Glowberry");
}
