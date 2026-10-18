use clap::{Parser, Subcommand};

pub mod critic;
pub mod docs;
pub mod eval;
pub mod memory;
pub mod retrieval;
pub mod schedule;
pub mod task;

#[derive(Parser)]
#[command(name = "kaioken")]
#[command(about = "KAIOKEN Agent - Repo-aware AI coding assistant", long_about = None)]
pub struct Cli {
    /// Run without making destructive filesystem modifications
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Automatically approve prompts without interactive confirmation
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

    /// Override the configured permission mode (read-only, edit-with-approval, auto-edit, safe-bash, full-auto)
    #[arg(long, global = true)]
    pub permission_mode: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage repository memory and configuration
    #[command(subcommand)]
    Memory(memory::MemoryCommands),
    /// Manage mirrored documentation and frontmatter
    #[command(subcommand)]
    Docs(docs::DocsCommands),
    /// Retrieve context packets, graph impact, and failure memory
    #[command(subcommand)]
    Retrieval(retrieval::RetrievalCommands),
    /// Execute autonomous coding tasks or generate plans
    #[command(subcommand)]
    Task(task::TaskCommands),
    /// Deterministic compiler verification and failure memory inspection
    #[command(subcommand)]
    Critic(critic::CriticCommands),
    /// Run scheduled repository health audits and reports
    Schedule(schedule::ScheduleArgs),
    /// Run quantitative evaluation benchmarks and emit scorecard
    #[command(subcommand)]
    Eval(eval::EvalCommands),
}
