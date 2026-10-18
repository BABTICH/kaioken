use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

use kaioken::memory::{
    CommitAnalyzer, CommitChangeRecord, CommitRecord, MemoryEngine,
};
use kaioken::scanner::{ChangeType, ChangedFile, GitCommitInfo, GitScanner, ScanLock};

#[test]
fn test_scan_lock_nonblocking_and_reclamation() {
    let temp = tempdir().unwrap();
    let agent_dir = temp.path().join(".agent");

    // Initially lock is free
    assert!(!ScanLock::is_locked(&agent_dir));
    let guard1 = ScanLock::try_acquire(&agent_dir).unwrap();
    assert!(guard1.is_some());
    assert!(ScanLock::is_locked(&agent_dir));

    // While guard1 is held, another try_acquire returns None (non-blocking)
    let guard2 = ScanLock::try_acquire(&agent_dir).unwrap();
    assert!(guard2.is_none());

    // Dropping guard1 releases the lock
    drop(guard1);
    assert!(!ScanLock::is_locked(&agent_dir));

    // Now try_acquire succeeds again
    let guard3 = ScanLock::try_acquire(&agent_dir).unwrap();
    assert!(guard3.is_some());
}

#[test]
fn test_parse_diff_hunks_changed_lines() {
    let diff_text = r#"
diff --git a/src/lib.rs b/src/lib.rs
index 123..456 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -10,3 +10,5 @@ pub mod scanner;
+pub mod memory;
+pub mod docs;
@@ -40,1 +42,3 @@ fn helper() {}
+fn new_helper() {}
diff --git a/src/auth.rs b/src/auth.rs
index abc..def 100644
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -1,5 +1,12 @@
+// Token handling
diff --git a/deleted.txt b/deleted.txt
deleted file mode 100644
--- a/deleted.txt
+++ /dev/null
@@ -1,5 +0,0 @@
-bye
"#;

    let hunks = GitScanner::parse_diff_hunks_changed_lines(diff_text);
    assert!(hunks.contains_key(&PathBuf::from("src/lib.rs")));
    assert!(hunks.contains_key(&PathBuf::from("src/auth.rs")));
    assert!(!hunks.contains_key(&PathBuf::from("deleted.txt")));

    let lib_ranges = &hunks[&PathBuf::from("src/lib.rs")];
    assert_eq!(lib_ranges.len(), 2);
    // First hunk: +10,5 -> lines 10..=14
    assert_eq!(lib_ranges[0], (10, 14));
    // Second hunk: +42,3 -> lines 42..=44
    assert_eq!(lib_ranges[1], (42, 44));

    let auth_ranges = &hunks[&PathBuf::from("src/auth.rs")];
    assert_eq!(auth_ranges.len(), 1);
    // +1,12 -> lines 1..=12
    assert_eq!(auth_ranges[0], (1, 12));
}

