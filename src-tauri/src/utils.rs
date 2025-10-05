use crate::constants;
use std::path::Path;

pub fn is_supported_extension(extension: &str) -> bool {
    constants::SUPPORTED_FILE_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

pub fn get_file_extension(file_name: &str) -> String {
    Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}

pub fn get_filename_from_path(file_path: &str) -> String {
    Path::new(file_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
