use crate::desktop::daemon::DaemonService;
use crate::desktop::ipc::DaemonIpcClient;
use crate::desktop::protocol::{IpcEvent, WireMessage};
use crate::memory::MemoryEngine;
use crate::watcher::WatchEngine;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Embedded in-process runtime for standalone desktop usage (Section 2.1).
pub struct EmbeddedRuntime {
    repo_root: PathBuf,
    memory: Arc<std::sync::Mutex<MemoryEngine>>,
    ipc_client: Arc<DaemonIpcClient>,
    event_tx: broadcast::Sender<IpcEvent>,
}

impl EmbeddedRuntime {
    /// Initialize an embedded runtime that hosts memory, watcher, and IPC in background tasks.
    pub async fn start(repo_root: PathBuf, spawn_watcher: bool) -> Result<Self> {
        let db_path = repo_root.join(".agent").join("memory.sqlite");
        let memory = if db_path.exists() {
            Arc::new(std::sync::Mutex::new(MemoryEngine::open(&db_path)?))
        } else {
            Arc::new(std::sync::Mutex::new(MemoryEngine::open_in_memory()?))
        };

        let (ipc_client, mut outgoing_rx, event_tx) = DaemonIpcClient::new_mock();
        let client_arc = Arc::new(ipc_client);

        // Spawn optional watcher in background thread
        if spawn_watcher {
            let root_clone = repo_root.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async move {
                    let db_path = root_clone.join(".agent").join("memory.sqlite");
                    if let Ok(mem) = MemoryEngine::open(&db_path) {
                        let mut watcher = WatchEngine::new(
                            root_clone,
                            Arc::new(mem),
                            300,
                            None,
                            None,
                            &[],
                            false,
                            false,
                        );
                        let _ = watcher.run().await;
                    }
                });
            });
        }

        // Spawn mock request-response dispatcher on a dedicated background thread.
        // It answers through the same `DaemonService` the real daemon uses, so
        // embedded and daemon modes cannot drift apart.
        let client_for_dispatch = client_arc.clone();
        let service = DaemonService::new(
            repo_root.clone(),
            memory.clone(),
            "embedded",
            "embedded in-process",
        );
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                while let Some(msg) = outgoing_rx.recv().await {
                    if let WireMessage::Request { id, request } = msg {
                        let response = service.handle(request);
                        let _ = client_for_dispatch.mock_respond(id, response).await;
                    }
                }
            });
        });

        Ok(Self {
            repo_root,
            memory,
            ipc_client: client_arc,
            event_tx,
        })
    }

    pub fn repo_root(&self) -> &std::path::Path {
        &self.repo_root
    }

    pub fn client(&self) -> Arc<DaemonIpcClient> {
        self.ipc_client.clone()
    }

    pub fn memory(&self) -> Arc<std::sync::Mutex<MemoryEngine>> {
        self.memory.clone()
    }

    pub fn emit_event(&self, event: IpcEvent) {
        let _ = self.event_tx.send(event);
    }
}