#[test]
fn test_sqlite_commit_crud_and_cascade() {
    let memory = MemoryEngine::open_in_memory().unwrap();

    let commit = CommitRecord {
        id: None,
        hash: "c0ffee1234567890abcdef".to_string(),
        message: "feat: add memory engine".to_string(),
        author: Some("Architect <architect@kaioken.dev>".to_string()),
        created_at: Some("2026-10-08T20:00:00Z".to_string()),
        summary: Some("Added memory engine with WAL SQLite".to_string()),
        risk_level: Some("medium".to_string()),
    };

    let changes = vec![
        CommitChangeRecord {
            id: None,
            commit_id: 0,
            file_id: None,
            symbol_id: None,
            change_type: "added".to_string(),
        },
        CommitChangeRecord {
            id: None,
            commit_id: 0,
            file_id: None,
            symbol_id: None,
            change_type: "modified".to_string(),
        },
    ];

    let commit_id = memory.record_commit(&commit, &changes).unwrap();
    assert!(commit_id > 0);

    // Verify commit count
    assert_eq!(memory.count_commits().unwrap(), 1);
    assert_eq!(memory.count_commit_changes().unwrap(), 2);

    // Query by hash
    let fetched = memory.get_commit_by_hash("c0ffee1234567890abcdef").unwrap();
    assert!(fetched.is_some());
    let fetched_commit = fetched.unwrap();
    assert_eq!(fetched_commit.id, Some(commit_id));
    assert_eq!(fetched_commit.message, "feat: add memory engine");
    assert_eq!(fetched_commit.risk_level.as_deref(), Some("medium"));

    // Query by ID
    let fetched_by_id = memory.get_commit_by_id(commit_id).unwrap();
    assert!(fetched_by_id.is_some());

    // Query changes
    let fetched_changes = memory.get_changes_for_commit(commit_id).unwrap();
    assert_eq!(fetched_changes.len(), 2);
    assert_eq!(fetched_changes[0].change_type, "added");
    assert_eq!(fetched_changes[1].change_type, "modified");

    // Test Cascade Deletion: deleting from commits should delete commit_changes
    let conn = memory.db().connection();
    conn.execute("DELETE FROM commits WHERE id = ?1;", rusqlite::params![commit_id]).unwrap();

    assert_eq!(memory.count_commits().unwrap(), 0);
    assert_eq!(memory.count_commit_changes().unwrap(), 0);
}

#[test]
fn test_commit_risk_assessment_rules() {
    // 1. Critical config (Cargo.toml) -> high
    let changed_high = vec![ChangedFile {
        path: PathBuf::from("Cargo.toml"),
        change_type: ChangeType::Modified,
        old_path: None,
    }];
    assert_eq!(CommitAnalyzer::calculate_risk_level(&changed_high, 0), "high");

    // 2. Security/Auth path -> high
    let changed_auth = vec![ChangedFile {
        path: PathBuf::from("src/auth/session.rs"),
        change_type: ChangeType::Modified,
        old_path: None,
    }];
    assert_eq!(CommitAnalyzer::calculate_risk_level(&changed_auth, 1), "high");

    // 3. Deletion -> high
    let changed_del = vec![ChangedFile {
        path: PathBuf::from("src/temp.rs"),
        change_type: ChangeType::Deleted,
        old_path: None,
    }];
    assert_eq!(CommitAnalyzer::calculate_risk_level(&changed_del, 0), "high");

    // 4. >2 files or >3 symbols touched -> medium
    let changed_med = vec![
        ChangedFile {
            path: PathBuf::from("src/file1.rs"),
            change_type: ChangeType::Modified,
            old_path: None,
        },
        ChangedFile {
            path: PathBuf::from("src/file2.rs"),
            change_type: ChangeType::Modified,
            old_path: None,
        },
        ChangedFile {
            path: PathBuf::from("src/file3.rs"),
            change_type: ChangeType::Modified,
            old_path: None,
        },
    ];
    assert_eq!(CommitAnalyzer::calculate_risk_level(&changed_med, 1), "medium");

    // 5. Small doc or single file edit -> low
    let changed_low = vec![ChangedFile {
        path: PathBuf::from("README.md"),
        change_type: ChangeType::Modified,
        old_path: None,
    }];
    assert_eq!(CommitAnalyzer::calculate_risk_level(&changed_low, 0), "low");
}

