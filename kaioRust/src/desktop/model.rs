use crate::desktop::protocol::ToolProposal;
use crate::review::scoring::RiskAssessment;
use crate::safety::audit::AuditLogEntry;
use serde::{Deserialize, Serialize};

/// The eleven first-class workspaces in Kaioken Desktop.
///
/// PRISM is intentionally absent: document intake is not part of the desktop
/// product surface. The remaining routes follow the original V1 workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ActiveWorkspaceTab {
    #[default]
    Chat,
    Research,
    Wiki,
    Graph,
    Cards,
    Editor,
    Browser,
    Activity,
    Extensions,
    Cost,
    Settings,
}

impl ActiveWorkspaceTab {
    pub fn all() -> &'static [ActiveWorkspaceTab] {
        &[
            ActiveWorkspaceTab::Chat,
            ActiveWorkspaceTab::Research,
            ActiveWorkspaceTab::Wiki,
            ActiveWorkspaceTab::Graph,
            ActiveWorkspaceTab::Cards,
            ActiveWorkspaceTab::Editor,
            ActiveWorkspaceTab::Browser,
            ActiveWorkspaceTab::Activity,
            ActiveWorkspaceTab::Extensions,
            ActiveWorkspaceTab::Cost,
            ActiveWorkspaceTab::Settings,
        ]
    }

    /// Stable route id used on the wire and in the vanilla router.
    pub fn id(&self) -> &'static str {
        match self {
            ActiveWorkspaceTab::Chat => "chat",
            ActiveWorkspaceTab::Research => "research",
            ActiveWorkspaceTab::Wiki => "wiki",
            ActiveWorkspaceTab::Graph => "graph",
            ActiveWorkspaceTab::Cards => "cards",
            ActiveWorkspaceTab::Editor => "editor",
            ActiveWorkspaceTab::Browser => "browser",
            ActiveWorkspaceTab::Activity => "activity",
            ActiveWorkspaceTab::Extensions => "extensions",
            ActiveWorkspaceTab::Cost => "cost",
            ActiveWorkspaceTab::Settings => "settings",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.to_ascii_lowercase().as_str() {
            "chat" | "agent-chat" => Some(ActiveWorkspaceTab::Chat),
            "research" => Some(ActiveWorkspaceTab::Research),
            "wiki" | "health" | "health-audit" => Some(ActiveWorkspaceTab::Wiki),
            "graph" | "memory" | "memory-graph" => Some(ActiveWorkspaceTab::Graph),
            "cards" | "failures" => Some(ActiveWorkspaceTab::Cards),
            "editor" | "review" | "code-review" => Some(ActiveWorkspaceTab::Editor),
            "browser" => Some(ActiveWorkspaceTab::Browser),
            "activity" | "audit" | "audit-safety" => Some(ActiveWorkspaceTab::Activity),
            "extensions" | "ext" => Some(ActiveWorkspaceTab::Extensions),
            "cost" | "usage" => Some(ActiveWorkspaceTab::Cost),
            "settings" | "config" => Some(ActiveWorkspaceTab::Settings),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActiveWorkspaceTab::Chat => "Chat",
            ActiveWorkspaceTab::Research => "Research",
            ActiveWorkspaceTab::Wiki => "Wiki",
            ActiveWorkspaceTab::Graph => "Graph",
            ActiveWorkspaceTab::Cards => "Cards",
            ActiveWorkspaceTab::Editor => "Editor",
            ActiveWorkspaceTab::Browser => "Browser",
            ActiveWorkspaceTab::Activity => "Activity",
            ActiveWorkspaceTab::Extensions => "Extensions",
            ActiveWorkspaceTab::Cost => "Cost",
            ActiveWorkspaceTab::Settings => "Settings",
        }
    }

    /// Icon key resolved by the embedded SVG sprite.
    pub fn icon(&self) -> &'static str {
        match self {
            ActiveWorkspaceTab::Chat => "chat",
            ActiveWorkspaceTab::Research => "radar",
            ActiveWorkspaceTab::Wiki => "book",
            ActiveWorkspaceTab::Graph => "waypoints",
            ActiveWorkspaceTab::Cards => "layers",
            ActiveWorkspaceTab::Editor => "code",
            ActiveWorkspaceTab::Browser => "globe",
            ActiveWorkspaceTab::Activity => "zap",
            ActiveWorkspaceTab::Extensions => "puzzle",
            ActiveWorkspaceTab::Cost => "wallet",
            ActiveWorkspaceTab::Settings => "settings",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            ActiveWorkspaceTab::Chat => "Ctrl+1",
            ActiveWorkspaceTab::Research => "Ctrl+2",
            ActiveWorkspaceTab::Wiki => "Ctrl+3",
            ActiveWorkspaceTab::Graph => "Ctrl+4",
            ActiveWorkspaceTab::Cards => "Ctrl+5",
            ActiveWorkspaceTab::Editor => "Ctrl+6",
            ActiveWorkspaceTab::Browser => "Ctrl+7",
            ActiveWorkspaceTab::Activity => "Ctrl+8",
            ActiveWorkspaceTab::Extensions => "Ctrl+9",
            ActiveWorkspaceTab::Cost => "",
            ActiveWorkspaceTab::Settings => "Ctrl+,",
        }
    }

    /// Workflow grouping used by the navigation rail and command palette.
    pub fn group(&self) -> &'static str {
        match self {
            ActiveWorkspaceTab::Chat | ActiveWorkspaceTab::Research => "Ask",
            ActiveWorkspaceTab::Wiki | ActiveWorkspaceTab::Graph | ActiveWorkspaceTab::Cards => {
                "Knowledge"
            }
            ActiveWorkspaceTab::Editor
            | ActiveWorkspaceTab::Browser
            | ActiveWorkspaceTab::Activity
            | ActiveWorkspaceTab::Extensions => "Tools",
            ActiveWorkspaceTab::Cost | ActiveWorkspaceTab::Settings => "System",
        }
    }

    /// Emoji fallback used by the headless ASCII snapshot.
    pub fn emoji(&self) -> &'static str {
        match self {
            ActiveWorkspaceTab::Chat => "💬",
            ActiveWorkspaceTab::Research => "📡",
            ActiveWorkspaceTab::Wiki => "📖",
            ActiveWorkspaceTab::Graph => "🕸️",
            ActiveWorkspaceTab::Cards => "🗂️",
            ActiveWorkspaceTab::Editor => "💻",
            ActiveWorkspaceTab::Browser => "🌐",
            ActiveWorkspaceTab::Activity => "⚡",
            ActiveWorkspaceTab::Extensions => "🧩",
            ActiveWorkspaceTab::Cost => "💰",
            ActiveWorkspaceTab::Settings => "⚙️",
        }
    }
}

