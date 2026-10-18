use std::sync::Arc;
use clap::{Args, Subcommand};
use crate::config::Config;
use crate::critic::engine::{Critic, CriticEngine};
use crate::memory::MemoryEngine;

#[derive(Subcommand, Debug)]
pub enum CriticCommands {
    /// Run deterministic verification (e.g. cargo check, cargo test) and report structured diagnostics
    Check(CheckArgs),
    /// Inspect or resolve compiler failure memories in repository memory
    Failures(FailuresArgs),
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Verification command override (default: auto-detected, e.g. cargo check)
    #[arg(long)]
    pub command: Option<String>,
}

#[derive(Args, Debug)]
pub struct FailuresArgs {
    /// Show all failure records (including resolved)
    #[arg(long)]
    pub all: bool,

    /// Mark a failure as resolved by its ID
    #[arg(long)]
    pub resolve: Option<i64>,

    /// Fix summary note when resolving a failure
    #[arg(long)]
    pub fix: Option<String>,
}

pub async fn handle_critic_command(cmd: CriticCommands) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
    let db_path = repo_root.join(&config.memory.database);
    let memory = MemoryEngine::open(&db_path).ok().map(Arc::new);

    match cmd {
        CriticCommands::Check(args) => {
            let mut engine = CriticEngine::new();
            if let Some(c) = args.command {
                engine = engine.with_command(c);
            }
            println!("🔍 Running deterministic verification for: {}", repo_root.display());
            let report = engine.evaluate(&repo_root).await?;

            if report.passed {
                println!("✅ Verification Passed: `{}` ({} ms)", report.command, report.duration_ms);
            } else {
                println!("❌ Verification Failed: `{}` (exit code: {})", report.command, report.exit_code);
                if !report.diagnostics.is_empty() {
                    println!("\nStructured Diagnostics ({} found):", report.diagnostics.len());
                    for d in &report.diagnostics {
                        let loc = match (&d.file_path, d.line, d.col) {
                            (Some(p), Some(l), Some(c)) => format!("{}:{}:{}", p, l, c),
                            (Some(p), Some(l), None) => format!("{}:{}", p, l),
                            (Some(p), None, None) => p.clone(),
                            _ => "unknown".to_string(),
                        };
                        let code_str = d.error_code.as_deref().unwrap_or("error");
                        println!("  • [{}] {}: {}", code_str, loc, d.message);
                    }
                }
                println!("\nCritic Feedback Message:\n{}", report.feedback_message);
            }
        }
        CriticCommands::Failures(args) => {
            let mem = match memory {
                Some(m) => m,
                None => {
                    eprintln!("⚠️  Could not open repository memory database at {}. Run `kaioken memory init` first.", db_path.display());
                    return Ok(());
                }
            };

            if let Some(id) = args.resolve {
                let fix = args.fix.as_deref().unwrap_or("Resolved via CLI");
                mem.resolve_failure(id, fix)?;
                println!("✅ Failure record #{} marked as resolved: \"{}\"", id, fix);
                return Ok(());
            }

            let records = if args.all {
                mem.get_recent_failures(50)?
            } else {
                mem.get_unresolved_failures()?
            };

            if records.is_empty() {
                if args.all {
                    println!("No failure records found in memory.");
                } else {
                    println!("🎉 No unresolved failures found. (Use `--all` to view resolved failures)");
                }
            } else {
                let title = if args.all { "All Failure Records" } else { "Unresolved Compiler Failures" };
                println!("📋 {} ({} records):\n", title, records.len());
                for r in records {
                    let id = r.id.map(|i| format!("#{}", i)).unwrap_or_else(|| "#?".into());
                    let is_resolved = r.status.as_deref() == Some("resolved");
                    let status_badge = if is_resolved { "✅ RESOLVED" } else { "❌ UNRESOLVED" };
                    println!("  {} [{}] [{}]", id, status_badge, r.error_type);
                    if let Some(fid) = r.file_id {
                        println!("     File ID: {}", fid);
                    }
                    if let Some(fix) = &r.fix_summary {
                        println!("     Fix: {}", fix);
                    }
                    let preview = r.message.lines().next().unwrap_or("");
                    println!("     Error: {}\n", preview);
                }
            }
        }
    }

    Ok(())
}
