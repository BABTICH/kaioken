use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use clap::Args;
use chrono::Utc;

use crate::config::Config;
use crate::health::HealthAuditEngine;
use crate::memory::MemoryEngine;

#[derive(Args, Debug)]
pub struct ScheduleArgs {
    /// Execute daily repository health audit scan
    #[arg(long)]
    pub daily: bool,

    /// Output report filepath (default: .agent/reports/health_report_YYYY-MM-DD.md)
    #[arg(long, short)]
    pub output: Option<String>,

    /// Emit report in JSON format instead of Markdown
    #[arg(long)]
    pub json: bool,

    /// Automatically generate documentation for stale files after audit
    #[arg(long)]
    pub fix_stale: bool,
}

pub async fn handle_schedule_command(args: ScheduleArgs) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
    let db_path = repo_root.join(&config.memory.database);
    let docs_root = repo_root.join(&config.docs.output);

    let memory = match MemoryEngine::open(&db_path) {
        Ok(m) => Arc::new(m),
        Err(e) => {
            eprintln!("⚠️ Could not open repository memory database at {}: {}. Run `kaioken memory init` first.", db_path.display(), e);
            return Ok(());
        }
    };

    if args.daily {
        println!("🏥 Executing KAIOKEN Daily Scheduled Health Audit...");
        println!("   Repository: {}", repo_root.display());

        let report = HealthAuditEngine::audit(&repo_root, &memory, &docs_root)?;

        let formatted = if args.json {
            report.to_json()?
        } else {
            report.to_markdown()
        };

        // Determine destination file
        let date_str = Utc::now().format("%Y-%m-%d").to_string();
        let target_path = match args.output {
            Some(p) => PathBuf::from(p),
            None => {
                let reports_dir = repo_root.join(".agent").join("reports");
                fs::create_dir_all(&reports_dir)?;
                let filename = if args.json {
                    format!("health_report_{}.json", date_str)
                } else {
                    format!("health_report_{}.md", date_str)
                };
                reports_dir.join(filename)
            }
        };

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target_path, &formatted)?;

        println!("✅ Health Audit Completed!");
        println!("   Health Score: {}/100 (Grade: {})", report.score.score, report.score.grade);
        println!("   Files Tracked: {} | AST Symbols: {}", report.overview.total_files, report.overview.total_symbols);
        println!("   Docs: {} fresh, {} stale, {} missing", report.doc_health.fresh_count, report.doc_health.stale_count, report.doc_health.missing_docs_count);
        println!("   Dead Code Hints: {} | Untested Files: {}", report.dead_code.unreferenced_symbols.len(), report.test_coverage.untested_files.len());
        println!("   Unresolved Failures: {}", report.failure_memory.unresolved_count);
        println!("\n📄 Saved comprehensive report to: {}\n", target_path.display());

        if !report.recommendations.is_empty() {
            println!("💡 Top Actionable Recommendations:");
            for (i, rec) in report.recommendations.iter().take(3).enumerate() {
                println!("   {}. {}", i + 1, rec);
            }
            println!();
        }

        if args.fix_stale && report.doc_health.stale_count > 0 {
            println!("🔄 Automatically refreshing {} stale documentation files...", report.doc_health.stale_count);
            let generator = crate::docs::DocGenerator::new(&repo_root, &docs_root);
            let options = crate::docs::DocGenerateOptions {
                level: 0,
                force: false,
                stale_only: true,
                no_llm: true,
                limit: Some(50),
            };
            let _ = generator.generate_all_docs(&memory, &options);
            println!("✅ Stale documentation refreshed.");
        }
    } else {
        println!("KAIOKEN Schedule Command");
        println!("Run with `--daily` to trigger the scheduled repository health audit scan.");
    }

    Ok(())
}
