use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Manager, State, WebviewUrl};

use crate::auth::keychain;
use crate::auth::microsoft::{self, MinecraftProfile, REDIRECT_URI};
use crate::error::LanternError;
use crate::state::AppState;

/// Open a Microsoft login window. Returns the profile on success.
/// The window closes automatically after login completes.
#[tauri::command]
pub async fn start_login(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<MinecraftProfile, LanternError> {
    // Close any leftover auth window from a previous attempt
    if let Some(w) = app.get_webview_window("auth-login") {
        let _ = w.close();
    }

    let auth_url = microsoft::build_auth_url();
    let parsed_url = url::Url::parse(&auth_url)
        .map_err(|e| LanternError::Auth(format!("Invalid auth URL: {e}")))?;

    // Channel to receive the auth code from the navigation callback
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, String>>();
    let tx = std::sync::Mutex::new(Some(tx));

    eprintln!("[auth] opening login window...");

    let window = WebviewWindowBuilder::new(&app, "auth-login", WebviewUrl::External(parsed_url))
        .title("Sign in — Lantern")
        .inner_size(500.0, 650.0)
        .resizable(true)
        .on_navigation(move |url| {
            let url_str = url.as_str();
            if !url_str.starts_with(REDIRECT_URI) {
                return true; // allow normal navigation
            }

            // Microsoft redirected back — extract code or error
            let result = if let Some(code) = url
                .query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.to_string())
            {
                Ok(code)
            } else if let Some(desc) = url
                .query_pairs()
                .find(|(k, _)| k == "error_description")
                .map(|(_, v)| v.to_string())
            {
                Err(desc)
            } else {
                Err("Login was cancelled".to_string())
            };

            if let Some(tx) = tx.lock().unwrap().take() {
                let _ = tx.send(result);
            }
            false // block redirect — we'll close the window
        })
        .build()
        .map_err(|e| LanternError::Auth(format!("Failed to open login window: {e}")))?;

    // Wait for the navigation callback to fire (or window to be closed)
    let auth_code = rx
        .await
        .map_err(|_| LanternError::Auth("Login window was closed".into()))?
        .map_err(LanternError::Auth)?;

    let _ = window.close();

    eprintln!("[auth] got auth code, exchanging tokens...");

    // Exchange auth code → MSA tokens
    let msa = microsoft::exchange_auth_code(&state.http_client, &auth_code).await?;

    // Full chain: MSA → XBL → XSTS → Minecraft
    let (auth_tokens, profile) =
        microsoft::full_token_exchange(&state.http_client, &msa.access_token, &msa.refresh_token)
            .await?;

    eprintln!("[auth] login complete: {}", profile.name);

    // Save refresh token to OS keychain
    keychain::save_refresh_token(&auth_tokens.msa_refresh_token)?;

    // Update in-memory state
    {
        let mut auth = state.auth.lock().unwrap();
        auth.profile = Some(profile.clone());
        auth.tokens = Some(auth_tokens);
    }

    Ok(profile)
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

    eprintln!("[auth] found stored token, restoring session...");

    let msa = match microsoft::refresh_msa_token(&state.http_client, &refresh_token).await {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("[auth] refresh failed: {e}");
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
        Err(e) => {
            eprintln!("[auth] token exchange failed during restore: {e}");
            let _ = keychain::delete_refresh_token();
            return Ok(None);
        }
    };

    eprintln!("[auth] session restored: {}", profile.name);

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
    }
    keychain::delete_refresh_token()?;
    eprintln!("[auth] logged out");
    Ok(())
}
