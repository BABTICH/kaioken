use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::KaiokenError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub project: ProjectConfig,
    pub memory: MemoryConfig,
    pub docs: DocsConfig,
    pub llm: LlmConfig,
    pub permissions: PermissionsConfig,
    pub scanner: ScannerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryConfig {
    pub database: String,
    pub update_on_commit: bool,
    pub daily_scan: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocsConfig {
    pub output: String,
    pub auto_update_changed: bool,
    pub deep_docs_for_high_risk: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub planner_model: String,
    pub coder_model: String,
    pub summarizer_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionMode {
    ReadOnly,
    EditWithApproval,
    AutoEdit,
    SafeBash,
    FullAuto,
}

impl Default for PermissionMode {
    fn default() -> Self {
        PermissionMode::EditWithApproval
    }
}

impl std::str::FromStr for PermissionMode {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read-only" | "readonly" => Ok(PermissionMode::ReadOnly),
            "edit-with-approval" => Ok(PermissionMode::EditWithApproval),
            "auto-edit" => Ok(PermissionMode::AutoEdit),
            "safe-bash" => Ok(PermissionMode::SafeBash),
            "full-auto" => Ok(PermissionMode::FullAuto),
            _ => Err(format!(
                "Unknown permission mode '{}'. Valid modes: read-only, edit-with-approval, auto-edit, safe-bash, full-auto",
                s
            )),
        }
    }
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionMode::ReadOnly => write!(f, "read-only"),
            PermissionMode::EditWithApproval => write!(f, "edit-with-approval"),
            PermissionMode::AutoEdit => write!(f, "auto-edit"),
            PermissionMode::SafeBash => write!(f, "safe-bash"),
            PermissionMode::FullAuto => write!(f, "full-auto"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionsConfig {
    #[serde(default)]
    pub mode: PermissionMode,
    pub auto_edit: bool,
    pub auto_bash: bool,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScannerConfig {
    pub ignore: Vec<String>,
}

impl Config {
    pub const AGENT_DIR: &'static str = ".agent";

    pub fn default_config() -> Self {
        Self {
            project: ProjectConfig {
                name: "my-project".to_string(),
                languages: vec!["rust".to_string(), "typescript".to_string()],
            },
            memory: MemoryConfig {
                database: ".agent/memory.sqlite".to_string(),
                update_on_commit: true,
                daily_scan: true,
            },
            docs: DocsConfig {
                output: ".agent-docs".to_string(),
                auto_update_changed: true,
                deep_docs_for_high_risk: true,
            },
            llm: LlmConfig {
                provider: "openrouter".to_string(),
                planner_model: "anthropic/claude-sonnet-4".to_string(),
                coder_model: "anthropic/claude-sonnet-4".to_string(),
                summarizer_model: "openai/gpt-4o-mini".to_string(),
                max_tokens: 8000,
                temperature: 0.2,
            },
            permissions: PermissionsConfig {
                mode: PermissionMode::EditWithApproval,
                auto_edit: false,
                auto_bash: false,
                allowed_commands: vec![
                    "cargo check".to_string(),
                    "cargo test".to_string(),
                    "npm test".to_string(),
                    "pytest".to_string(),
                ],
                blocked_commands: vec![
                    "rm -rf".to_string(),
                    "git push --force".to_string(),
                    "git reset --hard".to_string(),
                ],
            },
            scanner: ScannerConfig {
                ignore: vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    "dist".to_string(),
                    "build".to_string(),
                    ".git".to_string(),
                    ".agent".to_string(),
                    ".kaio".to_string(),
                    ".vercel".to_string(),
                    ".idea".to_string(),
                    ".vscode".to_string(),
                ],
            },
        }
    }

    pub fn load(repo_root: &Path) -> Result<Self, KaiokenError> {
        let config_path = repo_root.join(".agent").join("config.toml");
        if !config_path.exists() {
            return Err(KaiokenError::ConfigError(format!(
                "Config file not found at {:?}. Run `kaioken memory init` first.",
                config_path
            )));
        }
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| KaiokenError::ConfigError(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    pub fn save(&self, repo_root: &Path) -> Result<(), KaiokenError> {
        let agent_dir = repo_root.join(".agent");
        if !agent_dir.exists() {
            fs::create_dir_all(&agent_dir)?;
        }
        let config_path = agent_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| KaiokenError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
