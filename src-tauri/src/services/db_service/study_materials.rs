use super::connection::DbService;
use super::queries;
use crate::config::APP_PATHS;
use crate::errors::{AppError, AppResult};
use crate::materials::study_material::StudyMaterial;
use rusqlite::{params, Row};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug)]
struct StudyMaterialRow {
    id: String,
    display_name: String,
    original_path: String,
    markdown_path: String,
    cover_path: Option<String>,
    is_processing: bool,
    upload_date: String,
    extra_instructions: String,
    is_indexed: bool,
    chunk_count: Option<i32>,
    last_indexed: Option<String>,
    url: String,
}

impl StudyMaterialRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(StudyMaterialRow {
            id: row.get("id")?,
            display_name: row.get("name")?,
            original_path: row.get("original_path")?,
            markdown_path: row.get("markdown_path")?,
            cover_path: row.get("cover_path")?,
            is_processing: row.get("is_processing")?,
            upload_date: row.get("created_at")?,
            extra_instructions: row.get("extra_instructions")?,
            is_indexed: row.get("is_indexed").unwrap_or(false),
            chunk_count: row.get("chunk_count").ok(),
            last_indexed: row.get("last_indexed").ok(),
            url: row.get("url")?,
        })
    }

    fn into_study_material(self, tags: HashSet<String>) -> StudyMaterial {
        StudyMaterial {
            id: self.id,
            display_name: self.display_name,
            original_path: PathBuf::from(self.original_path),
            markdown_path: PathBuf::from(self.markdown_path),
            cover_path: self.cover_path.map(PathBuf::from),
            is_processing: self.is_processing,
            tags,
            upload_date: self.upload_date,
            extra_instructions: self.extra_instructions,
            is_indexed: self.is_indexed,
            chunk_count: self.chunk_count.map(|c| c as u32),
            last_indexed: self.last_indexed,
            url: self.url,
        }
    }
}

pub trait StudyMaterialOperations {
    fn add_study_material(&self, material: &StudyMaterial) -> AppResult<()>;
    fn get_study_materials(&self) -> AppResult<Vec<StudyMaterial>>;
    fn get_study_material_by_id(&self, id: &str) -> AppResult<Option<StudyMaterial>>;
    fn get_study_material_by_file_name(&self, file_name: &str) -> AppResult<Option<StudyMaterial>>;
    fn delete_study_material(&self, id: &str) -> AppResult<(String, String, Option<String>)>;
    fn update_study_material_processing_state(
        &self,
        id: &str,
        is_processing: bool,
    ) -> AppResult<()>;
    fn update_study_material_name(&self, id: &str, new_name: &str) -> AppResult<()>;
    fn update_study_material_indexed_state(
        &self,
        material_id: &str,
        is_indexed: bool,
        chunk_count: u32,
    ) -> AppResult<()>;
}

impl StudyMaterialOperations for DbService {
    fn add_study_material(&self, material: &StudyMaterial) -> AppResult<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        let original_path_str = material.original_path.to_string_lossy().to_string();
        let markdown_path_str = material.markdown_path.to_string_lossy().to_string();
        let cover_path_str = material
            .cover_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        tx.execute(
            queries::INSERT_STUDY_MATERIAL,
            params![
                material.id,
                material.display_name,
                original_path_str,
                markdown_path_str,
                cover_path_str,
                material.is_processing,
                material.extra_instructions,
                material.url
            ],
        )
        .map_err(|e| AppError::DbError(format!("Failed to insert study material: {}", e)))?;

        for tag in &material.tags {
            tx.execute(queries::INSERT_TAG, params![tag])
                .map_err(|e| AppError::DbError(format!("Failed to insert tag: {}", e)))?;

            let tag_id: i64 = tx
                .query_row(queries::GET_TAG_ID, params![tag], |row| row.get(0))
                .map_err(|e| AppError::DbError(format!("Failed to get tag ID: {}", e)))?;

            tx.execute(queries::INSERT_MATERIAL_TAG, params![material.id, tag_id])
                .map_err(|e| AppError::DbError(format!("Failed to link material to tag: {}", e)))?;
        }

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    fn get_study_materials(&self) -> AppResult<Vec<StudyMaterial>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_ALL_STUDY_MATERIALS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let material_iter = stmt
            .query_map([], |row| StudyMaterialRow::from_row(row))
            .map_err(|e| AppError::DbError(format!("Failed to query study materials: {}", e)))?;

        let mut materials = Vec::new();
        for material_result in material_iter {
            let row = material_result.map_err(|e| {
                AppError::DbError(format!("Failed to process study material row: {}", e))
            })?;

            let mut tag_stmt = conn
                .prepare(queries::GET_TAGS_BY_MATERIAL)
                .map_err(|e| AppError::DbError(format!("Failed to prepare tags query: {}", e)))?;

            let tag_iter = tag_stmt
                .query_map([&row.id], |row| row.get::<_, String>(0))
                .map_err(|e| AppError::DbError(format!("Failed to query tags: {}", e)))?;

            let mut tags = HashSet::new();
            for tag_result in tag_iter {
                tags.insert(
                    tag_result
                        .map_err(|e| AppError::DbError(format!("Failed to process tag: {}", e)))?,
                );
            }

            materials.push(row.into_study_material(tags));
        }

