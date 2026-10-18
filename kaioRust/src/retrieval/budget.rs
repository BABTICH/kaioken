use std::collections::HashMap;
use crate::retrieval::context_packet::{
    ContextPacket, FileContextSnippet, SymbolContextSummary,
    FailureMemorySnippet, CommitSummarySnippet, DecisionSnippet
};

pub struct TokenEstimator;

impl TokenEstimator {
    /// Fast estimation of token count based on byte/character length and whitespace.
    /// Approximately 4 characters per token in UTF-8 code and text.
    pub fn estimate_tokens(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        // Count chars and adjust for punctuation / whitespace
        let char_count = text.chars().count();
        (char_count + 3) / 4
    }
}

#[derive(Debug, Clone)]
pub struct BudgetConfig {
    pub max_files_tokens: usize,
    pub max_symbols_tokens: usize,
    pub max_graph_tokens: usize,
    pub max_failures_tokens: usize,
    pub max_docs_adrs_tokens: usize,
    pub total_budget_limit: usize,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_files_tokens: 4000,
            max_symbols_tokens: 1500,
            max_graph_tokens: 800,
            max_failures_tokens: 600,
            max_docs_adrs_tokens: 600,
            total_budget_limit: 8000,
        }
    }
}

pub struct BudgetManager {
    pub config: BudgetConfig,
}

impl Default for BudgetManager {
    fn default() -> Self {
        Self::new(BudgetConfig::default())
    }
}

impl BudgetManager {
    pub fn new(config: BudgetConfig) -> Self {
        Self { config }
    }

