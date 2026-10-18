use clap::{Parser, Subcommand};

pub mod critic;
pub mod docs;
pub mod memory;
pub mod retrieval;
pub mod task;

#[derive(Parser)]
#[command(name = "kaioken")]
#[command(about = "KAIOKEN Agent - Repo-aware AI coding assistant", long_about = None)]
pub struct Cli {
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
}