        Ok(materials)
    }

    fn get_study_material_by_id(&self, id: &str) -> AppResult<Option<StudyMaterial>> {
        let conn = self.conn.borrow();

        let row_result = conn.query_row(queries::GET_STUDY_MATERIAL_BY_ID, params![id], |row| {
            StudyMaterialRow::from_row(row)
        });

        match row_result {
            Ok(row) => {
                let mut tag_stmt = conn.prepare(queries::GET_TAGS_BY_MATERIAL).map_err(|e| {
                    AppError::DbError(format!("Failed to prepare tags query: {}", e))
                })?;

                let tag_iter = tag_stmt
                    .query_map([&row.id], |row| row.get::<_, String>(0))
                    .map_err(|e| AppError::DbError(format!("Failed to query tags: {}", e)))?;

                let mut tags = HashSet::new();
                for tag_result in tag_iter {
                    tags.insert(
                        tag_result.map_err(|e| {
                            AppError::DbError(format!("Failed to process tag: {}", e))
                        })?,
                    );
                }

                Ok(Some(row.into_study_material(tags)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::DbError(format!(
                "Failed to query study material: {}",
                e
            ))),
        }
    }

    fn get_study_material_by_file_name(&self, file_name: &str) -> AppResult<Option<StudyMaterial>> {
        let materials_dir = APP_PATHS.materials_dir.to_string_lossy();
        let search_pattern = format!("%{}%{}%", materials_dir, file_name);
        let conn = self.conn.borrow();

        let row_result = conn.query_row(
            queries::GET_STUDY_MATERIAL_BY_FILENAME,
            params![search_pattern],
            |row| StudyMaterialRow::from_row(row),
        );

        match row_result {
            Ok(row) => {
                let mut tag_stmt = conn.prepare(queries::GET_TAGS_BY_MATERIAL).map_err(|e| {
                    AppError::DbError(format!("Failed to prepare tags query: {}", e))
                })?;

                let tag_iter = tag_stmt
                    .query_map([&row.id], |row| row.get::<_, String>(0))
                    .map_err(|e| AppError::DbError(format!("Failed to query tags: {}", e)))?;

                let mut tags = HashSet::new();
                for tag_result in tag_iter {
                    tags.insert(
                        tag_result.map_err(|e| {
                            AppError::DbError(format!("Failed to process tag: {}", e))
                        })?,
                    );
                }

                Ok(Some(row.into_study_material(tags)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::DbError(format!(
                "Failed to query study material: {}",
                e
            ))),
        }
    }

    fn delete_study_material(&self, id: &str) -> AppResult<(String, String, Option<String>)> {
        let material = self
            .get_study_material_by_id(id)?
            .ok_or_else(|| AppError::NotFound(format!("Study material not found: {}", id)))?;

        let paths = (
            material.original_path.to_string_lossy().into_owned(),
            material.markdown_path.to_string_lossy().into_owned(),
            material
                .cover_path
                .map(|p| p.to_string_lossy().into_owned()),
        );

        let conn = self.conn.borrow_mut();
        conn.execute(queries::DELETE_MATERIAL_TAGS, params![id])
            .map_err(|e| AppError::DbError(format!("Failed to delete material tags: {}", e)))?;

        conn.execute(queries::DELETE_STUDY_MATERIAL, params![id])
            .map_err(|e| AppError::DbError(format!("Failed to delete study material: {}", e)))?;

        Ok(paths)
    }

    fn update_study_material_processing_state(
        &self,
        id: &str,
        is_processing: bool,
    ) -> AppResult<()> {
        let conn = self.conn.borrow_mut();
        conn.execute(
            queries::UPDATE_STUDY_MATERIAL_PROCESSING,
            params![is_processing, id],
        )
        .map_err(|e| AppError::DbError(format!("Failed to update processing state: {}", e)))?;
        Ok(())
    }

    fn update_study_material_name(&self, id: &str, new_name: &str) -> AppResult<()> {
        let conn = self.conn.borrow_mut();
        conn.execute(queries::UPDATE_STUDY_MATERIAL_NAME, params![new_name, id])
            .map_err(|e| AppError::DbError(format!("Failed to update name: {}", e)))?;
        Ok(())
    }

    fn update_study_material_indexed_state(
        &self,
        material_id: &str,
        is_indexed: bool,
        chunk_count: u32,
    ) -> AppResult<()> {
        let conn = self.conn.borrow_mut();
        conn.execute(
            queries::UPDATE_STUDY_MATERIAL_INDEXED_STATE,
            params![is_indexed, chunk_count as i32, material_id],
        )
        .map_err(|e| AppError::DbError(format!("Failed to update indexed state: {}", e)))?;
        Ok(())
    }
}
