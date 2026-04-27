use std::path::Path;

use crate::error::GlowberryError;
use crate::minecraft::java::JavaInfo;

/// Download a JRE from Adoptium and extract it to data_dir/java/.
pub async fn download_java(
    client: &reqwest::Client,
    data_dir: &Path,
    major_version: u32,
) -> Result<JavaInfo, GlowberryError> {
    let os = adoptium_os();
    let arch = adoptium_arch();

    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{major_version}/ga/{os}/{arch}/jre/hotspot/normal/eclipse"
    );

    eprintln!("[java] Downloading JRE {major_version} from Adoptium ({os}/{arch})...");

    let response = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            GlowberryError::Java(format!(
                "Failed to download Java {major_version} from Adoptium: {e}"
            ))
        })?;

    let java_dir = data_dir.join("java");
    tokio::fs::create_dir_all(&java_dir).await?;

    // Adoptium serves .zip on Windows, .tar.gz on macOS/Linux.
    let archive_ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    let archive_path = java_dir.join(format!("jre-{major_version}-download.{archive_ext}"));
    let bytes = response.bytes().await?;
    tokio::fs::write(&archive_path, &bytes).await?;

    eprintln!("[java] Extracting JRE...");
    let extract_dir = java_dir.clone();
    let archive_path_clone = archive_path.clone();

    // Extract in a blocking task (zip/tar APIs are synchronous).
    let extracted_name = tokio::task::spawn_blocking(move || {
        if cfg!(target_os = "windows") {
            extract_zip(&archive_path_clone, &extract_dir)
        } else {
            extract_tar_gz(&archive_path_clone, &extract_dir)
        }
    })
    .await
    .map_err(|e| GlowberryError::Java(format!("Extract task failed: {e}")))??;

    // Clean up archive
    let _ = tokio::fs::remove_file(&archive_path).await;

    // Build the path to the java binary inside the extracted directory.
    // macOS .tar.gz: <name>/Contents/Home/bin/java
    // Windows .zip:  <name>/bin/java.exe
    // Linux .tar.gz: <name>/bin/java
    let extracted_dir = java_dir.join(&extracted_name);
    let java_bin = if cfg!(target_os = "macos") {
        extracted_dir
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java")
    } else if cfg!(target_os = "windows") {
        extracted_dir.join("bin").join("java.exe")
    } else {
        extracted_dir.join("bin").join("java")
    };

    if !java_bin.exists() {
        return Err(GlowberryError::Java(format!(
            "Java binary not found at {}",
            java_bin.display()
        )));
    }

    // Make java executable on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&java_bin, std::fs::Permissions::from_mode(0o755));
    }

    eprintln!(
        "[java] JRE {major_version} installed at {}",
        extracted_dir.display()
    );

    Ok(JavaInfo {
        path: java_bin,
        version: format!("{major_version}"),
        major_version,
    })
}

// ── Archive extraction ────────────────────────────────────────────────────────

fn extract_tar_gz(
    archive_path: &std::path::Path,
    extract_dir: &std::path::Path,
) -> Result<String, GlowberryError> {
    let file = std::fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(extract_dir)?;

    // Re-read to find the top-level directory name.
    let file2 = std::fs::File::open(archive_path)?;
    let decoder2 = flate2::read::GzDecoder::new(file2);
    let mut archive2 = tar::Archive::new(decoder2);

    let mut top_dir = String::new();
    for entry in archive2.entries()? {
        let entry = entry?;
        let path = entry.path()?;
        if let Some(first) = path.components().next() {
            top_dir = first.as_os_str().to_string_lossy().to_string();
            break;
        }
    }

    if top_dir.is_empty() {
        return Err(GlowberryError::Java(
            "Could not determine extracted directory name".into(),
        ));
    }

    Ok(top_dir)
}

/// Extract a zip archive (used on Windows for Adoptium JRE downloads).
fn extract_zip(
    archive_path: &std::path::Path,
    extract_dir: &std::path::Path,
) -> Result<String, GlowberryError> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut top_dir = String::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let raw_name = entry.name().to_string();

        // Capture the top-level directory from the very first entry.
        if top_dir.is_empty() {
            // Zip paths always use '/' as separator.
            let first = raw_name.split('/').next().unwrap_or(&raw_name);
            if !first.is_empty() {
                top_dir = first.to_string();
            }
        }

        // Construct an output path, rejecting any suspicious ".." components.
        let out_path = extract_dir.join(&raw_name);
        if !out_path.starts_with(extract_dir) {
            // Zip-slip guard: skip entries that would escape the target dir.
            continue;
        }

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }

    if top_dir.is_empty() {
        return Err(GlowberryError::Java(
            "Could not determine extracted directory name from zip".into(),
        ));
    }

    Ok(top_dir)
}

// ── Platform helpers ──────────────────────────────────────────────────────────

fn adoptium_os() -> &'static str {
    if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

fn adoptium_arch() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    }
}
