use crate::constants;
use crate::materials::flashcard::{Flashcard, NewFlashcard};
use crate::materials::test::{NewTestQuestion, TestQuestionWithAnswers};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Main wrapper for review materials (decks, tests, etc.)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewMaterial {
    pub id: String,
    pub display_name: String,
    pub rm_type: ReviewMaterialType,
    pub last_review: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Types of review materials
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReviewMaterialType {
    #[serde(rename = "flashcard_deck")]
    FlashcardDeck,
    #[serde(rename = "test")]
    Test,
}

/// Review settings for flashcard sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashcardDeckReviewSettings {
    pub shuffle_deck: bool,
    pub use_only_pending: bool,
}

/// Review settings for test sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReviewSettings {
    pub shuffle_questions: bool,
    pub shuffle_answers: bool,
}

impl Default for FlashcardDeckReviewSettings {
    fn default() -> Self {
        Self {
            shuffle_deck: true,
            use_only_pending: true,
        }
    }
}

impl Default for TestReviewSettings {
    fn default() -> Self {
        Self {
            shuffle_questions: true,
            shuffle_answers: true,
        }
    }
}

impl From<String> for ReviewMaterialType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "flashcard_deck" => ReviewMaterialType::FlashcardDeck,
            "test" => ReviewMaterialType::Test,
            _ => ReviewMaterialType::FlashcardDeck,
        }
    }
}

impl From<ReviewMaterialType> for String {
    fn from(material_type: ReviewMaterialType) -> Self {
        match material_type {
            ReviewMaterialType::FlashcardDeck => "flashcard_deck".to_string(),
            ReviewMaterialType::Test => "test".to_string(),
        }
    }
}

pub fn validate_deck_input(name: &str, cards: &[NewFlashcard]) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Deck name cannot be empty".to_string());
    }

    if cards.is_empty() {
        return Err("At least one card is required".to_string());
    }

    for (i, card) in cards.iter().enumerate() {
        if card.front.trim().is_empty() {
            return Err(format!("Card {} front cannot be empty", i + 1));
        }
        if card.back.trim().is_empty() {
            return Err(format!("Card {} back cannot be empty", i + 1));
        }
    }

    Ok(())
}

pub fn validate_test(name: &str, questions: &[NewTestQuestion]) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Test name cannot be empty".to_string());
    }

    if questions.is_empty() {
        return Err("At least one question is required".to_string());
    }

    // Validate questions
    for (i, question) in questions.iter().enumerate() {
        if question.question.trim().is_empty() {
            return Err(format!("Question {} cannot be empty", i + 1));
        }
        if question.answers.is_empty() {
            return Err(format!("Question {} must have at least one answer", i + 1));
        }

        let correct_answers = question.answers.iter().filter(|a| a.is_correct).count();
        if correct_answers == 0 {
            return Err(format!(
                "Question {} must have at least one correct answer",
                i + 1
            ));
        }

        for (j, answer) in question.answers.iter().enumerate() {
            if answer.answer_text.trim().is_empty() {
                return Err(format!(
                    "Question {} answer {} cannot be empty",
                    i + 1,
                    j + 1
                ));
            }
        }
    }

    Ok(())
}

pub fn sort_review_materials(
    sort_by: Option<String>,
    mut materials: Vec<ReviewMaterial>,
) -> Vec<ReviewMaterial> {
    match sort_by.as_deref() {
        Some(constants::NAME_ASC) => {
            materials.sort_by(|a, b| {
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase())
            });
        }
        Some(constants::NAME_DESC) => {
            materials.sort_by(|a, b| {
                b.display_name
                    .to_lowercase()
                    .cmp(&a.display_name.to_lowercase())
            });
        }
        Some(constants::DATE_ASC) => {
            materials.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
        }
        Some(constants::DATE_DESC) => {
            materials.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        }
        _ => {
            // Default sorting: date ascending
            materials.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
        }
    }

    materials
}
