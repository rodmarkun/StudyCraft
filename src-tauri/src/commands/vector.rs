use crate::services::db_service::StudyMaterialOperations;
use crate::services::vector_service::vector_service::RelevantChunk;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn search_study_materials_global(
    query: String,
    max_results: Option<u32>,
    app_state: State<'_, AppState>,
) -> Result<Vec<RelevantChunk>, String> {
    let vector_service = app_state.vector.lock().await;
    let study_settings = app_state.study_settings.lock().await;
    let max_chunks = max_results.unwrap_or(study_settings.max_chunks_to_recover_from_search);

    vector_service
        .search_global(&query, max_chunks)
        .await
        .map_err(|e| format!("Failed to search materials: {}", e))
}

#[tauri::command]
pub async fn search_specific_materials(
    query: String,
    material_ids: Vec<String>,
    max_results: Option<u32>,
    app_state: State<'_, AppState>,
) -> Result<Vec<RelevantChunk>, String> {
    let vector_service = app_state.vector.lock().await;
    let max_chunks = max_results.unwrap_or(5);

    vector_service
        .search_material_specific(&query, &material_ids, max_chunks)
        .await
        .map_err(|e| format!("Failed to search specific materials: {}", e))
}

#[tauri::command]
pub async fn reindex_material(
    file_name: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let db_service = app_state.db.lock().await;

    let material = db_service
        .get_study_material_by_file_name(&file_name)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Study material not found".to_string())?;

    let material_id = material.id.to_string();
    drop(db_service);
    let mut vector_service = app_state.vector.lock().await;

    // delete existing entries
    if let Err(e) = vector_service.delete_material(&material_id).await {
        eprintln!(
            "Warning: Failed to delete existing material from vector database: {}",
            e
        );
    }

    // then reindex
    match vector_service.index_material(material).await {
        Ok(chunk_ids) => {
            drop(vector_service);

            let db_service = app_state.db.lock().await;
            if let Err(e) = db_service.update_study_material_indexed_state(
                &material_id,
                true,
                chunk_ids.len() as u32,
            ) {
                eprintln!("Failed to update indexed state: {}", e);
            }

            Ok(format!(
                "Successfully reindexed {} chunks for material: {}",
                chunk_ids.len(),
                file_name
            ))
        }
        Err(e) => Err(format!("Failed to reindex material: {}", e)),
    }
}

#[tauri::command]
pub async fn get_vector_stats(app_state: State<'_, AppState>) -> Result<String, String> {
    let vector_service = app_state.vector.lock().await;
    vector_service
        .get_stats()
        .await
        .map_err(|e| format!("Failed to get vector stats: {}", e))
}

#[tauri::command]
pub async fn delete_material_from_vector(
    material_id: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let mut vector_service = app_state.vector.lock().await;

    vector_service
        .delete_material(&material_id)
        .await
        .map_err(|e| format!("Failed to delete material from vector database: {}", e))?;

    Ok(format!(
        "Successfully deleted material {} from vector database",
        material_id
    ))
}

#[tauri::command]
pub async fn initialize_embedding_model(app_state: State<'_, AppState>) -> Result<(), String> {
    let vector_service = app_state.vector.lock().await;
    vector_service
        .initialize_embedding_model()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_embedding_model_ready(app_state: State<'_, AppState>) -> Result<bool, String> {
    let vector_service = app_state.vector.lock().await;
    Ok(vector_service.is_embedding_model_ready())
}
