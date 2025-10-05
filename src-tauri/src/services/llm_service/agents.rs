use crate::constants;
use flyllm::TaskDefinition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    ConceptExtractor,
    FlashcardContentCreator,
    TestContentCreator,
    ExplanationAgent,
    SearchAgent,
}

impl AgentType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::ConceptExtractor,
            Self::FlashcardContentCreator,
            Self::TestContentCreator,
            Self::ExplanationAgent,
            Self::SearchAgent,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConceptExtractor => "ConceptExtractor",
            Self::FlashcardContentCreator => "FlashcardContentCreator",
            Self::TestContentCreator => "TestContentCreator",
            Self::ExplanationAgent => "ExplanationAgent",
            Self::SearchAgent => "SearchAgent",
        }
    }

    pub fn get_task_definition(&self) -> TaskDefinition {
        match self {
            Self::ConceptExtractor => TaskDefinition::new("Concept Extraction")
                .with_max_tokens(2000)
                .with_temperature(0.3),

            Self::FlashcardContentCreator => TaskDefinition::new("Flashcard Content Creation")
                .with_max_tokens(3500)
                .with_temperature(0.25),

            Self::TestContentCreator => TaskDefinition::new("Test Content Creation")
                .with_max_tokens(4000)
                .with_temperature(0.25),

            Self::ExplanationAgent => TaskDefinition::new("Answer Explanation")
                .with_max_tokens(1500)
                .with_temperature(0.2),

            Self::SearchAgent => TaskDefinition::new("Knowledge Search")
                .with_max_tokens(2500)
                .with_temperature(0.3),
        }
    }

    pub fn get_prompt_template(&self) -> &'static str {
        match self {
            Self::ConceptExtractor => constants::CONCEPT_EXTRACTOR_PROMPT,
            Self::FlashcardContentCreator => constants::FLASHCARD_CONTENT_CREATOR_PROMPT,
            Self::TestContentCreator => constants::TEST_CONTENT_CREATOR_PROMPT,
            Self::ExplanationAgent => constants::EXPLANATION_AGENT_PROMPT,
            Self::SearchAgent => constants::SEARCH_AGENT_PROMPT,
        }
    }
}

impl From<String> for AgentType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ConceptExtractor" => Self::ConceptExtractor,
            "FlashcardContentCreator" => Self::FlashcardContentCreator,
            "TestContentCreator" => Self::TestContentCreator,
            "ExplanationAgent" => Self::ExplanationAgent,
            "SearchAgent" => Self::SearchAgent,
            _ => Self::ConceptExtractor, // will not be the case, but use concept one as failcase
        }
    }
}
