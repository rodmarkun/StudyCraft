//! FlyLLM is a Rust library that provides a load-balanced, multi-provider client for Large Language Models.
//!
//! It enables developers to seamlessly work with multiple LLM providers (OpenAI, Anthropic, Google, Mistral...)
//! through a unified API with request routing, load balancing, and failure handling.
//!
//! # Features
//!
//! - **Multi-provider support**: Integrate with OpenAI, Anthropic, Google, and Mistral
//! - **Load balancing**: Distribute requests across multiple providers
//! - **Automatic retries**: Handle provider failures with configurable retry policies
//! - **Task routing**: Route specific tasks to the most suitable providers
//! - **Metrics tracking**: Monitor response times, error rates, and token usage
//!
//! # Example
//!
//! ```ignore
//! use flyllm::{LlmManager, ProviderType, GenerationRequest, TaskDefinition};
//!
//! async fn example() {
//!     // Create a manager
//!     let mut manager = LlmManager::new();
//!     
//!     // Add providers
//!     manager.add_provider(
//!         ProviderType::OpenAI,
//!         "api-key".to_string(),
//!         "gpt-4-turbo".to_string(),
//!         vec![],
//!         true
//!     );
//!     
//!     // Generate a response
//!     let request = GenerationRequest {
//!         prompt: "Explain Rust in one paragraph".to_string(),
//!         task: None,
//!         params: None,
//!     };
//!     
//!     let responses = manager.generate_sequentially(vec![request]).await;
//!     println!("{}", responses[0].content);
//! }
//! ```

pub mod constants;
pub mod errors;
pub mod load_balancer;
pub mod providers;

pub use providers::{
    create_instance, AnthropicInstance, LlmInstance, LlmRequest, LlmResponse, ModelDiscovery,
    ModelInfo, OpenAIInstance, ProviderType,
};

pub use errors::{LlmError, LlmResult};

pub use load_balancer::{GenerationRequest, LlmManager, LlmManagerResponse, TaskDefinition};

/// Initialize the logging system
///
/// This should be called at the start of your application in case
/// you want to activate the library's debug and info logging.
pub fn use_logging() {
    env_logger::init();
}
