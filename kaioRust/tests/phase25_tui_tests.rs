use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kaioken::desktop::ipc::DaemonIpcClient;
use kaioken::desktop::protocol::{IpcEvent, PolicyTier, ToolProposal};
use kaioken::review::scoring::{RiskAssessment, RiskFactor, RiskLevel};
use kaioken::safety::audit::AuditLogEntry;
use kaioken::tui::app::{ActiveTab, DaemonConnectionState, TuiApp, TuiSymbolItem};
use kaioken::tui::client::TuiClient;
use kaioken::tui::event::{handle_key, TuiAction};
use kaioken::tui::render_tui_headless;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_tui_initialization_defaults() {
    let app = TuiApp::new("test-repo", "main", PathBuf::from("."));
    assert_eq!(app.repo_name, "test-repo");
    assert_eq!(app.branch, "main");
    assert_eq!(app.active_tab, ActiveTab::Dashboard);
    assert_eq!(app.daemon_state, DaemonConnectionState::Connected);
    assert_eq!(app.health_score, 95);
    assert_eq!(app.doc_coverage, 90);
    assert_eq!(app.total_files, 0);
    assert_eq!(app.total_symbols, 0);
    assert_eq!(app.active_locks, 0);
    assert!(!app.read_only);
    assert!(!app.should_quit);
    assert!(!app.is_paused);
    assert!(!app.show_help);
}

#[test]
fn test_tui_tab_cycling_and_navigation() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));

    // Forward cycling
    assert_eq!(app.active_tab, ActiveTab::Dashboard);
    app.next_tab();
    assert_eq!(app.active_tab, ActiveTab::Memory);
    app.next_tab();
    assert_eq!(app.active_tab, ActiveTab::Task);
    app.next_tab();
    assert_eq!(app.active_tab, ActiveTab::Review);
    app.next_tab();
    assert_eq!(app.active_tab, ActiveTab::Audit);
    app.next_tab();
    assert_eq!(app.active_tab, ActiveTab::Dashboard);

    // Backward cycling
    app.prev_tab();
    assert_eq!(app.active_tab, ActiveTab::Audit);
    app.prev_tab();
    assert_eq!(app.active_tab, ActiveTab::Review);

    // Named tab parsing
    assert_eq!(ActiveTab::from_str_name("memory"), ActiveTab::Memory);
    assert_eq!(ActiveTab::from_str_name("graph"), ActiveTab::Memory);
    assert_eq!(ActiveTab::from_str_name("task"), ActiveTab::Task);
    assert_eq!(ActiveTab::from_str_name("runner"), ActiveTab::Task);
    assert_eq!(ActiveTab::from_str_name("review"), ActiveTab::Review);
    assert_eq!(ActiveTab::from_str_name("diff"), ActiveTab::Review);
    assert_eq!(ActiveTab::from_str_name("audit"), ActiveTab::Audit);
    assert_eq!(ActiveTab::from_str_name("safety"), ActiveTab::Audit);
    assert_eq!(ActiveTab::from_str_name("dashboard"), ActiveTab::Dashboard);
    assert_eq!(ActiveTab::from_str_name("unknown"), ActiveTab::Dashboard);
}

#[test]
fn test_tui_pause_and_help_toggle() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));

    // Test pause
    assert!(!app.is_paused);
    let initial_len = app.live_activity.len();
    app.push_activity("event-1");
    assert_eq!(app.live_activity.len(), initial_len + 1);

    app.toggle_pause();
    assert!(app.is_paused);
    let paused_len = app.live_activity.len();
    app.push_activity("dropped-while-paused");
    assert_eq!(app.live_activity.len(), paused_len);

    app.toggle_pause();
    assert!(!app.is_paused);

    // Test help
    assert!(!app.show_help);
    app.toggle_help();
    assert!(app.show_help);
    app.toggle_help();
    assert!(!app.show_help);
}

