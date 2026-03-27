use std::io::Read;
use std::path::Path;

use crate::error::LanternError;
use crate::modrinth::types::MrpackIndex;

pub fn parse_mrpack(path: &Path) -> Result<MrpackIndex, LanternError> {
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
    locked_files: &std::collections::HashSet<String>,
) -> Result<Vec<String>, LanternError> {
    let file = std::fs::File::open(mrpack_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut extracted = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let raw_name = entry.name().to_string();

        // Process both overrides/ and client-overrides/ directories
        let relative_path = if let Some(rest) = raw_name.strip_prefix("client-overrides/") {
            rest.to_string()
        } else if let Some(rest) = raw_name.strip_prefix("overrides/") {
            rest.to_string()
        } else {
            continue;
        };

        if relative_path.is_empty() {
            continue;
        }

        // Skip locked files
        let is_locked = locked_files.contains(&relative_path)
            || locked_files
                .iter()
                .any(|l| l.ends_with('/') && relative_path.starts_with(l.as_str()));

        if is_locked {
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
