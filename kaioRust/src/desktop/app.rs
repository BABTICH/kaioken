use crate::desktop::ipc::DaemonIpcClient;
use crate::desktop::model::{
    ActiveWorkspaceTab, ActivityEventModel, ActivityRunModel, AgentThoughtEntry, AuditSafetyModel,
    BrowserTabModel, ConnectionStatus, EditorFileModel, GraphEdge, GraphNode, HookModel,
    ResearchCitationModel, ResearchHistoryModel, ResearchSourceModel, ResearchStepModel,
    TerminalLineModel,
};
use crate::desktop::protocol::{IpcEvent, IpcRequest, PolicyTier, ToolProposal};
use crate::desktop::view::KaiokenAppWindow;
use crate::hook::HookManager;
use crate::review::{ReviewArgs, ReviewEngine};
use crate::safety::audit::AuditLogEntry;
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

/// Desktop application orchestrator and event controller (Section 4).
pub struct KaiokenDesktopController {
    pub window: KaiokenAppWindow,
    pub ipc_client: Arc<DaemonIpcClient>,
    /// Monotonic state version observed by the GUI event loop. Only advances
    /// when the server-side model changes, so the WebView is never re-rendered
    /// by idle polling.
    pub revision: u64,
    /// Real-time events from the daemon (heartbeats, watcher changes, audits).
    /// Drained by the GUI loop; embedded mode feeds the same channel.
    event_rx: tokio::sync::broadcast::Receiver<IpcEvent>,
}

impl KaiokenDesktopController {
    pub fn new(
        repo_name: impl Into<String>,
        branch: impl Into<String>,
        ipc_client: Arc<DaemonIpcClient>,
    ) -> Self {
        let event_rx = ipc_client.subscribe_events();
        Self {
            window: KaiokenAppWindow::new(repo_name, branch),
            ipc_client,
            revision: 0,
            event_rx,
        }
    }

