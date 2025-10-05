use crate::constants;
use crate::errors::{AppError, AppResult};
use keyring::Entry;

pub struct ApiKeystore {
    app_name: &'static str,
}

impl ApiKeystore {
    pub fn new() -> Self {
        Self {
            app_name: constants::APP_NAME,
        }
    }

    fn get_entry(&self, provider: &str) -> AppResult<Entry> {
        Entry::new(self.app_name, provider)
            .map_err(|e| AppError::ConfigError(format!("Failed to create keystore entry: {}", e)))
    }

    pub fn store_api_key(&self, provider: &str, api_key: &str) -> AppResult<()> {
        let entry = self.get_entry(provider)?;
        entry
            .set_password(api_key)
            .map_err(|e| AppError::ConfigError(format!("Failed to store API key: {}", e)))
    }

    pub fn get_api_key(&self, provider: &str) -> AppResult<Option<String>> {
        let entry = self.get_entry(provider)?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::ConfigError(format!(
                "Failed to retrieve API key: {}",
                e
            ))),
        }
    }

    pub fn delete_api_key(&self, provider: &str) -> AppResult<()> {
        let entry = self.get_entry(provider)?;
        match entry.delete_password() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::ConfigError(format!(
                "Failed to delete API key: {}",
                e
            ))),
        }
    }
}
