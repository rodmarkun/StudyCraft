use super::connection::DbService;
use super::queries;
use crate::errors::{AppError, AppResult};
use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enhanced Review Session with optional fields for both flashcard and test data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub id: String,
    pub material_id: Uuid,
    pub material_type: String,
    pub session_start: DateTime<Utc>,
    pub session_end: DateTime<Utc>,
    pub total_duration_seconds: i32,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub material_name: Option<String>,

    // Flashcard-specific fields (will be null for test sessions)
    pub cards_studied: Option<i32>,
    pub again_count: Option<i32>,
    pub hard_count: Option<i32>,
    pub good_count: Option<i32>,
    pub easy_count: Option<i32>,
    pub new_cards_count: Option<i32>,
    pub learning_cards_count: Option<i32>,
    pub review_cards_count: Option<i32>,
    pub retention_rate: Option<f64>,
    pub average_response_time_seconds: Option<f64>,

    // Test-specific fields (will be null for flashcard sessions)
    pub questions_answered: Option<i32>,
    pub correct_answers: Option<i32>,
    pub incorrect_answers: Option<i32>,
    pub skipped_answers: Option<i32>,
    pub score_percentage: Option<f64>,
    pub time_per_question_seconds: Option<f64>,
}

/// Flashcard-specific review session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashcardReviewSession {
    pub session: ReviewSession,
    pub cards_studied: i32,
    pub again_count: i32,
    pub hard_count: i32,
    pub good_count: i32,
    pub easy_count: i32,
    pub new_cards_count: i32,
    pub learning_cards_count: i32,
    pub review_cards_count: i32,
    pub retention_rate: f64,
    pub average_response_time_seconds: f64,
}

/// Test-specific review session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReviewSession {
    pub session: ReviewSession,
    pub questions_answered: i32,
    pub correct_answers: i32,
    pub incorrect_answers: i32,
    pub skipped_answers: i32,
    pub score_percentage: f64,
    pub time_per_question_seconds: f64,
}

/// Enhanced review session stats with separated counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSessionStats {
    pub total_sessions: i32,
    pub total_study_time_seconds: i32,
    pub average_cards_per_session: Option<f64>,
    pub average_flashcard_retention: Option<f64>,
    pub average_test_score: Option<f64>,
    pub last_session: Option<DateTime<Utc>>,
    pub materials_studied: i32,
    pub flashcard_sessions: i32,
    pub test_sessions: i32,
}

/// Request to create a flashcard review session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFlashcardReviewSessionRequest {
    pub material_id: Uuid,
    pub session_start: DateTime<Utc>,
    pub session_end: DateTime<Utc>,
    pub total_duration_seconds: i32,
    pub cards_studied: i32,
    pub again_count: i32,
    pub hard_count: i32,
    pub good_count: i32,
    pub easy_count: i32,
    pub new_cards_count: i32,
    pub learning_cards_count: i32,
    pub review_cards_count: i32,
    pub retention_rate: f64,
    pub average_response_time_seconds: f64,
    pub completed: bool,
}

/// Request to create a test review session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestReviewSessionRequest {
    pub material_id: Uuid,
    pub session_start: DateTime<Utc>,
    pub session_end: DateTime<Utc>,
    pub total_duration_seconds: i32,
    pub questions_answered: i32,
    pub correct_answers: i32,
    pub incorrect_answers: i32,
    pub skipped_answers: i32,
    pub score_percentage: f64,
    pub time_per_question_seconds: f64,
    pub completed: bool,
}

/// Unified request type for creating any type of review session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "material_type")]
pub enum CreateReviewSessionRequest {
    #[serde(rename = "flashcard_deck")]
    FlashcardDeck {
        material_id: String,
        session_start: String, // ISO 8601 string
        session_end: String,   // ISO 8601 string
        total_duration_seconds: i32,
        cards_studied: i32,
        again_count: i32,
        hard_count: i32,
        good_count: i32,
        easy_count: i32,
        new_cards_count: i32,
        learning_cards_count: i32,
        review_cards_count: i32,
        retention_rate: f64,
        average_response_time_seconds: f64,
        completed: bool,
    },
    #[serde(rename = "test")]
    Test {
        material_id: String,
        session_start: String, // ISO 8601 string
        session_end: String,   // ISO 8601 string
        total_duration_seconds: i32,
        questions_answered: i32,
        correct_answers: i32,
        incorrect_answers: i32,
        skipped_answers: i32,
        score_percentage: f64,
        time_per_question_seconds: f64,
        completed: bool,
    },
}

