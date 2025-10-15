use serde_json;
use std::error::Error;
use std::fmt;

/// Custom error types for LLM operations
#[derive(Debug)]
pub enum LlmError {
    /// Error from the HTTP client
    RequestError(reqwest::Error),
    /// Error from the API provider
    ApiError(String),
    /// Rate limiting error
    RateLimit(String),
    /// Parsing error
    ParseError(String),
    /// Provider is disabled
    ProviderDisabled(String),
    /// Configuration error
    ConfigError(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::RequestError(err) => write!(f, "Request error: {}", err),
            LlmError::ApiError(msg) => write!(f, "API error: {}", msg),
            LlmError::RateLimit(msg) => write!(f, "Rate limit error: {}", msg),
            LlmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LlmError::ProviderDisabled(provider) => write!(f, "Provider disabled: {}", provider),
            LlmError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for LlmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LlmError::RequestError(err) => Some(err),
            _ => None,
        }
    }
}

/// Convert reqwest errors to LlmError
impl From<reqwest::Error> for LlmError {
    fn from(err: reqwest::Error) -> Self {
        LlmError::RequestError(err)
    }
}

/// Convert serde_json errors to LlmError
impl From<serde_json::Error> for LlmError {
    fn from(err: serde_json::Error) -> Self {
        LlmError::ParseError(err.to_string())
    }
}

/// Result type alias for LLM operations
pub type LlmResult<T> = Result<T, LlmError>;

impl LlmError {
    /// Returns RateLimit error for 429 status or rate limit keywords
    pub fn from_api_response(status: reqwest::StatusCode, error_message: String) -> Self {
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return LlmError::RateLimit(error_message);
        }

        // Check error message for rate limit indicators
        let msg_lower = error_message.to_lowercase();
        if msg_lower.contains("rate limit")
            || msg_lower.contains("too many requests")
            || msg_lower.contains("quota exceeded")
            || msg_lower.contains("overloaded")
            || msg_lower.contains("throttle")
        {
            return LlmError::RateLimit(error_message);
        }

        LlmError::ApiError(error_message)
    }
}
