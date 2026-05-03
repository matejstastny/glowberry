use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::download::manager::{DownloadManager, DownloadTask, ExpectedHash};
use crate::error::GlowberryError;
use crate::instance::manager::{Instance, ModLoader, ModpackInfo};
use crate::modrinth::api::ModrinthApi;
use crate::modrinth::mrpack::{extract_overrides, parse_mrpack};
use crate::modrinth::types::EnvSupport;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub message: String,
    pub current: u32,
    pub total: u32,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStage {
    Downloading,
    Parsing,
    InstallingMods,
    ExtractingOverrides,
    InstallingLoader,
    Finalizing,
    Complete,
}

fn emit_progress(app: &AppHandle, progress: &InstallProgress) {
    let _ = app.emit("install-progress", progress);
}

/// Clean game dir, keeping saves/ and servers.dat intact.
async fn clean_game_dir_preserve_persistent(game_dir: &Path) -> Result<(), GlowberryError> {
    if !game_dir.exists() {
        return Ok(());
    }
    let mut entries = tokio::fs::read_dir(game_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "saves" || name_str == "servers.dat" {
            continue;
        }
        let path = entry.path();
        if entry.file_type().await?.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            tokio::fs::remove_file(path).await?;
        }
    }
    Ok(())
}

/// Extract each top-level *.mrpack from a zip into presets_dir.
/// Returns the sorted list of preset names (file stems).
fn extract_preset_mrpacks(zip_path: &Path, presets_dir: &Path) -> Result<Vec<String>, GlowberryError> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut names = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();

        if entry_name.ends_with(".mrpack") && !entry_name.contains('/') {
            let dest = presets_dir.join(&entry_name);
            let mut outfile = std::fs::File::create(&dest)?;
            std::io::copy(&mut entry, &mut outfile)?;

            if let Some(stem) = Path::new(&entry_name).file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }

    names.sort();
    Ok(names)
}

