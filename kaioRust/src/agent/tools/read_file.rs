use std::path::Path;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::Result;
use crate::safety::path_guard::PathGuard;
use crate::safety::redact::RedactionEngine;
use super::{BoxFuture, Tool, ToolContext, ToolResult};

pub struct ReadFileTool;

#[derive(Deserialize)]
struct ReadFileArgs {
    path: String,
    start_line: Option<usize>,
    end_line: Option<usize>,
}

impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read contents of a file within the workspace, optionally specifying start_line and end_line (1-indexed)."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to file within workspace"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Optional starting line number (1-indexed, inclusive)"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Optional ending line number (1-indexed, inclusive)"
                }
            },
            "required": ["path"]
        })
    }

    fn execute<'a>(&'a self, args: &'a str, ctx: &'a ToolContext) -> BoxFuture<'a, Result<ToolResult>> {
        let repo_root = ctx.repo_root.clone();
        let audit_logger = ctx.audit_logger.clone();
        let args_str = args.to_string();
        Box::pin(async move {
            let parsed: ReadFileArgs = match serde_json::from_str(&args_str) {
                Ok(a) => a,
                Err(e) => return Ok(ToolResult::error(format!("Invalid arguments for read_file: {}", e))),
            };

            let validated_path = match PathGuard::validate_path(Path::new(&parsed.path), &repo_root, false) {
                Ok(p) => p,
                Err(e) => {
                    let _ = audit_logger.log("agent", "read_file", &parsed.path, false, Some(&e.to_string()));
                    return Ok(ToolResult::error(format!("Access blocked by path guard: {}", e)));
                }
            };

            if !validated_path.exists() {
                let _ = audit_logger.log("agent", "read_file", &parsed.path, false, Some("File not found"));
                return Ok(ToolResult::error(format!("File '{}' does not exist.", parsed.path)));
            }

            let raw_content = match std::fs::read_to_string(&validated_path) {
                Ok(c) => c,
                Err(e) => {
                    let _ = audit_logger.log("agent", "read_file", &parsed.path, false, Some(&e.to_string()));
                    return Ok(ToolResult::error(format!("Failed to read file '{}': {}", parsed.path, e)));
                }
            };

            let redactor = RedactionEngine::new();
            let sanitized = redactor.redact(&raw_content);
            let lines: Vec<&str> = sanitized.lines().collect();
            let total_lines = lines.len();

            let (start, end) = match (parsed.start_line, parsed.end_line) {
                (Some(s), Some(e)) => (s.max(1), e.min(total_lines)),
                (Some(s), None) => (s.max(1), total_lines),
                (None, Some(e)) => (1, e.min(total_lines)),
                (None, None) => (1, total_lines),
            };

            let output = if start > total_lines || start > end {
                format!("File '{}' has {} lines. Requested range ({}-{}) is empty.", parsed.path, total_lines, start, end)
            } else {
                let mut chunk = String::new();
                for (idx, line) in lines[start - 1..end].iter().enumerate() {
                    chunk.push_str(&format!("{:4} | {}\n", start + idx, line));
                }
                chunk
            };

            let _ = audit_logger.log("agent", "read_file", &parsed.path, true, None);
            Ok(ToolResult::success(output))
        })
    }
}
