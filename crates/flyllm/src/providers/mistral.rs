use crate::constants;
use crate::errors::{LlmError, LlmResult};
use crate::load_balancer::tasks::TaskDefinition;
use crate::providers::instances::{BaseInstance, LlmInstance};
use crate::providers::types::{LlmRequest, LlmResponse, Message, TokenUsage};

use async_trait::async_trait;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider implementation for Mistral AI's API
pub struct MistralInstance {
    base: BaseInstance,
}

/// Request structure for Mistral AI's chat completion API
#[derive(Serialize)]
struct MistralRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

/// Response structure from Mistral AI's chat completion API
#[derive(Deserialize, Debug)]
struct MistralResponse {
    id: String,
    model: String,
    object: String,
    created: u64,
    choices: Vec<MistralChoice>,
    usage: Option<MistralUsage>,
}

/// Individual choice from Mistral's response
#[derive(Deserialize, Debug)]
struct MistralChoice {
    index: u32, // Removed underscore prefix
    message: Message,
    finish_reason: Option<String>,
}

/// Token usage information from Mistral
#[derive(Deserialize, Debug)]
struct MistralUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl MistralInstance {
    /// Creates a new Mistral provider instance
    ///
    /// # Parameters
    /// * `api_key` - Mistral API key
    /// * `model` - Default model to use (e.g. "mistral-large")
    /// * `supported_tasks` - Map of tasks this provider supports
    /// * `enabled` - Whether this provider is enabled
    pub fn new(
        api_key: String,
        model: String,
        supported_tasks: HashMap<String, TaskDefinition>,
        enabled: bool,
    ) -> Self {
        let base = BaseInstance::new(
            "mistral".to_string(),
            api_key,
            model,
            supported_tasks,
            enabled,
        );
        Self { base }
    }
}

#[async_trait]
impl LlmInstance for MistralInstance {
    /// Generates a completion using Mistral AI's API
    ///
    /// # Parameters
    /// * `request` - The LLM request containing messages and parameters
    ///
    /// # Returns
    /// * `LlmResult<LlmResponse>` - The response from the model or an error
    async fn generate(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        if !self.base.is_enabled() {
            return Err(LlmError::ProviderDisabled("Mistral".to_string()));
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.base.api_key()))
                .map_err(|e| LlmError::ConfigError(format!("Invalid API key format: {}", e)))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.base.model().to_string());

        if request.messages.is_empty() {
            return Err(LlmError::ApiError(
                "Mistral requires at least one message".to_string(),
            ));
        }

        let mistral_request = MistralRequest {
            model,
            messages: request
                .messages
                .iter()
                .map(|m| Message {
                    role: match m.role.as_str() {
                        "system" | "user" | "assistant" => m.role.clone(),
                        _ => "user".to_string(),
                    },
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        let response = self
            .base
            .client()
            .post(constants::MISTRAL_API_ENDPOINT)
            .headers(headers)
            .json(&mistral_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error reading response body".to_string());
            return Err(LlmError::ApiError(format!(
                "Mistral API error ({}): {}",
                status, error_text
            )));
        }

        // Debug: Log raw response body for inspection if needed
        let response_body = response.text().await.map_err(|e| {
            LlmError::ApiError(format!("Failed to read Mistral response body: {}", e))
        })?;

        // Try to parse the response as JSON
        let mistral_response: MistralResponse =
            serde_json::from_str(&response_body).map_err(|e| {
                // Provide more context in the error message
                LlmError::ApiError(format!(
                    "Failed to parse Mistral JSON response: {}. Response body: {}",
                    e,
                    if response_body.len() > 200 {
                        format!("{}... (truncated)", &response_body[..200])
                    } else {
                        response_body.clone()
                    }
                ))
            })?;

        if mistral_response.choices.is_empty() {
            return Err(LlmError::ApiError(
                "No choices returned from Mistral".to_string(),
            ));
        }

        let choice = &mistral_response.choices[0];

        let usage = mistral_response.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LlmResponse {
            content: choice.message.content.clone(),
            model: mistral_response.model,
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
