use crate::commands::structure::responses;
use crate::config::api_config::LlmUserConfig;
use crate::config::app_settings::AgentSettings;
use crate::config::keystore::ApiKeystore;
use crate::config::utils::{internal_str_to_provider_type, str_to_agent_type};
use crate::services::llm_service::agents::AgentType;
use crate::state::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn get_api_key(provider: String) -> Result<Option<String>, String> {
    let keystore = ApiKeystore::new();
    keystore.get_api_key(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_api_key(
    state: State<'_, AppState>,
    provider: String,
    api_key: String,
) -> Result<(), String> {
    let mut config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    config
        .set_api_key(&provider, api_key)
        .map_err(|e| e.to_string())?;

    state
        .rebuild_llm_service()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn clear_api_key(state: State<'_, AppState>, provider: String) -> Result<(), String> {
    let mut config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    config.clear_api_key(&provider).map_err(|e| e.to_string())?;

    state
        .rebuild_llm_service()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn validate_api_key_and_fetch_models(
    provider: String,
    api_key: String,
) -> Result<responses::ValidationResponse, String> {
    let mut config = LlmUserConfig::load().map_err(|e| e.to_string())?;

    config
        .set_api_key(&provider, api_key)
        .map_err(|e| e.to_string())?;

    match config.refresh_available_models(&provider).await {
        Ok(()) => {
            let models = config
                .get_available_models(&provider)
                .map_err(|e| e.to_string())?;
            Ok(responses::ValidationResponse {
                valid: true,
                models,
                error_message: None,
            })
        }
        Err(e) => {
            config.clear_api_key(&provider).map_err(|e| e.to_string())?;
            Ok(responses::ValidationResponse {
                valid: false,
                models: vec![],
                error_message: Some(e.to_string()),
            })
        }
    }
}

#[tauri::command]
pub async fn refresh_provider_models(provider: String) -> Result<Vec<String>, String> {
    let mut config = LlmUserConfig::load().map_err(|e| e.to_string())?;

    match config.refresh_available_models(&provider).await {
        Ok(()) => config
            .get_available_models(&provider)
            .map_err(|e| e.to_string()),
        Err(e) => match config.get_available_models(&provider) {
            Ok(models) if !models.is_empty() => {
                eprintln!("Using cached models for {}: {}", provider, e);
                Ok(models)
            }
            _ => Err(format!("Failed to fetch models for {}: {}", provider, e)),
        },
    }
}

#[tauri::command]
pub async fn set_provider_enabled(
    state: State<'_, AppState>,
    provider: String,
    enabled: bool,
) -> Result<(), String> {
    let mut config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    config
        .set_enabled(&provider, enabled)
        .map_err(|e| e.to_string())?;
    state
        .rebuild_llm_service()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn is_provider_enabled(provider: String) -> Result<bool, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    config.is_enabled(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_providers() -> Result<Vec<String>, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    Ok(config.get_providers())
}

#[tauri::command]
pub fn get_enabled_providers() -> Result<Vec<String>, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    Ok(config.get_enabled_providers())
}

#[tauri::command]
pub fn get_provider_config(provider: String) -> Result<responses::ProviderConfigResponse, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    let provider_config = config
        .get_provider_config(&provider)
        .map_err(|e| e.to_string())?;
    let is_configured = config
        .is_provider_configured(&provider)
        .map_err(|e| e.to_string())?;

    let keystore = ApiKeystore::new();
    let api_key = keystore
        .get_api_key(&provider)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    Ok(responses::ProviderConfigResponse {
        provider_id: provider,
        api_key,
        enabled: provider_config.enabled,
        available_models: provider_config.available_models,
        is_configured,
    })
}

#[tauri::command]
pub fn get_all_provider_configs() -> Result<Vec<responses::ProviderConfigResponse>, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    let mut responses = Vec::new();

    for provider in config.get_providers() {
        match get_provider_config(provider.clone()) {
            Ok(response) => responses.push(response),
            Err(e) => eprintln!("Failed to get config for {}: {}", provider, e),
        }
    }

    Ok(responses)
}

#[tauri::command]
pub fn get_config_summary() -> Result<std::collections::HashMap<String, bool>, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    Ok(config.get_config_summary())
}

#[tauri::command]
pub fn save_config(config: LlmUserConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_config() -> Result<LlmUserConfig, String> {
    LlmUserConfig::load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_provider_configured(provider: String) -> Result<bool, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    config
        .is_provider_configured(&provider)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn validate_provider_setup(provider: String) -> Result<Vec<String>, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    let issues = config.validate_provider_setup(provider);

    Ok(issues)
}

#[tauri::command]
pub fn get_all_agent_model_configs() -> Result<Vec<responses::AgentModelConfigResponse>, String> {
    let agent_settings = AgentSettings::load().map_err(|e| e.to_string())?;
    let mut responses = Vec::new();

    for agent_type in AgentType::all() {
        let agent_str = agent_type.as_str().to_string();

        // Get models for this agent from agent_settings
        let provider_models = agent_settings
            .agent_models_per_provider
            .get(&agent_type)
            .map(|models| {
                models
                    .iter()
                    .map(|(provider_type, model)| (format!("{:?}", provider_type), model.clone()))
                    .collect()
            })
            .unwrap_or_default();

        responses.push(responses::AgentModelConfigResponse {
            agent_type: agent_str,
            provider_models,
        });
    }

    Ok(responses)
}

#[tauri::command]
pub async fn reset_agent_models_to_default(
    state: State<'_, AppState>,
    agent: String,
) -> Result<(), String> {
    let mut agent_settings = AgentSettings::load().map_err(|e| e.to_string())?;
    let agent_type = str_to_agent_type(&agent).map_err(|e| e.to_string())?;

    // Reset to default models for all providers
    let mut default_models = HashMap::new();
    for provider_type in crate::constants::ALL_PROVIDER_TYPES.iter() {
        let default_model = crate::constants::get_default_agent_model(&agent_type, provider_type);
        default_models.insert(provider_type.clone(), default_model.to_string());
    }

    agent_settings
        .agent_models_per_provider
        .insert(agent_type, default_models);
    agent_settings.save().map_err(|e| e.to_string())?;

    state
        .rebuild_llm_service()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_all_agents() -> Result<Vec<String>, String> {
    Ok(AgentType::all()
        .iter()
        .map(|a| a.as_str().to_string())
        .collect())
}

#[tauri::command]
pub async fn set_ollama_endpoint(
    state: State<'_, AppState>,
    endpoint_url: String,
) -> Result<(), String> {
    let mut config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    config.set_ollama_endpoint(Some(endpoint_url)).map_err(|e| e.to_string())?;
    state
        .rebuild_llm_service()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_ollama_endpoint() -> Result<String, String> {
    let config = LlmUserConfig::load().map_err(|e| e.to_string())?;
    config
        .get_ollama_endpoint()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No custom Ollama endpoint configured".to_string())
}