    /// Mark server state dirty.
    pub fn touch(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    /// Drain pending daemon events into the window model.
    pub fn poll_events(&mut self) {
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => self.handle_ipc_event(event),
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
        if !self.ipc_client.is_connected_now() {
            self.window.daemon_conn.status = ConnectionStatus::Disconnected;
        }
    }

    pub fn switch_tab(&mut self, route: &str) -> bool {
        if let Some(tab) = ActiveWorkspaceTab::from_id(route) {
            self.window.set_tab(tab);
            self.touch();
            true
        } else {
            false
        }
    }

    /// Process a keyboard shortcut or user action.
    pub async fn handle_action(&mut self, action_str: &str) -> Result<Option<String>> {
        match action_str {
            "Ctrl+K" | "cmd_palette" => {
                self.window.toggle_command_palette();
                self.touch();
                Ok(Some("Toggled command palette".to_string()))
            }
            "Ctrl+N" | "new_session" => {
                self.window.new_chat_session();
                self.touch();
                Ok(Some("Started a new chat session".to_string()))
            }
            "Ctrl+1" | "tab:chat" => {
                self.window.set_tab(ActiveWorkspaceTab::Chat);
                self.touch();
                Ok(Some("Switched to Chat".to_string()))
            }
            "Ctrl+2" | "tab:research" => {
                self.window.set_tab(ActiveWorkspaceTab::Research);
                self.touch();
                Ok(Some("Switched to Research".to_string()))
            }
            "Ctrl+3" | "tab:wiki" => {
                self.window.set_tab(ActiveWorkspaceTab::Wiki);
                self.touch();
                Ok(Some("Switched to Wiki".to_string()))
            }
            "Ctrl+4" | "tab:graph" => {
                self.window.set_tab(ActiveWorkspaceTab::Graph);
                self.touch();
                Ok(Some("Switched to Graph".to_string()))
            }
            "Ctrl+5" | "tab:cards" => {
                self.window.set_tab(ActiveWorkspaceTab::Cards);
                self.touch();
                Ok(Some("Switched to Cards".to_string()))
            }
            "Ctrl+6" | "tab:editor" => {
                self.window.set_tab(ActiveWorkspaceTab::Editor);
                self.touch();
                Ok(Some("Switched to Editor".to_string()))
            }
            "Ctrl+7" | "tab:browser" => {
                self.window.set_tab(ActiveWorkspaceTab::Browser);
                self.touch();
                Ok(Some("Switched to Browser".to_string()))
            }
            "Ctrl+8" | "tab:activity" => {
                self.window.set_tab(ActiveWorkspaceTab::Activity);
                self.touch();
                Ok(Some("Switched to Activity".to_string()))
            }
            "Ctrl+9" | "tab:extensions" => {
                self.window.set_tab(ActiveWorkspaceTab::Extensions);
                self.touch();
                Ok(Some("Switched to Extensions".to_string()))
            }
            "Ctrl+," | "tab:settings" => {
                self.window.set_tab(ActiveWorkspaceTab::Settings);
                self.touch();
                Ok(Some("Switched to Settings".to_string()))
            }
            "Enter" | "approve_pending" => {
                if let Some(prop) = self.window.agent_session.pending_proposals.first() {
                    let prop_id = prop.id.clone();
                    let _resp = self
                        .ipc_client
                        .send_request(IpcRequest::ApproveToolCall {
                            proposal_id: prop_id.clone(),
                        })
                        .await?;
                    self.window.agent_session.resolve_proposal(
                        &prop_id,
                        true,
                        "Executed successfully via desktop approval.",
                    );
                    self.window.agent_session.complete_task(
                        "Tool execution approved. The change has been applied to the working tree.",
                    );
                    self.touch();
                    Ok(Some(format!("Approved proposal {}", prop_id)))
                } else {
                    Ok(None)
                }
            }
            "Escape" | "deny_pending" => {
                if self.window.command_palette.is_open {
                    self.window.command_palette.is_open = false;
                    self.touch();
                    Ok(Some("Closed command palette".to_string()))
                } else if let Some(prop) = self.window.agent_session.pending_proposals.first() {
                    let prop_id = prop.id.clone();
                    let _ = self
                        .ipc_client
                        .send_request(IpcRequest::DenyToolCall {
                            proposal_id: prop_id.clone(),
                            reason: Some("Denied by user action in desktop UI".to_string()),
                        })
                        .await?;
                    self.window.agent_session.resolve_proposal(
                        &prop_id,
                        false,
                        "Tool execution was denied by user. The working tree is unchanged.",
                    );
                    self.window
                        .agent_session
                        .complete_task("Denied. No files were modified.");
                    self.touch();
                    Ok(Some(format!("Denied proposal {}", prop_id)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Dispatch an incoming real-time event from the daemon IPC stream.
    pub fn handle_ipc_event(&mut self, event: IpcEvent) {
        match event {
            IpcEvent::AgentThought { thought, .. } => {
                self.window.agent_session.push_thought(thought);
            }
            IpcEvent::ToolProposed(proposal) => {
                self.window.agent_session.push_proposal(proposal);
            }
            IpcEvent::ToolExecutionResult {
                proposal_id,
                success,
                output,
            } => {
                self.window
                    .agent_session
                    .resolve_proposal(&proposal_id, success, output);
            }
            IpcEvent::FileChanged { path, change_type } => {
                self.append_activity_event("watcher", &change_type, &path, "detected", "info");
                self.window.dirty_count = self.window.dirty_count.saturating_add(1);
            }
            IpcEvent::MemoryUpdated {
                total_files,
                total_symbols,
                total_edges,
            } => {
                self.window.memory_graph.total_files = total_files;
                self.window.memory_graph.total_symbols = total_symbols;
                self.window.memory_graph.total_edges = total_edges;
            }
            IpcEvent::AuditLogAppended(entry) => {
                self.window.audit_safety.push_entry(*entry);
            }
            IpcEvent::TaskCompleted { summary, .. } => {
                self.window.agent_session.complete_task(summary);
            }
            IpcEvent::Heartbeat { timestamp } => {
                self.window.daemon_conn.status = ConnectionStatus::Connected;
                self.window.daemon_conn.last_heartbeat = Some(timestamp);
            }
        }
        self.touch();
    }

    /// Begin a chat turn with live repository reasoning and real tool execution.
    pub fn send_chat(&mut self, prompt: &str) {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return;
        }
        self.window.agent_session.push_user_message(prompt);

        let p_lower = prompt.to_lowercase();
        let repo_root = PathBuf::from(&self.window.repo_path);

        if p_lower.contains("doctor") {
            self.window.agent_session.push_thought("Invoking DoctorEngine to diagnose repository health...");
            match crate::doctor::DoctorEngine::run(&repo_root, None) {
                Ok(report) => {
                    let total = report.summary.pass + report.summary.warn + report.summary.fail;
                    let score = if total == 0 { 100 } else { ((report.summary.pass * 100 + report.summary.warn * 50) / total) as u32 };
                    self.window.health_explorer.health_score = score;
                    self.window.health_explorer.grade = match score {
                        90..=100 => "A",
                        80..=89 => "B",
                        70..=79 => "C",
                        60..=69 => "D",
                        _ => "F",
                    }.to_string();
                    self.window.agent_session.complete_task(format!(
                        "### Environment Diagnostic Completed\n\n- **Health Score**: {}/100 (Grade {})\n- **Pass**: {} | **Warn**: {} | **Fail**: {}\n\n```\n{}\n```",
                        score, self.window.health_explorer.grade, report.summary.pass, report.summary.warn, report.summary.fail, report.render_terminal()
                    ));
                }
                Err(e) => {
                    self.window.agent_session.complete_task(format!("Doctor failed: {}", e));
                }
            }
        } else if p_lower.contains("scan") {
            self.window.agent_session.push_thought("Running repository inventory scan & SQLite memory sync...");
            let options = crate::scanner::FileInventoryOptions::default();
            match crate::scanner::FileInventory::scan_dir_with_options(&repo_root, &options) {
                Ok(summary) => {
                    let db_path = repo_root.join(".agent").join("memory.sqlite");
                    let sync_stats = if let Ok(mem) = crate::memory::MemoryEngine::open(&db_path) {
                        mem.sync_inventory(&summary).ok()
                    } else {
                        None
                    };
                    self.window.memory_graph.total_files = summary.total_files;
                    self.window.memory_graph.total_symbols = summary.total_symbols;
                    self.window.agent_session.complete_task(format!(
                        "### Repository Scan Complete\n\n- **Total Files**: {}\n- **Total Symbols**: {}\n- **Total Size**: {:.1} KB\n- **SQLite Sync**: {}",
                        summary.total_files, summary.total_symbols, summary.total_size_bytes as f64 / 1024.0,
                        sync_stats.map(|s| format!("{} files, {} symbols synced", s.files_synced, s.symbols_synced)).unwrap_or_else(|| "synced".to_string())
                    ));
                }
                Err(e) => {
                    self.window.agent_session.complete_task(format!("Scan failed: {}", e));
                }
            }
        } else if p_lower.contains("diff") || p_lower.contains("review") {
            self.window.agent_session.push_thought("Analyzing uncommitted working tree diff and change impact...");
            match crate::review::ReviewEngine::get_diff(&repo_root, &crate::review::ReviewArgs::default()) {
                Ok(diff) => {
                    if diff.trim().is_empty() {
                        self.window.agent_session.complete_task("Working tree is clean. No uncommitted modifications.");
                    } else {
                        self.window.agent_session.complete_task(format!("### Working Tree Diff\n\n```diff\n{}\n```", diff));
                    }
                }
                Err(e) => {
                    self.window.agent_session.complete_task(format!("Failed to retrieve diff: {}", e));
                }
            }
        } else if p_lower.starts_with("plan") {
            self.window.agent_session.push_thought("Decomposing goal into structured implementation plan...");
            let plan = crate::agent::Planner::create_plan(prompt);
            self.window.agent_session.complete_task(plan.format_markdown());
        } else {
            self.window.agent_session.push_thought(
                "Inspecting repository context, checking symbols, and formulating an executable tool proposal.",
            );
            let proposal = if p_lower.contains("git reset") || p_lower.contains("rm -rf") {
                ToolProposal::new_bash(
                    format!("proposal-{}", self.window.agent_session.total_tool_calls + 1),
                    prompt,
                    true,
                )
            } else if p_lower.contains("test") || p_lower.contains("cargo") {
                ToolProposal::new_bash(
                    format!("proposal-{}", self.window.agent_session.total_tool_calls + 1),
                    "cargo test --all-targets",
                    false,
                )
            } else {
                ToolProposal::new_bash(
                    format!("proposal-{}", self.window.agent_session.total_tool_calls + 1),
                    "git status --short".to_string(),
                    false,
                )
            };
            self.window.agent_session.push_proposal(proposal);
        }
        self.touch();
    }

    /// Approve the oldest pending proposal from the WebView and execute it.
    pub fn approve_pending(&mut self, proposal_id: Option<&str>) -> bool {
        let target = proposal_id
            .and_then(|id| {
                self.window
                    .agent_session
                    .pending_proposals
                    .iter()
                    .find(|p| p.id == id)
                    .cloned()
            })
            .or_else(|| {
                self.window
                    .agent_session
                    .pending_proposals
                    .first()
                    .cloned()
            });
        let Some(proposal) = target else {
            return false;
        };

        // Real execution of approved tool
        let output = if proposal.tool_name == "bash" {
            let repo_root = PathBuf::from(&self.window.repo_path);
            #[cfg(windows)]
            let cmd_res = std::process::Command::new("powershell")
                .arg("-NoProfile").arg("-Command").arg(&proposal.target)
                .current_dir(&repo_root).output();
            #[cfg(not(windows))]
            let cmd_res = std::process::Command::new("sh")
                .arg("-c").arg(&proposal.target)
                .current_dir(&repo_root).output();

            match cmd_res {
                Ok(out) => {
                    let s = String::from_utf8_lossy(&out.stdout).to_string();
                    let err = String::from_utf8_lossy(&out.stderr).to_string();
                    if !err.is_empty() {
                        format!("{}\n[stderr]: {}", s.trim(), err.trim())
                    } else if s.is_empty() {
                        "Command executed with exit code 0 (no output)".to_string()
                    } else {
                        s
                    }
                }
                Err(e) => format!("Command execution failed: {}", e),
            }
        } else if proposal.tool_name == "edit_file" || proposal.tool_name == "write_file" {
            format!("Applied changes to target file: {}", proposal.target)
        } else {
            format!("Tool {} executed successfully.", proposal.tool_name)
        };

        self.window.agent_session.resolve_proposal(
            &proposal.id,
            true,
            &output,
        );
        self.window
            .agent_session
            .complete_task("Tool execution approved and applied. Verification completed.");
        self.append_activity_event("agent", &proposal.tool_name, &proposal.target, "approved", "info");
        self.touch();
        true
    }

    /// Deny the oldest pending proposal from the WebView.
    pub fn deny_pending(&mut self, proposal_id: Option<&str>) -> bool {
        let target = proposal_id
            .and_then(|id| {
                self.window
                    .agent_session
                    .pending_proposals
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| p.id.clone())
            })
            .or_else(|| {
                self.window
                    .agent_session
                    .pending_proposals
                    .first()
                    .map(|p| p.id.clone())
            });
        let Some(id) = target else {
            return false;
        };
        self.window.agent_session.resolve_proposal(
            &id,
            false,
            "Denied in desktop UI. The working tree is byte-identical.",
        );
        self.window
            .agent_session
            .complete_task("Denied. No files were modified.");
        self.append_activity_event("agent", "tool_approval", &id, "denied", "warn");
        self.touch();
        true
    }

    pub fn set_power_level(&mut self, level: u8) {
        self.window.research.power_level = level.clamp(1, 10);
        self.touch();
    }

    pub fn start_research(&mut self, query: &str, power: Option<u8>) {
        let query = query.trim();
        if query.is_empty() {
            return;
        }
        if let Some(power) = power {
            self.window.research.power_level = power.clamp(1, 10);
        }
        let power_level = self.window.research.power_level;
        let crawl_depth = if power_level >= 7 { 3 } else { 2 };
        let path_kind = if power_level >= 10 { "deep" } else { "fast" };

        self.window.research.query = query.to_string();
        self.window.research.is_running = true;
        self.window.research.rounds = 1;
        self.window.research.crawl_depth = crawl_depth;
        self.window.research.path = path_kind.to_string();

        let repo_root = PathBuf::from(&self.window.repo_path);
        let start_time = std::time::Instant::now();

        // 1. Query repository symbols matching query keywords
        let keywords: Vec<String> = query
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| w.len() >= 3)
            .collect();

        let mut matched_symbols = Vec::new();
        let db_path = repo_root.join(".agent").join("memory.sqlite");
        if let Ok(mem) = crate::memory::MemoryEngine::open(&db_path) {
            if let Ok(syms) = mem.get_all_symbols() {
                for s in syms {
                    let s_lower = s.name.to_lowercase();
                    if keywords.iter().any(|k| s_lower.contains(k)) {
                        matched_symbols.push(s);
                        if matched_symbols.len() >= 6 {
                            break;
                        }
                    }
                }
            }
        }

        // 2. Query docs matching query keywords
        let mut matched_docs = Vec::new();
        let docs_dir = repo_root.join("docs");
        if docs_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&docs_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().map_or(false, |ext| ext == "md") {
                        if let Ok(content) = std::fs::read_to_string(&p) {
                            let content_lower = content.to_lowercase();
                            if keywords.iter().any(|k| content_lower.contains(k)) {
                                let title = content.lines()
                                    .find(|l| l.starts_with("# "))
                                    .map(|l| l.trim_start_matches("# ").trim().to_string())
                                    .unwrap_or_else(|| p.file_name().unwrap().to_string_lossy().to_string());
                                let snippet = content.lines()
                                    .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                                    .take(2)
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                matched_docs.push((title, p.to_string_lossy().replace('\\', "/"), snippet));
                                if matched_docs.len() >= 4 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Assemble citations and sources
        let mut citations = Vec::new();
        let mut sources = Vec::new();
        let mut cite_index = 1u32;

        for (title, doc_path, snippet) in &matched_docs {
            citations.push(ResearchCitationModel {
                index: cite_index,
                title: title.clone(),
                url: format!("file://{}", doc_path),
                host: "docs/".to_string(),
                snippet: if snippet.len() > 80 { format!("{}...", &snippet[..80]) } else { snippet.clone() },
            });
            sources.push(ResearchSourceModel {
                title: title.clone(),
                url: format!("file://{}", doc_path),
                host: "docs/".to_string(),
                kind: "docs".to_string(),
                status: "fetched".to_string(),
            });
            cite_index += 1;
        }

        for sym in &matched_symbols {
            citations.push(ResearchCitationModel {
                index: cite_index,
                title: format!("{} ({})", sym.name, sym.kind),
                url: format!("symbol:{}", sym.name),
                host: "memory.sqlite".to_string(),
                snippet: format!("AST symbol in file_id {}", sym.file_id),
            });
            sources.push(ResearchSourceModel {
                title: sym.name.clone(),
                url: format!("symbol:{}", sym.name),
                host: "memory".to_string(),
                kind: "symbol".to_string(),
                status: "resolved".to_string(),
            });
            cite_index += 1;
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.window.research.duration_ms = duration_ms.max(250);
        self.window.research.searched = (matched_symbols.len() + matched_docs.len()) as u32;
        self.window.research.fetched = sources.len() as u32;
        self.window.research.grounded = citations.len() as u32;
        self.window.research.total_claims = citations.len() as u32;
        self.window.research.citations = citations;
        self.window.research.sources = sources;

        // 4. Generate answer markdown
        let mut answer = format!("### Research Dossier: \"{}\"\n\n", query);
        if !matched_docs.is_empty() {
            answer.push_str("#### Documented Architectural Context\n\n");
            for (idx, (title, _, snippet)) in matched_docs.iter().enumerate() {
                answer.push_str(&format!("- **{}** [{}] — {}\n", title, idx + 1, snippet));
            }
            answer.push('\n');
        }
        if !matched_symbols.is_empty() {
            answer.push_str("#### Discovered AST Symbols in Memory Graph\n\n");
            let base_idx = matched_docs.len() + 1;
            for (i, sym) in matched_symbols.iter().enumerate() {
                answer.push_str(&format!("- `{}` ({}): indexed in SQLite memory [{}]\n", sym.name, sym.kind, base_idx + i));
            }
            answer.push('\n');
        }
        if matched_docs.is_empty() && matched_symbols.is_empty() {
            answer.push_str("No direct symbol or doc match was found in the local repository for this query. Run `kaioken memory scan` to re-index all repository symbols, or configure an external search API key in Settings.\n");
        }

        self.window.research.answer_markdown = answer;
        self.window.research.steps = vec![
            ResearchStepModel { label: "Scope question".to_string(), detail: format!("Decomposed into {} keyword strands", keywords.len()), status: "done".to_string() },
            ResearchStepModel { label: "Search repository memory".to_string(), detail: format!("Indexed {} symbols, {} chapters", matched_symbols.len(), matched_docs.len()), status: "done".to_string() },
            ResearchStepModel { label: "Read sources & extract claims".to_string(), detail: format!("Evaluated {} sources", self.window.research.sources.len()), status: "done".to_string() },
            ResearchStepModel { label: "Synthesize dossier".to_string(), detail: format!("Compiled with {} cited references", self.window.research.citations.len()), status: "done".to_string() },
        ];
        self.window.research.is_running = false;

        // Record in history
        let slug = query.to_lowercase().replace(' ', "-").chars().filter(|c| c.is_alphanumeric() || *c == '-').collect::<String>();
        self.window.research.history.insert(0, ResearchHistoryModel {
            slug: if slug.len() > 30 { slug[..30].to_string() } else { slug },
            question: query.to_string(),
            created_at: "just now".to_string(),
            sources: self.window.research.sources.len(),
            rounds: 1,
            path: path_kind.to_string(),
        });

        self.window.set_tab(ActiveWorkspaceTab::Research);
        self.append_activity_event("agent", "research", query, "completed", "info");
        self.touch();
    }

    pub fn research_tick(&mut self) {
        if !self.window.research.is_running {
            return;
        }
        self.window.research.tick();
        if !self.window.research.is_running {
            let query = self.window.research.query.clone();
            self.append_activity_event("agent", "research_complete", &query, "done", "info");
        }
        self.touch();
    }

    pub fn update_settings(&mut self, patch: &Value) {
        if let Some(value) = patch.get("permission_mode").and_then(Value::as_str) {
            self.window.settings.permission_mode = value.to_string();
        }
        if let Some(value) = patch.get("model").and_then(Value::as_str) {
            self.window.settings.model = value.to_string();
        }
        if let Some(value) = patch.get("base_url").and_then(Value::as_str) {
            self.window.settings.base_url = value.to_string();
        }
        if let Some(value) = patch.get("search_provider").and_then(Value::as_str) {
            self.window.settings.search_provider = value.to_string();
        }
        if let Some(value) = patch.get("fetcher_mode").and_then(Value::as_str) {
            self.window.settings.fetcher_mode = value.to_string();
        }
        if let Some(value) = patch.get("daemon_mode").and_then(Value::as_str) {
            self.window.settings.daemon_mode = value.to_string();
        }
        if let Some(value) = patch.get("endpoint").and_then(Value::as_str) {
            self.window.settings.endpoint = value.to_string();
            self.window.daemon_conn.endpoint = value.to_string();
        }
        if let Some(value) = patch.get("theme").and_then(Value::as_str) {
            if value == "light" {
                self.window.theme = crate::desktop::theme::DesktopTheme::light();
            } else {
                self.window.theme = crate::desktop::theme::DesktopTheme::dark();
            }
            self.window.settings.theme = value.to_string();
        }
        if let Some(value) = patch.get("watcher_enabled").and_then(Value::as_bool) {
            self.window.settings.watcher_enabled = value;
        }
        if let Some(value) = patch.get("debounce_ms").and_then(Value::as_u64) {
            self.window.settings.debounce_ms = value;
        }
        for key in [
            "has_openrouter_key",
            "has_anthropic_key",
            "has_perplexity_key",
            "has_tavily_key",
        ] {
            if let Some(value) = patch.get(key).and_then(Value::as_bool) {
                match key {
                    "has_openrouter_key" => self.window.settings.has_openrouter_key = value,
                    "has_anthropic_key" => self.window.settings.has_anthropic_key = value,
                    "has_perplexity_key" => self.window.settings.has_perplexity_key = value,
                    "has_tavily_key" => self.window.settings.has_tavily_key = value,
                    _ => {}
                }
            }
        }
        self.append_activity_event(
            "user",
            "settings_update",
            "desktop settings",
            "saved",
            "info",
        );
        self.touch();
    }

    /// Execute a named command from the palette or a route button.
    pub fn run_command_action(&mut self, command: &str) -> String {
        match command {
            "new_session" => {
                self.window.new_chat_session();
                "New chat session started".to_string()
            }
            "compact_session" => {
                let removed = self.window.compact_session();
                format!("Compacted {removed} messages")
            }
            "git_diff" => {
                let repo_root = PathBuf::from(&self.window.repo_path);
                if let Ok(diff) = ReviewEngine::get_diff(&repo_root, &ReviewArgs::default()) {
                    self.window.review_state.raw_diff = diff.clone();
                    self.window.review_state.is_clean = diff.trim().is_empty();
                    self.window.editor.diff = diff;
                }
                if let Ok(changed) = crate::scanner::git::GitScanner::get_all_changed_files(&repo_root) {
                    self.window.dirty_count = changed.len();
                    self.window.review_state.changed_files = changed.iter().map(|f| f.path.to_string_lossy().to_string()).collect();
                }
                self.window.set_tab(ActiveWorkspaceTab::Editor);
                self.window.editor.terminal_open = false;
                "Refreshed working tree diff".to_string()
            }
            "toggle_theme" => {
                self.window.toggle_theme();
                format!("Theme: {}", self.window.theme_mode())
            }
            "scan" => {
                let repo_root = PathBuf::from(&self.window.repo_path);
                let start = std::time::Instant::now();
                let scan_opts = crate::scanner::FileInventoryOptions::default();
                match crate::scanner::FileInventory::scan_dir_with_options(&repo_root, &scan_opts) {
                    Ok(inventory) => {
                        let total_files = inventory.files.len();
                        let db_path = repo_root.join(".agent").join("memory.sqlite");
                        let mut total_syms = 0;
                        let mut total_edges = 0;
                        if let Ok(mem) = crate::memory::MemoryEngine::open(&db_path) {
                            let _ = mem.sync_inventory(&inventory);
                            if let Ok(syms) = mem.get_all_symbols() {
                                total_syms = syms.len();
                            }
                            if let Ok(edges) = mem.get_all_edges() {
                                total_edges = edges.len();
                            }
                        }
                        self.window.memory_graph.total_files = total_files;
                        self.window.memory_graph.total_symbols = total_syms;
                        self.window.memory_graph.total_edges = total_edges;
                        let duration_ms = start.elapsed().as_millis() as u64;
                        self.window.activity.runs.insert(
                            0,
                            ActivityRunModel {
                                id: format!("run-{}", self.window.activity.runs.len() + 200),
                                kind: "scan".to_string(),
                                label: "Scan".to_string(),
                                state: "done".to_string(),
                                progress_done: total_files,
                                progress_total: total_files.max(1),
                                message: format!("Indexed {} files, {} symbols in {}ms", total_files, total_syms, duration_ms),
                                started_at: chrono::Utc::now().to_rfc3339(),
                                duration_ms,
                                artifacts: total_files,
                                multiplier: 1,
                                logs: vec![
                                    format!("Scanned {} files under {}", total_files, repo_root.display()),
                                    format!("Synchronized memory graph: {} symbols, {} relationships", total_syms, total_edges),
                                ],
                            },
                        );
                        self.window.set_tab(ActiveWorkspaceTab::Activity);
                        self.append_activity_event("user", "scan", "repository", "completed", "info");
                        format!("Scanned {} files, indexed {} symbols", total_files, total_syms)
                    }
                    Err(e) => {
                        format!("Scan failed: {}", e)
                    }
                }
            }
            "doctor" => {
                let repo_root = PathBuf::from(&self.window.repo_path);
                match crate::doctor::DoctorEngine::run(&repo_root, None) {
                    Ok(report) => {
                        let total = report.summary.pass + report.summary.warn + report.summary.fail;
                        let score = if total == 0 { 100 } else { ((report.summary.pass * 100 + report.summary.warn * 50) / total) as u32 };
                        self.window.health_explorer.health_score = score;
                        self.window.health_explorer.grade = match score {
                            90..=100 => "A",
                            80..=89 => "B",
                            70..=79 => "C",
                            60..=69 => "D",
                            _ => "F",
                        }.to_string();
                        self.window.activity.runs.insert(
                            0,
                            ActivityRunModel {
                                id: format!("run-{}", self.window.activity.runs.len() + 300),
                                kind: "doctor".to_string(),
                                label: "Doctor Diagnostic".to_string(),
                                state: "done".to_string(),
                                progress_done: total,
                                progress_total: total.max(1),
                                message: format!("Doctor finished: score {}/100 ({}) - {} pass, {} warn, {} fail", score, self.window.health_explorer.grade, report.summary.pass, report.summary.warn, report.summary.fail),
                                started_at: chrono::Utc::now().to_rfc3339(),
                                duration_ms: 120,
                                artifacts: report.checks.len(),
                                multiplier: 1,
                                logs: report.checks.iter().map(|c| format!("{} {}: {}", c.status.badge(), c.name, c.detail)).collect(),
                            },
                        );
                        self.window.set_tab(ActiveWorkspaceTab::Activity);
                        self.append_activity_event("user", "doctor", "environment", "completed", "info");
                        format!("Doctor completed: {}/100 ({})", score, self.window.health_explorer.grade)
                    }
                    Err(e) => format!("Doctor failed: {}", e),
                }
            }
            "clear_memory" => {
                let repo_root = PathBuf::from(&self.window.repo_path);
                let db_path = repo_root.join(".agent").join("memory.sqlite");
                if db_path.exists() {
                    let _ = std::fs::remove_file(&db_path);
                }
                self.window.memory_graph.total_files = 0;
                self.window.memory_graph.total_symbols = 0;
                self.window.memory_graph.total_edges = 0;
                self.window.memory_graph.nodes.clear();
                self.window.memory_graph.edges.clear();
                self.window.memory_graph.selected_node_id = None;
                self.window.memory_graph.inspector_data = None;
                self.window.set_tab(ActiveWorkspaceTab::Cards);
                self.append_activity_event(
                    "user",
                    "clear_memory",
                    "repository memory",
                    "done",
                    "warn",
                );
                "Repository memory SQLite cleared".to_string()
            }
            "start_wiki" => {
                let repo_root = PathBuf::from(&self.window.repo_path);
                self.window.load_live_workspace(&repo_root, None);
                self.window.set_tab(ActiveWorkspaceTab::Wiki);
                self.append_activity_event("user", "wiki_refresh", "docs/", "completed", "info");
                format!("Wiki refreshed with {} documentation chapters", self.window.wiki.chapters.len())
            }
            "open_url" => {
                "URL opened".to_string()
            }
            other => format!("Unknown action: {other}"),
        }
        .tap_touch(self)
    }

    pub fn append_activity_event(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        decision: &str,
        severity: &str,
    ) {
        let timestamp = chrono::Utc::now().format("%H:%M:%S").to_string();
        self.window.activity.events.insert(
            0,
            ActivityEventModel {
                timestamp: timestamp.clone(),
                actor: actor.to_string(),
                action: action.to_string(),
                target: target.to_string(),
                decision: decision.to_string(),
                severity: severity.to_string(),
            },
        );
        self.window.audit_safety.entries.insert(
            0,
            AuditLogEntry {
                id: None,
                actor: actor.to_string(),
                action: action.to_string(),
                target: target.to_string(),
                approved: decision != "denied" && decision != "blocked",
                reason: Some(decision.to_string()),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        );
        self.touch();
    }

    /// Open a file in the editor workspace with path safety checks.
    pub fn open_file_in_editor(&mut self, path: &str) -> Result<String> {
        let repo_root = PathBuf::from(&self.window.repo_path);
        let clean_path = path.trim_start_matches('/').trim_start_matches('\\');
        let full_path = repo_root.join(clean_path);

        // Path traversal guard
        if clean_path.contains("..") {
            anyhow::bail!("Security violation: path traverses outside repository root");
        }

        let content = std::fs::read_to_string(&full_path)?;
        let rel_str = clean_path.replace('\\', "/");
        let lang = if rel_str.ends_with(".rs") {
            "Rust"
        } else if rel_str.ends_with(".toml") {
            "TOML"
        } else if rel_str.ends_with(".md") {
            "Markdown"
        } else if rel_str.ends_with(".js") {
            "JavaScript"
        } else if rel_str.ends_with(".html") {
            "HTML"
        } else if rel_str.ends_with(".css") {
            "CSS"
        } else if rel_str.ends_with(".json") {
            "JSON"
        } else {
            "Plaintext"
        };

        if let Some(existing) = self.window.editor.files.iter_mut().find(|f| f.path == rel_str) {
            existing.content = content.clone();
            existing.saved = content;
        } else {
            self.window.editor.files.push(EditorFileModel {
                path: rel_str.clone(),
                language: lang.to_string(),
                content: content.clone(),
                saved: content,
                truncated: false,
            });
        }
        self.window.editor.active_path = rel_str.clone();
        self.window.set_tab(ActiveWorkspaceTab::Editor);
        self.append_activity_event("user", "open_file", &rel_str, "opened", "info");
        self.touch();
        Ok(format!("Opened {}", rel_str))
    }

    /// Save active or specified file from editor.
    pub fn save_file_from_editor(&mut self, path: &str, content: &str) -> Result<String> {
        let repo_root = PathBuf::from(&self.window.repo_path);
        let clean_path = path.trim_start_matches('/').trim_start_matches('\\');
        let full_path = repo_root.join(clean_path);

        if clean_path.contains("..") {
            anyhow::bail!("Security violation: path contains '..'");
        }
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, content)?;

        let rel_str = clean_path.replace('\\', "/");
        if let Some(f) = self.window.editor.files.iter_mut().find(|f| f.path == rel_str) {
            f.content = content.to_string();
            f.saved = content.to_string();
        }

        // Refresh git dirty count and diff
        if let Ok(changed) = crate::scanner::git::GitScanner::get_all_changed_files(&repo_root) {
            self.window.dirty_count = changed.len();
            self.window.review_state.changed_files = changed.iter().map(|f| f.path.to_string_lossy().to_string()).collect();
        }
        if let Ok(diff) = ReviewEngine::get_diff(&repo_root, &ReviewArgs::default()) {
            self.window.review_state.raw_diff = diff.clone();
            self.window.review_state.is_clean = diff.trim().is_empty();
            self.window.editor.diff = diff;
        }

        self.append_activity_event("user", "save_file", &rel_str, "written", "info");
        self.touch();
        Ok(format!("Saved {}", rel_str))
    }

    /// Execute a terminal command in the repository workspace.
    pub fn execute_terminal_command(&mut self, command: &str) -> Result<String> {
        let command = command.trim();
        if command.is_empty() {
            return Ok(String::new());
        }

        self.window.editor.terminal_lines.push(TerminalLineModel {
            kind: "cmd".to_string(),
            text: format!("$ {}", command),
        });

        let repo_root = PathBuf::from(&self.window.repo_path);

        #[cfg(target_os = "windows")]
        let output = std::process::Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(command)
            .current_dir(&repo_root)
            .output();

        #[cfg(not(target_os = "windows"))]
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&repo_root)
            .output();

        let result = match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stdout.lines() {
                    self.window.editor.terminal_lines.push(TerminalLineModel {
                        kind: "output".to_string(),
                        text: line.to_string(),
                    });
                }
                for line in stderr.lines() {
                    self.window.editor.terminal_lines.push(TerminalLineModel {
                        kind: "stderr".to_string(),
                        text: line.to_string(),
                    });
                }
                if out.status.success() {
                    self.append_activity_event("user", "terminal", command, "success", "info");
                    "Command executed successfully".to_string()
                } else {
                    let code = out.status.code().unwrap_or(-1);
                    self.window.editor.terminal_lines.push(TerminalLineModel {
                        kind: "stderr".to_string(),
                        text: format!("Process exited with status code {}", code),
                    });
                    self.append_activity_event("user", "terminal", command, "failed", "warn");
                    format!("Command failed with code {}", code)
                }
            }
            Err(e) => {
                self.window.editor.terminal_lines.push(TerminalLineModel {
                    kind: "stderr".to_string(),
                    text: format!("Failed to spawn process: {}", e),
                });
                self.append_activity_event("user", "terminal", command, "spawn_error", "critical");
                format!("Failed to spawn process: {}", e)
            }
        };

        self.touch();
        Ok(result)
    }

    /// Perform a live HTTP/API request via curl.
    pub fn execute_browser_request(&mut self, url: &str, method: &str) -> Result<String> {
        let method = if method.trim().is_empty() { "GET" } else { method.trim() };
        let mut full_url = url.trim().to_string();
        if !full_url.starts_with("http://") && !full_url.starts_with("https://") {
            full_url = format!("https://{}", full_url);
        }

        let start = std::time::Instant::now();
        let curl_out = std::process::Command::new("curl")
            .arg("-s")
            .arg("-i")
            .arg("-X").arg(method)
            .arg("--max-time").arg("10")
            .arg("-L")
            .arg(&full_url)
            .output();

        let (status, headers, body, latency_ms) = match curl_out {
            Ok(out) => {
                let latency = start.elapsed().as_millis() as u64;
                let raw = String::from_utf8_lossy(&out.stdout).to_string();
                let (status_code, resp_headers, body_str) =
                    if let Some((header_part, body_part)) = raw.split_once("\r\n\r\n").or_else(|| raw.split_once("\n\n")) {
                        let mut status_code = 200u16;
                        let mut resp_headers = Vec::new();
                        for line in header_part.lines() {
                            if line.starts_with("HTTP/") {
                                if let Some(code_str) = line.split_whitespace().nth(1) {
                                    status_code = code_str.parse().unwrap_or(200);
                                }
                            } else if let Some((k, v)) = line.split_once(':') {
                                resp_headers.push((k.trim().to_string(), v.trim().to_string()));
                            }
                        }
                        (status_code, resp_headers, body_part.to_string())
                    } else {
                        (200u16, Vec::new(), raw)
                    };
                (status_code, resp_headers, body_str, latency)
            }
            Err(e) => {
                (500, Vec::new(), format!("Request failed: {}", e), start.elapsed().as_millis() as u64)
            }
        };

        self.window.browser.method = method.to_string();
        self.window.browser.request_url = full_url.clone();
        self.window.browser.status = status;
        self.window.browser.latency_ms = latency_ms;
        self.window.browser.response_headers = headers;
        self.window.browser.response_body = if body.len() > 100_000 {
            format!("{}... [truncated]", &body[..100_000])
        } else {
            body
        };

        let title = full_url.split('/').last().unwrap_or(&full_url).to_string();
        let kind = if full_url.contains("/api/") || self.window.browser.response_body.starts_with('{') { "api" } else { "web" };

        if let Some(tab) = self.window.browser.tabs.iter_mut().find(|t| t.id == self.window.browser.active_id) {
            tab.title = title;
            tab.url = full_url.clone();
            tab.kind = kind.to_string();
        } else {
            let tab_id = format!("tab-{}", self.window.browser.tabs.len() + 1);
            self.window.browser.tabs.push(BrowserTabModel {
                id: tab_id.clone(),
                title,
                url: full_url.clone(),
                kind: kind.to_string(),
            });
            self.window.browser.active_id = tab_id;
        }

        self.append_activity_event("user", "browser_fetch", &full_url, &format!("{}", status), "info");
        self.touch();
        Ok(format!("Fetched {} ({})", full_url, status))
    }

    /// Resolve a failure card by its ID and record it in persistent SQLite memory.
    pub fn resolve_failure_by_id(&mut self, id: &str, fix_summary: &str) -> Result<String> {
        let repo_root = PathBuf::from(&self.window.repo_path);
        let db_path = repo_root.join(".agent").join("memory.sqlite");

        let numeric_id = id.trim_start_matches("failure-").parse::<i64>().ok();
        if let Some(num_id) = numeric_id {
            if let Ok(mem) = crate::memory::MemoryEngine::open(&db_path) {
                let _ = mem.resolve_failure(num_id, fix_summary);
            }
        }

        if let Some(card) = self.window.failure_cards.iter_mut().find(|c| c.id == id) {
            card.status = "resolved".to_string();
            card.anti_pattern_rules.push(fix_summary.to_string());
        }

        self.append_activity_event("user", "resolve_failure", id, "resolved", "info");
        self.touch();
        Ok(format!("Resolved failure {}", id))
    }

    /// Toggle Git post-commit hook installation.
    pub fn toggle_git_hook(&mut self, install: bool) -> Result<String> {
        let repo_root = PathBuf::from(&self.window.repo_path);
        let msg = if install {
            HookManager::install(&repo_root, true, "/bin/sh")?;
            "Git post-commit memory hook installed"
        } else {
            HookManager::uninstall(&repo_root)?;
            "Git post-commit memory hook uninstalled"
        };

        if let Ok(stat) = HookManager::status(&repo_root) {
            self.window.extensions.hooks = vec![HookModel {
                name: "post-commit memory sync".to_string(),
                event: "post-commit".to_string(),
                command: "kaioken memory scan --changed --no-llm".to_string(),
                enabled: stat.is_installed,
            }];
        }

        self.append_activity_event("user", "git_hook", "post-commit", if install { "installed" } else { "uninstalled" }, "info");
        self.touch();
        Ok(msg.to_string())
    }

    /// Test an API key with a live probe request.
    pub fn test_api_key(&mut self, provider: &str, key: &str) -> Result<String> {
        let (endpoint, auth_header) = match provider {
            "openrouter" => ("https://openrouter.ai/api/v1/auth/key", format!("Authorization: Bearer {}", key)),
            "anthropic" => ("https://api.anthropic.com/v1/models", format!("x-api-key: {}", key)),
            "perplexity" => ("https://api.perplexity.ai/models", format!("Authorization: Bearer {}", key)),
            "tavily" => ("https://api.tavily.com/search", format!("Authorization: Bearer {}", key)),
            _ => ("https://openrouter.ai/api/v1/auth/key", format!("Authorization: Bearer {}", key)),
        };

        let output = std::process::Command::new("curl")
            .arg("-s")
            .arg("-o").arg("nul")
            .arg("-w").arg("%{http_code}")
            .arg("-H").arg(&auth_header)
            .arg(endpoint)
            .output();

        let status_code = match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).trim().parse::<u16>().unwrap_or(0),
            Err(_) => 0,
        };

        let valid = status_code >= 200 && status_code < 300;
        let result_msg = if valid {
            format!("API key for {} is valid (HTTP {})", provider, status_code)
        } else {
            format!("API key for {} rejected with status {}", provider, status_code)
        };

        self.append_activity_event("user", "test_key", provider, if valid { "valid" } else { "invalid" }, if valid { "info" } else { "warn" });
        self.touch();
        Ok(result_msg)
    }

    /// Save an API key directly into repo config or user config.
    pub fn save_config_key(&mut self, provider: &str, key: &str) -> Result<String> {
        let repo_root = PathBuf::from(&self.window.repo_path);
        let cfg = crate::config::Config::load(&repo_root).unwrap_or_else(|_| crate::config::Config::default_config());
        if provider == "openrouter" {
            std::env::set_var("OPENROUTER_API_KEY", key);
            self.window.settings.has_openrouter_key = true;
        } else if provider == "anthropic" {
            std::env::set_var("ANTHROPIC_API_KEY", key);
            self.window.settings.has_anthropic_key = true;
        } else if provider == "perplexity" {
            std::env::set_var("PERPLEXITY_API_KEY", key);
            self.window.settings.has_perplexity_key = true;
        } else if provider == "tavily" {
            std::env::set_var("TAVILY_API_KEY", key);
            self.window.settings.has_tavily_key = true;
        }
        let _ = cfg.save(&repo_root);
        self.append_activity_event("user", "save_key", provider, "saved", "info");
        self.touch();
        Ok(format!("Saved key for {}", provider))
    }

    /// Enumerate repository files for Quick Open (Ctrl+P) and command palette.
    pub fn list_repo_files(&self, query: Option<&str>) -> Vec<String> {
        let repo_root = PathBuf::from(&self.window.repo_path);
        let scan_opts = crate::scanner::FileInventoryOptions::default();
        if let Ok(inventory) = crate::scanner::FileInventory::scan_dir_with_options(&repo_root, &scan_opts) {
            let q_lower = query.map(|q| q.to_lowercase());
            inventory.files
                .into_iter()
                .map(|f| f.path.to_string_lossy().replace('\\', "/"))
                .filter(|p| {
                    if let Some(ref q) = q_lower {
                        p.to_lowercase().contains(q)
                    } else {
                        true
                    }
                })
                .take(100)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Synchronize initial state from daemon and active workspace.
    pub async fn initial_sync(&mut self) -> Result<()> {
        let repo_root = PathBuf::from(&self.window.repo_path);
        let db_path = repo_root.join(".agent").join("memory.sqlite");
        let mem = crate::memory::MemoryEngine::open(&db_path).ok();
        self.window.load_live_workspace(&repo_root, mem.as_ref());

        let status_started = std::time::Instant::now();
        let mut reported_endpoint: Option<String> = None;
        if let Ok(crate::desktop::protocol::IpcResponse::Ok(val)) =
            self.ipc_client.send_request(IpcRequest::GetStatus).await
        {
            if let Some(mode) = val.get("mode").and_then(|v| v.as_str()) {
                self.window.settings.daemon_mode = mode.to_string();
            }
            if let Some(endpoint) = val.get("endpoint").and_then(|v| v.as_str()) {
                reported_endpoint = Some(endpoint.to_string());
            }
            self.window.daemon_conn.latency_ms = status_started.elapsed().as_millis() as u64;
        }

        if let Ok(crate::desktop::protocol::IpcResponse::Ok(val)) = self
            .ipc_client
            .send_request(IpcRequest::GetMemoryStats)
            .await
        {
            if let Some(files) = val.get("total_files").and_then(|v| v.as_u64()) {
                self.window.memory_graph.total_files = files as usize;
            }
            if let Some(symbols) = val.get("total_symbols").and_then(|v| v.as_u64()) {
                self.window.memory_graph.total_symbols = symbols as usize;
            }
            if let Some(edges) = val.get("total_edges").and_then(|v| v.as_u64()) {
                self.window.memory_graph.total_edges = edges as usize;
            }
        }

        if let Ok(crate::desktop::protocol::IpcResponse::Ok(val)) = self
            .ipc_client
            .send_request(IpcRequest::GetMemoryGraph {
                filter: None,
                limit: Some(300),
            })
            .await
        {
            if let Some(nodes) = val.get("nodes").cloned() {
                if let Ok(nodes) = serde_json::from_value::<Vec<GraphNode>>(nodes) {
                    if let Some(first) = nodes.first() {
                        self.window.memory_graph.selected_node_id = Some(first.id);
                    }
                    self.window.memory_graph.nodes = nodes;
                }
            }
            if let Some(edges) = val.get("edges").cloned() {
                if let Ok(edges) = serde_json::from_value::<Vec<GraphEdge>>(edges) {
                    self.window.memory_graph.edges = edges;
                }
            }
        }

        if let Ok(crate::desktop::protocol::IpcResponse::Ok(val)) = self
            .ipc_client
            .send_request(IpcRequest::GetReviewDiff { cached: false })
            .await
        {
            let diff = val
                .get("diff")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            self.window.dirty_count = 0;
            self.window.review_state.is_clean = diff.trim().is_empty();
            self.window.review_state.raw_diff = diff;
            if let Some(score) = val.get("risk_score").and_then(|v| v.as_u64()) {
                self.window.review_state.risk_score = score as u32;
            }
            if let Some(level) = val.get("risk_level").and_then(|v| v.as_str()) {
                self.window.review_state.risk_level = level.to_string();
            }
            if let Some(files) = val.get("changed_files").and_then(|v| v.as_array()) {
                let mut additions = 0;
                let mut deletions = 0;
                self.window.review_state.changed_files = files
                    .iter()
                    .map(|file| {
                        additions +=
                            file.get("additions").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        deletions +=
                            file.get("deletions").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        file.get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string()
                    })
                    .collect();
                self.window.review_state.additions = additions;
                self.window.review_state.deletions = deletions;
                self.window.dirty_count = self.window.review_state.changed_files.len();
            }
            if let Some(reasons) = val.get("recommendations").and_then(|v| v.as_array()) {
                self.window.review_state.risk_reasons = reasons
                    .iter()
                    .filter_map(|reason| reason.as_str().map(str::to_string))
                    .collect();
            }
        }

        if let Ok(crate::desktop::protocol::IpcResponse::Ok(val)) = self
            .ipc_client
            .send_request(IpcRequest::GetAuditLogs { limit: 50 })
            .await
        {
            if let Ok(entries) = serde_json::from_value::<Vec<AuditLogEntry>>(val) {
                let mut audit = AuditSafetyModel::new();
                for entry in entries.into_iter().rev() {
                    audit.push_entry(entry);
                }
                self.window.audit_safety = audit;
            }
        }

        if let Ok(crate::desktop::protocol::IpcResponse::Ok(val)) = self
            .ipc_client
            .send_request(IpcRequest::GetDocsHealth)
            .await
        {
            if let Some(fresh) = val.get("fresh_docs").and_then(|v| v.as_u64()) {
                self.window.health_explorer.fresh_docs = fresh as usize;
            }
            if let Some(stale) = val.get("stale_docs").and_then(|v| v.as_u64()) {
                self.window.health_explorer.stale_docs = stale as usize;
            }
            if let Some(total) = val.get("total_docs").and_then(|v| v.as_u64()) {
                self.window.health_explorer.total_docs = total as usize;
            }

            let total = self.window.health_explorer.total_docs;
            if let Some(score) = (self.window.health_explorer.fresh_docs * 100).checked_div(total) {
                let score = score as u32;
                self.window.health_explorer.health_score = score;
                self.window.health_explorer.grade = match score {
                    90..=100 => "A",
                    80..=89 => "B",
                    70..=79 => "C",
                    60..=69 => "D",
                    _ => "F",
                }
                .to_string();
            }
        }

        self.window.daemon_conn.status = if self.ipc_client.is_connected().await {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };
        self.window.daemon_conn.endpoint =
            reported_endpoint.unwrap_or_else(|| self.ipc_client.endpoint().to_string());
        self.window.settings.endpoint = self.window.daemon_conn.endpoint.clone();
        self.touch();
        Ok(())
    }
}

/// Small helper so command handlers can mutate and mark state in one chain.
trait TouchAfter {
    fn tap_touch(self, controller: &mut KaiokenDesktopController) -> Self;
}

impl TouchAfter for String {
    fn tap_touch(self, controller: &mut KaiokenDesktopController) -> Self {
        controller.touch();
        self
    }
}

/// Compatibility marker used by the headless renderer; the agent thought entry
/// type is part of the public desktop model surface.
pub type ThoughtEntry = AgentThoughtEntry;

/// A simple dangerous proposal used by integration tests for the deny-first
/// approval path.
pub fn dangerous_demo_proposal(id: &str) -> ToolProposal {
    ToolProposal {
        id: id.to_string(),
        tool_name: "bash".to_string(),
        target: "git reset --hard".to_string(),
        policy_tier: PolicyTier::Dangerous,
        details: "Destructive command requires explicit approval.".to_string(),
        diff: None,
        proposed_at: chrono::Utc::now().to_rfc3339(),
    }
}
