use crate::providers::LlmInstance;
use crate::{LlmResponse, LlmResult};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// An LLM provider instance with associated metrics
pub struct InstanceTracker {
    pub instance: Arc<dyn LlmInstance + Send + Sync>,
    pub last_used: Instant,
    pub response_times: Vec<Duration>,
    pub request_count: usize,
    pub error_count: usize,
}

impl InstanceTracker {
    /// Create a new LLM instance
    ///
    /// # Parameters
    /// * `id` - Unique identifier for this instance
    /// * `provider` - Reference to the provider implementation
    pub fn new(instance: Arc<dyn LlmInstance + Send + Sync>) -> Self {
        Self {
            instance: instance,
            last_used: Instant::now(),
            response_times: Vec::new(),
            request_count: 0,
            error_count: 0,
        }
    }

    /// Record the result of a request for metrics tracking
    ///
    /// # Parameters
    /// * `duration` - How long the request took
    /// * `result` - The result of the request (success or error)
    pub fn record_result(&mut self, duration: Duration, result: &LlmResult<LlmResponse>) {
        self.last_used = Instant::now();
        self.request_count += 1;

        match result {
            Ok(_) => {
                self.response_times.push(duration);
                if self.response_times.len() > 10 {
                    self.response_times.remove(0);
                }
            }
            Err(e) => {
                self.error_count += 1;
            }
        }
    }

    /// Calculate the average response time from recent requests
    ///
    /// # Returns
    /// * Average duration, or zero if no requests recorded
    pub fn avg_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            return Duration::from_millis(0);
        }
        let total: Duration = self.response_times.iter().sum();
        total / self.response_times.len().max(1) as u32 // Avoid division by zero
    }

    /// Calculate the error rate as a percentage
    ///
    /// # Returns
    /// * Error rate from 0.0 to 100.0, or 0.0 if no requests
    pub fn get_error_rate(&self) -> f64 {
        if self.request_count > 0 {
            (self.error_count as f64 / self.request_count as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if this instance is currently enabled
    ///
    /// # Returns
    /// * Whether this instance is enabled or not
    pub fn is_enabled(&self) -> bool {
        self.instance.is_enabled()
    }

    /// Check if this instance supports a specific task
    ///
    /// # Returns
    /// * Whether this instance supports this task or not
    pub fn supports_task(&self, task_name: &str) -> bool {
        self.instance.get_supported_tasks().contains_key(task_name)
    }
}
