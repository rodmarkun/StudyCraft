use std::collections::HashMap;

use crate::constants;
use crate::errors::{LlmError, LlmResult};
use crate::load_balancer::tasks::TaskDefinition;
use crate::providers::instances::{BaseInstance, LlmInstance};
use crate::providers::types::{LlmRequest, LlmResponse, Message, TokenUsage};

use async_trait::async_trait;
use reqwest::header;
use serde::{Deserialize, Serialize};

/// Provider implementation for LM Studio using its OpenAI-compatible API
pub struct LmStudioInstance {
    base: BaseInstance,
    chat_endpoint: String,
}

impl LmStudioInstance {
    pub fn new(
        api_key: String,
        model: String,
        supported_tasks: HashMap<String, TaskDefinition>,
        enabled: bool,
        endpoint_url: Option<String>,
    ) -> Self {
        let chat_endpoint = Self::build_chat_endpoint(endpoint_url.clone());
        let base = BaseInstance::new(
            "lmstudio".to_string(),
            api_key,
            model,
            supported_tasks,
            enabled,
        );

        Self {
            base,
            chat_endpoint,
        }
    }

    fn build_chat_endpoint(endpoint_url: Option<String>) -> String {
        let base = endpoint_url.unwrap_or_else(|| constants::LM_STUDIO_API_ENDPOINT.to_string());
        let trimmed = base.trim_end_matches('/');
        if trimmed.ends_with("/v1/chat/completions") {
            trimmed.to_string()
        } else if trimmed.ends_with("/v1") {
            format!("{}/chat/completions", trimmed)
        } else {
            format!("{}/v1/chat/completions", trimmed)
        }
    }
}

#[derive(Serialize)]
struct LmStudioRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct LmStudioResponse {
    choices: Vec<LmStudioChoice>,
    model: String,
    usage: Option<LmStudioUsage>,
}

#[derive(Deserialize)]
struct LmStudioChoice {
    message: Message,
}

#[derive(Deserialize)]
struct LmStudioUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LlmInstance for LmStudioInstance {
    async fn generate(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        if !self.base.is_enabled() {
            return Err(LlmError::ProviderDisabled("LmStudio".to_string()));
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        if !self.base.api_key().is_empty() {
            if let Ok(value) =
                header::HeaderValue::from_str(&format!("Bearer {}", self.base.api_key()))
            {
                headers.insert(header::AUTHORIZATION, value);
            }
        }

        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.base.model().to_string());

        let payload = LmStudioRequest {
            model,
            messages: request.messages.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let response = self
            .base
            .client()
            .post(&self.chat_endpoint)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "LmStudio API error ({}): {}",
                status, error_text
            )));
        }

        let parsed: LmStudioResponse = response.json().await?;
        let choice = parsed
            .choices
            .get(0)
            .ok_or_else(|| LlmError::ApiError("LmStudio returned no choices".to_string()))?;

        let usage = parsed.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LlmResponse {
            content: choice.message.content.clone(),
            model: parsed.model,
            usage,
        })
    }

    fn get_name(&self) -> &str {
        self.base.name()
    }

    fn get_model(&self) -> &str {
        self.base.model()
    }

    fn get_supported_tasks(&self) -> &HashMap<String, TaskDefinition> {
        &self.base.supported_tasks()
    }

    fn is_enabled(&self) -> bool {
        self.base.is_enabled()
    }
}
