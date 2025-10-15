use flyllm::{
    use_logging, GenerationRequest, LlmManager, LlmManagerResponse, LlmResult, ModelDiscovery,
    ModelInfo, ProviderType, TaskDefinition,
};
use futures::future::join_all;
use log::info;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> LlmResult<()> {
    env::set_var("RUST_LOG", "debug"); // Uncomment this to see debugging messages
    use_logging(); // Setup logging

    info!("Starting Task Routing Example");

    // --- API Keys ---
    let anthropic_api_key = env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let mistral_api_key = env::var("MISTRAL_API_KEY").expect("MISTRAL_API_KEY not set");
    let google_api_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set");

    // --- Fetch and print available models ---
    print_available_models(
        &anthropic_api_key,
        &openai_api_key,
        &mistral_api_key,
        &google_api_key,
    )
    .await;

    // --- Configure Manager using Builder ---
    let manager = LlmManager::builder()
        // Define tasks centrally
        .define_task(
            TaskDefinition::new("summary")
                .with_max_tokens(500) // Use helper or with_param
                .with_param("temperature", 0.3), // Use generic method
        )
        .define_task(
            TaskDefinition::new("creative_writing")
                .with_max_tokens(1500)
                .with_temperature(0.9),
        )
        .define_task(TaskDefinition::new("code_generation"))
        .define_task(
            TaskDefinition::new("short_poem")
                .with_max_tokens(100)
                .with_temperature(0.8),
        )
        // Add providers and link tasks by name
        // .add_instance(ProviderType::Ollama, "llama2:7b", "")
        //     .supports("summary") // Chain configuration for this provider
        //     .supports("code_generation")
        //     .custom_endpoint("http://localhost:11434/api/chat") // This is the default Ollama endpoint, but we can specify custom ones.
        //     // .enabled(true) // Optional, defaults to true
        .add_instance(
            ProviderType::Mistral,
            "mistral-large-latest",
            &mistral_api_key,
        )
        .supports("summary")
        .supports("code_generation")
        .add_instance(
            ProviderType::Anthropic,
            "claude-3-sonnet-20240229",
            &anthropic_api_key,
        )
        .supports("summary")
        .supports("creative_writing")
        .supports("code_generation")
        .add_instance(
            ProviderType::Anthropic,
            "claude-3-opus-20240229",
            &anthropic_api_key,
        )
        .supports_many(&["creative_writing", "short_poem"]) // Example using supports_many
        .add_instance(ProviderType::Google, "gemini-2.0-flash", &google_api_key)
        .supports("short_poem")
        .add_instance(ProviderType::OpenAI, "gpt-3.5-turbo", &openai_api_key)
        .supports("summary")
        // Example: Add a disabled provider
        // .add_instance(ProviderType::OpenAI, "gpt-4", &openai_api_key)
        //     .supports("creative_writing")
        //     .supports("code_generation")
        //     .enabled(false) // Explicitly disable
        // Adds a debug folder for debugging all requests made
        .debug_folder(PathBuf::from("debug_folder"))
        // Finalize the manager configuration
        .build()
        .await?; // Added .await here

    // Get provider count asynchronously
    let provider_count = manager.get_provider_count().await;
    info!("LlmManager configured with {} providers.", provider_count);

    // --- Define Requests using Builder ---
    let requests = vec![
        GenerationRequest::builder(
            "Summarize the following text: Climate change refers to long-term shifts...",
        )
        .task("summary")
        .build(),
        GenerationRequest::builder("Write a short story about a robot discovering emotions.")
            .task("creative_writing")
            .build(),
        GenerationRequest::builder(
            "Write a Python function that calculates the Fibonacci sequence up to n terms.",
        )
        .task("code_generation")
        .build(),
        // Example overriding parameters for a specific request
        GenerationRequest::builder("Write a VERY short poem about the rain.")
            .task("creative_writing") // Target creative writing task defaults...
            .max_tokens(50) // ...but override max_tokens just for this request
            // .param("temperature", 0.95) // Could override temperature too
            .build(),
        GenerationRequest::builder("Write a rust program to sum two input numbers via console.")
            .task("code_generation")
            .build(),
        GenerationRequest::builder("Craft a haiku about a silent dawn.")
            .task("short_poem")
            .build(),
    ];
    info!("Defined {} requests using builder pattern.", requests.len());

    // --- Run Requests (Sequential and Parallel) ---
    println!("\n=== Running requests sequentially... ===");
    let sequential_start = Instant::now();
    let sequential_results = manager.generate_sequentially(requests.clone()).await;
    let sequential_duration = sequential_start.elapsed();
    println!(
        "Sequential processing completed in {:?}",
        sequential_duration
    );
    print_results(&sequential_results);

    println!("\n=== Running requests in parallel... ===");
    let parallel_start = Instant::now();
    let parallel_results = manager.batch_generate(requests).await; // Use original requests vec
    let parallel_duration = parallel_start.elapsed();
    println!("Parallel processing completed in {:?}", parallel_duration);
    print_results(&parallel_results);

    info!("Task Routing Example Finished.");

    // --- Comparison ---
    println!("\n--- Comparison ---");
    println!("Sequential Duration: {:?}", sequential_duration);
    println!("Parallel Duration:   {:?}", parallel_duration);

    if parallel_duration < sequential_duration && parallel_duration.as_nanos() > 0 {
        let speedup = sequential_duration.as_secs_f64() / parallel_duration.as_secs_f64();
        println!("Parallel execution was roughly {:.2}x faster.", speedup);
    } else if parallel_duration >= sequential_duration {
        println!("Parallel execution was not faster (or was equal) in this run.");
    } else {
        println!("Parallel execution finished too quickly to measure speedup reliably.");
    }

    // Print token usage asynchronously
    manager.print_token_usage().await;

    Ok(())
}

