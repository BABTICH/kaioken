use kaioken::desktop::app::KaiokenDesktopController;
use kaioken::desktop::daemon::DaemonService;
use kaioken::desktop::ipc::{DaemonIpcClient, DaemonIpcServer};
use kaioken::desktop::model::ActiveWorkspaceTab;
use kaioken::desktop::protocol::{IpcRequest, IpcResponse};
use kaioken::desktop::view::KaiokenAppWindow;
use kaioken::memory::MemoryEngine;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

fn setup_test_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("create temp dir");
    let p = dir.path();

    // Initialize git repo
    let _ = Command::new("git").arg("init").current_dir(p).output();
    let _ = Command::new("git").args(["config", "user.name", "KaiokenTester"]).current_dir(p).output();
    let _ = Command::new("git").args(["config", "user.email", "tester@kaioken.dev"]).current_dir(p).output();

    // Create docs and files
    fs::create_dir_all(p.join("docs")).expect("create docs dir");
    fs::write(
        p.join("docs").join("architecture.md"),
        "# Architecture Overview\n\nKaioken uses hybrid memory graphs and live WebView2 surfaces.",
    )
    .expect("write arch doc");

    fs::create_dir_all(p.join("src")).expect("create src dir");
    fs::write(
        p.join("src").join("main.rs"),
        "fn main() {\n    println!(\"Hello Kaioken\");\n}\n",
    )
    .expect("write main.rs");

    fs::write(
        p.join("Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");

    // Commit baseline so git branch and uncommitted changes work
    let _ = Command::new("git").args(["add", "."]).current_dir(p).output();
    let _ = Command::new("git").args(["commit", "-m", "Initial baseline"]).current_dir(p).output();

    // Create a dirty uncommitted change
    fs::write(
        p.join("src").join("main.rs"),
        "fn main() {\n    println!(\"Modified Kaioken\");\n}\n",
    )
    .expect("modify main.rs");

    dir
}

#[tokio::test]
async fn test_live_workspace_loading() {
    let repo = setup_test_repo();
    let mut window = KaiokenAppWindow::new("sample", "main");

    window.load_live_workspace(repo.path(), None);

    // Verify dynamic repo details
    assert_eq!(window.repo_name, repo.path().file_name().unwrap().to_str().unwrap());
    assert_eq!(window.repo_path, repo.path().display().to_string());
    assert!(!window.branch.is_empty());
    assert!(window.dirty_count >= 1, "Should detect changed files");

    // Verify real wiki chapters loaded from docs/
    assert_eq!(window.wiki.chapters.len(), 1);
    assert_eq!(window.wiki.chapters[0].title, "Architecture Overview");
    assert!(window.wiki.chapters[0].markdown.contains("WebView2 surfaces"));

    // Verify real files loaded into Editor
    assert!(!window.editor.files.is_empty());
    let main_file = window.editor.files.iter().find(|f| f.path == "src/main.rs");
    assert!(main_file.is_some());
    assert!(main_file.unwrap().content.contains("Modified Kaioken"));

    // Verify real review diff populated
    assert!(!window.review_state.raw_diff.is_empty());
    assert!(window.review_state.raw_diff.contains("Modified Kaioken"));
}

#[tokio::test]
async fn test_live_terminal_and_file_operations() {
    let repo = setup_test_repo();
    let (client, _, _) = DaemonIpcClient::new_mock();
    let mut controller = KaiokenDesktopController::new("sample", "main", Arc::new(client));
    controller.window.repo_path = repo.path().display().to_string();

    // 1. Live Terminal Command
    let term_result = controller.execute_terminal_command("echo live_terminal_test").unwrap();
    assert!(term_result.contains("executed successfully") || term_result.is_empty());
    assert!(
        controller
            .window
            .editor
            .terminal_lines
            .iter()
            .any(|l| l.text.contains("live_terminal_test"))
    );

    // 2. Open and Read File
    let open_res = controller.open_file_in_editor("src/main.rs").unwrap();
    assert!(open_res.contains("Opened src/main.rs"));
    assert_eq!(controller.window.editor.active_path, "src/main.rs");

    // 3. Save File
    let new_content = "fn main() {\n    println!(\"Saved through Desktop Controller\");\n}\n";
    let save_res = controller.save_file_from_editor("src/main.rs", new_content).unwrap();
    assert!(save_res.contains("Saved src/main.rs"));
    let disk_content = fs::read_to_string(repo.path().join("src/main.rs")).unwrap();
    assert_eq!(disk_content, new_content);

    // 4. Path Traversal Guard
    let traversal_err = controller.open_file_in_editor("../outside.txt");
    assert!(traversal_err.is_err(), "Path traversal must be forbidden");
}

#[tokio::test]
async fn test_live_run_command_actions() {
    let repo = setup_test_repo();
    let (client, _, _) = DaemonIpcClient::new_mock();
    let mut controller = KaiokenDesktopController::new("sample", "main", Arc::new(client));
    controller.window.repo_path = repo.path().display().to_string();

    // 1. Run live scan
    let scan_res = controller.run_command_action("scan");
    assert!(scan_res.contains("Scanned"));
    assert_eq!(controller.window.active_tab, ActiveWorkspaceTab::Activity);
    assert!(controller.window.memory_graph.total_files >= 3);
    assert_eq!(controller.window.activity.runs[0].state, "done");

    // 2. Run live doctor
    let doc_res = controller.run_command_action("doctor");
    assert!(doc_res.contains("Doctor completed"));
    assert!(controller.window.health_explorer.health_score > 0);

    // 3. Run live git diff
    let diff_res = controller.run_command_action("git_diff");
    assert!(diff_res.contains("Refreshed working tree diff"));
    assert_eq!(controller.window.active_tab, ActiveWorkspaceTab::Editor);

    // 4. Run live wiki refresh
    let wiki_res = controller.run_command_action("start_wiki");
    assert!(wiki_res.contains("Wiki refreshed with 1 documentation chapters"));
    assert_eq!(controller.window.active_tab, ActiveWorkspaceTab::Wiki);
}