#[test]
fn test_tui_keyboard_actions() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));

    // Quit on 'q'
    let action = handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
    );
    assert_eq!(action, TuiAction::Quit);
    assert!(app.should_quit);

    // Refresh on 'r'
    let action = handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('r'), KeyModifiers::empty()),
    );
    assert_eq!(action, TuiAction::Refresh);

    // Jump to tab '2'
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('2'), KeyModifiers::empty()),
    );
    assert_eq!(app.active_tab, ActiveTab::Memory);

    // Jump to tab '3'
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('3'), KeyModifiers::empty()),
    );
    assert_eq!(app.active_tab, ActiveTab::Task);

    // Jump to tab '4'
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('4'), KeyModifiers::empty()),
    );
    assert_eq!(app.active_tab, ActiveTab::Review);

    // Jump to tab '5'
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('5'), KeyModifiers::empty()),
    );
    assert_eq!(app.active_tab, ActiveTab::Audit);

    // Jump to tab '1'
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty()),
    );
    assert_eq!(app.active_tab, ActiveTab::Dashboard);
}

#[test]
fn test_tui_interactive_tool_approval_flow() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));

    // Push proposal
    let prop = ToolProposal::new_bash("prop-101", "cargo check", false);
    app.set_proposal(prop);
    assert!(app.pending_proposal.is_some());
    assert_eq!(app.approved_proposals_count, 0);

    // Approve via 'y'
    let action = handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty()),
    );
    assert_eq!(action, TuiAction::ApproveProposal("prop-101".to_string()));
    assert_eq!(app.approved_proposals_count, 1);
    assert!(app.pending_proposal.is_none());
}

#[test]
fn test_tui_interactive_tool_rejection_flow() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));

    // Push proposal
    let prop = ToolProposal::new_edit("prop-202", "src/auth.rs", "+ fn login() {}");
    app.set_proposal(prop);
    assert!(app.pending_proposal.is_some());
    assert_eq!(app.rejected_proposals_count, 0);

    // Reject via 'n'
    let action = handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty()),
    );
    assert_eq!(action, TuiAction::RejectProposal("prop-202".to_string()));
    assert_eq!(app.rejected_proposals_count, 1);
    assert!(app.pending_proposal.is_none());
}

#[test]
fn test_tui_read_only_mode_blocks_approval() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));
    app.read_only = true;

    let prop = ToolProposal::new_bash("prop-303", "rm -rf target", true);
    app.set_proposal(prop);

    // Attempting to approve in read-only mode should be rejected/ignored
    let action = handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty()),
    );
    assert_eq!(action, TuiAction::None);
    assert!(app.pending_proposal.is_some());
    assert_eq!(app.approved_proposals_count, 0);
}

#[test]
fn test_tui_symbol_search_filtering_and_scrolling() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));
    app.symbols = vec![
        TuiSymbolItem {
            name: "parse_ast".to_string(),
            kind: "fn".to_string(),
            file_path: "src/parser.rs".to_string(),
            line_number: 10,
            visibility: "pub".to_string(),
            doc_summary: Some("Parses AST".to_string()),
            callers_count: 5,
        },
        TuiSymbolItem {
            name: "format_doc".to_string(),
            kind: "fn".to_string(),
            file_path: "src/docs.rs".to_string(),
            line_number: 45,
            visibility: "pub".to_string(),
            doc_summary: None,
            callers_count: 2,
        },
        TuiSymbolItem {
            name: "MemoryEngine".to_string(),
            kind: "struct".to_string(),
            file_path: "src/memory/db.rs".to_string(),
            line_number: 1,
            visibility: "pub".to_string(),
            doc_summary: Some("SQLite memory storage".to_string()),
            callers_count: 12,
        },
    ];

    assert_eq!(app.filtered_symbols().len(), 3);

    // Filter by "memory"
    app.symbol_search_query = "memory".to_string();
    let filtered = app.filtered_symbols();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "MemoryEngine");

    // Scroll down and up
    app.symbol_search_query.clear();
    app.active_tab = ActiveTab::Memory;
    assert_eq!(app.selected_symbol_idx, 0);
    app.scroll_down();
    assert_eq!(app.selected_symbol_idx, 1);
    app.scroll_down();
    assert_eq!(app.selected_symbol_idx, 2);
    app.scroll_down(); // clamp
    assert_eq!(app.selected_symbol_idx, 2);
    app.scroll_up();
    assert_eq!(app.selected_symbol_idx, 1);
}