/// A conversation row in the chat session sidebar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatSessionMeta {
    pub id: String,
    pub title: String,
    pub turns: usize,
    pub updated: String,
}

/// Agent thought log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentThoughtEntry {
    pub timestamp: String,
    pub thought: String,
}

/// An entry in the agent chat conversation history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatMessage {
    User {
        content: String,
        timestamp: String,
    },
    AgentThought {
        content: String,
        timestamp: String,
    },
    ToolProposalCard(ToolProposal),
    ToolResolution {
        proposal_id: String,
        approved: bool,
        output: String,
        timestamp: String,
    },
    AgentResponse {
        content: String,
        timestamp: String,
    },
}

/// Reactive State: Agent Chat & Interactive Task Runner (Section 3.1 & 4.2).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentSessionModel {
    pub current_task: Option<String>,
    pub is_running: bool,
    pub messages: Vec<ChatMessage>,
    pub pending_proposals: Vec<ToolProposal>,
    pub prompt_input: String,
    pub total_tool_calls: usize,
    pub total_approved: usize,
    pub total_denied: usize,
    pub sessions: Vec<ChatSessionMeta>,
    pub active_session_id: Option<String>,
}

impl AgentSessionModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_user_message(&mut self, content: impl Into<String>) {
        let text = content.into();
        self.current_task = Some(text.clone());
        self.is_running = true;
        self.messages.push(ChatMessage::User {
            content: text,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn push_thought(&mut self, thought: impl Into<String>) {
        self.messages.push(ChatMessage::AgentThought {
            content: thought.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn push_proposal(&mut self, proposal: ToolProposal) {
        self.total_tool_calls += 1;
        self.pending_proposals.push(proposal.clone());
        self.messages.push(ChatMessage::ToolProposalCard(proposal));
    }

    pub fn resolve_proposal(
        &mut self,
        proposal_id: &str,
        approved: bool,
        output: impl Into<String>,
    ) -> Option<ToolProposal> {
        if let Some(pos) = self
            .pending_proposals
            .iter()
            .position(|p| p.id == proposal_id)
        {
            let proposal = self.pending_proposals.remove(pos);
            if approved {
                self.total_approved += 1;
            } else {
                self.total_denied += 1;
            }
            self.messages.push(ChatMessage::ToolResolution {
                proposal_id: proposal_id.to_string(),
                approved,
                output: output.into(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
            Some(proposal)
        } else {
            None
        }
    }

    pub fn complete_task(&mut self, summary: impl Into<String>) {
        self.is_running = false;
        self.messages.push(ChatMessage::AgentResponse {
            content: summary.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }
}

/// A node in the codebase symbol topology graph (Section 3.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub is_exported: bool,
    pub is_test: bool,
    // Visual coordinates in GPU canvas
    pub x: f32,
    pub y: f32,
}

/// An edge between symbols in the memory graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEdge {
    pub source_id: i64,
    pub target_id: i64,
    pub edge_kind: String,
}

/// Detailed symbol inspector drawer data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SymbolInspectorData {
    pub symbol_id: i64,
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub file_path: String,
    pub doc_level: u8,
    pub doc_content: String,
    pub callers: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Reactive State: Interactive Memory & AST Graph (Section 3.2).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryGraphModel {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_edges: usize,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub selected_node_id: Option<i64>,
    pub inspector_data: Option<SymbolInspectorData>,
    pub filter_query: String,
    pub zoom_level: f32,
}

impl MemoryGraphModel {
    pub fn new() -> Self {
        Self {
            zoom_level: 1.0,
            ..Default::default()
        }
    }

    pub fn select_node(&mut self, node_id: i64) {
        self.selected_node_id = Some(node_id);
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            self.inspector_data = Some(SymbolInspectorData {
                symbol_id: node.id,
                name: node.name.clone(),
                kind: node.kind.clone(),
                signature: format!("pub fn {}()", node.name),
                file_path: node.file_path.clone(),
                doc_level: 0,
                doc_content: format!("Automatic deterministic facts for {}", node.name),
                callers: Vec::new(),
                dependencies: Vec::new(),
            });
        }
    }
}

/// Reactive State: Pre-Commit Code Review & Risk Inspector (Section 3.3).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewStateModel {
    pub changed_files: Vec<String>,
    pub additions: usize,
    pub deletions: usize,
    pub risk_score: u32,
    pub risk_level: String,
    pub risk_reasons: Vec<String>,
    pub raw_diff: String,
    pub is_clean: bool,
}

impl ReviewStateModel {
    pub fn from_assessment(assessment: &RiskAssessment, diff: impl Into<String>) -> Self {
        let risk_reasons = assessment.factors.iter().map(|f| f.description()).collect();
        Self {
            changed_files: Vec::new(),
            additions: 0,
            deletions: 0,
            risk_score: (assessment.score * 100.0) as u32,
            risk_level: format!("{:?}", assessment.level),
            risk_reasons,
            raw_diff: diff.into(),
            is_clean: assessment.factors.is_empty(),
        }
    }

    pub fn from_report(report: &crate::review::ReviewReport, diff: impl Into<String>) -> Self {
        let additions = report.changed_files.iter().map(|f| f.additions).sum();
        let deletions = report.changed_files.iter().map(|f| f.deletions).sum();
        let changed_files = report
            .changed_files
            .iter()
            .map(|f| f.path.clone())
            .collect();
        let risk_reasons = report
            .risk
            .factors
            .iter()
            .map(|f| f.description())
            .collect();
        Self {
            changed_files,
            additions,
            deletions,
            risk_score: (report.risk.score * 100.0) as u32,
            risk_level: format!("{:?}", report.risk.level),
            risk_reasons,
            raw_diff: diff.into(),
            is_clean: report.changed_files.is_empty(),
        }
    }
}

/// Reactive State: Safety & Audit Stream (Section 3.4).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditSafetyModel {
    pub entries: Vec<AuditLogEntry>,
    pub tool_filter: Option<String>,
    pub severity_filter: Option<String>,
    pub search_query: String,
    pub total_intercepted: usize,
    pub total_redacted: usize,
}

impl AuditSafetyModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_entry(&mut self, entry: AuditLogEntry) {
        if !entry.approved {
            self.total_intercepted += 1;
        }
        if entry.target.contains("***REDACTED_SECRET***") {
            self.total_redacted += 1;
        }
        self.entries.push(entry);
    }

    pub fn filtered_entries(&self) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(ref t) = self.tool_filter {
                    if !e.action.eq_ignore_ascii_case(t) {
                        return false;
                    }
                }
                if let Some(ref s) = self.severity_filter {
                    if !e.severity().eq_ignore_ascii_case(s) {
                        return false;
                    }
                }
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    if !e.target.to_lowercase().contains(&q)
                        && !e.action.to_lowercase().contains(&q)
                    {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

/// Item in the documentation mirror tree (Section 3.5).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocMirrorItem {
    pub source_path: String,
    pub doc_path: String,
    pub doc_level: u8,
    pub status: String, // "fresh", "stale", "missing"
    pub symbols_count: usize,
}

/// Reactive State: Health & Documentation Mirror Explorer (Section 3.5).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthExplorerModel {
    pub health_score: u32,
    pub grade: String,
    pub total_docs: usize,
    pub fresh_docs: usize,
    pub stale_docs: usize,
    pub missing_docs: usize,
    pub items: Vec<DocMirrorItem>,
}

/// Daemon IPC connection state (Section 2 & 4.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DaemonConnectionModel {
    pub status: ConnectionStatus,
    pub endpoint: String,
    pub latency_ms: u64,
    pub last_heartbeat: Option<String>,
}

/// Command Palette Action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaletteAction {
    pub title: String,
    pub shortcut: Option<String>,
    pub category: String,
    pub action_id: String,
}

/// Global Command Palette Model (Section 4.3).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandPaletteModel {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub actions: Vec<PaletteAction>,
}

impl CommandPaletteModel {
    pub fn default_actions() -> Vec<PaletteAction> {
        let mut actions = Vec::new();

        for tab in ActiveWorkspaceTab::all() {
            actions.push(PaletteAction {
                title: format!("Switch to {}", tab.label()),
                shortcut: if tab.shortcut().is_empty() {
                    None
                } else {
                    Some(tab.shortcut().to_string())
                },
                category: "Navigation".to_string(),
                action_id: format!("nav:{}", tab.id()),
            });
        }

        for (title, shortcut, action_id) in [
            ("New Chat Session", Some("Ctrl+N"), "action:new_session"),
            (
                "Compact Conversation",
                Some("Ctrl+Shift+C"),
                "action:compact_session",
            ),
            (
                "Review Working Tree Diff",
                Some("Ctrl+6"),
                "action:git_diff",
            ),
            ("Run Codebase Scan", Some("Ctrl+Shift+S"), "action:scan"),
            (
                "Run Environment Doctor",
                Some("Ctrl+Shift+D"),
                "action:doctor",
            ),
            ("Clear Repository Memory", None, "action:clear_memory"),
            ("Toggle Theme", None, "action:toggle_theme"),
        ] {
            actions.push(PaletteAction {
                title: title.to_string(),
                shortcut: shortcut.map(str::to_string),
                category: "Commands".to_string(),
                action_id: action_id.to_string(),
            });
        }

        actions
    }