#[test]
fn test_deterministic_markdown_summary_generation() {
    let commit_info = GitCommitInfo {
        hash: "abcdef1234567890".to_string(),
        author_name: "Developer".to_string(),
        author_email: "dev@example.com".to_string(),
        timestamp: "2026-10-08T12:00:00Z".to_string(),
        message: "fix: resolve session bug".to_string(),
    };

    let changed_files = vec![
        ChangedFile {
            path: PathBuf::from("src/session.rs"),
            change_type: ChangeType::Modified,
            old_path: None,
        },
    ];

    let touched_symbols = vec![
        kaioken::memory::TouchedSymbolInfo {
            symbol_id: Some(1),
            symbol_name: "validate_token".to_string(),
            symbol_kind: "function".to_string(),
            file_path: "src/session.rs".to_string(),
            line_start: 20,
            line_end: 35,
            change_type: "modified".to_string(),
        },
    ];

    let summary = CommitAnalyzer::generate_markdown_summary(
        &commit_info,
        &changed_files,
        &touched_symbols,
        "medium",
    );

    assert!(summary.contains("Commit abcdef12: fix: resolve session bug"));
    assert!(summary.contains("Developer <dev@example.com>"));
    assert!(summary.contains("Risk Level:** medium"));
    assert!(summary.contains("`src/session.rs` (modified)"));
    assert!(summary.contains("`validate_token` (function in `src/session.rs`:L20-L35, modified)"));
    assert!(summary.contains("Recommended Verification"));
}

#[test]
fn test_end_to_end_git_commit_analysis_and_touched_symbols() {
    let temp = tempdir().unwrap();
    let repo_dir = temp.path();

    // 1. Initialize git repo
    let run = |args: &[&str]| {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .args(args)
            .status()
            .expect("Failed to execute git command");
        assert!(status.success(), "Git command failed: {:?}", args);
    };

    run(&["init"]);
    run(&["config", "user.name", "Test Agent"]);
    run(&["config", "user.email", "agent@kaioken.dev"]);

    // 2. Create Rust source file with two distinct functions
    let src_dir = repo_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let math_file = src_dir.join("math.rs");

    let v1_code = r#"// Line 1
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#;
    fs::write(&math_file, v1_code).unwrap();

    run(&["add", "."]);
    run(&["commit", "-m", "Initial math functions"]);

    let head_commit = GitScanner::get_head_commit(repo_dir).unwrap().expect("HEAD commit missing");
    let memory = MemoryEngine::open_in_memory().unwrap();

    // 3. Analyze initial commit: both functions should be added
    let analysis1 = CommitAnalyzer::analyze(repo_dir, &head_commit, &memory).unwrap();
    assert_eq!(analysis1.changed_files_count, 1);
    assert_eq!(analysis1.affected_symbols_count, 2);
    let sym_names: Vec<&str> = analysis1.touched_symbols.iter().map(|s| s.symbol_name.as_str()).collect();
    assert!(sym_names.contains(&"add"));
    assert!(sym_names.contains(&"multiply"));

    let commit_id1 = memory.record_commit_analysis(&analysis1).unwrap();
    assert_eq!(commit_id1, 1);

    // 4. Modify ONLY the `add` function on lines 2..4
    let v2_code = r#"// Line 1
pub fn add(a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#;
    fs::write(&math_file, v2_code).unwrap();

    run(&["add", "."]);
    run(&["commit", "-m", "Refactor add implementation"]);

    let commit2 = GitScanner::get_head_commit(repo_dir).unwrap().expect("Commit 2 missing");
    let analysis2 = CommitAnalyzer::analyze(repo_dir, &commit2, &memory).unwrap();

    assert_eq!(analysis2.changed_files_count, 1);
    // Crucial check: only `add` should be touched, NOT `multiply`!
    assert_eq!(analysis2.affected_symbols_count, 1);
    assert_eq!(analysis2.touched_symbols[0].symbol_name, "add");
    assert_eq!(analysis2.touched_symbols[0].change_type, "modified");

    let commit_id2 = memory.record_commit_analysis(&analysis2).unwrap();
    assert_eq!(commit_id2, 2);

    // 5. Verify database records
    assert_eq!(memory.count_commits().unwrap(), 2);
    let recent = memory.get_recent_commits(2).unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].message, "Refactor add implementation");
    assert_eq!(recent[1].message, "Initial math functions");

    // Verify files table has updated last_commit
    let file_rec = memory.get_file_by_path("src/math.rs").unwrap().expect("File missing");
    assert_eq!(file_rec.last_commit.as_deref(), Some(commit2.hash.as_str()));
}
