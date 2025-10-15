use crate::materials::flashcard::Flashcard;
use crate::materials::review_material::ReviewMaterial;
use crate::materials::test::TestQuestionWithAnswers;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// These are responses the backend uses to communicate with the frontend

/// Response after creating a flashcard deck
#[derive(Serialize, Deserialize, Debug)]
pub struct FlashcardDeckCreationResponse {
    pub review_material_id: String,
    pub review_material_name: String,
    pub cards_count: usize,
}

/// Response after creating a test
#[derive(Serialize, Deserialize, Debug)]
pub struct TestCreationResponse {
    pub review_material_id: String,
    pub review_material_name: String,
    pub questions_count: usize,
}

/// Complete deck details for display/editing
#[derive(Serialize, Deserialize, Debug)]
pub struct FlashcardDeckDetailsResponse {
    pub deck: ReviewMaterial,
    pub cards: Vec<Flashcard>,
    pub tags: Vec<String>,
}

/// Complete test details for display/editing
#[derive(Serialize, Deserialize, Debug)]
pub struct TestDetailsResponse {
    pub test: ReviewMaterial,
    pub questions: Vec<TestQuestionWithAnswers>,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct ProviderConfigResponse {
    pub provider_id: String,
    pub api_key: String,
    pub enabled: bool,
    pub available_models: Vec<String>,
    pub is_configured: bool,
    pub endpoint_url: Option<String>,
}

#[derive(Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub models: Vec<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize)]
pub struct AgentModelConfigResponse {
    pub agent_type: String,
    pub provider_models: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct AvailableModelsResponse {
    pub provider: String,
    pub available_models: Vec<String>,
    pub current_model: String,
}
