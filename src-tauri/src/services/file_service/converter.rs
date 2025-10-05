use crate::services::file_service::markdown_renderer;
use crate::services::url_service::url_converter;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use std::path::Path;

pub async fn convert_file_to_markdown(file_path: &Path, app: AppHandle) -> Result<String, String> {
    let sidecar_command = app
        .shell()
        .sidecar("conversor") // Just the filename, not the full path
        .map_err(|e| format!("Failed to create sidecar: {}", e))?
        .args(file_path.to_str());
    
    let output = sidecar_command
        .output()
        .await
        .map_err(|e| format!("Failed to execute: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!("Command failed with code {}: {}", 
                   output.status.code().unwrap_or(-1), 
                   String::from_utf8_lossy(&output.stderr)))
    }
}

pub async fn extract_cover_image(
    file_path: &Path,
    output_path: &Path,
    file_extension: &str,
    url: &str,
    chromium_path: &Path,
    app: AppHandle
) -> Result<(), String> {
    match file_extension {
        "pdf" => extract_pdf_cover(file_path, output_path, app).await,
        "md" | "txt" => generate_text_cover(file_path, output_path, url, chromium_path).await,
        _ => Err(format!(
            "Unsupported file type for cover extraction: {}",
            file_extension
        )),
    }
}

async fn extract_pdf_cover(file_path: &Path, output_path: &Path, app: AppHandle) -> Result<(), String> {
    let file_path_str = file_path.to_str().ok_or("Invalid file path")?;
    let output_path_str = output_path.to_str().ok_or("Invalid output path")?;
    let sidecar_command = app
        .shell()
        .sidecar("cover-extractor") 
        .map_err(|e| format!("Failed to create sidecar: {}", e))?
        .args([file_path_str, output_path_str]);
    
    let output = sidecar_command
        .output()
        .await
        .map_err(|e| format!("Failed to execute: {}", e))?;
    
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Command failed with code {}: {}", 
                   output.status.code().unwrap_or(-1), 
                   String::from_utf8_lossy(&output.stderr)))
    }
}

async fn generate_text_cover(
    file_path: &Path,
    output_path: &Path,
    url: &str,
    chromium_path: &Path
) -> Result<(), String> {
    if !url.is_empty() && !url_converter::is_git_url(url) {
        markdown_renderer::url_to_screenshot(url, output_path, chromium_path)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        markdown_renderer::markdown_file_to_screenshot(file_path, output_path, chromium_path)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
