pub mod builder;
pub mod manager;
pub mod strategies;
pub mod tasks;
/// Load balancer module for distributing requests across multiple LLM providers
///
/// This module contains components for:
/// - Managing provider instances with metrics tracking
/// - Selecting appropriate providers based on tasks and load
/// - Implementing different load balancing strategies
/// - Handling retries and fallbacks when providers fail
/// - Tracking token usage across providers
pub mod tracker;
pub mod utils;

pub use manager::{GenerationRequest, LlmManager, LlmManagerResponse};
pub use tasks::TaskDefinition;
