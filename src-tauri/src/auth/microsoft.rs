use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::GlowberryError;

const CLIENT_ID: &str = "00000000402b5328";
const DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const DEVICE_CODE_SCOPE: &str = "XboxLive.signin offline_access";

// Types -----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub minecraft_access_token: String,
    pub msa_refresh_token: String,
}

/// Returned to the frontend so it can render the QR code and user code.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    /// Internal code used to poll for the token — not shown to the user.
    pub device_code: String,
}

#[derive(Debug, Deserialize)]
pub struct MsaTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// Internal types --------------------------------------------------------

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct PollResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XblResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XblDisplayClaims,
}

#[derive(Debug, Deserialize)]
struct XblDisplayClaims {
    xui: Vec<XblXui>,
}

#[derive(Debug, Deserialize)]
struct XblXui {
    uhs: String,
}

#[derive(Debug, Deserialize)]
struct XstsResponse {
    #[serde(rename = "Token")]
    token: String,
}

#[derive(Debug, Deserialize)]
struct MinecraftAuthResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct MinecraftProfileResponse {
    id: String,
    name: String,
}

// Public ----------------------------------------------------------------

pub async fn request_device_code(
    client: &reqwest::Client,
) -> Result<DeviceCodeInfo, GlowberryError> {
    let resp = client
        .post(DEVICE_CODE_URL)
        .form(&[("client_id", CLIENT_ID), ("scope", DEVICE_CODE_SCOPE)])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        eprintln!("[auth] device code request failed ({status}): {body}");
        return Err(GlowberryError::Auth(format!(
            "Failed to start device login ({status})"
        )));
    }

    let parsed: DeviceCodeResponse =
        serde_json::from_str(&body).map_err(|e| GlowberryError::Auth(e.to_string()))?;

    Ok(DeviceCodeInfo {
        user_code: parsed.user_code,
        verification_uri: parsed.verification_uri,
        expires_in: parsed.expires_in,
        interval: parsed.interval,
        device_code: parsed.device_code,
    })
}

pub enum PollOutcome {
    Pending,
    SlowDown,
    Tokens(MsaTokenResponse),
}

/// Poll once. Returns `Pending`/`SlowDown` when the user hasn't completed
/// auth yet, `Tokens` on success, or an error for expired/denied/other.
pub async fn poll_device_token(
    client: &reqwest::Client,
    device_code: &str,
) -> Result<PollOutcome, GlowberryError> {
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            (
                "grant_type",
                "urn:ietf:params:oauth:grant-type:device_code",
            ),
            ("device_code", device_code),
        ])
        .send()
        .await?;

    let body = resp.text().await?;
    let parsed: PollResponse =
        serde_json::from_str(&body).map_err(|e| GlowberryError::Auth(e.to_string()))?;

    if let (Some(access_token), Some(refresh_token)) = (parsed.access_token, parsed.refresh_token)
    {
        return Ok(PollOutcome::Tokens(MsaTokenResponse {
            access_token,
            refresh_token,
        }));
    }

    match parsed.error.as_deref() {
        Some("authorization_pending") => Ok(PollOutcome::Pending),
        Some("slow_down") => Ok(PollOutcome::SlowDown),
        Some("expired_token") => Err(GlowberryError::Auth("Device code expired".into())),
        Some("access_denied") => Err(GlowberryError::Auth("Sign-in was declined".into())),
        Some(other) => Err(GlowberryError::Auth(format!("Token poll error: {other}"))),
        None => Err(GlowberryError::Auth("Unexpected poll response".into())),
    }
}

pub async fn refresh_msa_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<MsaTokenResponse, GlowberryError> {
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", DEVICE_CODE_SCOPE),
        ])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(GlowberryError::Auth(format!(
            "Failed to refresh token ({status})"
        )));
    }

    serde_json::from_str(&body).map_err(|e| GlowberryError::Auth(e.to_string()))
}

async fn exchange_xbl_token(
    client: &reqwest::Client,
    msa_access_token: &str,
) -> Result<(String, String), GlowberryError> {
    // Azure AD / device-code tokens require the "d=" prefix.
    let rps_ticket = format!("d={msa_access_token}");

    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": rps_ticket
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let resp = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        eprintln!("[auth] XBL failed ({status}): {text}");
        return Err(GlowberryError::Auth(format!(
            "Xbox Live auth failed ({status})"
        )));
    }

    let parsed: XblResponse =
        serde_json::from_str(&text).map_err(|e| GlowberryError::Auth(e.to_string()))?;

    let uhs = parsed
        .display_claims
        .xui
        .first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| GlowberryError::Auth("No user hash in XBL response".into()))?;

    Ok((parsed.token, uhs))
}

async fn exchange_xsts_token(
    client: &reqwest::Client,
    xbl_token: &str,
) -> Result<String, GlowberryError> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let resp = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        eprintln!("[auth] XSTS failed ({status}): {text}");
        return Err(GlowberryError::Auth(format!("XSTS auth failed ({status})")));
    }

    let parsed: XstsResponse =
        serde_json::from_str(&text).map_err(|e| GlowberryError::Auth(e.to_string()))?;
    Ok(parsed.token)
}

async fn get_minecraft_token(
    client: &reqwest::Client,
    xsts_token: &str,
    user_hash: &str,
) -> Result<String, GlowberryError> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
    });

    let resp = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        eprintln!("[auth] MC auth failed ({status}): {text}");
        return Err(GlowberryError::Auth(format!(
            "Minecraft auth failed ({status})"
        )));
    }

    let parsed: MinecraftAuthResponse =
        serde_json::from_str(&text).map_err(|e| GlowberryError::Auth(e.to_string()))?;
    Ok(parsed.access_token)
}

pub async fn get_minecraft_profile(
    client: &reqwest::Client,
    mc_access_token: &str,
) -> Result<MinecraftProfile, GlowberryError> {
    let resp = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_access_token)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        eprintln!("[auth] MC profile failed ({status}): {text}");
        return Err(GlowberryError::Auth(format!(
            "Failed to get Minecraft profile ({status})"
        )));
    }

    let parsed: MinecraftProfileResponse =
        serde_json::from_str(&text).map_err(|e| GlowberryError::Auth(e.to_string()))?;

    Ok(MinecraftProfile {
        id: parsed.id,
        name: parsed.name,
    })
}

/// Full exchange chain: MSA access token → Minecraft access token + profile.
pub async fn full_token_exchange(
    client: &reqwest::Client,
    msa_access_token: &str,
    msa_refresh_token: &str,
) -> Result<(AuthTokens, MinecraftProfile), GlowberryError> {
    eprintln!("[auth] XBL exchange...");
    let (xbl_token, user_hash) = exchange_xbl_token(client, msa_access_token).await?;
    eprintln!("[auth] XSTS exchange...");
    let xsts_token = exchange_xsts_token(client, &xbl_token).await?;
    eprintln!("[auth] Minecraft token...");
    let mc_token = get_minecraft_token(client, &xsts_token, &user_hash).await?;
    eprintln!("[auth] Minecraft profile...");
    let profile = get_minecraft_profile(client, &mc_token).await?;

    let tokens = AuthTokens {
        minecraft_access_token: mc_token,
        msa_refresh_token: msa_refresh_token.to_string(),
    };

    Ok((tokens, profile))
}

pub fn poll_interval(base: u64, slow_downs: u32) -> Duration {
    Duration::from_secs(base + u64::from(slow_downs) * 5)
}
