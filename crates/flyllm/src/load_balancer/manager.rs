use crate::errors::{LlmError, LlmResult};
use crate::load_balancer::builder::LlmManagerBuilder;
use crate::load_balancer::strategies;
use crate::load_balancer::tasks::TaskDefinition;
use crate::load_balancer::tracker::InstanceTracker;
use crate::load_balancer::utils::{get_debug_path, write_to_debug_file};
use crate::providers::{LlmInstance, LlmRequest, Message, TokenUsage};
use crate::{constants, create_instance, ProviderType}; // TODO - ?
use futures::future::join_all;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// User-facing request for LLM generation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerationRequest {
    pub prompt: String,                                     // Prompt for the LLM
    pub task: Option<String>,                               // Task to route for
    pub params: Option<HashMap<String, serde_json::Value>>, // Extra parameters
}

impl Default for GenerationRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            task: None,
            params: None,
        }
    }
}

impl GenerationRequest {
    // Standard Constructor
    pub fn new(prompt: String) -> Self {
        GenerationRequest {
            prompt,
            ..Default::default()
        }
    }

    /// Creates a builder for a GenerationRequest.
    pub fn builder(prompt: impl Into<String>) -> GenerationRequest {
        GenerationRequest::new(prompt.into())
    }

    /// Sets the target task for this request.
    pub fn task(mut self, name: impl Into<String>) -> Self {
        self.task = Some(name.into());
        self
    }

    /// Adds or overrides a parameter specifically for this request.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.params
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Sets max tokens for this generation in specific
    pub fn max_tokens(self, tokens: u32) -> Self {
        self.param("max_tokens", json!(tokens))
    }

    /// Finalizes the GenerationRequest
    pub fn build(self) -> Self {
        self
    }
}

/// Internal request structure with additional retry information
#[derive(Clone)]
struct LlmManagerRequest {
    pub prompt: String,
    pub task: Option<String>,
    pub params: Option<HashMap<String, serde_json::Value>>,
    pub attempts: usize,
    pub failed_instances: Vec<usize>,
}

impl LlmManagerRequest {
    /// Convert a user-facing GenerationRequest to internal format
    fn from_generation_request(request: GenerationRequest) -> Self {
        Self {
            prompt: request.prompt,
            task: request.task,
            params: request.params,
            attempts: 0,
            failed_instances: Vec::new(),
        }
    }
}

