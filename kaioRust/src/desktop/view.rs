use crate::desktop::model::{
    ActiveWorkspaceTab, ActivityEventModel, ActivityRunModel, ActivityWorkspaceModel,
    AgentSessionModel, AuditSafetyModel, BrowserTabModel, BrowserWorkspaceModel, ChatMessage,
    ChatSessionMeta, CommandPaletteModel, CostBucketModel, CostTelemetryModel,
    DaemonConnectionModel, EditorFileModel, EditorWorkspaceModel, ExtensionModel,
    ExtensionsWorkspaceModel, FailureCardModel, HealthExplorerModel, HookModel, MemoryGraphModel,
    ResearchCitationModel, ResearchDossierModel, ResearchHistoryModel, ResearchSourceModel,
    ResearchStepModel, ReviewStateModel, SettingsModel, TerminalLineModel, WikiChapterModel,
    WikiWorkspaceModel,
};
use crate::desktop::protocol::{PolicyTier, ToolProposal};
use crate::desktop::theme::DesktopTheme;
use crate::config::Config;
use crate::hook::HookManager;
use crate::memory::MemoryEngine;
use crate::review::{ReviewArgs, ReviewEngine};
use crate::safety::audit::AuditLogEntry;
use serde_json::{json, Value};
use std::path::Path;

/// High-level desktop application window state and renderer (Section 4.1).
#[derive(Debug, Clone)]
pub struct KaiokenAppWindow {
    pub repo_name: String,
    pub repo_path: String,
    pub branch: String,
    pub dirty_count: usize,
    pub active_tab: ActiveWorkspaceTab,
    pub theme: DesktopTheme,

    // Reactive Models
    pub agent_session: AgentSessionModel,
    pub memory_graph: MemoryGraphModel,
    pub review_state: ReviewStateModel,
    pub audit_safety: AuditSafetyModel,
    pub health_explorer: HealthExplorerModel,
    pub daemon_conn: DaemonConnectionModel,
    pub command_palette: CommandPaletteModel,
    pub research: ResearchDossierModel,
    pub wiki: WikiWorkspaceModel,
    pub failure_cards: Vec<FailureCardModel>,
    pub editor: EditorWorkspaceModel,
    pub browser: BrowserWorkspaceModel,
    pub activity: ActivityWorkspaceModel,
    pub extensions: ExtensionsWorkspaceModel,
    pub cost: CostTelemetryModel,
    pub settings: SettingsModel,
}

impl KaiokenAppWindow {
    pub fn new(repo_name: impl Into<String>, branch: impl Into<String>) -> Self {
        let repo_name = repo_name.into();
        let branch = branch.into();
        let mut window = Self {
            repo_path: format!("~/repos/{}", repo_name),
            repo_name,
            branch,
            dirty_count: 3,
            active_tab: ActiveWorkspaceTab::Chat,
            theme: DesktopTheme::dark(),
            agent_session: AgentSessionModel::new(),
            memory_graph: MemoryGraphModel::new(),
            review_state: ReviewStateModel::default(),
            audit_safety: AuditSafetyModel::new(),
            health_explorer: HealthExplorerModel::default(),
            daemon_conn: DaemonConnectionModel {
                status: crate::desktop::model::ConnectionStatus::Connected,
                endpoint: "embedded in-process".to_string(),
                latency_ms: 1,
                last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
            },
            command_palette: CommandPaletteModel {
                actions: CommandPaletteModel::default_actions(),
                ..Default::default()
            },
            research: ResearchDossierModel::default(),
            wiki: WikiWorkspaceModel::default(),
            failure_cards: Vec::new(),
            editor: EditorWorkspaceModel::default(),
            browser: BrowserWorkspaceModel::default(),
            activity: ActivityWorkspaceModel::default(),
            extensions: ExtensionsWorkspaceModel::default(),
            cost: CostTelemetryModel::default(),
            settings: SettingsModel::default(),
        };
        window.seed_showcase();
        window
    }

    /// Switch active workspace tab.
    pub fn set_tab(&mut self, tab: ActiveWorkspaceTab) {
        self.active_tab = tab;
    }

    /// Toggle global command palette.
    pub fn toggle_command_palette(&mut self) {
        self.command_palette.is_open = !self.command_palette.is_open;
        if self.command_palette.is_open {
            self.command_palette.query.clear();
            self.command_palette.selected_index = 0;
        }
    }

    pub fn theme_mode(&self) -> &'static str {
        if self.theme.name.to_ascii_lowercase().contains("light") {
            "light"
        } else {
            "dark"
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme = if self.theme_mode() == "light" {
            DesktopTheme::dark()
        } else {
            DesktopTheme::light()
        };
        self.settings.theme = self.theme_mode().to_string();
    }

    /// Compact the visible transcript, keeping the most recent turns.
    pub fn compact_session(&mut self) -> usize {
        let before = self.agent_session.messages.len();
        let keep_from = before.saturating_sub(6);
        self.agent_session.messages = self.agent_session.messages[keep_from..].to_vec();
        self.agent_session.push_thought(format!(
            "Compacted conversation: {} older turns summarized out of the context window.",
            keep_from
        ));
        before.saturating_sub(self.agent_session.messages.len())
    }

    /// Start a fresh session while keeping the previous one in the rail.
    pub fn new_chat_session(&mut self) {
        if let Some(active) = self.agent_session.active_session_id.take() {
            if let Some(meta) = self
                .agent_session
                .sessions
                .iter_mut()
                .find(|s| s.id == active)
            {
                meta.turns = self.agent_session.messages.len();
                meta.updated = "just now".to_string();
            }
        }
        let id = format!("session-{}", self.agent_session.sessions.len() + 1);
        self.agent_session.sessions.insert(
            0,
            ChatSessionMeta {
                id: id.clone(),
                title: "New session".to_string(),
                turns: 0,
                updated: "just now".to_string(),
            },
        );
        self.agent_session.active_session_id = Some(id);
        self.agent_session.messages.clear();
        self.agent_session.pending_proposals.clear();
        self.agent_session.current_task = None;
        self.agent_session.is_running = false;
    }

