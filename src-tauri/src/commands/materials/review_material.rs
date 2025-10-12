use crate::commands::structure::requests;
use crate::commands::structure::responses;
use crate::constants;
use crate::materials::flashcard::export_to_anki;
use crate::materials::flashcard::{Flashcard, FlashcardCounts, NewFlashcard};
use crate::materials::review_material;
use crate::materials::test::{NewTestQuestion, TestQuestionWithAnswers};
use crate::services::db_service::review_sessions::{
    CreateFlashcardReviewSessionRequest, CreateReviewSessionRequest,
    CreateTestReviewSessionRequest, FlashcardReviewSession, ReviewSession, ReviewSessionStats,
    TestReviewSession,
};
use crate::services::db_service::{
    FlashcardDeckOperations, ReviewMaterialOperations, ReviewSessionOperations, TagOperations,
    TestOperations,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_review_materials(
    app_state: State<'_, AppState>,
    sort_by: Option<String>,
) -> Result<Vec<review_material::ReviewMaterial>, String> {
    let mut materials = Vec::new();
    let db_service = app_state.db.lock().await;

    let flashcard_decks = db_service
        .get_flashcard_decks()
        .map_err(|e| format!("Failed to get flashcard decks: {}", e))?;
    materials.extend(flashcard_decks);

    let tests = db_service
        .get_tests()
        .map_err(|e| format!("Failed to get tests: {}", e))?;
    materials.extend(tests);

    let sorted_materials = review_material::sort_review_materials(sort_by, materials);

    Ok(sorted_materials)
}

#[tauri::command]
pub async fn create_flashcard_deck(
    id: String,
    name: String,
    tags: Vec<String>,
    cards: Vec<NewFlashcard>,
    app_state: State<'_, AppState>,
) -> Result<responses::FlashcardDeckCreationResponse, String> {
    review_material::validate_deck_input(&name, &cards)?;
    let db_service = app_state.db.lock().await;

    let deck = db_service
        .create_flashcard_deck(&id, &name, &tags, &cards)
        .map_err(|e| format!("Failed to create flashcard deck: {}", e))?;

    let response = responses::FlashcardDeckCreationResponse {
        review_material_id: deck.id,
        review_material_name: deck.display_name,
        cards_count: cards.len(),
    };

    Ok(response)
}

#[tauri::command]
pub async fn get_flashcards(
    deck_id: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<Flashcard>, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_flashcards(&deck_id)
        .map_err(|e| format!("Failed to get flashcards: {}", e))
}

#[tauri::command]
pub async fn get_flashcard_counts(
    deck_id: String,
    app_state: State<'_, AppState>,
) -> Result<FlashcardCounts, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_flashcard_counts(&deck_id)
        .map_err(|e| format!("Failed to get flashcard counts: {}", e))
}

#[tauri::command]
pub async fn get_flashcards_for_review(
    deck_id: String,
    settings: review_material::FlashcardDeckReviewSettings,
    app_state: State<'_, AppState>,
) -> Result<Vec<Flashcard>, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_flashcards_for_review(&deck_id, &settings)
        .map_err(|e| format!("Failed to get flashcards for review: {}", e))
}

#[tauri::command]
pub async fn update_flashcard_after_review(
    card_id: i64,
    difficulty: String,
    app_state: State<'_, AppState>,
) -> Result<Flashcard, String> {
    match difficulty.as_str() {
        d if constants::VALID_DIFFICULTIES.contains(&d) => {}
        _ => {
            return Err("Invalid difficulty rating".to_string());
        }
    }
    let db_service = app_state.db.lock().await;

    db_service
        .update_flashcard_after_review(card_id, &difficulty)
        .map_err(|e| format!("Failed to update flashcard: {}", e))
}

#[tauri::command]
pub async fn update_flashcard_deck(
    request: requests::FlashcardDeckUpdateRequest,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let db_service = app_state.db.lock().await;

    db_service
        .update_flashcard_deck(request)
        .map_err(|e| format!("Failed to update flashcard deck: {}", e))
}

#[tauri::command]
pub async fn get_flashcard_deck_details(
    deck_id: String,
    app_state: State<'_, AppState>,
) -> Result<responses::FlashcardDeckDetailsResponse, String> {
    let db_service = app_state.db.lock().await;

    let deck = db_service
        .get_flashcard_deck(&deck_id)
        .map_err(|e| format!("Failed to get flashcard deck: {}", e))?
        .ok_or("Flashcard deck not found")?;

    let cards = db_service
        .get_flashcards(&deck_id)
        .map_err(|e| format!("Failed to get flashcards: {}", e))?;

    let tags = db_service
        .get_material_tags(&deck_id)
        .map_err(|e| format!("Failed to get material tags: {}", e))?
        .into_iter()
        .collect();

    Ok(responses::FlashcardDeckDetailsResponse { deck, cards, tags })
}

#[tauri::command]
pub async fn delete_flashcard_deck(
    deck_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let db_service = app_state.db.lock().await;

    db_service
        .delete_flashcard_deck(&deck_id)
        .map_err(|e| format!("Failed to delete flashcard deck: {}", e))
}

#[tauri::command]
pub async fn update_flashcard_deck_name(
    deck_id: String,
    name: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Deck name cannot be empty".to_string());
    }

    let db_service = app_state.db.lock().await;

    db_service
        .update_flashcard_deck_name(&deck_id, &name)
        .map_err(|e| format!("Failed to update flashcard deck name: {}", e))
}

