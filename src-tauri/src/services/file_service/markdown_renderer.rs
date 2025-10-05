use crate::constants;
use crate::errors::{AppError, AppResult};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::{CaptureScreenshotParams, CaptureScreenshotFormat};
use chromiumoxide::cdp::browser_protocol::page::Viewport;
use futures::StreamExt;
use base64;
use pulldown_cmark::{html, Options, Parser};
use std::{fs, path::Path, path::PathBuf};

pub struct MarkdownRenderer {
    temp_dir: PathBuf,
    browser: Browser,
    _handler_task: tokio::task::JoinHandle<()>
}

impl MarkdownRenderer {
    pub async fn new(chromium_path: &Path) -> AppResult<Self> {
        let temp_dir = std::env::temp_dir()
            .join(format!("markdown_renderer_{}", std::process::id()));
        
        fs::create_dir_all(&temp_dir).map_err(|e| AppError::IoError(e))?;

        // Download Chromium if not already present
        let (browser, handler_task) = Self::setup_browser(chromium_path).await?;

        Ok(Self { 
            temp_dir, 
            browser,
            _handler_task: handler_task,
        })
    }

    async fn setup_browser(chromium_path: &Path) -> AppResult<(Browser, tokio::task::JoinHandle<()>)> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .chrome_executable(chromium_path.to_path_buf())
                .window_size(
                    constants::COVER_IMAGE_DIMENSIONS.0,
                    constants::COVER_IMAGE_DIMENSIONS.1
                )
                .build()
                .map_err(|e| AppError::MarkdownRendererError(e.to_string()))?
        )
        .await
        .map_err(|e| AppError::MarkdownRendererError(
            format!("Failed to launch browser: {}", e)
        ))?;

        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    let error_msg = format!("{:?}", e);
                    if !error_msg.contains("Serde") {
                        eprintln!("Browser handler error: {:?}", e);
                    }
                }
            }
        });

        Ok((browser, handler_task))
    }

    fn get_chromium_cache_dir() -> AppResult<PathBuf> {
        let cache_dir = if cfg!(target_os = "macos") {
            dirs::cache_dir()
                .ok_or_else(|| AppError::MarkdownRendererError(
                    "Could not find cache directory".to_string()
                ))?
                .join("com.yourapp.studycraft")
        } else if cfg!(target_os = "linux") {
            dirs::cache_dir()
                .ok_or_else(|| AppError::MarkdownRendererError(
                    "Could not find cache directory".to_string()
                ))?
                .join("studycraft")
        } else {
            // Windows
            dirs::data_local_dir()
                .ok_or_else(|| AppError::MarkdownRendererError(
                    "Could not find local data directory".to_string()
                ))?
                .join("studycraft")
        };

        Ok(cache_dir.join("chromium"))
    }

    pub fn cleanup(&self) -> AppResult<()> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir).map_err(AppError::IoError)?;
        }
        Ok(())
    }

    pub async fn render_markdown_to_screenshot(
        &self,
        markdown_content: &str,
        output_path: &Path,
        width: u32,
        height: u32,
    ) -> AppResult<()> {
        let html_content = self.markdown_to_html(markdown_content)?;
        
        let html_file_path = self.temp_dir
            .join(format!("rendered_{}.html", std::process::id()));
        
        fs::write(&html_file_path, html_content)
            .map_err(|e| AppError::IoError(e))?;

        let page = self.browser
            .new_page(&format!("file://{}", html_file_path.to_string_lossy()))
            .await
            .map_err(|e| AppError::MarkdownRendererError(e.to_string()))?;

        // Set device metrics to control viewport size - same as URL screenshots
        let metrics = SetDeviceMetricsOverrideParams::builder()
            .width(width as i64)
            .height(height as i64)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
            .map_err(|e| AppError::MarkdownRendererError(e.to_string()))?;

        page.execute(metrics)
            .await
            .map_err(|e| AppError::MarkdownRendererError(e.to_string()))?;

        // Wait for page to render fully
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

        // Take screenshot with full dimensions (no 0.93 scaling)
        let screenshot = page
            .screenshot(
                chromiumoxide::page::ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .clip(Viewport {
                        x: 0.0,
                        y: 0.0,
                        width: width as f64,  // Changed from width * 0.93
                        height: height as f64,  // Changed from height * 0.93
                        scale: 1.0,
                    })
                    .build()
            )
            .await
            .map_err(|e| AppError::MarkdownRendererError(e.to_string()))?;

        if !output_path.exists() {
            fs::write(output_path, screenshot)
                .map_err(|e| AppError::IoError(e))?;
        }

        Ok(())
    }

    pub async fn url_to_screenshot(
        &self,
        url: &str,
        output_path: &Path,
        width: u32,
        height: u32,
    ) -> AppResult<()> {
        let screenshot_data = self.screenshot_url(url, width, height).await?;

        if !output_path.exists() {
            fs::write(output_path, &screenshot_data).map_err(|e| AppError::IoError(e))?;
        }

        Ok(())
    }

    fn markdown_to_html(&self, markdown: &str) -> AppResult<String> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        let styled_html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rendered Markdown</title>
    <style>
        {}
    </style>
</head>
<body>
    <div class="markdown-body">
        {}
    </div>
