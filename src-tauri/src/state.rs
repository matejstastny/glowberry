use std::path::PathBuf;
use std::sync::Mutex;

use crate::auth::microsoft::{AuthTokens, MinecraftProfile};
use crate::instance::manager::InstanceManager;

pub struct AuthState {
    pub profile: Option<MinecraftProfile>,
    pub tokens: Option<AuthTokens>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            profile: None,
            tokens: None,
        }
    }
}

pub struct AppState {
    pub http_client: reqwest::Client,
    pub instances: Mutex<InstanceManager>,
    pub auth: Mutex<AuthState>,
    pub data_dir: PathBuf,
}

fn migrate_game_dirs(data_dir: &std::path::Path) {
    let instances_dir = data_dir.join("instances");
    if !instances_dir.exists() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(&instances_dir) {
        for entry in entries.flatten() {
            let old = entry.path().join(".minecraft");
            let new = entry.path().join("game");
            if old.exists() && !new.exists() {
                let _ = std::fs::rename(&old, &new);
            }
        }
    }
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        std::fs::create_dir_all(data_dir.join("instances"))
            .expect("Failed to create instances dir");
        migrate_game_dirs(&data_dir);
        std::fs::create_dir_all(data_dir.join("versions")).expect("Failed to create versions dir");
        std::fs::create_dir_all(data_dir.join("assets")).expect("Failed to create assets dir");
        std::fs::create_dir_all(data_dir.join("libraries"))
            .expect("Failed to create libraries dir");
        std::fs::create_dir_all(data_dir.join("java")).expect("Failed to create java dir");

        Self {
            http_client: reqwest::Client::builder()
                .user_agent("glowberry/0.1.0")
                .build()
                .expect("Failed to create HTTP client"),
            instances: Mutex::new(InstanceManager::new(data_dir.join("instances"))),
            auth: Mutex::new(AuthState::new()),
            data_dir,
        }
    }
}