#[tauri::command]
pub async fn create_test(
    id: String,
    name: String,
    tags: Vec<String>,
    questions: Vec<NewTestQuestion>,
    app_state: State<'_, AppState>,
) -> Result<responses::TestCreationResponse, String> {
    review_material::validate_test(&name, &questions)?;
    let db_service = app_state.db.lock().await;

    let test = db_service
        .create_test(&id, &name, &tags, &questions)
        .map_err(|e| format!("Failed to create test: {}", e))?;

    let response = responses::TestCreationResponse {
        review_material_id: test.id,
        review_material_name: test.display_name,
        questions_count: questions.len(),
    };

    Ok(response)
}

#[tauri::command]
pub async fn get_test_details(
    test_id: String,
    app_state: State<'_, AppState>,
) -> Result<responses::TestDetailsResponse, String> {
    let db_service = app_state.db.lock().await;

    let test = db_service
        .get_test(&test_id)
        .map_err(|e| format!("Failed to get test: {}", e))?
        .ok_or("Test not found")?;

    let questions = db_service
        .get_test_questions(&test_id)
        .map_err(|e| format!("Failed to get test questions: {}", e))?;

    let tags = db_service
        .get_material_tags(&test_id)
        .map_err(|e| format!("Failed to get material tags: {}", e))?
        .into_iter()
        .collect();

    Ok(responses::TestDetailsResponse {
        test,
        questions,
        tags,
    })
}

#[tauri::command]
pub async fn get_test_questions(
    test_id: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<TestQuestionWithAnswers>, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_test_questions(&test_id)
        .map_err(|e| format!("Failed to get test questions: {}", e))
}

#[tauri::command]
pub async fn update_test(
    request: requests::TestUpdateRequest,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let db_service = app_state.db.lock().await;

    db_service
        .update_test(request)
        .map_err(|e| format!("Failed to update test: {}", e))
}

#[tauri::command]
pub async fn delete_test(test_id: String, app_state: State<'_, AppState>) -> Result<(), String> {
    let db_service = app_state.db.lock().await;

    db_service
        .delete_test(&test_id)
        .map_err(|e| format!("Failed to delete test: {}", e))
}

#[tauri::command]
pub async fn update_test_name(
    test_id: String,
    name: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Test name cannot be empty".to_string());
    }

    let db_service = app_state.db.lock().await;

    db_service
        .update_test_name(&test_id, &name)
        .map_err(|e| format!("Failed to update test name: {}", e))
}

#[tauri::command]
pub async fn create_review_session(
    request: CreateReviewSessionRequest,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .create_review_session(request)
        .map_err(|e| format!("Failed to create review session: {}", e))
}

#[tauri::command]
pub async fn update_last_review(
    material_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut db_service = app_state.db.lock().await;

    db_service
        .update_review_material_last_review(&material_id)
        .map_err(|e| format!("Failed to update last review time: {}", e))
}

#[tauri::command]
pub async fn create_flashcard_review_session(
    request: CreateFlashcardReviewSessionRequest,
    app_state: State<'_, AppState>,
) -> Result<FlashcardReviewSession, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .create_flashcard_review_session(request)
        .map_err(|e| format!("Failed to create flashcard review session: {}", e))
}

#[tauri::command]
pub async fn create_test_review_session(
    request: CreateTestReviewSessionRequest,
    app_state: State<'_, AppState>,
) -> Result<TestReviewSession, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .create_test_review_session(request)
        .map_err(|e| format!("Failed to create test review session: {}", e))
}

#[tauri::command]
pub async fn get_flashcard_review_sessions_by_material(
    material_id: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<FlashcardReviewSession>, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_flashcard_review_sessions_by_material(&material_id)
        .map_err(|e| format!("Failed to get flashcard review sessions: {}", e))
}

#[tauri::command]
pub async fn get_test_review_sessions_by_material(
    material_id: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<TestReviewSession>, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_test_review_sessions_by_material(&material_id)
        .map_err(|e| format!("Failed to get test review sessions: {}", e))
}

#[tauri::command]
pub async fn get_all_review_sessions(
    app_state: State<'_, AppState>,
) -> Result<Vec<ReviewSession>, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_all_review_sessions()
        .map_err(|e| format!("Failed to get all review sessions: {}", e))
}

#[tauri::command]
pub async fn get_review_session_stats(
    app_state: State<'_, AppState>,
) -> Result<ReviewSessionStats, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_review_session_stats()
        .map_err(|e| format!("Failed to get review session statistics: {}", e))
}

#[tauri::command]
pub async fn get_recent_review_sessions(
    limit: i32,
    app_state: State<'_, AppState>,
) -> Result<Vec<ReviewSession>, String> {
    let db_service = app_state.db.lock().await;

    if limit <= 0 || limit > constants::LIMIT_RECENT_SESSIONS_RETRIEVAL {
        return Err(format!(
            "Limit must be between 1 and {}",
            constants::LIMIT_RECENT_SESSIONS_RETRIEVAL
        ));
    }

    db_service
        .get_recent_review_sessions(limit)
        .map_err(|e| format!("Failed to get recent review sessions: {}", e))
}

#[tauri::command]
pub async fn get_material_tags(
    material_id: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let db_service = app_state.db.lock().await;

    db_service
        .get_material_tags(&material_id)
        .map_err(|e| format!("Failed to get material tags: {}", e))
        .map(|tags| tags.into_iter().collect())
}

#[tauri::command]
pub async fn export_flashcard_deck_to_anki(
    deck_id: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let db_service = app_state.db.lock().await;

    let flashcards = db_service
        .get_flashcards(&deck_id)
        .map_err(|e| format!("Failed to get flashcards: {}", e))?;

    Ok(export_to_anki(flashcards))
    }