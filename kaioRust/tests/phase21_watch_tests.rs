use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tempfile::tempdir;

use kaioken::cli::{Cli, Commands};
use kaioken::memory::MemoryEngine;
use kaioken::watcher::{
    compute_sha256, sync_path, DebounceManager, PathFilter, SyncOutcome, WatchEngine,
};

/// 1. test_watch_detects_change:
/// Create file -> modify -> verify memory updated with newly extracted symbols
#[test]
fn test_watch_detects_change() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let mem = MemoryEngine::open_in_memory().unwrap();

    let file_path = repo_root.join("src/calc.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "pub fn sum(a: i32, b: i32) -> i32 { a + b }").unwrap();

    // Initial sync
    let outcome1 = sync_path(repo_root, &file_path, &mem, false).unwrap();
    match outcome1 {
        SyncOutcome::NewFile {
            path,
            symbols_count,
            ..
        } => {
            assert_eq!(path, PathBuf::from("src/calc.rs"));
            assert_eq!(symbols_count, 1);
        }
        _ => panic!("Expected NewFile outcome"),
    }

    let file_rec1 = mem.get_file_by_path("src/calc.rs").unwrap().unwrap();
    let syms1 = mem.get_symbols_for_file(file_rec1.id.unwrap()).unwrap();
    assert_eq!(syms1.len(), 1);
    assert_eq!(syms1[0].name, "sum");

    // Modify file with second function
    fs::write(
        &file_path,
        "pub fn sum(a: i32, b: i32) -> i32 { a + b }\npub fn mul(a: i32, b: i32) -> i32 { a * b }",
    )
    .unwrap();

    let outcome2 = sync_path(repo_root, &file_path, &mem, false).unwrap();
    match outcome2 {
        SyncOutcome::Updated {
            path,
            symbols_count,
            changed_hash,
            ..
        } => {
            assert_eq!(path, PathBuf::from("src/calc.rs"));
            assert!(changed_hash);
            assert_eq!(symbols_count, 2);
        }
        _ => panic!("Expected Updated outcome"),
    }

    let file_rec2 = mem.get_file_by_path("src/calc.rs").unwrap().unwrap();
    let syms2 = mem.get_symbols_for_file(file_rec2.id.unwrap()).unwrap();
    assert_eq!(syms2.len(), 2);
    let names: Vec<&str> = syms2.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"sum"));
    assert!(names.contains(&"mul"));
}

/// 2. test_watch_debounce:
/// Rapid edits within debounce window -> single debounced processing event
#[test]
fn test_watch_debounce() {
    let mut dm = DebounceManager::new(80);
    let path = PathBuf::from("src/engine.rs");

    for _ in 0..5 {
        dm.record_event(path.clone());
    }

    assert_eq!(dm.pending_count(), 1);
    assert_eq!(dm.get_event_count(&path), 5);

    // Immediate poll: not ready yet
    let ready = dm.take_ready();
    assert!(ready.is_empty());

    // Sleep past debounce window
    sleep(Duration::from_millis(100));

    let ready_after = dm.take_ready();
    assert_eq!(ready_after.len(), 1);
    assert_eq!(ready_after[0], path);
    assert!(dm.is_empty());
}

/// 3. test_watch_hash_skip:
/// Touch file without changing content -> hash check skips AST re-parse
#[test]
fn test_watch_hash_skip() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let mem = MemoryEngine::open_in_memory().unwrap();

    let file_path = repo_root.join("src/constant.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "pub const PI: f64 = 3.14159;").unwrap();

    let initial = sync_path(repo_root, &file_path, &mem, false).unwrap();
    assert!(matches!(initial, SyncOutcome::NewFile { .. }));

    // Re-writing identical bytes (e.g. touch/save without changes)
    fs::write(&file_path, "pub const PI: f64 = 3.14159;").unwrap();
    let touch = sync_path(repo_root, &file_path, &mem, false).unwrap();

    match touch {
        SyncOutcome::Unchanged { path, hash } => {
            assert_eq!(path, PathBuf::from("src/constant.rs"));
            let expected_hash = compute_sha256(b"pub const PI: f64 = 3.14159;");
            assert_eq!(hash, expected_hash);
        }
        _ => panic!("Expected Unchanged outcome on identical hash"),
    }
}

/// 4. test_watch_new_file:
/// Create new .rs file -> symbols extracted and indexed
#[test]
fn test_watch_new_file() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let mem = MemoryEngine::open_in_memory().unwrap();

    let file_path = repo_root.join("src/models.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(
        &file_path,
        "pub struct UserProfile {\n    pub id: u64,\n    pub email: String,\n}\n\nimpl UserProfile {\n    pub fn is_active(&self) -> bool { true }\n}",
    )
    .unwrap();

    let outcome = sync_path(repo_root, &file_path, &mem, false).unwrap();
    match outcome {
        SyncOutcome::NewFile {
            path,
            symbols_count,
            ..
        } => {
            assert_eq!(path, PathBuf::from("src/models.rs"));
            assert!(symbols_count >= 2);
        }
        _ => panic!("Expected NewFile outcome"),
    }

    let file_rec = mem.get_file_by_path("src/models.rs").unwrap().unwrap();
    let syms = mem.get_symbols_for_file(file_rec.id.unwrap()).unwrap();
    let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"UserProfile"));
}

