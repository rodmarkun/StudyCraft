use crate::config::APP_PATHS;
use crate::constants;
use crate::errors::{AppError, AppResult};
use crate::services::llm_service::agents::AgentType;
use flyllm::ProviderType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudySettings {
    /// Learning steps in minutes for flashcard reviews (default [1, 10])
    pub learning_steps: Vec<i64>,
    /// Starting ease factor for new cards (default 2.5)
    pub starting_ease_factor: f64,
    /// Maximum ease factor (default 2.5)
    pub max_ease_factor: f64,
    /// Minimum ease factor (default 1.3)
    pub min_ease_factor: f64,
    /// Easy interval for new cards (default 4 days)
    pub easy_interval_new: i32,
    /// Easy interval for learning cards (default 4 days)
    pub easy_interval_learning: i32,
    /// Hard interval multiplier (default 1.2)
    pub hard_interval_multiplier: f64,
    /// Easy interval multiplier (default 1.3)
    pub easy_interval_multiplier: f64,
    /// Maximum interval in days (default 365)
    pub max_interval_days: i32,
    /// Maximum number of chunks to recover from a vectorial search (default 5)
    pub max_chunks_to_recover_from_search: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentSettings {
    pub agent_prompts: HashMap<AgentType, String>,
    pub agent_models_per_provider: HashMap<AgentType, HashMap<ProviderType, String>>,
    pub agent_max_tokens: HashMap<AgentType, u32>,
    pub agent_temperature: HashMap<AgentType, f32>,
}

impl Default for StudySettings {
    fn default() -> Self {
        Self {
            learning_steps: vec![1, 10],
            starting_ease_factor: 2.5,
            max_ease_factor: 2.5,
            min_ease_factor: 1.3,
            easy_interval_new: 4,
            easy_interval_learning: 4,
            hard_interval_multiplier: 1.2,
            easy_interval_multiplier: 1.3,
            max_interval_days: 365,
            max_chunks_to_recover_from_search: 5,
        }
    }
}

impl Default for AgentSettings {
    fn default() -> Self {
        let mut agent_prompts = HashMap::new();
        let mut agent_models_per_provider = HashMap::new();
        let mut agent_max_tokens = HashMap::new();
        let mut agent_temperature = HashMap::new();

        for agent_type in AgentType::all() {
            agent_prompts.insert(
                agent_type.clone(),
                agent_type.get_prompt_template().to_string(),
            );
            let task_def = agent_type.get_task_definition();
            agent_max_tokens.insert(
                agent_type.clone(),
                task_def
                    .parameters
                    .get("max_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3000) as u32,
            );

            agent_temperature.insert(
                agent_type.clone(),
                task_def
                    .parameters
                    .get("temperature")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.3) as f32,
            );

            let mut models_for_agent = HashMap::new();
            for provider_type in constants::ALL_PROVIDER_TYPES.iter() {
                models_for_agent.insert(
                    provider_type.clone(),
                    constants::get_default_agent_model(&agent_type, provider_type).to_string(),
                );
            }
            agent_models_per_provider.insert(agent_type.clone(), models_for_agent);
        }

        Self {
            agent_prompts,
            agent_models_per_provider,
            agent_max_tokens,
            agent_temperature,
        }
    }
}

impl StudySettings {
    pub fn load() -> AppResult<Self> {
        let settings_path = APP_PATHS
            .config_dir
            .join(constants::GENERAL_SETTINGS_FILE_NAME);

        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path).map_err(AppError::IoError)?;

            match serde_json::from_str::<Self>(&content) {
                Ok(settings) => {
                    // Validate settings and apply constraints
                    let validated = settings.validate_and_fix();
                    Ok(validated)
                }
                Err(e) => {
                    eprintln!(
                        "Failed to parse general settings file, using defaults: {}",
                        e
                    );
                    Ok(Self::default())
                }
            }
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> AppResult<()> {
        APP_PATHS.ensure_dirs_exist().map_err(AppError::IoError)?;

        let settings_path = APP_PATHS
            .config_dir
            .join(constants::GENERAL_SETTINGS_FILE_NAME);
        let validated = self.validate_and_fix();

        let content = serde_json::to_string_pretty(&validated).map_err(|e| {
            AppError::ConfigError(format!("Failed to serialize general settings: {}", e))
        })?;

        fs::write(&settings_path, content).map_err(AppError::IoError)?;

        Ok(())
    }

    /// Validate settings and fix any invalid values
    fn validate_and_fix(&self) -> Self {
        let mut fixed = self.clone();

        // Validate learning steps
        if fixed.learning_steps.is_empty() {
            fixed.learning_steps = vec![1, 10];
        } else {
            // positive values
            fixed.learning_steps = fixed
                .learning_steps
                .iter()
                .map(|&step| step.max(1))
                .collect();
        }

        // Validate ease factors
        fixed.starting_ease_factor = fixed.starting_ease_factor.max(1.0).min(5.0);
        fixed.max_ease_factor = fixed.max_ease_factor.max(fixed.min_ease_factor).min(5.0);
        fixed.min_ease_factor = fixed.min_ease_factor.max(1.0).min(fixed.max_ease_factor);

        // Validate intervals
        fixed.easy_interval_new = fixed.easy_interval_new.max(1).min(365);
        fixed.easy_interval_learning = fixed.easy_interval_learning.max(1).min(365);
        fixed.max_interval_days = fixed.max_interval_days.max(1).min(36500);

        // Validate multipliers
        fixed.hard_interval_multiplier = fixed.hard_interval_multiplier.max(1.0).min(2.0);
        fixed.easy_interval_multiplier = fixed.easy_interval_multiplier.max(1.0).min(3.0);

        // Validate vector settings
        fixed.max_chunks_to_recover_from_search =
            fixed.max_chunks_to_recover_from_search.max(1).min(50);

        fixed
    }

    pub fn reset_to_defaults(&mut self) -> AppResult<()> {
        *self = Self::default();
        self.save()
    }

    /// Get learning steps as a comma-separated string for UI display
    pub fn learning_steps_as_string(&self) -> String {
        self.learning_steps
            .iter()
            .map(|step| step.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Set learning steps from a comma-separated string
    pub fn set_learning_steps_from_string(&mut self, steps_str: &str) -> AppResult<()> {
        let steps: Result<Vec<i64>, _> = steps_str
            .split(',')
            .map(|s| s.trim().parse::<i64>())
            .collect();

        match steps {
            Ok(parsed_steps) => {
                if parsed_steps.is_empty() {
                    return Err(AppError::ValidationError(
                        "Learning steps cannot be empty".to_string(),
                    ));
                }

                if parsed_steps.iter().any(|&step| step <= 0) {
                    return Err(AppError::ValidationError(
                        "All learning steps must be positive".to_string(),
                    ));
                }

                self.learning_steps = parsed_steps;
                Ok(())
            }
            Err(_) => Err(AppError::ValidationError(
                "Invalid learning steps format. Use comma-separated positive numbers.".to_string(),
            )),
        }
    }

    /// Update a specific setting by name
    pub fn update_setting(&mut self, setting_name: &str, value: &str) -> AppResult<()> {
        match setting_name {
            "learning_steps" => self.set_learning_steps_from_string(value)?,
            "starting_ease_factor" => {
                self.starting_ease_factor = value.parse::<f64>().map_err(|_| {
                    AppError::ValidationError("Invalid ease factor value".to_string())
                })?;
            }
            "max_ease_factor" => {
                self.max_ease_factor = value.parse::<f64>().map_err(|_| {
                    AppError::ValidationError("Invalid max ease factor value".to_string())
                })?;
            }
            "min_ease_factor" => {
                self.min_ease_factor = value.parse::<f64>().map_err(|_| {
                    AppError::ValidationError("Invalid min ease factor value".to_string())
                })?;
            }
            "easy_interval_new" => {
                self.easy_interval_new = value.parse::<i32>().map_err(|_| {
                    AppError::ValidationError("Invalid easy interval value".to_string())
                })?;
            }
            "easy_interval_learning" => {
                self.easy_interval_learning = value.parse::<i32>().map_err(|_| {
                    AppError::ValidationError("Invalid easy learning interval value".to_string())
                })?;
            }
            "hard_interval_multiplier" => {
                self.hard_interval_multiplier = value.parse::<f64>().map_err(|_| {
                    AppError::ValidationError("Invalid hard multiplier value".to_string())
                })?;
            }
            "easy_interval_multiplier" => {
                self.easy_interval_multiplier = value.parse::<f64>().map_err(|_| {
                    AppError::ValidationError("Invalid easy multiplier value".to_string())
                })?;
            }
            "max_interval_days" => {
                self.max_interval_days = value.parse::<i32>().map_err(|_| {
                    AppError::ValidationError("Invalid max interval value".to_string())
                })?;
            }
            "max_chunks_to_recover_from_search" => {
                self.max_chunks_to_recover_from_search = value.parse::<u32>().map_err(|_| {
                    AppError::ValidationError("Invalid max chunks value".to_string())
                })?;
            }
            _ => {
                return Err(AppError::ValidationError(format!(
                    "Unknown setting: {}",
                    setting_name
                )))
            }
        }

        self.save()
    }
}

impl AgentSettings {
    /// Load agent settings from the configuration file, or return defaults.
    pub fn load() -> AppResult<Self> {
        let settings_path = APP_PATHS
            .config_dir
            .join(constants::AGENT_SETTINGS_FILE_NAME);

        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path).map_err(AppError::IoError)?;
            match serde_json::from_str::<Self>(&content) {
                Ok(settings) => Ok(settings),
                Err(e) => {
                    eprintln!("Failed to parse agent settings file, using defaults: {}", e);
                    Ok(Self::default())
                }
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Save the current agent settings to the configuration file.
    pub fn save(&self) -> AppResult<()> {
        APP_PATHS.ensure_dirs_exist().map_err(AppError::IoError)?;
        let settings_path = APP_PATHS
            .config_dir
            .join(constants::AGENT_SETTINGS_FILE_NAME);
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            AppError::ConfigError(format!("Failed to serialize agent settings: {}", e))
        })?;
        fs::write(&settings_path, content).map_err(AppError::IoError)?;
        Ok(())
    }

    /// Update a specific agent setting by name.
    pub fn update_setting(&mut self, setting_name: &str, value: String) -> AppResult<()> {
        let parts: Vec<&str> = setting_name.split('.').collect();
        if parts.is_empty() {
            return Err(AppError::ValidationError(
                "Invalid setting name".to_string(),
            ));
        }

        let setting_type = parts[0];
        let agent_type_str = parts
            .get(1)
            .ok_or_else(|| AppError::ValidationError("Missing agent type".to_string()))?;
        let agent_type = AgentType::from(agent_type_str.to_string());

        match setting_type {
            "agent_prompts" => {
                self.agent_prompts.insert(agent_type, value);
            }
            "agent_max_tokens" => {
                let max_tokens = value.parse::<u32>().map_err(|_| {
                    AppError::ValidationError("Invalid max tokens value".to_string())
                })?;
                self.agent_max_tokens.insert(agent_type, max_tokens);
            }
            "agent_temperature" => {
                let temperature = value.parse::<f32>().map_err(|_| {
                    AppError::ValidationError("Invalid temperature value".to_string())
                })?;
                self.agent_temperature.insert(agent_type, temperature);
            }
            "agent_models_per_provider" => {
                let provider_type_str = parts.get(2).ok_or_else(|| {
                    AppError::ValidationError("Missing provider type".to_string())
                })?;

                let provider_type = ProviderType::from(*provider_type_str);

                self.agent_models_per_provider
                    .entry(agent_type)
                    .or_default()
                    .insert(provider_type, value);
            }
            _ => {
                return Err(AppError::ValidationError(format!(
                    "Unknown setting type: {}",
                    setting_type
                )))
            }
        }

        self.save()
    }
}
