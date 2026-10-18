use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::critic::diagnostics::{CompilerDiagnostic, DiagnosticParser};
use crate::error::Result;
use crate::safety::command_guard::CommandGuard;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct CriticReport {
    pub passed: bool,
    pub command: String,
    pub exit_code: i32,
    pub diagnostics: Vec<CompilerDiagnostic>,
    pub feedback_message: String,
    pub raw_output: String,
    pub duration_ms: u128,
}

impl CriticReport {
    pub fn success(command: impl Into<String>, duration_ms: u128) -> Self {
        let cmd = command.into();
        Self {
            passed: true,
            command: cmd.clone(),
            exit_code: 0,
            diagnostics: Vec::new(),
            feedback_message: format!("Deterministic check `{}` passed successfully.", cmd),
            raw_output: String::new(),
            duration_ms,
        }
    }

    pub fn failure(
        command: impl Into<String>,
        exit_code: i32,
        diagnostics: Vec<CompilerDiagnostic>,
        raw_output: String,
        duration_ms: u128,
    ) -> Self {
        let cmd = command.into();
        let feedback_message = DiagnosticParser::format_feedback_block(&cmd, exit_code, &diagnostics, &raw_output);
        Self {
            passed: false,
            command: cmd,
            exit_code,
            diagnostics,
            feedback_message,
            raw_output,
            duration_ms,
        }
    }
}

pub trait Critic: Send + Sync {
    fn evaluate<'a>(&'a self, repo_root: &'a Path) -> BoxFuture<'a, Result<CriticReport>>;
}

pub struct CriticEngine {
    command_override: Option<String>,
    command_guard: Arc<CommandGuard>,
}

impl CriticEngine {
    pub fn new() -> Self {
        Self {
            command_override: None,
            command_guard: Arc::new(CommandGuard::default()),
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command_override = Some(command.into());
        self
    }

    pub fn with_guard(mut self, guard: Arc<CommandGuard>) -> Self {
        self.command_guard = guard;
        self
    }

    /// Automatically detects standard verification command for the repository.
    pub fn detect_command(repo_root: &Path) -> Option<String> {
        if repo_root.join("Cargo.toml").exists() {
            Some("cargo check".to_string())
        } else if repo_root.join("package.json").exists() {
            Some("npm test".to_string())
        } else if repo_root.join("pyproject.toml").exists() || repo_root.join("requirements.txt").exists() {
            Some("pytest".to_string())
        } else if repo_root.join("go.mod").exists() {
            Some("go test ./...".to_string())
        } else {
            None
        }
    }
}

impl Default for CriticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Critic for CriticEngine {
    fn evaluate<'a>(&'a self, repo_root: &'a Path) -> BoxFuture<'a, Result<CriticReport>> {
        let command = match self.command_override.clone() {
            Some(c) => c,
            None => match Self::detect_command(repo_root) {
                Some(cmd) => cmd,
                None => {
                    return Box::pin(async {
                        Ok(CriticReport::success("none (no build manifest detected)", 0))
                    });
                }
            },
        };
        let root = repo_root.to_path_buf();
        let guard = Arc::clone(&self.command_guard);
        Box::pin(async move {
            let start = Instant::now();

            let exec_res = match guard.execute(&command, &root, Some(120), None).await {
                Ok(res) => res,
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis();
                    return Ok(CriticReport::failure(
                        &command,
                        -1,
                        Vec::new(),
                        format!("Failed to execute verification command: {}", e),
                        duration_ms,
                    ));
                }
            };

            let duration_ms = start.elapsed().as_millis();
            let mut combined_output = String::new();
            if !exec_res.stdout.is_empty() {
                combined_output.push_str(&exec_res.stdout);
            }
            if !exec_res.stderr.is_empty() {
                if !combined_output.is_empty() {
                    combined_output.push('\n');
                }
                combined_output.push_str(&exec_res.stderr);
            }

            if exec_res.exit_code == 0 {
                Ok(CriticReport::success(&command, duration_ms))
            } else {
                let diagnostics = DiagnosticParser::parse(&combined_output);
                Ok(CriticReport::failure(
                    &command,
                    exec_res.exit_code,
                    diagnostics,
                    combined_output,
                    duration_ms,
                ))
            }
        })
    }
}

/// MockCritic enables deterministic simulation of compiler failures and subsequent resolutions in tests.
pub struct MockCritic {
    reports: Arc<Mutex<Vec<CriticReport>>>,
}

impl MockCritic {
    pub fn new() -> Self {
        Self {
            reports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_reports(reports: Vec<CriticReport>) -> Self {
        Self {
            reports: Arc::new(Mutex::new(reports)),
        }
    }

    pub fn push_report(&self, report: CriticReport) {
        self.reports.lock().unwrap().push(report);
    }
}

impl Default for MockCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl Critic for MockCritic {
    fn evaluate<'a>(&'a self, _repo_root: &'a Path) -> BoxFuture<'a, Result<CriticReport>> {
        Box::pin(async move {
            let mut list = self.reports.lock().unwrap();
            if !list.is_empty() {
                Ok(list.remove(0))
            } else {
                // Default passing report
                Ok(CriticReport::success("mock_check", 5))
            }
        })
    }
}
