use crate::desktop::app::KaiokenDesktopController;
use crate::desktop::embedded::EmbeddedRuntime;
use crate::desktop::ipc::{default_ipc_endpoint, DaemonIpcClient};
use crate::desktop::model::ActiveWorkspaceTab;
use crate::desktop::theme::DesktopTheme;
use anyhow::Result;
use clap::Args;
use std::sync::Arc;
use std::time::Duration;

const DESKTOP_CONNECT_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Args, Debug, Clone)]
pub struct DesktopArgs {
    /// Run standalone with embedded in-process memory & watcher
    #[arg(long)]
    pub embedded: bool,

    /// IPC endpoint path (named pipe or socket)
    #[arg(long)]
    pub endpoint: Option<String>,

    /// Initial workspace route (chat, research, wiki, graph, cards, editor,
    /// browser, activity, extensions, cost, settings)
    #[arg(long, default_value = "chat")]
    pub tab: String,

    /// Theme selection (dark or light)
    #[arg(long, default_value = "dark")]
    pub theme: String,

    /// Run in headless mode and print a terminal snapshot frame
    #[arg(long)]
    pub headless: bool,

    /// Optional initial task prompt to assign to the agent
    #[arg(long)]
    pub task: Option<String>,
}

pub async fn handle_desktop_command(args: DesktopArgs) -> Result<()> {
    let repo_root = std::env::current_dir()?;
    let repo_name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("kaioken")
        .to_string();

    println!("🚀 Launching Kaioken Desktop...");

    // Determine route
    let initial_tab = ActiveWorkspaceTab::from_id(&args.tab).unwrap_or(ActiveWorkspaceTab::Chat);

    // Initialize IPC Client. Daemon mode talks to `kaioken watch`; when no
    // daemon is running we say so and fall back to the embedded runtime rather
    // than silently rendering against a mock.
    let mut mode = "embedded".to_string();
    let mut endpoint = "embedded in-process".to_string();
    let client = if args.embedded {
        println!("📦 Running in embedded mode (in-process memory & daemon)...");
        let runtime = EmbeddedRuntime::start(repo_root.clone(), false).await?;
        runtime.client()
    } else {
        let daemon_endpoint = args.endpoint.clone().unwrap_or_else(default_ipc_endpoint);
        println!("🔌 Connecting to daemon at {}...", daemon_endpoint);
        match DaemonIpcClient::connect_with_timeout(&daemon_endpoint, DESKTOP_CONNECT_TIMEOUT).await
        {
            Ok(client) => {
                println!("✅ Connected to daemon.");
                mode = "daemon".to_string();
                endpoint = daemon_endpoint;
                client
            }
            Err(error) => {
                eprintln!(
                    "⚠️  No daemon at {}: {}\n    Start it with `kaioken watch --daemon`, or pass --embedded.",
                    daemon_endpoint, error
                );
                let runtime = EmbeddedRuntime::start(repo_root.clone(), false).await?;
                runtime.client()
            }
        }
    };

    let mut controller = KaiokenDesktopController::new(repo_name, "main", client);
    controller.window.repo_path = repo_root.display().to_string();
    controller.window.set_tab(initial_tab);

    if args.theme.eq_ignore_ascii_case("light") {
        controller.window.theme = DesktopTheme::light();
        controller.window.settings.theme = "light".to_string();
    }

    controller.window.settings.daemon_mode = mode;
    controller.window.settings.endpoint = endpoint;

    if let Some(ref t) = args.task {
        controller.send_chat(t);
    }

    let _ = controller.initial_sync().await;

    if args.headless {
        let frame = controller.window.render_terminal_frame();
        println!("{}", frame);
        return Ok(());
    }

    println!("✅ Desktop window initialized.");
    let controller_arc = Arc::new(std::sync::Mutex::new(controller));
    crate::desktop::gui::launch_desktop_gui(controller_arc)?;
    println!("\n👋 Kaioken Desktop exited.");

    Ok(())
}
