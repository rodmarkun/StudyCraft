use crate::services::db_service::TagOperations;
use crate::state::AppState;
use std::collections::HashSet;
use tauri::State;

#[tauri::command]
pub async fn remove_tag_from_material(
    file_path: String,
    tag: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let db_service = app_state.db.lock().await;
    db_service
        .remove_tag_from_material(&file_path, &tag)
        .map_err(|e| format!("Failed to remove tag from material: {}", e))
}

#[tauri::command]
pub async fn get_all_tags(app_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db_service = app_state.db.lock().await;
    let tags: HashSet<String> = db_service
        .get_all_tags()
        .map_err(|e| format!("Failed to get tags: {}", e))?;
    Ok(tags.into_iter().collect())
}

#[tauri::command]
pub async fn get_available_tags(app_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db_service = app_state.db.lock().await;
    db_service
        .get_available_tags()
        .map_err(|e| format!("Failed to get available tags: {}", e))
}

#[tauri::command]
pub async fn add_available_tag(tag: String, app_state: State<'_, AppState>) -> Result<(), String> {
    println!("Adding tag: {}", tag);
    let db_service = app_state.db.lock().await;

    match db_service.add_available_tag(tag) {
        Ok(()) => {
            println!("Tag added successfully");
            Ok(())
        }
        Err(e) => {
            println!("Failed to add tag: {}", e);
            Err(format!("Failed to add tag: {}", e))
        }
    }
}

#[tauri::command]
pub async fn remove_available_tag(
    tag: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let db_service = app_state.db.lock().await;
    db_service
        .remove_available_tag(&tag)
        .map_err(|e| format!("Failed to remove available tag: {}", e))
}

#[tauri::command]
pub async fn add_tag_to_material(
    file_path: String,
    tag: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    println!("Adding tag '{}' to material: {}", tag, file_path);

    let db_service = app_state.db.lock().await;
    db_service
        .add_tag_to_material(&file_path, tag)
        .map_err(|e| format!("Failed to add tag to material: {}", e))
}
