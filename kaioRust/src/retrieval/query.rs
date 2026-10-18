use std::path::Path;
use crate::config::Config;
use crate::error::Result;
use crate::memory::MemoryEngine;
use crate::retrieval::budget::{BudgetConfig, BudgetManager, TokenEstimator};
use crate::retrieval::cache::RetrievalCacheStore;
use crate::retrieval::context_packet::{
    CommitSummarySnippet, ContextPacket, DecisionSnippet, FileContextSnippet,
    SymbolContextSummary,
};
use crate::retrieval::doc_lookup::DocLookup;
use crate::retrieval::failure_lookup::FailureLookup;
use crate::retrieval::graph_query::GraphQueryEngine;
use crate::safety::redact::RedactionEngine;
use crate::scanner::GitScanner;

#[derive(Debug, Clone)]
pub struct RetrievalOptions {
    pub budget_tokens: Option<usize>,
    pub no_cache: bool,
    pub max_depth: usize,
    pub include_commits: bool,
}

impl Default for RetrievalOptions {
    fn default() -> Self {
        Self {
            budget_tokens: None,
            no_cache: false,
            max_depth: 1,
            include_commits: true,
        }
    }
}

pub struct RetrievalEngine<'a> {
    memory: &'a MemoryEngine,
    config: &'a Config,
    repo_root: &'a Path,
    redactor: RedactionEngine,
}

impl<'a> RetrievalEngine<'a> {
    pub fn new(memory: &'a MemoryEngine, config: &'a Config, repo_root: &'a Path) -> Self {
        Self {
            memory,
            config,
            repo_root,
            redactor: RedactionEngine::new(),
        }
    }

