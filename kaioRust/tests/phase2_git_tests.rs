use std::fs;
use std::process::Command;
use kaioken::scanner::{ChangeType, GitScanner, ScanLock};

#[test]
fn test_git_repo_root_discovery() {
    let current_dir = std::env::current_dir().unwrap();
    let root = GitScanner::find_repo_root(&current_dir);
    assert!(root.is_ok(), "Current workspace should be inside a git repository");
}

#[test]
fn test_git_head_commit_retrieval() {
    let current_dir = std::env::current_dir().unwrap();
    let root = GitScanner::find_repo_root(&current_dir).unwrap();
    let commit = GitScanner::get_head_commit(&root).unwrap();

    assert!(commit.is_some(), "Repository should have a HEAD commit");
    let c = commit.unwrap();
    assert!(!c.hash.is_empty(), "Commit hash should not be empty");
    assert!(!c.author_name.is_empty(), "Author name should not be empty");
}

#[test]
fn test_scan_lock_lifecycle_and_mutual_exclusion() {
    let temp_dir = tempfile::tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");

    // 1. First lock acquisition succeeds
    let guard1 = ScanLock::acquire(&agent_dir).unwrap();
    let lock_file = agent_dir.join("scan.lock");
    assert!(lock_file.exists(), "scan.lock file must exist while guard is held");

    // 2. Second concurrent acquisition must fail with LockError
    let guard2_res = ScanLock::acquire(&agent_dir);
    assert!(guard2_res.is_err(), "Second acquisition must fail while first is active");

    // 3. Dropping the first lock cleans up the file
    drop(guard1);
    assert!(!lock_file.exists(), "scan.lock must be removed when guard is dropped");

    // 4. Acquisition now succeeds again
    let guard3 = ScanLock::acquire(&agent_dir);
    assert!(guard3.is_ok(), "Acquisition should succeed after previous guard is dropped");
    drop(guard3);
}

#[test]
fn test_isolated_git_repo_change_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_dir = temp_dir.path();

    // Initialize clean test repository
    let init_res = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .arg("init")
        .output()
        .unwrap();
    assert!(init_res.status.success());

    // Configure local git identity for tests
    Command::new("git")
        .args(["-C", repo_dir.to_str().unwrap(), "config", "user.name", "Test Runner"])
        .output()
        .unwrap();
    Command::new("git")
        .args(["-C", repo_dir.to_str().unwrap(), "config", "user.email", "test@kaioken.dev"])
        .output()
        .unwrap();

    // Test fresh repo with 0 commits
    let head_initial = GitScanner::get_head_commit(repo_dir).unwrap();
    assert!(head_initial.is_none(), "Fresh repo should have no HEAD commit");

    // Create and commit a file
    let file1 = repo_dir.join("hello.rs");
    fs::write(&file1, "fn main() {}\n").unwrap();

    // Verify it is detected as untracked in working tree
    let working_changes = GitScanner::get_working_tree_changed_files(repo_dir).unwrap();
    assert_eq!(working_changes.len(), 1);
    assert_eq!(working_changes[0].change_type, ChangeType::Untracked);

    // Commit file1
    Command::new("git")
        .args(["-C", repo_dir.to_str().unwrap(), "add", "hello.rs"])
        .output()
        .unwrap();
    Command::new("git")
        .args(["-C", repo_dir.to_str().unwrap(), "commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    // Now HEAD commit exists and contains file1 as Added
    let commit = GitScanner::get_head_commit(repo_dir).unwrap();
    assert!(commit.is_some());
    assert_eq!(commit.unwrap().message, "Initial commit");

    let commit_changes = GitScanner::get_commit_changed_files(repo_dir).unwrap();
    assert_eq!(commit_changes.len(), 1);
    assert_eq!(commit_changes[0].change_type, ChangeType::Added);

    // Modify file1
    fs::write(&file1, "fn main() { println!(\"updated\"); }\n").unwrap();
    let modified_changes = GitScanner::get_working_tree_changed_files(repo_dir).unwrap();
    assert_eq!(modified_changes.len(), 1);
    assert_eq!(modified_changes[0].change_type, ChangeType::Modified);

    // Test diff generation
    let diff = GitScanner::get_diff(repo_dir, None).unwrap();
    assert!(diff.contains("updated"), "Diff should contain the updated modification");
}
