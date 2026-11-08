use crate::safety::audit::{AuditFilter, AuditLogger, AuditQueryResult};
use crate::scanner::git::GitScanner;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
pub enum AuditCommands {
    /// Display recent audit log events with filtering
    Show(AuditShowArgs),
    /// Display audit summary statistics
    Stats(AuditStatsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct AuditShowArgs {
    /// Number of recent events to show
    #[arg(long, default_value = "50")]
    pub last: usize,

    /// Filter events after this ISO 8601 timestamp
    #[arg(long)]
    pub since: Option<String>,

    /// Filter by event type or action (e.g. tool_call, safety_block, approval, bash, edit_file)
    #[arg(long)]
    pub r#type: Option<String>,

    /// Filter by severity level (info, warn, critical)
    #[arg(long)]
    pub severity: Option<String>,

    /// Filter by agent run ID
    #[arg(long)]
    pub run: Option<String>,

    /// Filter by tool name
    #[arg(long)]
    pub tool: Option<String>,

    /// Output as JSON array
    #[arg(long)]
    pub json: bool,

    /// Data source: sqlite or disk
    #[arg(long, default_value = "sqlite")]
    pub source: String,

    /// Override the workspace directory
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct AuditStatsArgs {
    /// Output as formatted JSON
    #[arg(long)]
    pub json: bool,

    /// Data source: sqlite or disk
    #[arg(long, default_value = "sqlite")]
    pub source: String,

    /// Override the workspace directory
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

pub async fn handle_audit_command(command: AuditCommands) -> anyhow::Result<()> {
    match command {
        AuditCommands::Show(args) => {
            let current_dir = std::env::current_dir()?;
            let workspace_root = args
                .dir
                .unwrap_or_else(|| GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir));

            let filter = AuditFilter {
                limit: args.last,
                since: args.since,
                event_type: args.r#type,
                severity: args.severity,
                run_id: args.run,
                tool_name: args.tool,
            };

            let result = if args.source.to_lowercase() == "disk" {
                let log_path = workspace_root.join(".agent").join("logs").join("audit.log");
                AuditLogger::parse_disk_log(&log_path, &filter)?
            } else {
                let logger = AuditLogger::new(Some(&workspace_root));
                match logger.query(&filter) {
                    Ok(r) if r.total > 0 => r,
                    _ => {
                        // Graceful degradation: fallback to disk log if SQLite is empty or inaccessible
                        let log_path = workspace_root.join(".agent").join("logs").join("audit.log");
                        if log_path.exists() {
                            AuditLogger::parse_disk_log(&log_path, &filter).unwrap_or(
                                AuditQueryResult {
                                    events: Vec::new(),
                                    total: 0,
                                    filtered: 0,
                                },
                            )
                        } else {
                            AuditQueryResult {
                                events: Vec::new(),
                                total: 0,
                                filtered: 0,
                            }
                        }
                    }
                }
            };

            if args.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{}", result.render_terminal());
            }
        }
        AuditCommands::Stats(args) => {
            let current_dir = std::env::current_dir()?;
            let workspace_root = args
                .dir
                .unwrap_or_else(|| GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir));

            let stats = if args.source.to_lowercase() == "disk" {
                let log_path = workspace_root.join(".agent").join("logs").join("audit.log");
                let disk_res = AuditLogger::parse_disk_log(
                    &log_path,
                    &AuditFilter {
                        limit: 100_000,
                        ..Default::default()
                    },
                )?;
                AuditLogger::stats_from_entries(&disk_res.events)
            } else {
                let logger = AuditLogger::new(Some(&workspace_root));
                match logger.compute_stats() {
                    Ok(s) if s.total_events > 0 => s,
                    _ => {
                        let log_path = workspace_root.join(".agent").join("logs").join("audit.log");
                        if log_path.exists() {
                            let disk_res = AuditLogger::parse_disk_log(
                                &log_path,
                                &AuditFilter {
                                    limit: 100_000,
                                    ..Default::default()
                                },
                            )
                            .unwrap_or(AuditQueryResult {
                                events: Vec::new(),
                                total: 0,
                                filtered: 0,
                            });
                            AuditLogger::stats_from_entries(&disk_res.events)
                        } else {
                            Default::default()
                        }
                    }
                }
            };

            if args.json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                println!("{}", stats.render_terminal());
            }
        }
    }

    Ok(())
}
