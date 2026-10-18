use clap::{Parser, Subcommand};

pub mod docs;
pub mod memory;

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
    // Task(task::TaskCommands),  // Phase 10
}
