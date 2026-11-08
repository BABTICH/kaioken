use kaioken::desktop::app::KaiokenDesktopController;
use kaioken::desktop::daemon::DaemonService;
use kaioken::desktop::ipc::{
    default_ipc_endpoint, DaemonIpcClient, DaemonIpcServer, DEFAULT_TCP_PORT,
};
use kaioken::desktop::model::ConnectionStatus;
use kaioken::desktop::protocol::{IpcEvent, IpcRequest, IpcResponse};
use kaioken::memory::{DocRecord, EdgeRecord, FileRecord, MemoryEngine, SymbolRecord};
use kaioken::tui::client::TuiClient;
use kaioken::watcher::WatchEngine;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::broadcast;

fn ok_data(response: IpcResponse) -> serde_json::Value {
    match response {
        IpcResponse::Ok(value) => value,
        IpcResponse::Error { code, message } => panic!("expected Ok, got {}: {}", code, message),
    }
}

fn seeded_memory() -> Arc<Mutex<MemoryEngine>> {
    let memory = MemoryEngine::open_in_memory().unwrap();
    let mut file = FileRecord::new("src/a.rs", 32);
    file.language = Some("rust".to_string());
    file.source_hash = Some("hash-a".to_string());
    let file_id = memory.upsert_file(&file).unwrap();

    memory
        .insert_symbols(
            file_id,
            &[
                SymbolRecord {
                    id: None,
                    file_id,
                    name: "alpha".to_string(),
                    kind: "function".to_string(),
                    line_start: 1,
                    line_end: 3,
                    visibility: Some("pub".to_string()),
                    signature: Some("pub fn alpha()".to_string()),
                    summary: None,
                    parent: None,
                },
                SymbolRecord {
                    id: None,
                    file_id,
                    name: "beta".to_string(),
                    kind: "function".to_string(),
                    line_start: 4,
                    line_end: 6,
                    visibility: Some("pub".to_string()),
                    signature: Some("pub fn beta()".to_string()),
                    summary: None,
                    parent: None,
                },
            ],
        )
        .unwrap();

    let symbols = memory.get_all_symbols().unwrap();
    let alpha = symbols
        .iter()
        .find(|symbol| symbol.name == "alpha")
        .and_then(|symbol| symbol.id)
        .unwrap();
    let beta = symbols
        .iter()
        .find(|symbol| symbol.name == "beta")
        .and_then(|symbol| symbol.id)
        .unwrap();
    memory
        .insert_edge(&EdgeRecord {
            id: None,
            source_symbol_id: Some(beta),
            target_symbol_id: Some(alpha),
            edge_type: "calls".to_string(),
        })
        .unwrap();

    memory
        .upsert_doc(&DocRecord {
            id: None,
            file_id,
            doc_path: "src/a.md".to_string(),
            doc_level: 0,
            source_hash: Some("hash-a".to_string()),
            status: "fresh".to_string(),
            updated_at: "2026-10-19T00:00:00Z".to_string(),
            stale_reason: None,
        })
        .unwrap();

    Arc::new(Mutex::new(memory))
}

#[test]
fn test_default_endpoint_is_loopback_tcp() {
    assert_eq!(
        default_ipc_endpoint(),
        format!("127.0.0.1:{}", DEFAULT_TCP_PORT)
    );
    assert_eq!(DEFAULT_TCP_PORT, 47890);
}

