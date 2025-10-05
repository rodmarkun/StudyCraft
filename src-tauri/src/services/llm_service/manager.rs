use crate::config::api_config::LlmUserConfig;
use crate::config::app_settings::AgentSettings;
use crate::config::keystore::ApiKeystore;
use crate::constants;
use crate::errors::{AppError, AppResult};
use crate::services::db_service::{
    FlashcardDeckOperations, StudyMaterialOperations, TestOperations,
};
use crate::services::llm_service::agents::AgentType;
use crate::services::vector_service::vector_service::RelevantChunk;
use crate::state::AppState;
use flyllm::{GenerationRequest, LlmManager, ProviderType, TaskDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConceptExtraction {
    pub concept: String,
    pub context: String,
    pub chunk_id: usize, // which chunk this came from
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConceptList {
    pub concepts: Vec<ConceptExtraction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlashcardBatch {
    pub flashcards: Vec<GeneratedCard>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratedCard {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialGenerationRequest {
    pub material_id: String,
    pub cards_to_generate: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlashcardGenerationRequest {
    pub materials: Vec<MaterialGenerationRequest>,
    pub language: Option<String>,
    pub concept: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub total_concepts_extracted: usize,
    pub total_flashcards_created: usize,
    pub concepts_used: Vec<ConceptExtraction>,
    pub flashcards: Vec<GeneratedCard>,
    pub workflow_stats: WorkflowStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStats {
    pub chunks_processed: usize,
    pub concept_extraction_requests: usize,
    pub flashcard_creation_requests: usize,
    pub concepts_per_chunk_avg: f32,
    pub flashcards_per_request_avg: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestQuestionBatch {
    pub test_questions: Vec<GeneratedTestQuestion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestMaterialRequest {
    pub material_id: String,
    pub questions_to_generate: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestGenerationRequest {
    pub materials: Vec<TestMaterialRequest>,
    pub language: Option<String>,
    pub concept: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratedTestQuestion {
    pub question: String,
    pub answers: Vec<TestAnswerGenerated>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestAnswerGenerated {
    pub answer_text: String,
    pub is_correct: bool,
    #[serde(default)]
    pub position: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestWorkflowResult {
    pub total_concepts_extracted: usize,
    pub total_questions_created: usize,
    pub concepts_used: Vec<ConceptExtraction>,
    pub test_questions: Vec<GeneratedTestQuestion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchAgentResponse {
    pub answer: String,
    pub sources: Vec<RelevantChunk>,
}

pub struct LlmService {
    manager: LlmManager,
}
impl LlmService {
    pub async fn new(config: &LlmUserConfig, agent_settings: &AgentSettings) -> Self {
        let manager = build_llm_manager_from_config(config, agent_settings).await;
        Self { manager }
    }

    pub async fn rebuild(&mut self, config: &LlmUserConfig, agent_settings: &AgentSettings) {
        self.manager = build_llm_manager_from_config(config, agent_settings).await;
    }

    pub async fn generate_flashcards_from_materials(
        &self,
        state: &State<'_, AppState>,
        request: FlashcardGenerationRequest,
    ) -> AppResult<Vec<GeneratedCard>> {
        if request.materials.is_empty() {
            return Ok(Vec::new());
        }
        println!(
            "Starting simplified flashcard generation for {} materials",
            request.materials.len()
        );

        let concept = request
            .concept
            .as_ref()
            .filter(|c| !c.trim().is_empty())
            .ok_or_else(|| {
                AppError::ValidationError(
                    "Concept is required for flashcard generation".to_string(),
                )
            })?;

        let language = request
            .language
            .clone()
            .unwrap_or_else(|| "English".to_string());

        let material_ids: Vec<String> = {
            let db_service = state.db.lock().await;
            let mut ids = Vec::new();

            for material_request in &request.materials {
                match db_service.get_study_material_by_id(&material_request.material_id) {
                    Ok(Some(_)) => ids.push(material_request.material_id.clone()),
                    Ok(None) => eprintln!("Material not found: {}", material_request.material_id),
                    Err(e) => eprintln!(
                        "Error looking up material {}: {}",
                        material_request.material_id, e
                    ),
                }
            }
            ids
        };

        if material_ids.is_empty() {
            return Err(AppError::ValidationError(
                "No valid materials found".to_string(),
            ));
        }

        // Perform vector search
        let relevant_chunks = {
            let vector_service = state.vector.lock().await;
            let total_cards_requested: u32 =
                request.materials.iter().map(|m| m.cards_to_generate).sum();
            let chunks_to_retrieve = std::cmp::max(5, total_cards_requested / 2);

            vector_service
                .search_material_specific(concept, &material_ids, chunks_to_retrieve)
                .await
                .map_err(|e| AppError::VectorError(format!("Vector search failed: {}", e)))?
        };

        if relevant_chunks.is_empty() {
            return Err(AppError::ValidationError(format!(
                "No relevant content found for concept '{}'",
                concept
            )));
        }

        // Extract chunk texts
        let chunk_texts: Vec<String> = relevant_chunks
            .iter()
            .map(|chunk| format!("From {}: {}", chunk.metadata.file_display_name, chunk.text))
            .collect();

        let total_flashcards_needed: u32 =
            request.materials.iter().map(|m| m.cards_to_generate).sum();

        // Get agent settings from state
        let agent_settings = state.agent_settings.lock().await;

        let workflow_result = self
            .generate_flashcards_from_material_chunks(
                chunk_texts,
                total_flashcards_needed,
                concept.clone(),
                Some(language),
                &agent_settings,
            )
            .await?;

        let generated_cards: Vec<GeneratedCard> = workflow_result.flashcards;

        println!(
            "Successfully generated {} flashcards using simplified workflow",
            generated_cards.len()
        );

        Ok(generated_cards)
    }

    pub async fn generate_flashcards_from_material_chunks(
        &self,
        chunks: Vec<String>,
        total_flashcards_needed: u32,
        focus_concept: String,
        language: Option<String>,
        agent_settings: &AgentSettings,
    ) -> AppResult<WorkflowResult> {
        let language = language.unwrap_or_else(|| "English".to_string());

        println!("=== CREATING FLASHCARDS ===");
        println!(
            "Chunks: {}, Target flashcards: {}, Focus: '{}', Language: '{}'",
            chunks.len(),
            total_flashcards_needed,
            focus_concept,
            language
        );

        // Extract concepts from all chunks in parallel
        let all_concepts = self
            .extract_concepts_from_chunks_parallel(
                chunks.clone(),
                focus_concept.clone(),
                language.clone(),
                agent_settings,
            )
            .await?;

        let total_concepts = all_concepts.len();
        println!(
            "Extracted {} total concepts from {} chunks",
            total_concepts,
            chunks.len()
        );

        // Create flashcards directly from concepts
        let flashcards = self
            .create_flashcards_from_concepts(
                all_concepts.clone(),
                total_flashcards_needed,
                language.clone(),
                agent_settings,
            )
            .await?;

        println!(
            "Created {} flashcards from {} concepts",
            flashcards.len(),
            total_concepts
        );

        let workflow_stats = WorkflowStats {
            chunks_processed: chunks.len(),
            concept_extraction_requests: chunks.len(),
            flashcard_creation_requests: self.calculate_flashcard_requests(total_flashcards_needed),
            concepts_per_chunk_avg: total_concepts as f32 / chunks.len() as f32,
            flashcards_per_request_avg: if flashcards.is_empty() {
                0.0
            } else {
                total_flashcards_needed as f32
                    / self.calculate_flashcard_requests(total_flashcards_needed) as f32
            },
        };

        let result = WorkflowResult {
            total_concepts_extracted: total_concepts,
            total_flashcards_created: flashcards.len(),
            concepts_used: all_concepts,
            flashcards,
            workflow_stats,
        };

        println!(
            "Success: {} concepts → {} flashcards",
            total_concepts, result.total_flashcards_created
        );

        Ok(result)
    }

    /// Extract concepts from chunks in parallel
    async fn extract_concepts_from_chunks_parallel(
        &self,
        chunks: Vec<String>,
        focus_concept: String,
        language: String,
        agent_settings: &AgentSettings,
    ) -> AppResult<Vec<ConceptExtraction>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        let task_definition =
            self.get_task_definition_from_settings(&AgentType::ConceptExtractor, agent_settings);
        let prompt_template = agent_settings
            .agent_prompts
            .get(&AgentType::ConceptExtractor)
            .map(|s| s.as_str())
            .unwrap_or(AgentType::ConceptExtractor.get_prompt_template());

        let requests: Vec<GenerationRequest> = chunks
            .iter()
            .enumerate()
            .map(|(chunk_id, chunk)| {
                let formatted_prompt = prompt_template
                    .replace("{content}", chunk)
                    .replace("{focus_concept}", &focus_concept)
                    .replace("{language}", &language)
                    .replace("{chunk_id}", chunk_id.to_string().as_str());

                GenerationRequest::builder(formatted_prompt)
                    .task(&task_definition.name)
                    .build()
            })
            .collect();

        println!(
            "Executing {} concept extraction requests in parallel",
            requests.len()
        );

        let responses = self.manager.batch_generate(requests).await;
        let mut all_concepts = Vec::new();

        // Process responses
        for (chunk_id, response) in responses.iter().enumerate() {
            if response.success {
                let cleaned_content = &clean_json_content(&response.content);
                match serde_json::from_str::<ConceptList>(cleaned_content) {
                    Ok(concept_list) => {
                        for mut concept in concept_list.concepts {
                            concept.chunk_id = chunk_id;
                            all_concepts.push(concept);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse concepts for chunk {}: {} | {:#?}", chunk_id, e, response);
                        all_concepts.push(ConceptExtraction {
                            concept: format!(
                                "Error extracting concept {} from chunk {}",
                                focus_concept,
                                chunk_id + 1
                            ),
                            context: "Failed to automatically extract specific concepts"
                                .to_string(),
                            chunk_id,
                        });
                    }
                }
            } else {
                eprintln!(
                    "Concept extraction failed for chunk {}: {}",
                    chunk_id,
                    response
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
                all_concepts.push(ConceptExtraction {
                    concept: format!(
                        "Error extracting concept {} from chunk {}",
                        focus_concept,
                        chunk_id + 1
                    ),
                    context: "Extraction failed, manual review needed".to_string(),
                    chunk_id,
                });
            }
        }

        Ok(all_concepts)
    }

    /// Create flashcards from concepts using smart batching to avoid duplicates
    async fn create_flashcards_from_concepts(
        &self,
        all_concepts: Vec<ConceptExtraction>,
        target_flashcards: u32,
        language: String,
        agent_settings: &AgentSettings,
    ) -> AppResult<Vec<GeneratedCard>> {
        if all_concepts.is_empty() || target_flashcards == 0 {
            return Ok(Vec::new());
        }

        // Creates balanced batches that distribute concepts across requests
        let flashcards_per_batch =
            std::cmp::min(constants::MAX_FLASHCARDS_PER_REQUEST, target_flashcards); // Max 5 flashcards per request

        // Request needed calculation
        let total_requests = (target_flashcards + flashcards_per_batch - 1) / flashcards_per_batch;

        println!(
            "Smart batching: {} concepts → {} requests ({} flashcards per request)",
            all_concepts.len(),
            total_requests,
            flashcards_per_batch
        );

        // Distributing concepts across requests to avoid overlap
        let concept_batches =
            self.distribute_concepts_evenly(all_concepts, total_requests as usize);
        let task_definition = self
            .get_task_definition_from_settings(&AgentType::FlashcardContentCreator, agent_settings);
        let prompt_template = agent_settings
            .agent_prompts
            .get(&AgentType::FlashcardContentCreator)
            .map(|s| s.as_str())
            .unwrap_or(AgentType::FlashcardContentCreator.get_prompt_template());

        let mut requests = Vec::new();
        let mut expected_flashcards_per_request = Vec::new();

        for (batch_id, batch_concepts) in concept_batches.iter().enumerate() {
            if batch_concepts.is_empty() {
                continue;
            }

            let remaining_flashcards = target_flashcards - (batch_id as u32 * flashcards_per_batch);
            let flashcards_for_this_batch =
                std::cmp::min(flashcards_per_batch, remaining_flashcards);

            if flashcards_for_this_batch == 0 {
                break;
            }

            let concepts_json = serde_json::to_string(&ConceptList {
                concepts: batch_concepts.clone(),
            })
            .unwrap_or_else(|_| "[]".to_string());

            let formatted_prompt = prompt_template
                .replace("{count}", &flashcards_for_this_batch.to_string())
                .replace("{concepts}", &concepts_json)
                .replace("{language}", &language)
                .replace("{batch_id}", &(batch_id + 1).to_string());

            let request = GenerationRequest::builder(formatted_prompt)
                .task(&task_definition.name)
                .build();

            requests.push(request);
            expected_flashcards_per_request.push(flashcards_for_this_batch);
        }

        println!("Executing {} flashcard creation requests", requests.len());

        let responses = self.manager.batch_generate(requests).await;
        let mut all_flashcards = Vec::new();
        let mut used_questions = HashSet::new();
        for (batch_id, response) in responses.iter().enumerate() {
            let expected_count = expected_flashcards_per_request.get(batch_id).unwrap_or(&0);

            if response.success {
                let cleaned_content = &clean_json_content(&response.content);
                match serde_json::from_str::<FlashcardBatch>(cleaned_content) {
                    Ok(batch_result) => {
                        let mut added_count = 0;

                        for flashcard in batch_result.flashcards {
                            // duplicate checking
                            let question_key = flashcard.question.to_lowercase();
                            if !used_questions.contains(&question_key)
                                && all_flashcards.len() < target_flashcards as usize
                            {
                                used_questions.insert(question_key);
                                all_flashcards.push(flashcard);
                                added_count += 1;
                            }
                        }

                        println!(
                            "Batch {}: Created {} flashcards (expected {})",
                            batch_id + 1,
                            added_count,
                            expected_count
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to parse flashcards for batch {}: {}", batch_id, e);
                        if let Some(batch_concepts) = concept_batches.get(batch_id) {
                            for (i, concept) in batch_concepts.iter().enumerate() {
                                if all_flashcards.len() >= target_flashcards as usize {
                                    break;
                                }
                                if i >= *expected_count as usize {
                                    break;
                                }

                                all_flashcards.push(GeneratedCard {
                                    question: format!(
                                        "Flashcard creation error while generating for: {}?",
                                        concept.concept
                                    ),
                                    answer: format!("Context: {}", concept.context),
                                });
                            }
                        }
                    }
                }
            } else {
                eprintln!(
                    "Flashcard creation failed for batch {}: {}",
                    batch_id,
                    response
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
            }
        }

        // Trim to exact target if we somehow got more
        all_flashcards.truncate(target_flashcards as usize);

        println!(
            "Final result: {} unique flashcards created (target: {})",
            all_flashcards.len(),
            target_flashcards
        );

        Ok(all_flashcards)
    }

    /// Generate flashcard explanation
    pub async fn generate_flashcard_explanation(
        &self,
        question: String,
        answer: String,
        agent_settings: &AgentSettings,
    ) -> AppResult<String> {
        match self
            .execute_flashcard_explanation(&question, &answer, agent_settings)
            .await
        {
            Ok(explanation_response) => {
                let cleaned_content = &clean_json_content(&explanation_response);
                match serde_json::from_str::<serde_json::Value>(cleaned_content) {
                    Ok(explanation_json) => {
                        if let Some(explanation_text) =
                            explanation_json.get("explanation").and_then(|e| e.as_str())
                        {
                            Ok(explanation_text.to_string())
                        } else {
                            Ok(explanation_response)
                        }
                    }
                    Err(e) => Err(AppError::JsonError(format!(
                        "Failed to parse explanation JSON: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Search study materials
    pub async fn search_study_materials(
        &self,
        state: &State<'_, AppState>,
        query: String,
    ) -> AppResult<SearchAgentResponse> {
        let study_settings = state.study_settings.lock().await;
        if query.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Search query cannot be empty".to_string(),
            ));
        }
        let has_materials = {
            let db_service = state.db.lock().await;
            match db_service.get_study_materials() {
                Ok(materials) => !materials.is_empty(),
                Err(_) => false,
            }
        };
        if !has_materials {
            return Err(AppError::ValidationError(
                "No study materials found. Please upload some PDF files first.".to_string(),
            ));
        }
        // Perform vector search
        let relevant_chunks = {
            let vector_service = state.vector.lock().await;
            let num_chunks = std::cmp::max(study_settings.max_chunks_to_recover_from_search, constants::VECDB_MIN_CHUNKS_RECOVERED_AI_SEARCH);
            let num_chunks = std::cmp::min(num_chunks, constants::VECDB_MAX_CHUNKS_RECOVERED_AI_SEARCH);
            vector_service.search_global(&query, num_chunks).await.map_err(|e| {
                AppError::VectorError(format!("Failed to search through your study materials {e}"))
            })?
        };
        if relevant_chunks.is_empty() {
            return Err(AppError::ValidationError(
                "No relevant content found in your study materials for this query.".to_string(),
            ));
        }
        // Prepare chunk data for SearchAgent
        let document_chunks_with_metadata: Vec<(String, Option<String>, Option<f32>)> =
            relevant_chunks
                .iter()
                .map(|chunk| {
                    (
                        chunk.text.clone(),
                        Some(chunk.metadata.file_display_name.clone()),
                        Some(chunk.score),
                    )
                })
                .collect();
        // Get agent settings from state
        let agent_settings = state.agent_settings.lock().await;
        // Use SearchAgent
        match self
            .execute_knowledge_search_with_metadata(
                &query,
                &document_chunks_with_metadata,
                &agent_settings,
            )
            .await
        {
            Ok(search_response) => {
                let cleaned_content = &clean_json_content(&search_response);
                match serde_json::from_str::<serde_json::Value>(cleaned_content) {
                    Ok(search_result) => {
                        let answer = search_result
                            .get("answer")
                            .and_then(|a| a.as_str())
                            .unwrap_or("No answer provided")
                            .to_string();

                        let sources = search_result
                            .get("sources")
                            .and_then(|s| s.as_array())
                            .map(|arr| {
                                let mut seen_files = HashSet::new();
                                let mut unique_sources = Vec::new();
                                
                                for source_value in arr.iter() {
                                    if let Some(source_text) = source_value.as_str() {
                                        if let Some(chunk) = relevant_chunks
                                            .iter()
                                            .find(|chunk| {
                                                source_text.contains(&chunk.metadata.file_display_name)
                                            })
                                        {
                                            // Only add if we haven't seen this file before
                                            if seen_files.insert(chunk.metadata.file_display_name.clone()) {
                                                unique_sources.push(chunk.clone());
                                            }
                                        }
                                    }
                                }
                                unique_sources
                            })
                            .unwrap_or_else(Vec::new);

                        Ok(SearchAgentResponse { answer, sources })
                    }
                    Err(e) => Err(AppError::JsonError(format!(
                        "Failed to parse SearchAgent response: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn generate_test_questions_from_materials(
        &self,
        state: &State<'_, AppState>,
        request: TestGenerationRequest,
    ) -> AppResult<Vec<GeneratedTestQuestion>> {
        if request.materials.is_empty() {
            return Ok(Vec::new());
        }

        println!(
            "Starting test question generation for {} materials",
            request.materials.len()
        );

        let concept = request
            .concept
            .as_ref()
            .filter(|c| !c.trim().is_empty())
            .ok_or_else(|| {
                AppError::ValidationError(
                    "Concept is required for test question generation".to_string(),
                )
            })?;

        let language = request
            .language
            .clone()
            .unwrap_or_else(|| "English".to_string());

        let material_ids: Vec<String> = {
            let db_service = state.db.lock().await;
            let mut ids = Vec::new();

            for material_request in &request.materials {
                match db_service.get_study_material_by_id(&material_request.material_id) {
                    Ok(Some(_)) => ids.push(material_request.material_id.clone()),
                    Ok(None) => eprintln!("Material not found: {}", material_request.material_id),
                    Err(e) => eprintln!(
                        "Error looking up material {}: {}",
                        material_request.material_id, e
                    ),
                }
            }
            ids
        };

        if material_ids.is_empty() {
            return Err(AppError::ValidationError(
                "No valid materials found".to_string(),
            ));
        }

        // Perform vector search
        let relevant_chunks = {
            let vector_service = state.vector.lock().await;
            let total_questions_requested: u32 = request
                .materials
                .iter()
                .map(|m| m.questions_to_generate)
                .sum();
            let chunks_to_retrieve = std::cmp::max(5, total_questions_requested / 2);

            vector_service
                .search_material_specific(concept, &material_ids, chunks_to_retrieve)
                .await
                .map_err(|e| AppError::VectorError(format!("Vector search failed: {}", e)))?
        };

        if relevant_chunks.is_empty() {
            return Err(AppError::ValidationError(format!(
                "No relevant content found for concept '{}'",
                concept
            )));
        }

        // Extract chunk texts
        let chunk_texts: Vec<String> = relevant_chunks
            .iter()
            .map(|chunk| format!("From {}: {}", chunk.metadata.file_display_name, chunk.text))
            .collect();

        let total_questions_needed: u32 = request
            .materials
            .iter()
            .map(|m| m.questions_to_generate)
            .sum();

        // Get agent settings from state
        let agent_settings = state.agent_settings.lock().await;

        let workflow_result = self
            .generate_test_questions_from_material_chunks(
                chunk_texts,
                total_questions_needed,
                concept.clone(),
                Some(language),
                &agent_settings,
            )
            .await?;

        let generated_questions: Vec<GeneratedTestQuestion> = workflow_result.test_questions;

        println!(
            "Successfully generated {} test questions",
            generated_questions.len()
        );

        Ok(generated_questions)
    }

    pub async fn generate_test_questions_from_material_chunks(
        &self,
        chunks: Vec<String>,
        total_questions_needed: u32,
        focus_concept: String,
        language: Option<String>,
        agent_settings: &AgentSettings,
    ) -> AppResult<TestWorkflowResult> {
        let language = language.unwrap_or_else(|| "English".to_string());

        println!("=== CREATING TEST QUESTIONS ===");
        println!(
            "Chunks: {}, Target questions: {}, Focus: '{}', Language: '{}'",
            chunks.len(),
            total_questions_needed,
            focus_concept,
            language
        );

        // Extract concepts from all chunks in parallel (reuse existing method)
        let all_concepts = self
            .extract_concepts_from_chunks_parallel(
                chunks.clone(),
                focus_concept.clone(),
                language.clone(),
                agent_settings,
            )
            .await?;

        let total_concepts = all_concepts.len();
        println!(
            "Extracted {} total concepts from {} chunks",
            total_concepts,
            chunks.len()
        );

        // Create test questions directly from concepts
        let test_questions = self
            .create_test_questions_from_concepts(
                all_concepts.clone(),
                total_questions_needed,
                language.clone(),
                agent_settings,
            )
            .await?;

        println!(
            "Created {} test questions from {} concepts",
            test_questions.len(),
            total_concepts
        );

        let result = TestWorkflowResult {
            total_concepts_extracted: total_concepts,
            total_questions_created: test_questions.len(),
            concepts_used: all_concepts,
            test_questions,
        };

        println!(
            "Success: {} concepts → {} test questions",
            total_concepts, result.total_questions_created
        );

        Ok(result)
    }

    /// Create test questions from concepts using smart batching
    async fn create_test_questions_from_concepts(
        &self,
        all_concepts: Vec<ConceptExtraction>,
        target_questions: u32,
        language: String,
        agent_settings: &AgentSettings,
    ) -> AppResult<Vec<GeneratedTestQuestion>> {
        if all_concepts.is_empty() || target_questions == 0 {
            return Ok(Vec::new());
        }

        // Max 3 test questions per request (they're more complex than flashcards)
        let questions_per_batch = std::cmp::min(3, target_questions);
        let total_requests = (target_questions + questions_per_batch - 1) / questions_per_batch;

        println!(
            "Smart batching: {} concepts → {} requests ({} questions per request)",
            all_concepts.len(),
            total_requests,
            questions_per_batch
        );

        // Distribute concepts across requests
        let concept_batches =
            self.distribute_concepts_evenly(all_concepts, total_requests as usize);
        let task_definition =
            self.get_task_definition_from_settings(&AgentType::TestContentCreator, agent_settings);
        let prompt_template = agent_settings
            .agent_prompts
            .get(&AgentType::TestContentCreator)
            .map(|s| s.as_str())
            .unwrap_or(AgentType::TestContentCreator.get_prompt_template());

        let mut requests = Vec::new();
        let mut expected_questions_per_request = Vec::new();

        for (batch_id, batch_concepts) in concept_batches.iter().enumerate() {
            if batch_concepts.is_empty() {
                continue;
            }

            let remaining_questions = target_questions - (batch_id as u32 * questions_per_batch);
            let questions_for_this_batch = std::cmp::min(questions_per_batch, remaining_questions);

            if questions_for_this_batch == 0 {
                break;
            }

            let concepts_json = serde_json::to_string(&ConceptList {
                concepts: batch_concepts.clone(),
            })
            .unwrap_or_else(|_| "[]".to_string());

            let formatted_prompt = prompt_template
                .replace("{count}", &questions_for_this_batch.to_string())
                .replace("{concepts}", &concepts_json)
                .replace("{language}", &language);

            let request = GenerationRequest::builder(formatted_prompt)
                .task(&task_definition.name)
                .build();

            requests.push(request);
            expected_questions_per_request.push(questions_for_this_batch);
        }

        println!(
            "Executing {} test question creation requests",
            requests.len()
        );

        let responses = self.manager.batch_generate(requests).await;
        let mut all_questions = Vec::new();
        let mut used_questions = HashSet::new();

        for (batch_id, response) in responses.iter().enumerate() {
            let expected_count = expected_questions_per_request.get(batch_id).unwrap_or(&0);

            if response.success {
                let cleaned_content = &clean_json_content(&response.content);
                match serde_json::from_str::<TestQuestionBatch>(cleaned_content) {
                    Ok(batch_result) => {
                        let mut added_count = 0;

                        for mut question in batch_result.test_questions {
                            // Duplicate checking
                            let question_key = question.question.to_lowercase();
                            if !used_questions.contains(&question_key)
                                && all_questions.len() < target_questions as usize
                            {
                                used_questions.insert(question_key);

                                // Simply assign positions 0, 1, 2, 3... to all answers
                                for (idx, answer) in question.answers.iter_mut().enumerate() {
                                    answer.position = idx as u32;
                                }

                                all_questions.push(question);
                                added_count += 1;
                            }
                        }

                        println!(
                            "Batch {}: Created {} test questions (expected {})",
                            batch_id + 1,
                            added_count,
                            expected_count
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to parse test questions for batch {}: {}",
                            batch_id, e
                        );
                        // Create fallback questions
                        if let Some(batch_concepts) = concept_batches.get(batch_id) {
                            for (i, concept) in batch_concepts.iter().enumerate() {
                                if all_questions.len() >= target_questions as usize {
                                    break;
                                }
                                if i >= *expected_count as usize {
                                    break;
                                }

                                all_questions.push(GeneratedTestQuestion {
                                    question: format!(
                                        "Test question generation error for concept: {}?",
                                        concept.concept
                                    ),
                                    answers: vec![TestAnswerGenerated {
                                        answer_text: "Generation failed".to_string(),
                                        is_correct: true,
                                        position: 0,
                                    }],
                                });
                            }
                        }
                    }
                }
            } else {
                eprintln!(
                    "Test question creation failed for batch {}: {}",
                    batch_id,
                    response
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
            }
        }

        // Trim to exact target if we somehow got more
        all_questions.truncate(target_questions as usize);

        println!(
            "Final result: {} unique test questions created (target: {})",
            all_questions.len(),
            target_questions
        );

        Ok(all_questions)
    }

    async fn execute_flashcard_explanation(
        &self,
        question: &str,
        answer: &str,
        agent_settings: &AgentSettings,
    ) -> AppResult<String> {
        let task_definition =
            self.get_task_definition_from_settings(&AgentType::ExplanationAgent, agent_settings);
        let prompt_template = agent_settings
            .agent_prompts
            .get(&AgentType::ExplanationAgent)
            .map(|s| s.as_str())
            .unwrap_or(AgentType::ExplanationAgent.get_prompt_template());

        let formatted_prompt = prompt_template
            .replace("{question}", question)
            .replace("{answer}", answer);

        let request = GenerationRequest::builder(formatted_prompt)
            .task(&task_definition.name)
            .build();

        let responses = self.manager.batch_generate(vec![request]).await;

        if let Some(response) = responses.first() {
            if response.success {
                Ok(response.content.clone())
            } else {
                Err(AppError::LlmError(format!(
                    "Explanation generation failed: {}",
                    response
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                )))
            }
        } else {
            Err(AppError::LlmError(
                "No response received for explanation generation".to_string(),
            ))
        }
    }

    async fn execute_knowledge_search_with_metadata(
        &self,
        query: &str,
        document_chunks: &[(String, Option<String>, Option<f32>)],
        agent_settings: &AgentSettings,
    ) -> AppResult<String> {
        let task_definition =
            self.get_task_definition_from_settings(&AgentType::SearchAgent, agent_settings);
        let prompt_template = agent_settings
            .agent_prompts
            .get(&AgentType::SearchAgent)
            .map(|s| s.as_str())
            .unwrap_or(AgentType::SearchAgent.get_prompt_template());

        let formatted_chunks = document_chunks
            .iter()
            .enumerate()
            .map(|(i, (content, source, score))| {
                let mut chunk_header = String::new();
                if let Some(src) = source {
                    chunk_header.push_str(&format!(" (Source: {})", src));
                }
                format!("{}:\n{}", chunk_header, content)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let formatted_prompt = prompt_template
            .replace("{query}", query)
            .replace("{document_chunks}", &formatted_chunks);

        let request = GenerationRequest::builder(formatted_prompt)
            .task(&task_definition.name)
            .build();

        let responses = self.manager.batch_generate(vec![request]).await;

        if let Some(response) = responses.first() {
            if response.success {
                Ok(response.content.clone())
            } else {
                Err(AppError::LlmError(format!(
                    "Knowledge search failed: {}",
                    response
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                )))
            }
        } else {
            Err(AppError::LlmError(
                "No response received for knowledge search".to_string(),
            ))
        }
    }

    /// Distribute concepts evenly across batches to minimize overlap
    fn distribute_concepts_evenly(
        &self,
        concepts: Vec<ConceptExtraction>,
        num_batches: usize,
    ) -> Vec<Vec<ConceptExtraction>> {
        if num_batches == 0 || concepts.is_empty() {
            return vec![];
        }

        let mut batches: Vec<Vec<ConceptExtraction>> = vec![Vec::new(); num_batches];

        for (i, concept) in concepts.into_iter().enumerate() {
            let batch_index = i % num_batches;
            batches[batch_index].push(concept);
        }

        batches
            .into_iter()
            .filter(|batch| !batch.is_empty())
            .collect()
    }

    /// Calculate the number of flashcard creation requests needed
    fn calculate_flashcard_requests(&self, target_flashcards: u32) -> usize {
        let flashcards_per_request = 5; // Max flashcards per request
        ((target_flashcards + flashcards_per_request - 1) / flashcards_per_request) as usize
    }

    /// Helper method to create TaskDefinition from agent settings
    fn get_task_definition_from_settings(
        &self,
        agent_type: &AgentType,
        agent_settings: &AgentSettings,
    ) -> TaskDefinition {
        let max_tokens = agent_settings
            .agent_max_tokens
            .get(agent_type)
            .copied()
            .unwrap_or_else(|| {
                agent_type
                    .get_task_definition()
                    .parameters
                    .get("max_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3000) as u32
            });

        let temperature = agent_settings
            .agent_temperature
            .get(agent_type)
            .copied()
            .unwrap_or_else(|| {
                agent_type
                    .get_task_definition()
                    .parameters
                    .get("temperature")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.3) as f32
            });

        TaskDefinition::new(&format!("{} Task", agent_type.as_str()))
            .with_max_tokens(max_tokens)
            .with_temperature(temperature)
    }
}

async fn build_llm_manager_from_config(
    config: &LlmUserConfig,
    agent_settings: &AgentSettings,
) -> LlmManager {
    let mut builder = LlmManager::builder();

    // Create keystore instance
    let keystore = ApiKeystore::new();

    // Define tasks using agent settings
    for agent_type in AgentType::all() {
        let max_tokens = agent_settings
            .agent_max_tokens
            .get(&agent_type)
            .copied()
            .unwrap_or_else(|| {
                agent_type
                    .get_task_definition()
                    .parameters
                    .get("max_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3000) as u32
            });

        let temperature = agent_settings
            .agent_temperature
            .get(&agent_type)
            .copied()
            .unwrap_or_else(|| {
                agent_type
                    .get_task_definition()
                    .parameters
                    .get("temperature")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.3) as f32
            });

        let task_def = TaskDefinition::new(&format!("{} Task", agent_type.as_str()))
            .with_max_tokens(max_tokens)
            .with_temperature(temperature);

        builder = builder.define_task(task_def);
    }

    let mut provider_count = 0;

    // For each enabled provider, create instances for each agent type that has a model configured
    for (provider_type, provider_config) in &config.provider_configs {
        if !provider_config.enabled {
            continue;
        }

        // Get API key from keystore
        let api_key = if *provider_type == ProviderType::Ollama {
            String::new()
        } else {
            let provider_str = format!("{:?}", provider_type);
            match keystore.get_api_key(&provider_str) {
                Ok(Some(key)) => key,
                Ok(None) => {
                    eprintln!(
                        "Skipping provider {:?}: API key required but not found in keystore",
                        provider_type
                    );
                    continue;
                }
                Err(e) => {
                    eprintln!(
                        "Skipping provider {:?}: Failed to retrieve API key from keystore: {}",
                        provider_type, e
                    );
                    continue;
                }
            }
        };

        // Create a separate instance for each agent type that has a model configured for this provider
        for agent_type in AgentType::all() {
            if let Some(agent_models) = agent_settings.agent_models_per_provider.get(&agent_type) {
                if let Some(model_name) = agent_models.get(provider_type) {
                    let task_name = format!("{} Task", agent_type.as_str());
                    
                    println!(
                        "Adding provider {:?} with model '{}' for task '{}'",
                        provider_type, model_name, task_name
                    );

                    builder = match provider_type {
                        ProviderType::Ollama => builder
                            .add_instance(*provider_type, model_name, "")
                            .supports(&task_name)
                            .custom_endpoint(constants::OLLAMA_CUSTOM_ENDPOINT),
                        _ => builder
                            .add_instance(*provider_type, model_name, &api_key)
                            .supports(&task_name),
                    };

                    provider_count += 1;
                }
            }
        }
    }

    match builder.build().await {
        Ok(manager) => {
            println!(
                "LLM Manager successfully built with {} provider instances",
                provider_count
            );
            manager
        }
        Err(e) => {
            eprintln!(
                "Failed to build LLM Manager: {}. Creating empty manager.",
                e
            );
            LlmManager::new()
        }
    }
}

fn clean_json_content(content: &str) -> String {
    let trimmed = content.trim();
    let without_prefix = trimmed.strip_prefix("```json")
                                .or_else(|| trimmed.strip_prefix("```"))
                                .unwrap_or(trimmed);
   
    let without_suffix = without_prefix.strip_suffix("```")
                                       .unwrap_or(without_prefix);
    
    let cleaned = without_suffix
           .replace("**", "")  // Remove bold markers
           .replace("*****", "")  // Remove emphasis markers
           .replace("*", "");  // Remove remaining asterisks

    sanitize_json_strings(cleaned.trim())
}

fn sanitize_json_strings(json_str: &str) -> String {
    let mut result = String::new();
    let mut chars = json_str.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;
    
    while let Some(ch) = chars.next() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }
        
        match ch {
            '"' => {
                in_string = !in_string;
                result.push(ch);
            }
            '\\' if in_string => {
                if let Some(&next_ch) = chars.peek() {
                    if matches!(next_ch, 'n' | 'r' | 't' | '"' | '\\' | '/') {
                        result.push(ch);
                    } else {
                        result.push(ch);
                    }
                } else {
                    result.push(ch);
                }
            }
            '\n' if in_string => {
                result.push_str("\\n");
            }
            '\r' if in_string => {
                result.push_str("\\r");
            }
            '\t' if in_string => {
                result.push_str("\\t");
            }
            _ => {
                result.push(ch);
            }
        }
    }
    
    result
}