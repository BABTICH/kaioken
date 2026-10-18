use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Markdown,
    Json,
    Toml,
    Yaml,
    Html,
    Css,
    Shell,
    Unknown,
}

impl Language {
    pub fn detect_from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts") | Some("tsx") => Language::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Language::JavaScript,
            Some("py") => Language::Python,
            Some("md") | Some("markdown") => Language::Markdown,
            Some("json") => Language::Json,
            Some("toml") => Language::Toml,
            Some("yaml") | Some("yml") => Language::Yaml,
            Some("html") | Some("htm") => Language::Html,
            Some("css") | Some("scss") | Some("sass") => Language::Css,
            Some("sh") | Some("bash") | Some("ps1") => Language::Shell,
            _ => Language::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Python => "python",
            Language::Markdown => "markdown",
            Language::Json => "json",
            Language::Toml => "toml",
            Language::Yaml => "yaml",
            Language::Html => "html",
            Language::Css => "css",
            Language::Shell => "shell",
            Language::Unknown => "unknown",
        }
    }

    pub fn is_ast_supported(&self) -> bool {
        matches!(
            self,
            Language::Rust | Language::TypeScript | Language::JavaScript | Language::Python
        )
    }
}
