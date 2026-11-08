use crate::desktop::ipc::DaemonIpcClient;
use crate::desktop::protocol::{IpcEvent, IpcRequest, IpcResponse};
use crate::memory::MemoryEngine;
use crate::review::{ReviewArgs, ReviewEngine};
use crate::safety::audit::{AuditLogEntry, AuditLogger};
use crate::tui::app::{DaemonConnectionState, TuiApp, TuiCommitItem, TuiSymbolItem};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub struct TuiClient {
    ipc_client: Option<Arc<DaemonIpcClient>>,
    event_sub: Option<broadcast::Receiver<IpcEvent>>,
}

impl TuiClient {
    pub fn new_offline() -> Self {
        Self {
            ipc_client: None,
            event_sub: None,
        }
    }

    pub fn new_with_client(client: Arc<DaemonIpcClient>) -> Self {
        let sub = client.subscribe_events();
        Self {
            ipc_client: Some(client),
            event_sub: Some(sub),
        }
    }

    pub async fn connect(daemon_url: Option<&str>) -> Self {
        if let Some(url) = daemon_url {
            if let Ok(client) =
                DaemonIpcClient::connect_with_timeout(url, Duration::from_millis(1200)).await
            {
                let sub = client.subscribe_events();
                return Self {
                    ipc_client: Some(client),
                    event_sub: Some(sub),
                };
            }
        }
        Self::new_offline()
    }

    pub fn is_connected(&self) -> bool {
        self.ipc_client.is_some()
    }

    /// Approve a pending tool proposal through the daemon.
    pub async fn approve(&self, proposal_id: &str) -> Result<()> {
        self.send_approval(IpcRequest::ApproveToolCall {
            proposal_id: proposal_id.to_string(),
        })
        .await
    }

    /// Deny a pending tool proposal through the daemon.
    pub async fn deny(&self, proposal_id: &str) -> Result<()> {
        self.send_approval(IpcRequest::DenyToolCall {
            proposal_id: proposal_id.to_string(),
            reason: Some("Denied from the terminal UI".to_string()),
        })
        .await
    }

    async fn send_approval(&self, request: IpcRequest) -> Result<()> {
        let Some(ipc) = &self.ipc_client else {
            return Err(anyhow::anyhow!(
                "the TUI is offline; no daemon is hosting an agent"
            ));
        };
        match ipc.send_request(request).await? {
            IpcResponse::Ok(_) => Ok(()),
            IpcResponse::Error { message, .. } => Err(anyhow::anyhow!(message)),
        }
    }

    /// Process any streaming events that arrived from the daemon.
    pub fn poll_events(&mut self, app: &mut TuiApp) {
        if let Some(sub) = &mut self.event_sub {
            while let Ok(event) = sub.try_recv() {
                match event {
                    IpcEvent::AgentThought { thought, .. } => {
                        app.push_agent_log(format!("💭 THOUGHT: {}", thought));
                    }
                    IpcEvent::ToolProposed(proposal) => {
                        app.set_proposal(proposal);
                    }
                    IpcEvent::ToolExecutionResult {
                        proposal_id,
                        success,
                        output,
                    } => {
                        let status_emoji = if success { "✅" } else { "❌" };
                        app.push_agent_log(format!(
                            "{} Tool [{}] result: {}",
                            status_emoji, proposal_id, output
                        ));
                    }
                    IpcEvent::FileChanged { path, change_type } => {
                        app.push_activity(format!("[WATCH] {} -> {}", path, change_type));
                    }
                    IpcEvent::MemoryUpdated {
                        total_files,
                        total_symbols,
                        ..
                    } => {
                        app.total_files = total_files;
                        app.total_symbols = total_symbols;
                        app.push_activity(format!(
                            "[MEMORY] Updated index ({} files, {} symbols)",
                            total_files, total_symbols
                        ));
                    }
                    IpcEvent::AuditLogAppended(entry) => {
                        app.push_activity(format!(
                            "[AUDIT] {} on {} ({})",
                            entry.action,
                            entry.target,
                            if entry.approved { "ALLOWED" } else { "DENIED" }
                        ));
                        app.audit_entries.insert(0, *entry);
                    }
                    IpcEvent::TaskCompleted {
                        task_id,
                        success,
                        summary,
                    } => {
                        app.task_title = "Idle".to_string();
                        let status_str = if success { "Success" } else { "Failed" };
                        app.push_agent_log(format!(
                            "🏁 Task {} finished ({}) - {}",
                            task_id, status_str, summary
                        ));
                    }
                    IpcEvent::Heartbeat { .. } => {}
                }
            }
        }
    }

