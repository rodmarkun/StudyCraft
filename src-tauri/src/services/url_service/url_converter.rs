use crate::constants;
use crate::errors::{AppError, AppResult};
use html2md::parse_html;
use regex;
use reqwest;
use scraper::{Html, Selector};
use std::time::Duration;
use url::Url;

fn create_http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; StudyCraftScraper/1.0;)")
        .timeout(Duration::from_secs(30))
        .build()
}

async fn check_robots_txt(client: &reqwest::Client, url: &Url) -> AppResult<bool> {
    let robots_url = format!(
        "{}://{}/robots.txt",
        url.scheme(),
        url.host_str().unwrap_or("")
    );

    match client.get(&robots_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let robots_content = response.text().await.map_err(|e| {
                    AppError::UrlError(format!("Error when reading robots.txt: {}", e))
                })?;
                let is_allowed = !is_path_disallowed(&robots_content, url.path());
                Ok(is_allowed)
            } else {
                Ok(true)
            }
        }
        Err(_) => {
            Ok(true) // If we cannot fetch it, we asume it is allowed
        }
    }
}

fn is_path_disallowed(robots_content: &str, path: &str) -> bool {
    let mut current_user_agent = false;
    let mut disallowed_paths = Vec::new();

    for line in robots_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.to_lowercase().starts_with("user-agent:") {
            let agent = line.split(':').nth(1).unwrap_or("").trim();
            current_user_agent = agent == "*" || agent.to_lowercase().contains("yourbot");
        } else if current_user_agent && line.to_lowercase().starts_with("disallow:") {
            if let Some(disallow_path) = line.split(':').nth(1) {
                let disallow_path = disallow_path.trim();
                if !disallow_path.is_empty() {
                    disallowed_paths.push(disallow_path);
                }
            }
        }
    }

    for disallow_path in disallowed_paths {
        if disallow_path == "/" {
            return true; // entire site disallowed
        }
        if path.starts_with(disallow_path) {
            return true;
        }
    }

    false
}

pub async fn convert_url_to_md(url: &str) -> AppResult<(String, String, String)> {
    let parsed_url =
        Url::parse(url).map_err(|e| AppError::UrlError(format!("Invalid URL: {}", e)))?;

    let client = create_http_client()
        .map_err(|e| AppError::UrlError(format!("Error when creating HTTP client")))?;
    if !check_robots_txt(&client, &parsed_url).await? {
        return Err(AppError::UrlError(format!(
            "Access to {} is disallowed by robots.txt",
            url
        )));
    }

    let material_name = parsed_url
        .path_segments()
        .and_then(|segments| segments.filter(|s| !s.is_empty()).last())
        .unwrap_or_else(|| parsed_url.host_str().unwrap_or("unknown"))
        .to_string();

    let response = client
        .get(url)
        .header(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.5")
        .send()
        .await
        .map_err(|e| AppError::UrlError(format!("Failed to fetch URL: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::UrlError(format!(
            "HTTP error {}: {}",
            response.status(),
            url
        )));
    }

    let html_content = response
        .text()
        .await
        .map_err(|e| AppError::UrlError(format!("Failed to read response: {}", e)))?;

    let doc = Html::parse_document(&html_content);
    let title_selector = Selector::parse("title").unwrap();
    let title = doc
        .select(&title_selector)
        .next()
        .map(|element| element.text().collect::<String>())
        .unwrap_or_else(|| "Untitled".to_string());

    let clean_html = extract_main_content(&html_content);
    let markdown_content = parse_html(&clean_html);

    Ok((title, material_name, markdown_content))
}

fn extract_main_content(html: &str) -> String {
    let doc = Html::parse_document(html);

    for selector_str in &constants::CONTENT_SELECTORS {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = doc.select(&selector).next() {
                return element.html();
            }
        }
    }

    // Fallback returning body content without scripts/styles
    let body_selector = Selector::parse("body").unwrap();
    if let Some(body) = doc.select(&body_selector).next() {
        let mut content = body.html();
        // Remove scripts and styles
        content = regex::Regex::new(r"(?s)<script[^>]*>.*?</script>")
            .unwrap()
            .replace_all(&content, "")
            .to_string();
        content = regex::Regex::new(r"(?s)<style[^>]*>.*?</style>")
            .unwrap()
            .replace_all(&content, "")
            .to_string();
        return content;
    }

    html.to_string()
}

pub fn is_git_url(url: &str) -> bool {
    if let Ok(parsed_url) = Url::parse(url) {
        let host = parsed_url.host_str().unwrap_or("");
        let is_git_host = host.contains("github.com")
            || host.contains("gitlab.com")
            || host.contains("bitbucket.org")
            || host.contains("dev.azure.com");

        let is_git_protocol = parsed_url.scheme() == "git"
            || url.ends_with(".git")
            || (is_git_host && !url.contains("/raw/"));

        return is_git_protocol;
    }
    false
}
