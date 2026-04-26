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

    let archive_path = java_dir.join(format!("jre-{major_version}-download.tar.gz"));
    let bytes = response.bytes().await?;
    tokio::fs::write(&archive_path, &bytes).await?;

    eprintln!("[java] Extracting JRE...");
    let extract_dir = java_dir.clone();
    let archive_path_clone = archive_path.clone();

    // Extract in a blocking task since tar/flate2 are synchronous
    let extracted_name = tokio::task::spawn_blocking(move || {
        extract_tar_gz(&archive_path_clone, &extract_dir)
    })
    .await
    .map_err(|e| GlowberryError::Java(format!("Extract task failed: {e}")))??;

    // Clean up archive
    let _ = tokio::fs::remove_file(&archive_path).await;

    // Find the java binary in the extracted directory
    let extracted_dir = java_dir.join(&extracted_name);
    let java_bin = if cfg!(target_os = "macos") {
        extracted_dir
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java")
    } else {
        extracted_dir.join("bin").join("java")
    };

    if !java_bin.exists() {
        return Err(GlowberryError::Java(format!(
            "Java binary not found at {}",
            java_bin.display()
        )));
    }

    // Make java executable on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&java_bin, std::fs::Permissions::from_mode(0o755));
    }

    eprintln!("[java] JRE {major_version} installed at {}", extracted_dir.display());

    Ok(JavaInfo {
        path: java_bin,
        version: format!("{major_version}"),
        major_version,
    })
}

fn extract_tar_gz(
    archive_path: &std::path::Path,
    extract_dir: &std::path::Path,
) -> Result<String, GlowberryError> {
    let file = std::fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    // Find the top-level directory name from the first entry
    let mut top_dir = String::new();

    archive.unpack(extract_dir)?;

    // Re-read to find the extracted directory name
    let file = std::fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries()? {
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