#[test]
fn test_daemon_service_reports_memory_and_docs_health() {
    let root = tempdir().unwrap();
    let service = DaemonService::new(
        root.path().to_path_buf(),
        seeded_memory(),
        "daemon",
        "127.0.0.1:47890",
    );

    let stats = ok_data(service.handle(IpcRequest::GetMemoryStats));
    assert_eq!(stats.get("total_files").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(stats.get("total_symbols").and_then(|v| v.as_u64()), Some(2));
    assert_eq!(stats.get("total_edges").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(stats.get("total_docs").and_then(|v| v.as_u64()), Some(1));

    let health = ok_data(service.handle(IpcRequest::GetDocsHealth));
    assert_eq!(health.get("total_docs").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(health.get("fresh_docs").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(health.get("stale_docs").and_then(|v| v.as_u64()), Some(0));

    let graph = ok_data(service.handle(IpcRequest::GetMemoryGraph {
        filter: Some("a.rs".to_string()),
        limit: Some(10),
    }));
    assert_eq!(graph.get("total").and_then(|v| v.as_u64()), Some(2));
    let nodes = graph.get("nodes").and_then(|v| v.as_array()).unwrap();
    assert_eq!(nodes.len(), 2);
    let alpha = nodes
        .iter()
        .find(|node| node.get("name").and_then(|v| v.as_str()) == Some("alpha"))
        .expect("alpha node");
    assert_eq!(
        alpha.get("file_path").and_then(|v| v.as_str()),
        Some("src/a.rs")
    );
    // Memory stores "function"; the desktop graph filters on "fn".
    assert_eq!(alpha.get("kind").and_then(|v| v.as_str()), Some("fn"));
    assert_eq!(
        graph.get("edges").and_then(|v| v.as_array()).map(Vec::len),
        Some(1)
    );
}

#[test]
fn test_daemon_service_status_and_unsupported_mutations() {
    let root = tempdir().unwrap();
    let service = DaemonService::new(
        root.path().to_path_buf(),
        seeded_memory(),
        "daemon",
        "127.0.0.1:47890",
    );

    let status = ok_data(service.handle(IpcRequest::GetStatus));
    assert_eq!(status.get("mode").and_then(|v| v.as_str()), Some("daemon"));
    assert_eq!(
        status.get("pid").and_then(|v| v.as_u64()),
        Some(std::process::id() as u64)
    );

    for request in [
        IpcRequest::StartAgentTask {
            task: "add tests".to_string(),
            mode: None,
        },
        IpcRequest::ApproveToolCall {
            proposal_id: "p1".to_string(),
        },
        IpcRequest::DenyToolCall {
            proposal_id: "p1".to_string(),
            reason: None,
        },
    ] {
        match service.handle(request) {
            IpcResponse::Error { code, message } => {
                assert_eq!(code, -32601);
                assert!(!message.is_empty());
            }
            IpcResponse::Ok(_) => panic!("daemon must not claim it executed an agent action"),
        }
    }
}

#[tokio::test]
async fn test_server_answers_requests_and_streams_events() {
    let server = DaemonIpcServer::bind(0).await.unwrap();
    let addr = server.addr();
    let events = server.event_sender();
    tokio::spawn(async move {
        let _ = server
            .serve(|_request| async move { IpcResponse::success("pong-over-tcp") })
            .await;
    });

    let client = DaemonIpcClient::connect_tcp(&addr).await.unwrap();
    let mut event_rx = client.subscribe_events();

    let response = client.send_request(IpcRequest::Ping).await.unwrap();
    assert_eq!(response, IpcResponse::success("pong-over-tcp"));

    // Give the server's per-connection event subscription a moment to attach.
    tokio::time::sleep(Duration::from_millis(50)).await;
    events
        .send(IpcEvent::FileChanged {
            path: "src/a.rs".to_string(),
            change_type: "modified".to_string(),
        })
        .unwrap();

    let event = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
        .await
        .expect("event should arrive")
        .expect("event channel should stay open");
    assert_eq!(
        event,
        IpcEvent::FileChanged {
            path: "src/a.rs".to_string(),
            change_type: "modified".to_string(),
        }
    );
}

#[tokio::test]
async fn test_slow_daemon_times_out_instead_of_hanging() {
    let server = DaemonIpcServer::bind(0).await.unwrap();
    let addr = server.addr();
    tokio::spawn(async move {
        let _ = server
            .serve(|_request| async move {
                tokio::time::sleep(Duration::from_millis(500)).await;
                IpcResponse::success("too late")
            })
            .await;
    });

    let client = DaemonIpcClient::connect_tcp(&addr).await.unwrap();
    let error = client
        .send_request_with_timeout(IpcRequest::Ping, Duration::from_millis(100))
        .await
        .expect_err("a slow daemon must time out");
    assert!(error.to_string().contains("timed out"), "got: {}", error);
}

#[tokio::test]
async fn test_desktop_controller_drains_ipc_events() {
    let (client, _, event_tx) = DaemonIpcClient::new_mock();
    let mut controller = KaiokenDesktopController::new("kaioRust", "main", Arc::new(client));
    let dirty_before = controller.window.dirty_count;

    event_tx
        .send(IpcEvent::Heartbeat {
            timestamp: "2026-10-21T00:00:00Z".to_string(),
        })
        .unwrap();
    event_tx
        .send(IpcEvent::FileChanged {
            path: "src/a.rs".to_string(),
            change_type: "modified".to_string(),
        })
        .unwrap();

    controller.poll_events();

    assert_eq!(controller.window.dirty_count, dirty_before + 1);
    assert_eq!(
        controller.window.daemon_conn.status,
        ConnectionStatus::Connected
    );
}

#[tokio::test]
async fn test_tui_approval_without_daemon_reports_error() {
    let client = TuiClient::new_offline();
    let error = client
        .approve("proposal-1")
        .await
        .expect_err("offline TUI cannot approve entries");
    assert!(error.to_string().contains("offline"), "got: {}", error);
}

#[tokio::test]
async fn test_watch_engine_publishes_file_and_memory_events() {
    let root = tempdir().unwrap();
    std::fs::write(root.path().join("a.rs"), "pub fn alpha() {}\n").unwrap();

    let memory = Arc::new(MemoryEngine::open_in_memory().unwrap());
    let (event_tx, mut event_rx) = broadcast::channel(8);
    let mut engine = WatchEngine::new(
        root.path().to_path_buf(),
        memory,
        1,
        None,
        None,
        &[],
        false,
        false,
    );
    engine.event_sink = Some(event_tx);

    engine.debounce.record_event(root.path().join("a.rs"));
    tokio::time::sleep(Duration::from_millis(10)).await;
    let outcomes = engine.process_ready();
    assert!(!outcomes.is_empty());

    let first = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
        .await
        .expect("file event should arrive")
        .expect("event channel should stay open");
    assert!(matches!(first, IpcEvent::FileChanged { .. }));

    let second = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
        .await
        .expect("memory event should arrive")
        .expect("event channel should stay open");
    match second {
        IpcEvent::MemoryUpdated { total_files, .. } => assert_eq!(total_files, 1),
        other => panic!("expected MemoryUpdated, got {:?}", other),
    }
}
