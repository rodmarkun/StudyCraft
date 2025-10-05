use super::connection::DbService;
use super::queries;
use crate::errors::{AppError, AppResult};
use crate::services::db_service::StudyMaterialOperations;
use rusqlite::{params, Row};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug)]
struct TagRow {
    id: i64,
    name: String,
    is_available: bool,
}

impl TagRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(TagRow {
            id: row.get("id")?,
            name: row.get("name")?,
            is_available: row.get("is_available")?,
        })
    }
}

pub trait TagOperations {
    fn get_material_tags(&self, material_id: &str) -> AppResult<Vec<String>>;
    fn get_all_tags(&self) -> AppResult<HashSet<String>>;
    fn get_available_tags(&self) -> AppResult<Vec<String>>;
    fn add_available_tag(&self, tag: String) -> AppResult<()>;
    fn remove_available_tag(&self, tag: &str) -> AppResult<()>;
    fn add_tag_to_material(&self, file_path: &str, tag: String) -> AppResult<()>;
    fn remove_tag_from_material(&self, file_path: &str, tag: &str) -> AppResult<()>;
}

impl TagOperations for DbService {
    fn get_material_tags(&self, material_id: &str) -> AppResult<Vec<String>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_TAGS_BY_MATERIAL)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let tag_iter = stmt
            .query_map(params![material_id], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::DbError(format!("Failed to query tags: {}", e)))?;

        let mut tags = Vec::new();
        for tag_result in tag_iter {
            tags.push(
                tag_result
                    .map_err(|e| AppError::DbError(format!("Failed to process tag: {}", e)))?,
            );
        }

        Ok(tags)
    }

    fn get_all_tags(&self) -> AppResult<HashSet<String>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_ALL_TAGS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let tag_iter = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::DbError(format!("Failed to query tags: {}", e)))?;

        let mut tags = HashSet::new();
        for tag_result in tag_iter {
            tags.insert(
                tag_result
                    .map_err(|e| AppError::DbError(format!("Failed to process tag: {}", e)))?,
            );
        }

        Ok(tags)
    }

    fn get_available_tags(&self) -> AppResult<Vec<String>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_AVAILABLE_TAGS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let tag_iter = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::DbError(format!("Failed to query available tags: {}", e)))?;

        let mut tags = Vec::new();
        for tag_result in tag_iter {
            tags.push(
                tag_result
                    .map_err(|e| AppError::DbError(format!("Failed to process tag: {}", e)))?,
            );
        }

        Ok(tags)
    }

    fn add_available_tag(&self, tag: String) -> AppResult<()> {
        if tag.trim().is_empty() {
            return Err(AppError::InvalidInput("Tag cannot be empty".to_string()));
        }

        let conn = self.conn.borrow_mut();
        conn.execute(queries::INSERT_AVAILABLE_TAG, params![tag.trim()])
            .map_err(|e| AppError::DbError(format!("Failed to add available tag: {}", e)))?;

        Ok(())
    }

    fn remove_available_tag(&self, tag: &str) -> AppResult<()> {
        let conn = self.conn.borrow_mut();
        conn.execute(queries::DELETE_AVAILABLE_TAG, params![tag])
            .map_err(|e| AppError::DbError(format!("Failed to remove available tag: {}", e)))?;

        Ok(())
    }

    fn add_tag_to_material(&self, file_path: &str, tag: String) -> AppResult<()> {
        if tag.trim().is_empty() {
            return Err(AppError::InvalidInput("Tag cannot be empty".to_string()));
        }

        // Get the material by file name
        let material = self
            .get_study_material_by_file_name(file_path)?
            .ok_or_else(|| {
                AppError::NotFound(format!("Study material not found: {}", file_path))
            })?;

        let material_id = &material.id;

        let mut conn = self.conn.borrow_mut();
        // Start a transaction
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        // Ensure the tag exists in the tags table
        tx.execute(queries::INSERT_TAG, params![tag.trim()])
            .map_err(|e| AppError::DbError(format!("Failed to insert tag: {}", e)))?;

        // Get the tag ID
        let tag_id: i64 = tx
            .query_row(queries::GET_TAG_ID, params![tag.trim()], |row| row.get(0))
            .map_err(|e| AppError::DbError(format!("Failed to get tag ID: {}", e)))?;

        // Check if the material already has this tag
        let has_tag = tx
            .query_row(
                queries::CHECK_MATERIAL_HAS_TAG,
                params![material_id, tag_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !has_tag {
            // Add the tag to the material
            tx.execute(queries::INSERT_MATERIAL_TAG, params![material_id, tag_id])
                .map_err(|e| AppError::DbError(format!("Failed to link material to tag: {}", e)))?;
        }

        // Commit the transaction
        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    fn remove_tag_from_material(&self, file_path: &str, tag: &str) -> AppResult<()> {
        // Get the material by file name
        let material = self
            .get_study_material_by_file_name(file_path)?
            .ok_or_else(|| {
                AppError::NotFound(format!("Study material not found: {}", file_path))
            })?;

        let material_id = &material.id;

        let conn = self.conn.borrow();
        // Get the tag ID
        let tag_id: Result<i64, rusqlite::Error> =
            conn.query_row(queries::GET_TAG_ID, params![tag], |row| row.get(0));

        match tag_id {
            Ok(id) => {
                drop(conn); // Release the read borrow before getting a write borrow
                let conn = self.conn.borrow_mut();
                // Remove the tag from the material
                conn.execute(queries::DELETE_MATERIAL_TAG, params![material_id, id])
                    .map_err(|e| {
                        AppError::DbError(format!("Failed to remove tag from material: {}", e))
                    })?;

                Ok(())
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Tag doesn't exist, which means it's not associated with the material
                Ok(())
            }
            Err(e) => Err(AppError::DbError(format!("Failed to get tag ID: {}", e))),
        }
    }
}
