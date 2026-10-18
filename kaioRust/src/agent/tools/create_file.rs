use std::path::Path;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::Result;
use crate::safety::path_guard::PathGuard;
use super::{BoxFuture, Tool, ToolContext, ToolResult};

pub struct CreateFileTool;

#[derive(Deserialize)]
struct CreateFileArgs {
    path: String,
    content: String,
    overwrite: Option<bool>,
}

impl Tool for CreateFileTool {
    fn name(&self) -> &'static str {
        "create_file"
    }

    fn description(&self) -> &'static str {
        "Create a new file in the workspace with the provided content. Overwriting existing files requires overwrite: true."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to file to create"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write into the file"
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Set to true to overwrite existing file (default: false)"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn execute<'a>(&'a self, args: &'a str, ctx: &'a ToolContext) -> BoxFuture<'a, Result<ToolResult>> {
        let repo_root = ctx.repo_root.clone();
        let audit_logger = ctx.audit_logger.clone();
        let args_str = args.to_string();
        Box::pin(async move {
            let parsed: CreateFileArgs = match serde_json::from_str(&args_str) {
                Ok(a) => a,
                Err(e) => return Ok(ToolResult::error(format!("Invalid arguments for create_file: {}", e))),
            };

            let validated_path = match PathGuard::validate_path(Path::new(&parsed.path), &repo_root, true) {
                Ok(p) => p,
                Err(e) => {
                    let _ = audit_logger.log("agent", "create_file", &parsed.path, false, Some(&e.to_string()));
                    return Ok(ToolResult::error(format!("Creation blocked by path guard: {}", e)));
                }
            };

            let is_dry_run = if let Some(policy) = &ctx.safety_policy {
                let details = format!("Creating {} bytes", parsed.content.len());
                match policy.check_action("create_file", &parsed.path, &details)? {
                    crate::safety::PermissionDecision::Allowed => false,
                    crate::safety::PermissionDecision::SimulatedDryRun => true,
                    crate::safety::PermissionDecision::Denied(r) => {
                        return Ok(ToolResult::error(format!("Creation blocked by safety policy: {}", r)));
                    }
                }
            } else {
                false
            };

            if validated_path.exists() && parsed.overwrite != Some(true) {
                let _ = audit_logger.log("agent", "create_file", &parsed.path, false, Some("File already exists without overwrite flag"));
                return Ok(ToolResult::error(format!(
                    "File '{}' already exists. Specify 'overwrite: true' or use 'edit_file' instead.",
                    parsed.path
                )));
            }

            if is_dry_run {
                return Ok(ToolResult::success(format!(
                    "[DRY-RUN SIMULATION] Would create file '{}' with {} bytes (no changes written to disk).",
                    parsed.path, parsed.content.len()
                )));
            }

            if let Some(parent) = validated_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    let _ = audit_logger.log("agent", "create_file", &parsed.path, false, Some(&e.to_string()));
                    return Ok(ToolResult::error(format!("Failed to create parent directories: {}", e)));
                }
            }

            if let Err(e) = std::fs::write(&validated_path, &parsed.content) {
                let _ = audit_logger.log("agent", "create_file", &parsed.path, false, Some(&e.to_string()));
                return Ok(ToolResult::error(format!("Failed to write file '{}': {}", parsed.path, e)));
            }

            let _ = audit_logger.log("agent", "create_file", &parsed.path, true, None);
            Ok(ToolResult::success(format!("Successfully created file '{}'.", parsed.path)))
        })
    }
}
