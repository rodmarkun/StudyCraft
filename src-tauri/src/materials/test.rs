use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Individual test question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestQuestion {
    pub id: Option<i64>,           // question id
    pub test_id: String,           // test id
    pub question: String,          // question text
    pub position: i32,             // position inside deck
    pub created_at: DateTime<Utc>, // creation date
    pub updated_at: DateTime<Utc>, // last update date
}

/// Individual test answer option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAnswer {
    pub id: Option<i64>,           // answer id TODO - Remove Option<> from here
    pub question_id: i64,          // question id
    pub answer_text: String,       // answer text
    pub is_correct: bool,          // is this answer the correct one
    pub position: i32,             // position inside this question
    pub created_at: DateTime<Utc>, // creation date
    pub updated_at: DateTime<Utc>, // last update date
}

/// Complete question with its answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestQuestionWithAnswers {
    pub question: TestQuestion,
    pub answers: Vec<TestAnswer>,
}

/// New test question for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTestQuestion {
    pub question: String,
    pub position: i32,
    pub answers: Vec<NewTestAnswer>,
}

/// New test answer for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTestAnswer {
    pub answer_text: String,
    pub is_correct: bool,
    pub position: i32,
}

/// Test update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUpdateRequest {
    pub test_id: String,
    pub name: Option<String>,
    pub questions_to_add: Option<Vec<NewTestQuestion>>,
    pub questions_to_update: Option<Vec<UpdateTestQuestion>>,
    pub questions_to_delete: Option<Vec<i64>>,
}

/// Update test question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTestQuestion {
    pub id: i64,
    pub question: String,
    pub position: i32,
    pub answers: Vec<UpdateTestAnswer>,
}

/// Update test answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTestAnswer {
    pub id: Option<i64>, // None for new answers
    pub answer_text: String,
    pub is_correct: bool,
    pub position: i32,
}

/// Test review settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReviewSettings {
    pub shuffle_questions: bool,
    pub shuffle_answers: bool,
}

impl Default for TestReviewSettings {
    fn default() -> Self {
        Self {
            shuffle_questions: true,
            shuffle_answers: true,
        }
    }
}
