pub mod bash;
pub mod create_file;
pub mod edit_file;
pub mod glob;
pub mod grep;
pub mod read_file;

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use serde_json::Value;
use crate::error::Result;
use crate::safety::audit::AuditLogger;
use crate::safety::command_guard::CommandGuard;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Clone)]
pub struct ToolContext {
    pub repo_root: PathBuf,
    pub command_guard: std::sync::Arc<CommandGuard>,
    pub audit_logger: AuditLogger,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub output: String,
    pub success: bool,
    pub truncated: bool,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            success: true,
            truncated: false,
        }
    }

    pub fn error(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            success: false,
            truncated: false,
        }
    }
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> Value;
    fn execute<'a>(&'a self, args: &'a str, ctx: &'a ToolContext) -> BoxFuture<'a, Result<ToolResult>>;
}

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Creates standard default registry populated with all core agent tools.
    pub fn default_registry() -> Self {
        let mut reg = Self::new();
        reg.register(Box::new(read_file::ReadFileTool));
        reg.register(Box::new(edit_file::EditFileTool));
        reg.register(Box::new(create_file::CreateFileTool));
        reg.register(Box::new(bash::BashTool));
        reg.register(Box::new(grep::GrepTool));
        reg.register(Box::new(glob::GlobTool));
        reg
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    pub fn find(&self, name: &str) -> Option<&(dyn Tool + 'static)> {
        self.tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }

    pub fn list(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}
