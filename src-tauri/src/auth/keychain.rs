use std::path::Path;

use crate::error::GlowberryError;

#[cfg(not(target_os = "linux"))]
const SERVICE_NAME: &str = "com.glowberry.launcher";
#[cfg(not(target_os = "linux"))]
const REFRESH_TOKEN_KEY: &str = "msa_refresh_token";

const TOKEN_FILE: &str = "refresh_token";

pub fn save_refresh_token(token: &str, data_dir: &Path) -> Result<(), GlowberryError> {
    #[cfg(target_os = "linux")]
    return save_to_file(token, data_dir);

    #[cfg(not(target_os = "linux"))]
    {
        let _ = data_dir;
        let entry = keyring::Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
            .map_err(|e| GlowberryError::Auth(format!("Keychain error: {e}")))?;
        entry
            .set_password(token)
            .map_err(|e| GlowberryError::Auth(format!("Failed to save token: {e}")))?;
        Ok(())
    }
}

pub fn load_refresh_token(data_dir: &Path) -> Result<Option<String>, GlowberryError> {
    #[cfg(target_os = "linux")]
    return load_from_file(data_dir);

    #[cfg(not(target_os = "linux"))]
    {
        let _ = data_dir;
        let entry = keyring::Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
            .map_err(|e| GlowberryError::Auth(format!("Keychain error: {e}")))?;
        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(GlowberryError::Auth(format!("Failed to load token: {e}"))),
        }
    }
}

pub fn delete_refresh_token(data_dir: &Path) -> Result<(), GlowberryError> {
    #[cfg(target_os = "linux")]
    return delete_from_file(data_dir);

    #[cfg(not(target_os = "linux"))]
    {
        let _ = data_dir;
        let entry = keyring::Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
            .map_err(|e| GlowberryError::Auth(format!("Keychain error: {e}")))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(GlowberryError::Auth(format!("Failed to delete token: {e}"))),
        }
    }
}

#[cfg(target_os = "linux")]
fn save_to_file(token: &str, data_dir: &Path) -> Result<(), GlowberryError> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let path = data_dir.join(TOKEN_FILE);
    fs::write(&path, token)
        .map_err(|e| GlowberryError::Auth(format!("Failed to save token: {e}")))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
        .map_err(|e| GlowberryError::Auth(format!("Failed to set token permissions: {e}")))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn load_from_file(data_dir: &Path) -> Result<Option<String>, GlowberryError> {
    let path = data_dir.join(TOKEN_FILE);
    match std::fs::read_to_string(&path) {
        Ok(token) => Ok(Some(token.trim().to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(GlowberryError::Auth(format!("Failed to load token: {e}"))),
    }
}

#[cfg(target_os = "linux")]
fn delete_from_file(data_dir: &Path) -> Result<(), GlowberryError> {
    let path = data_dir.join(TOKEN_FILE);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(GlowberryError::Auth(format!("Failed to delete token: {e}"))),
    }
}
