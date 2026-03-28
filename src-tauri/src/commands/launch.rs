use tauri::{AppHandle, State};

use crate::error::LanternError;
use crate::minecraft::launch;
use crate::state::AppState;

#[tauri::command]
pub async fn launch_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    online: bool,
    username: Option<String>,
) -> Result<(), LanternError> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.get(&instance_id)?
    };

    // Get auth info if online
    let (auth_name, auth_uuid, auth_token) = if online {
        let auth = state.auth.lock().unwrap();
        match (&auth.profile, &auth.tokens) {
            (Some(profile), Some(tokens)) => (
                Some(profile.name.clone()),
                Some(profile.id.clone()),
                Some(tokens.minecraft_access_token.clone()),
            ),
            _ => {
                return Err(LanternError::Launch(
                    "Not logged in — sign in or use offline mode".into(),
                ));
            }
        }
    } else {
        (username, None, None)
    };

    launch::launch_instance(app, &state, &instance, online, auth_name, auth_uuid, auth_token)
        .await?;

    // Update last_played
    {
        let instances = state.instances.lock().unwrap();
        let mut updated = instances.get(&instance_id)?;
        updated.last_played = Some(chrono::Utc::now());
        instances.save(&updated)?;
    }

    Ok(())
}
