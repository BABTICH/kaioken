use kaioken::desktop::app::KaiokenDesktopController;
use kaioken::desktop::embedded::EmbeddedRuntime;
use kaioken::desktop::ipc::{DaemonIpcClient, DaemonIpcServer};
use kaioken::desktop::model::{ActiveWorkspaceTab, ConnectionStatus, ReviewStateModel};
use kaioken::desktop::protocol::{
    IpcEvent, IpcRequest, IpcResponse, PolicyTier, ToolProposal, WireMessage,
};
use kaioken::desktop::view::KaiokenAppWindow;
use kaioken::review::scoring::{RiskAssessment, RiskFactor, RiskLevel};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_desktop_window_initialization_defaults() {
    let window = KaiokenAppWindow::new("test-repo", "main");
    assert_eq!(window.repo_name, "test-repo");
    assert_eq!(window.branch, "main");
    assert_eq!(window.active_tab, ActiveWorkspaceTab::Chat);
    assert_eq!(window.theme.name, "Kaioken Dark");
    assert_eq!(window.memory_graph.total_files, 128);
    assert_eq!(window.memory_graph.total_symbols, 12);
    assert!(!window.command_palette.is_open);
    assert_eq!(window.research.power_level, 4);
    assert_eq!(window.wiki.chapters.len(), 5);
    assert!(!window.failure_cards.is_empty());
    assert_eq!(window.settings.permission_mode, "edit-with-approval");
}

#[test]
fn test_desktop_route_catalog_excludes_prism() {
    let routes = ActiveWorkspaceTab::all();
    assert_eq!(routes.len(), 11);
    assert!(routes.iter().all(|route| route.id() != "prism"));
    assert_eq!(
        ActiveWorkspaceTab::from_id("review"),
        Some(ActiveWorkspaceTab::Editor)
    );
    assert_eq!(
        ActiveWorkspaceTab::from_id("audit"),
        Some(ActiveWorkspaceTab::Activity)
    );
    assert_eq!(
        ActiveWorkspaceTab::from_id("health"),
        Some(ActiveWorkspaceTab::Wiki)
    );
    assert_eq!(
        ActiveWorkspaceTab::from_id("settings").map(|route| route.shortcut()),
        Some("Ctrl+,")
    );
    assert_eq!(ActiveWorkspaceTab::Cost.shortcut(), "");
}

#[test]
fn test_desktop_tab_switching_navigation() {
    let mut window = KaiokenAppWindow::new("kaioken", "master");

    for tab in [
        ActiveWorkspaceTab::Research,
        ActiveWorkspaceTab::Wiki,
        ActiveWorkspaceTab::Graph,
        ActiveWorkspaceTab::Cards,
        ActiveWorkspaceTab::Editor,
        ActiveWorkspaceTab::Browser,
        ActiveWorkspaceTab::Activity,
        ActiveWorkspaceTab::Extensions,
        ActiveWorkspaceTab::Cost,
        ActiveWorkspaceTab::Settings,
        ActiveWorkspaceTab::Chat,
    ] {
        window.set_tab(tab);
        assert_eq!(window.active_tab, tab);
    }
}

#[test]
fn test_desktop_command_palette_toggle_and_filtering() {
    let mut window = KaiokenAppWindow::new("kaioken", "main");

    window.toggle_command_palette();
    assert!(window.command_palette.is_open);

    window.command_palette.query = "Doctor".to_string();
    let filtered = window.command_palette.filtered_actions();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Run Environment Doctor");

    window.command_palette.query = "Switch".to_string();
    let switch_actions = window.command_palette.filtered_actions();
    assert_eq!(switch_actions.len(), 11);
    assert!(switch_actions
        .iter()
        .all(|action| !action.title.contains("PRISM")));

    window.toggle_command_palette();
    assert!(!window.command_palette.is_open);
}

#[test]
fn test_desktop_terminal_frame_rendering() {
    let mut window = KaiokenAppWindow::new("kaioken", "feature/desktop");
    window.memory_graph.total_files = 42;
    window.memory_graph.total_symbols = 312;
    window.memory_graph.total_edges = 840;
    window.health_explorer.health_score = 94;

    let frame = window.render_terminal_frame();

    assert!(frame.contains("kaioken/feature/desktop"));
    assert!(frame.contains("health 94/100"));
    assert!(frame.contains("💬"));
    assert!(frame.contains("Chat"));
    assert!(frame.contains("Research"));
    assert!(frame.contains("Activity"));
    assert!(frame.contains("Settings"));
    assert!(frame.contains("42 files · 312 sym · 840 edges"));
    assert!(frame.contains("[Ctrl+K] Palette"));
    assert!(!frame.contains("PRISM"));
}

