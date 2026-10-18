use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::Result;
use super::{BoxFuture, Tool, ToolContext, ToolResult};

pub struct BashTool;

#[derive(Deserialize)]
struct BashArgs {
    command: String,
}

impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn description(&self) -> &'static str {
        "Execute a safe terminal or shell command within the workspace. Dangerous commands are blocked by policy."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The exact shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    fn execute<'a>(&'a self, args: &'a str, ctx: &'a ToolContext) -> BoxFuture<'a, Result<ToolResult>> {
        let repo_root = ctx.repo_root.clone();
        let command_guard = ctx.command_guard.clone();
        let audit_logger = ctx.audit_logger.clone();
        let args_str = args.to_string();
        Box::pin(async move {
            let parsed: BashArgs = match serde_json::from_str(&args_str) {
                Ok(a) => a,
                Err(e) => return Ok(ToolResult::error(format!("Invalid arguments for bash: {}", e))),
            };

            // 1. Blocklist inspection
            if let Some(reason) = command_guard.is_blocked(&parsed.command) {
                let _ = audit_logger.log("agent", "bash", &parsed.command, false, Some(&reason));
                return Ok(ToolResult::error(format!("Command execution blocked by safety policy: {}", reason)));
            }

            // 2. Execute command via CommandGuard with 120s ceiling
            let log_dir = repo_root.join(".agent").join("logs");
            let exec_res = match command_guard.execute(
                &parsed.command,
                &repo_root,
                Some(120),
                Some(&log_dir),
            ).await {
                Ok(res) => res,
                Err(e) => {
                    let _ = audit_logger.log("agent", "bash", &parsed.command, false, Some(&e.to_string()));
                    return Ok(ToolResult::error(format!("Command error: {}", e)));
                }
            };

            let _ = audit_logger.log(
                "agent",
                "bash",
                &parsed.command,
                true,
                Some(&format!("Exit code: {}", exec_res.exit_code)),
            );

            let mut out = String::new();
            if !exec_res.stdout.is_empty() {
                out.push_str(&exec_res.stdout);
            }
            if !exec_res.stderr.is_empty() {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&format!("STDERR:\n{}", exec_res.stderr));
            }
            if out.is_empty() {
                out = format!("Command exited with status code: {}", exec_res.exit_code);
            }

            let mut tool_res = if exec_res.exit_code == 0 {
                ToolResult::success(out)
            } else {
                ToolResult::error(format!("Exit code {}:\n{}", exec_res.exit_code, out))
            };
            tool_res.truncated = exec_res.truncated;

            Ok(tool_res)
        })
    }
}