    /// Load live repository context, files, documentation, and diagnostics from the active workspace.
    pub fn load_live_workspace(&mut self, repo_root: &Path, memory: Option<&MemoryEngine>) {
        self.repo_path = repo_root.display().to_string();
        if let Some(name) = repo_root.file_name().and_then(|n| n.to_str()) {
            self.repo_name = name.to_string();
        }

        // 1. Git branch and dirty count
        if let Ok(branch_out) = std::process::Command::new("git")
            .arg("-C").arg(repo_root)
            .arg("rev-parse").arg("--abbrev-ref").arg("HEAD")
            .output()
        {
            if branch_out.status.success() {
                let b = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();
                if !b.is_empty() {
                    self.branch = b;
                }
            }
        }
        if let Ok(changed) = crate::scanner::git::GitScanner::get_all_changed_files(repo_root) {
            self.dirty_count = changed.len();
        }

        // 2. Real Wiki Chapters from docs/
        let docs_dir = repo_root.join("docs");
        if docs_dir.is_dir() {
            let mut chapters = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&docs_dir) {
                let mut paths: Vec<_> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |ext| ext == "md"))
                    .collect();
                paths.sort();
                for p in paths {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        let rel_path = p.strip_prefix(repo_root)
                            .map(|r| r.to_string_lossy().replace('\\', "/"))
                            .unwrap_or_else(|_| p.to_string_lossy().replace('\\', "/"));
                        let title = content.lines()
                            .find(|line| line.starts_with("# "))
                            .map(|line| line.trim_start_matches("# ").trim().to_string())
                            .unwrap_or_else(|| p.file_stem().and_then(|s| s.to_str()).unwrap_or("Doc").to_string());
                        let words = content.split_whitespace().count();
                        let reading_minutes = (words / 200).max(1);
                        let section = if rel_path.starts_with("docs/0") || rel_path.starts_with("docs/1") {
                            "Architecture"
                        } else if rel_path.contains("report") || rel_path.contains("quality") {
                            "Reports"
                        } else {
                            "Guides"
                        }.to_string();
                        chapters.push(WikiChapterModel {
                            title,
                            path: rel_path,
                            section,
                            markdown: content,
                            words,
                            reading_minutes,
                        });
                    }
                }
            }
            if !chapters.is_empty() {
                self.wiki.selected_path = chapters[0].path.clone();
                self.wiki.chapters = chapters;
            }
        }

        // 3. Real Review & Diff
        if let Some(mem) = memory {
            if let Ok(report) = ReviewEngine::review(repo_root, mem, &ReviewArgs::default()) {
                let diff = ReviewEngine::get_diff(repo_root, &ReviewArgs::default()).unwrap_or_default();
                self.review_state = ReviewStateModel::from_report(&report, &diff);
                self.editor.diff = diff;
                self.editor.diff_summary = format!(
                    "+{} −{} across {} files · risk {}/100 ({})",
                    report.changed_files.iter().map(|f| f.additions).sum::<usize>(),
                    report.changed_files.iter().map(|f| f.deletions).sum::<usize>(),
                    report.changed_files.len(),
                    (report.risk.score * 100.0) as u32,
                    report.risk.level.as_str()
                );
            }
        } else if let Ok(diff) = ReviewEngine::get_diff(repo_root, &ReviewArgs::default()) {
            self.review_state.raw_diff = diff.clone();
            self.editor.diff = diff;
        }

        // 4. Real files into Editor
        let mut editor_files = Vec::new();
        for candidate in ["src/main.rs", "src/lib.rs", "Cargo.toml", "README.md", "src/desktop/app.rs"] {
            let p = repo_root.join(candidate);
            if p.is_file() {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    let lang = if candidate.ends_with(".rs") {
                        "Rust"
                    } else if candidate.ends_with(".toml") {
                        "TOML"
                    } else if candidate.ends_with(".md") {
                        "Markdown"
                    } else {
                        "Plaintext"
                    };
                    editor_files.push(EditorFileModel {
                        path: candidate.to_string(),
                        language: lang.to_string(),
                        content: content.clone(),
                        saved: content,
                        truncated: false,
                    });
                }
            }
        }
        if !editor_files.is_empty() {
            self.editor.active_path = editor_files[0].path.clone();
            self.editor.files = editor_files;
        }

        // 5. Real Failure Cards
        if let Some(mem) = memory {
            if let Ok(failures) = mem.get_recent_failures(50) {
                if !failures.is_empty() {
                    self.failure_cards = failures.into_iter().map(|f| FailureCardModel {
                        id: format!("failure-{:03}", f.id.unwrap_or(0)),
                        title: f.message,
                        module: f.error_type.clone(),
                        severity: "medium".to_string(),
                        occurrences: 1,
                        status: f.status.unwrap_or_else(|| "unresolved".to_string()),
                        last_seen: f.created_at,
                        gotchas: vec![format!("Error type: {}", f.error_type)],
                        anti_pattern_rules: f.fix_summary.map(|s| vec![s]).unwrap_or_default(),
                    }).collect();
                }
            }
        }

        // 6. Real Git Hook Status
        if let Ok(hook_stat) = HookManager::status(repo_root) {
            self.extensions.hooks = vec![HookModel {
                name: "post-commit memory sync".to_string(),
                event: "post-commit".to_string(),
                command: "kaioken memory scan --changed --no-llm".to_string(),
                enabled: hook_stat.is_installed,
            }];
        }

        // 7. Real Config / Settings
        if let Ok(cfg) = Config::load(repo_root) {
            self.settings.permission_mode = cfg.permissions.mode.to_string();
            self.settings.model = cfg.llm.coder_model;
        }
        self.settings.has_openrouter_key = std::env::var("OPENROUTER_API_KEY").is_ok();
        self.settings.has_anthropic_key = std::env::var("ANTHROPIC_API_KEY").is_ok();
        self.settings.has_perplexity_key = std::env::var("PERPLEXITY_API_KEY").is_ok();
        self.settings.has_tavily_key = std::env::var("TAVILY_API_KEY").is_ok();
    }

    /// Produce the complete serializable state consumed by the WebView.
    pub fn snapshot_json(&self) -> Value {
        let messages = self
            .agent_session
            .messages
            .iter()
            .map(|message| match message {
                ChatMessage::User { content, timestamp } => {
                    json!({ "type": "user", "content": content, "timestamp": timestamp })
                }
                ChatMessage::AgentThought { content, timestamp } => {
                    json!({ "type": "thought", "content": content, "timestamp": timestamp })
                }
                ChatMessage::ToolProposalCard(proposal) => {
                    json!({ "type": "proposal", "proposal": proposal })
                }
                ChatMessage::ToolResolution {
                    proposal_id,
                    approved,
                    output,
                    timestamp,
                } => json!({
                    "type": "result",
                    "proposal_id": proposal_id,
                    "approved": approved,
                    "success": approved,
                    "output": output,
                    "timestamp": timestamp,
                }),
                ChatMessage::AgentResponse { content, timestamp } => {
                    json!({ "type": "response", "content": content, "timestamp": timestamp })
                }
            })
            .collect::<Vec<_>>();

        let audit_entries = self
            .audit_safety
            .entries
            .iter()
            .map(|entry| {
                json!({
                    "id": entry.id,
                    "actor": entry.actor,
                    "action": entry.action,
                    "target": entry.target,
                    "approved": entry.approved,
                    "reason": entry.reason,
                    "severity": entry.severity(),
                    "created_at": entry.created_at,
                })
            })
            .collect::<Vec<_>>();

        json!({
            "active_tab": self.active_tab.id(),
            "theme": self.theme_mode(),
            "repo": {
                "name": self.repo_name,
                "path": self.repo_path,
                "branch": self.branch,
                "dirty_count": self.dirty_count,
                "is_repo": true,
            },
            "daemon": {
                "status": match self.daemon_conn.status {
                    crate::desktop::model::ConnectionStatus::Connected => "connected",
                    crate::desktop::model::ConnectionStatus::Connecting => "connecting",
                    crate::desktop::model::ConnectionStatus::Reconnecting(_) => "reconnecting",
                    crate::desktop::model::ConnectionStatus::Disconnected => "disconnected",
                },
                "mode": self.settings.daemon_mode,
                "version": "0.1.0",
                "endpoint": self.daemon_conn.endpoint,
                "port": self
                    .daemon_conn
                    .endpoint
                    .rsplit(':')
                    .next()
                    .and_then(|port| port.parse::<u16>().ok())
                    .unwrap_or(47890),
                "latency_ms": self.daemon_conn.latency_ms,
                "last_heartbeat": self.daemon_conn.last_heartbeat,
            },
            "agent": {
                "current_task": self.agent_session.current_task,
                "is_running": self.agent_session.is_running,
                "total_tool_calls": self.agent_session.total_tool_calls,
                "total_approved": self.agent_session.total_approved,
                "total_denied": self.agent_session.total_denied,
                "active_session_id": self.agent_session.active_session_id,
                "sessions": self.agent_session.sessions,
                "pending_proposals": self.agent_session.pending_proposals,
                "messages": messages,
            },
            "memory": {
                "total_files": self.memory_graph.total_files,
                "total_symbols": self.memory_graph.total_symbols,
                "total_edges": self.memory_graph.total_edges,
                "nodes": self.memory_graph.nodes,
                "edges": self.memory_graph.edges,
                "selected_node_id": self.memory_graph.selected_node_id,
                "inspector": self.memory_graph.inspector_data,
                "filter_query": self.memory_graph.filter_query,
                "zoom_level": self.memory_graph.zoom_level,
            },
            "review": {
                "changed_files": self.review_state.changed_files,
                "additions": self.review_state.additions,
                "deletions": self.review_state.deletions,
                "risk_score": self.review_state.risk_score,
                "risk_level": self.review_state.risk_level,
                "risk_reasons": self.review_state.risk_reasons,
                "diff": self.review_state.raw_diff,
                "is_clean": self.review_state.is_clean,
            },
            "health": {
                "score": self.health_explorer.health_score,
                "grade": self.health_explorer.grade,
                "total_docs": self.health_explorer.total_docs,
                "fresh_docs": self.health_explorer.fresh_docs,
                "stale_docs": self.health_explorer.stale_docs,
                "missing_docs": self.health_explorer.missing_docs,
            },
            "audit": {
                "entries": audit_entries,
                "total_intercepted": self.audit_safety.total_intercepted,
                "total_redacted": self.audit_safety.total_redacted,
            },
            "research": self.research,
            "wiki": self.wiki,
            "failure_cards": self.failure_cards,
            "editor": {
                "files": self.editor.files.iter().map(|file| json!({
                    "path": file.path,
                    "language": file.language,
                    "content": file.content,
                    "saved": file.saved,
                    "dirty": file.dirty(),
                    "truncated": file.truncated,
                })).collect::<Vec<_>>(),
                "active_path": self.editor.active_path,
                "diff": self.editor.diff,
                "diff_summary": self.editor.diff_summary,
                "terminal_open": self.editor.terminal_open,
                "terminal_lines": self.editor.terminal_lines,
            },
            "browser": self.browser,
            "activity": self.activity,
            "extensions": self.extensions,
            "cost": self.cost,
            "settings": self.settings,
            "tokens": {
                "total": self.cost.total_tokens(),
                "prompt": self.cost.prompt_tokens,
                "completion": self.cost.completion_tokens,
                "calls": self.cost.calls,
                "usd": self.cost.total_usd,
            },
            "palette_actions": self.command_palette.actions,
            "routes": ActiveWorkspaceTab::all().iter().map(|tab| json!({
                "id": tab.id(),
                "label": tab.label(),
                "icon": tab.icon(),
                "shortcut": tab.shortcut(),
                "group": tab.group(),
            })).collect::<Vec<_>>(),
        })
    }

    /// Render a high-density textual / terminal snapshot of the UI state (used for tests & headless view).
    pub fn render_terminal_frame(&self) -> String {
        let mut out = String::new();

        let conn_status = match self.daemon_conn.status {
            crate::desktop::model::ConnectionStatus::Connected => "connected",
            crate::desktop::model::ConnectionStatus::Connecting => "connecting",
            crate::desktop::model::ConnectionStatus::Reconnecting(_n) => "reconnecting",
            crate::desktop::model::ConnectionStatus::Disconnected => "disconnected",
        };
        let theme = self.theme_mode();
        let tokens = self.cost.total_tokens();

        out.push_str(&format!(
            "┌─ KAIOKEN ─ {}/{} · git ·{} ── {} · {} ── Σ {} tok ─┐\n",
            self.repo_name, self.branch, self.dirty_count, conn_status, theme, tokens
        ));

        let tab_rows: Vec<String> = ActiveWorkspaceTab::all()
            .iter()
            .map(|tab| {
                let marker = if self.active_tab == *tab { "►" } else { " " };
                let shortcut = if tab.shortcut().is_empty() {
                    "      "
                } else {
                    tab.shortcut()
                };
                format!(
                    "{} {} {:<12} {:<7}",
                    marker,
                    tab.emoji(),
                    tab.label(),
                    shortcut
                )
            })
            .collect();

        let content_lines = self.render_workspace_lines();

        out.push_str("│ ┌─ Navigation ──────────────────┐ ┌─ Workspace ────────────────────────────────────────┐ │\n");
        for i in 0..15 {
            let sidebar_part = if i < tab_rows.len() {
                format!("{:<32}", tab_rows[i])
            } else if i == 11 {
                "─".repeat(32)
            } else if i == 12 {
                "memory".to_string()
            } else if i == 13 {
                format!(
                    "{} files · {} sym · {} edges",
                    self.memory_graph.total_files,
                    self.memory_graph.total_symbols,
                    self.memory_graph.total_edges
                )
            } else if i == 14 {
                format!(
                    "health {}/100 · {} stale",
                    self.health_explorer.health_score, self.health_explorer.stale_docs
                )
            } else {
                String::new()
            };

            let content_part = content_lines
                .get(i)
                .map(|line| truncate_cells(line, 54))
                .unwrap_or_default();
            out.push_str(&format!(
                "│ │ {:<32}│ │ {:<54} │ │\n",
                sidebar_part, content_part
            ));
        }

        if self.command_palette.is_open {
            out.push_str("│ ┌─ Command Palette [Esc to close] ────────────────────────────────────────────┐ │\n");
            out.push_str(&format!(
                "│ │ > {:<76}│ │\n",
                truncate_cells(&self.command_palette.query, 76)
            ));
            for (idx, action) in self
                .command_palette
                .filtered_actions()
                .iter()
                .take(4)
                .enumerate()
            {
                let pointer = if idx == self.command_palette.selected_index {
                    "►"
                } else {
                    " "
                };
                let line = format!(
                    "{} [{}] {} {}",
                    pointer,
                    action.category,
                    action.title,
                    action.shortcut.as_deref().unwrap_or("")
                );
                out.push_str(&format!("│ │ {:<76}│ │\n", truncate_cells(&line, 76)));
            }
            out.push_str("│ └──────────────────────────────────────────────────────────────────────────────┘ │\n");
        }

        out.push_str("└────────────────────────────────┴──────────────────────────────────────────────────────┘\n");
        out.push_str("  [Ctrl+K] Palette | [Ctrl+1..9] Routes | [Ctrl+,] Settings | [Enter] Approve | [Esc] Deny\n");

        out
    }

    fn render_workspace_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        match self.active_tab {
            ActiveWorkspaceTab::Chat => {
                lines.push(format!(
                    "Session: {} · {} messages · {} pending",
                    self.agent_session
                        .active_session_id
                        .as_deref()
                        .unwrap_or("none"),
                    self.agent_session.messages.len(),
                    self.agent_session.pending_proposals.len()
                ));
                for message in self.agent_session.messages.iter().rev().take(4).rev() {
                    match message {
                        ChatMessage::User { content, .. } => {
                            lines.push(format!("you  > {}", truncate_cells(content, 54)))
                        }
                        ChatMessage::AgentThought { content, .. } => {
                            lines.push(format!("think> {}", truncate_cells(content, 54)))
                        }
                        ChatMessage::ToolProposalCard(p) => lines.push(format!(
                            "tool > {} · {} · {}",
                            p.tool_name, p.policy_tier, p.target
                        )),
                        ChatMessage::ToolResolution {
                            approved, output, ..
                        } => lines.push(format!(
                            "{} > {}",
                            if *approved { "ok" } else { "deny" },
                            truncate_cells(output, 52)
                        )),
                        ChatMessage::AgentResponse { content, .. } => {
                            lines.push(format!("kaito> {}", truncate_cells(content, 54)))
                        }
                    }
                }
                if let Some(proposal) = self.agent_session.pending_proposals.first() {
                    lines.push(format!(
                        "APPROVAL · {} · {} · [Enter] approve [Esc] deny",
                        proposal.tool_name, proposal.target
                    ));
                }
            }
            ActiveWorkspaceTab::Research => {
                lines.push(format!(
                    "Research ×{} · {} · {} rounds · {} sources",
                    self.research.power_level,
                    self.research.path,
                    self.research.rounds,
                    self.research.sources.len()
                ));
                lines.push(format!("Q: {}", truncate_cells(&self.research.query, 56)));
                for step in self.research.steps.iter().take(5) {
                    lines.push(format!("{:>7} · {}", step.status, step.label));
                }
                lines.push(format!(
                    "grounded {}/{} · ≈${:.4} · {} ms",
                    self.research.grounded,
                    self.research.total_claims,
                    self.research.cost_usd,
                    self.research.duration_ms
                ));
            }
            ActiveWorkspaceTab::Wiki => {
                lines.push(format!(
                    "Living codebase documentation · {} chapters · {} docs",
                    self.wiki.chapters.len(),
                    self.health_explorer.total_docs
                ));
                for chapter in self.wiki.chapters.iter().take(5) {
                    let marker = if chapter.path == self.wiki.selected_path {
                        "►"
                    } else {
                        " "
                    };
                    lines.push(format!(
                        "{} [{}] {} · {}",
                        marker, chapter.section, chapter.title, chapter.path
                    ));
                }
                if let Some(chapter) = self
                    .wiki
                    .chapters
                    .iter()
                    .find(|c| c.path == self.wiki.selected_path)
                {
                    lines.push(format!(
                        "reading {} · {} words · {} min",
                        chapter.title, chapter.words, chapter.reading_minutes
                    ));
                }
            }
            ActiveWorkspaceTab::Graph => {
                lines.push("AST symbol dependency topology".to_string());
                lines.push(format!(
                    "{} nodes · {} edges · zoom {:.1}x",
                    self.memory_graph.nodes.len(),
                    self.memory_graph.edges.len(),
                    self.memory_graph.zoom_level
                ));
                for node in self.memory_graph.nodes.iter().take(5) {
                    let marker = if Some(node.id) == self.memory_graph.selected_node_id {
                        "►"
                    } else {
                        " "
                    };
                    lines.push(format!(
                        "{} [{:>6}] {:<22} {}",
                        marker, node.kind, node.name, node.file_path
                    ));
                }
                if let Some(inspector) = &self.memory_graph.inspector_data {
                    lines.push(format!(
                        "inspect {} · callers {} · deps {}",
                        inspector.name,
                        inspector.callers.len(),
                        inspector.dependencies.len()
                    ));
                }
            }
            ActiveWorkspaceTab::Cards => {
                lines.push(format!(
                    "Failure memory · {} cards · deterministic prevention",
                    self.failure_cards.len()
                ));
                for card in self.failure_cards.iter().take(4) {
                    lines.push(format!(
                        "[{:<6}] {:<30} {} · {}x",
                        card.severity, card.title, card.module, card.occurrences
                    ));
                }
                if let Some(card) = self.failure_cards.first() {
                    if let Some(gotcha) = card.gotchas.first() {
                        lines.push(format!("gotcha: {}", truncate_cells(gotcha, 50)));
                    }
                }
            }
            ActiveWorkspaceTab::Editor => {
                let file = self
                    .editor
                    .files
                    .iter()
                    .find(|f| f.path == self.editor.active_path);
                lines.push(format!(
                    "{} · {} open files · {}",
                    self.editor.active_path,
                    self.editor.files.len(),
                    file.map(|f| f.language.as_str()).unwrap_or("text")
                ));
                lines.push(self.editor.diff_summary.clone());
                if let Some(file) = file {
                    for (idx, code_line) in file.content.lines().skip(4).take(7).enumerate() {
                        lines.push(format!(
                            "{:>4} │ {}",
                            idx + 5,
                            truncate_cells(code_line, 52)
                        ));
                    }
                }
                lines.push(format!(
                    "terminal: {} · {} lines",
                    if self.editor.terminal_open {
                        "open"
                    } else {
                        "collapsed"
                    },
                    self.editor.terminal_lines.len()
                ));
            }
            ActiveWorkspaceTab::Browser => {
                lines.push(format!(
                    "{} · {} tabs · {} {} · {} · {}ms",
                    self.browser.active_id,
                    self.browser.tabs.len(),
                    self.browser.method,
                    self.browser.request_url,
                    self.browser.status,
                    self.browser.latency_ms
                ));
                for tab in self.browser.tabs.iter().take(3) {
                    let marker = if tab.id == self.browser.active_id {
                        "►"
                    } else {
                        " "
                    };
                    lines.push(format!("{} [{}] {}", marker, tab.kind, tab.title));
                }
                lines.push("response headers".to_string());
                for (name, value) in self.browser.response_headers.iter().take(3) {
                    lines.push(format!("  {}: {}", name, value));
                }
            }
            ActiveWorkspaceTab::Activity => {
                lines.push("Task queues · background daemon · safety ledger".to_string());
                for run in self.activity.runs.iter().take(3) {
                    lines.push(format!(
                        "{:<9} {:<10} {:>3}/{:<3} {}",
                        run.state,
                        run.label,
                        run.progress_done,
                        run.progress_total,
                        truncate_cells(&run.message, 28)
                    ));
                }
                lines.push(format!(
                    "audit · {} events · {} intercepted · {} redacted",
                    self.audit_safety.entries.len(),
                    self.audit_safety.total_intercepted,
                    self.audit_safety.total_redacted
                ));
                for event in self.activity.events.iter().take(3) {
                    lines.push(format!(
                        "[{}] {:<9} {:<10} {}",
                        event.severity, event.action, event.decision, event.target
                    ));
                }
            }
            ActiveWorkspaceTab::Extensions => {
                lines.push(format!(
                    "Tool registry · {} installed · {} hooks",
                    self.extensions.installed.len(),
                    self.extensions.hooks.len()
                ));
                for ext in self.extensions.installed.iter().take(4) {
                    lines.push(format!(
                        "{:<18} v{:<7} {:<12} {:<9} {}",
                        ext.id,
                        ext.version,
                        ext.kind,
                        if ext.trusted { "trusted" } else { "untrusted" },
                        if ext.enabled { "enabled" } else { "disabled" }
                    ));
                }
                for hook in self.extensions.hooks.iter().take(2) {
                    lines.push(format!(
                        "hook {:<12} {:<18} {}",
                        hook.event, hook.name, hook.command
                    ));
                }
            }
            ActiveWorkspaceTab::Cost => {
                lines.push(format!(
                    "Spend ≈${:.4} · {} calls · Σ {} tokens",
                    self.cost.total_usd,
                    self.cost.calls,
                    self.cost.total_tokens()
                ));
                lines.push(format!(
                    "{} prompt · {} completion · {} local calls",
                    self.cost.prompt_tokens, self.cost.completion_tokens, self.cost.local_calls
                ));
                for bucket in self.cost.by_model.iter().take(4) {
                    lines.push(format!(
                        "{:<28} ${:<9.4} {:>8} tok · {} calls",
                        bucket.key, bucket.cost_usd, bucket.tokens, bucket.calls
                    ));
                }
            }
            ActiveWorkspaceTab::Settings => {
                lines.push("Desktop configuration".to_string());
                lines.push(format!(
                    "permission mode: {}",
                    self.settings.permission_mode
                ));
                lines.push(format!("model: {}", self.settings.model));
                lines.push(format!(
                    "search: {} · fetcher: {}",
                    self.settings.search_provider, self.settings.fetcher_mode
                ));
                lines.push(format!(
                    "daemon: {} · {} · watcher {} · {}ms debounce",
                    self.settings.daemon_mode,
                    self.settings.endpoint,
                    self.settings.watcher_enabled,
                    self.settings.debounce_ms
                ));
                lines.push(format!(
                    "keys: openrouter={} anthropic={} perplexity={} tavily={}",
                    self.settings.has_openrouter_key,
                    self.settings.has_anthropic_key,
                    self.settings.has_perplexity_key,
                    self.settings.has_tavily_key
                ));
            }
        }
        lines
    }

    fn seed_showcase(&mut self) {
        self.seed_agent();
        self.seed_memory_graph();
        self.seed_review();
        self.seed_audit();
        self.seed_health();
        self.seed_research();
        self.seed_wiki();
        self.seed_failure_cards();
        self.seed_editor();
        self.seed_browser();
        self.seed_activity();
        self.seed_extensions();
        self.seed_cost();
    }

    fn seed_agent(&mut self) {
        self.agent_session.sessions = vec![
            ChatSessionMeta {
                id: "session-1".to_string(),
                title: "JSON doctor output".to_string(),
                turns: 6,
                updated: "12m ago".to_string(),
            },
            ChatSessionMeta {
                id: "session-2".to_string(),
                title: "Audit ledger filters".to_string(),
                turns: 3,
                updated: "1h ago".to_string(),
            },
            ChatSessionMeta {
                id: "session-3".to_string(),
                title: "Wiki update pipeline".to_string(),
                turns: 11,
                updated: "yesterday".to_string(),
            },
        ];
        self.agent_session.active_session_id = Some("session-1".to_string());
        self.agent_session.current_task = Some("Add JSON doctor output".to_string());
        self.agent_session.total_tool_calls = 4;
        self.agent_session.total_approved = 4;
        self.agent_session.messages = vec![
            ChatMessage::User {
                content: "Add a `--json` flag to the doctor command and cover it with a test."
                    .to_string(),
                timestamp: "2026-10-19T09:42:11Z".to_string(),
            },
            ChatMessage::AgentThought {
                content: "Charting the command surface: `cli/doctor.rs` owns parsing and `doctor/checks.rs` owns the report. I will thread a JSON renderer through both and verify the stable envelope.".to_string(),
                timestamp: "2026-10-19T09:42:14Z".to_string(),
            },
            ChatMessage::ToolResolution {
                proposal_id: "seed-1".to_string(),
                approved: true,
                output: "read_file doctor.rs · 184 lines · no diagnostics".to_string(),
                timestamp: "2026-10-19T09:42:18Z".to_string(),
            },
            ChatMessage::AgentResponse {
                content: "### Plan\n\n1. Add `--json` to `DoctorArgs` and keep the human renderer untouched.\n2. Emit a stable `{ version, checks, summary }` envelope from `doctor/checks.rs`.\n3. Assert the envelope in `tests/phase16_doctor_tests.rs`.\n\nThe working tree stays byte-identical until you approve the first write.".to_string(),
                timestamp: "2026-10-19T09:42:21Z".to_string(),
            },
        ];
    }

    fn seed_memory_graph(&mut self) {
        let nodes = vec![
            (
                "DesktopController",
                "struct",
                "src/desktop/app.rs",
                180.0,
                90.0,
            ),
            ("WebViewBridge", "fn", "src/desktop/gui.rs", 390.0, 70.0),
            ("IpcHandler", "fn", "src/desktop/gui.rs", 400.0, 190.0),
            (
                "ActiveWorkspaceTab",
                "enum",
                "src/desktop/model.rs",
                180.0,
                260.0,
            ),
            (
                "KaiokenAppWindow",
                "struct",
                "src/desktop/view.rs",
                580.0,
                130.0,
            ),
            ("SnapshotJson", "fn", "src/desktop/view.rs", 590.0, 260.0),
            ("MemoryEngine", "struct", "src/memory/mod.rs", 60.0, 390.0),
            (
                "AstParser",
                "struct",
                "src/scanner/tree_sitter.rs",
                240.0,
                420.0,
            ),
            (
                "PathGuard",
                "struct",
                "src/safety/path_guard.rs",
                430.0,
                410.0,
            ),
            ("ApprovalGate", "fn", "src/safety/approval.rs", 620.0, 390.0),
            ("AuditLogger", "struct", "src/safety/audit.rs", 760.0, 300.0),
            (
                "RenderTerminalFrame",
                "fn",
                "src/desktop/view.rs",
                760.0,
                120.0,
            ),
        ];
        let mut graph_nodes = Vec::new();
        for (idx, (name, kind, file, x, y)) in nodes.iter().enumerate() {
            graph_nodes.push(crate::desktop::model::GraphNode {
                id: idx as i64 + 1,
                name: name.to_string(),
                kind: kind.to_string(),
                file_path: file.to_string(),
                is_exported: true,
                is_test: false,
                x: *x,
                y: *y,
            });
        }
        let edges = vec![
            (1, 2, "owns"),
            (1, 3, "wires"),
            (1, 4, "routes"),
            (2, 5, "renders"),
            (5, 6, "serializes"),
            (6, 7, "reads"),
            (7, 8, "scans"),
            (8, 7, "writes"),
            (5, 9, "guards"),
            (3, 10, "proposes"),
            (10, 11, "logs"),
            (5, 12, "renders"),
            (12, 5, "reads"),
        ];
        self.memory_graph.total_files = 128;
        self.memory_graph.total_symbols = graph_nodes.len();
        self.memory_graph.total_edges = edges.len();
        self.memory_graph.nodes = graph_nodes;
        self.memory_graph.edges = edges
            .into_iter()
            .map(
                |(source_id, target_id, edge_kind)| crate::desktop::model::GraphEdge {
                    source_id,
                    target_id,
                    edge_kind: edge_kind.to_string(),
                },
            )
            .collect();
        self.memory_graph.select_node(5);
        if let Some(inspector) = &mut self.memory_graph.inspector_data {
            inspector.signature = "pub struct KaiokenAppWindow".to_string();
            inspector.doc_content =
                "Owns every reactive desktop model and serializes the WebView snapshot."
                    .to_string();
            inspector.callers = vec![
                "KaiokenDesktopController::new".to_string(),
                "launch_desktop_gui".to_string(),
            ];
            inspector.dependencies = vec![
                "AgentSessionModel".to_string(),
                "ResearchDossierModel".to_string(),
                "SettingsModel".to_string(),
            ];
        }
    }

    fn seed_review(&mut self) {
        self.review_state = ReviewStateModel {
            changed_files: vec![
                "src/desktop/gui.rs".to_string(),
                "src/desktop/model.rs".to_string(),
                "src/desktop/ui/app.js".to_string(),
            ],
            additions: 486,
            deletions: 214,
            risk_score: 34,
            risk_level: "Medium".to_string(),
            risk_reasons: vec![
                "Public enum ActiveWorkspaceTab replaced: 5 downstream matches".to_string(),
                "WebView IPC surface expanded with 8 new actions".to_string(),
            ],
            raw_diff: concat!(
                "diff --git a/src/desktop/gui.rs b/src/desktop/gui.rs\n",
                "index 8f2a1c1..b104d77 100644\n",
                "--- a/src/desktop/gui.rs\n",
                "+++ b/src/desktop/gui.rs\n",
                "@@ -13,9 +13,18 @@ use wry::WebViewBuilder;\n",
                " pub fn launch_desktop_gui(controller: Arc<Mutex<KaiokenDesktopController>>) -> Result<()> {\n",
                "-    let event_loop = EventLoop::new();\n",
                "+    let event_loop = EventLoop::<UserEvent>::with_user_event();\n",
                "     let window = WindowBuilder::new()\n",
                "-        .with_title(\"Kaioken - AI Coding Assistant\")\n",
                "+        .with_title(\"Kaioken\")\n",
                "+        .with_decorations(false)\n",
                "         .with_inner_size(LogicalSize::new(1280.0, 840.0))\n",
                "         .build(&event_loop)?;\n",
                "+\n",
                "+    let html = build_html(&initial_state);\n",
                "+    let webview = WebViewBuilder::new(&window)\n",
                "+        .with_html(&html)\n",
                "+        .with_ipc_handler(handle_ipc)\n",
                "+        .build()?;\n",
                "     event_loop.run(move |event, _, control_flow| {\n",
                "-        *control_flow = ControlFlow::Wait;\n",
                "+        *control_flow = ControlFlow::WaitUntil(Instant::now() + POLL);\n",
                "@@ -96,7 +105,7 @@ pub fn launch_desktop_gui(controller: Arc<Mutex<KaiokenDesktopController>>) -> R\n",
                "-            \"active_tab\": format!(\"{:?}\", ctrl.window.active_tab),\n",
                "+            \"active_tab\": ctrl.window.active_tab.id(),\n",
                "             \"theme\": ctrl.window.theme.name,\n",
                "         })\n",
            )
            .to_string(),
            is_clean: false,
        };
    }

    fn seed_audit(&mut self) {
        let entries = vec![
            (
                "agent",
                "edit_file",
                "src/desktop/gui.rs",
                true,
                Some("approved by user"),
                "2026-10-19T09:43:02Z",
            ),
            (
                "agent",
                "bash",
                "cargo test --all-targets",
                true,
                Some("allowlisted command"),
                "2026-10-19T09:43:31Z",
            ),
            (
                "agent",
                "bash",
                "git reset --hard origin/main",
                false,
                Some("blocked command · destructive"),
                "2026-10-19T09:44:12Z",
            ),
            (
                "user",
                "fs_write",
                ".env.local",
                false,
                Some("denied: secret-bearing path"),
                "2026-10-19T09:44:19Z",
            ),
            (
                "agent",
                "read_file",
                "src/desktop/model.rs",
                true,
                None,
                "2026-10-19T09:44:40Z",
            ),
        ];
        self.audit_safety.entries = entries
            .into_iter()
            .enumerate()
            .map(
                |(idx, (actor, action, target, approved, reason, created_at))| AuditLogEntry {
                    id: Some(idx as i64 + 1),
                    actor: actor.to_string(),
                    action: action.to_string(),
                    target: target.to_string(),
                    approved,
                    reason: reason.map(str::to_string),
                    created_at: created_at.to_string(),
                },
            )
            .collect();
        self.audit_safety.total_intercepted = 2;
        self.audit_safety.total_redacted = 1;
    }

    fn seed_health(&mut self) {
        self.health_explorer = HealthExplorerModel {
            health_score: 94,
            grade: "A".to_string(),
            total_docs: 42,
            fresh_docs: 39,
            stale_docs: 3,
            missing_docs: 0,
            items: Vec::new(),
        };
    }

    fn seed_research(&mut self) {
        self.research = ResearchDossierModel {
            query: "How do Tauri v2 and Electron compare on memory and startup?".to_string(),
            power_level: 4,
            is_running: false,
            rounds: 3,
            searched: 14,
            fetched: 11,
            crawl_depth: 2,
            duration_ms: 42_800,
            path: "deep".to_string(),
            grounded: 18,
            total_claims: 19,
            cost_usd: 0.0184,
            answer_markdown: concat!(
                "Tauri v2 and Electron reach similar product outcomes through different process models. ",
                "Electron ships a complete Chromium and Node runtime per application, so its baseline footprint ",
                "is structurally larger [1]. Tauri v2 uses the system WebView and a Rust core, which keeps the ",
                "idle working set substantially lower on Windows [2].\n\n",
                "### Where the difference comes from\n\n",
                "- **Runtime ownership** — Electron bundles Chromium; Tauri reuses Edge WebView2 [2].\n",
                "- **Startup path** — Electron initializes Node before the first paint; Tauri starts a native `tao` event loop first [3].\n",
                "- **Memory scaling** — both grow with DOM size, but Electron carries a second JavaScript heap for the main process [1].\n\n",
                "The trade-off is portability: Tauri depends on a WebView runtime being present, while Electron is ",
                "self-contained by construction [4]. On Windows 10/11 with WebView2 installed, Tauri's installed size ",
                "and idle memory are both materially smaller."
            )
            .to_string(),
            citations: vec![
                ResearchCitationModel {
                    index: 1,
                    title: "Electron process model".to_string(),
                    url: "https://www.electronjs.org/docs/latest/tutorial/process-model".to_string(),
                    host: "electronjs.org".to_string(),
                    snippet: "Chromium renderer plus Node main process.".to_string(),
                },
                ResearchCitationModel {
                    index: 2,
                    title: "Tauri architecture".to_string(),
                    url: "https://v2.tauri.app/concept/architecture/".to_string(),
                    host: "v2.tauri.app".to_string(),
                    snippet: "Rust core with system webview.".to_string(),
                },
                ResearchCitationModel {
                    index: 3,
                    title: "tao window and event loop".to_string(),
                    url: "https://docs.rs/tao/0.28/tao/".to_string(),
                    host: "docs.rs".to_string(),
                    snippet: "Cross-platform window creation in Rust.".to_string(),
                },
                ResearchCitationModel {
                    index: 4,
                    title: "WebView2 distribution".to_string(),
                    url: "https://learn.microsoft.com/microsoft-edge/webview2/".to_string(),
                    host: "learn.microsoft.com".to_string(),
                    snippet: "Evergreen and fixed-version runtime channels.".to_string(),
                },
            ],
            sources: vec![
                ResearchSourceModel {
                    title: "Electron process model".to_string(),
                    url: "https://www.electronjs.org/docs/latest/tutorial/process-model"
                        .to_string(),
                    host: "electronjs.org".to_string(),
                    kind: "docs".to_string(),
                    status: "fetched".to_string(),
                },
                ResearchSourceModel {
                    title: "Tauri architecture".to_string(),
                    url: "https://v2.tauri.app/concept/architecture/".to_string(),
                    host: "v2.tauri.app".to_string(),
                    kind: "docs".to_string(),
                    status: "fetched".to_string(),
                },
                ResearchSourceModel {
                    title: "tao 0.28 API".to_string(),
                    url: "https://docs.rs/tao/0.28/tao/".to_string(),
                    host: "docs.rs".to_string(),
                    kind: "reference".to_string(),
                    status: "fetched".to_string(),
                },
                ResearchSourceModel {
                    title: "WebView2 runtime".to_string(),
                    url: "https://learn.microsoft.com/microsoft-edge/webview2/".to_string(),
                    host: "learn.microsoft.com".to_string(),
                    kind: "reference".to_string(),
                    status: "fetched".to_string(),
                },
            ],
            steps: vec![
                ResearchStepModel {
                    label: "Scope question".to_string(),
                    detail: "Split into runtime, startup, and portability strands.".to_string(),
                    status: "done".to_string(),
                },
                ResearchStepModel {
                    label: "Plan parallel strands".to_string(),
                    detail: "Three workers, 4 searches each, depth 2.".to_string(),
                    status: "done".to_string(),
                },
                ResearchStepModel {
                    label: "Search the web".to_string(),
                    detail: "14 queries across 4 engines.".to_string(),
                    status: "done".to_string(),
                },
                ResearchStepModel {
                    label: "Read sources".to_string(),
                    detail: "11 pages read, 4 retained for citation.".to_string(),
                    status: "done".to_string(),
                },
                ResearchStepModel {
                    label: "Write dossier".to_string(),
                    detail: "Fast path promoted to deep after first evidence pass.".to_string(),
                    status: "done".to_string(),
                },
                ResearchStepModel {
                    label: "Ground citations".to_string(),
                    detail: "18 of 19 claims verified against fetched pages.".to_string(),
                    status: "done".to_string(),
                },
            ],
            history: vec![
                ResearchHistoryModel {
                    slug: "tauri-electron-memory".to_string(),
                    question: "Tauri v2 vs Electron: memory and startup".to_string(),
                    created_at: "12m ago".to_string(),
                    sources: 4,
                    rounds: 3,
                    path: "deep".to_string(),
                },
                ResearchHistoryModel {
                    slug: "rust-async-runtimes".to_string(),
                    question: "Tokio vs async-std in 2026".to_string(),
                    created_at: "2h ago".to_string(),
                    sources: 7,
                    rounds: 2,
                    path: "fast".to_string(),
                },
                ResearchHistoryModel {
                    slug: "sqlite-wal-concurrency".to_string(),
                    question: "SQLite WAL mode under concurrent readers".to_string(),
                    created_at: "yesterday".to_string(),
                    sources: 5,
                    rounds: 3,
                    path: "deep".to_string(),
                },
            ],
        };
    }

    fn seed_wiki(&mut self) {
        self.wiki = WikiWorkspaceModel {
            selected_path: "docs/01-architecture.md".to_string(),
            chapters: vec![
                WikiChapterModel {
                    title: "Architecture Overview".to_string(),
                    path: "docs/01-architecture.md".to_string(),
                    section: "Foundations".to_string(),
                    words: 1_840,
                    reading_minutes: 8,
                    markdown: concat!(
                        "# Architecture Overview\n\n",
                        "Kaioken is a repository-aware coding agent with a deterministic memory core. ",
                        "The desktop shell is a native `tao` window hosting an Edge WebView2 surface.\n\n",
                        "## The four layers\n\n",
                        "1. **Trigger layer** — CLI, TUI, desktop, and the post-commit hook.\n",
                        "2. **Scanner layer** — git inventory plus tree-sitter AST extraction.\n",
                        "3. **Memory engine** — SQLite WAL with symbols, docs, and failure cards.\n",
                        "4. **Agent runtime** — retrieval, planning, tool execution, and approval.\n\n",
                        "> Every write crosses the approval gate. Denying leaves the working tree byte-identical.\n\n",
                        "## Process boundaries\n\n",
                        "The embedded runtime keeps the daemon in-process, while the daemon mode exposes the same ",
                        "JSON-RPC envelope over a Windows named pipe or loopback TCP.\n\n",
                        "```rust\n",
                        "pub struct KaiokenDesktopController {\n",
                        "    pub window: KaiokenAppWindow,\n",
                        "    pub ipc_client: Arc<DaemonIpcClient>,\n",
                        "}\n",
                        "```\n\n",
                        "## Design invariants\n\n",
                        "- Hairline rules and zero-radius controls.\n",
                        "- One accent red; ember for live output.\n",
                        "- No decorative glow. Motion marks an active process only."
                    )
                    .to_string(),
                },
                WikiChapterModel {
                    title: "Desktop Runtime & IPC".to_string(),
                    path: "docs/02-desktop.md".to_string(),
                    section: "Desktop".to_string(),
                    words: 2_260,
                    reading_minutes: 10,
                    markdown: concat!(
                        "# Desktop Runtime & IPC\n\n",
                        "The window is created by `tao`; `wry` attaches WebView2 and installs the IPC bridge.\n\n",
                        "## Message contract\n\n",
                        "Every message is a JSON object with an `action` field followed by action-specific data.\n\n",
                        "```js\n",
                        "window.ipc.postMessage(JSON.stringify({\n",
                        "  action: 'switch_tab',\n",
                        "  tab: 'research'\n",
                        "}))\n",
                        "```\n\n",
                        "## State push\n\n",
                        "The event loop polls a monotonic revision counter and evaluates `window.KAI.mergeState` ",
                        "only when the server state changed. Local UI state — composer text, palette query, zoom — ",
                        "is never overwritten by a push.\n\n",
                        "## Window controls\n\n",
                        "The titlebar is frameless and uses explicit IPC actions for minimize, maximize, drag, and close."
                    )
                    .to_string(),
                },
                WikiChapterModel {
                    title: "Memory Engine".to_string(),
                    path: "docs/03-memory.md".to_string(),
                    section: "Knowledge".to_string(),
                    words: 3_120,
                    reading_minutes: 14,
                    markdown: concat!(
                        "# Memory Engine\n\n",
                        "Memory is a relational graph persisted in SQLite with WAL enabled. Content hashes avoid ",
                        "re-parsing unchanged files.\n\n",
                        "## Tables\n\n",
                        "- `files` — path, hash, language, last scan.\n",
                        "- `symbols` — names, kinds, signatures, doc levels.\n",
                        "- `edges` — caller/callee and ownership relations.\n",
                        "- `docs` — generated chapters with freshness state.\n",
                        "- `failures` — gotchas distilled into deterministic rules.\n\n",
                        "## Incremental sync\n\n",
                        "The watcher debounces filesystem bursts, filters secrets and build output, then updates only ",
                        "the affected subgraph."
                    )
                    .to_string(),
                },
                WikiChapterModel {
                    title: "Safety & Approvals".to_string(),
                    path: "docs/04-safety.md".to_string(),
                    section: "Trust".to_string(),
                    words: 1_480,
                    reading_minutes: 7,
                    markdown: concat!(
                        "# Safety & Approvals\n\n",
                        "Three policy tiers classify every tool call: **read-only**, **write prompt**, and **dangerous**.\n\n",
                        "## Human in the loop\n\n",
                        "A tool proposal is a blocking modal. `Esc` denies, `Enter` approves, and dismissal is always a denial.\n\n",
                        "## Path guard\n\n",
                        "No write may escape the repository root. Symlink traversal is resolved before comparison.\n\n",
                        "## Audit ledger\n\n",
                        "Every decision is appended to `.agent/logs/audit.log` and the SQLite audit table."
                    )
                    .to_string(),
                },
                WikiChapterModel {
                    title: "Build & Release".to_string(),
                    path: "docs/05-release.md".to_string(),
                    section: "Operations".to_string(),
                    words: 960,
                    reading_minutes: 5,
                    markdown: concat!(
                        "# Build & Release\n\n",
                        "Target: `x86_64-pc-windows-gnu`, no Node.js runtime, no devserver.\n\n",
                        "## Verification gate\n\n",
                        "```powershell\n",
                        "cargo fmt -- --check\n",
                        "cargo clippy --all-targets --all-features -- -D warnings\n",
                        "cargo test\n",
                        "```\n\n",
                        "Frontend assets are compiled into the binary with `include_str!`."
                    )
                    .to_string(),
                },
            ],
        };
    }

    fn seed_failure_cards(&mut self) {
        self.failure_cards = vec![
            FailureCardModel {
                id: "failure-001".to_string(),
                title: "SQLite handle crossed a thread boundary".to_string(),
                module: "src/desktop/embedded.rs".to_string(),
                severity: "high".to_string(),
                occurrences: 3,
                status: "mitigated".to_string(),
                last_seen: "2h ago".to_string(),
                gotchas: vec![
                    "rusqlite::Connection is !Send and panics or corrupts when shared across threads."
                        .to_string(),
                    "The watcher opens its own connection in the worker runtime.".to_string(),
                ],
                anti_pattern_rules: vec![
                    "Never wrap MemoryEngine in Arc<Mutex> and move it into a Send future.".to_string(),
                    "Open SQLite only inside the thread that owns the current-thread runtime."
                        .to_string(),
                ],
            },
            FailureCardModel {
                id: "failure-002".to_string(),
                title: "WebView state push raced local edits".to_string(),
                module: "src/desktop/gui.rs".to_string(),
                severity: "medium".to_string(),
                occurrences: 2,
                status: "mitigated".to_string(),
                last_seen: "6h ago".to_string(),
                gotchas: vec![
                    "A full render on every poll cleared composer input and palette selection."
                        .to_string(),
                    "Server snapshots and UI-only fields must remain separate.".to_string(),
                ],
                anti_pattern_rules: vec![
                    "Merge server state; never replace the whole local UI object.".to_string(),
                    "Push only when the controller revision advances.".to_string(),
                ],
            },
            FailureCardModel {
                id: "failure-003".to_string(),
                title: "Graph node ids drifted after a rescan".to_string(),
                module: "src/memory/graph.rs".to_string(),
                severity: "medium".to_string(),
                occurrences: 4,
                status: "watching".to_string(),
                last_seen: "yesterday".to_string(),
                gotchas: vec![
                    "Edges referenced old rowids after a full symbol replacement.".to_string(),
                    "Selection survived but pointed at an unrelated symbol.".to_string(),
                ],
                anti_pattern_rules: vec![
                    "Key selection by (file_path, symbol_name), not rowid.".to_string(),
                    "Replace a graph atomically in one transaction.".to_string(),
                ],
            },
            FailureCardModel {
                id: "failure-004".to_string(),
                title: "Approval dialog dismissal authorized a write".to_string(),
                module: "src/desktop/ui/app.js".to_string(),
                severity: "high".to_string(),
                occurrences: 1,
                status: "fixed".to_string(),
                last_seen: "3d ago".to_string(),
                gotchas: vec![
                    "Scrim click and Escape once called approve by mistake.".to_string(),
                ],
                anti_pattern_rules: vec![
                    "Dismissal maps to deny, never approve.".to_string(),
                    "Deny receives initial focus; Enter requires an explicit intent state.".to_string(),
                ],
            },
        ];
    }

    fn seed_editor(&mut self) {
        let gui_code = concat!(
            "use crate::desktop::app::KaiokenDesktopController;\n",
            "use anyhow::Result;\n",
            "use std::sync::{Arc, Mutex};\n",
            "use tao::event_loop::{ControlFlow, EventLoop};\n",
            "use wry::WebViewBuilder;\n",
            "\n",
            "const DESIGN_CSS: &str = include_str!(\"ui/design.css\");\n",
            "const APP_HTML: &str = include_str!(\"ui/app.html\");\n",
            "const APP_JS: &str = include_str!(\"ui/app.js\");\n",
            "\n",
            "pub fn launch_desktop_gui(\n",
            "    controller: Arc<Mutex<KaiokenDesktopController>>,\n",
            ") -> Result<()> {\n",
            "    let event_loop = EventLoop::<UserEvent>::with_user_event();\n",
            "    let proxy = event_loop.create_proxy();\n",
            "    let window = WindowBuilder::new()\n",
            "        .with_title(\"Kaioken\")\n",
            "        .with_decorations(false)\n",
            "        .with_inner_size(LogicalSize::new(1280.0, 840.0))\n",
            "        .build(&event_loop)?;\n",
            "\n",
            "    let html = build_html(&initial_state)?;\n",
            "    let webview = WebViewBuilder::new(&window)\n",
            "        .with_html(&html)\n",
            "        .with_ipc_handler(move |request| {\n",
            "            handle_ipc(request.body(), &controller, &proxy);\n",
            "        })\n",
            "        .build()?;\n",
            "\n",
            "    event_loop.run(move |event, _, control_flow| {\n",
            "        *control_flow = ControlFlow::WaitUntil(\n",
            "            Instant::now() + Duration::from_millis(200),\n",
            "        );\n",
            "        if let Event::UserEvent(UserEvent::StateChanged) = event {\n",
            "            push_state(&webview, &controller);\n",
            "        }\n",
            "    });\n",
            "}\n"
        );
        let js_code = concat!(
            "const KAI = {\n",
            "  state: window.__KAIOKEN_INITIAL_STATE__,\n",
            "  ui: { route: 'chat', paletteOpen: false },\n",
            "  render() { renderShell(); renderRoute(); },\n",
            "  mergeState(next) {\n",
            "    this.state = deepMerge(this.state, next);\n",
            "    this.render();\n",
            "  },\n",
            "};\n",
            "\n",
            "document.addEventListener('click', (event) => {\n",
            "  const route = event.target.closest('[data-route]')?.dataset.route;\n",
            "  if (route) KAI.go(route);\n",
            "});\n",
            "\n",
            "window.addEventListener('keydown', (event) => {\n",
            "  if (event.ctrlKey && event.key >= '1' && event.key <= '9') {\n",
            "    event.preventDefault();\n",
            "    KAI.go(ROUTES[Number(event.key) - 1]);\n",
            "  }\n",
            "});\n"
        );
        self.editor.files = vec![
            EditorFileModel {
                path: "src/desktop/gui.rs".to_string(),
                language: "Rust".to_string(),
                content: gui_code.to_string(),
                saved: gui_code.to_string(),
                truncated: false,
            },
            EditorFileModel {
                path: "src/desktop/ui/app.js".to_string(),
                language: "JavaScript".to_string(),
                content: js_code.to_string(),
                saved: js_code.to_string(),
                truncated: false,
            },
            EditorFileModel {
                path: "src/desktop/model.rs".to_string(),
                language: "Rust".to_string(),
                content: "pub enum ActiveWorkspaceTab {\n    Chat,\n    Research,\n    Wiki,\n    Graph,\n    Cards,\n    Editor,\n    Browser,\n    Activity,\n    Extensions,\n    Cost,\n    Settings,\n}\n".to_string(),
                saved: "pub enum ActiveWorkspaceTab {\n    Chat,\n    Research,\n    Wiki,\n    Graph,\n    Cards,\n    Editor,\n    Browser,\n    Activity,\n    Extensions,\n    Cost,\n    Settings,\n}\n".to_string(),
                truncated: false,
            },
        ];
        self.editor.active_path = "src/desktop/gui.rs".to_string();
        self.editor.diff_summary = "+486 −214 across 3 files · risk 34/100 (Medium)".to_string();
        self.editor.diff = self.review_state.raw_diff.clone();
        self.editor.terminal_open = false;
        self.editor.terminal_lines = vec![
            TerminalLineModel {
                kind: "cmd".to_string(),
                text: "cargo check".to_string(),
            },
            TerminalLineModel {
                kind: "output".to_string(),
                text: "   Compiling kaioken v0.1.0 (D:\\project\\ai_now_know\\kaioRust)"
                    .to_string(),
            },
            TerminalLineModel {
                kind: "output".to_string(),
                text: "    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.82s"
                    .to_string(),
            },
            TerminalLineModel {
                kind: "cmd".to_string(),
                text: "cargo test --test phase24_desktop_tests".to_string(),
            },
            TerminalLineModel {
                kind: "ok".to_string(),
                text: "test result: ok. 11 passed; 0 failed; finished in 0.04s".to_string(),
            },
            TerminalLineModel {
                kind: "cmd".to_string(),
                text: "cargo run -- desktop --embedded".to_string(),
            },
            TerminalLineModel {
                kind: "ok".to_string(),
                text: "Kaioken Desktop Window successfully launched.".to_string(),
            },
        ];
    }

    fn seed_browser(&mut self) {
        self.browser = BrowserWorkspaceModel {
            tabs: vec![
                BrowserTabModel {
                    id: "tab-api".to_string(),
                    title: "POST /chat/completions".to_string(),
                    url: "http://127.0.0.1:47890/api/chat/completions".to_string(),
                    kind: "api".to_string(),
                },
                BrowserTabModel {
                    id: "tab-docs".to_string(),
                    title: "Kaioken documentation".to_string(),
                    url: "https://kaioken.dev/docs/desktop".to_string(),
                    kind: "web".to_string(),
                },
                BrowserTabModel {
                    id: "tab-new".to_string(),
                    title: "New tab".to_string(),
                    url: String::new(),
                    kind: "newtab".to_string(),
                },
            ],
            active_id: "tab-api".to_string(),
            method: "POST".to_string(),
            request_url: "http://127.0.0.1:47890/api/chat/completions".to_string(),
            status: 200,
            latency_ms: 184,
            response_headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("x-kaioken-contract".to_string(), "v1".to_string()),
                (
                    "x-kaioken-model".to_string(),
                    "anthropic/claude-sonnet-4".to_string(),
                ),
                ("cache-control".to_string(), "no-store".to_string()),
            ],
            response_body: concat!(
                "{\n",
                "  \"id\": \"chatcmpl_01JB9M4W7R6K2F\",\n",
                "  \"object\": \"chat.completion\",\n",
                "  \"created\": 1760867042,\n",
                "  \"model\": \"anthropic/claude-sonnet-4\",\n",
                "  \"choices\": [\n",
                "    {\n",
                "      \"index\": 0,\n",
                "      \"message\": {\n",
                "        \"role\": \"assistant\",\n",
                "        \"content\": \"I added the --json flag and wired the test.\"\n",
                "      },\n",
                "      \"finish_reason\": \"stop\"\n",
                "    }\n",
                "  ],\n",
                "  \"usage\": {\n",
                "    \"prompt_tokens\": 1840,\n",
                "    \"completion_tokens\": 412,\n",
                "    \"total_tokens\": 2252\n",
                "  }\n",
                "}"
            )
            .to_string(),
        };
    }

    fn seed_activity(&mut self) {
        self.activity = ActivityWorkspaceModel {
            runs: vec![
                ActivityRunModel {
                    id: "run-101".to_string(),
                    kind: "wiki".to_string(),
                    label: "Wiki".to_string(),
                    state: "running".to_string(),
                    progress_done: 42,
                    progress_total: 64,
                    message: "Writing chapter 9 of 14 · desktop runtime".to_string(),
                    started_at: "2026-10-19T09:40:00Z".to_string(),
                    duration_ms: 0,
                    artifacts: 0,
                    multiplier: 4,
                    logs: vec![
                        "plan: 14 sections across 6 modules".to_string(),
                        "generate: 8 chapters complete".to_string(),
                        "wiki: grounding citations for chapter 9".to_string(),
                    ],
                },
                ActivityRunModel {
                    id: "run-100".to_string(),
                    kind: "generate".to_string(),
                    label: "Generate".to_string(),
                    state: "done".to_string(),
                    progress_done: 6,
                    progress_total: 6,
                    message: "Knowledge cards written".to_string(),
                    started_at: "2026-10-19T09:12:00Z".to_string(),
                    duration_ms: 74_200,
                    artifacts: 6,
                    multiplier: 0,
                    logs: vec!["6 knowledge cards written to .agent/cards".to_string()],
                },
                ActivityRunModel {
                    id: "run-099".to_string(),
                    kind: "scan".to_string(),
                    label: "Scan".to_string(),
                    state: "done".to_string(),
                    progress_done: 128,
                    progress_total: 128,
                    message: "128 files indexed".to_string(),
                    started_at: "2026-10-19T08:58:00Z".to_string(),
                    duration_ms: 3_400,
                    artifacts: 0,
                    multiplier: 0,
                    logs: vec!["128 files · 1,204 symbols · 2,860 edges".to_string()],
                },
                ActivityRunModel {
                    id: "run-098".to_string(),
                    kind: "research".to_string(),
                    label: "Research".to_string(),
                    state: "done".to_string(),
                    progress_done: 3,
                    progress_total: 3,
                    message: "Dossier exported".to_string(),
                    started_at: "2026-10-19T08:41:00Z".to_string(),
                    duration_ms: 42_800,
                    artifacts: 1,
                    multiplier: 0,
                    logs: vec!["fast path promoted to deep · 4 sources cited".to_string()],
                },
            ],
            events: vec![
                ActivityEventModel {
                    timestamp: "09:44:40".to_string(),
                    actor: "agent".to_string(),
                    action: "read_file".to_string(),
                    target: "src/desktop/model.rs".to_string(),
                    decision: "allowed".to_string(),
                    severity: "info".to_string(),
                },
                ActivityEventModel {
                    timestamp: "09:44:19".to_string(),
                    actor: "user".to_string(),
                    action: "fs_write".to_string(),
                    target: ".env.local".to_string(),
                    decision: "denied".to_string(),
                    severity: "critical".to_string(),
                },
                ActivityEventModel {
                    timestamp: "09:44:12".to_string(),
                    actor: "agent".to_string(),
                    action: "bash".to_string(),
                    target: "git reset --hard origin/main".to_string(),
                    decision: "blocked".to_string(),
                    severity: "critical".to_string(),
                },
                ActivityEventModel {
                    timestamp: "09:43:31".to_string(),
                    actor: "agent".to_string(),
                    action: "bash".to_string(),
                    target: "cargo test --all-targets".to_string(),
                    decision: "allowed".to_string(),
                    severity: "info".to_string(),
                },
                ActivityEventModel {
                    timestamp: "09:43:02".to_string(),
                    actor: "agent".to_string(),
                    action: "edit_file".to_string(),
                    target: "src/desktop/gui.rs".to_string(),
                    decision: "approved".to_string(),
                    severity: "info".to_string(),
                },
            ],
            sessions_recorded: 38,
            learned_skills: 12,
            reinforced_skills: 7,
        };
    }

    fn seed_extensions(&mut self) {
        self.extensions = ExtensionsWorkspaceModel {
            installed: vec![
                ExtensionModel {
                    id: "superpowers".to_string(),
                    version: "1.4.0".to_string(),
                    kind: "declarative".to_string(),
                    description: "Curated workflows and skills for repository maintenance."
                        .to_string(),
                    repo: "babtix/kaioken-extensions".to_string(),
                    enabled: true,
                    trusted: true,
                    local: false,
                    skills: vec!["review".to_string(), "release".to_string()],
                    tools: Vec::new(),
                },
                ExtensionModel {
                    id: "mcp-filesystem".to_string(),
                    version: "0.9.2".to_string(),
                    kind: "mcp".to_string(),
                    description: "MCP server exposing read-only filesystem tools.".to_string(),
                    repo: "modelcontextprotocol/servers".to_string(),
                    enabled: true,
                    trusted: false,
                    local: false,
                    skills: Vec::new(),
                    tools: vec!["fs_read".to_string(), "fs_list".to_string()],
                },
                ExtensionModel {
                    id: "wasm-git-context".to_string(),
                    version: "0.3.1".to_string(),
                    kind: "wasm".to_string(),
                    description: "Sandboxed wasm tool that summarizes git ranges.".to_string(),
                    repo: "kaioken/git-context".to_string(),
                    enabled: true,
                    trusted: true,
                    local: false,
                    skills: Vec::new(),
                    tools: vec!["git_range_summary".to_string()],
                },
            ],
            registry: vec![
                ExtensionModel {
                    id: "diagram-forge".to_string(),
                    version: "2.1.0".to_string(),
                    kind: "declarative".to_string(),
                    description: "Mermaid and architecture diagram conventions.".to_string(),
                    repo: "kaioken/diagram-forge".to_string(),
                    enabled: false,
                    trusted: false,
                    local: false,
                    skills: vec!["mermaid".to_string()],
                    tools: Vec::new(),
                },
                ExtensionModel {
                    id: "mcp-browser".to_string(),
                    version: "0.6.0".to_string(),
                    kind: "mcp".to_string(),
                    description: "Headless browser tools over MCP.".to_string(),
                    repo: "kaioken/mcp-browser".to_string(),
                    enabled: false,
                    trusted: false,
                    local: false,
                    skills: Vec::new(),
                    tools: vec!["browse".to_string(), "screenshot".to_string()],
                },
                ExtensionModel {
                    id: "wasm-sqlite".to_string(),
                    version: "0.2.4".to_string(),
                    kind: "wasm".to_string(),
                    description: "Read-only SQLite inspection inside the wasm sandbox.".to_string(),
                    repo: "kaioken/wasm-sqlite".to_string(),
                    enabled: false,
                    trusted: false,
                    local: false,
                    skills: Vec::new(),
                    tools: vec!["sqlite_query".to_string()],
                },
            ],
            hooks: vec![
                HookModel {
                    name: "post-commit memory sync".to_string(),
                    event: "post-commit".to_string(),
                    command: "kaioken memory sync --incremental".to_string(),
                    enabled: true,
                },
                HookModel {
                    name: "pre-tool safety audit".to_string(),
                    event: "pre-tool".to_string(),
                    command: "kaioken audit append --stdin".to_string(),
                    enabled: true,
                },
                HookModel {
                    name: "wiki stale marker".to_string(),
                    event: "post-merge".to_string(),
                    command: "kaioken docs stale --changed".to_string(),
                    enabled: false,
                },
            ],
        };
    }

    fn seed_cost(&mut self) {
        let bucket =
            |key: &str, cost: f64, tokens: u64, calls: usize, share: f64| CostBucketModel {
                key: key.to_string(),
                cost_usd: cost,
                tokens,
                calls,
                estimated_share: share,
            };
        self.cost = CostTelemetryModel {
            total_usd: 3.8421,
            known_cost_usd: 3.6104,
            calls: 148,
            prompt_tokens: 1_284_000,
            completion_tokens: 306_000,
            local_calls: 9,
            pricing_stale: false,
            by_operation: vec![
                bucket("wiki", 1.842, 782_000, 34, 0.12),
                bucket("chat", 0.941, 388_000, 71, 0.08),
                bucket("research", 0.612, 241_000, 18, 0.21),
                bucket("generate", 0.331, 112_000, 12, 0.05),
                bucket("review", 0.116, 67_000, 13, 0.04),
            ],
            by_model: vec![
                bucket("anthropic/claude-sonnet-4", 2.418, 921_000, 62, 0.1),
                bucket("openai/gpt-4o-mini", 0.714, 402_000, 58, 0.18),
                bucket("deepseek/deepseek-coder", 0.522, 188_000, 19, 0.06),
                bucket("qwen/qwen-2.5-coder-32b", 0.188, 79_000, 9, 0.0),
            ],
            by_provider: vec![
                bucket("openrouter", 3.041, 1_331_000, 119, 0.08),
                bucket("local", 0.0, 118_000, 9, 0.0),
                bucket("anthropic", 0.801, 141_000, 20, 0.03),
            ],
            by_workspace: vec![
                bucket("kaioRust", 3.102, 1_284_000, 128, 0.12),
                bucket("website", 0.562, 148_000, 14, 0.04),
                bucket("cli", 0.178, 158_000, 6, 0.02),
            ],
            by_day: vec![
                bucket("10-13", 0.241, 92_000, 8, 0.0),
                bucket("10-14", 0.388, 142_000, 13, 0.0),
                bucket("10-15", 0.612, 244_000, 22, 0.0),
                bucket("10-16", 0.291, 188_000, 17, 0.0),
                bucket("10-17", 0.744, 286_000, 26, 0.0),
                bucket("10-18", 0.912, 341_000, 31, 0.0),
                bucket("10-19", 0.654, 297_000, 31, 0.0),
            ],
        };
    }
}

fn truncate_cells(value: &str, width: usize) -> String {
    let single_line = value.replace(['\n', '\r'], " ");
    if single_line.chars().count() <= width {
        single_line
    } else {
        let mut out: String = single_line.chars().take(width.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// Construct a dangerous command proposal for the interactive approval flow.
pub fn show_dangerous_proposal() -> ToolProposal {
    ToolProposal {
        id: "proposal-demo".to_string(),
        tool_name: "bash".to_string(),
        target: "git reset --hard origin/main".to_string(),
        policy_tier: PolicyTier::Dangerous,
        details: "Discard every uncommitted change and reset to the remote branch.".to_string(),
        diff: None,
        proposed_at: chrono::Utc::now().to_rfc3339(),
    }
}
