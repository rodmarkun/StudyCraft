use crate::constants::APP_NAME;
use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;

pub struct AppPaths {
    /// Base directory for application data
    pub base_dir: PathBuf,
    /// Directory for storing library configuration
    pub config_dir: PathBuf,
    /// Directory for storing study materials
    pub materials_dir: PathBuf,
    /// Directory for storing cover images
    pub covers_dir: PathBuf,
    /// Directory for storing a Chromium binary for rendering webpages and mds
    /// We do not create this one because we wanna later check if it is initialized or not
    pub chromium_dir: PathBuf
}

impl Default for AppPaths {
    fn default() -> Self {
        let base_dir = if cfg!(target_os = "windows") {
            env::var("LOCALAPPDATA")
                .or_else(|_| env::var("APPDATA"))
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    env::var("USERPROFILE")
                        .map(|p| PathBuf::from(p).join("AppData").join("Local"))
                        .unwrap_or_else(|_| PathBuf::from("."))
                })
                .join(APP_NAME)
        } else if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|home| home.join("Library").join("Application Support"))
                .unwrap_or_else(|| PathBuf::from("."))
                .join(APP_NAME)
        } else {
            env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    dirs::home_dir()
                        .map(|home| home.join(".local").join("share"))
                        .unwrap_or_else(|| PathBuf::from("."))
                })
                .join(APP_NAME)
        };

        let config_dir = base_dir.join("config");
        let materials_dir = base_dir.join("materials");
        let covers_dir = materials_dir.join("covers");
        let chromium_dir = base_dir.join("chromium");

        Self {
            base_dir,
            config_dir,
            materials_dir,
            covers_dir,
            chromium_dir
        }
    }
}

impl AppPaths {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_dirs_exist(&self) -> std::io::Result<()> {
        if !self.base_dir.exists() {
            std::fs::create_dir_all(&self.base_dir)?;
        }
        if !self.config_dir.exists() {
            std::fs::create_dir_all(&self.config_dir)?;
        }
        if !self.materials_dir.exists() {
            std::fs::create_dir_all(&self.materials_dir)?;
        }
        if !self.covers_dir.exists() {
            std::fs::create_dir_all(&self.covers_dir)?;
        }
        Ok(())
    }
}

pub static APP_PATHS: Lazy<AppPaths> = Lazy::new(AppPaths::new);