</body>
</html>"#,
            self.get_markdown_css(),
            html_output
        );

        Ok(styled_html)
    }

    fn get_markdown_css(&self) -> &'static str {
        // GitHub-like markdown styling
        r#"
        * {
        scrollbar-width: none; /* Firefox */
        -ms-overflow-style: none;  /* Internet Explorer 10+ */
    }
    
    *::-webkit-scrollbar { 
        display: none; /* WebKit */
    }
    
    html, body {
        overflow: hidden; /* Prevent scrolling entirely */
        margin: 0;
        padding: 0;
        width: 100%;
        height: 100vh;
    }
        .markdown-body {
            box-sizing: border-box;
            min-width: 200px;
            max-width: 980px;
            margin: 0 auto;
            padding: 45px;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
            font-size: 16px;
            line-height: 1.5;
            word-wrap: break-word;
        }
        
        .markdown-body h1, .markdown-body h2, .markdown-body h3, .markdown-body h4, .markdown-body h5, .markdown-body h6 {
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }
        
        .markdown-body h1 { font-size: 2em; border-bottom: 1px solid #eaecef; padding-bottom: 10px; }
        .markdown-body h2 { font-size: 1.5em; border-bottom: 1px solid #eaecef; padding-bottom: 10px; }
        .markdown-body h3 { font-size: 1.25em; }
        .markdown-body h4 { font-size: 1em; }
        .markdown-body h5 { font-size: 0.875em; }
        .markdown-body h6 { font-size: 0.85em; color: #6a737d; }
        
        .markdown-body p { margin-bottom: 16px; }
        
        .markdown-body blockquote {
            padding: 0 1em;
            color: #6a737d;
            border-left: 0.25em solid #dfe2e5;
            margin: 0 0 16px 0;
        }
        
        .markdown-body ul, .markdown-body ol {
            padding-left: 2em;
            margin-bottom: 16px;
        }
        
        .markdown-body code {
            padding: 0.2em 0.4em;
            margin: 0;
            font-size: 85%;
            background-color: rgba(27,31,35,0.05);
            border-radius: 3px;
            font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
        }
        
        .markdown-body pre {
            padding: 16px;
            overflow: auto;
            font-size: 85%;
            line-height: 1.45;
            background-color: #f6f8fa;
            border-radius: 6px;
            margin-bottom: 16px;
        }
        
        .markdown-body pre code {
            background-color: transparent;
            border: 0;
            display: inline;
            line-height: inherit;
            margin: 0;
            max-width: auto;
            overflow: visible;
            padding: 0;
            white-space: pre;
            word-wrap: normal;
        }
        
        .markdown-body table {
            border-spacing: 0;
            border-collapse: collapse;
            margin-bottom: 16px;
        }
        
        .markdown-body table th, .markdown-body table td {
            padding: 6px 13px;
            border: 1px solid #dfe2e5;
        }
        
        .markdown-body table th {
            background-color: #f6f8fa;
            font-weight: 600;
        }
        
        .markdown-body table tr:nth-child(2n) {
            background-color: #f6f8fa;
        }
        
        .markdown-body img {
            max-width: 100%;
            height: auto;
        }
        
        .markdown-body a {
            color: #0366d6;
            text-decoration: none;
        }
        
        .markdown-body a:hover {
            text-decoration: underline;
        }
        "#
    }

    async fn screenshot_url(
        &self,
        url: &str,
        width: u32,
        height: u32,
    ) -> AppResult<Vec<u8>> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| AppError::MarkdownRendererError(format!("Failed to create page: {}", e)))?;

        // Set device metrics to control viewport size
        let metrics = SetDeviceMetricsOverrideParams::builder()
            .width(width as i64)
            .height(height as i64)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
            .map_err(|e| AppError::MarkdownRendererError(e.to_string()))?; // Handle the Result here

        page.execute(metrics)
            .await
            .map_err(|e| AppError::MarkdownRendererError(e.to_string()))?;

        // Wait for page to load and render
        page.wait_for_navigation()
            .await
            .map_err(|e| AppError::MarkdownRendererError(format!("Failed to wait for navigation: {}", e)))?;

        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

        // Use Viewport for clipping the screenshot
        let clip_viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: width as f64,
            height: height as f64,
            scale: 1.0,
        };

        let screenshot_params = CaptureScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .quality(100)
            .clip(clip_viewport)
            .build();

        let screenshot_result = page
            .execute(screenshot_params)
            .await
            .map_err(|e| {
                AppError::MarkdownRendererError(format!("Failed to capture screenshot: {}", e))
            })?;

        let screenshot_data = base64::decode(&screenshot_result.data)
            .map_err(|e| AppError::MarkdownRendererError(format!("Failed to decode screenshot: {}", e)))?;

        Ok(screenshot_data)
    }}

impl Drop for MarkdownRenderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

pub async fn url_to_screenshot(url: &str, output_path: &Path, chromium_path: &Path) -> AppResult<()> {
    let renderer = MarkdownRenderer::new(chromium_path).await?;
    let (width, height) = constants::COVER_IMAGE_DIMENSIONS;
    let result = renderer
        .url_to_screenshot(url, output_path, width, height)
        .await;
    renderer.cleanup()?;
    result
}

pub async fn markdown_file_to_screenshot(
    markdown_file_path: &Path,
    output_path: &Path,
    chromium_path: &Path
) -> AppResult<()> {
    let markdown_content =
        fs::read_to_string(markdown_file_path).map_err(|e| AppError::IoError(e))?;
    let (width, height) = constants::COVER_IMAGE_DIMENSIONS;
    let renderer = MarkdownRenderer::new(chromium_path).await?;
    let result = renderer
        .render_markdown_to_screenshot(&markdown_content, output_path, width, height)
        .await;
    renderer.cleanup()?;
    result
}