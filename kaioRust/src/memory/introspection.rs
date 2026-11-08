use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::memory::commit_memory::CommitRecord;
use crate::memory::doc_memory::DocRecord;
use crate::memory::failure_memory::FailureRecord;
use crate::memory::file_memory::{FileRecord, RawImportRecord};
use crate::memory::graph::EdgeRecord;
use crate::memory::symbol_memory::SymbolRecord;

/// Detailed edge record with resolved source and target symbol/file paths
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetailedEdge {
    pub id: Option<i64>,
    pub source_symbol_id: Option<i64>,
    pub source_symbol_name: Option<String>,
    pub source_file_path: Option<String>,
    pub target_symbol_id: Option<i64>,
    pub target_symbol_name: Option<String>,
    pub target_file_path: Option<String>,
    pub edge_type: String,
}

/// Comprehensive internal memory profile for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExplanation {
    pub file: FileRecord,
    pub symbols: Vec<SymbolRecord>,
    pub outgoing_edges: Vec<EdgeRecord>,
    pub incoming_edges: Vec<EdgeRecord>,
    pub raw_imports: Vec<RawImportRecord>,
    pub documentation: Option<DocRecord>,
    pub recent_commits: Vec<CommitRecord>,
    pub failures: Vec<FailureRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detailed_outgoing: Vec<DetailedEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detailed_incoming: Vec<DetailedEdge>,
}