#[derive(Debug)]
struct ReviewSessionRow {
    id: String,
    material_id: String,
    material_type: String,
    session_start: String,
    session_end: String,
    total_duration_seconds: i32,
    completed: bool,
    created_at: String,
    material_name: Option<String>,
}

impl ReviewSessionRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(ReviewSessionRow {
            id: row.get("id")?,
            material_id: row.get("material_id")?,
            material_type: row.get("material_type")?,
            session_start: row.get("session_start")?,
            session_end: row.get("session_end")?,
            total_duration_seconds: row.get("total_duration_seconds")?,
            completed: row.get("completed")?,
            created_at: row.get("created_at")?,
            material_name: row.get("material_name").ok(),
        })
    }

    fn into_review_session(self) -> AppResult<ReviewSession> {
        let material_id = Uuid::parse_str(&self.material_id)
            .map_err(|e| AppError::DbError(format!("Failed to parse material ID: {}", e)))?;

        let session_start =
            chrono::NaiveDateTime::parse_from_str(&self.session_start, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse session_start: {}", e)))?
                .and_utc();
        let session_end =
            chrono::NaiveDateTime::parse_from_str(&self.session_end, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse session_end: {}", e)))?
                .and_utc();
        let created_at =
            chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse created_at: {}", e)))?
                .and_utc();

        Ok(ReviewSession {
            id: self.id,
            material_id,
            material_type: self.material_type,
            session_start,
            session_end,
            total_duration_seconds: self.total_duration_seconds,
            completed: self.completed,
            created_at,
            material_name: self.material_name,
            cards_studied: None,
            again_count: None,
            hard_count: None,
            good_count: None,
            easy_count: None,
            new_cards_count: None,
            learning_cards_count: None,
            review_cards_count: None,
            retention_rate: None,
            average_response_time_seconds: None,
            questions_answered: None,
            correct_answers: None,
            incorrect_answers: None,
            skipped_answers: None,
            score_percentage: None,
            time_per_question_seconds: None,
        })
    }
}

#[derive(Debug)]
struct EnhancedReviewSessionRow {
    id: String,
    material_id: String,
    material_type: String,
    session_start: String,
    session_end: String,
    total_duration_seconds: i32,
    material_name: Option<String>,
    completed: bool,
    // Flashcard fields (optional)
    cards_studied: Option<i32>,
    again_count: Option<i32>,
    hard_count: Option<i32>,
    good_count: Option<i32>,
    easy_count: Option<i32>,
    new_cards_count: Option<i32>,
    learning_cards_count: Option<i32>,
    review_cards_count: Option<i32>,
    retention_rate: Option<f64>,
    average_response_time_seconds: Option<f64>,
    // Test fields (optional)
    questions_answered: Option<i32>,
    correct_answers: Option<i32>,
    incorrect_answers: Option<i32>,
    skipped_answers: Option<i32>,
    score_percentage: Option<f64>,
    time_per_question_seconds: Option<f64>,
}

impl EnhancedReviewSessionRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(EnhancedReviewSessionRow {
            id: row.get(0)?,
            material_id: row.get(1)?,
            material_type: row.get(2)?,
            session_start: row.get(3)?,
            session_end: row.get(4)?,
            total_duration_seconds: row.get(5)?,
            material_name: row.get(6).ok(),
            completed: row.get(7)?,
            // Flashcard fields (columns 8-17)
            cards_studied: row.get(8).ok(),
            again_count: row.get(9).ok(),
            hard_count: row.get(10).ok(),
            good_count: row.get(11).ok(),
            easy_count: row.get(12).ok(),
            new_cards_count: row.get(13).ok(),
            learning_cards_count: row.get(14).ok(),
            review_cards_count: row.get(15).ok(),
            retention_rate: row.get(16).ok(),
            average_response_time_seconds: row.get(17).ok(),
            // Test fields (columns 18-23)
            questions_answered: row.get(18).ok(),
            correct_answers: row.get(19).ok(),
            incorrect_answers: row.get(20).ok(),
            skipped_answers: row.get(21).ok(),
            score_percentage: row.get(22).ok(),
            time_per_question_seconds: row.get(23).ok(),
        })
    }

    fn into_review_session(self) -> AppResult<ReviewSession> {
        let material_id = Uuid::parse_str(&self.material_id)
            .map_err(|e| AppError::DbError(format!("Failed to parse material ID: {}", e)))?;

        let session_start =
            chrono::NaiveDateTime::parse_from_str(&self.session_start, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse session_start: {}", e)))?
                .and_utc();
        let session_end =
            chrono::NaiveDateTime::parse_from_str(&self.session_end, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse session_end: {}", e)))?
                .and_utc();

        Ok(ReviewSession {
            id: self.id,
            material_id,
            material_type: self.material_type,
            session_start,
            session_end,
            total_duration_seconds: self.total_duration_seconds,
            completed: self.completed,
            created_at: session_start, // Use session_start as created_at for enhanced rows
            material_name: self.material_name,
            cards_studied: self.cards_studied,
            again_count: self.again_count,
            hard_count: self.hard_count,
            good_count: self.good_count,
            easy_count: self.easy_count,
            new_cards_count: self.new_cards_count,
            learning_cards_count: self.learning_cards_count,
            review_cards_count: self.review_cards_count,
            retention_rate: self.retention_rate,
            average_response_time_seconds: self.average_response_time_seconds,
            questions_answered: self.questions_answered,
            correct_answers: self.correct_answers,
            incorrect_answers: self.incorrect_answers,
            skipped_answers: self.skipped_answers,
            score_percentage: self.score_percentage,
            time_per_question_seconds: self.time_per_question_seconds,
        })
    }
}

#[derive(Debug)]
struct FlashcardReviewSessionRow {
    session: ReviewSessionRow,
    cards_studied: i32,
    again_count: i32,
    hard_count: i32,
    good_count: i32,
    easy_count: i32,
    new_cards_count: i32,
    learning_cards_count: i32,
    review_cards_count: i32,
    retention_rate: f64,
    average_response_time_seconds: f64,
}

impl FlashcardReviewSessionRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let session = ReviewSessionRow::from_row(row)?;
        Ok(FlashcardReviewSessionRow {
            session,
            cards_studied: row.get("cards_studied")?,
            again_count: row.get("again_count")?,
            hard_count: row.get("hard_count")?,
            good_count: row.get("good_count")?,
            easy_count: row.get("easy_count")?,
            new_cards_count: row.get("new_cards_count")?,
            learning_cards_count: row.get("learning_cards_count")?,
            review_cards_count: row.get("review_cards_count")?,
            retention_rate: row.get("retention_rate")?,
            average_response_time_seconds: row.get("average_response_time_seconds")?,
        })
    }

    fn into_flashcard_review_session(self) -> AppResult<FlashcardReviewSession> {
        let session = self.session.into_review_session()?;
        Ok(FlashcardReviewSession {
            session,
            cards_studied: self.cards_studied,
            again_count: self.again_count,
            hard_count: self.hard_count,
            good_count: self.good_count,
            easy_count: self.easy_count,
            new_cards_count: self.new_cards_count,
            learning_cards_count: self.learning_cards_count,
            review_cards_count: self.review_cards_count,
            retention_rate: self.retention_rate,
            average_response_time_seconds: self.average_response_time_seconds,
        })
    }
}

