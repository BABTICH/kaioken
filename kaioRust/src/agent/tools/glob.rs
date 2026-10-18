use std::path::Path;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::Result;
use crate::safety::redact::RedactionEngine;
use super::{BoxFuture, Tool, ToolContext, ToolResult};

pub struct GlobTool;

#[derive(Deserialize)]
struct GlobArgs {
    pattern: String,
}

impl Tool for GlobTool {
    fn name(&self) -> &'static str {
        "glob_search"
    }

    fn description(&self) -> &'static str {
        "Discover files matching a path pattern or keyword across the workspace (capped at 200 matches)."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Pattern or filename substring to match (e.g. '*.rs' or 'test')"
                }
            },
            "required": ["pattern"]
        })
    }

    fn execute<'a>(&'a self, args: &'a str, ctx: &'a ToolContext) -> BoxFuture<'a, Result<ToolResult>> {
        let repo_root = ctx.repo_root.clone();
        let audit_logger = ctx.audit_logger.clone();
        let args_str = args.to_string();
        Box::pin(async move {
            let parsed: GlobArgs = match serde_json::from_str(&args_str) {
                Ok(a) => a,
                Err(e) => return Ok(ToolResult::error(format!("Invalid arguments for glob_search: {}", e))),
            };

            let clean_pattern = parsed.pattern.trim_start_matches('*').trim_start_matches('.');
            let mut matched_paths = Vec::new();

            Self::walk_and_match(&repo_root, &repo_root, clean_pattern, &mut matched_paths, 200);

            let _ = audit_logger.log("agent", "glob_search", &parsed.pattern, true, None);

            if matched_paths.is_empty() {
                Ok(ToolResult::success(format!("No files matched pattern '{}'.", parsed.pattern)))
            } else {
                let count = matched_paths.len();
                let output = matched_paths.join("\n");
                let notice = if count >= 200 { "\n[Output capped at 200 matches]" } else { "" };
                Ok(ToolResult::success(format!("Matched {} files:\n{}{}", count, output, notice)))
            }
        })
    }
}

impl GlobTool {
    fn walk_and_match(current: &Path, root: &Path, pattern: &str, matches: &mut Vec<String>, limit: usize) {
        if matches.len() >= limit {
            return;
        }

        if current.is_file() {
            if RedactionEngine::is_sensitive_file(current) {
                return;
            }
            let rel = current.strip_prefix(root).unwrap_or(current).to_string_lossy().replace('\\', "/");
            if rel.contains(pattern) || rel.ends_with(pattern) {
                matches.push(rel);
            }
        } else if current.is_dir() {
            let name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" || name == "node_modules" || name == ".agent" {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(current) {
                for entry in entries.flatten() {
                    Self::walk_and_match(&entry.path(), root, pattern, matches, limit);
                }
            }
        }
    }
}
