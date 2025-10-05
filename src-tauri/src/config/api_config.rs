use crate::config::keystore::ApiKeystore;
use crate::config::utils::internal_str_to_provider_type;
use crate::config::APP_PATHS;
use crate::constants::{ALL_PROVIDER_TYPES, API_SETTINGS_FILE_NAME};
use crate::errors::{AppError, AppResult};
use flyllm::{ModelDiscovery, ProviderType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Config for each provider
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub provider: String,
    pub available_models: Vec<String>,
    pub enabled: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            available_models: Vec::new(),
            enabled: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmUserConfig {
    pub provider_configs: HashMap<ProviderType, ProviderConfig>,
}

impl Default for LlmUserConfig {
    fn default() -> Self {
        let mut provider_configs = HashMap::new();

        for provider_type in ALL_PROVIDER_TYPES.iter().cloned() {
            let mut config = ProviderConfig::default();
            config.provider = format!("{:?}", provider_type);
            provider_configs.insert(provider_type, config);
        }

        Self { provider_configs }
    }
}

impl LlmUserConfig {
    /// Load configuration from file
    pub fn load() -> AppResult<Self> {
        let settings_path = APP_PATHS.config_dir.join(API_SETTINGS_FILE_NAME);

        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path).map_err(AppError::IoError)?;

            match serde_json::from_str::<Self>(&content) {
                Ok(mut config) => {
                    config.ensure_all_providers();
                    Ok(config)
                }
                Err(e) => {
                    eprintln!("Failed to parse config file, using defaults: {}", e);
                    Ok(Self::default())
                }
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> AppResult<()> {
        APP_PATHS.ensure_dirs_exist().map_err(AppError::IoError)?;

        let settings_path = APP_PATHS.config_dir.join(API_SETTINGS_FILE_NAME);

        let content = serde_json::to_string_pretty(&self)
            .map_err(|e| AppError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&settings_path, content).map_err(AppError::IoError)?;

        Ok(())
    }

    /// Ensure all provider types have entries
    fn ensure_all_providers(&mut self) {
        for provider_type in ALL_PROVIDER_TYPES.iter().cloned() {
            if !self.provider_configs.contains_key(&provider_type) {
                let mut config = ProviderConfig::default();
                config.provider = format!("{:?}", provider_type);
                self.provider_configs.insert(provider_type, config);
            }
        }
    }

    pub fn get_api_key(&self, provider_str: &str) -> AppResult<Option<String>> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        if provider_type == ProviderType::Ollama {
            return Ok(Some(String::new()));
        }

        let keystore = ApiKeystore::new();
        keystore.get_api_key(provider_str)
    }

    pub fn set_api_key(&mut self, provider_str: &str, api_key: String) -> AppResult<()> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        if !api_key.is_empty() {
            let keystore = ApiKeystore::new();
            keystore.store_api_key(provider_str, &api_key)?;
        }

        let config = self
            .provider_configs
            .entry(provider_type)
            .or_insert_with(|| {
                let mut config = ProviderConfig::default();
                config.provider = provider_str.to_string();
                config
            });

        self.save()
    }

    pub fn clear_api_key(&mut self, provider_str: &str) -> AppResult<()> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        let keystore = ApiKeystore::new();
        keystore.delete_api_key(provider_str)?;

        if let Some(config) = self.provider_configs.get_mut(&provider_type) {
            config.available_models.clear();
            config.enabled = false;
        }

        self.save()
    }

    pub fn set_enabled(&mut self, provider_str: &str, enabled: bool) -> AppResult<()> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        let config = self
            .provider_configs
            .entry(provider_type)
            .or_insert_with(|| {
                let mut config = ProviderConfig::default();
                config.provider = provider_str.to_string();
                config
            });

        config.enabled = enabled;
        self.save()
    }

    pub fn is_enabled(&self, provider_str: &str) -> AppResult<bool> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        Ok(self
            .provider_configs
            .get(&provider_type)
            .map(|config| config.enabled)
            .unwrap_or(false))
    }

    pub fn get_available_models(&self, provider_str: &str) -> AppResult<Vec<String>> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        Ok(self
            .provider_configs
            .get(&provider_type)
            .map(|config| config.available_models.clone())
            .unwrap_or_default())
    }

    pub fn set_available_models(
        &mut self,
        provider_str: &str,
        models: Vec<String>,
    ) -> AppResult<()> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        if let Some(config) = self.provider_configs.get_mut(&provider_type) {
            config.available_models = models;
        } else {
            return Err(AppError::ApiError(format!(
                "Provider '{}' not found",
                provider_str
            )));
        }

        self.save()
    }

    pub fn get_providers(&self) -> Vec<String> {
        self.provider_configs
            .keys()
            .map(|provider_type| format!("{:?}", provider_type))
            .collect()
    }

    pub fn get_enabled_providers(&self) -> Vec<String> {
        self.provider_configs
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(provider_type, _)| format!("{:?}", provider_type))
            .collect()
    }

    pub fn get_provider_config(&self, provider_str: &str) -> AppResult<ProviderConfig> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        self.provider_configs
            .get(&provider_type)
            .cloned()
            .ok_or_else(|| {
                AppError::ApiError(format!(
                    "Provider configuration not found for: {}",
                    provider_str
                ))
            })
    }

    pub fn is_provider_configured(&self, provider_str: &str) -> AppResult<bool> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        if let Some(config) = self.provider_configs.get(&provider_type) {
            let has_key = if provider_type == ProviderType::Ollama {
                true
            } else {
                let keystore = ApiKeystore::new();
                keystore.get_api_key(provider_str)?.is_some()
            };
            Ok(has_key && config.enabled)
        } else {
            Ok(false)
        }
    }

    pub fn reset_provider(&mut self, provider_str: &str) -> AppResult<()> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        let mut default_config = ProviderConfig::default();
        default_config.provider = provider_str.to_string();

        self.provider_configs.insert(provider_type, default_config);
        self.save()
    }

    pub fn get_config_summary(&self) -> HashMap<String, bool> {
        self.provider_configs
            .iter()
            .map(|(provider_type, config)| {
                let provider_str = format!("{:?}", provider_type);
                let is_configured = self.is_provider_configured(&provider_str).unwrap_or(false);
                (provider_str, is_configured)
            })
            .collect()
    }

    pub async fn refresh_available_models(&mut self, provider_str: &str) -> AppResult<()> {
        let provider_type =
            internal_str_to_provider_type(provider_str).map_err(|e| AppError::ConfigError(e))?;

        let config = self
            .provider_configs
            .get(&provider_type)
            .ok_or_else(|| AppError::ApiError(format!("Provider '{}' not found", provider_str)))?;

        let models = if provider_type == ProviderType::Ollama {
            ModelDiscovery::list_ollama_models(None)
                .await
                .map_err(|e| AppError::ApiError(format!("Failed to fetch Ollama models: {}", e)))?
                .into_iter()
                .map(|m| m.name)
                .collect()
        } else {
            let keystore = ApiKeystore::new();
            let api_key = keystore.get_api_key(provider_str)?.ok_or_else(|| {
                AppError::ApiError(format!("API key required for provider '{}'", provider_str))
            })?;

            let discovered_models = match provider_type {
                ProviderType::OpenAI => ModelDiscovery::list_openai_models(&api_key).await,
                ProviderType::Anthropic => ModelDiscovery::list_anthropic_models(&api_key).await,
                ProviderType::Mistral => ModelDiscovery::list_mistral_models(&api_key).await,
                ProviderType::Google => ModelDiscovery::list_google_models(&api_key).await,
                _ => {
                    return Err(AppError::ApiError(format!(
                        "Model discovery not supported for provider '{}'",
                        provider_str
                    )));
                }
            };

            discovered_models
                .map_err(|e| AppError::ApiError(format!("Failed to fetch models: {}", e)))?
                .into_iter()
                .map(|m| m.name)
                .collect()
        };

        self.set_available_models(provider_str, models)
    }

    pub async fn refresh_all_models(&mut self) -> AppResult<()> {
        let mut errors = Vec::new();

        for provider in ALL_PROVIDER_TYPES {
            let provider_str = provider.to_string();
            match self.refresh_available_models(&provider_str).await {
                Ok(()) => {
                    match self.get_available_models(&provider_str) {
                        Ok(_models) => {
                            // Models retrieved successfully
                        }
                        Err(e) => {
                            errors
                                .push(format!("Failed to get models for {}: {}", provider_str, e));
                        }
                    }
                }
                Err(e) => {
                    match self.get_available_models(&provider_str) {
                        Ok(models) if !models.is_empty() => {
                            eprintln!("Using cached models for {}: {}", provider_str, e);
                            // Continue with cached models
                        }
                        _ => {
                            errors.push(format!(
                                "Failed to fetch models for {}: {}",
                                provider_str, e
                            ));
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            eprintln!("Some provider models could not be retrieved: {:?}", errors);
            Ok(())
        }
    }

    pub fn validate_provider_setup(&self, provider: String) -> Vec<String> {
        let mut issues = Vec::new();
        if let Ok(provider_config) = self.get_provider_config(&provider) {
            if provider != "Ollama" {
                let keystore = ApiKeystore::new();
                match keystore.get_api_key(&provider) {
                    Ok(Some(_)) => {}
                    Ok(None) => issues.push("API key is required".to_string()),
                    Err(_) => issues.push("Failed to check API key".to_string()),
                }
            }

            if provider_config.available_models.is_empty() {
                issues.push("No models available, try refreshing models".to_string());
            }

            if !provider_config.enabled {
                issues.push("Provider is disabled".to_string());
            }
        } else {
            issues.push("Provider configuration not found".to_string());
        }

        issues
    }
}
