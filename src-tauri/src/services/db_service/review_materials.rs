use super::connection::DbService;
use super::queries;
use crate::errors::{AppError, AppResult};
use crate::materials::review_material;
use crate::materials::review_material::ReviewMaterial;
use rusqlite::{params, Row, Transaction};

#[derive(Debug)]
struct ReviewMaterialRow {
    id: String,
    name: String,
    rm_type: String,
    last_review: Option<String>,
    created_at: String,
    updated_at: String,
    deleted: bool,
}

impl ReviewMaterialRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(ReviewMaterialRow {
            id: row.get("id")?,
            name: row.get("name")?,
            rm_type: row.get("type")?,
            last_review: row.get("last_review")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            deleted: row.get("deleted")?,
        })
    }

    fn into_review_material(self) -> AppResult<ReviewMaterial> {
        let last_review = if let Some(lr_str) = self.last_review {
            Some(
                chrono::NaiveDateTime::parse_from_str(&lr_str, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| AppError::DbError(format!("Failed to parse last_review: {}", e)))?
                    .and_utc(),
            )
        } else {
            None
        };

        let created_at =
            chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse created_at: {}", e)))?
                .and_utc();

        let updated_at =
            chrono::NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse updated_at: {}", e)))?
                .and_utc();

        Ok(ReviewMaterial {
            id: self.id,
            display_name: self.name,
            rm_type: review_material::ReviewMaterialType::from(self.rm_type),
            last_review,
            created_at,
            updated_at,
        })
    }
}

pub trait ReviewMaterialOperations {
    fn add_review_material_tx(&self, tx: &Transaction, material: &ReviewMaterial) -> AppResult<()>;
    fn get_review_materials(&self) -> AppResult<Vec<ReviewMaterial>>;
    fn get_review_material(&self, id: &str) -> AppResult<Option<ReviewMaterial>>;
    fn delete_review_material_tx(&self, tx: &Transaction, id: &str) -> AppResult<()>;
    fn update_review_material_name_tx(
        &self,
        tx: &Transaction,
        id: &str,
        new_name: &str,
    ) -> AppResult<()>;
    fn update_review_material_tx(&self, tx: &Transaction, id: &str) -> AppResult<()>;
    fn update_review_material_last_review(&mut self, id: &str) -> AppResult<()>;
}

impl ReviewMaterialOperations for DbService {
    fn add_review_material_tx(&self, tx: &Transaction, material: &ReviewMaterial) -> AppResult<()> {
        let last_review_str = material
            .last_review
            .map(|lr| lr.format("%Y-%m-%d %H:%M:%S").to_string());

        tx.execute(
            queries::INSERT_REVIEW_MATERIAL,
            params![
                material.id,
                material.display_name,
                String::from(material.rm_type.clone()),
                last_review_str
            ],
        )
        .map_err(|e| AppError::DbError(format!("Failed to insert review material: {}", e)))?;

        Ok(())
    }

    fn get_review_materials(&self) -> AppResult<Vec<ReviewMaterial>> {
        let conn = self.conn.borrow();
        let mut stmt = conn
            .prepare(queries::GET_ALL_REVIEW_MATERIALS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let material_iter = stmt
            .query_map([], |row| ReviewMaterialRow::from_row(row))
            .map_err(|e| AppError::DbError(format!("Failed to query review materials: {}", e)))?;

        let mut materials = Vec::new();
        for material_result in material_iter {
            let row = material_result.map_err(|e| {
                AppError::DbError(format!("Failed to process review material row: {}", e))
            })?;
            materials.push(row.into_review_material()?);
        }

        Ok(materials)
    }

    fn get_review_material(&self, id: &str) -> AppResult<Option<ReviewMaterial>> {
        let result =
            self.conn
                .borrow()
                .query_row(queries::GET_REVIEW_MATERIAL_BY_ID, params![id], |row| {
                    ReviewMaterialRow::from_row(row)
                });

        match result {
            Ok(row) => Ok(Some(row.into_review_material()?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::DbError(format!(
                "Failed to query review material: {}",
                e
            ))),
        }
    }

    fn delete_review_material_tx(&self, tx: &Transaction, id: &str) -> AppResult<()> {
        tx.execute(queries::DELETE_REVIEW_MATERIAL, params![id])
            .map_err(|e| AppError::DbError(format!("Failed to delete review material: {}", e)))?;

        Ok(())
    }

    fn update_review_material_name_tx(
        &self,
        tx: &Transaction,
        id: &str,
        new_name: &str,
    ) -> AppResult<()> {
        let rows_affected = tx
            .execute(queries::UPDATE_REVIEW_MATERIAL_NAME, params![new_name, id])
            .map_err(|e| {
                AppError::DbError(format!("Failed to update review material name: {}", e))
            })?;

        if rows_affected == 0 {
            return Err(AppError::DbError("Review material not found".to_string()));
        }

        Ok(())
    }

    fn update_review_material_last_review(&mut self, id: &str) -> AppResult<()> {
        self.conn
            .borrow()
            .execute(queries::UPDATE_LAST_REVIEW, params![id])
            .map_err(|e| AppError::DbError(format!("Failed to update last review: {}", e)))?;

        Ok(())
    }

    fn update_review_material_tx(&self, tx: &Transaction, id: &str) -> AppResult<()> {
        tx.execute(queries::UPDATE_REVIEW_MATERIAL_TIMESTAMP, params![id])
            .map_err(|e| {
                AppError::DbError(format!("Failed to update review material timestamp: {}", e))
            })?;

        Ok(())
    }
}
