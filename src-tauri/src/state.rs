use std::path::PathBuf;
use std::sync::Mutex;

use crate::instance::manager::InstanceManager;

pub struct AppState {
    pub http_client: reqwest::Client,
    pub instances: Mutex<InstanceManager>,
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
            data_dir,
        }
    }
}