    /// Assembles an immutable, sanitized, token-budgeted ContextPacket for a task.
    pub fn retrieve_context(&self, task_prompt: &str, opts: &RetrievalOptions) -> Result<ContextPacket> {
        let sanitized_task = self.redactor.redact(task_prompt);
        let budget_limit = opts.budget_tokens.unwrap_or(8000);

        // 1. Check Retrieval Cache
        let head_commit = GitScanner::get_head_commit(self.repo_root)
            .ok()
            .flatten()
            .map(|c| c.hash);
        let cache_key = RetrievalCacheStore::compute_key(
            &sanitized_task,
            head_commit.as_deref(),
            &format!("budget={}:depth={}", budget_limit, opts.max_depth),
        );

        let cache_store = RetrievalCacheStore::new(self.memory);
        if !opts.no_cache {
            if let Ok(Some(cached)) = cache_store.get(&cache_key, self.repo_root) {
                return Ok(cached);
            }
        }

        // 2. Query Decomposition & Path/Symbol Extraction
        let (mentioned_paths, mentioned_symbols) = self.decompose_task(&sanitized_task)?;

        // 3. Changed Files Discovery (Git Layer integration)
        let changed_files = GitScanner::get_working_tree_changed_files(self.repo_root).unwrap_or_default();

        // 4. Resolve Target Files & Build File Snippets
        let mut target_file_ids = Vec::new();
        let mut target_file_records = Vec::new();

        // Add explicitly mentioned files
        for p in &mentioned_paths {
            if let Ok(Some(rec)) = self.memory.get_file_by_path(p) {
                if let Some(id) = rec.id {
                    if !target_file_ids.contains(&id) {
                        target_file_ids.push(id);
                        target_file_records.push(rec);
                    }
                }
            }
        }

        // Add git changed files
        for cf in &changed_files {
            let path_str = cf.path.to_string_lossy().replace('\\', "/");
            if let Ok(Some(rec)) = self.memory.get_file_by_path(&path_str) {
                if let Some(id) = rec.id {
                    if !target_file_ids.contains(&id) {
                        target_file_ids.push(id);
                        target_file_records.push(rec);
                    }
                }
            }
        }

        // Build FileContextSnippets
        let mut relevant_files = Vec::new();
        for rec in &target_file_records {
            if RedactionEngine::is_sensitive_file(Path::new(&rec.path)) {
                continue;
            }

            let disk_path = self.repo_root.join(&rec.path);
            if !disk_path.exists() {
                continue;
            }

            if let Ok(raw_content) = std::fs::read_to_string(&disk_path) {
                let sanitized_content = self.redactor.redact(&raw_content);
                let line_count = sanitized_content.lines().count().max(1);
                let token_estimate = TokenEstimator::estimate_tokens(&sanitized_content) + 20;

                relevant_files.push(FileContextSnippet {
                    path: rec.path.clone(),
                    language: rec.language.clone().unwrap_or_else(|| "text".to_string()),
                    line_start: 1,
                    line_end: line_count,
                    content: sanitized_content,
                    source_hash: rec.source_hash.clone().unwrap_or_default(),
                    token_estimate,
                });
            }
        }

        // 5. AST Symbol Extraction & Lookup
        let mut target_symbol_ids = Vec::new();
        let mut relevant_symbols = Vec::new();

        // Mentioned symbols
        for sym_name in &mentioned_symbols {
            if let Ok(sym_list) = self.memory.find_symbols_by_name(sym_name) {
                for s in sym_list {
                    if let Some(sid) = s.id {
                        if !target_symbol_ids.contains(&sid) {
                            target_symbol_ids.push(sid);
                            let file_path = self.memory.get_file_by_id(s.file_id)
                                .ok()
                                .flatten()
                                .map(|f| f.path)
                                .unwrap_or_default();

                            let sig = self.redactor.redact(s.signature.as_deref().unwrap_or(""));
                            let cost = TokenEstimator::estimate_tokens(&sig) + 15;
                            relevant_symbols.push(SymbolContextSummary {
                                name: s.name,
                                kind: s.kind,
                                file_path,
                                signature: sig,
                                visibility: s.visibility.unwrap_or_default(),
                                line_start: s.line_start,
                                line_end: s.line_end,
                                token_estimate: cost,
                            });
                        }
                    }
                }
            }
        }

        // Also include symbols from the primary target file if symbol list is small
        if relevant_symbols.len() < 10 {
            for &fid in &target_file_ids {
                if let Ok(syms) = self.memory.get_symbols_for_file(fid) {
                    for s in syms {
                        if let Some(sid) = s.id {
                            if !target_symbol_ids.contains(&sid) {
                                target_symbol_ids.push(sid);
                                let file_path = self.memory.get_file_by_id(s.file_id)
                                    .ok()
                                    .flatten()
                                    .map(|f| f.path)
                                    .unwrap_or_default();

                                let sig = self.redactor.redact(s.signature.as_deref().unwrap_or(""));
                                let cost = TokenEstimator::estimate_tokens(&sig) + 15;
                                relevant_symbols.push(SymbolContextSummary {
                                    name: s.name,
                                    kind: s.kind,
                                    file_path,
                                    signature: sig,
                                    visibility: s.visibility.unwrap_or_default(),
                                    line_start: s.line_start,
                                    line_end: s.line_end,
                                    token_estimate: cost,
                                });
                                if relevant_symbols.len() >= 20 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 6. Graph Neighbor Traversal
        let graph_engine = GraphQueryEngine::new(self.memory);
        let dependency_graph_subset = graph_engine.query_subgraph(
            &target_symbol_ids,
            &target_file_ids,
            opts.max_depth,
        ).unwrap_or_default();

        // 7. Failure Memory Lookup
        let failure_engine = FailureLookup::new(self.memory);
        let related_failures = failure_engine.find_failures(
            &target_file_ids,
            &target_symbol_ids,
            &sanitized_task,
            5,
        ).unwrap_or_default();

        // 8. Documentation Staleness Check
        let doc_engine = DocLookup::new(self.memory, self.repo_root);
        let stale_docs_warnings = doc_engine.check_doc_staleness(&target_file_ids).unwrap_or_default();

        // 9. Architectural Decisions (ADRs)
        let decisions = self.memory.list_decisions(Some("accepted")).unwrap_or_default();
        let architectural_decisions: Vec<DecisionSnippet> = decisions.into_iter().map(|d| {
            DecisionSnippet {
                title: d.title,
                description: self.redactor.redact(&d.description),
                status: d.status,
            }
        }).collect();

        // 10. Recent Commits
        let mut recent_related_commits = Vec::new();
        if opts.include_commits {
            for &fid in &target_file_ids {
                if let Ok(commits) = self.memory.get_commits_for_file(fid, 3) {
                    for c in commits {
                        if !recent_related_commits.iter().any(|rc: &CommitSummarySnippet| rc.hash == c.hash) {
                            recent_related_commits.push(CommitSummarySnippet {
                                hash: c.hash,
                                message: self.redactor.redact(&c.message),
                                author: c.author.unwrap_or_default(),
                                created_at: c.created_at.unwrap_or_default(),
                                touched_files: vec![],
                            });
                        }
                    }
                }
            }
        }

        // 11. Recommended Verification Commands
        let mut recommended_verification_commands = Vec::new();
        if self.config.project.languages.iter().any(|l| l == "rust") {
            recommended_verification_commands.push("cargo check".to_string());
            if !dependency_graph_subset.affected_tests.is_empty() {
                for t in &dependency_graph_subset.affected_tests {
                    if let Some(stem) = Path::new(t).file_stem().and_then(|s| s.to_str()) {
                        let test_name = stem.trim_start_matches("test_");
                        recommended_verification_commands.push(format!("cargo test --test {}", test_name));
                    }
                }
            } else {
                recommended_verification_commands.push("cargo test".to_string());
            }
        }

        // 12. Assemble Initial ContextPacket
        let mut packet = ContextPacket {
            task_summary: sanitized_task,
            relevant_files,
            relevant_symbols,
            dependency_graph_subset,
            recent_related_commits,
            related_failures,
            stale_docs_warnings,
            architectural_decisions,
            recommended_verification_commands,
            token_budget_report: crate::retrieval::context_packet::TokenBudgetReport {
                section_tokens: std::collections::HashMap::new(),
                total_tokens: 0,
                budget_limit,
                truncated: false,
            },
        };

        // 13. Token Budget Enforcement & Priority Truncation
        let budget_config = BudgetConfig {
            total_budget_limit: budget_limit,
            ..Default::default()
        };
        let budget_manager = BudgetManager::new(budget_config);
        budget_manager.enforce_budget(&mut packet);

        // 14. Save in SQLite Retrieval Cache (TTL = 10 minutes = 600s)
        let _ = cache_store.put(&cache_key, &packet, 600);

        Ok(packet)
    }

    /// Helper to decompose task string into mentioned file paths and symbol names.
    fn decompose_task(&self, task: &str) -> Result<(Vec<String>, Vec<String>)> {
        let mut paths = Vec::new();
        let mut symbols = Vec::new();

        let all_files = self.memory.list_files()?;
        for f in &all_files {
            let path_str = &f.path;
            let file_name = Path::new(path_str).file_name().and_then(|n| n.to_str()).unwrap_or("");

            if task.contains(path_str) || (!file_name.is_empty() && task.contains(file_name)) {
                paths.push(path_str.clone());
            }
        }

        // Extract potential symbol words: alphanumeric + underscore, length >= 4
        let words = task.split(|c: char| !c.is_alphanumeric() && c != '_');
        for w in words {
            let candidate = w.trim();
            if candidate.len() >= 4 && !candidate.contains('.') {
                if let Ok(matches) = self.memory.find_symbols_by_name(candidate) {
                    if !matches.is_empty() && !symbols.contains(&candidate.to_string()) {
                        symbols.push(candidate.to_string());
                    }
                }
            }
        }

        Ok((paths, symbols))
    }
}
