use super::queries::*;
use crate::config::APP_PATHS;
use crate::errors::{AppError, AppResult};
use rusqlite::Connection;
use std::cell::RefCell;

pub use super::flashcard_decks::FlashcardDeckOperations;
pub use super::review_materials::ReviewMaterialOperations;
pub use super::review_sessions::ReviewSessionOperations;
pub use super::study_materials::StudyMaterialOperations;
pub use super::tags::TagOperations;
pub use super::tests::TestOperations;

pub struct DbService {
    pub conn: RefCell<Connection>,
}

impl DbService {
    pub fn new() -> AppResult<Self> {
        APP_PATHS
            .ensure_dirs_exist()
            .map_err(|e| AppError::IoError(e))?;
        let db_path = APP_PATHS.config_dir.join("studycraft.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::DbError(format!("Failed to open database: {}", e)))?;
        let service = Self {
            conn: RefCell::new(conn),
        };
        service.setup_tables()?;
        Ok(service)
    }

    fn setup_tables(&self) -> AppResult<()> {
        // Create study materials table
        self.conn
            .borrow()
            .execute(CREATE_STUDY_MATERIALS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!("Failed to create study_materials table: {}", e))
            })?;

        // Create review materials table
        self.conn
            .borrow()
            .execute(CREATE_REVIEW_MATERIALS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!("Failed to create review_materials table: {}", e))
            })?;

        // Create flashcards table
        self.conn
            .borrow()
            .execute(CREATE_FLASHCARDS_TABLE, [])
            .map_err(|e| AppError::DbError(format!("Failed to create flashcards table: {}", e)))?;

        // Create test questions table
        self.conn
            .borrow()
            .execute(CREATE_TEST_QUESTIONS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!("Failed to create test_questions table: {}", e))
            })?;

        // Create test answers table
        self.conn
            .borrow()
            .execute(CREATE_TEST_ANSWERS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!("Failed to create test_answers table: {}", e))
            })?;

        // Create tags table
        self.conn
            .borrow()
            .execute(CREATE_TAGS_TABLE, [])
            .map_err(|e| AppError::DbError(format!("Failed to create tags table: {}", e)))?;

        // Create material tags table
        self.conn
            .borrow()
            .execute(CREATE_MATERIAL_TAGS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!("Failed to create material_tags table: {}", e))
            })?;

        // Create general review sessions table
        self.conn
            .borrow()
            .execute(CREATE_REVIEW_SESSIONS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!("Failed to create review_sessions table: {}", e))
            })?;

        // Create flashcard review sessions table
        self.conn
            .borrow()
            .execute(CREATE_FLASHCARD_REVIEW_SESSIONS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!(
                    "Failed to create flashcard_review_sessions table: {}",
                    e
                ))
            })?;

        // Create test review sessions table
        self.conn
            .borrow()
            .execute(CREATE_TEST_REVIEW_SESSIONS_TABLE, [])
            .map_err(|e| {
                AppError::DbError(format!(
                    "Failed to create test_review_sessions table: {}",
                    e
                ))
            })?;

        Ok(())
    }
}
