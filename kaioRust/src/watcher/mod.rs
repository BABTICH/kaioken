pub mod debounce;
pub mod filter;
pub mod sync;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;

use crate::desktop::protocol::IpcEvent;
use crate::error::Result;
use crate::memory::MemoryEngine;

pub use debounce::{DebounceManager, EventMeta};
pub use filter::PathFilter;
pub use sync::{compute_sha256, sync_path, SyncOutcome};

pub struct WatchEngine {
    pub repo_root: PathBuf,
    pub memory: Arc<MemoryEngine>,
    pub filter: PathFilter,
    pub debounce: DebounceManager,
    pub skip_ast: bool,
    pub verbose: bool,
    /// Optional IPC event channel. When set, each synced change is published to
    /// connected desktop/TUI clients.
    pub event_sink: Option<broadcast::Sender<IpcEvent>>,
}

impl WatchEngine {
    pub fn new(
        repo_root: PathBuf,
        memory: Arc<MemoryEngine>,
        debounce_ms: u64,
        include: Option<&str>,
        exclude: Option<&str>,
        config_ignore: &[String],
        skip_ast: bool,
        verbose: bool,
    ) -> Self {
        Self {
            repo_root,
            memory,
            filter: PathFilter::new(include, exclude, config_ignore),
            debounce: DebounceManager::new(debounce_ms),
            skip_ast,
            verbose,
            event_sink: None,
        }
    }

    /// Process a single path through filter and memory sync.
    pub fn process_path(&mut self, path: &Path) -> Result<SyncOutcome> {
        if !self.filter.should_process(path, &self.repo_root) {
            let rel = path
                .strip_prefix(&self.repo_root)
                .unwrap_or(path)
                .to_path_buf();
            return Ok(SyncOutcome::Ignored {
                path: rel,
                reason: "Filtered by include/exclude rules".to_string(),
            });
        }
        sync_path(&self.repo_root, path, &self.memory, self.skip_ast)
    }

    /// Process all ready debounced paths.
    pub fn process_ready(&mut self) -> Vec<SyncOutcome> {
        let ready_paths = self.debounce.take_ready();
        let mut outcomes = Vec::new();
        for p in ready_paths {
            let start = Instant::now();
            match self.process_path(&p) {
                Ok(outcome) => {
                    let duration = start.elapsed();
                    self.render_outcome(&outcome, duration.as_millis());
                    outcomes.push(outcome);
                }
                Err(e) => {
                    let rel = p.strip_prefix(&self.repo_root).unwrap_or(&p).to_path_buf();
                    let failed = SyncOutcome::Failed {
                        path: rel,
                        error: e.to_string(),
                    };
                    self.render_outcome(&failed, 0);
                    outcomes.push(failed);
                }
            }
        }
        self.publish_outcomes(&outcomes);
        outcomes
    }

    /// Publish one `FileChanged` per mutation plus refreshed memory totals.
    fn publish_outcomes(&self, outcomes: &[SyncOutcome]) {
        let Some(sink) = &self.event_sink else {
            return;
        };

        let mut changed = false;
        for outcome in outcomes {
            let (path, change_type) = match outcome {
                SyncOutcome::Updated { path, .. } => (path, "modified"),
                SyncOutcome::NewFile { path, .. } => (path, "created"),
                SyncOutcome::Deleted { path } => (path, "deleted"),
                _ => continue,
            };
            changed = true;
            let _ = sink.send(IpcEvent::FileChanged {
                path: path.display().to_string(),
                change_type: change_type.to_string(),
            });
        }

        if changed {
            let _ = sink.send(IpcEvent::MemoryUpdated {
                total_files: self.memory.count_files().unwrap_or(0),
                total_symbols: self.memory.count_symbols().unwrap_or(0),
                total_edges: self.memory.count_edges().unwrap_or(0),
            });
        }
    }

