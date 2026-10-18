use clap::{Args, Subcommand};
use crate::config::Config;
use crate::memory::MemoryEngine;
use crate::retrieval::cache::RetrievalCacheStore;
use crate::retrieval::graph_query::GraphQueryEngine;
use crate::retrieval::query::{RetrievalEngine, RetrievalOptions};

#[derive(Subcommand, Debug)]
pub enum RetrievalCommands {
    /// Retrieve budgeted, sanitized context packet for a task
    Context(ContextArgs),
    /// Inspect blast radius, callers, and affected tests for a symbol or file
    Impact(ImpactArgs),
    /// Search and view past compiler, test, or runtime failures
    Failures(FailuresArgs),
    /// Manage SQLite retrieval cache
    Cache(CacheArgs),
}

#[derive(Args, Debug)]
pub struct ContextArgs {
    /// The task prompt or goal for context retrieval
    pub prompt: String,

    /// Output full context packet as JSON
    #[arg(long)]
    pub json: bool,

    /// Maximum token budget ceiling (default: 8000)
    #[arg(long)]
    pub budget: Option<usize>,

    /// Bypass retrieval cache
    #[arg(long)]
    pub no_cache: bool,

    /// Graph neighbor traversal depth (1 or 2, default: 1)
    #[arg(long, default_value = "1")]
    pub depth: usize,
}

#[derive(Args, Debug)]
pub struct ImpactArgs {
    /// Target symbol name or file path to analyze
    pub target: String,

    /// Traversal depth (default: 1)
    #[arg(long, default_value = "1")]
    pub depth: usize,
}

#[derive(Args, Debug)]
pub struct FailuresArgs {
    /// Optional keyword or error search query
    pub query: Option<String>,

    /// Max results to display (default: 10)
    #[arg(long, default_value = "10")]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct CacheArgs {
    /// Clear all cached retrieval packets
    #[arg(long)]
    pub clear: bool,
}

pub async fn handle_retrieval_command(cmd: RetrievalCommands) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir()?;
    let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());
    let db_path = repo_root.join(&config.memory.database);
    let memory = MemoryEngine::open(&db_path)?;

    match cmd {
        RetrievalCommands::Context(args) => {
            let start = std::time::Instant::now();
            let engine = RetrievalEngine::new(&memory, &config, &repo_root);
            let opts = RetrievalOptions {
                budget_tokens: args.budget,
                no_cache: args.no_cache,
                max_depth: args.depth,
                include_commits: true,
            };

            let packet = engine.retrieve_context(&args.prompt, &opts)?;
            let elapsed_ms = start.elapsed().as_millis();

            if args.json {
                println!("{}", packet.to_json()?);
            } else {
                println!("{}", packet.to_markdown());
                println!("⚡ Retrieval latency: {} ms", elapsed_ms);
            }
        }
        RetrievalCommands::Impact(args) => {
            let engine = GraphQueryEngine::new(&memory);
            
            // Check if symbol or file
            let mut symbol_ids = Vec::new();
            let mut file_ids = Vec::new();

            if let Ok(syms) = memory.find_symbols_by_name(&args.target) {
                for s in syms {
                    if let Some(id) = s.id {
                        symbol_ids.push(id);
                    }
                }
            }

            if let Ok(Some(f)) = memory.get_file_by_path(&args.target) {
                if let Some(fid) = f.id {
                    file_ids.push(fid);
                }
            }

            if symbol_ids.is_empty() && file_ids.is_empty() {
                println!("No matching symbol or file found in memory for: {}", args.target);
                return Ok(());
            }

            let subset = engine.query_subgraph(&symbol_ids, &file_ids, args.depth)?;
            println!("Impact Analysis for: '{}'", args.target);
            println!("- Direct Callers: {}", if subset.direct_callers.is_empty() { "none".to_string() } else { subset.direct_callers.join(", ") });
            println!("- Direct Callees: {}", if subset.direct_callees.is_empty() { "none".to_string() } else { subset.direct_callees.join(", ") });
            println!("- Affected Tests: {}", if subset.affected_tests.is_empty() { "none".to_string() } else { subset.affected_tests.join(", ") });
        }
        RetrievalCommands::Failures(args) => {
            let failures = if let Some(q) = args.query {
                let lookup = crate::retrieval::FailureLookup::new(&memory);
                lookup.find_failures(&[], &[], &q, args.limit)?
            } else {
                let recs = memory.get_recent_failures(args.limit)?;
                recs.into_iter().map(|f| crate::retrieval::FailureMemorySnippet {
                    error_type: f.error_type,
                    message: f.message,
                    file_path: None,
                    symbol_name: None,
                    fix_summary: f.fix_summary,
                    created_at: f.created_at,
                }).collect()
            };

            if failures.is_empty() {
                println!("No failure memory records found.");
            } else {
                println!("Past Failures (Total: {}):", failures.len());
                for f in failures {
                    println!("- [{}] {}: {}", f.created_at, f.error_type, f.message);
                    if let Some(fix) = f.fix_summary {
                        println!("  Fix: {}", fix);
                    }
                }
            }
        }
        RetrievalCommands::Cache(args) => {
            let cache_store = RetrievalCacheStore::new(&memory);
            if args.clear {
                let cleared = cache_store.clear()?;
                println!("Cleared {} cached retrieval packets.", cleared);
            } else {
                let conn = memory.db().connection();
                let count: i64 = conn.query_row("SELECT COUNT(*) FROM retrieval_cache", [], |r| r.get(0)).unwrap_or(0);
                println!("Retrieval Cache: {} entries stored.", count);
            }
        }
    }

    Ok(())
}
