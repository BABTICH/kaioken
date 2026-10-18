use std::path::Path;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::Result;
use crate::safety::path_guard::PathGuard;
use crate::safety::redact::RedactionEngine;
use super::{BoxFuture, Tool, ToolContext, ToolResult};

pub struct GrepTool;

#[derive(Deserialize)]
struct GrepArgs {
    query: String,
    path: Option<String>,
}

impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep_search"
    }

    fn description(&self) -> &'static str {
        "Search workspace files for regex matches. Respects workspace boundaries and ignores binary files."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Regex pattern or exact text to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Optional relative path to subdirectory or specific file"
                }
            },
            "required": ["query"]
        })
    }

    fn execute<'a>(&'a self, args: &'a str, ctx: &'a ToolContext) -> BoxFuture<'a, Result<ToolResult>> {
        let repo_root = ctx.repo_root.clone();
        let audit_logger = ctx.audit_logger.clone();
        let args_str = args.to_string();
        Box::pin(async move {
            let parsed: GrepArgs = match serde_json::from_str(&args_str) {
                Ok(a) => a,
                Err(e) => return Ok(ToolResult::error(format!("Invalid arguments for grep_search: {}", e))),
            };

            let search_dir = match &parsed.path {
                Some(sub) => match PathGuard::validate_path(Path::new(sub), &repo_root, false) {
                    Ok(p) => p,
                    Err(e) => return Ok(ToolResult::error(format!("Invalid search path: {}", e))),
                },
                None => repo_root.clone(),
            };

            let re = match Regex::new(&parsed.query) {
                Ok(r) => r,
                Err(e) => return Ok(ToolResult::error(format!("Invalid regex query '{}': {}", parsed.query, e))),
            };

            let mut matches = Vec::new();
            Self::walk_and_grep(&search_dir, &repo_root, &re, &mut matches, 100);

            let _ = audit_logger.log("agent", "grep_search", &parsed.query, true, None);

            if matches.is_empty() {
                Ok(ToolResult::success(format!("No matches found for '{}'.", parsed.query)))
            } else {
                let count = matches.len();
                let output = matches.join("\n");
                let notice = if count >= 100 { "\n[Output capped at 100 matches]" } else { "" };
                Ok(ToolResult::success(format!("Found {} matches:\n{}{}", count, output, notice)))
            }
        })
    }
}

impl GrepTool {
    fn walk_and_grep(current: &Path, root: &Path, re: &Regex, matches: &mut Vec<String>, limit: usize) {
        if matches.len() >= limit {
            return;
        }

        if current.is_file() {
            if RedactionEngine::is_sensitive_file(current) {
                return;
            }
            if let Ok(content) = std::fs::read_to_string(current) {
                let rel = current.strip_prefix(root).unwrap_or(current).to_string_lossy().replace('\\', "/");
                for (idx, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        matches.push(format!("{}:{}: {}", rel, idx + 1, line.trim()));
                        if matches.len() >= limit {
                            return;
                        }
                    }
                }
            }
        } else if current.is_dir() {
            let name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" || name == "node_modules" || name == ".agent" {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(current) {
                for entry in entries.flatten() {
                    Self::walk_and_grep(&entry.path(), root, re, matches, limit);
                }
            }
        }
    }
}
