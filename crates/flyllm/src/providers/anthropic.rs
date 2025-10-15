use crate::constants;
use crate::errors::{LlmError, LlmResult};
use crate::load_balancer::tasks::TaskDefinition;
use crate::providers::instances::{BaseInstance, LlmInstance};
use crate::providers::types::{LlmRequest, LlmResponse, TokenUsage};

use async_trait::async_trait;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider implementation for Anthropic's Claude API
pub struct AnthropicInstance {
    base: BaseInstance,
}

/// Request structure for the Anthropic Claude API
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Individual message structure for Anthropic's API
#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Response structure from Anthropic's Claude API
#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    model: String,
    usage: Option<AnthropicUsage>,
}

/// Content block from Anthropic's response
#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

/// Token usage information from Anthropic
#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicInstance {
    /// Creates a new Anthropic provider instance
    ///
    /// # Parameters
    /// * `api_key` - Anthropic API key
    /// * `model` - Default model to use (e.g. "claude-3-opus-20240229")
    /// * `supported_tasks` - Map of tasks this provider supports
    /// * `enabled` - Whether this provider is enabled
    pub fn new(
        api_key: String,
        model: String,
        supported_tasks: HashMap<String, TaskDefinition>,
        enabled: bool,
    ) -> Self {
        let base = BaseInstance::new(
            "anthropic".to_string(),
            api_key,
            model,
            supported_tasks,
            enabled,
        );
        Self { base }
    }
}

#[async_trait]
impl LlmInstance for AnthropicInstance {
    /// Generates a completion using Anthropic's Claude API
    ///
    /// # Parameters
    /// * `request` - The LLM request containing messages and parameters
    ///
    /// # Returns
    /// * `LlmResult<LlmResponse>` - The response from the model or an error
    async fn generate(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        if !self.base.is_enabled() {
            return Err(LlmError::ProviderDisabled("Anthropic".to_string()));
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(self.base.api_key())
                .map_err(|e| LlmError::ConfigError(format!("Invalid API key format: {}", e)))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_static(constants::ANTHROPIC_API_VERSION),
        );

        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.base.model().to_string());

        // Extract system message and regular messages
        let mut system_content = None;
        let mut regular_messages = Vec::new();

        for msg in &request.messages {
            if msg.role == "system" {
                system_content = Some(msg.content.clone());
            } else {
                regular_messages.push(AnthropicMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }

        // Ensure we have at least one message
        if regular_messages.is_empty() && system_content.is_some() {
            regular_messages.push(AnthropicMessage {
                role: "user".to_string(),
                content: format!("Using this context: {}", system_content.unwrap()),
            });
            system_content = None;
        }

        if regular_messages.is_empty() {
            return Err(LlmError::ApiError(
                "Anthropic requires at least one message".to_string(),
            ));
        }

        let anthropic_request = AnthropicRequest {
            model,
            system: system_content,
            messages: regular_messages,
            max_tokens: request.max_tokens.unwrap_or(constants::DEFAULT_MAX_TOKENS),
            temperature: request.temperature,
        };

        let response = self
            .base
            .client()
            .post(constants::ANTHROPIC_API_ENDPOINT)
            .headers(headers)
            .json(&anthropic_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Anthropic API error: {}",
                error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        if anthropic_response.content.is_empty() {
            return Err(LlmError::ApiError("No response from Anthropic".to_string()));
        }

        let usage = anthropic_response.usage.map(|u| TokenUsage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        let text = anthropic_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .collect::<Vec<String>>()
            .join("");

        Ok(LlmResponse {
            content: text,
            model: anthropic_response.model,
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
        self.base.supported_tasks()
    }

    /// Returns whether this provider is enabled
    fn is_enabled(&self) -> bool {
        self.base.is_enabled()
    }
}