/// Install (or update) the Starlight modpack from a GitHub release asset.
/// The asset is either a presets zip (*-client-presets.zip) or a plain mrpack
/// (*-client.mrpack) for older releases.
pub async fn install_from_github(
    app: AppHandle,
    state: &AppState,
    asset_url: String,
    asset_name: String,
    asset_size: u64,
    version_tag: String,
    existing_instance_id: Option<String>,
    existing_active_preset: Option<String>,
) -> Result<Instance, GlowberryError> {
    const SLUG: &str = "starlightmodpack";
    let client = &state.http_client;

    let api = ModrinthApi::new(client.clone());
    let (pack_title, icon_url) = match api.get_project(SLUG).await {
        Ok(p) => (p.title, p.icon_url),
        Err(_) => ("Starlight".to_string(), None),
    };

    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Downloading,
            message: format!("Downloading {asset_name}..."),
            current: 0,
            total: 1,
            bytes_downloaded: 0,
            bytes_total: asset_size,
            project_id: SLUG.to_string(),
        },
    );

    let temp_dir = state.data_dir.join("temp");
    tokio::fs::create_dir_all(&temp_dir).await?;
    let asset_path = temp_dir.join(&asset_name);

    DownloadManager::new(client.clone())
        .download_file(&DownloadTask {
            url: asset_url,
            dest: asset_path.clone(),
            expected_hash: ExpectedHash::None,
            file_name: asset_name.clone(),
        })
        .await?;

    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Parsing,
            message: "Reading modpack contents...".into(),
            current: 0,
            total: 0,
            bytes_downloaded: 0,
            bytes_total: 0,
            project_id: SLUG.to_string(),
        },
    );

    let is_update = existing_instance_id.is_some();
    let instance_id = existing_instance_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let instance_dir = state.data_dir.join("instances").join(&instance_id);
    let game_dir = instance_dir.join("game");
    let presets_dir = instance_dir.join("presets");

    tokio::fs::create_dir_all(&game_dir).await?;

    // Determine active preset and extract mrpacks from zip (or treat plain mrpack as "default")
    let (active_preset, mrpack_for_parsing) = if asset_name.ends_with("-client-presets.zip") {
        // Clear old presets and extract fresh ones
        if presets_dir.exists() {
            tokio::fs::remove_dir_all(&presets_dir).await?;
        }
        tokio::fs::create_dir_all(&presets_dir).await?;

        let asset_path_clone = asset_path.clone();
        let presets_dir_clone = presets_dir.clone();
        let preset_names = tokio::task::spawn_blocking(move || {
            extract_preset_mrpacks(&asset_path_clone, &presets_dir_clone)
        })
        .await
        .map_err(|e| GlowberryError::Other(format!("Preset extraction failed: {e}")))??;

        if preset_names.is_empty() {
            return Err(GlowberryError::Other(
                "No presets found in the release archive".into(),
            ));
        }

        // Restore previous preset if it still exists; otherwise fall back to first and notify.
        let active = match &existing_active_preset {
            Some(prev) if preset_names.contains(prev) => prev.clone(),
            Some(prev) => {
                let fallback = preset_names[0].clone();
                let _ = app.emit(
                    "preset-fallback",
                    serde_json::json!({ "requested": prev, "applied": fallback }),
                );
                fallback
            }
            None => preset_names[0].clone(),
        };

        let mrpack = presets_dir.join(format!("{active}.mrpack"));
        (Some(active), mrpack)
    } else {
        // Plain mrpack (legacy / no-preset release)
        (None, asset_path.clone())
    };

    let mrpack_clone = mrpack_for_parsing.clone();
    let index = tokio::task::spawn_blocking(move || parse_mrpack(&mrpack_clone))
        .await
        .map_err(|e| GlowberryError::Other(format!("Parse task failed: {e}")))??;

    let mc_version = index
        .dependencies
        .get("minecraft")
        .cloned()
        .ok_or_else(|| GlowberryError::Other("Modpack has no minecraft dependency".into()))?;

    let fabric_version = index.dependencies.get("fabric-loader").cloned();
    let quilt_version = index.dependencies.get("quilt-loader").cloned();
    let forge_version = index.dependencies.get("forge").cloned();
    let neoforge_version = index.dependencies.get("neoforge").cloned();

    let (loader, loader_version) = if let Some(v) = &fabric_version {
        (ModLoader::Fabric, Some(v.clone()))
    } else if let Some(v) = &quilt_version {
        (ModLoader::Quilt, Some(v.clone()))
    } else if let Some(v) = &forge_version {
        (ModLoader::Forge, Some(v.clone()))
    } else if let Some(v) = &neoforge_version {
        (ModLoader::NeoForge, Some(v.clone()))
    } else {
        (ModLoader::Vanilla, None)
    };

    if is_update {
        emit_progress(
            &app,
            &InstallProgress {
                stage: InstallStage::Finalizing,
                message: "Cleaning old files (keeping worlds and server list)...".into(),
                current: 0,
                total: 0,
                bytes_downloaded: 0,
                bytes_total: 0,
                project_id: SLUG.to_string(),
            },
        );
        clean_game_dir_preserve_persistent(&game_dir).await?;
    }

    let mod_files: Vec<_> = index
        .files
        .iter()
        .filter(|f| {
            f.env
                .as_ref()
                .is_none_or(|env| env.client != EnvSupport::Unsupported)
        })
        .collect();

    let total_files = mod_files.len() as u32;
    let total_bytes: u64 = mod_files.iter().map(|f| f.file_size).sum();
    let completed = Arc::new(AtomicU32::new(0));

    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::InstallingMods,
            message: format!("Downloading files (0 of {total_files})..."),
            current: 0,
            total: total_files,
            bytes_downloaded: 0,
            bytes_total: total_bytes,
            project_id: SLUG.to_string(),
        },
    );

    let mut tasks = Vec::new();
    let mut file_hashes: HashMap<String, String> = HashMap::new();

    for mf in &mod_files {
        let url = mf
            .downloads
            .first()
            .ok_or_else(|| GlowberryError::Other(format!("No download URL for {}", mf.path)))?
            .clone();
        let dest = game_dir.join(&mf.path);
        file_hashes.insert(mf.path.clone(), mf.hashes.sha512.clone());
        tasks.push(DownloadTask {
            url,
            dest,
            expected_hash: ExpectedHash::Sha512(mf.hashes.sha512.clone()),
            file_name: mf.path.rsplit('/').next().unwrap_or(&mf.path).to_string(),
        });
    }

    let mut handles = Vec::new();
    for task in tasks {
        let dm = DownloadManager::new(client.clone());
        let completed = Arc::clone(&completed);
        let app_clone = app.clone();
        handles.push(tokio::spawn(async move {
            dm.download_file(&task).await?;
            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
            emit_progress(
                &app_clone,
                &InstallProgress {
                    stage: InstallStage::InstallingMods,
                    message: format!("Downloading files ({done} of {total_files})..."),
                    current: done,
                    total: total_files,
                    bytes_downloaded: 0,
                    bytes_total: total_bytes,
                    project_id: SLUG.to_string(),
                },
            );
            Ok::<(), GlowberryError>(())
        }));
    }
    for handle in handles {
        handle
            .await
            .map_err(|e| GlowberryError::Other(format!("Download task panicked: {e}")))??;
    }

    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::ExtractingOverrides,
            message: "Extracting pack files...".into(),
            current: 0,
            total: 0,
            bytes_downloaded: 0,
            bytes_total: 0,
            project_id: SLUG.to_string(),
        },
    );

    let game_dir_clone = game_dir.clone();
    let mrpack_clone2 = mrpack_for_parsing.clone();
    let mut locked = std::collections::HashSet::new();
    locked.insert("saves/".to_string());
    locked.insert("servers.dat".to_string());
    let extracted = tokio::task::spawn_blocking(move || {
        extract_overrides(&mrpack_clone2, &game_dir_clone, &locked)
    })
    .await
    .map_err(|e| GlowberryError::Other(format!("Extract task failed: {e}")))??;

    for path in &extracted {
        file_hashes.insert(path.clone(), String::new());
    }

    if let Some(ref fv) = fabric_version {
        emit_progress(
            &app,
            &InstallProgress {
                stage: InstallStage::InstallingLoader,
                message: format!("Installing Fabric {fv}..."),
                current: 0,
                total: 1,
                bytes_downloaded: 0,
                bytes_total: 0,
                project_id: SLUG.to_string(),
            },
        );
        super::fabric::install_fabric(client, &state.data_dir, &mc_version, fv).await?;
    }

    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Finalizing,
            message: "Saving modpack data...".into(),
            current: 0,
            total: 0,
            bytes_downloaded: 0,
            bytes_total: 0,
            project_id: SLUG.to_string(),
        },
    );

    let manifest = serde_json::to_string_pretty(&file_hashes)?;
    tokio::fs::write(instance_dir.join("file_manifest.json"), manifest).await?;

    let mrpack_clone3 = mrpack_for_parsing.clone();
    let index_json = tokio::task::spawn_blocking(move || -> Result<String, GlowberryError> {
        let file = std::fs::File::open(&mrpack_clone3)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut entry = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut entry, &mut contents)?;
        Ok(contents)
    })
    .await
    .map_err(|e| GlowberryError::Other(format!("Read index task failed: {e}")))??;

    tokio::fs::write(instance_dir.join("last_mrpack_index.json"), index_json).await?;
    let _ = tokio::fs::remove_file(&asset_path).await;

    let memory_mb = {
        let instances = state.instances.lock().unwrap();
        instances
            .get(&instance_id)
            .ok()
            .map(|i| i.memory_mb)
            .unwrap_or(4096)
    };

    let instance = Instance {
        id: instance_id,
        name: pack_title.clone(),
        minecraft_version: mc_version,
        loader,
        loader_version,
        modpack: Some(ModpackInfo {
            project_id: SLUG.to_string(),
            version_id: version_tag.clone(),
            version_name: version_tag,
            project_slug: SLUG.to_string(),
            name: pack_title,
            icon_url,
        }),
        locked_files: std::collections::HashSet::new(),
        created_at: chrono::Utc::now(),
        last_played: None,
        jvm_args: Vec::new(),
        memory_mb,
        active_preset,
    };

    {
        let instances = state.instances.lock().unwrap();
        instances.save(&instance)?;
    }

    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Complete,
            message: format!("{} installed!", instance.name),
            current: total_files,
            total: total_files,
            bytes_downloaded: total_bytes,
            bytes_total: total_bytes,
            project_id: SLUG.to_string(),
        },
    );

    Ok(instance)
}