    /// Applies quota enforcement and priority truncation on a mutable ContextPacket.
    pub fn enforce_budget(&self, packet: &mut ContextPacket) {
        let mut truncated = false;
        let mut section_tokens = HashMap::new();

        // 1. Enforce Failures Section (Protected: max 3-5 latest failures, up to quota)
        let mut failures_tokens = 0;
        let mut kept_failures: Vec<FailureMemorySnippet> = Vec::new();
        for f in packet.related_failures.drain(..) {
            let cost = TokenEstimator::estimate_tokens(&f.message)
                + f.fix_summary.as_ref().map(|s| TokenEstimator::estimate_tokens(s)).unwrap_or(0)
                + 20;
            if failures_tokens + cost <= self.config.max_failures_tokens || kept_failures.is_empty() {
                failures_tokens += cost;
                kept_failures.push(f);
            } else {
                truncated = true;
            }
        }
        packet.related_failures = kept_failures;
        section_tokens.insert("failures".to_string(), failures_tokens);

        // 2. Enforce Symbols Section
        let mut symbols_tokens = 0;
        let mut kept_symbols: Vec<SymbolContextSummary> = Vec::new();
        for mut s in packet.relevant_symbols.drain(..) {
            let cost = TokenEstimator::estimate_tokens(&s.signature) + 15;
            s.token_estimate = cost;
            if symbols_tokens + cost <= self.config.max_symbols_tokens || kept_symbols.is_empty() {
                symbols_tokens += cost;
                kept_symbols.push(s);
            } else {
                truncated = true;
            }
        }
        packet.relevant_symbols = kept_symbols;
        section_tokens.insert("symbols".to_string(), symbols_tokens);

        // 3. Enforce Files Section
        let mut files_tokens = 0;
        let mut kept_files: Vec<FileContextSnippet> = Vec::new();
        for mut file in packet.relevant_files.drain(..) {
            let cost = TokenEstimator::estimate_tokens(&file.content) + 20;
            file.token_estimate = cost;
            if files_tokens + cost <= self.config.max_files_tokens || kept_files.is_empty() {
                files_tokens += cost;
                kept_files.push(file);
            } else {
                // If a single file exceeds budget, truncate its content lines
                let remaining_budget = self.config.max_files_tokens.saturating_sub(files_tokens);
                if remaining_budget >= 100 {
                    let max_chars = remaining_budget * 4;
                    let truncated_content: String = file.content.chars().take(max_chars).collect();
                    file.content = format!("{}\n// ... [TRUNCATED DUE TO TOKEN BUDGET]", truncated_content);
                    let new_cost = TokenEstimator::estimate_tokens(&file.content) + 20;
                    file.token_estimate = new_cost;
                    files_tokens += new_cost;
                    kept_files.push(file);
                }
                truncated = true;
            }
        }
        packet.relevant_files = kept_files;
        section_tokens.insert("files".to_string(), files_tokens);

        // 4. Enforce Graph Section
        let mut graph_tokens = 0;
        graph_tokens += packet.dependency_graph_subset.direct_callers.iter().map(|s| TokenEstimator::estimate_tokens(s)).sum::<usize>();
        graph_tokens += packet.dependency_graph_subset.direct_callees.iter().map(|s| TokenEstimator::estimate_tokens(s)).sum::<usize>();
        graph_tokens += packet.dependency_graph_subset.affected_tests.iter().map(|s| TokenEstimator::estimate_tokens(s)).sum::<usize>();
        if graph_tokens > self.config.max_graph_tokens {
            packet.dependency_graph_subset.direct_callers.truncate(10);
            packet.dependency_graph_subset.direct_callees.truncate(10);
            packet.dependency_graph_subset.affected_tests.truncate(10);
            graph_tokens = self.config.max_graph_tokens;
            truncated = true;
        }
        section_tokens.insert("graph".to_string(), graph_tokens);

        // 5. Enforce Docs and ADRs Section
        let mut adr_tokens = 0;
        let mut kept_adrs: Vec<DecisionSnippet> = Vec::new();
        for adr in packet.architectural_decisions.drain(..) {
            let cost = TokenEstimator::estimate_tokens(&adr.description) + 15;
            if adr_tokens + cost <= self.config.max_docs_adrs_tokens || kept_adrs.is_empty() {
                adr_tokens += cost;
                kept_adrs.push(adr);
            } else {
                truncated = true;
            }
        }
        packet.architectural_decisions = kept_adrs;
        section_tokens.insert("adrs".to_string(), adr_tokens);

        // 6. Commits Section
        let mut commits_tokens = 0;
        let mut kept_commits: Vec<CommitSummarySnippet> = Vec::new();
        for commit in packet.recent_related_commits.drain(..) {
            let cost = TokenEstimator::estimate_tokens(&commit.message) + 20;
            if commits_tokens + cost <= 500 {
                commits_tokens += cost;
                kept_commits.push(commit);
            } else {
                truncated = true;
            }
        }
        packet.recent_related_commits = kept_commits;
        section_tokens.insert("commits".to_string(), commits_tokens);

        // Compute total tokens across all sections
        let mut total_tokens = failures_tokens + symbols_tokens + files_tokens + graph_tokens + adr_tokens + commits_tokens;

        // 7. Global Hard Ceiling Check: If total_tokens > total_budget_limit, apply Priority Dropping Sequence
        if total_tokens > self.config.total_budget_limit {
            truncated = true;
            
            // Priority 1: Drop ADRs
            if !packet.architectural_decisions.is_empty() {
                total_tokens -= adr_tokens;
                section_tokens.insert("adrs".to_string(), 0);
                packet.architectural_decisions.clear();
            }

            // Priority 2: Drop Commits
            if total_tokens > self.config.total_budget_limit && !packet.recent_related_commits.is_empty() {
                total_tokens -= commits_tokens;
                section_tokens.insert("commits".to_string(), 0);
                packet.recent_related_commits.clear();
            }

            // Priority 3: Trim lower priority files (keeping first/most relevant)
            while total_tokens > self.config.total_budget_limit && packet.relevant_files.len() > 1 {
                if let Some(removed) = packet.relevant_files.pop() {
                    total_tokens = total_tokens.saturating_sub(removed.token_estimate);
                }
            }

            // Priority 4: Further truncate last file snippet if still over budget
            if total_tokens > self.config.total_budget_limit && !packet.relevant_files.is_empty() {
                let excess = total_tokens - self.config.total_budget_limit;
                let file = &mut packet.relevant_files[0];
                let chars_to_drop = excess * 4;
                let current_chars = file.content.chars().count();
                if current_chars > chars_to_drop + 100 {
                    let keep_chars = current_chars - chars_to_drop;
                    let truncated_content: String = file.content.chars().take(keep_chars).collect();
                    file.content = format!("{}\n// ... [TRUNCATED TO FIT GLOBAL CEILING]", truncated_content);
                    file.token_estimate = TokenEstimator::estimate_tokens(&file.content) + 20;
                }
                total_tokens = self.config.total_budget_limit;
            }
        }

        packet.token_budget_report = crate::retrieval::context_packet::TokenBudgetReport {
            section_tokens,
            total_tokens,
            budget_limit: self.config.total_budget_limit,
            truncated,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimator() {
        let text = "Hello world! This is a test.";
        let tokens = TokenEstimator::estimate_tokens(text);
        assert!(tokens > 0);
        assert_eq!(tokens, (text.chars().count() + 3) / 4);
    }

    #[test]
    fn test_budget_manager_truncation() {
        let mut packet = ContextPacket::empty("test task", 500);
        let config = BudgetConfig {
            max_files_tokens: 100,
            max_symbols_tokens: 50,
            max_graph_tokens: 50,
            max_failures_tokens: 50,
            max_docs_adrs_tokens: 50,
            total_budget_limit: 200,
        };

        let manager = BudgetManager::new(config);

        packet.relevant_files.push(FileContextSnippet {
            path: "src/large.rs".to_string(),
            language: "rust".to_string(),
            line_start: 1,
            line_end: 200,
            content: "a".repeat(2000), // ~500 tokens
            source_hash: "hash".to_string(),
            token_estimate: 500,
        });

        manager.enforce_budget(&mut packet);

        assert!(packet.token_budget_report.truncated);
        assert!(packet.token_budget_report.total_tokens <= 200);
    }
}
