use crate::config::api_config::LlmUserConfig;
use crate::config::app_settings::{AgentSettings, StudySettings};
use crate::errors::AppResult;
use crate::services::db_service::DbService;
use crate::services::llm_service::manager::LlmService;
use crate::services::vector_service::vector_service::VectorService;
use crate::services::file_service::chromium_service::ChromiumService;
use tokio::sync::Mutex;

/// Application state containing database service, LLM service, and vector service
pub struct AppState {
    pub db: Mutex<DbService>,
    pub llm: Mutex<LlmService>,
    pub vector: Mutex<VectorService>,
    pub chromium: Mutex<ChromiumService>,
    pub study_settings: Mutex<StudySettings>,
    pub agent_settings: Mutex<AgentSettings>,
}

impl AppState {
    pub async fn new() -> Self {
        let db_service = DbService::new().expect("Failed to initialize database service");

        // Load configuration to initialize LlmService
        let mut config = LlmUserConfig::load().unwrap_or_else(|e| {
            eprintln!(
                "FATAL: Failed to load LlmUserConfig during AppState::new: {}. Using default.",
                e
            );
            LlmUserConfig::default()
        });

        config.refresh_all_models().await.unwrap_or_else(|e| {
            eprintln!("Could not retrieve models for some Providers. API Keys probably not set or Provider service is down: {}", e);
        });

        let vector_service = VectorService::new().await.unwrap_or_else(|e| {
            eprintln!("FATAL: Failed to initialize VectorService: {}. This may affect search functionality.", e);
            panic!("Vector service is required for the application to function properly");
        });

        let chromium_service = ChromiumService::new();
        if chromium_service.is_chromium_downloaded() {
            chromium_service.initialize().await.unwrap_or_else(|e| {
                eprintln!("Failed to initialize Chromium: {}. Document preview features may not work.", e);
            });
        }

        let study_settings_loaded = StudySettings::load().unwrap_or_else(|e| {
            eprintln!("FATAL: Failed to load general study settings during AppState::new: {}. Using default.", e);
            StudySettings::default()
        });

        let agent_settings_loaded = AgentSettings::load().unwrap_or_else(|e| {
            eprintln!(
                "FATAL: Failed to load agent settings during AppState::new: {}. Using default.",
                e
            );
            AgentSettings::default()
        });

        let llm_service = LlmService::new(&config, &agent_settings_loaded).await;

        Self {
            db: Mutex::new(db_service),
            llm: Mutex::new(llm_service),
            vector: Mutex::new(vector_service),
            chromium: Mutex::new(chromium_service),
            study_settings: Mutex::new(study_settings_loaded),
            agent_settings: Mutex::new(agent_settings_loaded),
        }
    }

    pub async fn rebuild_llm_service(&self) -> AppResult<()> {
        let config = LlmUserConfig::load()?;
        let agent_settings = AgentSettings::load()?;
        let mut llm_service = self.llm.lock().await;
        llm_service.rebuild(&config, &agent_settings).await;

        println!("LLM service successfully rebuilt with current configuration");
        Ok(())
    }
}
