use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::download::manager::{DownloadManager, DownloadTask, ExpectedHash};
use crate::error::LanternError;
use crate::instance::manager::{Instance, ModLoader};
use crate::state::AppState;

use super::version::{
    fetch_asset_index, fetch_version_json, fetch_version_manifest, filter_libraries,
    load_version_json, maven_to_path, merge_version_json, resolve_game_arguments,
    resolve_jvm_arguments, AssetIndexRef, VersionJson,
};

#[derive(Clone, Serialize)]
struct GameStarted {
    instance_id: String,
}

#[derive(Clone, Serialize)]
struct GameLog {
    instance_id: String,
    line: String,
    stream: String,
}

#[derive(Clone, Serialize)]
struct GameExit {
    instance_id: String,
    exit_code: Option<i32>,
}

pub async fn launch_instance(
    app: AppHandle,
    state: &AppState,
    instance: &Instance,
    online: bool,
    auth_name: Option<String>,
    auth_uuid: Option<String>,
    auth_token: Option<String>,
) -> Result<(), LanternError> {
    let client = &state.http_client;
    let data_dir = &state.data_dir;

    // 1. Resolve Java
    eprintln!("[launch] Resolving Java for MC {}...", instance.minecraft_version);
    let java = super::java::ensure_java(client, data_dir, &instance.minecraft_version).await?;
    eprintln!("[launch] Using Java {} at {}", java.version, java.path.display());

    // 2. Fetch version manifest + version JSON
    let manifest = fetch_version_manifest(client).await?;
    let manifest_entry = manifest
        .versions
        .iter()
        .find(|v| v.id == instance.minecraft_version)
        .ok_or_else(|| {
            LanternError::Launch(format!(
                "Minecraft version {} not found in manifest",
                instance.minecraft_version
            ))
        })?;

    let vanilla_json =
        fetch_version_json(client, data_dir, &instance.minecraft_version, &manifest_entry.url)
            .await?;

    // 3. If using a mod loader, merge version JSONs
    let version_json = match (&instance.loader, &instance.loader_version) {
        (ModLoader::Fabric, Some(loader_ver)) => {
            let fabric_json_path = super::fabric::install_fabric(
                client,
                data_dir,
                &instance.minecraft_version,
                loader_ver,
            )
            .await?;

            let fabric_json = load_version_json(&fabric_json_path).await?;
            merge_version_json(fabric_json, vanilla_json)
        }
        _ => vanilla_json,
    };

    // 4. Download client JAR
    let client_jar = download_client_jar(client, data_dir, &instance.minecraft_version, &version_json).await?;

    // 5. Download libraries
    let library_paths = download_libraries(client, data_dir, &version_json).await?;

    // 6. Download assets
    if let Some(ref asset_ref) = version_json.asset_index {
        download_assets(client, data_dir, asset_ref).await?;
    }

    // 7. Build classpath
    let classpath = build_classpath(&library_paths, &client_jar);

    // 8. Build arguments
    let minecraft_dir = state
        .data_dir
        .join("instances")
        .join(&instance.id)
        .join(".minecraft");

    let natives_dir = data_dir
        .join("versions")
        .join(&instance.minecraft_version)
        .join("natives");
    tokio::fs::create_dir_all(&natives_dir).await?;

    let player_name = if online {
        auth_name.unwrap_or_else(|| "Player".into())
    } else {
        auth_name.unwrap_or_else(|| "Player".into())
    };

    let player_uuid = if online {
        auth_uuid.unwrap_or_else(|| "0".repeat(32))
    } else {
        // Generate offline UUID from username
        offline_uuid(&player_name)
    };

    let access_token = if online {
        auth_token.unwrap_or_default()
    } else {
        "0".into()
    };

    let asset_index_id = version_json
        .asset_index
        .as_ref()
        .map(|a| a.id.clone())
        .unwrap_or_else(|| instance.minecraft_version.clone());

    let mut subs: HashMap<&str, String> = HashMap::new();
    subs.insert("natives_directory", natives_dir.to_string_lossy().to_string());
    subs.insert("launcher_name", "Lantern".into());
    subs.insert("launcher_version", "0.1.0".into());
    subs.insert("classpath", classpath);
    subs.insert("auth_player_name", player_name);
    subs.insert("version_name", version_json.id.clone());
    subs.insert("game_directory", minecraft_dir.to_string_lossy().to_string());
    subs.insert(
        "assets_root",
        data_dir.join("assets").to_string_lossy().to_string(),
    );
    subs.insert("assets_index_name", asset_index_id);
    subs.insert("auth_uuid", player_uuid);
    subs.insert("auth_access_token", access_token);
    subs.insert(
        "user_type",
        if online { "msa" } else { "legacy" }.into(),
    );
    subs.insert("version_type", version_json.version_type.clone());
    subs.insert("classpath_separator", classpath_separator().into());
    subs.insert("library_directory", data_dir.join("libraries").to_string_lossy().to_string());

    let jvm_args = resolve_jvm_arguments(&version_json, &subs);
    let game_args = resolve_game_arguments(&version_json, &subs);

    // 9. Build memory args
    let mut memory_args = vec![
        format!("-Xmx{}m", instance.memory_mb),
        format!("-Xms{}m", instance.memory_mb / 2),
    ];
    memory_args.extend(instance.jvm_args.clone());

    // 10. Spawn process
    // Command: java [memory_args] [jvm_args] MainClass [game_args]
    eprintln!("[launch] Starting Minecraft...");
    eprintln!("[launch] Java: {}", java.path.display());
    eprintln!("[launch] Main class: {}", version_json.main_class);

    let mut cmd = Command::new(&java.path);
    cmd.args(&memory_args);
    cmd.args(&jvm_args);
    cmd.arg(&version_json.main_class);
    cmd.args(&game_args);
    cmd.current_dir(&minecraft_dir);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        LanternError::Launch(format!("Failed to start Minecraft: {e}"))
    })?;

    let instance_id = instance.id.clone();
    let _ = app.emit("game-started", GameStarted {
        instance_id: instance_id.clone(),
    });

    // 11. Stream logs in background
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_stdout = app.clone();
    let id_stdout = instance_id.clone();
    if let Some(stdout) = stdout {
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_stdout.emit(
                    "game-log",
                    GameLog {
                        instance_id: id_stdout.clone(),
                        line,
                        stream: "stdout".into(),
                    },
                );
            }
        });
    }

    let app_stderr = app.clone();
    let id_stderr = instance_id.clone();
    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_stderr.emit(
                    "game-log",
                    GameLog {
                        instance_id: id_stderr.clone(),
                        line,
                        stream: "stderr".into(),
                    },
                );
            }
        });
    }

    // 12. Wait for exit in background
    let app_exit = app.clone();
    let id_exit = instance_id;
    tokio::spawn(async move {
        let status = child.wait().await;
        let exit_code = status.ok().and_then(|s| s.code());
        eprintln!("[launch] Minecraft exited with code: {exit_code:?}");
        let _ = app_exit.emit(
            "game-exit",
            GameExit {
                instance_id: id_exit,
                exit_code,
            },
        );
    });

    Ok(())
}

