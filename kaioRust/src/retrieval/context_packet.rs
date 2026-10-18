use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    /// Normalized summary of the user task and intended goal
    pub task_summary: String,

    /// Exact file paths with targeted line ranges and source snippets
    pub relevant_files: Vec<FileContextSnippet>,

    /// Specific AST symbol definitions, signatures, and locations
    pub relevant_symbols: Vec<SymbolContextSummary>,

    /// Localized sub-graph of callers, callees, and importing modules
    pub dependency_graph_subset: GraphSubset,

    /// Recent commits modifying the relevant files or symbols
    pub recent_related_commits: Vec<CommitSummarySnippet>,

    /// Previous failures, compiler errors, or bug fixes on touched areas
    pub related_failures: Vec<FailureMemorySnippet>,

    /// Warnings if mirrored docs or referenced specs are stale
    pub stale_docs_warnings: Vec<DocStalenessWarning>,

    /// Architectural decisions (ADRs) constraining this area of code
    pub architectural_decisions: Vec<DecisionSnippet>,

    /// Exact verification commands recommended (e.g. `cargo test --test auth`)
    pub recommended_verification_commands: Vec<String>,

    /// Detailed token budgeting report tracking consumption across sections
    pub token_budget_report: TokenBudgetReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContextSnippet {
    pub path: String,
    pub language: String,
    pub line_start: usize,
    pub line_end: usize,
    pub content: String,
    pub source_hash: String,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolContextSummary {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub signature: String,
    pub visibility: String,
    pub line_start: usize,
    pub line_end: usize,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphSubset {
    pub direct_callers: Vec<String>,
    pub direct_callees: Vec<String>,
    pub direct_imports: Vec<String>,
    pub affected_tests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummarySnippet {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub created_at: String,
    pub touched_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMemorySnippet {
    pub error_type: String,
    pub message: String,
    pub file_path: Option<String>,
    pub symbol_name: Option<String>,
    pub fix_summary: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocStalenessWarning {
    pub doc_path: String,
    pub file_path: String,
    pub reason: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSnippet {
    pub title: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgetReport {
    pub section_tokens: HashMap<String, usize>,
    pub total_tokens: usize,
    pub budget_limit: usize,
    pub truncated: bool,
}

impl ContextPacket {
    pub fn empty(task_summary: &str, budget_limit: usize) -> Self {
        Self {
            task_summary: task_summary.to_string(),
            relevant_files: Vec::new(),
            relevant_symbols: Vec::new(),
            dependency_graph_subset: GraphSubset::default(),
            recent_related_commits: Vec::new(),
            related_failures: Vec::new(),
            stale_docs_warnings: Vec::new(),
            architectural_decisions: Vec::new(),
            recommended_verification_commands: Vec::new(),
            token_budget_report: TokenBudgetReport {
                section_tokens: HashMap::new(),
                total_tokens: 0,
                budget_limit,
                truncated: false,
            },
        }
    }

    /// Serializes the context packet into structured JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Formats the context packet into structured Markdown ready for LLM prompt context.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();

        out.push_str("# Repository Context Packet\n\n");
        out.push_str(&format!("**Task Goal**: {}\n\n", self.task_summary));

        // 1. Warnings / Staleness
        if !self.stale_docs_warnings.is_empty() {
            out.push_str("## ⚠️ Documentation Staleness Warnings\n");
            for warn in &self.stale_docs_warnings {
                out.push_str(&format!(
                    "- `{}` (for `{}`): {} [status: {}] - *Rely on source AST code as truth.*\n",
                    warn.doc_path, warn.file_path, warn.reason, warn.status
                ));
            }
            out.push('\n');
        }

        // 2. Failure Memory
        if !self.related_failures.is_empty() {
            out.push_str("## 🚨 Past Failure Memory & Known Pitfalls\n");
            for f in &self.related_failures {
                let location = match (&f.file_path, &f.symbol_name) {
                    (Some(fp), Some(sn)) => format!("{}:{}", fp, sn),
                    (Some(fp), None) => fp.clone(),
                    (None, Some(sn)) => sn.clone(),
                    (None, None) => "workspace".to_string(),
                };
                out.push_str(&format!("- **[{}]** on `{}`: {}\n", f.error_type, location, f.message));
                if let Some(fix) = &f.fix_summary {
                    out.push_str(&format!("  *Past Fix / Solution*: {}\n", fix));
                }
            }
            out.push('\n');
        }

        // 3. Relevant Symbols
        if !self.relevant_symbols.is_empty() {
            out.push_str("## 🧩 Relevant AST Symbols\n");
            for sym in &self.relevant_symbols {
                out.push_str(&format!(
                    "- `{}` (`{}`) in `{}:{}-{}`\n  ```rust\n  {}\n  ```\n",
                    sym.name, sym.kind, sym.file_path, sym.line_start, sym.line_end, sym.signature
                ));
            }
            out.push('\n');
        }

        // 4. Relevant Files
        if !self.relevant_files.is_empty() {
            out.push_str("## 📄 Relevant Source Snippets\n");
            for file in &self.relevant_files {
                out.push_str(&format!(
                    "### `{}` (lines {}-{})\n```{}\n{}\n```\n\n",
                    file.path, file.line_start, file.line_end, file.language, file.content
                ));
            }
        }

        // 5. Dependency Graph
        let g = &self.dependency_graph_subset;
        if !g.direct_callers.is_empty() || !g.direct_callees.is_empty() || !g.affected_tests.is_empty() {
            out.push_str("## 🕸️ Dependency Graph Neighbors\n");
            if !g.direct_callers.is_empty() {
                out.push_str(&format!("- **Direct Callers**: {}\n", g.direct_callers.join(", ")));
            }
            if !g.direct_callees.is_empty() {
                out.push_str(&format!("- **Direct Callees**: {}\n", g.direct_callees.join(", ")));
            }
            if !g.affected_tests.is_empty() {
                out.push_str(&format!("- **Affected Tests**: {}\n", g.affected_tests.join(", ")));
            }
            out.push('\n');
        }

        // 6. Architectural Decisions
        if !self.architectural_decisions.is_empty() {
            out.push_str("## 🏛️ Architectural Decisions (ADRs)\n");
            for adr in &self.architectural_decisions {
                out.push_str(&format!("- **{}** (status: {}): {}\n", adr.title, adr.status, adr.description));
            }
            out.push('\n');
        }

        // 7. Verification Commands
        if !self.recommended_verification_commands.is_empty() {
            out.push_str("## 🧪 Recommended Verification Commands\n");
            for cmd in &self.recommended_verification_commands {
                out.push_str(&format!("- `{}`\n", cmd));
            }
            out.push('\n');
        }

        // 8. Token Budget Report
        out.push_str("## 📊 Token Budget Report\n");
        out.push_str(&format!(
            "Total tokens: {} / {} limit (Truncated: {})\n",
            self.token_budget_report.total_tokens,
            self.token_budget_report.budget_limit,
            self.token_budget_report.truncated
        ));
        for (sec, cnt) in &self.token_budget_report.section_tokens {
            out.push_str(&format!("- {}: {} tokens\n", sec, cnt));
        }

        out
    }
}
