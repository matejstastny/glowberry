use serde::Serialize;
use tauri::State;

use crate::auth::keychain;
use crate::auth::microsoft::{self, LoginPollResult, MinecraftProfile};
use crate::error::LanternError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
}

/// Begin the Microsoft device code login flow.
/// Returns the code the user needs to enter at the verification URL.
#[tauri::command]
pub async fn start_login(state: State<'_, AppState>) -> Result<DeviceCodeInfo, LanternError> {
    let resp = microsoft::request_device_code(&state.http_client).await?;

    let info = DeviceCodeInfo {
        user_code: resp.user_code.clone(),
        verification_uri: resp.verification_uri.clone(),
    };

    {
        let mut auth = state.auth.lock().unwrap();
        auth.pending_device_code = Some(resp);
    }

    Ok(info)
}

/// Poll to check if the user has completed the login flow.
/// Call this on an interval (every ~5s) after start_login.
#[tauri::command]
pub async fn check_login_status(
    state: State<'_, AppState>,
) -> Result<LoginPollResult, LanternError> {
    let device_code = {
        let auth = state.auth.lock().unwrap();
        auth.pending_device_code
            .as_ref()
            .map(|dc| dc.device_code.clone())
    };

    let device_code = device_code
        .ok_or_else(|| LanternError::Auth("No login flow in progress".into()))?;

    let msa_result = microsoft::poll_for_msa_token(&state.http_client, &device_code).await?;

    let msa_tokens = match msa_result {
        None => return Ok(LoginPollResult::Pending),
        Some(tokens) => tokens,
    };

    // Full exchange: MSA → XBL → XSTS → Minecraft
    let (auth_tokens, profile) = microsoft::full_token_exchange(
        &state.http_client,
        &msa_tokens.access_token,
        &msa_tokens.refresh_token,
    )
    .await?;

    // Save refresh token to OS keychain
    keychain::save_refresh_token(&auth_tokens.msa_refresh_token)?;

    // Update in-memory state
    {
        let mut auth = state.auth.lock().unwrap();
        auth.pending_device_code = None;
        auth.profile = Some(profile.clone());
        auth.tokens = Some(auth_tokens);
    }

    Ok(LoginPollResult::Complete { profile })
}

/// Get the current auth status (logged in profile or null).
#[tauri::command]
pub fn get_auth_status(state: State<'_, AppState>) -> Option<MinecraftProfile> {
    let auth = state.auth.lock().unwrap();
    auth.profile.clone()
}

/// Try to restore a previous session using the stored refresh token.
/// Call this once on app startup.
#[tauri::command]
pub async fn try_restore_session(
    state: State<'_, AppState>,
) -> Result<Option<MinecraftProfile>, LanternError> {
    let refresh_token = match keychain::load_refresh_token()? {
        Some(t) => t,
        None => return Ok(None),
    };

    let msa = match microsoft::refresh_msa_token(&state.http_client, &refresh_token).await {
        Ok(tokens) => tokens,
        Err(_) => {
            // Refresh token expired or invalid — clear it
            let _ = keychain::delete_refresh_token();
            return Ok(None);
        }
    };

    let (auth_tokens, profile) = match microsoft::full_token_exchange(
        &state.http_client,
        &msa.access_token,
        &msa.refresh_token,
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            let _ = keychain::delete_refresh_token();
            return Ok(None);
        }
    };

    keychain::save_refresh_token(&auth_tokens.msa_refresh_token)?;

    {
        let mut auth = state.auth.lock().unwrap();
        auth.profile = Some(profile.clone());
        auth.tokens = Some(auth_tokens);
    }

    Ok(Some(profile))
}

/// Log out: clear tokens from memory and keychain.
#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<(), LanternError> {
    {
        let mut auth = state.auth.lock().unwrap();
        auth.profile = None;
        auth.tokens = None;
        auth.pending_device_code = None;
    }
    keychain::delete_refresh_token()?;
    Ok(())
}