async fn download_client_jar(
    client: &reqwest::Client,
    data_dir: &Path,
    version_id: &str,
    version_json: &VersionJson,
) -> Result<PathBuf, LanternError> {
    let jar_path = data_dir
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.jar"));

    if jar_path.exists() {
        return Ok(jar_path);
    }

    let downloads = version_json
        .downloads
        .as_ref()
        .ok_or_else(|| LanternError::Launch("Version JSON has no downloads section".into()))?;

    eprintln!("[launch] Downloading client JAR...");
    let dm = DownloadManager::new(client.clone());
    dm.download_file(&DownloadTask {
        url: downloads.client.url.clone(),
        dest: jar_path.clone(),
        expected_size: downloads.client.size,
        expected_hash: ExpectedHash::Sha1(downloads.client.sha1.clone()),
        file_name: format!("{version_id}.jar"),
    })
    .await?;

    Ok(jar_path)
}

async fn download_libraries(
    client: &reqwest::Client,
    data_dir: &Path,
    version_json: &VersionJson,
) -> Result<Vec<PathBuf>, LanternError> {
    let libraries = filter_libraries(&version_json.libraries);
    let libraries_dir = data_dir.join("libraries");

    let mut paths = Vec::new();
    let mut handles = Vec::new();

    for lib in &libraries {
        // Get download info from explicit downloads or construct from Maven coords
        let (url, path, sha1, size) = if let Some(ref dl) = lib.downloads {
            if let Some(ref artifact) = dl.artifact {
                (
                    artifact.url.clone(),
                    artifact.path.clone(),
                    Some(artifact.sha1.clone()),
                    artifact.size,
                )
            } else {
                continue; // no artifact to download
            }
        } else if let Some(ref base_url) = lib.url {
            // Fabric-style: url base + maven path
            if let Some(maven_path) = maven_to_path(&lib.name) {
                let url = format!("{}{}", base_url, maven_path);
                (url, maven_path, None, 0)
            } else {
                continue;
            }
        } else if let Some(maven_path) = maven_to_path(&lib.name) {
            // Default to Mojang's Maven repo
            let url = format!("https://libraries.minecraft.net/{maven_path}");
            (url, maven_path, None, 0)
        } else {
            continue;
        };

        let dest = libraries_dir.join(&path);
        paths.push(dest.clone());

        if dest.exists() {
            continue;
        }

        let dm = DownloadManager::new(client.clone());
        let hash = sha1
            .map(ExpectedHash::Sha1)
            .unwrap_or(ExpectedHash::None);

        handles.push(tokio::spawn(async move {
            dm.download_file(&DownloadTask {
                url,
                dest,
                expected_size: size,
                expected_hash: hash,
                file_name: path,
            })
            .await
        }));
    }

    if !handles.is_empty() {
        eprintln!("[launch] Downloading {} libraries...", handles.len());
    }

    for handle in handles {
        handle
            .await
            .map_err(|e| LanternError::Launch(format!("Library download panicked: {e}")))??;
    }

    Ok(paths)
}