#[derive(Debug)]
struct TestReviewSessionRow {
    session: ReviewSessionRow,
    questions_answered: i32,
    correct_answers: i32,
    incorrect_answers: i32,
    skipped_answers: i32,
    score_percentage: f64,
    time_per_question_seconds: f64,
}

impl TestReviewSessionRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let session = ReviewSessionRow::from_row(row)?;
        Ok(TestReviewSessionRow {
            session,
            questions_answered: row.get("questions_answered")?,
            correct_answers: row.get("correct_answers")?,
            incorrect_answers: row.get("incorrect_answers")?,
            skipped_answers: row.get("skipped_answers")?,
            score_percentage: row.get("score_percentage")?,
            time_per_question_seconds: row.get("time_per_question_seconds")?,
        })
    }

    fn into_test_review_session(self) -> AppResult<TestReviewSession> {
        let session = self.session.into_review_session()?;
        Ok(TestReviewSession {
            session,
            questions_answered: self.questions_answered,
            correct_answers: self.correct_answers,
            incorrect_answers: self.incorrect_answers,
            skipped_answers: self.skipped_answers,
            score_percentage: self.score_percentage,
            time_per_question_seconds: self.time_per_question_seconds,
        })
    }
}

pub trait ReviewSessionOperations {
    fn create_flashcard_review_session(
        &self,
        request: CreateFlashcardReviewSessionRequest,
    ) -> AppResult<FlashcardReviewSession>;
    fn create_test_review_session(
        &self,
        request: CreateTestReviewSessionRequest,
    ) -> AppResult<TestReviewSession>;
    fn get_flashcard_review_sessions_by_material(
        &self,
        material_id: &str,
    ) -> AppResult<Vec<FlashcardReviewSession>>;
    fn get_test_review_sessions_by_material(
        &self,
        material_id: &str,
    ) -> AppResult<Vec<TestReviewSession>>;
    fn get_all_review_sessions(&self) -> AppResult<Vec<ReviewSession>>;
    fn get_review_session_stats(&self) -> AppResult<ReviewSessionStats>;
    fn get_recent_review_sessions(&self, limit: i32) -> AppResult<Vec<ReviewSession>>;
    fn create_review_session(&self, request: CreateReviewSessionRequest) -> AppResult<String>;
}