#[test]
fn test_research_dossier_tick_progression() {
    let mut model = kaioken::desktop::model::ResearchDossierModel {
        is_running: true,
        total_claims: 4,
        steps: vec![
            kaioken::desktop::model::ResearchStepModel {
                label: "Scope".to_string(),
                detail: String::new(),
                status: "active".to_string(),
            },
            kaioken::desktop::model::ResearchStepModel {
                label: "Write".to_string(),
                detail: String::new(),
                status: "pending".to_string(),
            },
        ],
        ..Default::default()
    };

    model.tick();
    assert_eq!(model.steps[0].status, "done");
    assert_eq!(model.steps[1].status, "active");
    assert!(model.is_running);

    model.tick();
    assert_eq!(model.steps[1].status, "done");
    assert!(!model.is_running);
    assert_eq!(model.rounds, 1);
    assert_eq!(model.searched, 4);
}

#[test]
fn test_snapshot_exposes_all_routes_and_showcase_models() {
    let window = KaiokenAppWindow::new("kaioken", "main");
    let snapshot = window.snapshot_json();
    let routes = snapshot["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 11);
    assert!(routes
        .iter()
        .all(|route| route["id"] != serde_json::json!("prism")));
    assert!(snapshot["research"]["answer_markdown"]
        .as_str()
        .unwrap()
        .contains("Tauri"));
    assert!(snapshot["wiki"]["chapters"].as_array().unwrap().len() >= 5);
    assert!(snapshot["editor"]["diff"]
        .as_str()
        .unwrap()
        .contains("gui.rs"));
    assert_eq!(snapshot["active_tab"], serde_json::json!("chat"));
}

#[tokio::test]
async fn test_ipc_mock_client_request_response() {
    let (client, mut outgoing_rx, _) = DaemonIpcClient::new_mock();
    let client = Arc::new(client);
    let client_responder = client.clone();

    // Spawn mock server handler
    tokio::spawn(async move {
        if let Some(WireMessage::Request { id, request }) = outgoing_rx.recv().await {
            assert_eq!(request, IpcRequest::Ping);
            let _ = client_responder
                .mock_respond(id, IpcResponse::success("pong-from-mock"))
                .await;
        }
    });

    let resp = client.send_request(IpcRequest::Ping).await.unwrap();
    assert_eq!(resp, IpcResponse::success("pong-from-mock"));
}

#[tokio::test]
async fn test_ipc_mock_event_streaming() {
    let (client, _, event_tx) = DaemonIpcClient::new_mock();
    let mut sub = client.subscribe_events();

    let thought_event = IpcEvent::AgentThought {
        task_id: "task-101".to_string(),
        thought: "Analyzing parser error recovery...".to_string(),
        timestamp: "2026-10-19T10:00:00Z".to_string(),
    };

    let _ = event_tx.send(thought_event.clone());

    let received = sub.recv().await.unwrap();
    assert_eq!(received, thought_event);
}

#[tokio::test]
async fn test_ipc_real_tcp_server_client() {
    // Bind server to ephemeral port 0
    let server = DaemonIpcServer::bind(0).await.unwrap();
    let addr = server.addr();

    tokio::spawn(async move {
        let _ = server
            .serve(|req| async move {
                match req {
                    IpcRequest::Ping => IpcResponse::success("pong-tcp"),
                    _ => IpcResponse::success("ack"),
                }
            })
            .await;
    });

    // Connect client over real TCP
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let client = DaemonIpcClient::connect_tcp(&addr).await.unwrap();

    let resp = client.send_request(IpcRequest::Ping).await.unwrap();
    assert_eq!(resp, IpcResponse::success("pong-tcp"));
    assert!(client.is_connected().await);
}

#[tokio::test]
async fn test_tool_proposal_interactive_approval_flow() {
    let (client, mut outgoing_rx, _) = DaemonIpcClient::new_mock();
    let client_arc = Arc::new(client);
    let mut controller = KaiokenDesktopController::new("kaioken", "main", client_arc.clone());

    // Mock daemon answering tool approvals
    let client_for_respond = client_arc.clone();
    tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if let WireMessage::Request {
                id,
                request: IpcRequest::ApproveToolCall { proposal_id },
            } = msg
            {
                let _ = client_for_respond
                    .mock_respond(
                        id,
                        IpcResponse::success(serde_json::json!({
                            "proposal_id": proposal_id,
                            "approved": true
                        })),
                    )
                    .await;
            }
        }
    });

    // Push proposal
    let prop = ToolProposal::new_bash("prop-1", "cargo check", false);
    controller.handle_ipc_event(IpcEvent::ToolProposed(prop));

    assert_eq!(controller.window.agent_session.pending_proposals.len(), 1);

    // User approves via Enter key
    let res = controller.handle_action("Enter").await.unwrap();
    assert!(res.is_some());
    assert!(res.unwrap().contains("Approved proposal prop-1"));

    // Proposal resolved
    assert_eq!(controller.window.agent_session.pending_proposals.len(), 0);
    assert_eq!(controller.window.agent_session.total_approved, 5);
}

