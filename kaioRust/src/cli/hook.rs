use crate::hook::{HookManager, InstallOutcome, UninstallOutcome};
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
pub enum HookCommands {
    /// Install the non-blocking post-commit git hook
    Install(HookInstallArgs),
    /// Remove the Kaioken hook block (preserves existing hooks)
    Uninstall(HookDirArgs),
    /// Inspect git hook installation status and health
    Status(HookStatusArgs),
}

#[derive(Args, Debug, Clone)]
pub struct HookInstallArgs {
    /// Overwrite existing Kaioken hook block if present
    #[arg(long)]
    pub force: bool,

    /// Shell interpreter path for shebang (default: /bin/sh)
    #[arg(long, default_value = "/bin/sh")]
    pub shell: String,

    /// Override the workspace directory
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct HookDirArgs {
    /// Override the workspace directory
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct HookStatusArgs {
    /// Output status as formatted JSON
    #[arg(long)]
    pub json: bool,

    /// Override the workspace directory
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

pub async fn handle_hook_command(command: HookCommands) -> anyhow::Result<()> {
    match command {
        HookCommands::Install(args) => {
            let workspace_root = match args.dir {
                Some(d) => d,
                None => std::env::current_dir()?,
            };

            let outcome = HookManager::install(&workspace_root, args.force, &args.shell)?;
            match outcome {
                InstallOutcome::InstalledFresh(path) => {
                    println!("✅ Git post-commit hook installed successfully!");
                    println!("   Location:    {}", path.display());
                    println!("   Interpreter: {}", args.shell);
                    println!("   Mode:        Non-blocking background memory scan (& disown)");
                }
                InstallOutcome::Appended(path) => {
                    println!("✅ Git post-commit hook appended successfully!");
                    println!("   Location:    {}", path.display());
                    println!(
                        "   Note:        Existing hooks preserved; Kaioken block appended at end"
                    );
                }
                InstallOutcome::Replaced(path) => {
                    println!("✅ Git post-commit hook updated successfully (--force)!");
                    println!("   Location:    {}", path.display());
                    println!(
                        "   Note:        Kaioken sentinel block refreshed; other hooks preserved"
                    );
                }
                InstallOutcome::AlreadyInstalled(path) => {
                    println!(
                        "ℹ️  Git post-commit hook is already installed at {}",
                        path.display()
                    );
                    println!("   To update or reinstall, run: `kaioken hook install --force`");
                }
            }
        }
        HookCommands::Uninstall(args) => {
            let workspace_root = match args.dir {
                Some(d) => d,
                None => std::env::current_dir()?,
            };

            let outcome = HookManager::uninstall(&workspace_root)?;
            match outcome {
                UninstallOutcome::RemovedAndDeletedFile(path) => {
                    println!("✅ Kaioken post-commit hook removed successfully!");
                    println!("   Location: {}", path.display());
                    println!("   Note:     File had no other hooks and was safely deleted");
                }
                UninstallOutcome::RemovedBlock(path) => {
                    println!("✅ Kaioken hook block removed successfully!");
                    println!("   Location: {}", path.display());
                    println!("   Note:     Other custom hooks in post-commit have been preserved");
                }
                UninstallOutcome::NotInstalled(path) => {
                    println!("ℹ️  No Kaioken hook was installed in {}", path.display());
                }
                UninstallOutcome::FileNotFound(path) => {
                    println!("ℹ️  No post-commit hook file exists at {}", path.display());
                }
            }
        }
        HookCommands::Status(args) => {
            let workspace_root = match args.dir {
                Some(d) => d,
                None => std::env::current_dir()?,
            };

            let status = HookManager::status(&workspace_root)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!("{}", status.render_terminal());
            }
        }
    }

    Ok(())
}
