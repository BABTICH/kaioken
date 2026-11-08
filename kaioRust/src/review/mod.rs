pub mod diff;
pub mod impact;
pub mod scoring;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::error::{KaiokenError, Result};
use crate::memory::MemoryEngine;
pub use diff::{
    AffectedSymbolInfo, BreakingChange, DiffAnalyzer, DiffHunk, ParsedDiff, ReviewFileSummary,
};
pub use impact::{AffectedCaller, FixSuggestion, ImpactAnalyzer, StaleDocInfo, TestGap};
pub use scoring::{RiskAssessment, RiskFactor, RiskLevel};

#[derive(clap::Args, Debug, Clone, Default)]
pub struct ReviewArgs {
    /// Review only staged changes
    #[arg(long)]
    pub staged: bool,

    /// Review a specific commit hash or reference
    #[arg(long)]
    pub commit: Option<String>,

    /// Diff against a specific branch or target ref
    #[arg(long)]
    pub against: Option<String>,

    /// Minimum severity to display (info, warn, breaking)
    #[arg(long, default_value = "info")]
    pub severity: String,

    /// Output as structured JSON
    #[arg(long)]
    pub json: bool,

    /// Generate remediation fix suggestions
    #[arg(long)]
    pub fix: bool,

    /// Override the workspace directory
    #[arg(long)]
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub changed_files: Vec<ReviewFileSummary>,
    pub breaking_changes: Vec<BreakingChange>,
    pub affected_callers: Vec<AffectedCaller>,
    pub test_gaps: Vec<TestGap>,
    pub stale_docs: Vec<StaleDocInfo>,
    pub risk: RiskAssessment,
    pub recommendations: Vec<String>,
    pub fix_suggestions: Vec<FixSuggestion>,
}

impl ReviewReport {
    /// Render human-readable ANSI terminal output report
    pub fn render_terminal(&self) -> String {
        let mut out = String::new();
        out.push_str("\x1b[1;36m🔍 kaioken review — Change Analysis Report\x1b[0m\n");
        out.push_str("═══════════════════════════════════════════════════════════════\n\n");

        if self.changed_files.is_empty() {
            out.push_str("  ✅ No changes detected to review (Working tree clean).\n\n");
            out.push_str("═══════════════════════════════════════════════════════════════\n");
            return out;
        }

        // 1. Changed Files
        out.push_str(&format!(
            "  \x1b[1m📁 Changed Files: {}\x1b[0m\n",
            self.changed_files.len()
        ));
        for (idx, cf) in self.changed_files.iter().enumerate() {
            let is_last = idx + 1 == self.changed_files.len();
            let branch = if is_last { "└──" } else { "├──" };
            out.push_str(&format!(
                "  {} {:<36} (+{}, -{})\n",
                branch, cf.path, cf.additions, cf.deletions
            ));
        }
        out.push('\n');

        // 2. Breaking Changes
        if !self.breaking_changes.is_empty() {
            out.push_str(&format!(
                "  \x1b[1;31m⚠️  Breaking Changes ({})\x1b[0m\n",
                self.breaking_changes.len()
            ));
            out.push_str("  ─────────────────────────\n");
            for bc in &self.breaking_changes {
                out.push_str(&format!("  • {}:L{}\n", bc.file_path, bc.line_number));
                if let Some(ref new_sig) = bc.new_signature {
                    out.push_str(&format!("    {} → {}\n", bc.old_signature, new_sig));
                } else {
                    out.push_str(&format!("    {}\n", bc.old_signature));
                }
                out.push_str(&format!("    {}\n", bc.description));

                // Find callers matching this breaking change
                let callers: Vec<&AffectedCaller> = self
                    .affected_callers
                    .iter()
                    .filter(|c| c.target_symbol == bc.symbol_name)
                    .collect();

                if !callers.is_empty() {
                    out.push_str(&format!("    Affected callers ({}):\n", callers.len()));
                    for c in callers {
                        let sym_str = c.caller_symbol.as_deref().unwrap_or("module");
                        let line_str = c
                            .caller_line
                            .map(|l| format!(":L{}", l))
                            .unwrap_or_default();
                        out.push_str(&format!(
                            "      → {}{} ({})\n",
                            c.caller_file, line_str, sym_str
                        ));
                    }
                }
            }
            out.push('\n');
        }

        // 3. Test Gaps
        if !self.test_gaps.is_empty() {
            out.push_str(&format!(
                "  \x1b[1;33m🧪 Test Gaps ({})\x1b[0m\n",
                self.test_gaps.len()
            ));
            out.push_str("  ─────────────────\n");
            for tg in &self.test_gaps {
                out.push_str(&format!("  • {} — No test file found\n", tg.file));
                out.push_str(&format!("    Suggestion: {}\n", tg.suggestion));
            }
            out.push('\n');
        }

        // 4. Documentation Impact
        if !self.stale_docs.is_empty() {
            out.push_str(&format!(
                "  \x1b[1;34m📝 Documentation Impact ({})\x1b[0m\n",
                self.stale_docs.len()
            ));
            out.push_str("  ────────────────────────────\n");
            for sd in &self.stale_docs {
                out.push_str(&format!("  • {} → {}\n", sd.doc_path, sd.reason));
            }
            out.push('\n');
        }

        // 5. Risk Assessment
        out.push_str("  \x1b[1m📊 Risk Assessment\x1b[0m\n");
        out.push_str("  ──────────────────\n");
        out.push_str(&format!(
            "  Score: {:.2} / 1.00  {}\n",
            self.risk.score,
            self.risk.level.badge()
        ));
        if !self.risk.factors.is_empty() {
            out.push_str("  Factors:\n");
            for f in &self.risk.factors {
                out.push_str(&format!("    {:+0.2}  {}\n", f.weight(), f.description()));
            }
        }
        out.push('\n');

        // 6. Fix Suggestions if present
        if !self.fix_suggestions.is_empty() {
            out.push_str("  \x1b[1;32m🛠️ Fix Suggestions\x1b[0m\n");
            out.push_str("  ─────────────────\n");
            for fix in &self.fix_suggestions {
                out.push_str(&format!("  • [{}] {}\n", fix.target, fix.description));
                out.push_str(&format!("    Action: {}\n", fix.action));
            }
            out.push('\n');
        }

        out.push_str("═══════════════════════════════════════════════════════════════\n");

        // 7. Recommendations
        if !self.recommendations.is_empty() {
            for rec in &self.recommendations {
                out.push_str(&format!("  💡 Recommendation: {}\n", rec));
            }
        }

        out
    }
}

