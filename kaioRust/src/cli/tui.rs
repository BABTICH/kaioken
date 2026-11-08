use crate::desktop::ipc::default_ipc_endpoint;
use crate::tui::app::TuiApp;
use crate::tui::client::TuiClient;
use crate::tui::{render_tui_headless, run_tui};
use anyhow::Result;
use clap::Args;
use std::env;

#[derive(Args, Debug, Clone)]
pub struct TuiArgs {
    /// Initial active tab: dashboard, memory, task, review, audit
    #[arg(long, default_value = "dashboard")]
    pub tab: String,

    /// Daemon IPC socket or TCP address (e.g. "127.0.0.1:47890")
    #[arg(long)]
    pub daemon_url: Option<String>,

    /// UI refresh interval in milliseconds
    #[arg(long, default_value = "250")]
    pub poll: u64,

    /// Run in read-only inspection mode
    #[arg(long)]
    pub read_only: bool,

    /// Render a single snapshot frame and exit (headless mode)
    #[arg(long)]
    pub headless: bool,
}

pub async fn run(args: TuiArgs) -> Result<()> {
    let repo_root = env::current_dir()?;
    let repo_name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace")
        .to_string();
    let branch = "main".to_string();

    let mut app = TuiApp::new(repo_name, branch, repo_root).with_initial_tab(&args.tab);
    app.read_only = args.read_only;

    // Try the local daemon unless the caller named another endpoint. When no
    // daemon is running, `TuiClient::connect` yields the offline local view.
    let daemon_url = args
        .daemon_url
        .clone()
        .or_else(|| Some(default_ipc_endpoint()));

    if args.headless {
        let client = TuiClient::connect(daemon_url.as_deref()).await;
        let _ = client.refresh(&mut app).await;
        let frame = render_tui_headless(&app, 100, 30)?;
        println!("{}", frame);
        return Ok(());
    }

    println!("📟 Launching Kaioken Interactive Terminal UI (ratatui)...");
    run_tui(app, args.poll, daemon_url).await
}