    fn render_outcome(&self, outcome: &SyncOutcome, duration_ms: u128) {
        let now_hms = chrono::Local::now().format("%H:%M:%S").to_string();
        let now_hms_ms = chrono::Local::now().format("%H:%M:%S%.3f").to_string();

        if self.verbose {
            match outcome {
                SyncOutcome::Updated {
                    path,
                    symbols_count,
                    old_hash,
                    new_hash,
                    ..
                } => {
                    let old_short = old_hash
                        .as_deref()
                        .map(|h| if h.len() >= 6 { &h[..6] } else { h })
                        .unwrap_or("none");
                    let new_short = if new_hash.len() >= 6 {
                        &new_hash[..6]
                    } else {
                        new_hash
                    };

                    println!("  [{}] EVENT   Modify  {}", now_hms_ms, path.display());
                    println!(
                        "  [{}] HASH    Changed  {}→{}  {}",
                        now_hms_ms,
                        old_short,
                        new_short,
                        path.display()
                    );
                    if !self.skip_ast {
                        println!(
                            "  [{}] AST     Extracted {} symbols",
                            now_hms_ms, symbols_count
                        );
                    }
                    println!(
                        "  [{}] MEMORY  Upserted file + {} symbols to SQLite",
                        now_hms_ms, symbols_count
                    );
                    println!(
                        "  [{}] DONE    {} ({}ms)",
                        now_hms_ms,
                        path.display(),
                        duration_ms
                    );
                }
                SyncOutcome::NewFile {
                    path,
                    symbols_count,
                    new_hash,
                } => {
                    let new_short = if new_hash.len() >= 6 {
                        &new_hash[..6]
                    } else {
                        new_hash
                    };
                    println!("  [{}] EVENT   Create  {}", now_hms_ms, path.display());
                    println!(
                        "  [{}] HASH    New  {}  {}",
                        now_hms_ms,
                        new_short,
                        path.display()
                    );
                    if !self.skip_ast {
                        println!(
                            "  [{}] AST     Extracted {} symbols",
                            now_hms_ms, symbols_count
                        );
                    }
                    println!("  [{}] MEMORY  Created record in SQLite", now_hms_ms);
                    println!(
                        "  [{}] DONE    {} ({}ms)",
                        now_hms_ms,
                        path.display(),
                        duration_ms
                    );
                }
                SyncOutcome::Deleted { path } => {
                    println!("  [{}] EVENT   Delete  {}", now_hms_ms, path.display());
                    println!("  [{}] MEMORY  Removed file from SQLite", now_hms_ms);
                    println!(
                        "  [{}] DONE    {} ({}ms)",
                        now_hms_ms,
                        path.display(),
                        duration_ms
                    );
                }
                SyncOutcome::Unchanged { path, .. } => {
                    println!(
                        "  [{}] HASH    Unchanged (skip AST)  {}",
                        now_hms_ms,
                        path.display()
                    );
                }
                SyncOutcome::Ignored { path, reason } => {
                    println!("  [{}] IGNORE  {} ({})", now_hms_ms, path.display(), reason);
                }
                SyncOutcome::Failed { path, error } => {
                    println!("  [{}] ERROR   {} ({})", now_hms_ms, path.display(), error);
                }
            }
        } else {
            // Normal mode output
            match outcome {
                SyncOutcome::Updated {
                    path,
                    symbols_count,
                    ..
                } => {
                    let sym_label = if *symbols_count == 1 {
                        "symbol"
                    } else {
                        "symbols"
                    };
                    println!(
                        "  [{}] ✏️  {} ({} {} updated)",
                        now_hms,
                        path.display(),
                        symbols_count,
                        sym_label
                    );
                }
                SyncOutcome::NewFile {
                    path,
                    symbols_count,
                    ..
                } => {
                    let sym_label = if *symbols_count == 1 {
                        "symbol"
                    } else {
                        "symbols"
                    };
                    println!(
                        "  [{}] ➕  {} (new file, {} {} extracted)",
                        now_hms,
                        path.display(),
                        symbols_count,
                        sym_label
                    );
                }
                SyncOutcome::Deleted { path } => {
                    println!(
                        "  [{}] 🗑️  {} (removed from memory)",
                        now_hms,
                        path.display()
                    );
                }
                SyncOutcome::Failed { path, error } => {
                    println!("  [{}] ❌  {} ({})", now_hms, path.display(), error);
                }
                _ => {}
            }
        }
    }

    /// Run the watcher with an external shutdown receiver (useful for testing and graceful termination).
    pub async fn run_with_shutdown(
        &mut self,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<()> {
        let (tx, rx): (
            std::sync::mpsc::Sender<notify::Result<Event>>,
            Receiver<notify::Result<Event>>,
        ) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )
        .map_err(|e| {
            crate::error::KaiokenError::ScannerError(format!("Watcher init error: {}", e))
        })?;

        watcher
            .watch(&self.repo_root, RecursiveMode::Recursive)
            .map_err(|e| {
                crate::error::KaiokenError::ScannerError(format!("Watch register error: {}", e))
            })?;

        println!("👁️  Watching {} for changes...", self.repo_root.display());
        println!("   Press Ctrl+C to stop\n");

        loop {
            // Check for external shutdown
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            // Poll events with non-blocking timeout
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(event)) => {
                    for path in event.paths {
                        if self.filter.should_process(&path, &self.repo_root) {
                            let count = self.debounce.record_event(path.clone());
                            if self.verbose && count > 1 {
                                let now_hms_ms =
                                    chrono::Local::now().format("%H:%M:%S%.3f").to_string();
                                let rel = path.strip_prefix(&self.repo_root).unwrap_or(&path);
                                println!(
                                    "  [{}] EVENT   Modify  {}  (debounce merge #{} )",
                                    now_hms_ms,
                                    rel.display(),
                                    count
                                );
                            }
                        }
                    }
                }
                Ok(Err(err)) => {
                    tracing::warn!("Watch error: {}", err);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }

            // Process debounced events that are ready
            self.process_ready();

            // Cooperative yield
            tokio::task::yield_now().await;
        }

        // Flush remaining events before shutdown
        let remaining = self.debounce.take_all();
        let mut flushed = Vec::new();
        for p in remaining {
            let start = Instant::now();
            if let Ok(outcome) = self.process_path(&p) {
                self.render_outcome(&outcome, start.elapsed().as_millis());
                flushed.push(outcome);
            }
        }
        self.publish_outcomes(&flushed);

        Ok(())
    }

    /// Run the watcher interactively until Ctrl+C is received.
    pub async fn run(&mut self) -> Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = tx.send(());
        });

        self.run_with_shutdown(rx).await
    }
}
