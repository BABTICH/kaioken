use clap::Subcommand;

use crate::config::Config;
use crate::docs::{DocGenerateOptions, DocGenerator, StalenessCascadeEngine, StalenessDetector};
use crate::error::{KaiokenError, Result};
use crate::memory::MemoryEngine;
use crate::scanner::GitScanner;

#[derive(Subcommand)]
pub enum DocsCommands {
    /// Generate mirrored markdown documentation in .agent-docs/
    Generate {
        /// Specific file path to generate documentation for (defaults to all files)
        #[arg(long)]
        path: Option<String>,

        /// Documentation level: 0=Raw Facts, 1=Summary, 2=Detailed, 3=Deep (default 0)
        #[arg(long, default_value = "0")]
        level: u8,

        /// Force overwrite existing documentation even if up to date
        #[arg(long)]
        force: bool,

        /// Generate only for files marked stale or not yet documented
        #[arg(long)]
        stale: bool,

        /// Run deterministic generation only (zero LLM API cost, default true)
        #[arg(long, default_value = "true")]
        no_llm: bool,

        /// Maximum number of documents to generate in this run (budget control)
        #[arg(long)]
        limit: Option<usize>,

        /// Output results in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Show documentation coverage status and statistics
    Status {
        /// Output status in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Verify mirrored documentation consistency, hashes, and orphaned files
    Verify {
        /// Delete orphaned documentation files whose source files no longer exist
        #[arg(long)]
        clean: bool,

        /// Output verification results in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Trigger staleness cascade across dependency graph for specified files
    Cascade {
        /// File paths that changed (comma-separated or multiple arguments)
        #[arg(long, value_delimiter = ',')]
        paths: Vec<String>,

        /// Output results in structured JSON format
        #[arg(long)]
        json: bool,
    },
}

pub async fn handle_docs_command(cmd: DocsCommands) -> Result<()> {
    match cmd {
        DocsCommands::Generate {
            path,
            level,
            force,
            stale,
            no_llm,
            limit,
            json,
        } => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
            let docs_root = repo_root.join(&config.docs.output);
            let db_path = repo_root.join(&config.memory.database);

            let memory = MemoryEngine::open(&db_path)?;
            let generator = DocGenerator::new(&repo_root, &docs_root);
            let options = DocGenerateOptions {
                level,
                force,
                stale_only: stale,
                no_llm,
                limit,
            };

            if let Some(target_path) = path {
                let normalized = target_path.replace('\\', "/");
                let file_rec = memory
                    .get_file_by_path(&normalized)?
                    .ok_or_else(|| KaiokenError::DocError(format!("File '{}' not found in memory database. Run `kaioken memory scan` first.", normalized)))?;

                let res = generator.generate_file_doc(&file_rec, &memory, &options)?;

                if json {
                    let json_str = serde_json::to_string_pretty(&res)
                        .map_err(|e| KaiokenError::DocError(e.to_string()))?;
                    println!("{}", json_str);
                    return Ok(());
                }

                println!("✅ Generated mirrored documentation:");
                println!("   Source:    {}", res.source_path);
                println!("   Mirrored:  {}", res.relative_doc_path);
                println!("   Level:     {} (Facts)", res.doc_level);
                println!("   Symbols:   {} indexed", res.symbols_count);
                println!("   Imports:   {} indexed", res.imports_count);
                println!("   Status:    {}", res.status);
                return Ok(());
            }

            // Batch generation
            let summary = generator.generate_all_docs(&memory, &options)?;

            if json {
                let json_obj = serde_json::json!({
                    "repo_root": repo_root.display().to_string(),
                    "docs_root": docs_root.display().to_string(),
                    "summary": summary,
                    "level": level,
                    "mode": if no_llm { "deterministic" } else { "llm" }
                });
                let json_str = serde_json::to_string_pretty(&json_obj)
                    .map_err(|e| KaiokenError::DocError(e.to_string()))?;
                println!("{}", json_str);
                return Ok(());
            }

            println!("┌────────────────────────────────────────────────────────┐");
            println!("│           KAIOKEN AGENT: DOCUMENTATION GENERATOR       │");
            println!("├────────────────────────────────────────────────────────┤");
            println!("│ Repository Root: {:<38} │", repo_root.file_name().unwrap_or_default().to_string_lossy());
            println!("│ Output Path:     {:<38} │", config.docs.output);
            println!("│ Target Level:    {:<38} │", format!("Level {} (Raw Facts)", level));
            println!("│ Mode:            {:<38} │", if no_llm { "Deterministic (--no-llm)" } else { "Standard" });
            println!("├────────────────────────────────────────────────────────┤");
            println!("│ Total Processed: {:<38} │", summary.total_processed);
            println!("│ Newly Generated: {:<38} │", summary.generated);
            println!("│ Updated Docs:    {:<38} │", summary.updated);
            println!("│ Skipped (Fresh): {:<38} │", summary.skipped);
            if summary.errors > 0 {
                println!("│ Errors:          {:<38} │", summary.errors);
            }
            println!("└────────────────────────────────────────────────────────┘");

            println!(
                "\n✓ Mirrored documentation generated in {} with cryptographic frontmatter.",
                docs_root.display()
            );
        }
        DocsCommands::Status { json } => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
            let db_path = repo_root.join(&config.memory.database);

            let memory = MemoryEngine::open(&db_path)?;
            let total_files = memory.count_files().unwrap_or(0);
            let total_docs = memory.count_docs().unwrap_or(0);
            let by_status = memory.count_docs_by_status().unwrap_or_default();

            let fresh_count = *by_status.get("fresh").unwrap_or(&0);
            let stale_count = *by_status.get("stale").unwrap_or(&0);
            let maybe_stale_count = *by_status.get("maybe_stale").unwrap_or(&0);

            if json {
                let json_obj = serde_json::json!({
                    "total_files": total_files,
                    "total_docs": total_docs,
                    "fresh": fresh_count,
                    "stale": stale_count,
                    "maybe_stale": maybe_stale_count,
                    "coverage_percent": if total_files > 0 { (total_docs as f64 / total_files as f64) * 100.0 } else { 0.0 }
                });
                let json_str = serde_json::to_string_pretty(&json_obj)
                    .map_err(|e| KaiokenError::DocError(e.to_string()))?;
                println!("{}", json_str);
                return Ok(());
            }

            let coverage = if total_files > 0 {
                (total_docs as f64 / total_files as f64) * 100.0
            } else {
                0.0
            };

            println!("┌────────────────────────────────────────────────────────┐");
            println!("│           KAIOKEN AGENT: DOCUMENTATION STATUS          │");
            println!("├────────────────────────────────────────────────────────┤");
            println!("│ Output Path:     {:<38} │", config.docs.output);
            println!("│ Indexed Files:   {:<38} │", total_files);
            println!("│ Documented Docs: {:<38} │", total_docs);
            println!("│ Coverage Rate:   {:<38} │", format!("{:.1}%", coverage));
            println!("├────────────────────────────────────────────────────────┤");
            println!("│ Fresh Docs:      {:<38} │", fresh_count);
            println!("│ Stale Docs:      {:<38} │", stale_count);
            println!("│ Maybe Stale:     {:<38} │", maybe_stale_count);
            println!("└────────────────────────────────────────────────────────┘");
        }
        DocsCommands::Verify { clean, json } => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
            let docs_root = repo_root.join(&config.docs.output);
            let db_path = repo_root.join(&config.memory.database);

            let memory = MemoryEngine::open(&db_path)?;
            let report = StalenessDetector::audit_and_clean(&repo_root, &docs_root, &memory, clean)?;

            if json {
                let json_str = serde_json::to_string_pretty(&report)
                    .map_err(|e| KaiokenError::DocError(e.to_string()))?;
                println!("{}", json_str);
                return Ok(());
            }

            println!("┌────────────────────────────────────────────────────────┐");
            println!("│           KAIOKEN AGENT: DOCUMENTATION VERIFY          │");
            println!("├────────────────────────────────────────────────────────┤");
            println!("│ Docs Audited:    {:<38} │", report.total_docs);
            println!("│ Valid & Fresh:   {:<38} │", report.valid_docs);
            println!("│ Stale (Hash):    {:<38} │", report.stale_docs);
            println!("│ Orphaned Docs:   {:<38} │", report.orphaned_docs);
            if clean {
                println!("│ Cleaned Docs:    {:<38} │", report.cleaned_docs);
            }
            println!("└────────────────────────────────────────────────────────┘");

            if report.stale_docs > 0 || report.orphaned_docs > 0 {
                println!("\nDetected Issues:");
                for item in &report.items {
                    if !item.is_valid {
                        println!("  - {} -> {:?} ({})", item.file_path, item.status, item.doc_path);
                    }
                }
                if clean {
                    println!("\n🧹 Cleaned {} orphaned documentation files.", report.cleaned_docs);
                } else {
                    println!("\nRun `kaioken docs generate --stale` to refresh stale documentation.");
                    if report.orphaned_docs > 0 {
                        println!("Run `kaioken docs verify --clean` to remove orphaned documentation.");
                    }
                }
            } else {
                println!("\n✅ All {} documentation files verified successfully with matching cryptographic hashes.", report.total_docs);
            }
        }
        DocsCommands::Cascade { paths, json } => {
            let current_dir = std::env::current_dir()?;
            let repo_root = GitScanner::find_repo_root(&current_dir).unwrap_or(current_dir);
            let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
            let db_path = repo_root.join(&config.memory.database);

            let memory = MemoryEngine::open(&db_path)?;
            let report = StalenessCascadeEngine::cascade_changes(&repo_root, &paths, &memory)?;

            if json {
                let json_str = serde_json::to_string_pretty(&report)
                    .map_err(|e| KaiokenError::DocError(e.to_string()))?;
                println!("{}", json_str);
                return Ok(());
            }

            println!("┌────────────────────────────────────────────────────────┐");
            println!("│           KAIOKEN AGENT: STALENESS CASCADE             │");
            println!("├────────────────────────────────────────────────────────┤");
            println!("│ Changed Files:   {:<38} │", report.changed_files.len());
            println!("│ Directly Stale:  {:<38} │", report.directly_stale.len());
            println!("│ 1-Hop Dependents:{:<38} │", report.maybe_stale_dependents.len());
            println!("│ Cascade Depth:   {:<38} │", report.cascade_depth);
            println!("└────────────────────────────────────────────────────────┘");

            if !report.directly_stale.is_empty() {
                println!("\nDirectly Stale (marked 'stale'):");
                for f in &report.directly_stale {
                    println!("  - {}", f);
                }
            }
            if !report.maybe_stale_dependents.is_empty() {
                println!("\nDirect Dependents (marked 'maybe_stale'):");
                for f in &report.maybe_stale_dependents {
                    println!("  - {}", f);
                }
            }
            println!("\nRun `kaioken docs generate --stale` to refresh stale and maybe_stale documentation.");
        }
    }
    Ok(())
}
