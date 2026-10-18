use std::fs;
use std::path::PathBuf;
use clap::{Args, Subcommand};
use crate::eval::{BenchmarkResult, EvaluationRunner};

#[derive(Subcommand, Debug)]
pub enum EvalCommands {
    /// Execute benchmark evaluation suite and produce a quantitative scorecard
    Run(EvalRunArgs),
}

#[derive(Args, Debug)]
pub struct EvalRunArgs {
    /// Filter benchmark tasks by name substring
    #[arg(long, short)]
    pub filter: Option<String>,

    /// Save report to specific output file (defaults to .agent/reports/eval_report_<DATE>.md)
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Emit report in JSON format
    #[arg(long)]
    pub json: bool,
}

pub async fn handle_eval_command(cmd: EvalCommands) -> anyhow::Result<()> {
    match cmd {
        EvalCommands::Run(args) => {
            let repo_root = std::env::current_dir()?;
            println!("🚀 Running KAIOKEN Quantitative Evaluation & Benchmark Suite...");
            println!("   Repository: {}", repo_root.display());
            if let Some(f) = &args.filter {
                println!("   Filter: '{}'", f);
            }
            println!();

            let runner = EvaluationRunner::new();
            let progress = |res: &BenchmarkResult| {
                let badge = if res.passed { "✅ PASS" } else { "❌ FAIL" };
                println!(
                    "   [{}] {:<28} ({} ms, {} tool calls, {} critic iter)",
                    badge, res.name, res.duration_ms, res.tool_calls, res.critic_iterations
                );
            };

            let scorecard = runner.run_all(args.filter.as_deref(), Some(&progress)).await?;

            println!("\n{}", scorecard.to_markdown());

            // Determine output path
            let reports_dir = repo_root.join(".agent").join("reports");
            fs::create_dir_all(&reports_dir)?;

            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let report_path = if let Some(out) = args.output {
                out
            } else if args.json {
                reports_dir.join(format!("eval_report_{}.json", today))
            } else {
                reports_dir.join(format!("eval_report_{}.md", today))
            };

            let content = if args.json {
                scorecard.to_json()?
            } else {
                scorecard.to_markdown()
            };

            fs::write(&report_path, content)?;
            println!("📄 Saved evaluation scorecard to: {}", report_path.display());
        }
    }

    Ok(())
}
