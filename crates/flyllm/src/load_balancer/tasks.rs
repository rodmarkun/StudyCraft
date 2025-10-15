use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Definition of a task that can be routed to specific providers
///
/// Tasks represent specialized capabilities or configurations that
/// certain providers might be better suited for. Each task can have
/// associated parameters that affect how the request is processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

impl TaskDefinition {
    /// Creates a new TaskDefinition with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        TaskDefinition {
            name: name.into(),
            parameters: HashMap::new(),
        }
    }

    /// Adds or updates a parameter for this task definition.
    /// Accepts any value that can be converted into a serde_json::Value.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Sets the `max_tokens` parameter to a given value.
    pub fn with_max_tokens(self, tokens: u32) -> Self {
        self.with_param("max_tokens", json!(tokens))
    }

    /// Sets the `temperature` parameter to a given value.
    pub fn with_temperature(self, temp: f32) -> Self {
        self.with_param("temperature", json!(temp))
    }
}
