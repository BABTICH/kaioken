use crate::desktop::protocol::ToolProposal;
use crate::review::scoring::RiskAssessment;
use crate::safety::audit::AuditLogEntry;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard = 0,
    Memory = 1,
    Task = 2,
    Review = 3,
    Audit = 4,
}

impl ActiveTab {
    pub fn all() -> [ActiveTab; 5] {
        [
            ActiveTab::Dashboard,
            ActiveTab::Memory,
            ActiveTab::Task,
            ActiveTab::Review,
            ActiveTab::Audit,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            ActiveTab::Dashboard => "[1] Dashboard",
            ActiveTab::Memory => "[2] Memory Graph",
            ActiveTab::Task => "[3] Task Runner",
            ActiveTab::Review => "[4] Review",
            ActiveTab::Audit => "[5] Audit Log",
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => ActiveTab::Memory,
            2 => ActiveTab::Task,
            3 => ActiveTab::Review,
            4 => ActiveTab::Audit,
            _ => ActiveTab::Dashboard,
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }

    pub fn next(self) -> Self {
        Self::from_index((self.to_index() + 1) % 5)
    }

    pub fn prev(self) -> Self {
        Self::from_index((self.to_index() + 4) % 5)
    }

    pub fn from_str_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "memory" | "graph" => ActiveTab::Memory,
            "task" | "runner" | "chat" => ActiveTab::Task,
            "review" | "diff" => ActiveTab::Review,
            "audit" | "safety" => ActiveTab::Audit,
            _ => ActiveTab::Dashboard,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonConnectionState {
    Connected,
    Disconnected,
    Embedded,
}

#[derive(Debug, Clone)]
pub struct TuiCommitItem {
    pub commit_hash: String,
    pub message: String,
    pub files_changed: usize,
    pub symbols_delta: String,
    pub risk_level: String,
}

#[derive(Debug, Clone)]
pub struct TuiSymbolItem {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_number: usize,
    pub visibility: String,
    pub doc_summary: Option<String>,
    pub callers_count: usize,
}

#[derive(Debug, Clone)]
pub struct TuiApp {
    pub repo_name: String,
    pub branch: String,
    pub repo_path: PathBuf,
    pub active_tab: ActiveTab,
    pub daemon_state: DaemonConnectionState,
    pub sync_time: String,
    pub read_only: bool,
    pub should_quit: bool,
    pub is_paused: bool,
    pub show_help: bool,

    // Tab 1: Dashboard
    pub health_score: u32,
    pub doc_coverage: u32,
    pub total_files: usize,
    pub total_symbols: usize,
    pub active_locks: usize,
    pub recent_commits: Vec<TuiCommitItem>,
    pub live_activity: Vec<String>,

    // Tab 2: Memory Graph
    pub symbols: Vec<TuiSymbolItem>,
    pub selected_symbol_idx: usize,
    pub symbol_search_query: String,

    // Tab 3: Task Runner
    pub task_title: String,
    pub task_prompt: String,
    pub agent_logs: Vec<String>,
    pub pending_proposal: Option<ToolProposal>,
    pub approved_proposals_count: usize,
    pub rejected_proposals_count: usize,

    // Tab 4: Review
    pub diff_content: String,
    pub risk_score: u32,
    pub risk_level: String,
    pub risk_reasons: Vec<String>,
    pub review_scroll: usize,

    // Tab 5: Audit Log
    pub audit_entries: Vec<AuditLogEntry>,
    pub selected_audit_idx: usize,
}

impl TuiApp {
    pub fn new(
        repo_name: impl Into<String>,
        branch: impl Into<String>,
        repo_path: PathBuf,
    ) -> Self {
        Self {
            repo_name: repo_name.into(),
            branch: branch.into(),
            repo_path,
            active_tab: ActiveTab::Dashboard,
            daemon_state: DaemonConnectionState::Connected,
            sync_time: "just now".to_string(),
            read_only: false,
            should_quit: false,
            is_paused: false,
            show_help: false,

            health_score: 95,
            doc_coverage: 90,
            total_files: 0,
            total_symbols: 0,
            active_locks: 0,
            recent_commits: Vec::new(),
            live_activity: vec![
                "[SYSTEM] Kaioken interactive TUI initialized".to_string(),
                "[DAEMON] Connected to background daemon".to_string(),
            ],

            symbols: Vec::new(),
            selected_symbol_idx: 0,
            symbol_search_query: String::new(),

            task_title: "Idle".to_string(),
            task_prompt: String::new(),
            agent_logs: vec![
                "Ready for tasks. Use [3] Task Runner to inspect active agent loops.".to_string(),
            ],
            pending_proposal: None,
            approved_proposals_count: 0,
            rejected_proposals_count: 0,

            diff_content: "No uncommitted changes detected. Working tree clean.".to_string(),
            risk_score: 0,
            risk_level: "Clean".to_string(),
            risk_reasons: Vec::new(),
            review_scroll: 0,

            audit_entries: Vec::new(),
            selected_audit_idx: 0,
        }
    }

    pub fn with_initial_tab(mut self, tab_str: &str) -> Self {
        self.active_tab = ActiveTab::from_str_name(tab_str);
        self
    }

    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = self.active_tab.prev();
    }

    pub fn set_tab(&mut self, tab: ActiveTab) {
        self.active_tab = tab;
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        let status = if self.is_paused { "Paused" } else { "Resumed" };
        self.push_activity(format!("[TUI] Event stream {}", status));
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn push_activity(&mut self, msg: impl Into<String>) {
        if self.is_paused {
            return;
        }
        self.live_activity.push(msg.into());
        if self.live_activity.len() > 100 {
            self.live_activity.remove(0);
        }
    }

    pub fn push_agent_log(&mut self, msg: impl Into<String>) {
        self.agent_logs.push(msg.into());
        if self.agent_logs.len() > 200 {
            self.agent_logs.remove(0);
        }
    }

    pub fn set_proposal(&mut self, proposal: ToolProposal) {
        self.push_agent_log(format!(
            "⚠️ Proposed tool execution: {} [{}]",
            proposal.tool_name, proposal.policy_tier
        ));
        self.pending_proposal = Some(proposal);
    }

    pub fn approve_pending_proposal(&mut self) -> Option<String> {
        if let Some(prop) = self.pending_proposal.take() {
            self.approved_proposals_count += 1;
            let msg = format!("✅ Approved proposal {}: {}", prop.id, prop.tool_name);
            self.push_agent_log(&msg);
            self.push_activity(&msg);
            Some(prop.id)
        } else {
            None
        }
    }

    pub fn reject_pending_proposal(&mut self) -> Option<String> {
        if let Some(prop) = self.pending_proposal.take() {
            self.rejected_proposals_count += 1;
            let msg = format!("❌ Rejected proposal {}: {}", prop.id, prop.tool_name);
            self.push_agent_log(&msg);
            self.push_activity(&msg);
            Some(prop.id)
        } else {
            None
        }
    }

    pub fn scroll_up(&mut self) {
        match self.active_tab {
            ActiveTab::Memory if self.selected_symbol_idx > 0 => {
                self.selected_symbol_idx -= 1;
            }
            ActiveTab::Review if self.review_scroll > 0 => {
                self.review_scroll -= 1;
            }
            ActiveTab::Audit if self.selected_audit_idx > 0 => {
                self.selected_audit_idx -= 1;
            }
            _ => {}
        }
    }

    pub fn scroll_down(&mut self) {
        match self.active_tab {
            ActiveTab::Memory => {
                let filtered_len = self.filtered_symbols().len();
                if filtered_len > 0 && self.selected_symbol_idx + 1 < filtered_len {
                    self.selected_symbol_idx += 1;
                }
            }
            ActiveTab::Review => {
                self.review_scroll += 1;
            }
            ActiveTab::Audit
                if !self.audit_entries.is_empty()
                    && self.selected_audit_idx + 1 < self.audit_entries.len() =>
            {
                self.selected_audit_idx += 1;
            }
            _ => {}
        }
    }

    pub fn filtered_symbols(&self) -> Vec<&TuiSymbolItem> {
        if self.symbol_search_query.trim().is_empty() {
            self.symbols.iter().collect()
        } else {
            let q = self.symbol_search_query.to_lowercase();
            self.symbols
                .iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&q) || s.file_path.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn apply_risk_assessment(&mut self, assessment: &RiskAssessment, diff: &str) {
        self.diff_content = diff.to_string();
        self.risk_score = (assessment.score * 100.0) as u32;
        self.risk_level = format!("{:?}", assessment.level);
        self.risk_reasons = assessment
            .factors
            .iter()
            .map(|f| format!("- {}", f.description()))
            .collect();
    }
}
