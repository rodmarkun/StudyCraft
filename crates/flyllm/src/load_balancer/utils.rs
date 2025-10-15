use crate::errors::LlmError;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

pub fn get_debug_path(
    debug_folder: &PathBuf,
    timestamp: u64,
    instance_id: usize,
    instance_provider: &str,
    instance_model: &str,
) -> PathBuf {
    let timestamp_folder = debug_folder.join(timestamp.to_string());
    let instance_folder = timestamp_folder.join(format!(
        "{}_{}_{}",
        instance_id, instance_provider, instance_model
    ));
    instance_folder.join("debug.json")
}

pub fn write_to_debug_file(file_path: &PathBuf, contents: &str) -> Result<(), LlmError> {
    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent).map_err(|e| {
            LlmError::ConfigError(format!("Failed to create debug directories: {}", e))
        })?;
    }

    let mut file = File::create(file_path)
        .map_err(|e| LlmError::ConfigError(format!("Failed to create debug file: {}", e)))?;

    file.write_all(contents.as_bytes())
        .map_err(|e| LlmError::ConfigError(format!("Failed to write to debug file: {}", e)))?;

    Ok(())
}
