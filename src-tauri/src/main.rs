#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod config;
mod constants;
mod errors;
mod materials;
mod os_dep;
mod services;
mod state;
mod utils;

use crate::commands::*;
use crate::config::APP_PATHS;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        // std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
    }
    let app_state = AppState::new().await;

    APP_PATHS.ensure_dirs_exist().unwrap_or_else(|e| {
        eprintln!("Failed to create directories: {}", e);
        std::process::exit(1);
    });

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let handle = app.handle().clone();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Study Material Commands
            commands::materials::study_material::add_study_material,
            commands::materials::study_material::add_study_material_from_path,
            commands::materials::study_material::add_study_material_from_link,
            commands::materials::study_material::delete_study_material,
            commands::materials::study_material::get_study_materials,
            commands::materials::study_material::open_material,
            commands::materials::study_material::get_markdown_content,
            commands::materials::study_material::save_markdown_content,
            commands::materials::study_material::get_material_markdown_path,
            commands::materials::study_material::update_material_name,
            commands::materials::study_material::get_cover_image_raw,
            commands::materials::study_material::initialize_chromium,
            commands::materials::study_material::is_chromium_downloaded,
            // Review Material Commands (General)
            commands::materials::review_material::get_review_materials,
            commands::materials::review_material::get_material_tags,
            commands::materials::review_material::update_last_review,
            // Flashcard Deck Commands
            commands::materials::review_material::create_flashcard_deck,
            commands::materials::review_material::get_flashcards,
            commands::materials::review_material::get_flashcard_counts,
            commands::materials::review_material::get_flashcards_for_review,
            commands::materials::review_material::update_flashcard_after_review,
            commands::materials::review_material::delete_flashcard_deck,
            commands::materials::review_material::update_flashcard_deck_name,
            commands::materials::review_material::update_flashcard_deck,
            commands::materials::review_material::get_flashcard_deck_details,
            commands::materials::review_material::export_flashcard_deck_to_anki,
            // Test Commands
            commands::materials::review_material::create_test,
            commands::materials::review_material::get_test_details,
            commands::materials::review_material::get_test_questions,
            commands::materials::review_material::update_test,
            commands::materials::review_material::delete_test,
            commands::materials::review_material::update_test_name,
            // Review Session Commands
            commands::materials::review_material::create_review_session,
            commands::materials::review_material::create_flashcard_review_session,
            commands::materials::review_material::create_test_review_session,
            commands::materials::review_material::get_flashcard_review_sessions_by_material,
            commands::materials::review_material::get_test_review_sessions_by_material,
            commands::materials::review_material::get_all_review_sessions,
            commands::materials::review_material::get_review_session_stats,
            commands::materials::review_material::get_recent_review_sessions,
            // Tag Commands
            tags::add_tag_to_material,
            tags::remove_tag_from_material,
            tags::get_all_tags,
            tags::get_available_tags,
            tags::add_available_tag,
            tags::remove_available_tag,
            // API Settings Commands
            api::get_api_key,
            api::set_api_key,
            api::clear_api_key,
            api::validate_api_key_and_fetch_models,
            api::refresh_provider_models,
            api::set_provider_enabled,
            api::is_provider_enabled,
            api::get_providers,
            api::get_enabled_providers,
            api::get_provider_config,
            api::get_all_provider_configs,
            api::get_config_summary,
            api::save_config,
            api::load_config,
            api::is_provider_configured,
            api::validate_provider_setup,
            api::get_all_agent_model_configs,
            api::set_provider_custom_endpoint,
            api::get_provider_custom_endpoint,
            api::set_ollama_endpoint,
            api::get_ollama_endpoint,
            // Settings commands
            settings::get_study_settings,
            settings::update_study_settings,
            settings::reset_study_settings_to_defaults,
            settings::save_study_settings,
            settings::get_agent_settings,
            settings::save_agent_settings,
            settings::update_agent_setting,
            settings::reset_agent_settings_to_defaults,
            // LLM Manager Commands
            llm::generate_flashcards,
            llm::explain_flashcard,
            llm::search_in_study_materials,
            llm::generate_test_questions,
            // Vector DB Commands
            vector::search_study_materials_global,
            vector::search_specific_materials,
            vector::get_vector_stats,
            vector::reindex_material,
            vector::delete_material_from_vector,
            vector::initialize_embedding_model,
            vector::is_embedding_model_ready,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Run app
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {}
        _ => {}
    });
}
