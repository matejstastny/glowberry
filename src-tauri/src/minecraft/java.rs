use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::LanternError;

#[derive(Debug)]
pub struct JavaInfo {
    pub path: PathBuf,
    pub version: String,
    pub major_version: u32,
}

/// Detect all available Java installations.
pub fn detect_all_java(data_dir: &Path) -> Vec<JavaInfo> {
    let mut found = Vec::new();

    // Check JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_path = PathBuf::from(&java_home).join("bin").join("java");
        if let Some(info) = probe_java(&java_path) {
            found.push(info);
        }
    }

    // Check system java
    if let Some(info) = probe_java(&PathBuf::from("java")) {
        if !found.iter().any(|j: &JavaInfo| j.major_version == info.major_version) {
            found.push(info);
        }
    }

    // Check common macOS paths
    #[cfg(target_os = "macos")]
    {
        if let Ok(entries) = std::fs::read_dir("/Library/Java/JavaVirtualMachines") {
            for entry in entries.flatten() {
                let java_path = entry
                    .path()
                    .join("Contents")
                    .join("Home")
                    .join("bin")
                    .join("java");
                if java_path.exists() {
                    if let Some(info) = probe_java(&java_path) {
                        if !found.iter().any(|j| j.major_version == info.major_version) {
                            found.push(info);
                        }
                    }
                }
            }
        }
    }

    // Check common Linux paths
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/usr/lib/jvm") {
            for entry in entries.flatten() {
                let java_path = entry.path().join("bin").join("java");
                if java_path.exists() {
                    if let Some(info) = probe_java(&java_path) {
                        if !found.iter().any(|j| j.major_version == info.major_version) {
                            found.push(info);
                        }
                    }
                }
            }
        }
    }

    // Check Lantern-managed Java (Adoptium downloads)
    let java_dir = data_dir.join("java");
    if let Ok(entries) = std::fs::read_dir(&java_dir) {
        for entry in entries.flatten() {
            let java_path = if cfg!(target_os = "macos") {
                entry
                    .path()
                    .join("Contents")
                    .join("Home")
                    .join("bin")
                    .join("java")
            } else {
                entry.path().join("bin").join("java")
            };
            if java_path.exists() {
                if let Some(info) = probe_java(&java_path) {
                    if !found.iter().any(|j| j.major_version == info.major_version) {
                        found.push(info);
                    }
                }
            }
        }
    }

    found
}

/// Find a suitable Java for the given Minecraft version, downloading if needed.
pub async fn ensure_java(
    client: &reqwest::Client,
    data_dir: &Path,
    minecraft_version: &str,
) -> Result<JavaInfo, LanternError> {
    let required = required_java_version(minecraft_version);
    let all = detect_all_java(data_dir);

    // Prefer exact match, then any version >= required
    if let Some(info) = all.into_iter().find(|j| j.major_version == required) {
        return Ok(info);
    }

    eprintln!("[java] No Java {required} found, downloading from Adoptium...");
    super::adoptium::download_java(client, data_dir, required).await
}

fn probe_java(path: &PathBuf) -> Option<JavaInfo> {
    let output = Command::new(path).arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    let version_line = stderr.lines().next()?;
    let version = version_line.split('"').nth(1)?.to_string();

    let major = parse_major_version(&version)?;

    Some(JavaInfo {
        path: path.clone(),
        version,
        major_version: major,
    })
}

fn parse_major_version(version: &str) -> Option<u32> {
    let first_part = version.split('.').next()?;
    let major: u32 = first_part.parse().ok()?;
    if major == 1 {
        version.split('.').nth(1)?.parse().ok()
    } else {
        Some(major)
    }
}

pub fn required_java_version(minecraft_version: &str) -> u32 {
    let parts: Vec<u32> = minecraft_version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    match (parts.first(), parts.get(1)) {
        (Some(1), Some(minor)) if *minor >= 21 => 21,
        (Some(1), Some(minor)) if *minor >= 18 => 17,
        (Some(1), Some(minor)) if *minor >= 17 => 16,
        _ => 8,
    }
}
