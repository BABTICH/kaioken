use kaioken::hook::{HookManager, InstallOutcome, UninstallOutcome, SENTINEL_END, SENTINEL_START};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_hook_install_fresh() {
    let temp_dir = tempdir().unwrap();
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    let outcome = HookManager::install(temp_dir.path(), false, "/bin/sh").unwrap();
    let hook_file = hooks_dir.join("post-commit");

    assert_eq!(outcome, InstallOutcome::InstalledFresh(hook_file.clone()));
    assert!(hook_file.exists(), "Hook file should exist after install");

    let content = fs::read_to_string(&hook_file).unwrap();
    assert!(content.starts_with("#!/bin/sh"));
    assert!(content.contains(SENTINEL_START));
    assert!(content.contains(SENTINEL_END));
    assert!(content.contains("kaioken memory scan --changed --no-llm >/dev/null 2>&1 &"));
    assert!(content.contains("disown 2>/dev/null || true"));
}

#[test]
fn test_hook_install_custom_shell() {
    let temp_dir = tempdir().unwrap();
    let outcome = HookManager::install(temp_dir.path(), false, "/usr/bin/env bash").unwrap();
    let hook_file = temp_dir
        .path()
        .join(".git")
        .join("hooks")
        .join("post-commit");

    assert_eq!(outcome, InstallOutcome::InstalledFresh(hook_file.clone()));
    let content = fs::read_to_string(&hook_file).unwrap();
    assert!(content.starts_with("#!/usr/bin/env bash"));
}

#[test]
fn test_hook_install_existing_preserves_content() {
    let temp_dir = tempdir().unwrap();
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_file = hooks_dir.join("post-commit");

    let pre_existing = "#!/bin/sh\necho 'running linting'\nnpm test\n";
    fs::write(&hook_file, pre_existing).unwrap();

    let outcome = HookManager::install(temp_dir.path(), false, "/bin/sh").unwrap();
    assert_eq!(outcome, InstallOutcome::Appended(hook_file.clone()));

    let content = fs::read_to_string(&hook_file).unwrap();
    assert!(content.contains("running linting"));
    assert!(content.contains("npm test"));
    assert!(content.contains(SENTINEL_START));
    assert!(content.contains(SENTINEL_END));
}

#[test]
fn test_hook_install_idempotent() {
    let temp_dir = tempdir().unwrap();

    let outcome1 = HookManager::install(temp_dir.path(), false, "/bin/sh").unwrap();
    let hook_file = temp_dir
        .path()
        .join(".git")
        .join("hooks")
        .join("post-commit");
    assert_eq!(outcome1, InstallOutcome::InstalledFresh(hook_file.clone()));

    let outcome2 = HookManager::install(temp_dir.path(), false, "/bin/sh").unwrap();
    assert_eq!(
        outcome2,
        InstallOutcome::AlreadyInstalled(hook_file.clone())
    );

    // Verify sentinel appears only once
    let content = fs::read_to_string(&hook_file).unwrap();
    assert_eq!(content.matches(SENTINEL_START).count(), 1);
    assert_eq!(content.matches(SENTINEL_END).count(), 1);
}

#[test]
fn test_hook_install_force_replaces_block() {
    let temp_dir = tempdir().unwrap();
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_file = hooks_dir.join("post-commit");

    let old_content = format!(
        "#!/bin/sh\necho 'custom pre-hook'\n{}\n# OLD KAIOKEN BLOCK\n{}\necho 'custom post-hook'\n",
        SENTINEL_START, SENTINEL_END
    );
    fs::write(&hook_file, old_content).unwrap();

    let outcome = HookManager::install(temp_dir.path(), true, "/bin/sh").unwrap();
    assert_eq!(outcome, InstallOutcome::Replaced(hook_file.clone()));

    let content = fs::read_to_string(&hook_file).unwrap();
    assert!(content.contains("custom pre-hook"));
    assert!(content.contains("custom post-hook"));
    assert!(!content.contains("# OLD KAIOKEN BLOCK"));
    assert!(content.contains("kaioken memory scan --changed --no-llm"));
    assert_eq!(content.matches(SENTINEL_START).count(), 1);
}

#[test]
fn test_hook_uninstall_clean_deletes_file() {
    let temp_dir = tempdir().unwrap();
    HookManager::install(temp_dir.path(), false, "/bin/sh").unwrap();
    let hook_file = temp_dir
        .path()
        .join(".git")
        .join("hooks")
        .join("post-commit");
    assert!(hook_file.exists());

    let outcome = HookManager::uninstall(temp_dir.path()).unwrap();
    assert_eq!(
        outcome,
        UninstallOutcome::RemovedAndDeletedFile(hook_file.clone())
    );
    assert!(
        !hook_file.exists(),
        "Hook file should be deleted when only Kaioken content was present"
    );
}

#[test]
fn test_hook_uninstall_preserves_other_hooks() {
    let temp_dir = tempdir().unwrap();
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_file = hooks_dir.join("post-commit");

    let content_with_custom = format!(
        "#!/bin/sh\n# Pre-existing custom hook\n./scripts/deploy_notify.sh\n\n{}\n# Kaioken content\n{}\n",
        SENTINEL_START, SENTINEL_END
    );
    fs::write(&hook_file, content_with_custom).unwrap();

    let outcome = HookManager::uninstall(temp_dir.path()).unwrap();
    assert_eq!(outcome, UninstallOutcome::RemovedBlock(hook_file.clone()));
    assert!(
        hook_file.exists(),
        "Hook file should NOT be deleted when other hooks exist"
    );

    let remaining = fs::read_to_string(&hook_file).unwrap();
    assert!(remaining.contains("./scripts/deploy_notify.sh"));
    assert!(!remaining.contains(SENTINEL_START));
}

#[test]
fn test_hook_uninstall_when_not_installed() {
    let temp_dir = tempdir().unwrap();
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_file = hooks_dir.join("post-commit");
    fs::write(&hook_file, "#!/bin/sh\necho 'hello'\n").unwrap();

    let outcome = HookManager::uninstall(temp_dir.path()).unwrap();
    assert_eq!(outcome, UninstallOutcome::NotInstalled(hook_file.clone()));
    assert!(hook_file.exists());
}

#[test]
fn test_hook_status_reporting() {
    let temp_dir = tempdir().unwrap();

    // 1. Status before install
    let status_pre = HookManager::status(temp_dir.path()).unwrap();
    assert!(!status_pre.exists);
    assert!(!status_pre.is_installed);
    let pre_terminal = status_pre.render_terminal();
    assert!(pre_terminal.contains("Not installed"));

    // 2. Status after install
    HookManager::install(temp_dir.path(), false, "/bin/sh").unwrap();
    let status_post = HookManager::status(temp_dir.path()).unwrap();
    assert!(status_post.exists);
    assert!(status_post.is_installed);
    let post_terminal = status_post.render_terminal();
    assert!(post_terminal.contains("✅ Installed"));
}
