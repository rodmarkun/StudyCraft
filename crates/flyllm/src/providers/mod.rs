/// Module for various LLM provider implementations
///
/// This module contains implementations for different LLM providers:
/// - Anthropic (Claude models)
/// - OpenAI (GPT models)
/// - Mistral AI
/// - Google (Gemini models)
/// - Ollama
///
/// Each provider implements a common interface for generating text
/// completions through their respective APIs.
pub mod anthropic;
pub mod google;
pub mod instances;
pub mod lmstudio;
pub mod mistral;
pub mod model_discovery;
pub mod ollama;
pub mod openai;
pub mod types;

pub use anthropic::AnthropicInstance;
pub use instances::{create_instance, LlmInstance};
pub use lmstudio::LmStudioInstance;
pub use model_discovery::ModelDiscovery;
pub use openai::OpenAIInstance;
pub use types::{LlmRequest, LlmResponse, Message, ModelInfo, ProviderType, TokenUsage};
