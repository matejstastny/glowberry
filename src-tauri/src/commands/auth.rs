use serde::Serialize;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl};

use crate::auth::keychain;
use crate::auth::microsoft::{self, MinecraftProfile, REDIRECT_URI};
use crate::error::GlowberryError;
use crate::state::AppState;

#[derive(Clone, Serialize)]
struct AuthComplete {
    profile: MinecraftProfile,
}

#[derive(Clone, Serialize)]
struct AuthError {
    message: String,
}

/// Open the Microsoft login webview and return the auth URL immediately.
/// Emits "auth-complete" { profile } on success, "auth-error" { message } on failure.
#[tauri::command]
pub async fn start_login(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, GlowberryError> {
    if let Some(w) = app.get_webview_window("auth-login") {
        let _ = w.close();
    }

    let auth_url = microsoft::build_auth_url();
    let parsed_url = url::Url::parse(&auth_url)
        .map_err(|e| GlowberryError::Auth(format!("Invalid auth URL: {e}")))?;

    let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, String>>();
    let tx = std::sync::Mutex::new(Some(tx));

    let _window = WebviewWindowBuilder::new(&app, "auth-login", WebviewUrl::External(parsed_url))
        .title("Sign in to Microsoft — Glowberry")
        .inner_size(480.0, 640.0)
        .resizable(false)
        .on_navigation(move |url| {
            if !url.as_str().starts_with(REDIRECT_URI) {
                return true;
            }
            let result = if let Some(code) = url
                .query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.to_string())
            {
                Ok(code)
            } else {
                let desc = url
                    .query_pairs()
                    .find(|(k, _)| k == "error_description")
                    .map(|(_, v)| v.to_string())
                    .unwrap_or_else(|| "Login was cancelled".to_string());
                Err(desc)
            };
            if let Some(tx) = tx.lock().unwrap().take() {
                let _ = tx.send(result);
            }
            false
        })
        .build()
        .map_err(|e| GlowberryError::Auth(format!("Failed to open login window: {e}")))?;

    let http_client = state.http_client.clone();
    let app_bg = app.clone();

    tokio::spawn(async move {
        let result: Result<MinecraftProfile, GlowberryError> = async {
            let auth_code = rx
                .await
                .map_err(|_| GlowberryError::Auth("Login window closed".into()))?
                .map_err(GlowberryError::Auth)?;

            if let Some(w) = app_bg.get_webview_window("auth-login") {
                let _ = w.close();
            }

            eprintln!("[auth] got auth code, exchanging tokens...");

            let msa = microsoft::exchange_auth_code(&http_client, &auth_code).await?;
            let (auth_tokens, profile) =
                microsoft::full_token_exchange(&http_client, &msa.access_token, &msa.refresh_token)
                    .await?;

            eprintln!("[auth] login complete: {}", profile.name);

            {
                let state = app_bg.state::<AppState>();
                keychain::save_refresh_token(&auth_tokens.msa_refresh_token, &state.data_dir)?;
                let mut auth = state.auth.lock().unwrap();
                auth.profile = Some(profile.clone());
                auth.tokens = Some(auth_tokens);
            }

            Ok(profile)
        }
        .await;

        match result {
            Ok(profile) => {
                let _ = app_bg.emit("auth-complete", AuthComplete { profile });
            }
            Err(e) => {
                eprintln!("[auth] login failed: {e}");
                let _ = app_bg.emit(
                    "auth-error",
                    AuthError {
                        message: e.to_string(),
                    },
                );
            }
        }
    });

    Ok(auth_url)
}

#[tauri::command]
pub fn cancel_login(app: AppHandle) {
    if let Some(w) = app.get_webview_window("auth-login") {
        let _ = w.close();
    }
}

#[tauri::command]
pub fn get_auth_status(state: State<'_, AppState>) -> Option<MinecraftProfile> {
    let auth = state.auth.lock().unwrap();
    auth.profile.clone()
}

#[tauri::command]
pub async fn try_restore_session(
    state: State<'_, AppState>,
) -> Result<Option<MinecraftProfile>, GlowberryError> {
    let refresh_token = match keychain::load_refresh_token(&state.data_dir)? {
        Some(t) => t,
        None => return Ok(None),
    };

    eprintln!("[auth] found stored token, restoring session...");

    let msa = match microsoft::refresh_msa_token(&state.http_client, &refresh_token).await {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("[auth] refresh failed: {e}");
            let _ = keychain::delete_refresh_token(&state.data_dir);
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
            let _ = keychain::delete_refresh_token(&state.data_dir);
            return Ok(None);
        }
    };

    eprintln!("[auth] session restored: {}", profile.name);

    keychain::save_refresh_token(&auth_tokens.msa_refresh_token, &state.data_dir)?;

    {
        let mut auth = state.auth.lock().unwrap();
        auth.profile = Some(profile.clone());
        auth.tokens = Some(auth_tokens);
    }

    Ok(Some(profile))
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<(), GlowberryError> {
    {
        let mut auth = state.auth.lock().unwrap();
        auth.profile = None;
        auth.tokens = None;
    }
    keychain::delete_refresh_token(&state.data_dir)?;
    eprintln!("[auth] logged out");
    Ok(())
}