impl ReviewSessionOperations for DbService {
    fn create_flashcard_review_session(
        &self,
        request: CreateFlashcardReviewSessionRequest,
    ) -> AppResult<FlashcardReviewSession> {
        let session_id = Uuid::new_v4();
        let session_id_str = session_id.to_string();
        let material_id_str = request.material_id.to_string();

        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        // Format timestamps for SQLite
        let session_start_str = request
            .session_start
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let session_end_str = request.session_end.format("%Y-%m-%d %H:%M:%S").to_string();

        // Insert base review session
        tx.execute(
            queries::INSERT_REVIEW_SESSION,
            params![
                session_id_str,
                material_id_str,
                "flashcard_deck",
                session_start_str,
                session_end_str,
                request.total_duration_seconds,
                request.completed
            ],
        )
        .map_err(|e| AppError::DbError(format!("Failed to create review session: {}", e)))?;

        // Insert flashcard-specific data
        tx.execute(
            queries::INSERT_FLASHCARD_REVIEW_SESSION,
            params![
                session_id_str,
                request.cards_studied,
                request.again_count,
                request.hard_count,
                request.good_count,
                request.easy_count,
                request.new_cards_count,
                request.learning_cards_count,
                request.review_cards_count,
                request.retention_rate,
                request.average_response_time_seconds
            ],
        )
        .map_err(|e| {
            AppError::DbError(format!("Failed to create flashcard review session: {}", e))
        })?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(FlashcardReviewSession {
            session: ReviewSession {
                id: session_id_str,
                material_id: request.material_id,
                material_type: "flashcard_deck".to_string(),
                session_start: request.session_start,
                session_end: request.session_end,
                total_duration_seconds: request.total_duration_seconds,
                completed: request.completed,
                created_at: Utc::now(),
                material_name: None,
                cards_studied: Some(request.cards_studied),
                again_count: Some(request.again_count),
                hard_count: Some(request.hard_count),
                good_count: Some(request.good_count),
                easy_count: Some(request.easy_count),
                new_cards_count: Some(request.new_cards_count),
                learning_cards_count: Some(request.learning_cards_count),
                review_cards_count: Some(request.review_cards_count),
                retention_rate: Some(request.retention_rate),
                average_response_time_seconds: Some(request.average_response_time_seconds),
                questions_answered: None,
                correct_answers: None,
                incorrect_answers: None,
                skipped_answers: None,
                score_percentage: None,
                time_per_question_seconds: None,
            },
            cards_studied: request.cards_studied,
            again_count: request.again_count,
            hard_count: request.hard_count,
            good_count: request.good_count,
            easy_count: request.easy_count,
            new_cards_count: request.new_cards_count,
            learning_cards_count: request.learning_cards_count,
            review_cards_count: request.review_cards_count,
            retention_rate: request.retention_rate,
            average_response_time_seconds: request.average_response_time_seconds,
        })
    }

