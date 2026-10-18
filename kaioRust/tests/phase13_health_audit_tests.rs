use std::fs;
use tempfile::TempDir;

use kaioken::cli::schedule::{handle_schedule_command, ScheduleArgs};
use kaioken::config::Config;
use kaioken::health::HealthAuditEngine;
use kaioken::memory::{DocRecord, EdgeRecord, FailureRecord, FileRecord, MemoryEngine, SymbolRecord};

#[test]
fn test_health_audit_computes_score_and_overview() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let memory = MemoryEngine::open_in_memory().unwrap();

    let f1 = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/auth.rs".to_string(),
        language: Some("rust".to_string()),
        size: 120,
        source_hash: Some("h1".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: Some("Authentication module".to_string()),
        risk_level: None,
        doc_path: Some(".agent-docs/src/auth.md".to_string()),
        doc_status: Some("synced".to_string()),
    }).unwrap();

    let f2 = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        size: 50,
        source_hash: Some("h2".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: Some("Library entrypoint".to_string()),
        risk_level: None,
        doc_path: Some(".agent-docs/src/lib.md".to_string()),
        doc_status: Some("synced".to_string()),
    }).unwrap();

    let f_test = memory.upsert_file(&FileRecord {
        id: None,
        path: "tests/auth_tests.rs".to_string(),
        language: Some("rust".to_string()),
        size: 80,
        source_hash: Some("h3".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    // Insert symbols
    memory.insert_symbols(f1, &[SymbolRecord {
        id: None,
        file_id: f1,
        name: "login".to_string(),
        kind: "function".to_string(),
        line_start: 10,
        line_end: 20,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn login()".to_string()),
        summary: None,
        parent: None,
    }]).unwrap();

    memory.insert_symbols(f2, &[SymbolRecord {
        id: None,
        file_id: f2,
        name: "init".to_string(),
        kind: "function".to_string(),
        line_start: 1,
        line_end: 5,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn init()".to_string()),
        summary: None,
        parent: None,
    }]).unwrap();

    memory.insert_symbols(f_test, &[SymbolRecord {
        id: None,
        file_id: f_test,
        name: "test_login".to_string(),
        kind: "function".to_string(),
        line_start: 1,
        line_end: 5,
        visibility: None,
        signature: Some("fn test_login()".to_string()),
        summary: None,
        parent: None,
    }]).unwrap();

    // Query symbol IDs
    let all_symbols = memory.get_all_symbols().unwrap();
    let s1_id = all_symbols.iter().find(|s| s.name == "login").unwrap().id.unwrap();
    let stest_id = all_symbols.iter().find(|s| s.name == "test_login").unwrap().id.unwrap();

    // Test calls login
    memory.insert_edge(&EdgeRecord {
        id: None,
        source_symbol_id: Some(stest_id),
        target_symbol_id: Some(s1_id),
        edge_type: "calls".to_string(),
    }).unwrap();

    // Insert docs
    memory.upsert_doc(&DocRecord {
        id: None,
        file_id: f1,
        doc_path: ".agent-docs/src/auth.md".to_string(),
        source_hash: Some("h1".to_string()),
        doc_level: 0,
        status: "synced".to_string(),
        updated_at: "2026-10-18T00:00:00Z".to_string(),
        stale_reason: None,
    }).unwrap();

    memory.upsert_doc(&DocRecord {
        id: None,
        file_id: f2,
        doc_path: ".agent-docs/src/lib.md".to_string(),
        source_hash: Some("h2".to_string()),
        doc_level: 0,
        status: "synced".to_string(),
        updated_at: "2026-10-18T00:00:00Z".to_string(),
        stale_reason: None,
    }).unwrap();

    let docs_dir = root.join(".agent-docs");
    let report = HealthAuditEngine::audit(root, &memory, &docs_dir).unwrap();

    assert_eq!(report.overview.total_files, 3);
    assert_eq!(report.overview.total_symbols, 3);
    assert_eq!(report.overview.total_edges, 1);
    assert_eq!(report.overview.total_docs, 2);
    assert!(report.score.score >= 80, "Score should be high for clean repository");
    assert_eq!(report.doc_health.fresh_count, 2);
    assert_eq!(report.doc_health.stale_count, 0);
}

#[test]
fn test_health_audit_detects_stale_and_missing_docs() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let memory = MemoryEngine::open_in_memory().unwrap();

    // File with missing doc
    let _f1 = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/billing.rs".to_string(),
        language: Some("rust".to_string()),
        size: 100,
        source_hash: Some("h1".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    // File with stale doc
    let f2 = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/orders.rs".to_string(),
        language: Some("rust".to_string()),
        size: 150,
        source_hash: Some("h2".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: Some(".agent-docs/src/orders.md".to_string()),
        doc_status: Some("stale".to_string()),
    }).unwrap();

    memory.upsert_doc(&DocRecord {
        id: None,
        file_id: f2,
        doc_path: ".agent-docs/src/orders.md".to_string(),
        source_hash: Some("old_h2".to_string()),
        doc_level: 0,
        status: "stale".to_string(),
        updated_at: "2026-10-18T00:00:00Z".to_string(),
        stale_reason: Some("Code modified".to_string()),
    }).unwrap();

    let docs_dir = root.join(".agent-docs");
    let report = HealthAuditEngine::audit(root, &memory, &docs_dir).unwrap();

    assert_eq!(report.doc_health.stale_count, 1);
    assert!(report.doc_health.stale_doc_paths.contains(&".agent-docs/src/orders.md".to_string()));
    assert!(report.doc_health.missing_doc_files.contains(&"src/billing.rs".to_string()));

    // Verify deductions
    assert!(report.score.deductions.iter().any(|d| d.reason.contains("stale documentation")));
    assert!(report.score.deductions.iter().any(|d| d.reason.contains("lack mirrored documentation")));
    assert!(report.recommendations.iter().any(|r| r.contains("kaioken docs generate --stale")));
}

#[test]
fn test_health_audit_detects_dead_code_candidates() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let memory = MemoryEngine::open_in_memory().unwrap();

    let f = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/utils.rs".to_string(),
        language: Some("rust".to_string()),
        size: 100,
        source_hash: Some("h1".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    // Internal private function with 0 incoming calls
    memory.insert_symbols(f, &[SymbolRecord {
        id: None,
        file_id: f,
        name: "unused_private_helper".to_string(),
        kind: "function".to_string(),
        line_start: 15,
        line_end: 25,
        visibility: None, // private
        signature: Some("fn unused_private_helper()".to_string()),
        summary: None,
        parent: None,
    }]).unwrap();

    let docs_dir = root.join(".agent-docs");
    let report = HealthAuditEngine::audit(root, &memory, &docs_dir).unwrap();

    assert_eq!(report.dead_code.unreferenced_symbols.len(), 1);
    assert_eq!(report.dead_code.unreferenced_symbols[0].name, "unused_private_helper");
    assert!(report.score.deductions.iter().any(|d| d.reason.contains("unreferenced internal symbol")));
}

#[test]
fn test_health_audit_detects_untested_files_and_unresolved_imports() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let memory = MemoryEngine::open_in_memory().unwrap();

    let f = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/payment_gateway.rs".to_string(),
        language: Some("rust".to_string()),
        size: 100,
        source_hash: Some("h1".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    // Add unresolved import
    memory.insert_raw_imports(f, &["use external_unresolved::Gateway;".to_string()]).unwrap();

    let docs_dir = root.join(".agent-docs");
    let report = HealthAuditEngine::audit(root, &memory, &docs_dir).unwrap();

    assert!(report.test_coverage.untested_files.contains(&"src/payment_gateway.rs".to_string()));
    assert_eq!(report.graph_integrity.unresolved_imports.len(), 1);
    assert_eq!(report.graph_integrity.unresolved_imports[0].0, "src/payment_gateway.rs");
}

#[test]
fn test_health_audit_markdown_and_json_rendering() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let memory = MemoryEngine::open_in_memory().unwrap();

    let f = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/calc.rs".to_string(),
        language: Some("rust".to_string()),
        size: 50,
        source_hash: Some("h1".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    memory.insert_failure(&FailureRecord {
        id: None,
        error_type: "compiler".to_string(),
        message: "E0308 type mismatch".to_string(),
        file_id: Some(f),
        symbol_id: None,
        fix_summary: None,
        status: Some("unresolved".to_string()),
        resolved_at: None,
        created_at: "2026-10-18T10:00:00Z".to_string(),
    }).unwrap();

    let docs_dir = root.join(".agent-docs");
    let report = HealthAuditEngine::audit(root, &memory, &docs_dir).unwrap();

    // 1. Markdown rendering
    let md = report.to_markdown();
    assert!(md.contains("# 🏥 KAIOKEN Repository Health Audit Report"));
    assert!(md.contains("Overall Health Score"));
    assert!(md.contains("Repository Inventory Overview"));
    assert!(md.contains("Documentation Mirror Health"));
    assert!(md.contains("Dead Code Detection"));
    assert!(md.contains("Test Coverage Hints"));
    assert!(md.contains("Active Unresolved Failures"));
    assert!(md.contains("Actionable Recommendations"));

    // 2. JSON roundtrip
    let json_str = report.to_json().unwrap();
    assert!(json_str.contains("\"score\":"));
    assert!(json_str.contains("\"unresolved_count\": 1"));
}

#[tokio::test]
async fn test_cli_schedule_daily_outputs_report_file() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create config
    let config = Config::default_config();
    let cfg_file = root.join(".agent/config.toml");
    fs::create_dir_all(cfg_file.parent().unwrap()).unwrap();
    fs::write(&cfg_file, toml::to_string(&config).unwrap()).unwrap();

    // Initialize sqlite
    let db_path = root.join(&config.memory.database);
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let memory = MemoryEngine::open(&db_path).unwrap();

    memory.upsert_file(&FileRecord {
        id: None,
        path: "src/main.rs".to_string(),
        language: Some("rust".to_string()),
        size: 50,
        source_hash: Some("main_hash".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: Some("Entrypoint".to_string()),
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    let custom_output = root.join("reports/daily_audit.md");

    // Execute schedule daily via CLI handler
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(root).unwrap();

    let args = ScheduleArgs {
        daily: true,
        output: Some(custom_output.display().to_string()),
        json: false,
        fix_stale: false,
    };

    let res = handle_schedule_command(args).await;
    std::env::set_current_dir(current_dir).unwrap();

    assert!(res.is_ok(), "Schedule command should succeed");
    assert!(custom_output.exists(), "Report markdown file must be written to disk");

    let report_content = fs::read_to_string(&custom_output).unwrap();
    assert!(report_content.contains("# 🏥 KAIOKEN Repository Health Audit Report"));
    assert!(report_content.contains("Overall Health Score"));
}
