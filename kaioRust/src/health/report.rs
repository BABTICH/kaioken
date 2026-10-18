use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub score: u32, // 0 - 100
    pub grade: String, // "A", "B", "C", "D", "F"
    pub deductions: Vec<ScoreDeduction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDeduction {
    pub reason: String,
    pub points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub generated_at: String,
    pub repo_root: String,
    pub score: HealthScore,
    pub overview: RepoOverview,
    pub architecture: ArchitectureSummary,
    pub doc_health: DocHealthSummary,
    pub dead_code: DeadCodeSummary,
    pub test_coverage: TestCoverageSummary,
    pub graph_integrity: GraphIntegritySummary,
    pub failure_memory: FailureMemorySummary,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoOverview {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_edges: usize,
    pub total_docs: usize,
    pub total_failures: usize,
    pub languages: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureSummary {
    pub modules: Vec<ModuleBreakdown>,
    pub core_dependencies: Vec<String>,
    pub max_dependency_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleBreakdown {
    pub module_name: String,
    pub file_count: usize,
    pub symbol_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocHealthSummary {
    pub total_docs: usize,
    pub fresh_count: usize,
    pub stale_count: usize,
    pub maybe_stale_count: usize,
    pub missing_docs_count: usize,
    pub stale_doc_paths: Vec<String>,
    pub missing_doc_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeSummary {
    pub unreferenced_symbols: Vec<UnreferencedSymbol>,
    pub orphan_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreferencedSymbol {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageSummary {
    pub untested_files: Vec<String>,
    pub untested_public_symbols: Vec<String>,
    pub test_file_count: usize,
    pub implementation_file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphIntegritySummary {
    pub total_edges: usize,
    pub unresolved_imports: Vec<(String, String)>,
    pub broken_link_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMemorySummary {
    pub unresolved_count: usize,
    pub resolved_count: usize,
    pub unresolved_failures: Vec<String>,
}

impl HealthReport {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();

        out.push_str("# 🏥 KAIOKEN Repository Health Audit Report\n\n");
        out.push_str(&format!("- **Timestamp**: {}\n", self.generated_at));
        out.push_str(&format!("- **Repository**: `{}`\n", self.repo_root));
        out.push_str(&format!(
            "- **Overall Health Score**: **{}/100** (Grade: **{}**)\n\n",
            self.score.score, self.score.grade
        ));

        // Deductions breakdown
        if !self.score.deductions.is_empty() {
            out.push_str("### 🔻 Score Deductions\n");
            for d in &self.score.deductions {
                out.push_str(&format!("- [-{} pts] {}\n", d.points, d.reason));
            }
            out.push('\n');
        }

        // 1. Overview
        out.push_str("## 1. Repository Inventory Overview\n\n");
        out.push_str("| Metric | Count |\n");
        out.push_str("|---|---|\n");
        out.push_str(&format!("| Total Tracked Files | {} |\n", self.overview.total_files));
        out.push_str(&format!("| Extracted AST Symbols | {} |\n", self.overview.total_symbols));
        out.push_str(&format!("| Dependency Graph Edges | {} |\n", self.overview.total_edges));
        out.push_str(&format!("| Mirrored Documentation Files | {} |\n", self.overview.total_docs));
        out.push_str(&format!("| Failure Memory Records | {} |\n\n", self.overview.total_failures));

        if !self.overview.languages.is_empty() {
            out.push_str("**Language Breakdown**:\n");
            for (lang, count) in &self.overview.languages {
                out.push_str(&format!("- `{}`: {} file(s)\n", lang, count));
            }
            out.push('\n');
        }

        // 2. Architecture Summary
        out.push_str("## 2. Architecture & Modules\n\n");
        if !self.architecture.modules.is_empty() {
            out.push_str("| Module Area | Files | AST Symbols |\n");
            out.push_str("|---|---|---|\n");
            for m in &self.architecture.modules {
                out.push_str(&format!("| `{}` | {} | {} |\n", m.module_name, m.file_count, m.symbol_count));
            }
            out.push('\n');
        }
        if !self.architecture.core_dependencies.is_empty() {
            out.push_str(&format!("- **Core Shared Dependencies**: {}\n\n", self.architecture.core_dependencies.join(", ")));
        }

        // 3. Documentation Health
        out.push_str("## 3. Documentation Mirror Health\n\n");
        out.push_str(&format!("- **Fresh / Synchronized**: {}\n", self.doc_health.fresh_count));
        out.push_str(&format!("- **Stale Docs**: {}\n", self.doc_health.stale_count));
        out.push_str(&format!("- **Maybe-Stale Dependents**: {}\n", self.doc_health.maybe_stale_count));
        out.push_str(&format!("- **Missing Docs**: {}\n\n", self.doc_health.missing_docs_count));

        if !self.doc_health.stale_doc_paths.is_empty() {
            out.push_str("### ⚠️ Stale Documentation Files\n");
            for p in &self.doc_health.stale_doc_paths {
                out.push_str(&format!("- `{}` (needs regeneration)\n", p));
            }
            out.push('\n');
        }

        if !self.doc_health.missing_doc_files.is_empty() {
            out.push_str("### 📝 Source Files Missing Documentation\n");
            for p in &self.doc_health.missing_doc_files {
                out.push_str(&format!("- `{}`\n", p));
            }
            out.push('\n');
        }

        // 4. Dead Code Hints
        out.push_str("## 4. Dead Code Detection & Hints\n\n");
        if self.dead_code.unreferenced_symbols.is_empty() && self.dead_code.orphan_files.is_empty() {
            out.push_str("✅ No obvious unreferenced private symbols or orphan files detected.\n\n");
        } else {
            if !self.dead_code.unreferenced_symbols.is_empty() {
                out.push_str(&format!("### 🔍 Unreferenced Internal Symbols ({} detected)\n", self.dead_code.unreferenced_symbols.len()));
                for sym in &self.dead_code.unreferenced_symbols {
                    out.push_str(&format!("- `{}` (`{}`) in `{}:{}`\n", sym.name, sym.kind, sym.file_path, sym.line));
                }
                out.push('\n');
            }
            if !self.dead_code.orphan_files.is_empty() {
                out.push_str("### 📁 Orphan Files (Not imported by any module)\n");
                for f in &self.dead_code.orphan_files {
                    out.push_str(&format!("- `{}`\n", f));
                }
                out.push('\n');
            }
        }

        // 5. Test Coverage Hints
        out.push_str("## 5. Test Coverage Hints\n\n");
        out.push_str(&format!("- **Test Files**: {} | **Implementation Files**: {}\n\n", self.test_coverage.test_file_count, self.test_coverage.implementation_file_count));
        if !self.test_coverage.untested_files.is_empty() {
            out.push_str("### 🧪 Source Files Lacking Dedicated Unit Tests\n");
            for f in &self.test_coverage.untested_files {
                out.push_str(&format!("- `{}`\n", f));
            }
            out.push('\n');
        }
        if !self.test_coverage.untested_public_symbols.is_empty() {
            out.push_str("### 🧩 Public Symbols Without Test Invocations\n");
            for s in &self.test_coverage.untested_public_symbols {
                out.push_str(&format!("- `{}`\n", s));
            }
            out.push('\n');
        }

        // 6. Graph Integrity
        out.push_str("## 6. Dependency Graph & Link Integrity\n\n");
        if self.graph_integrity.unresolved_imports.is_empty() {
            out.push_str("✅ All AST imports resolved cleanly across the workspace graph.\n\n");
        } else {
            out.push_str(&format!("⚠️ **Unresolved Imports / Broken Links** ({}):\n", self.graph_integrity.unresolved_imports.len()));
            for (file, raw) in &self.graph_integrity.unresolved_imports {
                out.push_str(&format!("- In `{}`: `{}`\n", file, raw));
            }
            out.push('\n');
        }

        // 7. Failure Memory
        out.push_str("## 7. Failure Memory Status\n\n");
        out.push_str(&format!("- **Unresolved Failures**: {}\n", self.failure_memory.unresolved_count));
        out.push_str(&format!("- **Resolved Historical Fixes**: {}\n\n", self.failure_memory.resolved_count));
        if !self.failure_memory.unresolved_failures.is_empty() {
            out.push_str("### 🚨 Active Unresolved Failures\n");
            for f in &self.failure_memory.unresolved_failures {
                out.push_str(&format!("- {}\n", f));
            }
            out.push('\n');
        }

        // 8. Recommendations
        out.push_str("## 8. Actionable Recommendations\n\n");
        if self.recommendations.is_empty() {
            out.push_str("🎉 Repository health is in excellent state! No immediate remediation needed.\n");
        } else {
            for (i, rec) in self.recommendations.iter().enumerate() {
                out.push_str(&format!("{}. {}\n", i + 1, rec));
            }
        }

        out
    }
}
