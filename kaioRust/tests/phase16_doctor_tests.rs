use kaioken::config::Config;
use kaioken::doctor::{checks, CheckStatus, DiagnosticReport, DoctorEngine, OverallStatus};
use kaioken::memory::Database;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_doctor_workspace_run() {
    let current_dir = std::env::current_dir().unwrap();
    let report = DoctorEngine::run(&current_dir, None).expect("Doctor check should execute");

    assert_eq!(
        report.checks.len(),
        13,
        "Expected exactly 13 diagnostic checks"
    );
    assert_eq!(
        report.summary.fail, 0,
        "Current workspace should have 0 failures"
    );
    assert!(
        matches!(
            report.overall,
            OverallStatus::Ready | OverallStatus::ReadyLimited
        ),
        "Overall status should be Ready or ReadyLimited"
    );

    let terminal_output = report.render_terminal();
    assert!(terminal_output.contains("🔍 kaioken doctor — Environment Diagnostic Report"));
    assert!(terminal_output.contains("Git Availability"));
    assert!(terminal_output.contains("SQLite Runtime"));
    assert!(terminal_output.contains("Tree-sitter Rust"));
    assert!(terminal_output.contains("Disk Space"));
}

#[test]
fn test_doctor_non_git_directory() {
    let temp_dir = tempdir().unwrap();
    let report = DoctorEngine::run(temp_dir.path(), Some("git_repository")).unwrap();

    assert_eq!(report.checks.len(), 1);
    let git_check = &report.checks[0];
    assert_eq!(git_check.name, "git_repository");
    assert_eq!(git_check.status, CheckStatus::Fail);
    assert!(git_check.detail.contains("Not inside a Git working tree"));
    assert_eq!(report.overall, OverallStatus::NotReady);
}

#[test]
fn test_doctor_no_config_warns() {
    let temp_dir = tempdir().unwrap();
    let report = DoctorEngine::run(temp_dir.path(), Some("config_file")).unwrap();

    assert_eq!(report.checks.len(), 1);
    let cfg_check = &report.checks[0];
    assert_eq!(cfg_check.name, "config_file");
    assert_eq!(cfg_check.status, CheckStatus::Warn);
    assert!(cfg_check.detail.contains(".agent/config.toml not found"));
}

#[test]
fn test_doctor_with_valid_config() {
    let temp_dir = tempdir().unwrap();
    let config = Config::default_config();
    config.save(temp_dir.path()).unwrap();

    let report = DoctorEngine::run(temp_dir.path(), Some("config_file")).unwrap();
    assert_eq!(report.checks.len(), 1);
    assert_eq!(report.checks[0].status, CheckStatus::Pass);
    assert!(report.checks[0].detail.contains(".agent/config.toml valid"));
}

#[test]
fn test_doctor_stale_lock_detection() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();

    let lock_file = agent_dir.join("scan.lock");
    fs::write(&lock_file, "pid=99999\ntimestamp=2020-01-01T00:00:00Z\n").unwrap();

    // Set file modification time to 10 minutes ago
    let ten_mins_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(600);
    let f = fs::OpenOptions::new().write(true).open(&lock_file).unwrap();
    f.set_modified(ten_mins_ago).unwrap();
    drop(f);

    let report = DoctorEngine::run(temp_dir.path(), Some("lockfile_safety")).unwrap();
    assert_eq!(report.checks.len(), 1);
    let lock_check = &report.checks[0];
    assert_eq!(lock_check.status, CheckStatus::Warn);
    assert!(lock_check.detail.contains("Stale scan lock detected"));
}

#[test]
fn test_doctor_filtering_only() {
    let current_dir = std::env::current_dir().unwrap();
    let report = DoctorEngine::run(&current_dir, Some("git_availability,rust_toolchain")).unwrap();

    assert_eq!(report.checks.len(), 2);
    let names: Vec<String> = report.checks.into_iter().map(|c| c.name).collect();
    assert_eq!(names, vec!["git_availability", "rust_toolchain"]);
}

#[test]
fn test_doctor_incomplete_schema() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();

    let db_path = agent_dir.join("memory.sqlite");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    // Create an incomplete database with only one table
    conn.execute("CREATE TABLE files (id INTEGER PRIMARY KEY);", [])
        .unwrap();

    let report = DoctorEngine::run(temp_dir.path(), Some("sqlite_schema")).unwrap();
    assert_eq!(report.checks.len(), 1);
    assert_eq!(report.checks[0].status, CheckStatus::Fail);
    assert!(report.checks[0].detail.contains("Missing tables"));
    assert_eq!(report.overall, OverallStatus::NotReady);
}

#[test]
fn test_doctor_complete_schema() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();

    let db_path = agent_dir.join("memory.sqlite");
    let _db = Database::open(&db_path).unwrap();

    let report = DoctorEngine::run(temp_dir.path(), Some("sqlite_schema")).unwrap();
    assert_eq!(report.checks.len(), 1);
    assert_eq!(report.checks[0].status, CheckStatus::Pass);
    assert!(report.checks[0].detail.contains("13/13 tables present"));
}

#[test]
fn test_doctor_json_serialization_roundtrip() {
    let current_dir = std::env::current_dir().unwrap();
    let report = DoctorEngine::run(&current_dir, Some("treesitter_rust,treesitter_ts_js")).unwrap();

    let json_str = serde_json::to_string_pretty(&report).unwrap();
    let deserialized: DiagnosticReport = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.checks.len(), 2);
    assert_eq!(deserialized.summary.pass, report.summary.pass);
    assert_eq!(deserialized.summary.warn, report.summary.warn);
    assert_eq!(deserialized.summary.fail, report.summary.fail);
    assert_eq!(deserialized.overall, report.overall);
}

#[test]
fn test_individual_check_functions() {
    // Tree-sitter Rust
    let ts_rust = checks::check_treesitter_rust();
    assert_eq!(ts_rust.status, CheckStatus::Pass);

    // Tree-sitter TS/JS
    let ts_js = checks::check_treesitter_ts_js();
    assert_eq!(ts_js.status, CheckStatus::Warn);

    // Tree-sitter Python
    let ts_py = checks::check_treesitter_python();
    assert_eq!(ts_py.status, CheckStatus::Warn);
}
