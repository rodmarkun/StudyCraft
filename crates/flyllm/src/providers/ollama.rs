use crate::constants;
use crate::errors::{LlmError, LlmResult};
use crate::load_balancer::tasks::TaskDefinition;
use crate::providers::instances::{BaseInstance, LlmInstance};
use crate::providers::types::{LlmRequest, LlmResponse, Message, TokenUsage};
use async_trait::async_trait;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Provider implementation for Ollama (local LLMs)
pub struct OllamaInstance {
    base: BaseInstance,
    // Specific URL for this provider instance
    endpoint_url: String,
}

/// Request structure for Ollama's chat API
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize, Default)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>, // Corresponds to max_tokens
}

/// Response structure from Ollama's chat API (non-streaming)
#[derive(Deserialize, Debug)]
struct OllamaResponse {
    model: String,
    created_at: String,
    message: Message,
    done: bool, // Should be true for non-streaming response
    #[serde(default)] // Use default (0) if not present
    prompt_eval_count: u32,
    #[serde(default)] // Use default (0) if not present
    eval_count: u32, // Corresponds roughly to completion tokens
}

impl OllamaInstance {
    /// Creates a new Ollama provider instance
    ///
    /// # Parameters
    /// * `api_key` - Unused for Ollama by default, but kept for consistency. Could be repurposed (e.g., for future auth or endpoint override).
    /// * `model` - Default model to use (e.g., "llama3")
    /// * `supported_tasks` - Map of tasks this provider supports
    /// * `enabled` - Whether this provider is enabled
    /// * `endpoint_url` - Optional base endpoint URL override. If None, uses the default from constants.
    pub fn new(
        api_key: String,
        model: String,
        supported_tasks: HashMap<String, TaskDefinition>,
        enabled: bool,
        endpoint_url: Option<String>,
    ) -> Self {
        // Determine the endpoint: use provided one or default
        let base_endpoint =
            endpoint_url.unwrap_or_else(|| constants::OLLAMA_API_ENDPOINT.to_string());

        // Validate and ensure the path ends correctly
        let final_endpoint = match Url::parse(&base_endpoint) {
            Ok(mut url) => {
                if !url.path().ends_with("/api/chat") {
                    if url.path() == "/" {
                        url.set_path("api/chat");
                    } else {
                        let current_path = url.path().trim_end_matches('/');
                        url.set_path(&format!("{}/api/chat", current_path));
                    }
                }
                url.to_string()
            }
            Err(_) => {
                eprintln!(
                    "Warning: Invalid Ollama endpoint URL '{}' provided. Falling back to default: {}",
                    base_endpoint, constants::OLLAMA_API_ENDPOINT
                );
                constants::OLLAMA_API_ENDPOINT.to_string()
            }
        };

        // Create BaseProvider with the actual API key (even if empty/unused)
        let base = BaseInstance::new(
            "ollama".to_string(),
            api_key,
            model,
            supported_tasks,
            enabled,
        );

        Self {
            base,
            endpoint_url: final_endpoint,
        }
    }
}

#[async_trait]
impl LlmInstance for OllamaInstance {
    /// Generates a completion using Ollama's API
    ///
    /// # Parameters
    /// * `request` - The LLM request containing messages and parameters
    ///
    /// # Returns
    /// * `LlmResult<LlmResponse>` - The response from the model or an error
    async fn generate(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        if !self.base.is_enabled() {
            return Err(LlmError::ProviderDisabled("Ollama".to_string()));
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        // Add Authorization header if an API key is actually provided and non-empty
        if !self.base.api_key().is_empty() {
            match header::HeaderValue::from_str(&format!("Bearer {}", self.base.api_key())) {
                Ok(val) => {
                    headers.insert(header::AUTHORIZATION, val);
                }
                Err(e) => {
                    return Err(LlmError::ConfigError(format!(
                        "Invalid API key format for Ollama: {}",
                        e
                    )))
                }
            }
        }

        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.base.model().to_string());

        // Map common parameters to Ollama options
        let mut options = OllamaOptions::default();
        if request.temperature.is_some() {
            options.temperature = request.temperature;
        }
        if request.max_tokens.is_some() {
            options.num_predict = request.max_tokens;
        }

        let ollama_request = OllamaRequest {
            model,
            messages: request.messages.clone(),
            stream: false,
            options: if options.temperature.is_some() || options.num_predict.is_some() {
                Some(options)
            } else {
                None
            },
        };

        let response = self
            .base
            .client()
            .post(&self.endpoint_url)
            .headers(headers)
            .json(&ollama_request)
            .send()
            .await?;

        let response_status = response.status();
        if !response_status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| format!("Unknown error. Status: {}", response_status));
            return Err(LlmError::ApiError(format!(
                "Ollama API error: {}",
                error_text
            )));
        }

        let response_text = response.text().await?;
        if response_text.is_empty() {
            return Err(LlmError::ApiError(
                "Received empty response body from Ollama".to_string(),
            ));
        }

        // Attempt to parse the JSON response
        let ollama_response: OllamaResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                LlmError::ApiError(format!(
                    "Failed to parse Ollama JSON response: {}. Body: {}",
                    e, response_text
                ))
            })?;

        // Map Ollama token counts to unified format.
        // Note: Ollama's `eval_count` is often used for completion tokens. `prompt_eval_count` for prompt.
        // The exact definition might vary slightly depending on the model and Ollama version.
        let usage = Some(TokenUsage {
            prompt_tokens: ollama_response.prompt_eval_count,
            completion_tokens: ollama_response.eval_count,
            total_tokens: ollama_response.prompt_eval_count + ollama_response.eval_count,
        });

        Ok(LlmResponse {
            content: ollama_response.message.content.clone(),
            model: ollama_response.model,
            usage,
        })
    }

    /// Returns provider name
    fn get_name(&self) -> &str {
        self.base.name()
    }

    /// Returns current model name
    fn get_model(&self) -> &str {
        self.base.model()
    }

    /// Returns supported tasks for this provider
    fn get_supported_tasks(&self) -> &HashMap<String, TaskDefinition> {
        &self.base.supported_tasks()
    }

    /// Returns whether this provider is enabled
    fn is_enabled(&self) -> bool {
        self.base.is_enabled()
    }
}
