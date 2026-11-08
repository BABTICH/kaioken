pub mod checks;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckStatus {
    pub fn badge(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "✅",
            CheckStatus::Warn => "⚠️",
            CheckStatus::Fail => "❌",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub display_name: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverallStatus {
    Ready,
    ReadyLimited,
    NotReady,
}

impl OverallStatus {
    pub fn display_str(&self) -> &'static str {
        match self {
            OverallStatus::Ready => "READY (all operations available)",
            OverallStatus::ReadyLimited => "READY (non-agent operations available)",
            OverallStatus::NotReady => "NOT READY (critical prerequisites missing)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub pass: usize,
    pub warn: usize,
    pub fail: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub checks: Vec<DiagnosticCheck>,
    pub summary: DiagnosticSummary,
    pub overall: OverallStatus,
}

impl DiagnosticReport {
    pub fn from_checks(checks: Vec<DiagnosticCheck>) -> Self {
        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;

        for check in &checks {
            match check.status {
                CheckStatus::Pass => pass += 1,
                CheckStatus::Warn => warn += 1,
                CheckStatus::Fail => fail += 1,
            }
        }

        let overall = if fail > 0 {
            OverallStatus::NotReady
        } else if warn > 0 {
            OverallStatus::ReadyLimited
        } else {
            OverallStatus::Ready
        };

        Self {
            checks,
            summary: DiagnosticSummary { pass, warn, fail },
            overall,
        }
    }

    pub fn has_failures(&self) -> bool {
        self.summary.fail > 0
    }

    pub fn render_terminal(&self) -> String {
        let mut out = String::new();
        out.push_str("🔍 kaioken doctor — Environment Diagnostic Report\n");
        out.push_str("══════════════════════════════════════════════════\n\n");

        for check in &self.checks {
            out.push_str(&format!(
                "  {}  {:<24} {}\n",
                check.status.badge(),
                check.display_name,
                check.detail
            ));
        }

        out.push_str("\n══════════════════════════════════════════════════\n");
        out.push_str(&format!(
            "  Result: {} passed · {} warnings · {} failed\n",
            self.summary.pass, self.summary.warn, self.summary.fail
        ));
        out.push_str(&format!("  Status: {}\n", self.overall.display_str()));

        out
    }
}

pub struct DoctorEngine;

impl DoctorEngine {
    /// Execute the full diagnostic suite or a filtered subset
    pub fn run(workspace_root: &Path, only_filter: Option<&str>) -> Result<DiagnosticReport> {
        let filters: Option<HashSet<String>> = only_filter.map(|f| {
            f.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        });

        let mut checks = Vec::with_capacity(13);

        macro_rules! execute_check {
            ($name:expr, $func:expr) => {
                if filters.as_ref().map_or(true, |f| f.contains($name)) {
                    checks.push($func);
                }
            };
        }

        // 1. Git Availability
        execute_check!("git_availability", checks::check_git_availability());

        // 2. Git Repository
        execute_check!(
            "git_repository",
            checks::check_git_repository(workspace_root)
        );

        // 3. Rust Toolchain
        execute_check!("rust_toolchain", checks::check_rust_toolchain());

        // 4. SQLite Runtime
        execute_check!(
            "sqlite_runtime",
            checks::check_sqlite_runtime(workspace_root)
        );

        // 5. SQLite Schema
        execute_check!("sqlite_schema", checks::check_sqlite_schema(workspace_root));

        // 6. Tree-sitter Rust
        execute_check!("treesitter_rust", checks::check_treesitter_rust());

        // 7. Tree-sitter TS/JS
        execute_check!("treesitter_ts_js", checks::check_treesitter_ts_js());

        // 8. Tree-sitter Python
        execute_check!("treesitter_python", checks::check_treesitter_python());

        // 9. Config File
        execute_check!("config_file", checks::check_config_file(workspace_root));

        // 10. API Key (OpenRouter)
        execute_check!("api_key_openrouter", checks::check_api_key_openrouter());

        // 11. Docs Directory
        execute_check!(
            "docs_directory",
            checks::check_docs_directory(workspace_root)
        );

        // 12. Lockfile Safety
        execute_check!(
            "lockfile_safety",
            checks::check_lockfile_safety(workspace_root)
        );

        // 13. Disk Space
        execute_check!("disk_space", checks::check_disk_space(workspace_root));

        Ok(DiagnosticReport::from_checks(checks))
    }
}
