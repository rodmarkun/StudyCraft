use super::LlmManager;
use crate::errors::{LlmError, LlmResult};
use crate::load_balancer::strategies::{LeastRecentlyUsedStrategy, LoadBalancingStrategy};
use crate::load_balancer::tasks::TaskDefinition;
use crate::{constants, ProviderType};
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;

/// Internal helper struct for Builder
#[derive(Clone)]
struct ProviderConfig {
    provider_type: ProviderType,
    api_key: String,
    model: String,
    supported_task_names: Vec<String>,
    enabled: bool,
    custom_endpoint: Option<String>,
}

/// LlmManager Builder
pub struct LlmManagerBuilder {
    defined_tasks: HashMap<String, TaskDefinition>,
    providers_to_build: Vec<ProviderConfig>,
    strategy: Box<dyn LoadBalancingStrategy + Send + Sync>,
    max_retries: usize,
    debug_folder: Option<PathBuf>,
}

impl LlmManagerBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        LlmManagerBuilder {
            defined_tasks: HashMap::new(),
            providers_to_build: Vec::new(),
            strategy: Box::new(LeastRecentlyUsedStrategy::new()), // Default strategy
            max_retries: constants::DEFAULT_MAX_TRIES,            // Default retries
            debug_folder: None,
        }
    }

    /// Defines a task that providers can later reference by name.
    pub fn define_task(mut self, task_def: TaskDefinition) -> Self {
        self.defined_tasks.insert(task_def.name.clone(), task_def);
        self
    }

    /// Sets the load balancing strategy for the manager.
    pub fn strategy(mut self, strategy: Box<dyn LoadBalancingStrategy + Send + Sync>) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets the maximum number of retries for failed requests.
    pub fn max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Begins configuring a new provider instance.
    /// Subsequent calls like `.supports()`, `.enabled()`, `.custom_endpoint()` will apply to this provider.
    pub fn add_instance(
        mut self,
        provider_type: ProviderType,
        model: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        let config = ProviderConfig {
            provider_type,
            api_key: api_key.into(),
            model: model.into(),
            supported_task_names: Vec::new(),
            enabled: true, // Default to enabled
            custom_endpoint: None,
        };
        self.providers_to_build.push(config);
        self // Return self to allow chaining provider configurations
    }

    /// Specifies that the *last added* provider supports the task with the given name.
    /// Panics if `add_instance` was not called before this.
    pub fn supports(mut self, task_name: &str) -> Self {
        match self.providers_to_build.last_mut() {
            Some(last_provider) => {
                if !self.defined_tasks.contains_key(task_name) {
                    // Optional: Warn or error early if task isn't defined yet
                    log::warn!("Provider configured to support task '{}' which has not been defined yet with define_task().", task_name);
                }
                last_provider
                    .supported_task_names
                    .push(task_name.to_string());
            }
            None => {
                panic!("'.supports()' called before '.add_instance()'");
            }
        }
        self
    }

    /// Specifies that the *last added* provider supports multiple tasks by name.
    /// Panics if `add_provider` was not called before this.
    pub fn supports_many(mut self, task_names: &[&str]) -> Self {
        match self.providers_to_build.last_mut() {
            Some(last_provider) => {
                for task_name in task_names {
                    if !self.defined_tasks.contains_key(*task_name) {
                        log::warn!("Provider configured to support task '{}' which has not been defined yet with define_task().", task_name);
                    }
                    last_provider
                        .supported_task_names
                        .push(task_name.to_string());
                }
            }
            None => {
                panic!("'.supports_many()' called before '.add_provider()'");
            }
        }
        self
    }

    /// Sets the enabled status for the *last added* provider.
    /// Panics if `add_provider` was not called before this.
    pub fn enabled(mut self, enabled: bool) -> Self {
        match self.providers_to_build.last_mut() {
            Some(last_provider) => {
                last_provider.enabled = enabled;
            }
            None => {
                panic!("'.enabled()' called before '.add_provider()'");
            }
        }
        self
    }

    pub fn debug_folder(mut self, path: impl Into<PathBuf>) -> Self {
        self.debug_folder = Some(path.into());
        self
    }

    /// Sets a custom endpoint for the *last added* provider.
    /// Panics if `add_provider` was not called before this.
    pub fn custom_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        match self.providers_to_build.last_mut() {
            Some(last_provider) => {
                last_provider.custom_endpoint = Some(endpoint.into());
            }
            None => {
                panic!("'.custom_endpoint()' called before '.add_provider()'");
            }
        }
        self
    }

    /// Consumes the builder and constructs the `LlmManager`.
    /// Returns an error if a referenced task was not defined.
    pub async fn build(self) -> LlmResult<LlmManager> {
        let mut manager =
            LlmManager::new_with_strategy_and_retries(self.strategy, self.max_retries);

        // Set debug folder if specified
        manager.debug_folder = self.debug_folder;

        for provider_config in self.providers_to_build {
            // Resolve TaskDefinition structs from names
            let mut provider_tasks: Vec<TaskDefinition> = Vec::new();
            for task_name in &provider_config.supported_task_names {
                match self.defined_tasks.get(task_name) {
                    Some(task_def) => provider_tasks.push(task_def.clone()),
                    None => return Err(LlmError::ConfigError(format!(
                        "Build failed: Task '{}' referenced by provider '{}' ({}) was not defined using define_task()",
                        task_name, provider_config.provider_type, provider_config.model
                    ))),
                }
            }

            manager
                .add_instance(
                    provider_config.provider_type,
                    provider_config.api_key,
                    provider_config.model.clone(),
                    provider_tasks,
                    provider_config.enabled,
                    provider_config.custom_endpoint,
                )
                .await;
            debug!(
                "Built and added provider: {} ({})",
                provider_config.provider_type, provider_config.model
            );
        }

        // Check if the manager has instances
        let trackers = manager.trackers.lock().await;
        let is_empty = trackers.is_empty();
        drop(trackers);

        if is_empty {
            log::warn!("LlmManager built with no provider instances.");
        }

        Ok(manager)
    }
}
