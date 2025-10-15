use serde::{Deserialize, Serialize};

/// Enum representing the different LLM providers supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ProviderType {
    Anthropic,
    OpenAI,
    Mistral,
    Google,
    Ollama,
    LmStudio,
}

/// Unified request structure used across all providers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Standard message format used across providers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Unified response structure returned by all providers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

/// Token usage information returned by providers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

/// Information about an LLM model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub provider: ProviderType,
}

/// Display implementation for ProviderType
impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Anthropic => write!(f, "Anthropic"),
            ProviderType::OpenAI => write!(f, "OpenAI"),
            ProviderType::Mistral => write!(f, "Mistral"),
            ProviderType::Google => write!(f, "Google"),
            ProviderType::Ollama => write!(f, "Ollama"),
            ProviderType::LmStudio => write!(f, "LmStudio"),
        }
    }
}

impl From<&str> for ProviderType {
    fn from(value: &str) -> Self {
        match value {
            "Anthropic" => ProviderType::Anthropic,
            "OpenAI" => ProviderType::OpenAI,
            "Mistral" => ProviderType::Mistral,
            "Google" => ProviderType::Google,
            "Ollama" => ProviderType::Ollama,
            "LmStudio" => ProviderType::LmStudio,
            _ => panic!("Unknown provider: {}", value),
        }
    }
}