impl FileExplanation {
    /// Render human-readable ANSI formatted explanation for terminal display
    pub fn render_terminal(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\x1b[1;36m📖 Memory Explain — {}\x1b[0m\n",
            self.file.path
        ));
        out.push_str("════════════════════════════════════════════════════════════\n\n");

        // 1. File Metadata
        out.push_str("  \x1b[1m📄 File Metadata\x1b[0m\n");
        out.push_str("  ────────────────\n");
        let lang_str = match self.file.language.as_deref() {
            Some(l) if l.eq_ignore_ascii_case("rust") => "Rust",
            Some(l) if l.eq_ignore_ascii_case("python") => "Python",
            Some(l) if l.eq_ignore_ascii_case("typescript") => "TypeScript",
            Some(l) if l.eq_ignore_ascii_case("javascript") => "JavaScript",
            Some(other) => other,
            None => "Unknown",
        };
        out.push_str(&format!("  Language:      {}\n", lang_str));

        let size_str = format!("{} bytes", self.file.size);
        let line_count_str = if !self.symbols.is_empty() {
            let max_line = self.symbols.iter().map(|s| s.line_end).max().unwrap_or(0);
            if max_line > 0 {
                format!(" (~{} lines)", max_line)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  Size:          {}{}\n",
            size_str, line_count_str
        ));

        let hash_preview = self
            .file
            .source_hash
            .as_deref()
            .map(|h| {
                if h.len() > 16 {
                    format!("{}...", &h[0..16])
                } else {
                    h.to_string()
                }
            })
            .unwrap_or_else(|| "None".to_string());
        out.push_str(&format!("  SHA-256:       {}\n", hash_preview));
        out.push_str(&format!(
            "  Parse Status:  {}\n",
            self.file.parse_status.as_deref().unwrap_or("ok")
        ));
        out.push_str(&format!(
            "  Last Scanned:  {}\n\n",
            self.file.last_modified.as_deref().unwrap_or("Never")
        ));

        // 2. Symbols
        let sym_count = self.symbols.len();
        out.push_str(&format!(
            "  \x1b[1m🔣 Symbols ({} extracted)\x1b[0m\n",
            sym_count
        ));
        out.push_str("  ────────────────────────────\n");
        if self.symbols.is_empty() {
            out.push_str("  (No symbols extracted)\n\n");
        } else {
            // Group symbols by parent if available
            let mut roots = Vec::new();
            let mut children: std::collections::HashMap<String, Vec<&SymbolRecord>> =
                std::collections::HashMap::new();

            for s in &self.symbols {
                if let Some(p) = &s.parent {
                    children.entry(p.clone()).or_default().push(s);
                } else {
                    roots.push(s);
                }
            }

            for (idx, root) in roots.iter().enumerate() {
                let is_last = idx + 1 == roots.len() && !children.contains_key(&root.name);
                let branch = if is_last { "└──" } else { "├──" };
                let vis = root.visibility.as_deref().unwrap_or("priv");
                let line_span = format!("L{}-L{}", root.line_start, root.line_end);
                out.push_str(&format!(
                    "  {} {:<6} {:<18} ({:<4})   {}\n",
                    branch, root.kind, root.name, vis, line_span
                ));

                if let Some(kids) = children.get(&root.name) {
                    for (c_idx, kid) in kids.iter().enumerate() {
                        let c_last = c_idx + 1 == kids.len();
                        let sub_branch = if c_last { "└──" } else { "├──" };
                        let k_vis = kid.visibility.as_deref().unwrap_or("priv");
                        let k_span = format!("L{}-L{}", kid.line_start, kid.line_end);
                        out.push_str(&format!(
                            "  │   {} {:<6} {:<14} ({:<4})   {}\n",
                            sub_branch, kid.kind, kid.name, k_vis, k_span
                        ));
                    }
                }
            }
            out.push('\n');
        }

        // 3. Dependencies (Outgoing Edges)
        let total_outgoing =
            self.detailed_outgoing.len().max(self.outgoing_edges.len()) + self.raw_imports.len();
        out.push_str(&format!(
            "  \x1b[1m🔗 Dependencies (Outgoing Edges: {})\x1b[0m\n",
            total_outgoing
        ));
        out.push_str("  ─────────────────────────────────────\n");
        if total_outgoing == 0 {
            out.push_str("  (No outgoing dependencies recorded)\n\n");
        } else {
            let mut shown = std::collections::HashSet::new();
            // Show detailed outgoing first
            for de in &self.detailed_outgoing {
                let target_file = de.target_file_path.as_deref().unwrap_or("external");
                let sym_desc = if let Some(sym) = &de.target_symbol_name {
                    format!("{}: {}", de.edge_type, sym)
                } else {
                    de.edge_type.clone()
                };
                let line = format!("  → {:<24} ({})\n", target_file, sym_desc);
                if shown.insert(line.clone()) {
                    out.push_str(&line);
                }
            }

            // Raw imports
            for imp in &self.raw_imports {
                let line = format!("  → {:<24} (import)\n", imp.raw_import);
                if shown.insert(line.clone()) {
                    out.push_str(&line);
                }
            }
            out.push('\n');
        }

        // 4. Dependents (Incoming Edges)
        let total_incoming = self.detailed_incoming.len().max(self.incoming_edges.len());
        out.push_str(&format!(
            "  \x1b[1m📥 Dependents (Incoming Edges: {})\x1b[0m\n",
            total_incoming
        ));
        out.push_str("  ──────────────────────────────────\n");
        if total_incoming == 0 {
            out.push_str("  (No incoming dependents recorded)\n\n");
        } else {
            let mut shown = std::collections::HashSet::new();
            for di in &self.detailed_incoming {
                let source_file = di.source_file_path.as_deref().unwrap_or("unknown");
                let sym_desc = if let Some(sym) = &di.source_symbol_name {
                    format!("{}: {}", di.edge_type, sym)
                } else {
                    di.edge_type.clone()
                };
                let line = format!("  ← {:<24} ({})\n", source_file, sym_desc);
                if shown.insert(line.clone()) {
                    out.push_str(&line);
                }
            }
            out.push('\n');
        }

        // 5. Documentation
        out.push_str("  \x1b[1m📝 Documentation\x1b[0m\n");
        out.push_str("  ─────────────────\n");
        if let Some(doc) = &self.documentation {
            out.push_str(&format!("  Doc Path:   {}\n", doc.doc_path));
            let level_desc = match doc.doc_level {
                0 => "Level 0 (Facts)",
                1 => "Level 1 (Summary)",
                2 => "Level 2 (Deep Architecture)",
                _ => "Custom Level",
            };
            out.push_str(&format!("  Doc Level:  {}\n", level_desc));
            out.push_str(&format!("  Status:     {}\n", doc.status));
            let staleness_str = match doc.status.as_str() {
                "fresh" | "current" => "\x1b[32m✅ Up to date\x1b[0m".to_string(),
                "stale" => {
                    let r = doc.stale_reason.as_deref().unwrap_or("Modified source");
                    format!("\x1b[31m⚠️ Stale ({})\x1b[0m", r)
                }
                "maybe_stale" => {
                    let r = doc.stale_reason.as_deref().unwrap_or("Dependency changed");
                    format!("\x1b[33m⚠️ Possibly Stale ({})\x1b[0m", r)
                }
                other => other.to_string(),
            };
            out.push_str(&format!("  Staleness:  {}\n\n", staleness_str));
        } else {
            out.push_str("  (No documentation generated yet)\n\n");
        }

        // 6. Recent Commits
        let commit_count = self.recent_commits.len();
        out.push_str(&format!(
            "  \x1b[1m📊 Recent Commits ({})\x1b[0m\n",
            commit_count
        ));
        out.push_str("  ──────────────────────\n");
        if self.recent_commits.is_empty() {
            out.push_str("  (No recent commits touching this file)\n\n");
        } else {
            for c in self.recent_commits.iter().take(10) {
                let short_hash = if c.hash.len() >= 7 {
                    &c.hash[0..7]
                } else {
                    &c.hash
                };
                let date = c.created_at.as_deref().unwrap_or("recent");
                let risk = format_risk_badge(c.risk_level.as_deref());
                out.push_str(&format!(
                    "  • {} ({}) — \"{}\"  {}\n",
                    short_hash, date, c.message, risk
                ));
            }
            out.push('\n');
        }

        // 7. Failure Memory
        let failure_count = self.failures.len();
        out.push_str(&format!(
            "  \x1b[1m⚠️ Failure Memory ({})\x1b[0m\n",
            failure_count
        ));
        out.push_str("  ──────────────────────\n");
        if self.failures.is_empty() {
            out.push_str("  (No failures recorded for this file)\n");
        } else {
            for f in &self.failures {
                let status_badge = match f.status.as_deref().unwrap_or("unresolved") {
                    "resolved" => "\x1b[32m[RESOLVED]\x1b[0m",
                    _ => "\x1b[31m[UNRESOLVED]\x1b[0m",
                };
                let fix_info = f
                    .fix_summary
                    .as_deref()
                    .map(|s| format!(" — {}", s))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  • {} \"{}: {}\"{}\n",
                    status_badge, f.error_type, f.message, fix_info
                ));
            }
        }

        out
    }
}