/// Response structure returned to users
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmManagerResponse {
    pub content: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Main manager for LLM providers that handles load balancing and retries
///
/// The LlmManager:
/// - Manages multiple LLM instances ( from different providers)
/// - Maps tasks to compatible instances
/// - Routes requests to appropriate instances
/// - Implements retries and fallbacks
/// - Tracks performance metrics and token usage
pub struct LlmManager {
    pub trackers: Arc<Mutex<HashMap<usize, InstanceTracker>>>, // Current instance trackers in the manager (contains the instances themselves)
    pub strategy: Arc<Mutex<Box<dyn strategies::LoadBalancingStrategy + Send + Sync>>>, // Current strategy for load balancing being used
    pub tasks_to_instances: Arc<Mutex<HashMap<String, Vec<usize>>>>, // Map of which instances handle which tasks
    pub instance_counter: Mutex<usize>, // Used for giving unique IDs to each instance in this manager
    pub max_retries: usize, // Controls how many times a failed request will be tried before giving up
    pub total_usage: Mutex<HashMap<usize, TokenUsage>>, // Token usage of each instance
    pub debug_folder: Option<PathBuf>, // Path where JSONs with debug inputs/outputs of each model will be stored
    pub creation_time: SystemTime,
}

impl LlmManager {
    /// Create a new LlmManager with default settings
    pub fn new() -> Self {
        Self {
            trackers: Arc::new(Mutex::new(HashMap::new())),
            strategy: Arc::new(Mutex::new(Box::new(
                strategies::LeastRecentlyUsedStrategy::new(),
            ))),
            tasks_to_instances: Arc::new(Mutex::new(HashMap::new())),
            instance_counter: Mutex::new(0),
            max_retries: constants::DEFAULT_MAX_TRIES,
            total_usage: Mutex::new(HashMap::new()),
            debug_folder: None,
            creation_time: SystemTime::now(),
        }
    }

    /// Creates a new builder to configure the LlmManager.
    pub fn builder() -> LlmManagerBuilder {
        LlmManagerBuilder::new()
    }

    /// Create a new LlmManager with a custom load balancing strategy
    ///
    /// # Parameters
    /// * `strategy` - The load balancing strategy to use
    pub fn new_with_strategy(
        strategy: Box<dyn strategies::LoadBalancingStrategy + Send + Sync>,
    ) -> Self {
        Self {
            trackers: Arc::new(Mutex::new(HashMap::new())),
            strategy: Arc::new(Mutex::new(strategy)),
            tasks_to_instances: Arc::new(Mutex::new(HashMap::new())),
            instance_counter: Mutex::new(0),
            max_retries: constants::DEFAULT_MAX_TRIES,
            total_usage: Mutex::new(HashMap::new()),
            debug_folder: None,
            creation_time: SystemTime::now(),
        }
    }

    /// Constructor used by the builder.
    pub fn new_with_strategy_and_retries(
        strategy: Box<dyn strategies::LoadBalancingStrategy + Send + Sync>,
        max_retries: usize,
    ) -> Self {
        Self {
            trackers: Arc::new(Mutex::new(HashMap::new())),
            strategy: Arc::new(Mutex::new(strategy)),
            tasks_to_instances: Arc::new(Mutex::new(HashMap::new())),
            instance_counter: Mutex::new(0),
            max_retries, // Use passed value
            total_usage: Mutex::new(HashMap::new()),
            debug_folder: None,
            creation_time: SystemTime::now(),
        }
    }

    /// Adds a new LLM instance by creating it from basic parameters
    ///
    /// # Parameters
    /// * `provider_type` - Which LLM provider to use (Anthropic, OpenAI, etc)
    /// * `api_key` - API key for the provider
    /// * `model` - Model identifier to use
    /// * `tasks` - List of tasks this provider supports
    /// * `enabled` - Whether this provider should be enabled
    /// * `custom_endpont` - Optional specification on where the requests for this instance should go
    pub async fn add_instance(
        &mut self,
        provider_type: ProviderType,
        api_key: String,
        model: String,
        tasks: Vec<TaskDefinition>,
        enabled: bool,
        custom_endpoint: Option<String>,
    ) {
        debug!("Creating provider with model {}", model);
        let instance = create_instance(
            provider_type,
            api_key,
            model.clone(),
            tasks.clone(),
            enabled,
            custom_endpoint,
        );
        self.add_instance_to_manager(instance).await;
        info!(
            "Added Provider Instance ({}) - Model: {} - Supports Tasks: {:?}",
            provider_type,
            model,
            tasks.iter().map(|t| t.name.as_str()).collect::<Vec<&str>>()
        );
    }

    /// Add a pre-created provider instance
    ///
    /// # Parameters
    /// * `provider` - The provider instance to add
    pub async fn add_instance_to_manager(&mut self, instance: Arc<dyn LlmInstance + Send + Sync>) {
        let id = {
            let mut counter = self.instance_counter.lock().await;
            let current_id = *counter;
            *counter += 1;
            current_id
        };

        let tracker = InstanceTracker::new(instance.clone());
        debug!("Adding instance {} ({})", id, instance.get_name());

        let supported_tasks_names: Vec<String> =
            instance.get_supported_tasks().keys().cloned().collect();

        {
            let mut task_map = self.tasks_to_instances.lock().await;
            for task_name in &supported_tasks_names {
                task_map
                    .entry(task_name.clone())
                    .or_insert_with(Vec::new)
                    .push(id);
                debug!("Added instance {} to task mapping for '{}'", id, task_name);
            }
        }

        {
            let mut trackers = self.trackers.lock().await;
            trackers.insert(id, tracker);
        }

        {
            let mut usage_map = self.total_usage.lock().await;
            usage_map.insert(id, TokenUsage::default()); // TODO - Implement default
        }
    }

    /// Set a new load balancing strategy
    ///
    /// # Parameters
    /// * `strategy` - The new load balancing strategy to use
    pub async fn set_strategy(
        &mut self,
        strategy: Box<dyn strategies::LoadBalancingStrategy + Send + Sync>,
    ) {
        let mut current_strategy = self.strategy.lock().await;
        *current_strategy = strategy;
    }

    /// Process multiple requests sequentially
    ///
    /// # Parameters
    /// * `requests` - List of generation requests to process
    ///
    /// # Returns
    /// * List of responses in the same order as the requests
    pub async fn generate_sequentially(
        &self,
        requests: Vec<GenerationRequest>,
    ) -> Vec<LlmManagerResponse> {
        let mut responses = Vec::with_capacity(requests.len());
        info!(
            "Entering generate_sequentially with {} requests",
            requests.len()
        );

        for (index, request) in requests.into_iter().enumerate() {
            info!("Starting sequential request index: {}", index);
            let internal_request = LlmManagerRequest::from_generation_request(request);

            let response_result = self.generate_response(internal_request, None).await;
            info!(
                "Sequential request index {} completed generate_response call.",
                index
            );

            let response = match response_result {
                Ok(content) => {
                    info!("Sequential request index {} succeeded.", index);
                    LlmManagerResponse {
                        content,
                        success: true,
                        error: None,
                    }
                }
                Err(e) => {
                    warn!("Sequential request index {} failed: {}", index, e);
                    LlmManagerResponse {
                        content: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    }
                }
            };

            debug!("Pushing response for sequential request index {}", index);
            responses.push(response);
            info!("Finished processing sequential request index {}", index);
        }

        info!("Exiting generate_sequentially");
        responses
    }

    /// Process multiple requests in parallel
    ///
    /// # Parameters
    /// * `requests` - List of generation requests to process
    ///
    /// # Returns
    /// * List of responses in the same order as the requests
    pub async fn batch_generate(
        &self,
        requests: Vec<GenerationRequest>,
    ) -> Vec<LlmManagerResponse> {
        info!("Entering batch_generate with {} requests", requests.len());
        let internal_requests = requests
            .into_iter()
            .map(|request| LlmManagerRequest::from_generation_request(request))
            .collect::<Vec<_>>();

        let futures = internal_requests
            .into_iter()
            .enumerate()
            .map(|(index, request)| async move {
                info!("Starting parallel request index: {}", index);
                match self.generate_response(request, None).await {
                    Ok(content) => {
                        info!("Parallel request index {} succeeded.", index);
                        LlmManagerResponse {
                            content,
                            success: true,
                            error: None,
                        }
                    }
                    Err(e) => {
                        warn!("Parallel request index {} failed: {}", index, e);
                        LlmManagerResponse {
                            content: String::new(),
                            success: false,
                            error: Some(e.to_string()),
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        let results = join_all(futures).await;
        info!("Exiting batch_generate");
        results
    }

    /// Core function to generate a response with retries
    ///
    /// # Parameters
    /// * `request` - The internal request with retry state
    /// * `max_attempts` - Optional override for maximum retry attempts
    ///
    /// # Returns
    /// * Result with either the generated content or an error
    async fn generate_response(
        &self,
        request: LlmManagerRequest,
        max_attempts: Option<usize>,
    ) -> LlmResult<String> {
        let start_time = Instant::now();
        let mut attempts = request.attempts;
        let mut failed_instances = request.failed_instances.clone();
        let prompt_preview = request.prompt.chars().take(50).collect::<String>();
        let task = request.task.as_deref();
        let request_params = request.params.clone();
        let max_retries = max_attempts.unwrap_or(self.max_retries);

        info!(
            "generate_response called for task: {:?}, prompt: '{}...'",
            task, prompt_preview
        );

        while attempts <= max_retries {
            debug!(
                "Attempt {} of {} for request (task: {:?})",
                attempts + 1,
                max_retries + 1,
                task
            );

            let attempt_result = self
                .instance_selection(
                    &request.prompt,
                    task,
                    request_params.clone(),
                    &failed_instances,
                )
                .await;

            match attempt_result {
                Ok((content, instance_id)) => {
                    let duration = start_time.elapsed();
                    info!(
                        "Request successful on attempt {} with instance {} after {:?}",
                        attempts + 1,
                        instance_id,
                        duration
                    );
                    return Ok(content);
                }
                Err((error, instance_id)) => {
                    warn!(
                        "Attempt {} failed with instance {}: {}",
                        attempts + 1,
                        instance_id,
                        error
                    );

                    // Check if this is a rate limit error
                    if matches!(error, LlmError::RateLimit(_)) {
                        warn!(
                            "Rate limit detected for instance {}. Waiting before retry...",
                            instance_id
                        );

                        // Wait before retrying (exponential backoff)
                        let wait_time =
                            std::time::Duration::from_secs(2_u64.pow(attempts as u32).min(60));
                        tokio::time::sleep(wait_time).await;

                        // Don't mark this instance as failed for rate limits
                        // Just increment attempts and try again
                        attempts += 1;
                    } else {
                        // For non-rate-limit errors, mark instance as failed
                        failed_instances.push(instance_id);
                        attempts += 1;
                    }

                    if attempts > max_retries {
                        warn!(
                            "Max retries ({}) reached for task: {:?}. Returning last error.",
                            max_retries + 1,
                            task
                        );
                        return Err(error);
                    }

                    debug!(
                        "Retrying with next eligible instance for task: {:?}...",
                        task
                    );
                }
            }
        }

        warn!("Exited retry loop unexpectedly for task: {:?}", task);
        Err(LlmError::ConfigError(
            "No available providers after all retry attempts".to_string(),
        ))
    }

    /// Select an appropriate instance and execute the request
    ///
    /// This function:
    /// 1. Identifies instances that support the requested task
    /// 2. Filters out failed and disabled instances
    /// 3. Uses the load balancing strategy to select an instance
    /// 4. Merges task and request parameters
    /// 5. Executes the request against the selected provider
    /// 6. Updates metrics based on the result
    ///
    /// # Parameters
    /// * `prompt` - The prompt text to send
    /// * `task` - Optional task identifier
    /// * `request_params` - Optional request parameters
    /// * `failed_instances` - List of instance IDs that have failed
    ///
    /// # Returns
    /// * Success: (generated content, instance ID)
    /// * Error: (error, instance ID that failed)
    async fn instance_selection(
        &self,
        prompt: &str,
        task: Option<&str>,
        request_params: Option<HashMap<String, serde_json::Value>>,
        failed_instances: &[usize],
    ) -> Result<(String, usize), (LlmError, usize)> {
        debug!(
            "instance_selection: Starting selection for task: {:?}",
            task
        );

        // 1. Get candidate instance IDs based on task (if any)
        let candidate_ids: Option<Vec<usize>> = match task {
            Some(task_name) => {
                let task_map = self.tasks_to_instances.lock().await;
                task_map.get(task_name).cloned()
            }
            None => None, // No specific task, consider all instances initially
        };

        if task.is_some() && candidate_ids.is_none() {
            warn!("No instances found supporting task: '{}'", task.unwrap());
            debug!("instance_selection returning Err (no task support)");
            return Err((
                LlmError::ConfigError(format!(
                    "No providers available for task: {}",
                    task.unwrap()
                )),
                0,
            ));
        }

        // 2. Filter candidates by availability and collect all needed data in one go
        let eligible_instances_data: Vec<(
            usize,
            String,
            Arc<dyn LlmInstance + Send + Sync>,
            Option<TaskDefinition>,
        )>;

        // Get eligible instance IDs for strategy selection
        let eligible_instance_ids: Vec<usize>;

        // Scope the lock to ensure it's dropped before strategy selection
        {
            let trackers_guard = self.trackers.lock().await;
            debug!("instance_selection: Acquired trackers lock (1st time)");

            if trackers_guard.is_empty() {
                warn!("No LLM providers configured.");
                return Err((
                    LlmError::ConfigError("No LLM providers available".to_string()),
                    0,
                ));
            }

            // Extract all the data we need while holding the lock
            match candidate_ids {
                Some(ids) => {
                    debug!(
                        "Filtering instances for task '{}' using IDs: {:?}",
                        task.unwrap(),
                        ids
                    );
                    eligible_instances_data = trackers_guard
                        .iter()
                        .filter(|(id, tracker)| {
                            ids.contains(id)
                                && tracker.is_enabled()
                                && !failed_instances.contains(id)
                        })
                        .map(|(id, tracker)| {
                            let task_def = task.and_then(|t| {
                                tracker.instance.get_supported_tasks().get(t).cloned()
                            });
                            (
                                *id,
                                tracker.instance.get_name().to_string(),
                                tracker.instance.clone(),
                                task_def,
                            )
                        })
                        .collect();
                    debug!(
                        "Found {} eligible instances for task '{}'",
                        eligible_instances_data.len(),
                        task.unwrap()
                    );
                }
                None => {
                    debug!("No specific task. Filtering all enabled instances.");
                    eligible_instances_data = trackers_guard
                        .iter()
                        .filter(|(id, tracker)| {
                            tracker.is_enabled() && !failed_instances.contains(id)
                        })
                        .map(|(id, tracker)| {
                            let task_def = task.and_then(|t| {
                                tracker.instance.get_supported_tasks().get(t).cloned()
                            });
                            (
                                *id,
                                tracker.instance.get_name().to_string(),
                                tracker.instance.clone(),
                                task_def,
                            )
                        })
                        .collect();
                    debug!(
                        "Found {} eligible instances (no task)",
                        eligible_instances_data.len()
                    );
                }
            }

            // Extract just the IDs for strategy selection
            eligible_instance_ids = eligible_instances_data
                .iter()
                .map(|(id, _, _, _)| *id)
                .collect();

            // No eligible instances check
            if eligible_instances_data.is_empty() {
                let error_msg = format!(
                    "No enabled providers available{}{}",
                    task.map_or_else(|| "".to_string(), |t| format!(" for task: '{}'", t)),
                    if !failed_instances.is_empty() {
                        format!(" (excluded {} failed instances)", failed_instances.len())
                    } else {
                        "".to_string()
                    }
                );
                warn!("{}", error_msg);
                return Err((LlmError::ConfigError(error_msg), 0));
            }
        }

        // 5. Select instance using strategy (need to re-acquire lock for metrics)
        let selected_instance_id = {
            let trackers_guard = self.trackers.lock().await;
            let mut strategy = self.strategy.lock().await;
            debug!("instance_selection: Acquired strategy and trackers locks");

            // Build the trackers slice for the strategy
            let eligible_trackers: Vec<(usize, &InstanceTracker)> = eligible_instance_ids
                .iter()
                .filter_map(|id| trackers_guard.get(id).map(|tracker| (*id, tracker)))
                .collect();

            let selected_metric_index = strategy.select_instance(&eligible_trackers);
            let selected_id = eligible_trackers[selected_metric_index].0;

            debug!("instance_selection: Released strategy lock");
            selected_id
        };

        // Find the corresponding instance in our extracted data
        let selected_instance = eligible_instances_data
            .iter()
            .find(|(id, _, _, _)| *id == selected_instance_id)
            .expect("Selected instance ID from metrics not found in eligible list - LOGIC ERROR!");

        // Unpack the tuple
        let (selected_id, selected_name, selected_provider_arc, task_def) = (
            selected_instance.0,
            &selected_instance.1,
            &selected_instance.2,
            &selected_instance.3,
        );

        debug!(
            "Selected instance {} ({}) for the request.",
            selected_id, selected_name
        );

        // 6. Merge parameters
        let mut final_params = HashMap::new();
        if let Some(task_def) = task_def {
            final_params.extend(task_def.parameters.clone());
            debug!("Applied parameters from task for instance {}", selected_id);
        }

        if let Some(req_params) = request_params {
            final_params.extend(req_params);
            debug!(
                "Applied request-specific parameters for instance {}",
                selected_id
            );
        }

        // Create and execute the request
        let max_tokens = final_params
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let temperature = final_params
            .get("temperature")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);

        let request = LlmRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            model: None, // Let provider use its configured model
            max_tokens,
            temperature,
        };

        debug!(
            "Instance {} ({}) sending request to provider...",
            selected_id, selected_name
        );
        let start_time = Instant::now();
        let result = selected_provider_arc.generate(&request).await;
        let duration = start_time.elapsed();
        info!(
            "Instance {} ({}) received result in {:?}",
            selected_id, selected_name, duration
        );

        // Update metrics regardless of success or failure
        {
            debug!("instance_selection: Attempting to acquire trackers lock (2nd time) for metrics update");
            let mut trackers_guard = self.trackers.lock().await;
            debug!("instance_selection: Acquired trackers lock (2nd time)");
            if let Some((_id, instance_tracker)) = trackers_guard
                .iter_mut()
                .find(|(id, _tracker)| **id == selected_id)
            {
                debug!("Recording result for instance {}", selected_id);
                instance_tracker.record_result(duration, &result);
                debug!("Finished recording result for instance {}", selected_id);
            } else {
                warn!(
                    "Instance {} not found for metric update after request completion.",
                    selected_id
                );
            }
            debug!("instance_selection: Releasing trackers lock (2nd time) after metrics update");
            // Lock released when trackers_guard goes out of scope here
        }

        // Write debug information if debug folder is configured
        self.write_debug_info(
            selected_id,
            selected_name,
            &selected_provider_arc.get_model(),
            prompt,
            task,
            &final_params,
            &result,
            duration,
        )
        .await;

        // Return either content or error with the instance ID
        match result {
            Ok(response) => {
                if let Some(usage) = &response.usage {
                    self.update_instance_usage(selected_id, usage).await;
                    debug!(
                        "Updated token usage for instance {}: {:?}",
                        selected_id, usage
                    );
                }
                debug!(
                    "instance_selection returning Ok for instance {}",
                    selected_id
                );
                Ok((response.content, selected_id))
            }
            Err(e) => {
                debug!(
                    "instance_selection returning Err for instance {}: {}",
                    selected_id, e
                );
                Err((e, selected_id))
            }
        }
    }

    /// Write debug information for a request/response to the debug folder
    async fn write_debug_info(
        &self,
        instance_id: usize,
        instance_name: &str,
        instance_model: &str,
        prompt: &str,
        task: Option<&str>,
        final_params: &HashMap<String, serde_json::Value>,
        result: &Result<crate::providers::LlmResponse, LlmError>,
        duration: std::time::Duration,
    ) {
        if let Some(debug_folder) = &self.debug_folder {
            let timestamp = self
                .creation_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let debug_path = get_debug_path(
                debug_folder,
                timestamp,
                instance_id,
                instance_name,
                instance_model,
            );

            // Create the new generation entry
            let generation_entry = json!({
                "metadata": {
                    "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                    "instance_id": instance_id,
                    "instance_name": instance_name,
                    "instance_model": instance_model,
                    "duration_ms": duration.as_millis()
                },
                "input": {
                    "prompt": prompt,
                    "task": task,
                    "parameters": final_params
                },
                "output": match result {
                    Ok(response) => json!({
                        "success": true,
                        "content": response.content,
                        "usage": response.usage
                    }),
                    Err(error) => json!({
                        "success": false,
                        "error": error.to_string()
                    })
                }
            });

            // Read existing file or create new array
            let mut generations: Vec<Value> = if debug_path.exists() {
                match fs::read_to_string(&debug_path) {
                    Ok(content) => match serde_json::from_str::<Vec<Value>>(&content) {
                        Ok(array) => array,
                        Err(e) => {
                            warn!("Failed to parse existing debug file as JSON array, creating new: {}", e);
                            Vec::new()
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read existing debug file, creating new: {}", e);
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };

            // Append new generation
            generations.push(generation_entry);

            // Write updated array back to file
            let json_string = match serde_json::to_string_pretty(&generations) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to serialize debug data: {}", e);
                    return;
                }
            };

            if let Err(e) = write_to_debug_file(&debug_path, &json_string) {
                warn!("Failed to write debug file: {}", e);
            }
        }
    }

    /// Update token usage for a specific instance
    ///
    /// # Parameters
    /// * `instance_id` - ID of the instance to update
    /// * `usage` - The token usage to add
    async fn update_instance_usage(&self, instance_id: usize, usage: &TokenUsage) {
        let mut usage_map = self.total_usage.lock().await;

        let instance_usage = usage_map.entry(instance_id).or_insert(TokenUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

        instance_usage.prompt_tokens += usage.prompt_tokens;
        instance_usage.completion_tokens += usage.completion_tokens;
        instance_usage.total_tokens += usage.total_tokens;

        debug!(
            "Updated usage for instance {}: current total is {} tokens",
            instance_id, instance_usage.total_tokens
        );
    }

    /// Get token usage for a specific instance
    ///
    /// # Parameters
    /// * `instance_id` - ID of the instance to query
    ///
    /// # Returns
    /// * Token usage for the specified instance, if found
    pub async fn get_instance_usage(&self, instance_id: usize) -> Option<TokenUsage> {
        let usage_map = self.total_usage.lock().await;
        usage_map.get(&instance_id).cloned()
    }

    /// Get total token usage across all instances
    ///
    /// # Returns
    /// * Combined token usage statistics
    pub async fn get_total_usage(&self) -> TokenUsage {
        let usage_map = self.total_usage.lock().await;

        usage_map.values().fold(
            TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            |mut acc, usage| {
                acc.prompt_tokens += usage.prompt_tokens;
                acc.completion_tokens += usage.completion_tokens;
                acc.total_tokens += usage.total_tokens;
                acc
            },
        )
    }

    /// Get the number of configured provider instances
    ///
    /// # Returns
    /// * Number of provider instances in the manager
    pub async fn get_provider_count(&self) -> usize {
        let trackers = self.trackers.lock().await;
        trackers.len()
    }

    /// Print token usage statistics to console
    pub async fn print_token_usage(&self) {
        println!("\n--- Token Usage Statistics ---");
        println!(
            "{:<5} {:<15} {:<30} {:<15} {:<15} {:<15}",
            "ID", "Provider", "Model", "Prompt Tokens", "Completion Tokens", "Total Tokens"
        );
        println!("{}", "-".repeat(95));

        let trackers = self.trackers.lock().await;
        let usage_map = self.total_usage.lock().await;

        // Print usage for each instance
        for (instance_id, tracker) in trackers.iter() {
            if let Some(usage) = usage_map.get(instance_id) {
                println!(
                    "{:<5} {:<15} {:<30} {:<15} {:<15} {:<15}",
                    instance_id,
                    tracker.instance.get_name(),
                    tracker.instance.get_model(),
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.total_tokens
                );
            }
        }
    }
}
