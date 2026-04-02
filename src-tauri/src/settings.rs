use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Override for the game data directory. When set, all game data
    /// (instances, libraries, assets, etc.) is stored here instead of
    /// the platform default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { data_dir: None }
    }
}

impl Settings {
    /// Load settings from the config directory. Returns defaults if the
    /// file doesn't exist or can't be parsed.
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join(SETTINGS_FILE);
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save settings to the config directory.
    pub fn save(&self, config_dir: &Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(config_dir)?;
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(config_dir.join(SETTINGS_FILE), data)
    }

    /// Resolve the effective data directory: the override if set,
    /// otherwise the provided default.
    pub fn resolve_data_dir(&self, default: &Path) -> PathBuf {
        self.data_dir
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| default.to_path_buf())
    }
}
