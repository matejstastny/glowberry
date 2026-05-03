use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::GlowberryError;

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

// -- Version manifest (top-level index of all MC versions) --

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub versions: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestEntry {
    pub id: String,
    pub url: String,
}

// -- Individual version JSON --

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub id: String,
    pub main_class: String,
    #[serde(default)]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default)]
    pub downloads: Option<VersionDownloads>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "type", default)]
    pub version_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDownloads {
    pub client: DownloadEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadEntry {
    pub sha1: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
}

// -- Libraries --

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    /// Feature conditions (e.g. is_demo_user, has_custom_resolution).
    /// Rules with features only match if all specified features are active.
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
}

// -- Arguments --

#[derive(Debug, Clone, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Plain(String),
    Conditional {
        rules: Vec<Rule>,
        value: SingleOrVec,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SingleOrVec {
    Single(String),
    Multiple(Vec<String>),
}

// -- Asset index --

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

// -- Functions --

pub async fn fetch_version_manifest(
    client: &reqwest::Client,
) -> Result<VersionManifest, GlowberryError> {
    let manifest = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<VersionManifest>()
        .await?;
    Ok(manifest)
}

pub async fn fetch_version_json(
    client: &reqwest::Client,
    data_dir: &Path,
    version_id: &str,
    url: &str,
) -> Result<VersionJson, GlowberryError> {
    let version_dir = data_dir.join("versions").join(version_id);
    let json_path = version_dir.join(format!("{version_id}.json"));

    // Use cached version if it exists
    if json_path.exists() {
        let data = tokio::fs::read_to_string(&json_path).await?;
        return Ok(serde_json::from_str(&data)?);
    }

    let json_text = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    tokio::fs::create_dir_all(&version_dir).await?;
    tokio::fs::write(&json_path, &json_text).await?;

    Ok(serde_json::from_str(&json_text)?)
}

/// Load a version JSON from a local path (e.g. Fabric profile JSON).
pub async fn load_version_json(path: &Path) -> Result<VersionJson, GlowberryError> {
    let data = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&data)?)
}

/// Build a deduplication key from a Maven coordinate, stripping only the version.
/// 3-part "group:artifact:version"            -> "group:artifact"
/// 4-part "group:artifact:version:classifier"  -> "group:artifact:classifier"
/// This keeps native JARs (e.g. lwjgl:natives-macos-arm64) distinct from base JARs.
fn maven_dedup_key(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() >= 4 {
        format!("{}:{}:{}", parts[0], parts[1], parts[3])
    } else if parts.len() >= 2 {
        format!("{}:{}", parts[0], parts[1])
    } else {
        name.to_string()
    }
}

/// Merge a child version JSON (e.g. Fabric) with its parent (vanilla).
/// The child overrides mainClass and adds its libraries.
/// Libraries are deduplicated by group:artifact — child versions win.
pub fn merge_version_json(child: VersionJson, parent: VersionJson) -> VersionJson {
    let mut merged = parent;
    merged.id = child.id;
    merged.main_class = child.main_class;

    // Deduplicate libraries: child versions take priority over parent
    let mut seen = std::collections::HashSet::new();
    let mut libs = Vec::new();

    // Add child libs first (they win on conflicts)
    for lib in child.libraries {
        let key = maven_dedup_key(&lib.name);
        seen.insert(key);
        libs.push(lib);
    }

    // Add parent libs only if not shadowed by child
    for lib in merged.libraries {
        let key = maven_dedup_key(&lib.name);
        if !seen.contains(&key) {
            seen.insert(key);
            libs.push(lib);
        }
    }

    merged.libraries = libs;

    // Merge arguments
    if let Some(child_args) = child.arguments {
        if let Some(ref mut parent_args) = merged.arguments {
            parent_args.game.extend(child_args.game);
            parent_args.jvm.extend(child_args.jvm);
        } else {
            merged.arguments = Some(child_args);
        }
    }

    merged
}

/// Returns the current OS name in Mojang's format.
pub fn current_os_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "osx"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

/// Check if a library/argument should be included based on its rules.
/// Rules with `features` conditions are rejected (we don't enable
/// is_demo_user, has_custom_resolution, quickPlay, etc.).
fn library_allowed(rules: &[Rule]) -> bool {
    let os = current_os_name();
    let mut allowed = false;

    for rule in rules {
        // Feature-gated rules: only match if all required features are false
        // (i.e. we never enable demo mode, custom resolution, quickPlay, etc.)
        if let Some(features) = &rule.features {
            if features.values().any(|&v| v) {
                // Rule requires a feature to be true, but we don't enable any
                continue;
            }
        }

        let os_matches = match &rule.os {
            Some(os_rule) => os_rule.name.as_deref().is_none_or(|name| name == os),
            None => true,
        };

        if os_matches {
            allowed = rule.action == "allow";
        }
    }

    allowed
}

/// Filter libraries to only those allowed on the current OS.
pub fn filter_libraries(libraries: &[Library]) -> Vec<&Library> {
    libraries
        .iter()
        .filter(|lib| match &lib.rules {
            Some(rules) => library_allowed(rules),
            None => true, // no rules = always included
        })
        .collect()
}

