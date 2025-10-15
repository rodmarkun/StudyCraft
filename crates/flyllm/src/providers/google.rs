use crate::constants;
use crate::errors::{LlmError, LlmResult};
use crate::load_balancer::tasks::TaskDefinition;
use crate::providers::instances::{BaseInstance, LlmInstance};
use crate::providers::types::{LlmRequest, LlmResponse, Message, TokenUsage};

use async_trait::async_trait;
use log::debug;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider implementation for Google's Gemini AI models
pub struct GoogleInstance {
    base: BaseInstance,
}

/// Request structure for Google's Gemini API
#[derive(Serialize)]
struct GoogleGenerateContentRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "generationConfig")]
    generation_config: Option<GoogleGenerationConfig>,
}

/// Content structure for Google's Gemini API messages
#[derive(Serialize, Deserialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

/// Individual content part for Google's Gemini API
#[derive(Serialize, Deserialize)]
struct GooglePart {
    text: String,
}

/// Generation configuration for Google's Gemini API
#[derive(Serialize, Default)]
struct GoogleGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // top_k: Option<u32>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // stop_sequences: Option<Vec<String>>,
}

/// Response structure from Google's Gemini API
#[derive(Deserialize)]
struct GoogleGenerateContentResponse {
    candidates: Vec<GoogleCandidate>,
}

/// Individual candidate from Google's Gemini API response
#[derive(Deserialize)]
struct GoogleCandidate {
    content: GoogleContent,
    #[serde(rename = "tokenCount")]
    #[serde(default)]
    token_count: u32, // Note: Google provides total token count here
                      // safety_ratings: Vec<SafetyRating>, // We don't use this currently
}

impl GoogleInstance {
    /// Creates a new Google provider instance
    ///
    /// # Parameters
    /// * `api_key` - Google API key
    /// * `model` - Default model to use (e.g. "gemini-pro")
    /// * `supported_tasks` - Map of tasks this provider supports
    /// * `enabled` - Whether this provider is enabled
    pub fn new(
        api_key: String,
        model: String,
        supported_tasks: HashMap<String, TaskDefinition>,
        enabled: bool,
    ) -> Self {
        let base = BaseInstance::new(
            "google".to_string(),
            api_key,
            model,
            supported_tasks,
            enabled,
        );
        Self { base }
    }

    /// Maps standard message format to Google's expected format
    ///
    /// This function handles several Google-specific requirements:
    /// - Converts "assistant" role to "model" role
    /// - Prepends system messages to the first user message
    /// - Validates that the first message is from the user
    ///
    /// # Parameters
    /// * `messages` - Array of messages in our standard format
    ///
    /// # Returns
    /// * `LlmResult<Vec<GoogleContent>>` - Mapped contents or an error
    fn map_messages_to_contents(messages: &[Message]) -> LlmResult<Vec<GoogleContent>> {
        let mut contents = Vec::new();
        let mut system_prompt: Option<String> = None;
        let mut first_user_message_index: Option<usize> = None;
        for (_, msg) in messages.iter().enumerate() {
            match msg.role.as_str() {
                "system" => {
                    if system_prompt.is_some() {
                        return Err(LlmError::ApiError("Multiple system messages are not supported by Google provider mapping.".to_string()));
                    }
                    system_prompt = Some(msg.content.clone());
                }
                "user" | "model" | "assistant" => {
                    let role = if msg.role == "assistant" {
                        "model"
                    } else {
                        &msg.role
                    };
                    if role == "user" && first_user_message_index.is_none() {
                        first_user_message_index = Some(contents.len());
                    }
                    contents.push(GoogleContent {
                        role: role.to_string(),
                        parts: vec![GooglePart {
                            text: msg.content.clone(),
                        }],
                    });
                }
                _ => {
                    log::warn!("Ignoring message with unknown role: {}", msg.role);
                }
            }
        }

        if let Some(sys_prompt) = &system_prompt {
            if let Some(user_idx) = first_user_message_index {
                if let Some(user_content) = contents.get_mut(user_idx) {
                    if let Some(part) = user_content.parts.get_mut(0) {
                        part.text = format!("{}\n\n{}", sys_prompt, part.text);
                    }
                } else {
                    return Err(LlmError::ApiError(
                        "System message provided but no user message found.".to_string(),
                    ));
                }
            } else {
                return Err(LlmError::ApiError(
                    "System message provided but no user message found.".to_string(),
                ));
            }
        }

        if contents.is_empty() {
            return Err(LlmError::ApiError(
                "No valid messages found for Google provider.".to_string(),
            ));
        }
        if contents[0].role != "user" {
            return Err(LlmError::ApiError(format!(
                "Google chat must start with a 'user' role message, found '{}'.",
                contents[0].role
            )));
        }
        Ok(contents)
    }
}

#[async_trait]
impl LlmInstance for GoogleInstance {
    /// Generates a completion using Google's Gemini API
    ///
    /// # Parameters
    /// * `request` - The LLM request containing messages and parameters
    ///
    /// # Returns
    /// * `LlmResult<LlmResponse>` - The response from the model or an error
    async fn generate(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        if !self.base.is_enabled() {
            return Err(LlmError::ProviderDisabled("Google".to_string()));
        }

        let model_name = self.base.model();
        let api_key = self.base.api_key();

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            constants::GOOGLE_API_ENDPOINT_PREFIX,
            model_name,
            api_key
        );

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let contents = Self::map_messages_to_contents(&request.messages)?;

        let mut generation_config = GoogleGenerationConfig::default();
        generation_config.temperature = request.temperature;
        generation_config.max_output_tokens = request.max_tokens;

        let google_request = GoogleGenerateContentRequest {
            contents,
            generation_config: Some(generation_config)
                .filter(|gc| gc.temperature.is_some() || gc.max_output_tokens.is_some()),
        };

        let response = self
            .base
            .client()
            .post(&url)
            .headers(headers)
            .json(&google_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_json: Result<serde_json::Value, _> = response.json().await;
            let error_details = match error_json {
                Ok(json) => json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("Unknown error structure: {}", json)),
                Err(_) => "Failed to parse error response body".to_string(),
            };

            return Err(LlmError::ApiError(format!(
                "Google API error ({}): {}",
                status, error_details
            )));
        }

        let google_response: GoogleGenerateContentResponse =
            response.json().await.map_err(|e| {
                LlmError::ApiError(format!("Failed to parse Google JSON response: {}", e))
            })?;

        if google_response.candidates.is_empty() {
            return Err(LlmError::ApiError(
                "No candidates returned from Google. Content may have been blocked.".to_string(),
            ));
        }

        let candidate = &google_response.candidates[0];

        let combined_content = candidate
            .content
            .parts
            .iter()
            .map(|part| part.text.clone())
            .collect::<Vec<String>>()
            .join("");

        let usage = if candidate.token_count > 0 {
            // Simply use the token count as the total
            Some(TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: candidate.token_count,
            })
        } else {
            None
        };

        debug!("Google usage: {:?}", usage);

        Ok(LlmResponse {
            content: combined_content,
            model: model_name.to_string(),
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
