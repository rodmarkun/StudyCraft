use crate::config::APP_PATHS;
use crate::materials::study_material;
use crate::os_dep;
use crate::services::db_service::StudyMaterialOperations;
use crate::services::file_service;
use crate::services::url_service;
use crate::state::AppState;
use crate::utils;
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::State;

#[tauri::command]
pub async fn get_study_materials(
    app_state: State<'_, AppState>,
    sort_by: Option<String>,
) -> Result<Vec<study_material::StudyMaterial>, String> {
    let db_service = app_state.db.lock().await;

    let materials = db_service
        .get_study_materials()
        .map_err(|e| format!("Failed to get study materials: {}", e))?;

    let sorted_materials = study_material::sort_study_materials(sort_by, materials);

    Ok(sorted_materials)
}

#[tauri::command]
pub async fn add_study_material(
    id: String,
    file_content: Vec<u8>,
    file_name: String,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    println!("Starting to process file: {}", file_name);

    let library_path = &APP_PATHS.materials_dir;
    let destination_path = library_path.join(&file_name);

    let file_extension = utils::get_file_extension(&file_name);
    if !utils::is_supported_extension(&file_extension) {
        return Err(format!("Unsupported file type: {}", file_extension));
    }

    let file_stem = file_name.rsplit_once('.').unwrap_or((&file_name, "")).0;
    let md_file_name = format!("{}.md", file_stem);
    let cover_file_name = format!("{}.png", file_stem);

    let md_destination_path = library_path.join(md_file_name);
    let cover_destination_path = APP_PATHS.covers_dir.join(cover_file_name);

    std::fs::write(&destination_path, file_content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    let study_material = study_material::StudyMaterial::new(
        id,
        destination_path.clone(),
        md_destination_path,
        cover_destination_path,
        true,
    );

    let db_service = app_state.db.lock().await;
    db_service
        .add_study_material(&study_material)
        .map_err(|e| format!("Failed to add study material to database: {}", e))?;
    drop(db_service);

    println!("Material added to database");

    let markdown_content = match file_extension.as_str() {
        "pdf" => {
            println!("Starting markdown conversion for PDF...");
            file_service::convert_file_to_markdown(&destination_path, app_handle.clone()).await
        }
        _ => {
            println!("Reading file directly...");
            std::fs::read_to_string(&destination_path)
                .map_err(|e| format!("Failed to read markdown file: {}", e))
        }
    };

    match markdown_content {
        Ok(content) => {
            process_study_material_async(
                app_handle,
                &app_state,
                study_material,
                content,
                file_extension,
            )
            .await;
        }
        Err(e) => {
            eprintln!("Failed to process file content: {}", e);
        }
    }

    Ok(format!("File '{}' uploaded, processing started", file_name))
}

#[tauri::command]
pub async fn add_study_material_from_path(
    id: String,
    file_path: String,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let file_content =
        fs::read(&file_path).map_err(|e| format!("Error when reading file content: {}", e))?;
    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Error when extracting filename from file path")?
        .to_string();

    add_study_material(id, file_content, file_name, app_state, app_handle).await
}

#[tauri::command]
pub async fn add_study_material_from_link(
    id: String,
    link: String,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let is_git_repo = url_service::url_converter::is_git_url(&link);
    let (title, material_name, content) = if is_git_repo {
        let content = url_service::git_converter::convert_repo_to_md(&link).await?;
        let material_name = url_service::git_converter::extract_repo_name(&link)?;
        let title = material_name.clone();
        (title, material_name, content)
    } else {
        url_service::url_converter::convert_url_to_md(&link).await?
    };

    let library_path = &APP_PATHS.materials_dir;
    let md_file_name = format!("{}.md", material_name);
    let cover_file_name = format!("{}.png", material_name);

    let md_destination_path = library_path.join(md_file_name);
    let cover_destination_path = APP_PATHS.covers_dir.join(cover_file_name);

    let study_material = study_material::StudyMaterial::new_from_url(
        id,
        title,
        link.clone(),
        md_destination_path.clone(),
        cover_destination_path.clone(),
        true,
    );

    let db_service = app_state.db.lock().await;
    db_service
        .add_study_material(&study_material)
        .map_err(|e| format!("Failed to add study material to database: {}", e))?;
    drop(db_service);

    println!("Material added to database");

    process_study_material_async(
        app_handle,
        &app_state,
        study_material,
        content,
        "md".to_string(),
    )
    .await;

    Ok(format!("Link '{}' processed, processing started", link))
}

async fn process_vector_indexing(
    state: &AppState,
    study_material: study_material::StudyMaterial,
) -> Result<(), String> {
    println!("Indexing material: {}", study_material.display_name);

    let material_id = study_material.id.clone().to_string();
    let mut vector_service = state.vector.lock().await;
    match tokio::time::timeout(
        std::time::Duration::from_secs(300), // 5 minute timeout
        vector_service.index_material(study_material),
    )
    .await
    {
        Ok(Ok(chunk_ids)) => {
            println!(
                "Successfully indexed {} chunks for material",
                chunk_ids.len()
            );

            drop(vector_service);
            let db_service = state.db.lock().await;

            if let Err(e) = db_service.update_study_material_indexed_state(
                &material_id,
                true,
                chunk_ids.len() as u32,
            ) {
                eprintln!("Failed to update indexed state: {}", e);
                return Err(format!("Failed to update indexed state: {}", e));
            }
        }
        Ok(Err(e)) => {
            eprintln!("Failed to index material in vector database: {}", e);
            return Err(format!("Vector indexing failed: {}", e));
        }
        Err(_) => {
            eprintln!("Vector indexing timed out after 5 minutes");
            return Err("Vector indexing timed out".to_string());
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_study_material(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let db_service = app_state.db.lock().await;
    println!("Attempting to delete material with ID: {id}");

    let material = db_service
        .get_study_material_by_id(&id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Study material not found".to_string())?;

    let material_id = material.id.to_string();
    {
        let mut vector_service = app_state.vector.lock().await;
        if let Err(e) = vector_service.delete_material(&material_id).await {
            eprintln!("Failed to delete material from vector database: {}", e);
        }
    }

    let (original_path, markdown_path, cover_path) = db_service
        .delete_study_material(&id)
        .map_err(|e| format!("Failed to delete material from database: {}", e))?;

    if let Err(e) = std::fs::remove_file(&original_path) {
        eprintln!("Failed to delete original file: {}", e);
    }
    if original_path != markdown_path {
        if let Err(e) = std::fs::remove_file(&markdown_path) {
            eprintln!("Failed to delete markdown file: {}", e);
        }
    }
    if let Some(cover_path) = cover_path {
        if let Err(e) = std::fs::remove_file(&cover_path) {
            eprintln!("Failed to delete cover image: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn update_material_name(
    id: String,
    new_name: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let db_service = app_state.db.lock().await;
    db_service
        .update_study_material_name(&id, &new_name)
        .map_err(|e| format!("Failed to update material name: {}", e))
}

#[tauri::command]
pub async fn open_material(id: String, app_state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    let db_service = app_state.db.lock().await;

    let material = db_service
        .get_study_material_by_id(&id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Study material not found".to_string())?;

    let path_str = match material.url.is_empty() {
        true => material
            .original_path
            .to_str()
            .ok_or("Invalid file path encoding")?,
        false => &material.url,
    };

    os_dep::open_study_material(path_str, app_handle).await
}

#[tauri::command]
pub async fn get_material_markdown_path(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let db_service = state.db.lock().await;

    match db_service
        .get_study_material_by_id(&id)
        .map_err(|e| format!("Database error: {}", e))?
    {
        Some(material) => Ok(material.markdown_path.to_string_lossy().into_owned()),
        None => Err("Study material not found".to_string()),
    }
}

#[tauri::command]
pub async fn get_markdown_content(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    if path.extension().map_or(false, |ext| ext != "md") {
        return Err("File is not a markdown file".to_string());
    }

    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

#[tauri::command]
pub async fn save_markdown_content(
    file_path: String,
    content: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let path = Path::new(&file_path);

    if let Err(e) = fs::write(path, content) {
        return Err(format!("Failed to write file: {}", e));
    }

    let db_service = app_state.db.lock().await;
    let materials = db_service
        .get_study_materials()
        .map_err(|e| format!("Database error: {}", e))?;
    drop(db_service);

    let material = materials
        .into_iter()
        .find(|material| {
            let material_path = material.markdown_path.to_string_lossy();
            let matches = material_path == file_path;
            matches
        })
        .ok_or_else(|| format!("Study material not found for markdown file: {}", file_path))?;

    let was_indexed = material.is_indexed;
    let material_id = material.id.to_string();

    // Only re-index if the material was previously indexed
    // there is no reason why a material would not be indexed right now,
    // but check just in case we do something in the future with it
    if was_indexed {
        println!(
            "Re-indexing material {} after content update",
            material.display_name
        );
        let mut vector_service = app_state.vector.lock().await;

        if let Err(e) = vector_service.delete_material(&material_id).await {
            eprintln!("Warning: Failed to delete existing chunks: {}", e);
        }

        match tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minute timeout
            vector_service.index_material(material),
        )
        .await
        {
            Ok(Ok(chunk_ids)) => {
                drop(vector_service);

                let db_service = app_state.db.lock().await;
                if let Err(e) = db_service.update_study_material_indexed_state(
                    &material_id,
                    true,
                    chunk_ids.len() as u32,
                ) {
                    eprintln!("Failed to update indexed state: {}", e);
                }
            }
            Ok(Err(e)) => {
                eprintln!("Failed to re-index material after content update: {}", e);
            }
            Err(_) => {
                eprintln!("Re-indexing timed out after 5 minutes");
            }
        }
    }

    Ok("Content saved successfully".to_string())
}

#[tauri::command]
pub fn get_cover_image_raw(path: String) -> Result<Vec<u8>, String> {
    println!("Reading cover image from: {}", path);

    if !std::path::Path::new(&path).exists() {
        return Err(format!("Cover image file not found: {}", path));
    }

    match fs::read(&path) {
        Ok(image_data) => Ok(image_data),
        Err(e) => {
            let error_msg = format!("Failed to read cover image: {}", e);
            eprintln!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn initialize_chromium(app_state: State<'_, AppState>) -> Result<(), String> {
    app_state
        .chromium
        .lock()
        .await
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize Chromium: {}", e))
}

#[tauri::command]
pub async fn is_chromium_downloaded(app_state: State<'_, AppState>) -> Result<bool, String> {
    let chromium = app_state.chromium.lock().await;
    Ok(chromium.is_chromium_downloaded())
}

async fn process_study_material_async(
    app_handle: AppHandle,
    state: &AppState,
    study_material: study_material::StudyMaterial,
    content: String,
    file_extension: String,
) {
    let material_id = study_material.id.clone();
    let md_destination_path = study_material.markdown_path.clone();
    let destination_path = study_material.original_path.clone();
    let cover_destination_path = study_material.cover_path.clone().unwrap();

    println!(
        "About to write markdown content to: {:?}",
        md_destination_path
    );

    // Write markdown content
    if let Err(e) = std::fs::write(&md_destination_path, content) {
        eprintln!("Failed to write markdown content: {}", e);
        return;
    }
    println!("Markdown content written successfully");

    // Extract cover image
    println!("Starting cover extraction for {}...", file_extension);

    let chromium_path = {
        let chromium = state.chromium.lock().await;
        match chromium.get_executable_path().await {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to get chromium executable path: {}", e);
                println!("Skipping cover extraction, proceeding to vector indexing...");
                process_vector_and_finalize(state, study_material, material_id, app_handle).await;
                return;
            }
        }
    };

    if let Err(e) = file_service::extract_cover_image(
        &destination_path,
        &cover_destination_path,
        &file_extension,
        &study_material.url,
        chromium_path.as_path(),
        app_handle.clone()
    )
    .await
    {
        eprintln!("Failed to extract cover: {}", e);
    } else {
        if let Err(e) = app_handle.emit(
            "cover-ready",
            &destination_path.to_string_lossy().to_string(),
        ) {
            eprintln!("Failed to emit cover-ready event: {}", e);
        }
    }

    process_vector_and_finalize(state, study_material, material_id, app_handle).await;
}


async fn process_vector_and_finalize(
    state: &AppState,
    study_material: study_material::StudyMaterial,
    material_id: String,
    app_handle: AppHandle,
) {
    println!("Starting vector indexing...");
    if let Err(e) = process_vector_indexing(state, study_material).await {
        eprintln!("Vector indexing failed: {}", e);
    }
    
    let db_service = state.db.lock().await;
    match db_service.update_study_material_processing_state(&material_id, false) {
        Ok(_) => {
            println!("Processing status updated");
            let _ = app_handle.emit("material-updated", ());
        }
        Err(e) => eprintln!("Failed to update processing status: {}", e),
    }
}