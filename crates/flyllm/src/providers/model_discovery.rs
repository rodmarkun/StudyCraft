use crate::constants;
use crate::errors::{LlmError, LlmResult};
use crate::providers::types::{ModelInfo, ProviderType};
use reqwest::{header, Client};
use serde::Deserialize;
use std::time::Duration;

/// Helper module for listing available models from providers
/// without requiring a fully initialized provider instance
pub struct ModelDiscovery;

impl ModelDiscovery {
    /// Create a standardized HTTP client for model discovery
    fn create_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }

    /// List available models from Anthropic
    ///
    /// # Parameters
    /// * `api_key` - Anthropic API key
    ///
    /// # Returns
    /// * Vector of ModelInfo structs containing model names
    pub async fn list_anthropic_models(api_key: &str) -> LlmResult<Vec<ModelInfo>> {
        let client = Self::create_client();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(api_key).map_err(|e| {
                LlmError::ConfigError(format!("Invalid API key format for Anthropic: {}", e))
            })?,
        );
        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_static(constants::ANTHROPIC_API_VERSION),
        );

        let models_endpoint = "https://api.anthropic.com/v1/models";

        let response = client.get(models_endpoint).headers(headers).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| {
                format!(
                    "Unknown error reading error response body, status: {}",
                    status
                )
            });
            return Err(LlmError::ApiError(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )));
        }

        let response_bytes = response.bytes().await?;

        #[derive(Deserialize, Debug)]
        struct AnthropicModelsResponse {
            data: Vec<AnthropicModelInfo>,
        }
        #[derive(Deserialize, Debug)]
        struct AnthropicModelInfo {
            id: String,
            display_name: String,
        }

        let anthropic_response: AnthropicModelsResponse = serde_json::from_slice(&response_bytes)
            .map_err(|e| {
            let snippet_len = std::cmp::min(response_bytes.len(), 256);
            let response_snippet =
                String::from_utf8_lossy(response_bytes.get(0..snippet_len).unwrap_or_default());
            LlmError::ParseError(format!(
                "Error decoding Anthropic models JSON: {}. Response snippet: '{}'",
                e, response_snippet
            ))
        })?;

        let models = anthropic_response
            .data
            .into_iter()
            .map(|m| ModelInfo {
                name: m.id,
                provider: ProviderType::Anthropic,
            })
            .collect();

        Ok(models)
    }

    /// List available models from OpenAI
    ///
    /// # Parameters
    /// * `api_key` - OpenAI API key
    ///
    /// # Returns
    /// * Vector of ModelInfo structs containing model names
    pub async fn list_openai_models(api_key: &str) -> LlmResult<Vec<ModelInfo>> {
        let client = Self::create_client();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| LlmError::ConfigError(format!("Invalid API key format: {}", e)))?,
        );

        let models_endpoint = "https://api.openai.com/v1/models";

        let response = client.get(models_endpoint).headers(headers).send().await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "OpenAI API error: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct OpenAIModelsResponse {
            data: Vec<OpenAIModelInfo>,
        }

        #[derive(Deserialize)]
        struct OpenAIModelInfo {
            id: String,
        }

        let openai_response: OpenAIModelsResponse = response.json().await?;

        let models = openai_response
            .data
            .into_iter()
            .filter(|m| m.id.starts_with("gpt-"))
            .map(|m| ModelInfo {
                name: m.id,
                provider: ProviderType::OpenAI,
            })
            .collect();

        Ok(models)
    }

    /// List available models from Mistral
    ///
    /// # Parameters
    /// * `api_key` - Mistral API key
    ///
    /// # Returns
    /// * Vector of ModelInfo structs containing model names
    pub async fn list_mistral_models(api_key: &str) -> LlmResult<Vec<ModelInfo>> {
        let client = Self::create_client();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| LlmError::ConfigError(format!("Invalid API key format: {}", e)))?,
        );

        let models_endpoint = "https://api.mistral.ai/v1/models";

        let response = client.get(models_endpoint).headers(headers).send().await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Mistral API error: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct MistralModelsResponse {
            data: Vec<MistralModelInfo>,
        }

        #[derive(Deserialize)]
        struct MistralModelInfo {
            id: String,
        }

        let mistral_response: MistralModelsResponse = response.json().await?;

        let models = mistral_response
            .data
            .into_iter()
            .map(|m| ModelInfo {
                name: m.id,
                provider: ProviderType::Mistral,
            })
            .collect();

        Ok(models)
    }

    /// List available models from Google
    ///
    /// # Parameters
    /// * `api_key` - Google API key
    ///
    /// # Returns
    /// * Vector of ModelInfo structs containing model names
    pub async fn list_google_models(api_key: &str) -> LlmResult<Vec<ModelInfo>> {
        let client = Self::create_client();

        let models_endpoint = format!(
            "{}/v1beta/models?key={}",
            constants::GOOGLE_API_ENDPOINT_PREFIX,
            api_key
        );

        let response = client.get(&models_endpoint).send().await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Google API error: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct GoogleModelsResponse {
            models: Vec<GoogleModelInfo>,
        }

        #[derive(Deserialize)]
        struct GoogleModelInfo {
            name: String,
            #[serde(default)]
            display_name: Option<String>,
        }

        let google_response: GoogleModelsResponse = response.json().await?;

        let models = google_response
            .models
            .into_iter()
            .map(|m| {
                let name = m
                    .display_name
                    .unwrap_or_else(|| m.name.split('/').last().unwrap_or(&m.name).to_string());

                ModelInfo {
                    name,
                    provider: ProviderType::Google,
                }
            })
            .collect();

        Ok(models)
    }

    /// List available models from Ollama
    ///
    /// # Parameters
    /// * `base_url` - Optional base URL for Ollama API, defaults to localhost
    ///
    /// # Returns
    /// * Vector of ModelInfo structs containing model names
    pub async fn list_ollama_models(base_url: Option<&str>) -> LlmResult<Vec<ModelInfo>> {
        let client = Self::create_client();

        // Use provided base URL or default to localhost
        let base_url = base_url.unwrap_or("http://localhost:11434");
        let models_endpoint = format!("{}/api/tags", base_url.trim_end_matches('/'));

        let response = client.get(&models_endpoint).send().await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Ollama API error: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct OllamaModelsResponse {
            models: Vec<OllamaModelInfo>,
        }

        #[derive(Deserialize)]
        struct OllamaModelInfo {
            name: String,
        }

        let ollama_response: OllamaModelsResponse = response.json().await?;

        let models = ollama_response
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                provider: ProviderType::Ollama,
            })
            .collect();

        Ok(models)
    }

    /// List available models from LM Studio
    ///
    /// # Parameters
    /// * `base_url` - Optional base URL for LM Studio API, defaults to localhost
    ///
    /// # Returns
    /// * Vector of ModelInfo structs containing model names
    pub async fn list_lmstudio_models(base_url: Option<&str>) -> LlmResult<Vec<ModelInfo>> {
        let client = Self::create_client();

        let base_url = base_url.unwrap_or("http://127.0.0.1:1234");
        let models_endpoint = format!("{}/v1/models", base_url.trim_end_matches('/'));

        let response = client.get(&models_endpoint).send().await?;
        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            let message = String::from_utf8_lossy(&body).to_string();
            return Err(LlmError::ApiError(format!(
                "LmStudio API error: {}",
                message
            )));
        }

        #[derive(Deserialize)]
        struct LmStudioModelInfo {
            id: String,
            #[serde(default)]
            object: Option<String>,
            #[serde(default)]
            owned_by: Option<String>,
        }

        #[derive(Deserialize)]
        struct LmStudioModelsResponse {
            data: Vec<LmStudioModelInfo>,
        }

        let model_names = match serde_json::from_slice::<LmStudioModelsResponse>(&body) {
            Ok(parsed) => parsed
                .data
                .into_iter()
                .map(|m| m.id)
                .collect::<Vec<String>>(),
            Err(_) => {
                let value: serde_json::Value = serde_json::from_slice(&body)?;
                if let Some(models) = value.get("models").and_then(|v| v.as_array()) {
                    models
                        .iter()
                        .filter_map(|model| model.get("id").and_then(|id| id.as_str()))
                        .map(|name| name.to_string())
                        .collect()
                } else {
                    return Err(LlmError::ParseError(format!(
                        "Unexpected LM Studio models response: {}",
                        String::from_utf8_lossy(&body)
                    )));
                }
            }
        };

        Ok(model_names
            .into_iter()
            .map(|name| ModelInfo {
                name,
                provider: ProviderType::LmStudio,
            })
            .collect())
    }

    /// List all models from a specific provider
    ///
    /// # Parameters
    /// * `provider_type` - Type of provider to query
    /// * `api_key` - API key for authentication
    /// * `base_url` - Optional base URL (only used for Ollama)
    ///
    /// # Returns
    /// * Vector of ModelInfo structs containing model names
    pub async fn list_models(
        provider_type: ProviderType,
        api_key: &str,
        base_url: Option<&str>,
    ) -> LlmResult<Vec<ModelInfo>> {
        match provider_type {
            ProviderType::Anthropic => Self::list_anthropic_models(api_key).await,
            ProviderType::OpenAI => Self::list_openai_models(api_key).await,
            ProviderType::Mistral => Self::list_mistral_models(api_key).await,
            ProviderType::Google => Self::list_google_models(api_key).await,
            ProviderType::Ollama => Self::list_ollama_models(base_url).await,
            ProviderType::LmStudio => Self::list_lmstudio_models(base_url).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ModelDiscovery;
    use crate::providers::types::ProviderType;
    use httpmock::prelude::*;

    fn sample_models_response() -> serde_json::Value {
        serde_json::json!({
            "data": [
                { "id": "local/model-1", "object": "model", "owned_by": "local" },
                { "id": "local/model-2", "object": "model", "owned_by": "local" }
            ],
            "object": "list"
        })
    }

    #[tokio::test]
    async fn list_lmstudio_models_returns_models_from_server() {
        let server = MockServer::start_async().await;

        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/models");
                then.status(200).json_body(sample_models_response());
            })
            .await;

        let base_url = format!("http://{}", server.address());

        let result = ModelDiscovery::list_lmstudio_models(Some(&base_url))
            .await
            .expect("models should be parsed");

        let names: Vec<String> = result.into_iter().map(|m| m.name).collect();
        assert_eq!(
            names,
            vec!["local/model-1".to_string(), "local/model-2".to_string()]
        );
    }

    #[tokio::test]
    async fn list_lmstudio_models_returns_error_on_bad_status() {
        let server = MockServer::start_async().await;

        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/models");
                then.status(404).body("Not Found");
            })
            .await;

        let base_url = format!("http://{}", server.address());

        let err = ModelDiscovery::list_lmstudio_models(Some(&base_url))
            .await
            .expect_err("should return error for non-success status");

        match err {
            crate::errors::LlmError::ApiError(message) => {
                assert!(
                    message.contains("LmStudio"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected ApiError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn list_models_dispatches_to_lmstudio() {
        let server = MockServer::start_async().await;

        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/v1/models");
                then.status(200).json_body(sample_models_response());
            })
            .await;

        let base_url = format!("http://{}", server.address());

        let models = ModelDiscovery::list_models(ProviderType::LmStudio, "", Some(&base_url))
            .await
            .expect("models should succeed");

        assert_eq!(models.len(), 2);
    }
}
