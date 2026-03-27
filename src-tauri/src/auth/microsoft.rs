use serde::{Deserialize, Serialize};

use crate::error::LanternError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

// Microsoft OAuth client ID - use a public one or register your own at Azure
const CLIENT_ID: &str = "00000000402b5328"; // Minecraft launcher client ID

pub async fn request_device_code(
    client: &reqwest::Client,
) -> Result<DeviceCodeResponse, LanternError> {
    let resp = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", CLIENT_ID),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<DeviceCodeResponse>()
        .await?;
    Ok(resp)
}

// The full auth chain (Microsoft -> XBL -> XSTS -> Minecraft) will be implemented
// when we build the auth feature. The types and device code request are the foundation.