/// Fetches models from all providers and prints them in a table format
async fn print_available_models(
    anthropic_api_key: &str,
    openai_api_key: &str,
    mistral_api_key: &str,
    google_api_key: &str,
) {
    println!("\n=== AVAILABLE MODELS ===");

    // Clone the API keys for use in the spawned tasks
    let anthropic_key = anthropic_api_key.to_string();
    let openai_key = openai_api_key.to_string();
    let mistral_key = mistral_api_key.to_string();
    let google_key = google_api_key.to_string();

    // Fetch models from different providers in parallel
    let futures = vec![
        tokio::spawn(async move { ModelDiscovery::list_anthropic_models(&anthropic_key).await }),
        tokio::spawn(async move { ModelDiscovery::list_openai_models(&openai_key).await }),
        tokio::spawn(async move { ModelDiscovery::list_mistral_models(&mistral_key).await }),
        tokio::spawn(async move { ModelDiscovery::list_google_models(&google_key).await }),
        tokio::spawn(async { ModelDiscovery::list_ollama_models(None).await }),
    ];

    let results = join_all(futures).await;

    // Create a map to store models by provider
    let mut models_by_provider: HashMap<ProviderType, Vec<ModelInfo>> = HashMap::new();

    // Define the provider order for each index
    let providers = [
        ProviderType::Anthropic,
        ProviderType::OpenAI,
        ProviderType::Mistral,
        ProviderType::Google,
        ProviderType::Ollama,
    ];

    // Process results
    for (i, result) in results.into_iter().enumerate() {
        if i >= providers.len() {
            continue;
        }
        let provider = providers[i];

        match result {
            Ok(Ok(models)) => {
                models_by_provider.insert(provider, models);
            }
            Ok(Err(e)) => {
                println!("Error fetching {} models: {}", provider, e);
            }
            Err(e) => {
                println!("Task error fetching {} models: {}", provider, e);
            }
        }
    }

    // Print models in a table format
    println!("\n{:<15} {:<40}", "PROVIDER", "MODEL NAME");
    println!("{}", "=".repeat(55));

    // Print models in the specified provider order
    for provider in providers.iter() {
        if let Some(models) = models_by_provider.get(provider) {
            for model in models {
                println!("{:<15} {:<40}", provider.to_string(), model.name);
            }
            // Add a separator between providers
            println!("{}", "-".repeat(55));
        }
    }
}

fn print_results(results: &[LlmManagerResponse]) {
    println!("\n--- Request Results ---");

    let task_labels = [
        "Summary Request",
        "Creative Writing Request",
        "Code Generation Request",
        "Short Poem Request (Override)",
        "Rust Code Request",
        "Haiku Request",
    ];

    for (i, result) in results.iter().enumerate() {
        let task_label = task_labels
            .get(i)
            .map_or_else(|| "Unknown Task", |&name| name);
        println!("{}:", task_label);
        if result.success {
            let content_preview = result.content.chars().take(150).collect::<String>();
            let ellipsis = if result.content.chars().count() > 150 {
                "..."
            } else {
                ""
            };
            println!("  Success: {}{}\n", content_preview, ellipsis);
        } else {
            println!(
                "  Error: {}\n",
                result
                    .error
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
            );
        }
    }
}
