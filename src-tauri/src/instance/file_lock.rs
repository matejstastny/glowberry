use serde::Serialize;
use std::path::Path;

use crate::error::GlowberryError;
use crate::instance::manager::Instance;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub is_locked: bool,
}

pub fn list_files(
    minecraft_dir: &Path,
    instance: &Instance,
    search: Option<&str>,
) -> Result<Vec<FileEntry>, GlowberryError> {
    let mut entries = Vec::new();

    if !minecraft_dir.exists() {
        return Ok(entries);
    }

    collect_files(
        minecraft_dir,
        minecraft_dir,
        &instance.locked_files,
        &mut entries,
    )?;

    if let Some(query) = search {
        let query = query.to_lowercase();
        entries.retain(|e| {
            e.path.to_lowercase().contains(&query) || e.name.to_lowercase().contains(&query)
        });
    }

    entries.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then(a.name.cmp(&b.name))
    });

    Ok(entries)
}

fn collect_files(
    base: &Path,
    dir: &Path,
    locked: &std::collections::HashSet<String>,
    entries: &mut Vec<FileEntry>,
) -> Result<(), GlowberryError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let relative_str = relative.to_string_lossy().to_string();
        let metadata = entry.metadata()?;

        let is_locked = locked.contains(&relative_str)
            || locked
                .iter()
                .any(|l| l.ends_with('/') && relative_str.starts_with(l));

        entries.push(FileEntry {
            path: relative_str.clone(),
            name: entry.file_name().to_string_lossy().to_string(),
            is_directory: metadata.is_dir(),
            size: metadata.len(),
            is_locked,
        });

        if metadata.is_dir() {
            collect_files(base, &path, locked, entries)?;
        }
    }
    Ok(())
}

pub fn is_file_locked(instance: &Instance, path: &str) -> bool {
    instance.locked_files.contains(path)
        || instance
            .locked_files
            .iter()
            .any(|l| l.ends_with('/') && path.starts_with(l.as_str()))
}
