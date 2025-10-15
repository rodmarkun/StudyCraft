use crate::services::llm_service::agents::AgentType;

pub fn internal_str_to_provider_type(provider_str: &str) -> Result<flyllm::ProviderType, String> {
    match provider_str.to_lowercase().as_str() {
        "openai" => Ok(flyllm::ProviderType::OpenAI),
        "anthropic" => Ok(flyllm::ProviderType::Anthropic),
        "mistral" => Ok(flyllm::ProviderType::Mistral),
        "google" => Ok(flyllm::ProviderType::Google),
        "ollama" => Ok(flyllm::ProviderType::Ollama),
        "lmstudio" => Ok(flyllm::ProviderType::LmStudio),
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

#[cfg(test)]
mod tests {
    use super::internal_str_to_provider_type;
    use flyllm::ProviderType;

    #[test]
    fn maps_lmstudio_provider_string() {
        let provider = internal_str_to_provider_type("lmstudio").expect("should map");
        assert_eq!(provider, ProviderType::LmStudio);
    }
}