/// Detailed commit memory entry for timeline view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMemoryDetail {
    pub commit: CommitRecord,
    pub files: Vec<String>,
    pub symbols: Vec<String>,
    pub summary: Option<String>,
    pub fixes: Vec<String>,
}

/// Result of querying recent commit memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecentResult {
    pub total_commits: usize,
    pub shown_commits: usize,
    pub commits: Vec<CommitMemoryDetail>,
}

impl MemoryRecentResult {
    /// Render human-readable ANSI formatted commit timeline
    pub fn render_terminal(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\x1b[1;36m🕐 Recent Commit Memories — Last {}\x1b[0m\n",
            self.shown_commits
        ));
        out.push_str("════════════════════════════════════════════════════════════════\n\n");

        if self.commits.is_empty() {
            out.push_str("  No commit memories matching criteria found.\n\n");
        } else {
            for detail in &self.commits {
                let short_hash = if detail.commit.hash.len() >= 7 {
                    &detail.commit.hash[0..7]
                } else {
                    &detail.commit.hash
                };
                let date = detail.commit.created_at.as_deref().unwrap_or("unknown");
                let risk_badge = format_risk_badge(detail.commit.risk_level.as_deref());

                out.push_str(&format!(
                    "  ┌─ \x1b[1m{}\x1b[0m  {}  {}\n",
                    short_hash, date, risk_badge
                ));
                let author = detail.commit.author.as_deref().unwrap_or("unknown");
                out.push_str(&format!("  │  Author:  {}\n", author));
                out.push_str(&format!("  │  Message: \"{}\"\n", detail.commit.message));

                if !detail.files.is_empty() {
                    let files_str = if detail.files.len() > 3 {
                        format!(
                            "{} changed ({}, ...)",
                            detail.files.len(),
                            detail.files[..3].join(", ")
                        )
                    } else {
                        format!(
                            "{} changed ({})",
                            detail.files.len(),
                            detail.files.join(", ")
                        )
                    };
                    out.push_str(&format!("  │  Files:   {}\n", files_str));
                }

                if !detail.symbols.is_empty() {
                    let syms_str = if detail.symbols.len() > 3 {
                        format!(
                            "{} touched ({}, ...)",
                            detail.symbols.len(),
                            detail.symbols[..3].join(", ")
                        )
                    } else {
                        format!(
                            "{} touched ({})",
                            detail.symbols.len(),
                            detail.symbols.join(", ")
                        )
                    };
                    out.push_str(&format!("  │  Symbols: {}\n", syms_str));
                }

                if let Some(summary) = &detail.commit.summary {
                    out.push_str(&format!("  │  Summary: {}\n", summary.replace('\n', " ")));
                }

                for fix in &detail.fixes {
                    out.push_str(&format!("  │  Fixes:   \x1b[32m{}\x1b[0m\n", fix));
                }

                out.push_str("  └──────────────────────────────────────────────────────────\n\n");
            }
        }

        out.push_str("═══════════════════════════════════════════════════════════════\n");
        out.push_str(&format!(
            "  Showing {} of {} total commit memories\n",
            self.shown_commits, self.total_commits
        ));

        out
    }
}