pub struct ReviewEngine;

impl ReviewEngine {
    /// Execute the complete diff and intelligence analysis pipeline
    pub fn review(
        repo_root: &Path,
        memory: &MemoryEngine,
        args: &ReviewArgs,
    ) -> Result<ReviewReport> {
        // Step 1: Retrieve Git diff based on user flags
        let diff_text = Self::get_diff(repo_root, args)?;
        if diff_text.trim().is_empty() {
            return Ok(ReviewReport {
                changed_files: Vec::new(),
                breaking_changes: Vec::new(),
                affected_callers: Vec::new(),
                test_gaps: Vec::new(),
                stale_docs: Vec::new(),
                risk: RiskAssessment::calculate(Vec::new()),
                recommendations: vec!["Working tree is clean. No changes to review.".to_string()],
                fix_suggestions: Vec::new(),
            });
        }

        // Step 2: Parse changed files and hunks
        let parsed_diff = DiffAnalyzer::parse_unified_diff(&diff_text);

        // Step 3: Match symbols to hunks and identify breaking changes
        let (affected_symbols, mut breaking_changes) =
            DiffAnalyzer::match_symbols(repo_root, &parsed_diff, memory)?;

        // Step 4: Identify affected callers across the dependency graph
        let affected_callers = ImpactAnalyzer::find_affected_callers(&affected_symbols, memory)?;

        // Populate caller counts in breaking changes
        for bc in &mut breaking_changes {
            bc.affected_callers_count = affected_callers
                .iter()
                .filter(|c| c.target_symbol == bc.symbol_name)
                .count();
        }

        // Step 5: Check test coverage gaps
        let test_gaps = ImpactAnalyzer::check_test_coverage(repo_root, &parsed_diff.files, memory)?;

        // Step 6: Check documentation staleness
        let stale_docs = ImpactAnalyzer::check_doc_staleness(&parsed_diff.files, memory)?;

        // Step 7: Calculate risk factors and score
        let mut factors = Vec::new();

        // Breaking signature changes (+0.30)
        for bc in &breaking_changes {
            factors.push(RiskFactor::BreakingSignatureChange {
                symbol: bc.symbol_name.clone(),
                weight: 0.30,
            });
        }

        // Public API changes (+0.25)
        for aff in &affected_symbols {
            let is_pub = aff
                .symbol
                .visibility
                .as_deref()
                .map(|v| v == "pub" || v == "public")
                .unwrap_or(false);
            if is_pub
                && !breaking_changes
                    .iter()
                    .any(|b| b.symbol_name == aff.symbol.name)
            {
                factors.push(RiskFactor::PublicApiChange {
                    symbol: aff.symbol.name.clone(),
                    weight: 0.25,
                });
            }
        }

        // High fan-out symbols (>5 callers: +0.20, or callers present)
        for aff in &affected_symbols {
            let caller_count = affected_callers
                .iter()
                .filter(|c| c.target_symbol == aff.symbol.name)
                .count();
            if caller_count >= 5 {
                factors.push(RiskFactor::HighFanOutSymbol {
                    symbol: aff.symbol.name.clone(),
                    callers: caller_count,
                    weight: 0.20,
                });
            } else if caller_count > 0
                && !breaking_changes
                    .iter()
                    .any(|b| b.symbol_name == aff.symbol.name)
            {
                factors.push(RiskFactor::HighFanOutSymbol {
                    symbol: aff.symbol.name.clone(),
                    callers: caller_count,
                    weight: 0.10,
                });
            }
        }

        // Test gaps (+0.15)
        for tg in &test_gaps {
            factors.push(RiskFactor::NoTestCoverage {
                file: tg.file.clone(),
                weight: 0.15,
            });
        }

        // Previous unresolved failures in touched files (+0.15)
        for cf in &parsed_diff.files {
            if let Ok(failures) = memory.get_failures_for_file_path(&cf.path) {
                for f in failures {
                    if f.status.as_deref() == Some("unresolved") {
                        factors.push(RiskFactor::PreviousFailure {
                            error_sig: format!("{}: {}", f.error_type, f.message),
                            weight: 0.15,
                        });
                        break;
                    }
                }
            }
        }

        // Stale documentation (+0.10)
        if !stale_docs.is_empty() {
            factors.push(RiskFactor::StaleDocumentation {
                doc_path: format!("{} doc(s) affected", stale_docs.len()),
                weight: 0.10,
            });
        }

        // Large change set (>10 files: +0.10)
        if parsed_diff.files.len() > 10 {
            factors.push(RiskFactor::LargeChangeSet {
                files_changed: parsed_diff.files.len(),
                weight: 0.10,
            });
        }

        let risk = RiskAssessment::calculate(factors);

        // Step 8: Build recommendations
        let mut recommendations = Vec::new();
        if !breaking_changes.is_empty() {
            recommendations.push(
                "Update affected callers before committing to prevent build breaks.".to_string(),
            );
        }
        if !stale_docs.is_empty() {
            recommendations.push(
                "Run `kaioken docs generate --stale` after commit to keep documentation synced."
                    .to_string(),
            );
        }
        if !test_gaps.is_empty() {
            recommendations
                .push("Add test coverage for modified modules before merging.".to_string());
        }
        if recommendations.is_empty() {
            recommendations.push("Changes look safe to commit.".to_string());
        }

        // Step 9: Build fix suggestions if --fix was requested
        let mut fix_suggestions = Vec::new();
        if args.fix {
            for bc in &breaking_changes {
                fix_suggestions.push(FixSuggestion {
                    target: bc.symbol_name.clone(),
                    description: format!(
                        "Update callers of `{}` to match signature: {}",
                        bc.symbol_name,
                        bc.new_signature.as_deref().unwrap_or(&bc.old_signature)
                    ),
                    action: format!("Check callers in {}", bc.file_path),
                });
            }
            for tg in &test_gaps {
                fix_suggestions.push(FixSuggestion {
                    target: tg.file.clone(),
                    description: format!("Generate missing test file for `{}`", tg.file),
                    action: tg.suggestion.clone(),
                });
            }
        }

        Ok(ReviewReport {
            changed_files: parsed_diff.files,
            breaking_changes,
            affected_callers,
            test_gaps,
            stale_docs,
            risk,
            recommendations,
            fix_suggestions,
        })
    }

    /// Retrieve diff text from git based on flags
    pub fn get_diff(repo_root: &Path, args: &ReviewArgs) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(repo_root);

        if let Some(ref commit) = args.commit {
            return crate::scanner::git::GitScanner::get_commit_diff(repo_root, commit);
        } else if let Some(ref against) = args.against {
            cmd.arg("diff").arg(against);
        } else if args.staged {
            cmd.arg("diff").arg("--staged");
        } else {
            // Default: diff against HEAD to include both staged and unstaged changes
            cmd.arg("diff").arg("HEAD");
        }

        let output = cmd
            .output()
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to run git diff: {}", e)))?;

        if !output.status.success() {
            // If git diff HEAD failed (e.g. fresh repository with no HEAD yet), try git diff alone
            let mut fallback_cmd = Command::new("git");
            fallback_cmd.arg("-C").arg(repo_root).arg("diff");
            if let Ok(fallback_out) = fallback_cmd.output() {
                if fallback_out.status.success() {
                    return Ok(String::from_utf8_lossy(&fallback_out.stdout).to_string());
                }
            }
            return Ok(String::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
