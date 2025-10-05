use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::params;
use pulldown_cmark::{Parser, Event, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use sqlite_vec::sqlite3_vec_init;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::APP_PATHS;
use crate::constants::{
    self, VECDB_BATCH_SIZE, VECDB_MAX_CHUNKS_PER_MATERIAL, VECDB_MAX_CHUNK_SIZE,
    VECDB_MAX_FILE_SIZE, VECDB_OVERLAP,
};
use crate::errors::{AppError, AppResult};
use crate::materials::study_material::StudyMaterial;
use crate::services::vector_service::embedding_provider::LocalEmbeddingProvider;
use crate::services::vector_service::queries;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkMetadata {
    pub chunk_id: String,
    pub material_id: String,
    pub file_display_name: String,
    pub chunk_index: usize,
    pub chunk_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantChunk {
    pub chunk_id: String,
    pub text: String,
    pub score: f32,
    pub metadata: ChunkMetadata,
}

type DbPool = Pool<SqliteConnectionManager>;
type DbConnection = PooledConnection<SqliteConnectionManager>;

pub struct VectorService {
    pool: Arc<DbPool>,
    embedding_provider: Arc<LocalEmbeddingProvider>,
}

impl VectorService {
    pub async fn new() -> AppResult<Self> {
        println!("Initializing SQLite vector service with sqlite-vec...");

        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }

        let embedding_provider = Arc::new(LocalEmbeddingProvider::new());

        let storage_path = APP_PATHS.base_dir.join("vector_data");
        std::fs::create_dir_all(&storage_path).map_err(|e| {
            AppError::VectorError(format!("Failed to create vector storage directory: {}", e))
        })?;

        let db_path = storage_path.join("vectors.db");
        let manager = SqliteConnectionManager::file(&db_path).with_init(|c| {
            c.execute_batch(queries::PRAGMA_SETTINGS)?;
            Ok(())
        });

        let pool = Pool::builder().max_size(3).build(manager).map_err(|e| {
            AppError::VectorError(format!("Failed to create connection pool: {}", e))
        })?;

        let service = Self {
            pool: Arc::new(pool),
            embedding_provider,
        };

        service.setup_tables().await?;

        if service.embedding_provider.is_model_downloaded() {
            println!("Embedding model found, initializing...");
            if let Err(e) = service.embedding_provider.initialize_model().await {
                println!("Warning: Failed to initialize existing model: {}", e);
            }
        } else {
            println!("Embedding model not found - will initialize when requested");
        }

        let count = service.get_vector_count().await?;
        println!("Vector service initialized with {} vectors", count);

        Ok(service)
    }

    pub async fn initialize_embedding_model(&self) -> AppResult<()> {
        self.embedding_provider.initialize_model().await
    }

    pub fn is_embedding_model_ready(&self) -> bool {
        self.embedding_provider.is_initialized()
    }

    fn get_connection(&self) -> AppResult<DbConnection> {
        self.pool
            .get()
            .map_err(|e| AppError::VectorError(format!("Failed to get database connection: {}", e)))
    }

    async fn setup_tables(&self) -> AppResult<()> {
        let conn = self.get_connection()?;

        // Main chunks table (metadata only)
        conn.execute(queries::CREATE_CHUNKS_TABLE, [])
            .map_err(|e| AppError::VectorError(format!("Failed to create chunks table: {}", e)))?;

        // sqlite-vec virtual table for embeddings
        conn.execute(queries::CREATE_VEC_CHUNKS_TABLE, [])
            .map_err(|e| AppError::VectorError(format!("Failed to create vector table: {}", e)))?;

        // Indexes
        conn.execute(queries::CREATE_CHUNKS_MATERIAL_ID_INDEX, [])
            .map_err(|e| {
                AppError::VectorError(format!("Failed to create material_id index: {}", e))
            })?;

        conn.execute(queries::CREATE_CHUNKS_LENGTH_INDEX, [])
            .map_err(|e| AppError::VectorError(format!("Failed to create length index: {}", e)))?;

        conn.execute(queries::CREATE_CHUNKS_FTS_TABLE, [])
            .map_err(|e| AppError::VectorError(format!("Failed to create FTS table: {}", e)))?;

        conn.execute(queries::CREATE_FTS_INSERT_TRIGGER, [])
            .map_err(|e| {
                AppError::VectorError(format!("Failed to create FTS insert trigger: {}", e))
            })?;

        conn.execute(queries::CREATE_FTS_DELETE_TRIGGER, [])
            .map_err(|e| {
                AppError::VectorError(format!("Failed to create FTS delete trigger: {}", e))
            })?;

        conn.execute(queries::CREATE_FTS_UPDATE_TRIGGER, [])
            .map_err(|e| {
                AppError::VectorError(format!("Failed to create FTS update trigger: {}", e))
            })?;

        Ok(())
    }

    async fn get_vector_count(&self) -> AppResult<usize> {
        let conn = self.get_connection()?;
        let count: i64 = conn
            .query_row(queries::COUNT_ALL_CHUNKS, [], |row| row.get(0))
            .map_err(|e| AppError::VectorError(format!("Failed to get vector count: {}", e)))?;

        Ok(count as usize)
    }

    pub async fn index_material(
        &mut self,
        study_material: StudyMaterial,
    ) -> AppResult<Vec<String>> {
        println!(
            "Indexing material: {} ({})",
            study_material.display_name, study_material.id
        );

        let file_metadata = std::fs::metadata(study_material.markdown_path.clone())
            .map_err(|e| AppError::VectorError(format!("Failed to read file metadata: {}", e)))?;

        let file_size = file_metadata.len();
        if file_size > VECDB_MAX_FILE_SIZE {
            return Err(AppError::VectorError(format!(
                "File {} is too large ({} bytes). Maximum size for Markdown is {} MB.",
                study_material.display_name,
                file_size,
                VECDB_MAX_FILE_SIZE / (1024 * 1024)
            )));
        }

        let markdown_path_string = study_material.markdown_path.to_string_lossy().to_string();
        let chunks =
            tokio::task::spawn_blocking(move || Self::chunk_markdown(&markdown_path_string))
                .await
                .map_err(|e| AppError::VectorError(format!("Chunking task failed: {}", e)))??;

        if chunks.is_empty() {
            println!(
                "No chunks found for material: {}",
                study_material.display_name
            );
            return Ok(Vec::new());
        }

        let chunks = if chunks.len() > VECDB_MAX_CHUNKS_PER_MATERIAL {
            println!(
                "Warning: Material {} has {} chunks, limiting to {}",
                study_material.display_name,
                chunks.len(),
                VECDB_MAX_CHUNKS_PER_MATERIAL
            );
            chunks
                .into_iter()
                .take(VECDB_MAX_CHUNKS_PER_MATERIAL)
                .collect()
        } else {
            chunks
        };

        let chunks: Vec<String> = chunks
            .into_iter()
            .filter_map(|chunk| {
                let trimmed = chunk.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect();

        println!(
            "Processing {} valid chunks for material: {}",
            chunks.len(),
            study_material.display_name
        );

        let mut all_chunk_ids = Vec::new();
        let material_id = study_material.id.to_string();
        let file_display_name = study_material.display_name.to_string();

        for (batch_index, batch_chunks) in chunks.chunks(VECDB_BATCH_SIZE).enumerate() {
            println!(
                "Processing batch {} of {} chunks",
                batch_index + 1,
                batch_chunks.len()
            );
            let batch_chunks = batch_chunks.to_vec();
            let embedding_provider = self.embedding_provider.clone();

            let batch_data = tokio::task::spawn_blocking(move || {
                Self::process_batch_embeddings(batch_chunks, batch_index, &embedding_provider)
            })
            .await
            .map_err(|e| AppError::VectorError(format!("Embedding batch task failed: {}", e)))??;

            if batch_data.is_empty() {
                println!("No valid embeddings in batch {}, skipping", batch_index + 1);
                continue;
            }

            match self
                .insert_batch(&material_id, &file_display_name, batch_data)
                .await
            {
                Ok(chunk_ids) => {
                    all_chunk_ids.extend(chunk_ids.clone());
                    println!(
                        "Successfully inserted batch {} with {} chunks",
                        batch_index + 1,
                        chunk_ids.len()
                    );
                }
                Err(e) => {
                    eprintln!("Failed to insert batch {}: {}", batch_index + 1, e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        println!(
            "Successfully indexed {} total chunks for material: {}",
            all_chunk_ids.len(),
            file_display_name
        );
        Ok(all_chunk_ids)
    }

    fn process_batch_embeddings(
        batch_chunks: Vec<String>,
        batch_index: usize,
        embedding_provider: &LocalEmbeddingProvider,
    ) -> AppResult<Vec<(String, usize, String, Vec<f32>, i32)>> {
        let mut batch_data = Vec::new();

        for (local_index, chunk_text) in batch_chunks.iter().enumerate() {
            if chunk_text.trim().is_empty() {
                continue;
            }

            let global_index = batch_index * VECDB_BATCH_SIZE + local_index;
            let chunk_id = Uuid::new_v4().to_string();

            let embedding = match embedding_provider.create_embedding(chunk_text) {
                Ok(emb) => emb,
                Err(e) => {
                    eprintln!(
                        "Failed to create embedding for chunk {}: {}",
                        global_index, e
                    );
                    continue;
                }
            };

            let text_length = chunk_text.len() as i32;

            batch_data.push((
                chunk_id,
                global_index,
                chunk_text.clone(),
                embedding,
                text_length,
            ));
        }

        Ok(batch_data)
    }

    async fn insert_batch(
        &self,
        material_id: &str,
        file_display_name: &str,
        batch_data: Vec<(String, usize, String, Vec<f32>, i32)>,
    ) -> AppResult<Vec<String>> {
        let mut conn = self.get_connection()?;

        let tx = conn
            .transaction()
            .map_err(|e| AppError::VectorError(format!("Failed to start transaction: {}", e)))?;

        let mut chunk_ids = Vec::new();
        let mut success_count = 0;

        for (chunk_id, index, chunk_text, embedding, text_length) in batch_data {
            // Insert into chunks table
            match tx.execute(
                queries::INSERT_CHUNK,
                params![
                    chunk_id.clone(),
                    material_id,
                    file_display_name,
                    index as i32,
                    chunk_text,
                    text_length
                ],
            ) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to insert chunk metadata {}: {}", index, e);
                    continue;
                }
            }

            // Insert into vector table using sqlite-vec
            match tx.execute(
                queries::INSERT_VECTOR,
                params![
                    chunk_id.clone(),
                    Self::vec_f32_to_bytes(&embedding) // Use zerocopy for efficient conversion
                ],
            ) {
                Ok(_) => {
                    chunk_ids.push(chunk_id);
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to insert vector for chunk {}: {}", index, e);
                    // Try to clean up the metadata entry
                    let _ = tx.execute("DELETE FROM chunks WHERE id = ?1", params![chunk_id]);
                }
            }
        }

        tx.commit().map_err(|e| {
            AppError::VectorError(format!("Failed to commit batch transaction: {}", e))
        })?;

        println!(
            "Successfully inserted {}/{} chunks in batch",
            success_count,
            chunk_ids.len()
        );
        Ok(chunk_ids)
    }

    pub async fn search_global(
        &self,
        query: &str,
        max_chunks: u32,
    ) -> AppResult<Vec<RelevantChunk>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        if query.len() > 1000 {
            return Err(AppError::VectorError(
                "Query too long (max 1000 characters)".to_string(),
            ));
        }

        println!(
            "🔍 DEBUG: Starting sqlite-vec search for query: '{}'",
            query
        );

        let conn = self.get_connection()?;

        // Generate query embedding
        let query_embedding = self.embedding_provider.create_embedding(query)?;

        // Use sqlite-vec's vector similarity search
        let mut stmt = conn.prepare(queries::GLOBAL_VECTOR_SEARCH)?;

        let results = stmt.query_map(
            params![Self::vec_f32_to_bytes(&query_embedding), max_chunks as i64],
            |row| {
                let chunk_id: String = row.get(0)?;
                let material_id: String = row.get(1)?;
                let file_display_name: String = row.get(2)?;
                let chunk_index: i32 = row.get(3)?;
                let chunk_text: String = row.get(4)?;
                let distance: f32 = row.get(5)?;
                Ok((
                    chunk_id,
                    material_id,
                    file_display_name,
                    chunk_index,
                    chunk_text,
                    distance,
                ))
            },
        )?;

        let mut relevant_chunks = Vec::new();

        for result in results {
            let (chunk_id, material_id, file_display_name, chunk_index, chunk_text, distance) =
                result.map_err(|e| {
                    AppError::VectorError(format!("Failed to process search result: {}", e))
                })?;

            // Convert distance to similarity score (sqlite-vec typically returns distance, lower = more similar)
            let score = 1.0 - (distance / 2.0);

            let metadata = ChunkMetadata {
                chunk_id: chunk_id.clone(),
                material_id,
                file_display_name,
                chunk_index: chunk_index as usize,
                chunk_text: chunk_text.clone(),
            };

            relevant_chunks.push(RelevantChunk {
                chunk_id,
                text: chunk_text,
                score,
                metadata,
            });
        }

        println!(
            "🔍 DEBUG: sqlite-vec returned {} results",
            relevant_chunks.len()
        );
        for (i, chunk) in relevant_chunks.iter().take(5).enumerate() {
            println!(
                "  {}. Score: {:.4}, Text: '{}'",
                i + 1,
                chunk.score,
                chunk.text.chars().take(100).collect::<String>()
            );
        }

        Ok(relevant_chunks)
    }

    pub async fn search_material_specific(
        &self,
        query: &str,
        material_ids: &[String],
        max_chunks: u32,
    ) -> AppResult<Vec<RelevantChunk>> {
        if query.trim().is_empty() || material_ids.is_empty() {
            return Ok(Vec::new());
        }

        if material_ids.len() > 100 {
            return Err(AppError::VectorError(
                "Too many material IDs (max 100)".to_string(),
            ));
        }

        let conn = self.get_connection()?;

        // Generate query embedding
        let query_embedding = self.embedding_provider.create_embedding(query)?;

        // Build the dynamic SQL query
        let sql = queries::build_material_specific_vector_search(material_ids.len());

        let mut stmt = conn.prepare(&sql)?;

        // Build parameters: embedding + material_ids + limit
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params.push(Box::new(Self::vec_f32_to_bytes(&query_embedding)));
        for material_id in material_ids {
            params.push(Box::new(material_id.clone()));
        }
        params.push(Box::new(max_chunks as i64));

        // Convert to references for the query
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let results = stmt.query_map(&param_refs[..], |row| {
            let chunk_id: String = row.get(0)?;
            let material_id: String = row.get(1)?;
            let file_display_name: String = row.get(2)?;
            let chunk_index: i32 = row.get(3)?;
            let chunk_text: String = row.get(4)?;
            let distance: f32 = row.get(5)?;
            Ok((
                chunk_id,
                material_id,
                file_display_name,
                chunk_index,
                chunk_text,
                distance,
            ))
        })?;

        let mut relevant_chunks = Vec::new();

        for result in results {
            let (chunk_id, material_id, file_display_name, chunk_index, chunk_text, distance) =
                result.map_err(|e| {
                    AppError::VectorError(format!("Failed to process search result: {}", e))
                })?;

            // Convert distance to similarity score
            let score = 1.0 - (distance / 2.0);

            let metadata = ChunkMetadata {
                chunk_id: chunk_id.clone(),
                material_id,
                file_display_name,
                chunk_index: chunk_index as usize,
                chunk_text: chunk_text.clone(),
            };

            relevant_chunks.push(RelevantChunk {
                chunk_id,
                text: chunk_text,
                score,
                metadata,
            });
        }

        for (i, chunk) in relevant_chunks.iter().take(3).enumerate() {
            println!(
                "  {}. Score: {:.4}, Material: {}, Text: '{}'",
                i + 1,
                chunk.score,
                chunk.metadata.material_id,
                chunk.text.chars().take(80).collect::<String>()
            );
        }

        Ok(relevant_chunks)
    }

    pub async fn delete_material(&mut self, material_id: &str) -> AppResult<()> {
        let conn = self.get_connection()?;

        // Get chunk IDs to delete from vector table
        let chunk_ids: Vec<String> = {
            let mut stmt = conn.prepare(queries::GET_CHUNK_IDS_BY_MATERIAL)?;
            let results =
                stmt.query_map(params![material_id], |row| Ok(row.get::<_, String>(0)?))?;

            results
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::VectorError(format!("Failed to get chunk IDs: {}", e)))?
        };

        if chunk_ids.is_empty() {
            return Ok(());
        }

        // Delete from vector table
        for chunk_id in &chunk_ids {
            conn.execute(queries::DELETE_VECTOR_BY_CHUNK_ID, params![chunk_id])?;
        }

        // Delete from chunks table
        let deleted_count = conn
            .execute(queries::DELETE_CHUNKS_BY_MATERIAL, params![material_id])
            .map_err(|e| {
                AppError::VectorError(format!("Failed to delete material chunks: {}", e))
            })?;

        if deleted_count > 0 {
            println!(
                "Deleted {} chunks for material {} from vector database",
                deleted_count, material_id
            );
        }

        Ok(())
    }

    fn chunk_markdown(file_path: &str) -> AppResult<Vec<String>> {
        let text = std::fs::read_to_string(file_path)
            .map_err(|e| AppError::VectorError(format!("Failed to read file: {}", e)))?;
        let text = text.trim();
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let mut final_chunks = Vec::new();
        let mut current_semantic_chunk = String::new();
        let parser = Parser::new(text);

        for event in parser {
            match event {
                // A heading marks a new semantic section.
                Event::Start(Tag::Heading { .. } ) | Event::Start(Tag::CodeBlock(_)) => {
                    // Process the previously accumulated chunk before starting a new one.
                    if !current_semantic_chunk.trim().is_empty() {
                        process_and_add_chunk(&mut final_chunks, std::mem::take(&mut current_semantic_chunk));
                    }
                }
                Event::Text(t) => {
                    current_semantic_chunk.push_str(&t);
                    current_semantic_chunk.push(' ');
                }
                Event::End(TagEnd::Paragraph) | Event::End(TagEnd::CodeBlock) => {
                    current_semantic_chunk.push('\n');
                }
                _ => (),
            }
        }
        // Process any remaining text at the end of the document.
        if !current_semantic_chunk.trim().is_empty() {
            process_and_add_chunk(&mut final_chunks, current_semantic_chunk);
        }
        
        Ok(final_chunks)
    }

    // Add other utility methods as needed...
    pub async fn get_stats(&self) -> AppResult<String> {
        let total_vectors = self.get_vector_count().await?;

        let conn = self.get_connection()?;
        let material_count: i64 = conn
            .query_row(queries::COUNT_DISTINCT_MATERIALS, [], |row| row.get(0))
            .map_err(|e| AppError::VectorError(format!("Failed to get material count: {}", e)))?;

        let db_path = APP_PATHS.base_dir.join("vector_data").join("vectors.db");
        let file_size = std::fs::metadata(&db_path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        let stats = format!(
            "SQLite vector service with sqlite-vec is running\n\
            Total vectors: {}\n\
            Unique materials: {}\n\
            Database size: {} KB\n\
            Database path: {:?}",
            total_vectors,
            material_count,
            file_size / 1024,
            db_path
        );

        Ok(stats)
    }

    // Helper function to convert Vec<f32> to Vec<u8> for sqlite-vec
    fn vec_f32_to_bytes(vec: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(vec.len() * 4);
        for &f in vec {
            bytes.extend_from_slice(&f.to_le_bytes());
        }
        bytes
    }
}

fn process_and_add_chunk(final_chunks: &mut Vec<String>, chunk_text: String) {
    let trimmed_chunk = chunk_text.trim();
    if trimmed_chunk.is_empty() {
        return;
    }

    if trimmed_chunk.chars().count() <= constants::VECDB_MAX_CHUNK_SIZE {
        add_or_merge_chunk(final_chunks, trimmed_chunk.to_string());
        return;
    }

    println!(
        "Warning: Semantic chunk is too large ({} chars, {} bytes). Smart splitting.",
        trimmed_chunk.chars().count(),
        trimmed_chunk.len()
    );

    let smart_chunks = split_at_smart_boundaries(trimmed_chunk, constants::VECDB_MAX_CHUNK_SIZE);

    for chunk in smart_chunks {
        if chunk.chars().count() <= constants::VECDB_MAX_CHUNK_SIZE {
            add_or_merge_chunk(final_chunks, chunk);
        } else {
            let windowed_chunks = split_large_chunk(
                &chunk,
                constants::VECDB_MAX_CHUNK_SIZE,
                constants::VECDB_OVERLAP,
            );
            for sub_chunk in windowed_chunks {
                add_or_merge_chunk(final_chunks, sub_chunk);
            }
        }
    }
}

fn split_large_chunk(text: &str, max_size: usize, overlap: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    
    let mut start_char = 0;
    while start_char < chars.len() {
        let end_char = std::cmp::min(start_char + max_size, chars.len());
        let chunk: String = chars[start_char..end_char].iter().collect();
        chunks.push(chunk);
        
        if end_char >= chars.len() {
            break;
        }
        
        // Move forward, but overlap by the specified amount
        start_char = if end_char > overlap {
            end_char - overlap
        } else {
            end_char
        };
    }
    
    chunks
}

fn split_at_smart_boundaries(text: &str, max_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut current_start = 0;
    
    while current_start < chars.len() {
        let end_char = std::cmp::min(current_start + max_size, chars.len());
        
        if end_char >= chars.len() {
            // Last chunk - take everything remaining
            let remaining: String = chars[current_start..].iter().collect();
            chunks.push(remaining);
            break;
        }
        
        // Get the chunk text up to max_size characters
        let chunk_text: String = chars[current_start..end_char].iter().collect();
        
        // Find the best split point within this chunk
        if let Some(split_pos) = find_best_split_point(&chunk_text) {
            // split_pos is in bytes within chunk_text, which is safe
            let actual_chunk = &chunk_text[..split_pos];
            chunks.push(actual_chunk.to_string());
            
            // Convert back to character position
            let chars_taken = actual_chunk.chars().count();
            current_start += chars_taken;
        } else {
            // No good split point found, take the whole chunk
            chunks.push(chunk_text);
            current_start = end_char;
        }
    }
    
    chunks
}

fn find_best_split_point(text: &str) -> Option<usize> {
    // Priority order for split points (from best to worst)
    let boundaries = [
        "\n\n",    // Paragraph breaks (best)
        "\n",      // Line breaks
        ". ",      // Sentence ends (only with space after)
        "? ",      // Question ends
        "! ",      // Exclamation ends
        ", ",      // Comma breaks
        " ",       // Word boundaries (last resort)
    ];
    
    for boundary in &boundaries {
        if let Some(pos) = text.rfind(boundary) {
            // Make sure we're not too close to the beginning (at least 25% through)
            if pos > text.len() / 4 {
                return Some(pos + boundary.len());
            }
        }
    }
    
    None
}

fn add_or_merge_chunk(final_chunks: &mut Vec<String>, chunk_text: String) {
    let trimmed_chunk = chunk_text.trim();
    if trimmed_chunk.is_empty() {
        return;
    }

    // Check if the last chunk is small and can be merged with
    if let Some(last_chunk) = final_chunks.last_mut() {
        let last_chunk_len = last_chunk.chars().count();
        let new_chunk_len = trimmed_chunk.chars().count();

        // Condition: Is the last chunk too small AND will the combined chunk be under the max limit?
        if last_chunk_len < constants::VECDB_MIN_CHUNK_SIZE &&
           (last_chunk_len + new_chunk_len) <= constants::VECDB_MAX_CHUNK_SIZE {
            
            // Merge the new chunk into the previous one
            last_chunk.push_str("\n\n"); 
            last_chunk.push_str(trimmed_chunk);
            return;
        }
    }

    // If we didn't merge, just add the new chunk as usual.
    final_chunks.push(trimmed_chunk.to_string());
}