#[tokio::test]
async fn test_live_failure_resolution_and_hooks() {
    let repo = setup_test_repo();
    let (client, _, _) = DaemonIpcClient::new_mock();
    let mut controller = KaiokenDesktopController::new("sample", "main", Arc::new(client));
    controller.window.repo_path = repo.path().display().to_string();

    // Seed a test failure card
    controller.window.failure_cards.push(kaioken::desktop::model::FailureCardModel {
        id: "failure-001".to_string(),
        title: "Test compiler error".to_string(),
        module: "compiler".to_string(),
        severity: "high".to_string(),
        occurrences: 1,
        status: "unresolved".to_string(),
        last_seen: "now".to_string(),
        gotchas: vec!["gotcha 1".to_string()],
        anti_pattern_rules: vec![],
    });

    let res = controller.resolve_failure_by_id("failure-001", "Fixed by adding type annotation").unwrap();
    assert!(res.contains("Resolved failure failure-001"));
    assert_eq!(controller.window.failure_cards[0].status, "resolved");
    assert!(controller.window.failure_cards[0].anti_pattern_rules.iter().any(|r| r.contains("Fixed by adding type annotation")));

    // Toggle Git Hook
    let hook_install = controller.toggle_git_hook(true).unwrap();
    assert!(hook_install.contains("installed"));
    assert_eq!(controller.window.extensions.hooks[0].enabled, true);

    let hook_uninstall = controller.toggle_git_hook(false).unwrap();
    assert!(hook_uninstall.contains("uninstalled"));
    assert_eq!(controller.window.extensions.hooks[0].enabled, false);
}

#[tokio::test]
async fn test_live_daemon_ipc_requests() {
    let repo = setup_test_repo();
    let repo_root = repo.path().to_path_buf();

    // Bind real TCP IPC server
    let server = DaemonIpcServer::bind(0).await.unwrap();
    let addr = server.addr();

    let db_path = repo_root.join(".agent").join("memory.sqlite");
    let _ = fs::create_dir_all(repo_root.join(".agent"));
    let memory = Arc::new(Mutex::new(MemoryEngine::open(&db_path).unwrap()));
    let service = DaemonService::new(repo_root.clone(), memory, "daemon", addr.to_string());

    tokio::spawn(async move {
        let _ = server.serve(move |req| {
            let svc = service.clone();
            async move { svc.handle(req) }
        }).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let client = DaemonIpcClient::connect_tcp(&addr).await.unwrap();

    // 1. Read File via IPC
    let read_resp = client.send_request(IpcRequest::ReadFile { path: "src/main.rs".to_string() }).await.unwrap();
    if let IpcResponse::Ok(val) = read_resp {
        assert!(val.get("content").unwrap().as_str().unwrap().contains("Modified Kaioken"));
    } else {
        panic!("ReadFile IPC failed: {:?}", read_resp);
    }

    // 2. Save File via IPC
    let save_resp = client.send_request(IpcRequest::SaveFile {
        path: "src/hello.rs".to_string(),
        content: "pub fn hello() {}".to_string(),
    }).await.unwrap();
    assert!(matches!(save_resp, IpcResponse::Ok(_)));
    assert!(repo_root.join("src/hello.rs").exists());

    // 3. Execute Terminal Command via IPC
    let cmd_resp = client.send_request(IpcRequest::ExecuteTerminalCommand {
        command: "echo ipc_terminal_success".to_string(),
    }).await.unwrap();
    if let IpcResponse::Ok(val) = cmd_resp {
        assert!(val.get("stdout").unwrap().as_str().unwrap().contains("ipc_terminal_success"));
    } else {
        panic!("ExecuteTerminalCommand IPC failed: {:?}", cmd_resp);
    }

    // 4. Run Doctor via IPC
    let doc_resp = client.send_request(IpcRequest::RunDoctor).await.unwrap();
    if let IpcResponse::Ok(val) = doc_resp {
        assert!(val.get("health_score").is_some());
    } else {
        panic!("RunDoctor IPC failed: {:?}", doc_resp);
    }

    // 5. Run Scan via IPC
    let scan_resp = client.send_request(IpcRequest::RunScan).await.unwrap();
    if let IpcResponse::Ok(val) = scan_resp {
        assert!(val.get("total_files").unwrap().as_u64().unwrap() >= 3);
    } else {
        panic!("RunScan IPC failed: {:?}", scan_resp);
    }

    // 6. List Repo Files via IPC
    let list_resp = client.send_request(IpcRequest::ListRepoFiles).await.unwrap();
    if let IpcResponse::Ok(val) = list_resp {
        let files = val.get("files").unwrap().as_array().unwrap();
        assert!(files.iter().any(|f| f.as_str().unwrap().contains("main.rs")));
    } else {
        panic!("ListRepoFiles IPC failed: {:?}", list_resp);
    }
}