    fn create_test_review_session(
        &self,
        request: CreateTestReviewSessionRequest,
    ) -> AppResult<TestReviewSession> {
        let session_id = Uuid::new_v4();
        let session_id_str = session_id.to_string();
        let material_id_str = request.material_id.to_string();

        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        // Format timestamps for SQLite
        let session_start_str = request
            .session_start
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let session_end_str = request.session_end.format("%Y-%m-%d %H:%M:%S").to_string();

        // Insert base review session
        tx.execute(
            queries::INSERT_REVIEW_SESSION,
            params![
                session_id_str,
                material_id_str,
                "test",
                session_start_str,
                session_end_str,
                request.total_duration_seconds,
                request.completed
            ],
        )
        .map_err(|e| AppError::DbError(format!("Failed to create review session: {}", e)))?;

        // Insert test-specific data
        tx.execute(
            queries::INSERT_TEST_REVIEW_SESSION,
            params![
                session_id_str,
                request.questions_answered,
                request.correct_answers,
                request.incorrect_answers,
                request.skipped_answers,
                request.score_percentage,
                request.time_per_question_seconds,
            ],
        )
        .map_err(|e| AppError::DbError(format!("Failed to create test review session: {}", e)))?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(TestReviewSession {
            session: ReviewSession {
                id: session_id_str,
                material_id: request.material_id,
                material_type: "test".to_string(),
                session_start: request.session_start,
                session_end: request.session_end,
                total_duration_seconds: request.total_duration_seconds,
                completed: request.completed,
                created_at: Utc::now(),
                material_name: None,
                cards_studied: None,
                again_count: None,
                hard_count: None,
                good_count: None,
                easy_count: None,
                new_cards_count: None,
                learning_cards_count: None,
                review_cards_count: None,
                retention_rate: None,
                average_response_time_seconds: None,
                questions_answered: Some(request.questions_answered),
                correct_answers: Some(request.correct_answers),
                incorrect_answers: Some(request.incorrect_answers),
                skipped_answers: Some(request.skipped_answers),
                score_percentage: Some(request.score_percentage),
                time_per_question_seconds: Some(request.time_per_question_seconds),
            },
            questions_answered: request.questions_answered,
            correct_answers: request.correct_answers,
            incorrect_answers: request.incorrect_answers,
            skipped_answers: request.skipped_answers,
            score_percentage: request.score_percentage,
            time_per_question_seconds: request.time_per_question_seconds,
        })
    }

