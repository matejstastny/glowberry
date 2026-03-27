use std::path::PathBuf;
use std::process::Command;

use crate::error::LanternError;

pub fn detect_java() -> Option<JavaInfo> {
    // Try JAVA_HOME first
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_path = PathBuf::from(&java_home).join("bin").join("java");
        if let Some(info) = probe_java(&java_path) {
            return Some(info);
        }
    }

    // Try system java
    if let Some(info) = probe_java(&PathBuf::from("java")) {
        return Some(info);
    }

    None
}

pub struct JavaInfo {
    pub path: PathBuf,
    pub version: String,
    pub major_version: u32,
}

fn probe_java(path: &PathBuf) -> Option<JavaInfo> {
    let output = Command::new(path).arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse version from output like: openjdk version "21.0.1" or java version "1.8.0_301"
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
    // Java 1.8 -> major 8, Java 21 -> major 21
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
        (Some(1), Some(minor)) if *minor >= 21 => 21, // 1.21+
        (Some(1), Some(minor)) if *minor >= 18 => 17, // 1.18-1.20.x
        (Some(1), Some(minor)) if *minor >= 17 => 16, // 1.17
        _ => 8,                                       // 1.16 and below
    }
}
