use crate::config::app_settings::AgentSettings;
use crate::config::app_settings::StudySettings;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_study_settings(state: State<'_, AppState>) -> Result<StudySettings, String> {
    let settings = state.study_settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn update_study_settings(
    state: State<'_, AppState>,
    setting_name: String,
    value: String,
) -> Result<(), String> {
    let mut settings = state.study_settings.lock().await;
    settings
        .update_setting(&setting_name, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_study_settings_to_defaults(state: State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.study_settings.lock().await;
    settings.reset_to_defaults().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_study_settings(
    state: State<'_, AppState>,
    new_settings: StudySettings,
) -> Result<(), String> {
    let mut settings = state.study_settings.lock().await;
    *settings = new_settings;
    settings.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_settings(state: State<'_, AppState>) -> Result<AgentSettings, String> {
    let settings = state.agent_settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_agent_settings(
    state: State<'_, AppState>,
    settings: AgentSettings,
) -> Result<(), String> {
    let mut agent_settings = state.agent_settings.lock().await;
    *agent_settings = settings;
    agent_settings.save().map_err(|e| e.to_string())?;
    
    drop(agent_settings);
    
    state
        .rebuild_llm_service()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn update_agent_setting(
    state: State<'_, AppState>,
    setting_name: String,
    value: String,
) -> Result<(), String> {
    let mut settings = state.agent_settings.lock().await;
    settings
        .update_setting(&setting_name, value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_agent_settings_to_defaults(state: State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.agent_settings.lock().await;
    *settings = AgentSettings::default();
    settings.save().map_err(|e| e.to_string())
}
