use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::Config;
use crate::desktop::daemon::DaemonService;
use crate::desktop::ipc::DaemonIpcServer;
use crate::desktop::protocol::IpcEvent;
use crate::memory::MemoryEngine;
use crate::watcher::WatchEngine;
use anyhow::Context;
use tokio::sync::broadcast;

#[derive(Args, Debug, Clone)]
pub struct WatchArgs {
    /// Debounce interval in milliseconds
    #[arg(long, default_value = "500")]
    pub debounce: u64,
    /// Glob patterns to include (comma-separated)
    #[arg(long)]
    pub include: Option<String>,
    /// Glob patterns to exclude (comma-separated)
    #[arg(long)]
    pub exclude: Option<String>,
    /// Skip AST re-parsing, only track file changes
    #[arg(long)]
    pub no_ast: bool,
    /// Run as a background daemon
    #[arg(long)]
    pub daemon: bool,
    /// PID file path for daemon management
    #[arg(long)]
    pub pid_file: Option<String>,
    /// Log every file change event
    #[arg(long)]
    pub verbose: bool,
    /// TCP port for the local desktop/TUI IPC server
    #[arg(long, default_value_t = crate::desktop::ipc::DEFAULT_TCP_PORT)]
    pub ipc_port: u16,
    /// Do not start the local IPC server
    #[arg(long)]
    pub no_ipc: bool,
}

pub async fn handle_watch_command(args: WatchArgs) -> anyhow::Result<()> {
    let repo_root = std::env::current_dir().context("Failed to get current directory")?;

    let config = Config::load(&repo_root).unwrap_or_else(|_| Config::default_config());

    // Daemon mode: spawn background child process and return immediately
    if args.daemon {
        let pid_file_path = args
            .pid_file
            .map(PathBuf::from)
            .unwrap_or_else(|| repo_root.join(".agent").join("watch.pid"));

        let log_dir = repo_root.join(".agent").join("logs");
        fs::create_dir_all(&log_dir).context("Failed to create logs directory")?;

        let log_file_path = log_dir.join("watch.log");
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .context("Failed to open watch log file")?;
        let log_err = log_file.try_clone().context("Failed to clone log handle")?;

        let current_exe =
            std::env::current_exe().context("Failed to get current executable path")?;

        let mut cmd = Command::new(current_exe);
        cmd.arg("watch");
        cmd.arg("--debounce").arg(args.debounce.to_string());
        if let Some(inc) = &args.include {
            cmd.arg("--include").arg(inc);
        }
        if let Some(exc) = &args.exclude {
            cmd.arg("--exclude").arg(exc);
        }
        if args.no_ast {
            cmd.arg("--no-ast");
        }
        if args.verbose {
            cmd.arg("--verbose");
        }
        if args.no_ipc {
            cmd.arg("--no-ipc");
        } else {
            cmd.arg("--ipc-port").arg(args.ipc_port.to_string());
        }
        cmd.arg("--pid-file").arg(&pid_file_path);

        cmd.stdout(log_file);
        cmd.stderr(log_err);

        let child = cmd
            .spawn()
            .context("Failed to spawn watcher daemon process")?;

        let pid = child.id();
        if let Some(parent) = pid_file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&pid_file_path, pid.to_string()).context("Failed to write PID file")?;

        println!("👁️  Watcher daemon started (PID: {})", pid);
        if !args.no_ipc {
            println!("   IPC: 127.0.0.1:{}", args.ipc_port);
        }
        println!("   Log: {}", log_file_path.display());
        println!("   PID file: {}", pid_file_path.display());
        return Ok(());
    }

    // Normal foreground execution (interactive or child of daemon)
    let pid_file_path = args.pid_file.map(PathBuf::from);
    if let Some(ref p) = pid_file_path {
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(p, std::process::id().to_string());
    }

    let db_path = repo_root.join(&config.memory.database);
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let memory = Arc::new(MemoryEngine::open(&db_path)?);

    // The desktop and TUI connect here. A failed bind is reported but does not
    // stop watching: file synchronization is the primary job, IPC is the view.
    let event_sink = if args.no_ipc {
        None
    } else {
        match start_ipc_server(&repo_root, &db_path, args.ipc_port).await {
            Ok((events, addr)) => {
                println!("🔌 IPC server listening on {}", addr);
                spawn_heartbeat(events.clone());
                Some(events)
            }
            Err(error) => {
                eprintln!(
                    "⚠️  IPC server unavailable on port {}: {} (watcher continues)",
                    args.ipc_port, error
                );
                None
            }
        }
    };

    let mut engine = WatchEngine::new(
        repo_root,
        memory,
        args.debounce,
        args.include.as_deref(),
        args.exclude.as_deref(),
        &config.scanner.ignore,
        args.no_ast,
        args.verbose,
    );
    engine.event_sink = event_sink;

    let res = engine.run().await;

    // Cleanup PID file on exit
    if let Some(ref p) = pid_file_path {
        let _ = fs::remove_file(p);
    }

    Ok(res?)
}

/// Bind the local IPC endpoint and serve daemon requests on a background task.
///
/// The server gets its own SQLite connection. WAL mode allows the watcher to
/// keep writing while clients read, and it avoids sharing a `!Sync` connection
/// across the spawned connection tasks.
async fn start_ipc_server(
    repo_root: &Path,
    db_path: &Path,
    port: u16,
) -> anyhow::Result<(broadcast::Sender<IpcEvent>, String)> {
    let server = DaemonIpcServer::bind(port).await?;
    let addr = server.addr();
    let events = server.event_sender();
    let memory = Arc::new(Mutex::new(MemoryEngine::open(db_path)?));
    let service = DaemonService::new(repo_root.to_path_buf(), memory, "daemon", addr.clone());

    tokio::spawn(async move {
        let result = server
            .serve(move |request| {
                let service = service.clone();
                async move { service.handle(request) }
            })
            .await;
        if let Err(error) = result {
            tracing::warn!("IPC server stopped: {}", error);
        }
    });

    Ok((events, addr))
}

fn spawn_heartbeat(events: broadcast::Sender<IpcEvent>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(15));
        loop {
            ticker.tick().await;
            let event = IpcEvent::Heartbeat {
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            if events.send(event).is_err() {
                // No clients attached; keep ticking, a client may connect later.
            }
        }
    });
}
