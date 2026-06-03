use std::io::Read;
use std::path::Path;

use crate::error::GlowberryError;
use crate::modrinth::types::MrpackIndex;

const LOCKED: &[&str] = &[
    "saves/",
    "servers.dat",
    "cherishedworlds-favorites.dat",
    "journeymap",
    "xaero",
    "bluemap",
    "schematics/",
    "screenshots/",
];

pub fn is_path_locked(path: &str) -> bool {
    LOCKED.iter().any(|l| {
        if l.ends_with('/') {
            path.starts_with(l) || path.trim_end_matches('/') == l.trim_end_matches('/')
        } else {
            path == *l
        }
    })
}

pub fn locked_paths() -> std::collections::HashSet<String> {
    LOCKED.iter().map(|s| s.to_string()).collect()
}

pub fn parse_mrpack(path: &Path) -> Result<MrpackIndex, GlowberryError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut index_file = archive.by_name("modrinth.index.json")?;
    let mut contents = String::new();
    index_file.read_to_string(&mut contents)?;

    let index: MrpackIndex = serde_json::from_str(&contents)?;
    Ok(index)
}

pub fn extract_overrides(
    mrpack_path: &Path,
    target_dir: &Path,
) -> Result<Vec<String>, GlowberryError> {
    let file = std::fs::File::open(mrpack_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut extracted = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let raw_name = entry.name().to_string();

        let relative_path = if let Some(rest) = raw_name.strip_prefix("client-overrides/") {
            rest.to_string()
        } else if let Some(rest) = raw_name.strip_prefix("overrides/") {
            rest.to_string()
        } else {
            continue;
        };

        if relative_path.is_empty() || is_path_locked(&relative_path) {
            continue;
        }

        let target_path = target_dir.join(&relative_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&target_path)?;
            std::io::copy(&mut entry, &mut outfile)?;
            extracted.push(relative_path);
        }
    }

    Ok(extracted)
}