/// 5. test_watch_delete_file:
/// Delete file -> removed from memory database along with its symbols
#[test]
fn test_watch_delete_file() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let mem = MemoryEngine::open_in_memory().unwrap();

    let file_path = repo_root.join("src/temporary.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "pub fn temp() {}").unwrap();

    let _ = sync_path(repo_root, &file_path, &mem, false).unwrap();
    assert!(mem.get_file_by_path("src/temporary.rs").unwrap().is_some());

    // Delete file from disk
    fs::remove_file(&file_path).unwrap();

    let del_outcome = sync_path(repo_root, &file_path, &mem, false).unwrap();
    match del_outcome {
        SyncOutcome::Deleted { path } => {
            assert_eq!(path, PathBuf::from("src/temporary.rs"));
        }
        _ => panic!("Expected Deleted outcome"),
    }

    // Must be gone from SQLite memory
    assert!(mem.get_file_by_path("src/temporary.rs").unwrap().is_none());
}

/// 6. test_watch_exclude_patterns:
/// Edit file in target/ or node_modules/ -> ignored by PathFilter
#[test]
fn test_watch_exclude_patterns() {
    let filter = PathFilter::new(None, Some("custom_ignored/**"), &["vendor".to_string()]);
    let repo_root = PathBuf::from("D:/project/repo");

    // Standard ignores
    assert!(!filter.should_process(&repo_root.join("target/debug/build.rs"), &repo_root));
    assert!(!filter.should_process(&repo_root.join("node_modules/pkg/index.js"), &repo_root));
    assert!(!filter.should_process(&repo_root.join(".git/COMMIT_EDITMSG"), &repo_root));
    assert!(!filter.should_process(&repo_root.join(".agent/watch.pid"), &repo_root));

    // Config ignore
    assert!(!filter.should_process(&repo_root.join("vendor/thirdparty.rs"), &repo_root));

    // Custom exclude
    assert!(!filter.should_process(&repo_root.join("custom_ignored/test.rs"), &repo_root));

    // Temporary files
    assert!(!filter.should_process(&repo_root.join("src/main.rs.swp"), &repo_root));
    assert!(!filter.should_process(&repo_root.join("src/main.rs.tmp"), &repo_root));
    assert!(!filter.should_process(&repo_root.join("src/main.rs~"), &repo_root));

    // Valid source file
    assert!(filter.should_process(&repo_root.join("src/lib.rs"), &repo_root));
    assert!(filter.should_process(&repo_root.join("src/app.ts"), &repo_root));
    assert!(filter.should_process(&repo_root.join("scripts/run.py"), &repo_root));
}

/// 7. test_watch_burst_protection:
/// >50 files change simultaneously (e.g. git checkout) -> burst protection batches and processes immediately
#[test]
fn test_watch_burst_protection() {
    let mut dm = DebounceManager::new(10_000); // 10 second window

    for i in 0..60 {
        dm.record_event(PathBuf::from(format!("src/file_{}.rs", i)));
    }

    assert_eq!(dm.pending_count(), 60);

    // Burst threshold (>50) should allow immediate ready take
    let ready = dm.take_ready();
    assert_eq!(ready.len(), 60);
    assert!(dm.is_empty());
}

/// 8. test_watch_cli_args_parsing:
/// Verify CLI args with defaults and custom overrides
#[test]
fn test_watch_cli_args_parsing() {
    let cli = Cli::parse_from([
        "kaioken",
        "watch",
        "--debounce",
        "350",
        "--include",
        "**/*.rs,**/*.py",
        "--exclude",
        "target/**,dist/**",
        "--no-ast",
        "--verbose",
        "--pid-file",
        ".agent/custom.pid",
    ]);

    match cli.command {
        Commands::Watch(args) => {
            assert_eq!(args.debounce, 350);
            assert_eq!(args.include.as_deref(), Some("**/*.rs,**/*.py"));
            assert_eq!(args.exclude.as_deref(), Some("target/**,dist/**"));
            assert!(args.no_ast);
            assert!(args.verbose);
            assert_eq!(args.pid_file.as_deref(), Some(".agent/custom.pid"));
            assert!(!args.daemon);
        }
        _ => panic!("Expected Watch command variant"),
    }
}

/// 9. test_watch_engine_run_with_shutdown:
/// Verify WatchEngine starts, registers with notify, and terminates cleanly when shutdown triggered
#[tokio::test]
async fn test_watch_engine_run_with_shutdown() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path().to_path_buf();
    let mem = Arc::new(MemoryEngine::open_in_memory().unwrap());

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let mut engine = WatchEngine::new(repo_root.clone(), mem, 50, None, None, &[], false, false);

    // Immediately trigger shutdown
    let _ = shutdown_tx.send(());

    let res = engine.run_with_shutdown(shutdown_rx).await;
    assert!(
        res.is_ok(),
        "WatchEngine should terminate cleanly on shutdown"
    );
}
