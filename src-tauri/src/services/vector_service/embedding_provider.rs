use crate::config::APP_PATHS;
use crate::errors::{AppError, AppResult};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use std::sync::Mutex;

pub struct LocalEmbeddingProvider {
    model: Arc<Mutex<Option<TextEmbedding>>>,
}

impl LocalEmbeddingProvider {
    // Create provider without initializing model
    pub fn new() -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
        }
    }

    // Initialize the model (download if needed)
    pub async fn initialize_model(&self) -> AppResult<()> {
        println!("Initializing local embedding model...");

        let models_dir = APP_PATHS.base_dir.join("models");
        std::fs::create_dir_all(&models_dir).map_err(|e| {
            AppError::VectorError(format!("Failed to create models directory: {}", e))
        })?;

        // Initialize with AllMiniLmL6V2
        // TODO - Allow user to change model
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_cache_dir(models_dir)
                .with_show_download_progress(true),
        )
        .map_err(|e| {
            AppError::VectorError(format!("Failed to initialize embedding model: {}", e))
        })?;

        // Store the model
        let mut model_lock = self.model.lock().unwrap();
        *model_lock = Some(model);

        println!("Local embedding model loaded successfully!");
        Ok(())
    }

    pub fn is_model_downloaded(&self) -> bool {
        let models_dir = APP_PATHS.base_dir.join("models");
        // Check if the model directory exists and contains model files
        // You might need to adjust this path based on where fastembed stores the AllMiniLML6V2 model
        let model_path = models_dir.join("models--Qdrant--all-MiniLM-L6-v2-onnx");
        model_path.exists() && model_path.is_dir()
    }

    pub fn is_initialized(&self) -> bool {
        self.model.lock().unwrap().is_some()
    }

    pub fn create_embedding(&self, text: &str) -> AppResult<Vec<f32>> {
        let model_lock = self.model.lock().unwrap();
        let model = model_lock.as_ref().ok_or_else(|| {
            AppError::VectorError(
                "Embedding model not initialized. Please initialize first.".to_string(),
            )
        })?;

        let cleaned_text = text.trim();
        if cleaned_text.is_empty() {
            return Ok(vec![0.0; 384]);
        }

        let embeddings = model
            .embed(vec![cleaned_text.to_string()], None)
            .map_err(|e| AppError::VectorError(format!("Embedding generation failed: {}", e)))?;

        if embeddings.is_empty() {
            return Err(AppError::VectorError("No embeddings generated".to_string()));
        }

        Ok(embeddings[0].clone())
    }
}
