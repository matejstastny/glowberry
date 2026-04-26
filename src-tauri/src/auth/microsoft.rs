use serde::{Deserialize, Serialize};

use crate::error::GlowberryError;

const CLIENT_ID: &str = "00000000402b5328";
const MSA_AUTHORIZE_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const MSA_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const MSA_SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";
pub const REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";

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

#[derive(Debug, Deserialize)]
pub struct MsaTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

// Token exchange types --------------------------------------------------

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

pub fn build_auth_url() -> String {
    format!(
        "{MSA_AUTHORIZE_URL}?client_id={CLIENT_ID}\
         &response_type=code\
         &redirect_uri={REDIRECT_URI}\
         &scope={MSA_SCOPE}\
         &prompt=select_account"
    )
}

pub async fn exchange_auth_code(
    client: &reqwest::Client,
    code: &str,
) -> Result<MsaTokenResponse, GlowberryError> {
    let resp = client
        .post(MSA_TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", REDIRECT_URI),
            ("scope", MSA_SCOPE),
        ])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        eprintln!("[auth] MSA token exchange failed ({status}): {body}");
        return Err(GlowberryError::Auth(format!(
            "Microsoft token exchange failed ({status})"
        )));
    }

    serde_json::from_str(&body).map_err(|e| GlowberryError::Auth(e.to_string()))
}

pub async fn refresh_msa_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<MsaTokenResponse, GlowberryError> {
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
        return Err(GlowberryError::Auth(format!(
            "Failed to refresh token ({status})"
        )));
    }

    serde_json::from_str(&body).map_err(|e| GlowberryError::Auth(e.to_string()))
}

async fn exchange_xbl_token(
    client: &reqwest::Client,
    msa_access_token: &str,
    // live.com tokens: use as-is; OAuth2 device-code tokens: prepend "d="
    use_d_prefix: bool,
) -> Result<(String, String), GlowberryError> {
    let rps_ticket = if use_d_prefix {
        format!("d={msa_access_token}")
    } else {
        msa_access_token.to_string()
    };

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
/// `use_d_prefix`: false for live.com tokens, true for OAuth2 device-code tokens.
pub async fn full_token_exchange(
    client: &reqwest::Client,
    msa_access_token: &str,
    msa_refresh_token: &str,
    use_d_prefix: bool,
) -> Result<(AuthTokens, MinecraftProfile), GlowberryError> {
    eprintln!("[auth] XBL exchange...");
    let (xbl_token, user_hash) =
        exchange_xbl_token(client, msa_access_token, use_d_prefix).await?;
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
