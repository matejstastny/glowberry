use std::path::PathBuf;
use std::sync::Mutex;

use crate::auth::microsoft::{AuthTokens, DeviceCodeResponse, MinecraftProfile};
use crate::instance::manager::InstanceManager;

/// In-memory auth session: active device code flow + current login.
pub struct AuthState {
    /// Active device code flow (while user is logging in).
    pub pending_device_code: Option<DeviceCodeResponse>,
    /// Current logged-in profile + tokens.
    pub profile: Option<MinecraftProfile>,
    pub tokens: Option<AuthTokens>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            pending_device_code: None,
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

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        std::fs::create_dir_all(data_dir.join("instances"))
            .expect("Failed to create instances dir");

        Self {
            http_client: reqwest::Client::builder()
                .user_agent("lantern/0.1.0")
                .build()
                .expect("Failed to create HTTP client"),
            instances: Mutex::new(InstanceManager::new(data_dir.join("instances"))),
            auth: Mutex::new(AuthState::new()),
            data_dir,
        }
    }
}
