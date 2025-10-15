use crate::services::llm_service::agents::AgentType;
use flyllm::ProviderType;

// General
pub const APP_NAME: &str = "StudyCraft";
pub const MAX_FLASHCARDS_PER_REQUEST: u32 = 5;
pub const LIMIT_RECENT_SESSIONS_RETRIEVAL: i32 = 9999;
pub const COVER_IMAGE_DIMENSIONS: (u32, u32) = (850, 1300);

// Individual difficulty constants
pub const AGAIN: &str = "again";
pub const HARD: &str = "hard";
pub const GOOD: &str = "good";
pub const EASY: &str = "easy";

// List of all valid difficulties
pub const VALID_DIFFICULTIES: [&str; 4] = [AGAIN, HARD, GOOD, EASY];

// Sorting parameters
pub const NAME_ASC: &str = "name_asc";
pub const NAME_DESC: &str = "name_desc";
pub const DATE_ASC: &str = "date_asc";
pub const DATE_DESC: &str = "date_desc";

// Valid file format (study materials, aside from links)
pub const SUPPORTED_FILE_EXTENSIONS: [&str; 3] = ["pdf", "md", "txt"];

// Settings
pub const GENERAL_SETTINGS_FILE_NAME: &str = "general_settings.json";
pub const API_SETTINGS_FILE_NAME: &str = "api_settings.json";
pub const AGENT_SETTINGS_FILE_NAME: &str = "agent_settings.json";

// Vector database
pub const VECDB_JOURNAL_MODE: &str = "WAL";
pub const VECDB_SYNCHRONOUS: &str = "NORMAL";
pub const VECDB_CACHE_SIZE: &str = "5000";
pub const VECDB_TEMP_STORE: &str = "memory";
pub const VECDB_MMAP_SIZE: &str = "134217728;";
pub const VECDB_EMBEDDING_DIMENSIONS: &str = "384"; // This is embedding model dependent. If model is changed this might have to change as well.

pub const VECDB_MAX_CHUNKS_PER_MATERIAL: usize = 500;
pub const VECDB_MAX_CHUNK_SIZE: usize = 2048;
pub const VECDB_MIN_CHUNK_SIZE: usize = 256;
pub const VECDB_BATCH_SIZE: usize = 20;
pub const VECDB_MAX_FILE_SIZE: u64 = 20 * 1024 * 1024; // This is not the file *itself*, but the markdown conversion
pub const VECDB_OVERLAP: usize = 150;
pub const VECDB_MIN_CHUNKS_RECOVERED_AI_SEARCH: u32 = 8;
pub const VECDB_MAX_CHUNKS_RECOVERED_AI_SEARCH: u32 = 12; // These two constants are mainly for giving the LLM enough context but not too much to not end up with big costs / errors due to a big context

// HTML
pub const CONTENT_SELECTORS: [&'static str; 7] = [
    "main",
    "article",
    "[role='main']",
    ".content",
    "#content",
    ".main-content",
    "body",
];
// LLMs
pub const ALL_PROVIDER_TYPES: [ProviderType; 6] = [
    ProviderType::OpenAI,
    ProviderType::Anthropic,
    ProviderType::Mistral,
    ProviderType::Google,
    ProviderType::Ollama,
    ProviderType::LmStudio,
];
pub const OLLAMA_CUSTOM_ENDPOINT: &str = "http://localhost:11434";
pub const LM_STUDIO_DEFAULT_ENDPOINT: &str = "http://127.0.0.1:1234";

// Prompts
pub const CONCEPT_EXTRACTOR_PROMPT: &str =
    include_str!("services/llm_service/prompts/concept_extractor.txt");
pub const FLASHCARD_CONTENT_CREATOR_PROMPT: &str =
    include_str!("services/llm_service/prompts/flashcard_content_creator.txt");
pub const TEST_CONTENT_CREATOR_PROMPT: &str =
    include_str!("services/llm_service/prompts/test_content_creator.txt");
pub const EXPLANATION_AGENT_PROMPT: &str =
    include_str!("services/llm_service/prompts/explanation_agent.txt");
pub const SEARCH_AGENT_PROMPT: &str = include_str!("services/llm_service/prompts/search_agent.txt");

/// Get default model for a specific agent and provider combination
pub fn get_default_agent_model(agent: &AgentType, provider: &ProviderType) -> &'static str {
    match (agent, provider) {
        // ConceptExtractor
        (AgentType::ConceptExtractor, ProviderType::OpenAI) => "gpt-4.1-mini-2025-04-14",
        (AgentType::ConceptExtractor, ProviderType::Anthropic) => "claude-sonnet-4-20250514",
        (AgentType::ConceptExtractor, ProviderType::Google) => "gemini-2.5-flash",
        (AgentType::ConceptExtractor, ProviderType::Mistral) => "mistral-large-latest",
        (AgentType::ConceptExtractor, ProviderType::Ollama) => "llama3",
        (AgentType::ConceptExtractor, ProviderType::LmStudio) => "openai/gpt-oss-20b",

        // FlashcardContentCreator
        (AgentType::FlashcardContentCreator, ProviderType::OpenAI) => "gpt-4.1-mini-2025-04-14",
        (AgentType::FlashcardContentCreator, ProviderType::Anthropic) => "claude-sonnet-4-20250514",
        (AgentType::FlashcardContentCreator, ProviderType::Google) => "gemini-2.5-flash",
        (AgentType::FlashcardContentCreator, ProviderType::Mistral) => "mistral-large-latest",
        (AgentType::FlashcardContentCreator, ProviderType::Ollama) => "llama3",
        (AgentType::FlashcardContentCreator, ProviderType::LmStudio) => "openai/gpt-oss-20b",

        // TestContentCreator
        (AgentType::TestContentCreator, ProviderType::OpenAI) => "gpt-4.1-mini-2025-04-14",
        (AgentType::TestContentCreator, ProviderType::Anthropic) => "claude-sonnet-4-20250514",
        (AgentType::TestContentCreator, ProviderType::Google) => "gemini-2.5-flash",
        (AgentType::TestContentCreator, ProviderType::Mistral) => "mistral-large-latest",
        (AgentType::TestContentCreator, ProviderType::Ollama) => "llama3",
        (AgentType::TestContentCreator, ProviderType::LmStudio) => "openai/gpt-oss-20b",

        // ExplanationAgent
        (AgentType::ExplanationAgent, ProviderType::OpenAI) => "gpt-4.1-mini-2025-04-14",
        (AgentType::ExplanationAgent, ProviderType::Anthropic) => "claude-sonnet-4-20250514",
        (AgentType::ExplanationAgent, ProviderType::Google) => "gemini-2.5-flash",
        (AgentType::ExplanationAgent, ProviderType::Mistral) => "mistral-large-latest",
        (AgentType::ExplanationAgent, ProviderType::Ollama) => "llama3",
        (AgentType::ExplanationAgent, ProviderType::LmStudio) => "openai/gpt-oss-20b",

        // SearchAgent
        (AgentType::SearchAgent, ProviderType::OpenAI) => "gpt-4.1-mini-2025-04-14",
        (AgentType::SearchAgent, ProviderType::Anthropic) => "claude-sonnet-4-20250514",
        (AgentType::SearchAgent, ProviderType::Google) => "gemini-2.5-flash",
        (AgentType::SearchAgent, ProviderType::Mistral) => "mistral-large-latest",
        (AgentType::SearchAgent, ProviderType::Ollama) => "llama3",
        (AgentType::SearchAgent, ProviderType::LmStudio) => "openai/gpt-oss-20b",
    }
}