#[test]
fn test_tui_risk_assessment_application() {
    let mut app = TuiApp::new("test-repo", "main", PathBuf::from("."));

    let factors = vec![
        RiskFactor::BreakingSignatureChange {
            symbol: "run()".to_string(),
            weight: 0.40,
        },
        RiskFactor::HighFanOutSymbol {
            symbol: "run()".to_string(),
            callers: 15,
            weight: 0.35,
        },
    ];
    let assessment = RiskAssessment::calculate(factors);
    assert_eq!(assessment.level, RiskLevel::Critical);

    app.apply_risk_assessment(&assessment, "diff --git a/src/lib.rs b/src/lib.rs");
    assert_eq!(app.risk_score, 75);
    assert_eq!(app.risk_level, "Critical");
    assert_eq!(app.risk_reasons.len(), 2);
    assert!(app.diff_content.contains("diff --git"));
}

#[test]
fn test_tui_headless_frame_snapshot() {
    let mut app = TuiApp::new("kaioken", "feature/tui", PathBuf::from("."));
    app.health_score = 92;
    app.doc_coverage = 87;
    app.total_files = 48;
    app.total_symbols = 340;

    let frame = render_tui_headless(&app, 100, 30).unwrap();

    assert!(frame.contains("Kaioken v0.1.0"));
    assert!(frame.contains("Repo: kaioken/feature/tui"));
    assert!(frame.contains("[1] Dashboard"));
    assert!(frame.contains("[2] Memory Graph"));
    assert!(frame.contains("[3] Task Runner"));
    assert!(frame.contains("[4] Review"));
    assert!(frame.contains("[5] Audit Log"));
    assert!(frame.contains("Workspace Health"));
    assert!(frame.contains("Health Score:   92/100"));
    assert!(frame.contains("Doc Coverage:   87%"));
    assert!(frame.contains("Total Files:    48"));
    assert!(frame.contains("Total Symbols:  340"));
    assert!(frame.contains("Live Activity / Agent Output"));
    assert!(frame.contains("[Tab] Switch"));
}

#[test]
fn test_tui_headless_help_modal_overlay() {
    let mut app = TuiApp::new("kaioken", "main", PathBuf::from("."));
    app.show_help = true;

    let frame = render_tui_headless(&app, 100, 30).unwrap();

    assert!(frame.contains("Keyboard Shortcuts & Help"));
    assert!(frame.contains("Tab / Shift+Tab"));
    assert!(frame.contains("1 .. 5"));
    assert!(frame.contains("y / Enter"));
    assert!(frame.contains("n / Esc"));
}

#[test]
fn test_tui_client_daemon_streaming_events() {
    let (ipc_client, _, event_tx) = DaemonIpcClient::new_mock();
    let mut client = TuiClient::new_with_client(Arc::new(ipc_client));
    let mut app = TuiApp::new("kaioken", "main", PathBuf::from("."));

    // 1. Send Agent Thought
    event_tx
        .send(IpcEvent::AgentThought {
            task_id: "t-1".to_string(),
            thought: "Refactoring AST parser...".to_string(),
            timestamp: "now".to_string(),
        })
        .unwrap();

    // 2. Send Tool Proposal
    event_tx
        .send(IpcEvent::ToolProposed(ToolProposal {
            id: "p-1".to_string(),
            tool_name: "edit_file".to_string(),
            target: "src/tui.rs".to_string(),
            policy_tier: PolicyTier::WritePrompt,
            details: "Add TUI helper".to_string(),
            diff: None,
            proposed_at: "now".to_string(),
        }))
        .unwrap();

    // 3. Send Memory Updated
    event_tx
        .send(IpcEvent::MemoryUpdated {
            total_files: 55,
            total_symbols: 420,
            total_edges: 890,
        })
        .unwrap();

    // 4. Send Audit Log
    event_tx
        .send(IpcEvent::AuditLogAppended(Box::new(AuditLogEntry {
            id: Some(1),
            actor: "agent".to_string(),
            action: "bash".to_string(),
            target: "cargo check".to_string(),
            approved: true,
            reason: None,
            created_at: "2026-10-19T12:00:00Z".to_string(),
        })))
        .unwrap();

    // Poll events into app
    client.poll_events(&mut app);

    assert!(app
        .agent_logs
        .iter()
        .any(|l| l.contains("Refactoring AST parser...")));
    assert!(app.pending_proposal.is_some());
    assert_eq!(app.total_files, 55);
    assert_eq!(app.total_symbols, 420);
    assert_eq!(app.audit_entries.len(), 1);
    assert_eq!(app.audit_entries[0].action, "bash");
}
