use std::collections::HashMap;
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
    Failed,
}

fn emit_progress(app: &AppHandle, progress: &InstallProgress) {
    let _ = app.emit("install-progress", progress);
}

pub async fn install_modpack(
    app: AppHandle,
    state: &AppState,
    project_id: String,
    version_id: String,
) -> Result<Instance, GlowberryError> {
    let client = &state.http_client;
    let api = ModrinthApi::new(client.clone());

    // Fetch project + version info
    let project = api.get_project(&project_id).await?;
    let version = api.get_version(&version_id).await?;

    let primary_file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| GlowberryError::Other("No files in version".into()))?;

    // Stage: Downloading mrpack
    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Downloading,
            message: format!("Downloading {}...", primary_file.filename),
            current: 0,
            total: 1,
            bytes_downloaded: 0,
            bytes_total: primary_file.size,
            project_id: project_id.clone(),
        },
    );

    let temp_dir = state.data_dir.join("temp");
    tokio::fs::create_dir_all(&temp_dir).await?;
    let mrpack_path = temp_dir.join(&primary_file.filename);

    let dm = DownloadManager::new(client.clone());
    dm.download_file(&DownloadTask {
        url: primary_file.url.clone(),
        dest: mrpack_path.clone(),
        expected_size: primary_file.size,
        expected_hash: primary_file
            .hashes
            .sha512
            .as_ref()
            .map(|h| ExpectedHash::Sha512(h.clone()))
            .unwrap_or(ExpectedHash::None),
        file_name: primary_file.filename.clone(),
    })
    .await?;

    // Stage: Parsing
    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Parsing,
            message: "Reading modpack contents...".into(),
            current: 0,
            total: 0,
            bytes_downloaded: 0,
            bytes_total: 0,
            project_id: project_id.clone(),
        },
    );

    let mrpack_path_clone = mrpack_path.clone();
    let index =
        tokio::task::spawn_blocking(move || parse_mrpack(&mrpack_path_clone))
            .await
            .map_err(|e| GlowberryError::Other(format!("Parse task failed: {e}")))??;

    // Read dependencies
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

    // Create instance
    let instance_id = uuid::Uuid::new_v4().to_string();
    let minecraft_dir = state
        .data_dir
        .join("instances")
        .join(&instance_id)
        .join(".minecraft");
    tokio::fs::create_dir_all(&minecraft_dir).await?;

    // Stage: Installing mods
    // Filter files: skip server-only
    let mod_files: Vec<_> = index
        .files
        .iter()
        .filter(|f| {
            f.env
                .as_ref()
                .map_or(true, |env| env.client != EnvSupport::Unsupported)
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
            project_id: project_id.clone(),
        },
    );

    // Build download tasks
    let mut tasks = Vec::new();
    let mut file_hashes: HashMap<String, String> = HashMap::new();

    for mf in &mod_files {
        let url = mf
            .downloads
            .first()
            .ok_or_else(|| {
                GlowberryError::Other(format!("No download URL for {}", mf.path))
            })?
            .clone();

        let dest = minecraft_dir.join(&mf.path);

        file_hashes.insert(mf.path.clone(), mf.hashes.sha512.clone());

        tasks.push(DownloadTask {
            url,
            dest,
            expected_size: mf.file_size,
            expected_hash: ExpectedHash::Sha512(mf.hashes.sha512.clone()),
            file_name: mf
                .path
                .rsplit('/')
                .next()
                .unwrap_or(&mf.path)
                .to_string(),
        });
    }

    // Download all files concurrently
    let mut handles = Vec::new();
    for task in tasks {
        let dm = DownloadManager::new(client.clone());
        let completed = Arc::clone(&completed);
        let app_clone = app.clone();
        let project_id_clone = project_id.clone();

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
                    bytes_downloaded: 0, // simplified — we track by file count
                    bytes_total: total_bytes,
                    project_id: project_id_clone,
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

    // Stage: Extracting overrides
    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::ExtractingOverrides,
            message: "Extracting pack files...".into(),
            current: 0,
            total: 0,
            bytes_downloaded: 0,
            bytes_total: 0,
            project_id: project_id.clone(),
        },
    );

    let mc_dir_clone = minecraft_dir.clone();
    let mrpack_path_clone2 = mrpack_path.clone();
    let extracted = tokio::task::spawn_blocking(move || {
        extract_overrides(
            &mrpack_path_clone2,
            &mc_dir_clone,
            &std::collections::HashSet::new(),
        )
    })
    .await
    .map_err(|e| GlowberryError::Other(format!("Extract task failed: {e}")))??;

    // Add extracted override files to manifest
    for path in &extracted {
        // We don't have hashes for overrides, so use empty string
        file_hashes.insert(path.clone(), String::new());
    }

    // Stage: Installing loader
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
                project_id: project_id.clone(),
            },
        );

        super::fabric::install_fabric(client, &state.data_dir, &mc_version, fv).await?;
    }

    // Stage: Finalizing
    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Finalizing,
            message: "Saving modpack data...".into(),
            current: 0,
            total: 0,
            bytes_downloaded: 0,
            bytes_total: 0,
            project_id: project_id.clone(),
        },
    );

    // Write file_manifest.json
    let manifest = serde_json::to_string_pretty(&file_hashes)?;
    let instance_dir = state.data_dir.join("instances").join(&instance_id);
    tokio::fs::write(instance_dir.join("file_manifest.json"), manifest).await?;

    // Write last_mrpack_index.json (copy of the modrinth.index.json)
    let mrpack_path_clone3 = mrpack_path.clone();
    let index_json = tokio::task::spawn_blocking(move || -> Result<String, GlowberryError> {
        let file = std::fs::File::open(&mrpack_path_clone3)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut index_entry = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut index_entry, &mut contents)?;
        Ok(contents)
    })
    .await
    .map_err(|e| GlowberryError::Other(format!("Read index task failed: {e}")))??;

    tokio::fs::write(instance_dir.join("last_mrpack_index.json"), index_json).await?;

    // Clean up mrpack temp file
    let _ = tokio::fs::remove_file(&mrpack_path).await;

    // Create and save instance
    let instance = Instance {
        id: instance_id,
        name: project.title.clone(),
        minecraft_version: mc_version,
        loader,
        loader_version,
        modpack: Some(ModpackInfo {
            project_id: project.id.clone(),
            version_id: version.id.clone(),
            version_name: version.name.clone(),
            project_slug: project.slug.clone(),
            name: project.title.clone(),
            icon_url: project.icon_url.clone(),
        }),
        locked_files: std::collections::HashSet::new(),
        created_at: chrono::Utc::now(),
        last_played: None,
        jvm_args: Vec::new(),
        memory_mb: 4096,
    };

    {
        let instances = state.instances.lock().unwrap();
        instances.save(&instance)?;
    }

    // Stage: Complete
    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Complete,
            message: format!("{} installed!", project.title),
            current: total_files,
            total: total_files,
            bytes_downloaded: total_bytes,
            bytes_total: total_bytes,
            project_id: project_id.clone(),
        },
    );

    Ok(instance)
}

