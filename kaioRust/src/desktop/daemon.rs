use crate::config::Config;
use crate::desktop::protocol::{IpcRequest, IpcResponse};
use crate::doctor::DoctorEngine;
use crate::hook::HookManager;
use crate::memory::MemoryEngine;
use crate::review::{ReviewArgs, ReviewEngine};
use crate::safety::audit::AuditLogger;
use crate::scanner::{FileInventory, FileInventoryOptions};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn resolve_safe_path(root: &Path, rel_path: &str) -> std::result::Result<PathBuf, String> {
    let clean = rel_path.trim_start_matches(['/', '\\']);
    let candidate = root.join(clean);
    if candidate.components().any(|c| c == std::path::Component::ParentDir) {
        return Err("Path traversal forbidden".to_string());
    }
    Ok(candidate)
}

/// Request handler shared by the embedded runtime and the `kaioken watch` daemon.
///
/// Both transports speak the same wire protocol, so keeping one implementation
/// is what stops the desktop from working against a mock and failing against a
/// real daemon (or the reverse).
#[derive(Clone)]
pub struct DaemonService {
    repo_root: PathBuf,
    memory: Arc<Mutex<MemoryEngine>>,
    started_at: Instant,
    mode: String,
    endpoint: String,
}

impl DaemonService {
    pub fn new(
        repo_root: PathBuf,
        memory: Arc<Mutex<MemoryEngine>>,
        mode: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            repo_root,
            memory,
            started_at: Instant::now(),
            mode: mode.into(),
            endpoint: endpoint.into(),
        }
    }

    /// Answer one request. Read-only queries are served; mutations that need a
    /// hosted agent runtime return a precise error instead of pretending.
    pub fn handle(&self, request: IpcRequest) -> IpcResponse {
        let memory = match self.memory.lock() {
            Ok(memory) => memory,
            Err(_) => return IpcResponse::err(-32603, "memory lock poisoned"),
        };

        match request {
            IpcRequest::Ping => IpcResponse::success(json!({
                "pong": true,
                "mode": self.mode,
                "endpoint": self.endpoint,
            })),
            IpcRequest::GetStatus => IpcResponse::success(json!({
                "status": "active",
                "mode": self.mode,
                "endpoint": self.endpoint,
                "repo_root": self.repo_root.display().to_string(),
                "pid": std::process::id(),
                "uptime_secs": self.started_at.elapsed().as_secs(),
                "version": env!("CARGO_PKG_VERSION"),
            })),
            IpcRequest::GetMemoryStats => IpcResponse::success(json!({
                "total_files": memory.count_files().unwrap_or(0),
                "total_symbols": memory.count_symbols().unwrap_or(0),
                "total_edges": memory.count_edges().unwrap_or(0),
                "total_docs": memory.count_docs().unwrap_or(0),
                "total_commits": memory.count_commits().unwrap_or(0),
                "total_failures": memory.count_failures().unwrap_or(0),
            })),
            IpcRequest::GetMemoryGraph { filter, limit } => {
                let needle = filter.unwrap_or_default().to_lowercase();
                let limit = limit.unwrap_or(200).max(1);
                let file_paths: std::collections::HashMap<i64, String> = memory
                    .list_files()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|file| file.id.map(|id| (id, file.path)))
                    .collect();

                let mut symbols = memory.get_all_symbols().unwrap_or_default();
                if !needle.is_empty() {
                    symbols.retain(|symbol| {
                        let path = file_paths
                            .get(&symbol.file_id)
                            .map(String::as_str)
                            .unwrap_or("");
                        symbol.name.to_lowercase().contains(&needle)
                            || path.to_lowercase().contains(&needle)
                    });
                }
                let total = symbols.len();
                symbols.truncate(limit);

                let nodes: Vec<_> = symbols
                    .iter()
                    .enumerate()
                    .filter_map(|(index, symbol)| {
                        let id = symbol.id?;
                        let path = file_paths
                            .get(&symbol.file_id)
                            .cloned()
                            .unwrap_or_else(|| format!("file_id:{}", symbol.file_id));
                        // Golden-angle spiral: stable positions without a layout
                        // pass, so the WebView can render the same graph twice.
                        let angle = index as f32 * 2.399_963;
                        let radius = 90.0 + (index % 6) as f32 * 18.0;
                        Some(json!({
                            "id": id,
                            "name": symbol.name,
                            // The desktop graph filters on the short AST kind;
                            // memory stores the scanner's longer spelling.
                            "kind": match symbol.kind.as_str() {
                                "function" | "method" => "fn",
                                other => other,
                            },
                            "file_path": path,
                            "is_exported": symbol
                                .visibility
                                .as_deref()
                                .map(|visibility| visibility.starts_with("pub"))
                                .unwrap_or(false),
                            "is_test": symbol.name.starts_with("test_")
                                || path.contains("/tests/")
                                || path.starts_with("tests/"),
                            "x": 430.0 + angle.cos() * radius,
                            "y": 240.0 + angle.sin() * radius,
                        }))
                    })
                    .collect();

                let included: std::collections::HashSet<i64> = nodes
                    .iter()
                    .filter_map(|node| node.get("id").and_then(|id| id.as_i64()))
                    .collect();
                let edges: Vec<_> = memory
                    .get_all_edges()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|edge| {
                        let source = edge.source_symbol_id?;
                        let target = edge.target_symbol_id?;
                        if !included.contains(&source) || !included.contains(&target) {
                            return None;
                        }
                        Some(json!({
                            "source_id": source,
                            "target_id": target,
                            "edge_kind": edge.edge_type,
                        }))
                    })
                    .collect();

                IpcResponse::success(json!({
                    "total": total,
                    "nodes": nodes,
                    "edges": edges,
                }))
            }
            IpcRequest::GetReviewDiff { cached: _ } => {
                match ReviewEngine::review(&self.repo_root, &memory, &ReviewArgs::default()) {
                    Ok(report) => {
                        let diff =
                            ReviewEngine::get_diff(&self.repo_root, &ReviewArgs::default())
                                .unwrap_or_default();
                        IpcResponse::success(json!({
                            "diff": diff,
                            "risk_score": (report.risk.score * 100.0).round() as u32,
                            "risk_level": report.risk.level.as_str(),
                            "changed_files": report.changed_files,
                            "breaking_changes": report.breaking_changes,
                            "affected_callers": report.affected_callers,
                            "test_gaps": report.test_gaps,
                            "stale_docs": report.stale_docs,
                            "recommendations": report.recommendations,
                        }))
                    }
                    Err(error) => IpcResponse::err(-32001, error.to_string()),
                }
            }
            IpcRequest::GetAuditLogs { limit } => {
                let logger = AuditLogger::new(Some(&self.repo_root));
                match logger.list_recent(limit) {
                    Ok(entries) => IpcResponse::success(entries),
                    Err(error) => IpcResponse::err(-32002, error.to_string()),
                }
            }
            IpcRequest::GetDocsHealth => {
                let docs = memory.list_docs().unwrap_or_default();
                let fresh = docs.iter().filter(|doc| doc.status == "fresh").count();
                let stale = docs.iter().filter(|doc| doc.status == "stale").count();
                let maybe_stale = docs.iter().filter(|doc| doc.status == "maybe_stale").count();
                IpcResponse::success(json!({
                    "total_docs": docs.len(),
                    "fresh_docs": fresh,
                    "stale_docs": stale,
                    "maybe_stale_docs": maybe_stale,
                }))
            }
            IpcRequest::StartAgentTask { .. } => IpcResponse::err(
                -32601,
                "the watcher daemon does not host agent tasks; run `kaioken task`",
            ),
            IpcRequest::ApproveToolCall { .. } | IpcRequest::DenyToolCall { .. } => IpcResponse::err(
                -32601,
                "no pending tool proposals exist in the watcher daemon",
            ),
            IpcRequest::CancelTask { .. } => {
                IpcResponse::err(-32601, "no agent task is running in the watcher daemon")
            }
            IpcRequest::RegenerateDocs { .. } => IpcResponse::err(
                -32601,
                "document regeneration is not hosted by the watcher daemon; run `kaioken docs generate`",
            ),
            IpcRequest::ExecuteTerminalCommand { command } => {
                #[cfg(windows)]
                let mut cmd = {
                    let mut c = Command::new("powershell");
                    c.arg("-NoProfile").arg("-Command").arg(&command);
                    c
                };
                #[cfg(not(windows))]
                let mut cmd = {
                    let mut c = Command::new("sh");
                    c.arg("-c").arg(&command);
                    c
                };
                cmd.current_dir(&self.repo_root);
                match cmd.output() {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        let exit_code = out.status.code().unwrap_or(-1);
                        IpcResponse::success(json!({
                            "stdout": stdout,
                            "stderr": stderr,
                            "exit_code": exit_code,
                            "success": out.status.success(),
                        }))
                    }
                    Err(e) => IpcResponse::err(-32000, format!("Execution failed: {}", e)),
                }
            }
            IpcRequest::ReadFile { path } => {
                let safe_path = match resolve_safe_path(&self.repo_root, &path) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(-32003, e),
                };
                match std::fs::read_to_string(&safe_path) {
                    Ok(content) => IpcResponse::success(json!({
                        "path": path,
                        "content": content,
                    })),
                    Err(e) => IpcResponse::err(-32004, format!("Failed to read file: {}", e)),
                }
            }
            IpcRequest::SaveFile { path, content } => {
                let safe_path = match resolve_safe_path(&self.repo_root, &path) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(-32003, e),
                };
                if let Some(parent) = safe_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(&safe_path, content) {
                    Ok(()) => IpcResponse::success(json!({
                        "path": path,
                        "saved": true,
                    })),
                    Err(e) => IpcResponse::err(-32005, format!("Failed to write file: {}", e)),
                }
            }
            IpcRequest::ListRepoFiles => {
                let mut files = Vec::new();
                let mut stack = vec![self.repo_root.clone()];
                while let Some(dir) = stack.pop() {
                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let file_name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("");
                            if file_name.starts_with('.')
                                || file_name == "target"
                                || file_name == "node_modules"
                            {
                                continue;
                            }
                            if path.is_dir() {
                                stack.push(path);
                            } else if path.is_file() {
                                if let Ok(rel) = path.strip_prefix(&self.repo_root) {
                                    files.push(rel.to_string_lossy().replace('\\', "/"));
                                }
                            }
                        }
                    }
                }
                files.sort();
                IpcResponse::success(json!({ "files": files }))
            }
            IpcRequest::BrowserFetch {
                url,
                method,
                headers,
                body,
            } => {
                let start = Instant::now();
                let mut cmd = Command::new("curl");
                cmd.arg("-s")
                    .arg("-S")
                    .arg("-i")
                    .arg("-X")
                    .arg(&method)
                    .arg(&url)
                    .arg("--max-time")
                    .arg("15");
                for (k, v) in &headers {
                    cmd.arg("-H").arg(format!("{}: {}", k, v));
                }
                if let Some(ref b) = body {
                    cmd.arg("-d").arg(b);
                }
                match cmd.output() {
                    Ok(out) => {
                        let latency_ms = start.elapsed().as_millis() as u64;
                        let raw = String::from_utf8_lossy(&out.stdout).to_string();
                        let mut status = 200u16;
                        let mut resp_headers = Vec::new();
                        let mut resp_body = raw.clone();
                        if let Some(pos) = raw.find("\r\n\r\n").or_else(|| raw.find("\n\n")) {
                            let (head, rest) = raw.split_at(pos);
                            resp_body = rest
                                .trim_start_matches("\r\n\r\n")
                                .trim_start_matches("\n\n")
                                .to_string();
                            let mut lines = head.lines();
                            if let Some(status_line) = lines.next() {
                                let parts: Vec<&str> = status_line.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    if let Ok(code) = parts[1].parse::<u16>() {
                                        status = code;
                                    }
                                }
                            }
                            for line in lines {
                                if let Some((k, v)) = line.split_once(':') {
                                    resp_headers.push((k.trim().to_string(), v.trim().to_string()));
                                }
                            }
                        }
                        IpcResponse::success(json!({
                            "status": status,
                            "latency_ms": latency_ms,
                            "headers": resp_headers,
                            "body": resp_body,
                        }))
                    }
                    Err(e) => IpcResponse::err(-32000, format!("Fetch failed: {}", e)),
                }
            }
            IpcRequest::RunScan => {
                let options = FileInventoryOptions::default();
                match FileInventory::scan_dir_with_options(&self.repo_root, &options) {
                    Ok(summary) => match memory.sync_inventory(&summary) {
                        Ok(sync_stats) => IpcResponse::success(json!({
                            "total_files": summary.total_files,
                            "total_symbols": summary.total_symbols,
                            "total_size_bytes": summary.total_size_bytes,
                            "sync_stats": sync_stats,
                        })),
                        Err(e) => IpcResponse::err(
                            -32000,
                            format!("Failed to sync inventory to SQLite: {}", e),
                        ),
                    },
                    Err(e) => IpcResponse::err(-32000, format!("Inventory scan failed: {}", e)),
                }
            }
            IpcRequest::RunDoctor => match DoctorEngine::run(&self.repo_root, None) {
                Ok(report) => {
                    let total = report.summary.pass + report.summary.warn + report.summary.fail;
                    let health_score = if total == 0 {
                        100
                    } else {
                        ((report.summary.pass * 100 + report.summary.warn * 50) / total) as u32
                    };
                    IpcResponse::success(json!({
                        "health_score": health_score,
                        "has_failures": report.has_failures(),
                        "checks": report.checks,
                        "summary": report.render_terminal(),
                    }))
                }
                Err(e) => IpcResponse::err(-32000, format!("Doctor run failed: {}", e)),
            },
            IpcRequest::GetFailureCards => match memory.get_recent_failures(50) {
                Ok(failures) => {
                    let cards: Vec<_> = failures
                        .into_iter()
                        .map(|f| {
                            json!({
                                "id": format!("failure-{:03}", f.id.unwrap_or(0)),
                                "raw_id": f.id.unwrap_or(0),
                                "title": f.message,
                                "module": f.error_type,
                                "severity": "medium",
                                "occurrences": 1,
                                "status": f.status.unwrap_or_else(|| "unresolved".to_string()),
                                "last_seen": f.created_at,
                                "gotchas": vec![format!("Error type: {}", f.error_type)],
                                "anti_pattern_rules": f.fix_summary.map(|s| vec![s]).unwrap_or_default(),
                            })
                        })
                        .collect();
                    IpcResponse::success(cards)
                }
                Err(e) => IpcResponse::err(-32000, format!("Failed to read failures: {}", e)),
            },
            IpcRequest::ResolveFailureCard { id, fix_summary } => {
                match memory.resolve_failure(id, &fix_summary) {
                    Ok(()) => IpcResponse::success(json!({ "resolved": true, "id": id })),
                    Err(e) => IpcResponse::err(-32000, format!("Failed to resolve failure: {}", e)),
                }
            }
            IpcRequest::GetGitHookStatus => match HookManager::status(&self.repo_root) {
                Ok(status) => IpcResponse::success(json!({
                    "is_installed": status.is_installed,
                    "exists": status.exists,
                    "is_executable": status.is_executable,
                    "hook_path": status.hook_path,
                })),
                Err(e) => IpcResponse::err(-32000, format!("Failed to check hook status: {}", e)),
            },
            IpcRequest::SetGitHook { install } => {
                let res = if install {
                    HookManager::install(&self.repo_root, false, "/bin/sh").map(|_| true)
                } else {
                    HookManager::uninstall(&self.repo_root).map(|_| false)
                };
                match res {
                    Ok(installed) => IpcResponse::success(json!({ "installed": installed })),
                    Err(e) => IpcResponse::err(-32000, format!("Failed to set hook: {}", e)),
                }
            }
            IpcRequest::TestApiKey { provider, key } => {
                let mut cmd = Command::new("curl");
                cmd.arg("-s").arg("-S").arg("--max-time").arg("10");
                if provider.to_lowercase().contains("openrouter") {
                    cmd.arg("https://openrouter.ai/api/v1/auth/key")
                        .arg("-H")
                        .arg(format!("Authorization: Bearer {}", key));
                } else if provider.to_lowercase().contains("perplexity") {
                    cmd.arg("https://api.perplexity.ai/chat/completions")
                        .arg("-H")
                        .arg(format!("Authorization: Bearer {}", key))
                        .arg("-H")
                        .arg("Content-Type: application/json")
                        .arg("-d")
                        .arg("{\"model\":\"sonar\",\"messages\":[{\"role\":\"user\",\"content\":\"ping\"}]}");
                } else {
                    return IpcResponse::success(json!({
                        "valid": !key.trim().is_empty(),
                        "provider": provider,
                    }));
                }
                match cmd.output() {
                    Ok(out) => {
                        let body = String::from_utf8_lossy(&out.stdout).to_string();
                        let valid = !body.contains("Invalid API key")
                            && !body.contains("Unauthorized")
                            && out.status.success();
                        IpcResponse::success(json!({
                            "valid": valid,
                            "provider": provider,
                            "response": body,
                        }))
                    }
                    Err(e) => IpcResponse::err(-32000, format!("Probe failed: {}", e)),
                }
            }
            IpcRequest::SaveConfigSettings { settings } => {
                let mut config =
                    Config::load(&self.repo_root).unwrap_or_else(|_| Config::default_config());
                if let Some(mode) = settings.get("permission_mode").and_then(|v| v.as_str()) {
                    if let Ok(m) = mode.parse() {
                        config.permissions.mode = m;
                    }
                }
                if let Some(model) = settings.get("model").and_then(|v| v.as_str()) {
                    config.llm.coder_model = model.to_string();
                }
                if let Err(e) = config.save(&self.repo_root) {
                    IpcResponse::err(-32000, format!("Failed to save config: {}", e))
                } else {
                    IpcResponse::success(json!({ "saved": true }))
                }
            }
        }
    }
}

