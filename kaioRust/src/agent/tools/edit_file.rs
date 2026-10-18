use std::path::Path;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::Result;
use crate::safety::path_guard::PathGuard;
use super::{BoxFuture, Tool, ToolContext, ToolResult};

pub struct EditFileTool;

#[derive(Deserialize)]
struct EditFileArgs {
    path: String,
    target_content: String,
    replacement_content: String,
}

impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Perform exact search-and-replace modification on a file in the workspace. target_content must match uniquely."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to file within workspace"
                },
                "target_content": {
                    "type": "string",
                    "description": "The exact character sequence in the file to be replaced"
                },
                "replacement_content": {
                    "type": "string",
                    "description": "The replacement content to substitute in place of target_content"
                }
            },
            "required": ["path", "target_content", "replacement_content"]
        })
    }

    fn execute<'a>(&'a self, args: &'a str, ctx: &'a ToolContext) -> BoxFuture<'a, Result<ToolResult>> {
        let repo_root = ctx.repo_root.clone();
        let audit_logger = ctx.audit_logger.clone();
        let args_str = args.to_string();
        Box::pin(async move {
            let parsed: EditFileArgs = match serde_json::from_str(&args_str) {
                Ok(a) => a,
                Err(e) => return Ok(ToolResult::error(format!("Invalid arguments for edit_file: {}", e))),
            };

            let validated_path = match PathGuard::validate_path(Path::new(&parsed.path), &repo_root, true) {
                Ok(p) => p,
                Err(e) => {
                    let _ = audit_logger.log("agent", "edit_file", &parsed.path, false, Some(&e.to_string()));
                    return Ok(ToolResult::error(format!("Edit blocked by path guard: {}", e)));
                }
            };

            if !validated_path.exists() {
                let _ = audit_logger.log("agent", "edit_file", &parsed.path, false, Some("File not found"));
                return Ok(ToolResult::error(format!("File '{}' does not exist. Use create_file to create new files.", parsed.path)));
            }

            let existing_content = match std::fs::read_to_string(&validated_path) {
                Ok(c) => c,
                Err(e) => {
                    let _ = audit_logger.log("agent", "edit_file", &parsed.path, false, Some(&e.to_string()));
                    return Ok(ToolResult::error(format!("Failed to read file '{}': {}", parsed.path, e)));
                }
            };

            // Count occurrences of target_content
            let matches_count = existing_content.matches(&parsed.target_content).count();

            if matches_count == 0 {
                let _ = audit_logger.log("agent", "edit_file", &parsed.path, false, Some("target_content not found"));
                return Ok(ToolResult::error(format!(
                    "target_content was not found in '{}'. Use read_file to inspect current contents and ensure exact whitespace matching.",
                    parsed.path
                )));
            }

            if matches_count > 1 {
                let _ = audit_logger.log("agent", "edit_file", &parsed.path, false, Some("target_content matches multiple times"));
                return Ok(ToolResult::error(format!(
                    "target_content matched {} times in '{}'. Please provide a larger unique block of surrounding context lines.",
                    matches_count, parsed.path
                )));
            }

            // Replace single exact occurrence
            let new_content = existing_content.replacen(&parsed.target_content, &parsed.replacement_content, 1);

            if let Err(e) = std::fs::write(&validated_path, new_content) {
                let _ = audit_logger.log("agent", "edit_file", &parsed.path, false, Some(&e.to_string()));
                return Ok(ToolResult::error(format!("Failed to write updated file '{}': {}", parsed.path, e)));
            }

            let _ = audit_logger.log("agent", "edit_file", &parsed.path, true, None);
            Ok(ToolResult::success(format!("Successfully applied search-and-replace edit to '{}'.", parsed.path)))
        })
    }
}
