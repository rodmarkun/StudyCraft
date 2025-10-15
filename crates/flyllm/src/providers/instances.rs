use crate::errors::LlmResult;
use crate::load_balancer::tasks::TaskDefinition;
use crate::providers::anthropic::AnthropicInstance;
use crate::providers::google::GoogleInstance;
use crate::providers::lmstudio::LmStudioInstance;
use crate::providers::mistral::MistralInstance;
use crate::providers::ollama::OllamaInstance;
use crate::providers::openai::OpenAIInstance;
use crate::providers::types::{LlmRequest, LlmResponse, ProviderType};
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

/// Common interface for all LLM instances
///
/// This trait defines the interface that all LLM instances must implement
/// to be compatible with the load balancer system.
#[async_trait]
pub trait LlmInstance {
    /// Generate a completion from the LLM instance
    async fn generate(&self, request: &LlmRequest) -> LlmResult<LlmResponse>;
    /// Get the name of this instance
    fn get_name(&self) -> &str;
    /// Get the currently configured model name
    fn get_model(&self) -> &str;
    /// Get the tasks this instance supports
    fn get_supported_tasks(&self) -> &HashMap<String, TaskDefinition>;
    /// Check if this instance is enabled
    fn is_enabled(&self) -> bool;
}

/// Base instance implementation with common functionality
///
/// Handles common properties and functionality shared across all instances:
/// - HTTP client with timeout
/// - API key storage
/// - Model selection
/// - Task support
/// - Enable/disable status
pub struct BaseInstance {
    name: String,
    client: Client,
    api_key: String,
    model: String,
    supported_tasks: HashMap<String, TaskDefinition>,
    enabled: bool,
}

impl BaseInstance {
    /// Create a new Baseinstance with specified parameters
    ///
    /// # Parameters
    /// * `name` - instance name identifier
    /// * `api_key` - API key for authentication
    /// * `model` - Default model identifier to use
    /// * `supported_tasks` - Map of tasks this instance supports
    /// * `enabled` - Whether this instance is enabled
    pub fn new(
        name: String,
        api_key: String,
        model: String,
        supported_tasks: HashMap<String, TaskDefinition>,
        enabled: bool,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            name,
            client,
            api_key,
            model,
            supported_tasks,
            enabled,
        }
    }

    /// Get the HTTP client instance
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the current model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Check if this instance is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the instance name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the map of supported tasks
    pub fn supported_tasks(&self) -> &HashMap<String, TaskDefinition> {
        &self.supported_tasks
    }
}

/// Factory function to create a instance instance based on type
///
/// # Parameters
/// * `instance_type` - Which instance type to create
/// * `api_key` - API key for authentication
/// * `model` - Default model identifier
/// * `supported_tasks` - List of tasks this instance supports
/// * `enabled` - Whether this instance should be enabled
///
/// # Returns
/// * Arc-wrapped trait object implementing Llminstance
pub fn create_instance(
    instance_type: ProviderType,
    api_key: String,
    model: String,
    supported_tasks: Vec<TaskDefinition>,
    enabled: bool,
    endpoint_url: Option<String>,
) -> Arc<dyn LlmInstance + Send + Sync> {
    let supported_tasks: HashMap<String, TaskDefinition> = supported_tasks
        .into_iter()
        .map(|task| (task.name.clone(), task))
        .collect();
    match instance_type {
        ProviderType::Anthropic => Arc::new(AnthropicInstance::new(
            api_key,
            model,
            supported_tasks,
            enabled,
        )),
        ProviderType::OpenAI => Arc::new(OpenAIInstance::new(
            api_key,
            model,
            supported_tasks,
            enabled,
        )),
        ProviderType::Mistral => Arc::new(MistralInstance::new(
            api_key,
            model,
            supported_tasks,
            enabled,
        )),
        ProviderType::Google => Arc::new(GoogleInstance::new(
            api_key,
            model,
            supported_tasks,
            enabled,
        )),
        ProviderType::Ollama => Arc::new(OllamaInstance::new(
            api_key,
            model,
            supported_tasks,
            enabled,
            endpoint_url,
        )),
        ProviderType::LmStudio => Arc::new(LmStudioInstance::new(
            api_key,
            model,
            supported_tasks,
            enabled,
            endpoint_url,
        )),
    }
}