    fn get_flashcard_review_sessions_by_material(
        &self,
        material_id: &str,
    ) -> AppResult<Vec<FlashcardReviewSession>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_FLASHCARD_REVIEW_SESSIONS_BY_MATERIAL)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let session_iter = stmt
            .query_map(params![material_id], |row| {
                FlashcardReviewSessionRow::from_row(row)
            })
            .map_err(|e| {
                AppError::DbError(format!("Failed to query flashcard review sessions: {}", e))
            })?;

        let mut sessions = Vec::new();
        for session_result in session_iter {
            let row = session_result.map_err(|e| {
                AppError::DbError(format!(
                    "Failed to process flashcard review session row: {}",
                    e
                ))
            })?;
            sessions.push(row.into_flashcard_review_session()?);
        }

        Ok(sessions)
    }

    fn get_test_review_sessions_by_material(
        &self,
        material_id: &str,
    ) -> AppResult<Vec<TestReviewSession>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_TEST_REVIEW_SESSIONS_BY_MATERIAL)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let session_iter = stmt
            .query_map(params![material_id], |row| {
                TestReviewSessionRow::from_row(row)
            })
            .map_err(|e| {
                AppError::DbError(format!("Failed to query test review sessions: {}", e))
            })?;

        let mut sessions = Vec::new();
        for session_result in session_iter {
            let row = session_result.map_err(|e| {
                AppError::DbError(format!("Failed to process test review session row: {}", e))
            })?;
            sessions.push(row.into_test_review_session()?);
        }

        Ok(sessions)
    }

    fn get_all_review_sessions(&self) -> AppResult<Vec<ReviewSession>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_ALL_REVIEW_SESSIONS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let session_iter = stmt
            .query_map(params![], |row| ReviewSessionRow::from_row(row))
            .map_err(|e| AppError::DbError(format!("Failed to query review sessions: {}", e)))?;

        let mut sessions = Vec::new();
        for session_result in session_iter {
            let row = session_result.map_err(|e| {
                AppError::DbError(format!("Failed to process review session row: {}", e))
            })?;
            sessions.push(row.into_review_session()?);
        }

        Ok(sessions)
    }

    fn get_review_session_stats(&self) -> AppResult<ReviewSessionStats> {
        let conn = self.conn.borrow();

        let result = conn
            .query_row(queries::GET_REVIEW_SESSION_STATS, [], |row| {
                let total_sessions: i32 = row.get(0)?;
                let total_study_time: Option<i64> = row.get(1)?;
                let avg_cards_per_session: Option<f64> = row.get(2)?;
                let avg_flashcard_retention: Option<f64> = row.get(3)?;
                let avg_test_score: Option<f64> = row.get(4)?;
                let last_session_str: Option<String> = row.get(5)?;
                let materials_studied: i32 = row.get(6)?;
                let flashcard_sessions: i32 = row.get(7)?;
                let test_sessions: i32 = row.get(8)?;

                let last_session = last_session_str.and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| DateTime::from_utc(dt, Utc))
                });

                Ok(ReviewSessionStats {
                    total_sessions,
                    total_study_time_seconds: total_study_time.unwrap_or(0) as i32,
                    average_cards_per_session: avg_cards_per_session,
                    average_flashcard_retention: avg_flashcard_retention,
                    average_test_score: avg_test_score,
                    last_session,
                    materials_studied,
                    flashcard_sessions,
                    test_sessions,
                })
            })
            .map_err(|e| AppError::DbError(format!("Failed to get review session stats: {}", e)))?;

        Ok(result)
    }

    fn get_recent_review_sessions(&self, limit: i32) -> AppResult<Vec<ReviewSession>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_RECENT_REVIEW_SESSIONS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let session_iter = stmt
            .query_map(params![limit], |row| {
                EnhancedReviewSessionRow::from_row(row)
            })
            .map_err(|e| {
                AppError::DbError(format!("Failed to query recent review sessions: {}", e))
            })?;

        let mut sessions = Vec::new();
        for session_result in session_iter {
            let row = session_result.map_err(|e| {
                AppError::DbError(format!(
                    "Failed to process recent review session row: {}",
                    e
                ))
            })?;
            sessions.push(row.into_review_session()?);
        }

        Ok(sessions)
    }

    fn create_review_session(&self, request: CreateReviewSessionRequest) -> AppResult<String> {
        match request {
            CreateReviewSessionRequest::FlashcardDeck {
                material_id,
                session_start,
                session_end,
                total_duration_seconds,
                cards_studied,
                again_count,
                hard_count,
                good_count,
                easy_count,
                new_cards_count,
                learning_cards_count,
                review_cards_count,
                retention_rate,
                average_response_time_seconds,
                completed,
            } => {
                // Parse the material ID
                let uuid = Uuid::parse_str(&material_id)
                    .map_err(|_| AppError::DbError("Invalid material ID format".to_string()))?;

                // Parse timestamps
                let session_start_dt = chrono::DateTime::parse_from_rfc3339(&session_start)
                    .map_err(|_| AppError::DbError("Invalid session start timestamp".to_string()))?
                    .with_timezone(&chrono::Utc);

                let session_end_dt = chrono::DateTime::parse_from_rfc3339(&session_end)
                    .map_err(|_| AppError::DbError("Invalid session end timestamp".to_string()))?
                    .with_timezone(&chrono::Utc);

                // Create the flashcard review session request
                let flashcard_request = CreateFlashcardReviewSessionRequest {
                    material_id: uuid,
                    session_start: session_start_dt,
                    session_end: session_end_dt,
                    total_duration_seconds,
                    cards_studied,
                    again_count,
                    hard_count,
                    good_count,
                    easy_count,
                    new_cards_count,
                    learning_cards_count,
                    review_cards_count,
                    retention_rate,
                    average_response_time_seconds,
                    completed,
                };

                let session = self.create_flashcard_review_session(flashcard_request)?;
                Ok(session.session.id.to_string())
            }

            CreateReviewSessionRequest::Test {
                material_id,
                session_start,
                session_end,
                total_duration_seconds,
                questions_answered,
                correct_answers,
                incorrect_answers,
                skipped_answers,
                score_percentage,
                time_per_question_seconds,
                completed,
            } => {
                // Parse the material ID
                let uuid = Uuid::parse_str(&material_id)
                    .map_err(|_| AppError::DbError("Invalid material ID format".to_string()))?;

                // Parse timestamps
                let session_start_dt = chrono::DateTime::parse_from_rfc3339(&session_start)
                    .map_err(|_| AppError::DbError("Invalid session start timestamp".to_string()))?
                    .with_timezone(&chrono::Utc);

                let session_end_dt = chrono::DateTime::parse_from_rfc3339(&session_end)
                    .map_err(|_| AppError::DbError("Invalid session end timestamp".to_string()))?
                    .with_timezone(&chrono::Utc);

                // Create the test review session request
                let test_request = CreateTestReviewSessionRequest {
                    material_id: uuid,
                    session_start: session_start_dt,
                    session_end: session_end_dt,
                    total_duration_seconds,
                    questions_answered,
                    correct_answers,
                    incorrect_answers,
                    skipped_answers,
                    score_percentage,
                    time_per_question_seconds,
                    completed,
                };

                let session = self.create_test_review_session(test_request)?;
                Ok(session.session.id)
            }
        }
    }
}
