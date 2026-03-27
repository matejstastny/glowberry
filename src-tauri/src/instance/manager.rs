use std::collections::HashSet;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::LanternError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,
    pub modpack: Option<ModpackInfo>,
    pub locked_files: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
    pub jvm_args: Vec<String>,
    pub memory_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
    Quilt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackInfo {
    pub project_id: String,
    pub version_id: String,
    pub version_name: String,
    pub project_slug: String,
    pub name: String,
    pub icon_url: Option<String>,
}

pub struct InstanceManager {
    instances_dir: PathBuf,
}

impl InstanceManager {
    pub fn new(instances_dir: PathBuf) -> Self {
        Self { instances_dir }
    }

    pub fn list(&self) -> Result<Vec<Instance>, LanternError> {
        let mut instances = Vec::new();

        if !self.instances_dir.exists() {
            return Ok(instances);
        }

        for entry in std::fs::read_dir(&self.instances_dir)? {
            let entry = entry?;
            let meta_path = entry.path().join("lantern_instance.json");
            if meta_path.exists() {
                let data = std::fs::read_to_string(&meta_path)?;
                let instance: Instance = serde_json::from_str(&data)?;
                instances.push(instance);
            }
        }

        instances.sort_by(|a, b| b.last_played.cmp(&a.last_played));
        Ok(instances)
    }

    pub fn get(&self, id: &str) -> Result<Instance, LanternError> {
        let meta_path = self.instances_dir.join(id).join("lantern_instance.json");
        if !meta_path.exists() {
            return Err(LanternError::Instance(format!("Instance not found: {id}")));
        }
        let data = std::fs::read_to_string(&meta_path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self, instance: &Instance) -> Result<(), LanternError> {
        let instance_dir = self.instances_dir.join(&instance.id);
        std::fs::create_dir_all(&instance_dir)?;
        let data = serde_json::to_string_pretty(instance)?;
        std::fs::write(instance_dir.join("lantern_instance.json"), data)?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), LanternError> {
        let instance_dir = self.instances_dir.join(id);
        if instance_dir.exists() {
            std::fs::remove_dir_all(&instance_dir)?;
        }
        Ok(())
    }

    pub fn instance_dir(&self, id: &str) -> PathBuf {
        self.instances_dir.join(id)
    }

    pub fn minecraft_dir(&self, id: &str) -> PathBuf {
        self.instances_dir.join(id).join(".minecraft")
    }
}