#[tokio::test]
async fn test_tool_proposal_interactive_denial_flow() {
    let (client, mut outgoing_rx, _) = DaemonIpcClient::new_mock();
    let client_arc = Arc::new(client);
    let mut controller = KaiokenDesktopController::new("kaioken", "main", client_arc.clone());

    // Mock daemon answering tool denials
    let client_for_respond = client_arc.clone();
    tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if let WireMessage::Request { id, .. } = msg {
                let _ = client_for_respond
                    .mock_respond(id, IpcResponse::success("denied-ack"))
                    .await;
            }
        }
    });

    // Push edit proposal
    let prop = ToolProposal::new_edit("prop-2", "src/scanner/git.rs", "+ fn diff() {}");
    controller.handle_ipc_event(IpcEvent::ToolProposed(prop));

    assert_eq!(controller.window.agent_session.pending_proposals.len(), 1);

    // User denies via Escape key
    let res = controller.handle_action("Escape").await.unwrap();
    assert!(res.is_some());
    assert!(res.unwrap().contains("Denied proposal prop-2"));

    // Proposal resolved as denied
    assert_eq!(controller.window.agent_session.pending_proposals.len(), 0);
    assert_eq!(controller.window.agent_session.total_denied, 1);
}

#[test]
fn test_policy_tier_classification_and_review_state() {
    assert_eq!(PolicyTier::ReadOnly.to_string(), "READ_ONLY");
    assert_eq!(PolicyTier::WritePrompt.to_string(), "WRITE_PROMPT");
    assert_eq!(PolicyTier::Dangerous.to_string(), "DANGEROUS");

    // Review risk calculation
    let factors = vec![
        RiskFactor::BreakingSignatureChange {
            symbol: "parse()".to_string(),
            weight: 0.35,
        },
        RiskFactor::HighFanOutSymbol {
            symbol: "parse()".to_string(),
            callers: 12,
            weight: 0.25,
        },
    ];
    let assessment = RiskAssessment::calculate(factors);
    assert_eq!(assessment.level, RiskLevel::High);

    let review_model = ReviewStateModel::from_assessment(&assessment, "diff --git");
    assert_eq!(review_model.risk_score, 60);
    assert_eq!(review_model.risk_level, "High");
    assert_eq!(review_model.risk_reasons.len(), 2);
}

#[test]
fn test_snapshot_reports_endpoint_port() {
    let mut window = KaiokenAppWindow::new("kaioRust", "main");
    window.daemon_conn.endpoint = "127.0.0.1:5555".to_string();
    let snapshot = window.snapshot_json();
    assert_eq!(snapshot["daemon"]["port"], 5555);
    assert_eq!(snapshot["daemon"]["endpoint"], "127.0.0.1:5555");
}

#[tokio::test]
async fn test_embedded_runtime_start_and_query() {
    let temp = tempdir().unwrap();
    let runtime = EmbeddedRuntime::start(temp.path().to_path_buf(), false)
        .await
        .unwrap();

    let client = runtime.client();
    let resp = client.send_request(IpcRequest::Ping).await.unwrap();
    match resp {
        IpcResponse::Ok(value) => {
            assert_eq!(value.get("pong").and_then(|v| v.as_bool()), Some(true));
            assert_eq!(value.get("mode").and_then(|v| v.as_str()), Some("embedded"));
        }
        IpcResponse::Error { code, message } => panic!("ping failed with {}: {}", code, message),
    }

    let stats_resp = client
        .send_request(IpcRequest::GetMemoryStats)
        .await
        .unwrap();
    if let IpcResponse::Ok(val) = stats_resp {
        assert_eq!(val.get("total_files").unwrap().as_u64().unwrap(), 0);
        assert_eq!(val.get("total_symbols").unwrap().as_u64().unwrap(), 0);
    } else {
        panic!("Expected Ok response from GetMemoryStats");
    }

    let mut controller = KaiokenDesktopController::new("kaioRust", "main", client);
    controller.initial_sync().await.unwrap();
    assert_eq!(controller.window.settings.daemon_mode, "embedded");
    assert_eq!(controller.window.settings.endpoint, "embedded in-process");
    assert_eq!(
        controller.window.daemon_conn.status,
        ConnectionStatus::Connected
    );
    // A fresh store must report 0, not the seeded showcase numbers.
    assert_eq!(controller.window.memory_graph.total_files, 0);
    assert_eq!(controller.window.health_explorer.total_docs, 0);
}