    pub fn filtered_actions(&self) -> Vec<&PaletteAction> {
        if self.query.is_empty() {
            self.actions.iter().collect()
        } else {
            let q = self.query.to_lowercase();
            self.actions
                .iter()
                .filter(|a| {
                    a.title.to_lowercase().contains(&q) || a.category.to_lowercase().contains(&q)
                })
                .collect()
        }
    }
}

// ── Research ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResearchCitationModel {
    pub index: u32,
    pub title: String,
    pub url: String,
    pub host: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResearchSourceModel {
    pub title: String,
    pub url: String,
    pub host: String,
    pub kind: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResearchStepModel {
    pub label: String,
    pub detail: String,
    /// `done`, `active`, or `pending`.
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResearchHistoryModel {
    pub slug: String,
    pub question: String,
    pub created_at: String,
    pub sources: usize,
    pub rounds: u32,
    pub path: String,
}

/// Reactive state for the Perplexity-style research dossier.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchDossierModel {
    pub query: String,
    pub power_level: u8,
    pub is_running: bool,
    pub rounds: u32,
    pub searched: u32,
    pub fetched: u32,
    pub crawl_depth: u8,
    pub duration_ms: u64,
    /// `fast` or `deep`.
    pub path: String,
    pub grounded: u32,
    pub total_claims: u32,
    pub cost_usd: f64,
    pub answer_markdown: String,
    pub citations: Vec<ResearchCitationModel>,
    pub sources: Vec<ResearchSourceModel>,
    pub steps: Vec<ResearchStepModel>,
    pub history: Vec<ResearchHistoryModel>,
}

impl ResearchDossierModel {
    /// Advance the simulated/cached dossier trail by one step.
    pub fn tick(&mut self) {
        if !self.is_running {
            return;
        }
        let mut advanced = false;
        for step in &mut self.steps {
            if step.status == "active" {
                step.status = "done".to_string();
                advanced = true;
                break;
            }
        }
        if let Some(next) = self.steps.iter_mut().find(|step| step.status == "pending") {
            next.status = "active".to_string();
        } else if advanced {
            self.is_running = false;
            self.searched += 4;
            self.fetched += 3;
            self.rounds += 1;
            self.duration_ms += 1_750;
            self.grounded = self.grounded.max(self.total_claims.saturating_sub(1));
        }
    }
}

// ── Wiki ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WikiChapterModel {
    pub title: String,
    pub path: String,
    pub section: String,
    pub markdown: String,
    pub words: usize,
    pub reading_minutes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiWorkspaceModel {
    pub chapters: Vec<WikiChapterModel>,
    pub selected_path: String,
}

// ── Failure memory cards ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FailureCardModel {
    pub id: String,
    pub title: String,
    pub module: String,
    pub severity: String,
    pub occurrences: usize,
    pub status: String,
    pub last_seen: String,
    pub gotchas: Vec<String>,
    pub anti_pattern_rules: Vec<String>,
}

// ── Editor ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EditorFileModel {
    pub path: String,
    pub language: String,
    pub content: String,
    pub saved: String,
    pub truncated: bool,
}

