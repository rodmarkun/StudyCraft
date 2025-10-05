use std::error::Error;
use std::fmt;
use std::io;

/// AppError definition
#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    ConfigError(String),
    MaterialNotFound(String),
    NotFound(String),
    TagError(String),
    ApiError(String),
    ConversionError(String),
    DbError(String),
    InvalidInput(String),
    LlmError(String),
    JsonError(String),
    ValidationError(String),
    VectorError(String),
    UrlError(String),
    GitError(String),
    MarkdownRendererError(String),
    ChromiumError(String)
}

/// Displays for AppError
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(err) => write!(f, "IO Error: {}", err),
            AppError::ConfigError(msg) => write!(f, "Configuration Error: {}", msg),
            AppError::MaterialNotFound(id) => write!(f, "Material Not Found: {}", id),
            AppError::NotFound(item) => write!(f, "Not Found: {}", item),
            AppError::TagError(msg) => write!(f, "Tag Error: {}", msg),
            AppError::ApiError(msg) => write!(f, "API Error: {}", msg),
            AppError::ConversionError(msg) => write!(f, "Conversion Error: {}", msg),
            AppError::DbError(msg) => write!(f, "Database Error: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            AppError::LlmError(msg) => write!(f, "LLM Error: {}", msg),
            AppError::JsonError(msg) => write!(f, "JSON Error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            AppError::VectorError(msg) => write!(f, "Vector Database Error: {}", msg),
            AppError::UrlError(msg) => write!(f, "URL Error: {}", msg),
            AppError::GitError(msg) => write!(f, "Git Error: {}", msg),
            AppError::MarkdownRendererError(msg) => write!(f, "Markdown Renderer Error: {}", msg),
            AppError::ChromiumError(msg) => write!(f, "Chromium error: {}", msg)
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::ConfigError(error.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError::DbError(error.to_string())
    }
}

impl From<r2d2::Error> for AppError {
    fn from(error: r2d2::Error) -> Self {
        AppError::DbError(error.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(error: url::ParseError) -> Self {
        AppError::UrlError(format!("Invalid URL format: {}", error))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            AppError::UrlError(format!("Request timeout: {}", error))
        } else if error.is_connect() {
            AppError::UrlError(format!("Connection failed: {}", error))
        } else if error.is_status() {
            AppError::UrlError(format!("HTTP error: {}", error))
        } else {
            AppError::UrlError(format!("Request failed: {}", error))
        }
    }
}

impl From<git2::Error> for AppError {
    fn from(error: git2::Error) -> Self {
        AppError::GitError(error.to_string())
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        if error.contains("material not found") || error.contains("study material not found") {
            AppError::MaterialNotFound(error)
        } else if error.contains("not found") {
            AppError::NotFound(error)
        } else if error.contains("tag") {
            AppError::TagError(error)
        } else if error.contains("api") || error.contains("API") {
            AppError::ApiError(error)
        } else if error.contains("convert") {
            AppError::ConversionError(error)
        } else if error.contains("database") || error.contains("sql") || error.contains("SQL") {
            AppError::DbError(error)
        } else if error.contains("invalid") || error.contains("validation") {
            AppError::InvalidInput(error)
        } else if error.contains("vector")
            || error.contains("Vector")
            || error.contains("qdrant")
            || error.contains("embedding")
        {
            AppError::VectorError(error)
        } else {
            AppError::ConfigError(error)
        }
    }
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;
