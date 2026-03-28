use crate::error::LanternError;

const SERVICE_NAME: &str = "com.lantern.launcher";
const REFRESH_TOKEN_KEY: &str = "msa_refresh_token";

pub fn save_refresh_token(token: &str) -> Result<(), LanternError> {
    let entry = keyring::Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .map_err(|e| LanternError::Auth(format!("Keychain error: {e}")))?;
    entry
        .set_password(token)
        .map_err(|e| LanternError::Auth(format!("Failed to save token: {e}")))?;
    Ok(())
}

pub fn load_refresh_token() -> Result<Option<String>, LanternError> {
    let entry = keyring::Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .map_err(|e| LanternError::Auth(format!("Keychain error: {e}")))?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(LanternError::Auth(format!(
            "Failed to load token: {e}"
        ))),
    }
}

pub fn delete_refresh_token() -> Result<(), LanternError> {
    let entry = keyring::Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .map_err(|e| LanternError::Auth(format!("Keychain error: {e}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(LanternError::Auth(format!(
            "Failed to delete token: {e}"
        ))),
    }
}
