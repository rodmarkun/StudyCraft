use crate::services::llm_service;
use crate::services::llm_service::manager::{
    ConceptExtraction, FlashcardGenerationRequest, GeneratedCard, GeneratedTestQuestion,
    SearchAgentResponse, TestGenerationRequest,
};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize)]
pub struct ProviderInfo {
    pub provider: String,
    pub model: String,
    pub enabled: bool,
    pub instance_id: usize,
}

#[tauri::command]
pub async fn generate_flashcards(
    state: State<'_, AppState>,
    request: FlashcardGenerationRequest,
) -> Result<Vec<GeneratedCard>, String> {
    let llm_service = state.llm.lock().await;

    llm_service
        .generate_flashcards_from_materials(&state, request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_test_questions(
    state: State<'_, AppState>,
    request: TestGenerationRequest,
) -> Result<Vec<GeneratedTestQuestion>, String> {
    let llm_service = state.llm.lock().await;

    llm_service
        .generate_test_questions_from_materials(&state, request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn explain_flashcard(
    state: State<'_, AppState>,
    question: String,
    answer: String,
) -> Result<String, String> {
    let agent_settings = state.agent_settings.lock().await;
    let llm_service = state.llm.lock().await;

    llm_service
        .generate_flashcard_explanation(question, answer, &agent_settings)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_in_study_materials(
    state: State<'_, AppState>,
    query: String,
) -> Result<SearchAgentResponse, String> {
    let llm_service = state.llm.lock().await;

    llm_service
        .search_study_materials(&state, query)
        .await
        .map_err(|e| e.to_string())
}
