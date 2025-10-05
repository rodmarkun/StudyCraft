use crate::materials::flashcard::{NewFlashcard, UpdateFlashcard};
use crate::materials::test::{NewTestQuestion, UpdateTestQuestion};
use crate::services::llm_service::manager::ConceptExtraction;
use serde::{Deserialize, Serialize};

// These are more complex request objects that frontend uses to communicate with backend

/// Request for updating a flashcard deck
#[derive(Serialize, Deserialize, Debug)]
pub struct FlashcardDeckUpdateRequest {
    pub deck_id: String,
    pub name: Option<String>,
    pub cards_to_add: Option<Vec<NewFlashcard>>,
    pub cards_to_update: Option<Vec<UpdateFlashcard>>,
    pub cards_to_delete: Option<Vec<i64>>,
}

/// Request for updating a test
#[derive(Serialize, Deserialize, Debug)]
pub struct TestUpdateRequest {
    pub test_id: String,
    pub name: Option<String>,
    pub questions_to_add: Option<Vec<NewTestQuestion>>,
    pub questions_to_update: Option<Vec<UpdateTestQuestion>>,
    pub questions_to_delete: Option<Vec<i64>>,
}
