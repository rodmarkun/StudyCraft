use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

pub async fn open_study_material(path_str: &str, app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let command = app
            .shell()
            .command("cmd")
            .args(["/C", "start", "", path_str]);
        
        command
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        let command = app
            .shell()
            .command("open")
            .arg(path_str);
        
        command
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        let command = app
            .shell()
            .command("xdg-open")
            .arg(path_str);
        
        command
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    Ok(())
}