use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager, State};

use crate::auth::keychain;
use crate::auth::microsoft::{self, DeviceCodeInfo, MinecraftProfile, PollOutcome};
use crate::error::GlowberryError;
use crate::state::AppState;

/// Emitted when login completes successfully.
#[derive(Clone, Serialize)]
struct AuthComplete {
    profile: MinecraftProfile,
}

/// Emitted when login fails or is cancelled.
#[derive(Clone, Serialize)]
struct AuthError {
    message: String,
}

/// Begin the device-code login flow.
/// Returns device code info immediately so the frontend can render the QR code
/// and user code. Spawns a background task that polls Microsoft until the user
/// completes sign-in, then emits:
///   - "auth-complete" { profile } on success
///   - "auth-error"    { message } on failure / cancellation
#[tauri::command]
pub async fn start_login(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<DeviceCodeInfo, GlowberryError> {
    // Reset cancellation flag from any previous attempt.
    state
        .login_cancelled
        .store(false, Ordering::Relaxed);

    let info = microsoft::request_device_code(&state.http_client).await?;

    let http_client = state.http_client.clone();
    let cancelled: Arc<AtomicBool> = state.login_cancelled.clone();
    let device_code = info.device_code.clone();
    let base_interval = info.interval;
    let app_bg = app.clone();

    tokio::spawn(async move {
        let result: Result<MinecraftProfile, GlowberryError> = async {
            let mut slow_downs: u32 = 0;

            loop {
                tokio::time::sleep(microsoft::poll_interval(base_interval, slow_downs)).await;

                if cancelled.load(Ordering::Relaxed) {
                    return Err(GlowberryError::Auth("Login cancelled".into()));
                }

                match microsoft::poll_device_token(&http_client, &device_code).await? {
                    PollOutcome::Pending => continue,
                    PollOutcome::SlowDown => {
                        slow_downs += 1;
                        continue;
                    }
                    PollOutcome::Tokens(msa) => {
                        eprintln!("[auth] got MSA tokens, exchanging...");
                        let (auth_tokens, profile) = microsoft::full_token_exchange(
                            &http_client,
                            &msa.access_token,
                            &msa.refresh_token,
                        )
                        .await?;

                        eprintln!("[auth] login complete: {}", profile.name);

                        {
                            let state = app_bg.state::<AppState>();
                            keychain::save_refresh_token(
                                &auth_tokens.msa_refresh_token,
                                &state.data_dir,
                            )?;
                            let mut auth = state.auth.lock().unwrap();
                            auth.profile = Some(profile.clone());
                            auth.tokens = Some(auth_tokens);
                        }

                        return Ok(profile);
                    }
                }
            }
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

    Ok(info)
}

/// Cancel an in-progress device code login.
#[tauri::command]
pub fn cancel_login(state: State<'_, AppState>) {
    state.login_cancelled.store(true, Ordering::Relaxed);
}

/// Get the current auth status (logged-in profile or null).
#[tauri::command]
pub fn get_auth_status(state: State<'_, AppState>) -> Option<MinecraftProfile> {
    let auth = state.auth.lock().unwrap();
    auth.profile.clone()
}

/// Try to restore a previous session using the stored refresh token.
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

/// Log out: clear in-memory tokens and keychain entry.
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
