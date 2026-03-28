use serde::{Deserialize, Serialize};

use crate::error::LanternError;

// Azure AD application — register your own at portal.azure.com
// (Personal Microsoft accounts only, "Allow public client flows" enabled)
const CLIENT_ID: &str = "REPLACE_WITH_YOUR_AZURE_CLIENT_ID";
const MSA_DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MSA_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const MSA_SCOPE: &str = "XboxLive.signin offline_access";

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub minecraft_access_token: String,
    pub msa_refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct MsaTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct MsaErrorResponse {
    error: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MsaPollResponse {
    Success(MsaTokenResponse),
    Error(MsaErrorResponse),
}

/// Result of polling: either still pending, or completed with tokens + profile.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum LoginPollResult {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "complete")]
    Complete { profile: MinecraftProfile },
}

// ── Xbox / Minecraft token exchange internal types ─────────────────────

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

// ── Step 1: Request device code ────────────────────────────────────────

pub async fn request_device_code(
    client: &reqwest::Client,
) -> Result<DeviceCodeResponse, LanternError> {
    let resp = client
        .post(MSA_DEVICE_CODE_URL)
        .form(&[("client_id", CLIENT_ID), ("scope", MSA_SCOPE)])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(LanternError::Auth(format!(
            "Device code request failed ({status}): {body}"
        )));
    }

    let parsed: DeviceCodeResponse =
        serde_json::from_str(&body).map_err(|e| LanternError::Auth(e.to_string()))?;
    Ok(parsed)
}

// ── Step 2: Poll for MSA token ─────────────────────────────────────────

/// Poll Microsoft for token. Returns None if user hasn't completed login yet.
pub async fn poll_for_msa_token(
    client: &reqwest::Client,
    device_code: &str,
) -> Result<Option<MsaTokenResponse>, LanternError> {
    let resp = client
        .post(MSA_TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
        ])
        .send()
        .await?;

    let body = resp.text().await?;
    let parsed: MsaPollResponse =
        serde_json::from_str(&body).map_err(|e| LanternError::Auth(e.to_string()))?;

    match parsed {
        MsaPollResponse::Success(tokens) => Ok(Some(tokens)),
        MsaPollResponse::Error(err) => match err.error.as_str() {
            "authorization_pending" | "slow_down" => Ok(None),
            "authorization_declined" => Err(LanternError::Auth("Login was declined".into())),
            "expired_token" => Err(LanternError::Auth("Login code expired".into())),
            other => Err(LanternError::Auth(format!("Auth error: {other}"))),
        },
    }
}

// ── Step 3: Refresh MSA token ──────────────────────────────────────────

pub async fn refresh_msa_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<MsaTokenResponse, LanternError> {
    let resp = client
        .post(MSA_TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", MSA_SCOPE),
        ])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(LanternError::Auth(format!(
            "Failed to refresh token ({status}): {body}"
        )));
    }

    let parsed: MsaTokenResponse =
        serde_json::from_str(&body).map_err(|e| LanternError::Auth(e.to_string()))?;
    Ok(parsed)
}

// ── Step 4: MSA → XBL → XSTS → Minecraft token exchange ───────────────

async fn exchange_xbl_token(
    client: &reqwest::Client,
    msa_access_token: &str,
) -> Result<(String, String), LanternError> {
    // Azure AD v2.0 tokens require the "d=" prefix on the RpsTicket.
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={msa_access_token}")
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
        return Err(LanternError::Auth(format!(
            "Xbox Live auth failed ({status}): {text}"
        )));
    }

    let parsed: XblResponse =
        serde_json::from_str(&text).map_err(|e| LanternError::Auth(e.to_string()))?;

    let uhs = parsed
        .display_claims
        .xui
        .first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| LanternError::Auth("No user hash in XBL response".into()))?;

    Ok((parsed.token, uhs))
}

async fn exchange_xsts_token(
    client: &reqwest::Client,
    xbl_token: &str,
) -> Result<String, LanternError> {
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
        return Err(LanternError::Auth(format!(
            "XSTS auth failed ({status}): {text}"
        )));
    }

    let parsed: XstsResponse =
        serde_json::from_str(&text).map_err(|e| LanternError::Auth(e.to_string()))?;
    Ok(parsed.token)
}

async fn get_minecraft_token(
    client: &reqwest::Client,
    xsts_token: &str,
    user_hash: &str,
) -> Result<String, LanternError> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
    });

    let resp = client
        .post("https://api.minecraftservices.com/authentication/loginWithXbox")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(LanternError::Auth(format!(
            "Minecraft auth failed ({status}): {text}"
        )));
    }

    let parsed: MinecraftAuthResponse =
        serde_json::from_str(&text).map_err(|e| LanternError::Auth(e.to_string()))?;
    Ok(parsed.access_token)
}

pub async fn get_minecraft_profile(
    client: &reqwest::Client,
    mc_access_token: &str,
) -> Result<MinecraftProfile, LanternError> {
    let resp = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_access_token)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(LanternError::Auth(format!(
            "Failed to get Minecraft profile ({status}): {text}"
        )));
    }

    let parsed: MinecraftProfileResponse =
        serde_json::from_str(&text).map_err(|e| LanternError::Auth(e.to_string()))?;

    Ok(MinecraftProfile {
        id: parsed.id,
        name: parsed.name,
    })
}

/// Run the full token exchange: MSA access token → Minecraft access token + profile.
pub async fn full_token_exchange(
    client: &reqwest::Client,
    msa_access_token: &str,
    msa_refresh_token: &str,
) -> Result<(AuthTokens, MinecraftProfile), LanternError> {
    let (xbl_token, user_hash) = exchange_xbl_token(client, msa_access_token).await?;
    let xsts_token = exchange_xsts_token(client, &xbl_token).await?;
    let mc_token = get_minecraft_token(client, &xsts_token, &user_hash).await?;
    let profile = get_minecraft_profile(client, &mc_token).await?;

    let tokens = AuthTokens {
        minecraft_access_token: mc_token,
        msa_refresh_token: msa_refresh_token.to_string(),
    };

    Ok((tokens, profile))
}
