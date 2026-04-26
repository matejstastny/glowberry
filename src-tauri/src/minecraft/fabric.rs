use std::path::{Path, PathBuf};

use crate::error::GlowberryError;

const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

/// Download the Fabric loader profile JSON and save it to the versions directory.
/// Returns the path to the saved JSON file.
pub async fn install_fabric(
    client: &reqwest::Client,
    data_dir: &Path,
    mc_version: &str,
    loader_version: &str,
) -> Result<PathBuf, GlowberryError> {
    let version_id = format!("{mc_version}-fabric-{loader_version}");
    let version_dir = data_dir.join("versions").join(&version_id);
    let json_path = version_dir.join(format!("{version_id}.json"));

    // Already installed
    if json_path.exists() {
        eprintln!("[fabric] Profile already exists: {version_id}");
        return Ok(json_path);
    }

    let url =
        format!("{FABRIC_META_URL}/versions/loader/{mc_version}/{loader_version}/profile/json");

    eprintln!("[fabric] Downloading profile: {version_id}");

    let text = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            GlowberryError::Other(format!(
                "Failed to download Fabric profile for {mc_version}/{loader_version}: {e}"
            ))
        })?
        .text()
        .await?;

    // Validate that it parses
    let _: super::version::VersionJson = serde_json::from_str(&text)
        .map_err(|e| GlowberryError::Other(format!("Invalid Fabric profile JSON: {e}")))?;

    tokio::fs::create_dir_all(&version_dir).await?;
    tokio::fs::write(&json_path, &text).await?;

    eprintln!("[fabric] Installed: {version_id}");
    Ok(json_path)
}