/// Get the artifact path for a library from its Maven coordinate.
/// e.g. "org.lwjgl:lwjgl:3.3.1" -> "org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1.jar"
pub fn maven_to_path(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    Some(format!(
        "{group}/{artifact}/{version}/{artifact}-{version}.jar"
    ))
}

/// Resolve argument template variables.
/// Replaces ${key} with values from the substitutions map.
fn substitute(template: &str, subs: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in subs {
        result = result.replace(&format!("${{{key}}}"), value);
    }
    result
}

/// Check if an argument's rules allow it on the current OS.
fn argument_allowed(rules: &[Rule]) -> bool {
    library_allowed(rules) // same logic
}

/// Resolve JVM arguments from a version JSON.
pub fn resolve_jvm_arguments(version: &VersionJson, subs: &HashMap<&str, String>) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(ref arguments) = version.arguments {
        for val in &arguments.jvm {
            collect_argument_value(val, subs, &mut args);
        }
    } else {
        // Legacy format (pre-1.13): use default JVM args
        args.extend(default_jvm_args(subs));
    }

    args
}

/// Resolve game arguments from a version JSON.
pub fn resolve_game_arguments(version: &VersionJson, subs: &HashMap<&str, String>) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(ref arguments) = version.arguments {
        for val in &arguments.game {
            collect_argument_value(val, subs, &mut args);
        }
    } else if let Some(ref mc_args) = version.minecraft_arguments {
        for token in mc_args.split_whitespace() {
            args.push(substitute(token, subs));
        }
    }

    args
}

fn collect_argument_value(
    val: &ArgumentValue,
    subs: &HashMap<&str, String>,
    out: &mut Vec<String>,
) {
    match val {
        ArgumentValue::Plain(s) => {
            out.push(substitute(s, subs));
        }
        ArgumentValue::Conditional { rules, value } => {
            if argument_allowed(rules) {
                match value {
                    SingleOrVec::Single(s) => out.push(substitute(s, subs)),
                    SingleOrVec::Multiple(v) => {
                        for s in v {
                            out.push(substitute(s, subs));
                        }
                    }
                }
            }
        }
    }
}

/// Default JVM arguments for legacy version JSONs (pre-1.13).
fn default_jvm_args(subs: &HashMap<&str, String>) -> Vec<String> {
    let templates = [
        "-Djava.library.path=${natives_directory}",
        "-Dminecraft.launcher.brand=${launcher_name}",
        "-Dminecraft.launcher.version=${launcher_version}",
        "-cp",
        "${classpath}",
    ];
    templates.iter().map(|t| substitute(t, subs)).collect()
}

/// On Linux aarch64, Mojang's version JSON only ships `natives-linux` (x86_64) JARs.
/// This swaps every LWJGL `natives-linux` entry for a `natives-linux-arm64` one
/// fetched from Maven Central, where LWJGL 3.3.x officially publishes arm64 builds.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn patch_lwjgl_for_linux_arm64(libraries: &mut Vec<Library>) {
    const MAVEN_CENTRAL: &str = "https://repo1.maven.org/maven2";

    let mut replacements: Vec<(usize, Library)> = Vec::new();

    for (i, lib) in libraries.iter().enumerate() {
        if !lib.name.starts_with("org.lwjgl:") || !lib.name.ends_with(":natives-linux") {
            continue;
        }
        let parts: Vec<&str> = lib.name.split(':').collect();
        if parts.len() != 4 {
            continue;
        }
        let group_path = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        let classifier = "natives-linux-arm64";
        let filename = format!("{artifact}-{version}-{classifier}.jar");
        let path = format!("{group_path}/{artifact}/{version}/{filename}");
        let url = format!("{MAVEN_CENTRAL}/{path}");

        replacements.push((
            i,
            Library {
                name: format!("{}:{}:{}:{}", parts[0], parts[1], parts[2], classifier),
                downloads: Some(LibraryDownloads {
                    artifact: Some(LibraryArtifact {
                        path,
                        sha1: String::new(),
                        size: 0,
                        url,
                    }),
                }),
                rules: Some(vec![Rule {
                    action: "allow".to_string(),
                    os: Some(OsRule {
                        name: Some("linux".to_string()),
                    }),
                    features: None,
                }]),
                url: None,
            },
        ));
    }

    for (i, replacement) in replacements.into_iter().rev() {
        libraries[i] = replacement;
    }
}

// -- Fetch helpers for assets --

pub async fn fetch_asset_index(
    client: &reqwest::Client,
    data_dir: &Path,
    asset_ref: &AssetIndexRef,
) -> Result<AssetIndex, GlowberryError> {
    let index_dir = data_dir.join("assets").join("indexes");
    let index_path = index_dir.join(format!("{}.json", asset_ref.id));

    if index_path.exists() {
        let data = tokio::fs::read_to_string(&index_path).await?;
        return Ok(serde_json::from_str(&data)?);
    }

    let text = client
        .get(&asset_ref.url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    tokio::fs::create_dir_all(&index_dir).await?;
    tokio::fs::write(&index_path, &text).await?;

    Ok(serde_json::from_str(&text)?)
}
