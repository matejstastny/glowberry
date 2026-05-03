use serde::Serialize;
use tauri::State;

use crate::error::GlowberryError;
use crate::state::AppState;

const GITHUB_REPO: &str = "matejstastny/starlight";

#[derive(Debug, Clone, Serialize)]
pub struct GithubRelease {
    pub tag: String,
    pub asset_url: String,
    pub asset_name: String,
    pub asset_size: u64,
}

/// Fetch the latest GitHub release for the Starlight modpack.
/// Returns `None` if there are no releases yet or no presets zip asset is attached.
#[tauri::command]
pub async fn check_starlight_update(
    state: State<'_, AppState>,
) -> Result<Option<GithubRelease>, GlowberryError> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");

    let resp = state
        .http_client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    if resp.status().as_u16() == 404 {
        return Ok(None);
    }

    let body: serde_json::Value = resp
        .error_for_status()
        .map_err(|e| GlowberryError::Other(format!("GitHub API error: {e}")))?
        .json()
        .await?;

    let tag = body["tag_name"].as_str().unwrap_or("").to_string();
    if tag.is_empty() {
        return Ok(None);
    }

    let assets = match body["assets"].as_array() {
        Some(a) => a,
        None => return Ok(None),
    };

    // Prefer a presets zip; fall back to a plain client mrpack for older releases.
    let asset = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .is_some_and(|n| n.ends_with("-client-presets.zip"))
        })
        .or_else(|| {
            assets.iter().find(|a| {
                a["name"]
                    .as_str()
                    .is_some_and(|n| n.ends_with("-client.mrpack"))
            })
        });

    let asset = match asset {
        Some(a) => a,
        None => return Ok(None),
    };

    let asset_url = asset["browser_download_url"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let asset_name = asset["name"].as_str().unwrap_or("").to_string();
    let asset_size = asset["size"].as_u64().unwrap_or(0);

    if asset_url.is_empty() {
        return Ok(None);
    }

    Ok(Some(GithubRelease {
        tag,
        asset_url,
        asset_name,
        asset_size,
    }))
}
