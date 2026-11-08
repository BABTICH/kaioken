use clap::Parser;
use kaioken::cli::{self, Cli, Commands};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Memory(memory_cmd) => {
            cli::memory::handle_memory_command(memory_cmd).await?;
        }
        Commands::Docs(docs_cmd) => {
            cli::docs::handle_docs_command(docs_cmd).await?;
        }
        Commands::Retrieval(retrieval_cmd) => {
            cli::retrieval::handle_retrieval_command(retrieval_cmd).await?;
        }
        Commands::Task(task_cmd) => {
            cli::task::handle_task_command(task_cmd, cli.dry_run, cli.yes, cli.permission_mode)
                .await?;
        }
        Commands::Critic(critic_cmd) => {
            cli::critic::handle_critic_command(critic_cmd).await?;
        }
        Commands::Schedule(schedule_cmd) => {
            cli::schedule::handle_schedule_command(schedule_cmd).await?;
        }
        Commands::Eval(eval_cmd) => {
            cli::eval::handle_eval_command(eval_cmd).await?;
        }
        Commands::Doctor(doctor_cmd) => {
            cli::doctor::handle_doctor_command(doctor_cmd).await?;
        }
        Commands::Hook(hook_cmd) => {
            cli::hook::handle_hook_command(hook_cmd).await?;
        }
        Commands::Audit(audit_cmd) => {
            cli::audit::handle_audit_command(audit_cmd).await?;
        }
        Commands::Review(review_cmd) => {
            cli::review::handle_review_command(review_cmd).await?;
        }
        Commands::Watch(watch_cmd) => {
            cli::watch::handle_watch_command(watch_cmd).await?;
        }
        Commands::Desktop(desktop_cmd) => {
            cli::desktop::handle_desktop_command(desktop_cmd).await?;
        }
        Commands::Tui(tui_cmd) => {
            cli::tui::run(tui_cmd).await?;
        }
    }

    Ok(())
}