// ── GitHub-sourced install ───────────────────────────────────────────────────

/// Install (or update) the Starlight modpack from a direct mrpack download URL.
/// If `existing_instance_id` is provided, the existing instance directory is
/// reused (in-place update); otherwise a fresh UUID is generated.
pub async fn install_from_github(
    app: AppHandle,
    state: &AppState,
    mrpack_url: String,
    mrpack_name: String,
    mrpack_size: u64,
    version_tag: String,
    existing_instance_id: Option<String>,
) -> Result<Instance, GlowberryError> {
    const SLUG: &str = "starlightmodpack";
    let client = &state.http_client;

    // Fetch project metadata from Modrinth (icon URL, title)
    let api = ModrinthApi::new(client.clone());
    let (pack_title, icon_url) = match api.get_project(SLUG).await {
        Ok(p) => (p.title, p.icon_url),
        Err(_) => ("Starlight".to_string(), None),
    };

    // Stage: Downloading mrpack
    emit_progress(
        &app,
        &InstallProgress {
            stage: InstallStage::Downloading,
            message: format!("Downloading {mrpack_name}..."),
            current: 0,
            total: 1,
            bytes_downloaded: 0,
            bytes_total: mrpack_size,
            project_id: SLUG.to_string(),
        },
    );

    let temp_dir = state.data_dir.join("temp");
    tokio::fs::create_dir_all(&temp_dir).await?;
    let mrpack_path = temp_dir.join(&mrpack_name);

    let dm = DownloadManager::new(client.clone());
    dm.download_file(&DownloadTask {
        url: mrpack_url,
        dest: mrpack_path.clone(),
        expected_size: mrpack_size,
        expected_hash: ExpectedHash::None, // GitHub releases don't provide sha512 in the API
        file_name: mrpack_name.clone(),
    })
    .await?;

    // Stage: Parsing — from here the pipeline is identical to install_modpack
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

    let mrpack_path_clone = mrpack_path.clone();
    let index = tokio::task::spawn_blocking(move || parse_mrpack(&mrpack_path_clone))
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

    let instance_id = existing_instance_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let minecraft_dir = state
        .data_dir
        .join("instances")
        .join(&instance_id)
        .join(".minecraft");
    tokio::fs::create_dir_all(&minecraft_dir).await?;

    // Filter: skip server-only files
    let mod_files: Vec<_> = index
        .files
        .iter()
        .filter(|f| {
            f.env
                .as_ref()
                .map_or(true, |env| env.client != EnvSupport::Unsupported)
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

        let dest = minecraft_dir.join(&mf.path);
        file_hashes.insert(mf.path.clone(), mf.hashes.sha512.clone());

        tasks.push(DownloadTask {
            url,
            dest,
            expected_size: mf.file_size,
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

    let mc_dir_clone = minecraft_dir.clone();
    let mrpack_path_clone2 = mrpack_path.clone();
    let extracted = tokio::task::spawn_blocking(move || {
        extract_overrides(
            &mrpack_path_clone2,
            &mc_dir_clone,
            &std::collections::HashSet::new(),
        )
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
    let instance_dir = state.data_dir.join("instances").join(&instance_id);
    tokio::fs::write(instance_dir.join("file_manifest.json"), manifest).await?;

    let mrpack_path_clone3 = mrpack_path.clone();
    let index_json = tokio::task::spawn_blocking(move || -> Result<String, GlowberryError> {
        let file = std::fs::File::open(&mrpack_path_clone3)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut index_entry = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut index_entry, &mut contents)?;
        Ok(contents)
    })
    .await
    .map_err(|e| GlowberryError::Other(format!("Read index task failed: {e}")))??;

    tokio::fs::write(instance_dir.join("last_mrpack_index.json"), index_json).await?;
    let _ = tokio::fs::remove_file(&mrpack_path).await;

    // Preserve memory_mb from an existing instance if we have one
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
