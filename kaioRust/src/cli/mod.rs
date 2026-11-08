use clap::{Parser, Subcommand};

pub mod audit;
pub mod critic;
pub mod desktop;
pub mod docs;
pub mod doctor;
pub mod eval;
pub mod hook;
pub mod memory;
pub mod retrieval;
pub mod review;
pub mod schedule;
pub mod task;
pub mod tui;
pub mod watch;

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
    /// Run environment diagnostic checks
    Doctor(doctor::DoctorArgs),
    /// Manage Git post-commit hooks for continuous memory sync
    #[command(subcommand)]
    Hook(hook::HookCommands),
    /// Inspect security, approval, and tool execution audit trails
    #[command(subcommand)]
    Audit(audit::AuditCommands),
    /// Analyze git diffs, detect breaking changes, test gaps and change risk
    Review(review::ReviewArgs),
    /// Real-time file watcher daemon with debounced memory synchronization
    #[command(alias = "daemon")]
    Watch(watch::WatchArgs),
    /// Launch the native desktop cockpit
    Desktop(desktop::DesktopArgs),
    /// Launch the interactive terminal UI (ratatui dashboard)
    Tui(tui::TuiArgs),
}
