use crate::errors::{AppError, AppResult};
use git2::Repository;
use ignore::WalkBuilder;
use mime;
use mime_guess;
use std::fs;
use std::path::Path;

pub async fn convert_repo_to_md(url: &str) -> AppResult<String> {
    let repo_name = extract_repo_name(url)?;
    let clone_path = std::env::temp_dir().join(format!("gitclone{}", repo_name));
    println!(
        "Cloning path: {}",
        clone_path.to_str().unwrap_or_else(|| "Error")
    );
    if clone_path.exists() {
        fs::remove_dir_all(&clone_path).map_err(|e| AppError::IoError(e))?;
    }
    let repo = Repository::clone(url, &clone_path)
        .map_err(|e| AppError::GitError(format!("Failed to clone repository: {}", e)))?;
    let markdown_content = process_repository(&clone_path, &repo_name).await?;
    Ok(markdown_content)
}

pub fn extract_repo_name(url: &str) -> AppResult<String> {
    let url = if url.starts_with("http") {
        url.to_string()
    } else if url.contains(':') {
        // SSH format: git@github.com:user/repo.git
        format!("https://{}", url.split(':').nth(1).unwrap_or(url))
    } else {
        return Err(AppError::GitError(
            "Invalid repository URL format".to_string(),
        ));
    };

    let name = url
        .split('/')
        .last()
        .unwrap_or("unknown")
        .replace(".git", "");

    println!("Extracted repo name: {}", name);

    Ok(name)
}

async fn process_repository(repo_path: &Path, repo_name: &str) -> AppResult<String> {
    let mut markdown_sections = Vec::new();

    markdown_sections.push(format!("# Repository: {}\n", repo_name));

    if let Ok(readme_content) = find_and_read_readme(repo_path) {
        markdown_sections.push("## README\n".to_string());
        markdown_sections.push(readme_content);
        markdown_sections.push("\n---\n".to_string());
    }

    markdown_sections.push("## Source Files\n".to_string());

    // respect .gitignore
    let walker = WalkBuilder::new(repo_path)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .build();

    for entry in walker {
        let entry = entry.map_err(|e| {
            AppError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Walk error: {}", e),
            ))
        })?;

        let path = entry.path();

        if path.is_file() && is_text_file(path) {
            if let Ok(relative_path) = path.strip_prefix(repo_path) {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        markdown_sections.push(format!("\n### {}\n", relative_path.display()));
                        markdown_sections.push("```".to_string());

                        // language hint based on file extension
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            markdown_sections.push(ext.to_string());
                        }
                        markdown_sections.push("\n".to_string());
                        markdown_sections.push(content);
                        markdown_sections.push("\n```\n".to_string());
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not read file {}: {}",
                            relative_path.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(markdown_sections.join(""))
}

fn find_and_read_readme(repo_path: &Path) -> Result<String, std::io::Error> {
    let readme_names = [
        "README.md",
        "README.txt",
        "README",
        "readme.md",
        "readme.txt",
        "readme",
    ];

    for name in &readme_names {
        let readme_path = repo_path.join(name);
        if readme_path.exists() {
            return fs::read_to_string(readme_path);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No README found",
    ))
}

fn is_text_file(path: &Path) -> bool {
    let mime_type = mime_guess::from_path(path).first();

    if let Some(mime_type) = mime_type {
        if mime_type.type_() == mime::TEXT {
            return true;
        }

        if mime_type.type_() == mime::APPLICATION {
            match mime_type.subtype().as_str() {
                "json" | "xml" | "javascript" | "x-javascript" | "x-sh" | "x-shellscript"
                | "x-yaml" | "yaml" | "toml" | "x-toml" => return true,
                _ => {}
            }
        }
    }

    // files without extensions
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        let lower_name = file_name.to_lowercase();

        if matches!(
            lower_name.as_str(),
            "dockerfile"
                | "makefile"
                | "rakefile"
                | "gemfile"
                | "procfile"
                | "license"
                | "changelog"
                | "authors"
                | "contributors"
                | "copying"
                | "install"
                | "news"
                | "todo"
        ) {
            return true;
        }
    }

    false
}
