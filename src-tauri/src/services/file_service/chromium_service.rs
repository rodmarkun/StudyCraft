use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use crate::{config::APP_PATHS, errors::{AppError, AppResult}};
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions};

pub struct ChromiumService {
    chromium_path: Arc<Mutex<Option<PathBuf>>>,
}

impl ChromiumService {
    pub fn new() -> Self {
        Self {
            chromium_path: Arc::new(Mutex::new(None))
        }
    }

    pub async fn initialize(&self) -> AppResult<()> {
        let chromium_dir = Self::get_chromium_cache_dir()?;
        std::fs::create_dir_all(&chromium_dir).map_err(|e| {
            AppError::ChromiumError(format!("Failed to create Chromium dir: {e}"))
        })?;

        let fetcher = BrowserFetcher::new(
            BrowserFetcherOptions::builder()
                .with_path(&chromium_dir)
                .build()
                .map_err(|e| {
                    AppError::ChromiumError(format!("Error while fetching Chromium: {e}"))
                })?
        );

        let info = fetcher
            .fetch()
            .await
            .map_err(|e| AppError::ChromiumError(
                format!("Failed to fetch Chromium: {e}")
            ))?;


        let mut path_lock = self.chromium_path.lock().await;
        *path_lock = Some(info.executable_path);

        println!("Chromium browser initialized!");
        Ok(())
    }

    pub fn is_chromium_downloaded(&self) -> bool {
        let chromium_dir = Self::get_chromium_cache_dir().ok();
        if let Some(dir) = chromium_dir {
            dir.exists() && dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false)
        } else {
            false
        }
    }

    pub async fn is_initialized(&self) -> bool {
        self.chromium_path.lock().await.is_some()
    } 

    pub async fn get_executable_path(&self) -> AppResult<PathBuf> {
        let path_lock = self.chromium_path.lock().await;
        path_lock.clone().ok_or_else(|| {
            AppError::ChromiumError(format!("Chromium not initialized. Please initialize first."))
        })
    }

    fn get_chromium_cache_dir() -> AppResult<PathBuf> {
        Ok(APP_PATHS.chromium_dir.to_path_buf())
    }
}
