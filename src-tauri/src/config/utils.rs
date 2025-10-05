use crate::services::llm_service::agents::AgentType;

pub fn internal_str_to_provider_type(provider_str: &str) -> Result<flyllm::ProviderType, String> {
    match provider_str.to_lowercase().as_str() {
        "openai" => Ok(flyllm::ProviderType::OpenAI),
        "anthropic" => Ok(flyllm::ProviderType::Anthropic),
        "mistral" => Ok(flyllm::ProviderType::Mistral),
        "google" => Ok(flyllm::ProviderType::Google),
        "ollama" => Ok(flyllm::ProviderType::Ollama),
        _ => Err(format!("Unknown provider string: {}", provider_str)),
    }
}

pub fn str_to_agent_type(agent_str: &str) -> Result<AgentType, String> {
    match agent_str {
        "ConceptExtractor" => Ok(AgentType::ConceptExtractor),
        "FlashcardContentCreator" => Ok(AgentType::FlashcardContentCreator),
        "TestContentCreator" => Ok(AgentType::TestContentCreator),
        "ExplanationAgent" => Ok(AgentType::ExplanationAgent),
        "SearchAgent" => Ok(AgentType::SearchAgent),
        _ => Err(format!("Unknown agent type: {}", agent_str)),
    }
}