/// Numerical rank for risk level filtering
pub fn risk_rank(risk: &str) -> u8 {
    match risk.to_lowercase().as_str() {
        "critical" => 3,
        "high" => 2,
        "medium" | "med" => 1,
        "low" => 0,
        _ => 0,
    }
}

/// Format risk level with ANSI colors and optional warning icons
pub fn format_risk_badge(risk: Option<&str>) -> String {
    match risk.map(|r| r.to_lowercase()).as_deref() {
        Some("critical") => "\x1b[1;31m[Risk: CRITICAL 🚨]\x1b[0m".to_string(),
        Some("high") => "\x1b[31m[Risk: HIGH ⚠️]\x1b[0m".to_string(),
        Some("medium") | Some("med") => "\x1b[33m[Risk: MEDIUM ⚠️]\x1b[0m".to_string(),
        Some("low") => "\x1b[32m[Risk: LOW]\x1b[0m".to_string(),
        Some(other) => format!("[Risk: {}]", other.to_uppercase()),
        None => "[Risk: LOW]".to_string(),
    }
}

/// Normalize path to repo-relative Unix format
pub fn normalize_file_path(input: &str, repo_root: &Path) -> String {
    let trimmed = input.trim();
    let p = Path::new(trimmed);
    let abs_path = if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo_root.join(p)
    };

    if let Ok(rel) = abs_path.strip_prefix(repo_root) {
        rel.to_string_lossy().replace('\\', "/")
    } else {
        trimmed
            .replace('\\', "/")
            .trim_start_matches("./")
            .to_string()
    }
}