impl EditorFileModel {
    pub fn dirty(&self) -> bool {
        self.content != self.saved
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TerminalLineModel {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorWorkspaceModel {
    pub files: Vec<EditorFileModel>,
    pub active_path: String,
    pub diff: String,
    pub diff_summary: String,
    pub terminal_open: bool,
    pub terminal_lines: Vec<TerminalLineModel>,
}

// ── Browser ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BrowserTabModel {
    pub id: String,
    pub title: String,
    pub url: String,
    /// `web`, `api`, or `newtab`.
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrowserWorkspaceModel {
    pub tabs: Vec<BrowserTabModel>,
    pub active_id: String,
    pub method: String,
    pub request_url: String,
    pub status: u16,
    pub latency_ms: u64,
    pub response_headers: Vec<(String, String)>,
    pub response_body: String,
}

// ── Activity ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ActivityRunModel {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub state: String,
    pub progress_done: usize,
    pub progress_total: usize,
    pub message: String,
    pub started_at: String,
    pub duration_ms: u64,
    pub artifacts: usize,
    pub multiplier: u8,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ActivityEventModel {
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub decision: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityWorkspaceModel {
    pub runs: Vec<ActivityRunModel>,
    pub events: Vec<ActivityEventModel>,
    pub sessions_recorded: usize,
    pub learned_skills: usize,
    pub reinforced_skills: usize,
}

// ── Extensions ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ExtensionModel {
    pub id: String,
    pub version: String,
    pub kind: String,
    pub description: String,
    pub repo: String,
    pub enabled: bool,
    pub trusted: bool,
    pub local: bool,
    pub skills: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HookModel {
    pub name: String,
    pub event: String,
    pub command: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionsWorkspaceModel {
    pub installed: Vec<ExtensionModel>,
    pub registry: Vec<ExtensionModel>,
    pub hooks: Vec<HookModel>,
}

// ── Cost telemetry ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CostBucketModel {
    pub key: String,
    pub cost_usd: f64,
    pub tokens: u64,
    pub calls: usize,
    pub estimated_share: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostTelemetryModel {
    pub total_usd: f64,
    pub known_cost_usd: f64,
    pub calls: usize,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub local_calls: usize,
    pub pricing_stale: bool,
    pub by_operation: Vec<CostBucketModel>,
    pub by_model: Vec<CostBucketModel>,
    pub by_provider: Vec<CostBucketModel>,
    pub by_workspace: Vec<CostBucketModel>,
    pub by_day: Vec<CostBucketModel>,
}

impl CostTelemetryModel {
    pub fn total_tokens(&self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }
}

// ── Settings ────────────────────────────────────────────────────────────────

/// Desktop-visible settings. API keys are represented by presence flags only;
/// the actual secrets never enter the UI state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsModel {
    pub permission_mode: String,
    pub model: String,
    pub base_url: String,
    pub search_provider: String,
    pub fetcher_mode: String,
    pub daemon_mode: String,
    pub endpoint: String,
    pub theme: String,
    pub has_openrouter_key: bool,
    pub has_anthropic_key: bool,
    pub has_perplexity_key: bool,
    pub has_tavily_key: bool,
    pub watcher_enabled: bool,
    pub debounce_ms: u64,
}

impl Default for SettingsModel {
    fn default() -> Self {
        Self {
            permission_mode: "edit-with-approval".to_string(),
            model: "deepseek/deepseek-coder".to_string(),
            base_url: String::new(),
            search_provider: "both".to_string(),
            fetcher_mode: "auto".to_string(),
            daemon_mode: "embedded".to_string(),
            endpoint: "embedded in-process".to_string(),
            theme: "dark".to_string(),
            has_openrouter_key: true,
            has_anthropic_key: false,
            has_perplexity_key: true,
            has_tavily_key: false,
            watcher_enabled: true,
            debounce_ms: 300,
        }
    }
}