async fn download_assets(
    client: &reqwest::Client,
    data_dir: &Path,
    asset_ref: &AssetIndexRef,
) -> Result<(), LanternError> {
    let index = fetch_asset_index(client, data_dir, asset_ref).await?;

    let objects_dir = data_dir.join("assets").join("objects");
    let mut handles = Vec::new();

    for (_name, obj) in &index.objects {
        let prefix = &obj.hash[..2];
        let dest = objects_dir.join(prefix).join(&obj.hash);

        if dest.exists() {
            continue;
        }

        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            prefix, obj.hash
        );
        let hash = obj.hash.clone();
        let size = obj.size;

        let dm = DownloadManager::new(client.clone());
        handles.push(tokio::spawn(async move {
            dm.download_file(&DownloadTask {
                url,
                dest,
                expected_size: size,
                expected_hash: ExpectedHash::Sha1(hash.clone()),
                file_name: hash,
            })
            .await
        }));
    }

    if !handles.is_empty() {
        eprintln!("[launch] Downloading {} assets...", handles.len());
    }

    for handle in handles {
        handle
            .await
            .map_err(|e| LanternError::Launch(format!("Asset download panicked: {e}")))??;
    }

    Ok(())
}

fn build_classpath(library_paths: &[PathBuf], client_jar: &Path) -> String {
    let sep = classpath_separator();
    let mut parts: Vec<String> = library_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    parts.push(client_jar.to_string_lossy().to_string());
    parts.join(sep)
}

fn classpath_separator() -> &'static str {
    if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }
}

fn offline_uuid(username: &str) -> String {
    // Minecraft offline UUIDs are v3 UUIDs based on "OfflinePlayer:" + username
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("OfflinePlayer:{username}").as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    // Take first 32 hex chars
    hash[..32].to_string()
}