    /// Synchronize data from daemon IPC or fallback local SQLite database.
    pub async fn refresh(&self, app: &mut TuiApp) -> Result<()> {
        if let Some(ipc) = &self.ipc_client {
            app.daemon_state = DaemonConnectionState::Connected;
            // Fetch memory stats
            if let Ok(IpcResponse::Ok(val)) = ipc.send_request(IpcRequest::GetMemoryStats).await {
                if let Some(files) = val.get("total_files").and_then(|v| v.as_u64()) {
                    app.total_files = files as usize;
                }
                if let Some(syms) = val.get("total_symbols").and_then(|v| v.as_u64()) {
                    app.total_symbols = syms as usize;
                }
            }

            // Fetch review diff
            if let Ok(IpcResponse::Ok(val)) = ipc
                .send_request(IpcRequest::GetReviewDiff { cached: false })
                .await
            {
                if let Some(diff) = val.get("diff").and_then(|v| v.as_str()) {
                    if !diff.trim().is_empty() {
                        app.diff_content = diff.to_string();
                    }
                }
                if let Some(score) = val.get("risk_score").and_then(|v| v.as_u64()) {
                    app.risk_score = score as u32;
                }
                if let Some(lvl) = val.get("risk_level").and_then(|v| v.as_str()) {
                    app.risk_level = lvl.to_string();
                }
            }

            // Audit trail and documentation health live in the daemon, not in
            // the TUI process. Fetch them instead of showing empty panels.
            if let Ok(IpcResponse::Ok(val)) = ipc
                .send_request(IpcRequest::GetAuditLogs { limit: 30 })
                .await
            {
                if let Ok(entries) = serde_json::from_value::<Vec<AuditLogEntry>>(val) {
                    app.audit_entries = entries;
                }
            }

            if let Ok(IpcResponse::Ok(val)) = ipc.send_request(IpcRequest::GetDocsHealth).await {
                let total = val.get("total_docs").and_then(|v| v.as_u64()).unwrap_or(0);
                let fresh = val.get("fresh_docs").and_then(|v| v.as_u64()).unwrap_or(0);
                if let Some(score) = (fresh * 100).checked_div(total) {
                    app.health_score = score as u32;
                }
            }
        } else {
            // Local fallback querying
            Self::refresh_local(app);
        }

        app.sync_time = "just now".to_string();
        Ok(())
    }

    /// Query repository files, SQLite memory, and Git directly if offline.
    pub fn refresh_local(app: &mut TuiApp) {
        app.daemon_state = DaemonConnectionState::Disconnected;
        let db_path = app.repo_path.join(".agent").join("memory.sqlite");
        if db_path.exists() {
            if let Ok(mem) = MemoryEngine::open(&db_path) {
                // Populate files & symbols count
                if let Ok(files) = mem.list_files() {
                    app.total_files = files.len();
                }
                if let Ok(symbols) = mem.get_all_symbols() {
                    app.total_symbols = symbols.len();
                    app.symbols = symbols
                        .into_iter()
                        .take(100)
                        .map(|s| TuiSymbolItem {
                            name: s.name,
                            kind: s.kind,
                            file_path: format!("file_id:{}", s.file_id),
                            line_number: s.line_start,
                            visibility: s.visibility.unwrap_or_else(|| "pub".to_string()),
                            doc_summary: s.summary,
                            callers_count: 0,
                        })
                        .collect();
                }

                // Populate recent commits
                if let Ok(commits) = mem.get_recent_commits(5) {
                    app.recent_commits = commits
                        .into_iter()
                        .map(|c| {
                            let hash_short = if c.hash.len() > 7 {
                                c.hash[..7].to_string()
                            } else {
                                c.hash.clone()
                            };
                            TuiCommitItem {
                                commit_hash: hash_short,
                                message: c.message.lines().next().unwrap_or("").to_string(),
                                files_changed: 1,
                                symbols_delta: c.summary.unwrap_or_else(|| "recorded".to_string()),
                                risk_level: c.risk_level.unwrap_or_else(|| "Low".to_string()),
                            }
                        })
                        .collect();
                }

                // Populate local review diff if git repository exists
                let review_args = ReviewArgs::default();
                if let Ok(report) = ReviewEngine::review(&app.repo_path, &mem, &review_args) {
                    let diff_text =
                        ReviewEngine::get_diff(&app.repo_path, &review_args).unwrap_or_default();
                    app.apply_risk_assessment(&report.risk, &diff_text);
                }
            }
        }

        // Populate local audit logs
        let audit_logger = AuditLogger::new(Some(&app.repo_path));
        if let Ok(entries) = audit_logger.list_recent(30) {
            app.audit_entries = entries;
        }
    }
}
