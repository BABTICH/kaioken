use kaioken::memory::Database;
use kaioken::safety::audit::{AuditFilter, AuditLogger};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_audit_empty_graceful() {
    let temp_dir = tempdir().unwrap();
    let logger = AuditLogger::new(Some(temp_dir.path()));

    let query_res = logger.query(&AuditFilter::default()).unwrap();
    assert_eq!(query_res.total, 0);
    assert_eq!(query_res.filtered, 0);
    assert!(query_res.events.is_empty());

    let terminal_out = query_res.render_terminal();
    assert!(terminal_out.contains("No audit events matching criteria found."));

    let stats = logger.compute_stats().unwrap();
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.tool_calls, 0);
    let stats_terminal = stats.render_terminal();
    assert!(stats_terminal.contains("Total Events:        0"));
}

#[test]
fn test_audit_show_default_and_ordering() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();
    let db_path = agent_dir.join("memory.sqlite");
    let _db = Database::open(&db_path).unwrap();

    let logger = AuditLogger::new(Some(temp_dir.path()));

    // Insert 10 events
    for i in 1..=10 {
        logger
            .log(
                "agent",
                "read_file",
                &format!("src/file_{}.rs", i),
                true,
                Some("normal read"),
            )
            .unwrap();
    }

    let res = logger.query(&AuditFilter::default()).unwrap();
    assert_eq!(res.total, 10);
    assert_eq!(res.filtered, 10);
    assert_eq!(res.events.len(), 10);

    // Most recent first (id 10 down to 1)
    assert_eq!(res.events[0].target, "src/file_10.rs");
    assert_eq!(res.events[9].target, "src/file_1.rs");

    // Test limit
    let filter_limit = AuditFilter {
        limit: 3,
        ..Default::default()
    };
    let res_limit = logger.query(&filter_limit).unwrap();
    assert_eq!(res_limit.total, 10);
    assert_eq!(res_limit.filtered, 3);
    assert_eq!(res_limit.events.len(), 3);
    assert_eq!(res_limit.events[0].target, "src/file_10.rs");
}

#[test]
fn test_audit_show_filtered_by_severity_and_tool() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();
    let db_path = agent_dir.join("memory.sqlite");
    let _db = Database::open(&db_path).unwrap();

    let logger = AuditLogger::new(Some(temp_dir.path()));

    // 1. Tool call allowed (INFO)
    logger
        .log("agent", "read_file", "src/main.rs", true, None)
        .unwrap();
    // 2. Edit file allowed (INFO)
    logger
        .log(
            "agent",
            "edit_file",
            "src/lib.rs",
            true,
            Some("added functions"),
        )
        .unwrap();
    // 3. Safety block (WARN)
    logger
        .log(
            "agent",
            "read_file",
            "../../../etc/passwd",
            false,
            Some("Path traversal blocked"),
        )
        .unwrap();
    // 4. Bash dangerous blocked (CRITICAL)
    logger
        .log(
            "agent",
            "bash",
            "rm -rf /",
            false,
            Some("Critical blocked command: rm -rf"),
        )
        .unwrap();

    // Filter by tool: edit_file
    let tool_filter = AuditFilter {
        tool_name: Some("edit_file".to_string()),
        ..Default::default()
    };
    let tool_res = logger.query(&tool_filter).unwrap();
    assert_eq!(tool_res.filtered, 1);
    assert_eq!(tool_res.events[0].action, "edit_file");

    // Filter by severity: warn
    let warn_filter = AuditFilter {
        severity: Some("warn".to_string()),
        ..Default::default()
    };
    let warn_res = logger.query(&warn_filter).unwrap();
    assert_eq!(warn_res.filtered, 2); // Both blocked events

    // Filter by severity: critical
    let crit_filter = AuditFilter {
        severity: Some("critical".to_string()),
        ..Default::default()
    };
    let crit_res = logger.query(&crit_filter).unwrap();
    assert_eq!(crit_res.filtered, 1);
    assert_eq!(crit_res.events[0].target, "rm -rf /");

    // Filter by severity: info
    let info_filter = AuditFilter {
        severity: Some("info".to_string()),
        ..Default::default()
    };
    let info_res = logger.query(&info_filter).unwrap();
    assert_eq!(info_res.filtered, 2);
}

#[test]
fn test_audit_show_since_timestamp() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();
    let db_path = agent_dir.join("memory.sqlite");
    let _db = Database::open(&db_path).unwrap();

    let logger = AuditLogger::new(Some(temp_dir.path()));

    // Insert events with distinct timestamps manually via connection
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO audit_logs (actor, action, target, approved, reason, created_at)
         VALUES ('agent', 'read_file', 'src/old.rs', 1, NULL, '2020-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO audit_logs (actor, action, target, approved, reason, created_at)
         VALUES ('agent', 'read_file', 'src/new.rs', 1, NULL, '2026-10-19T12:00:00Z')",
        [],
    )
    .unwrap();

    let since_filter = AuditFilter {
        since: Some("2026-01-01T00:00:00Z".to_string()),
        ..Default::default()
    };
    let res = logger.query(&since_filter).unwrap();
    assert_eq!(res.filtered, 1);
    assert_eq!(res.events[0].target, "src/new.rs");
}

#[test]
fn test_audit_stats_computation() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();
    let db_path = agent_dir.join("memory.sqlite");
    let _db = Database::open(&db_path).unwrap();

    let logger = AuditLogger::new(Some(temp_dir.path()));

    logger
        .log("agent", "read_file", "src/main.rs", true, None)
        .unwrap();
    logger
        .log("agent", "read_file", "src/lib.rs", true, None)
        .unwrap();
    logger
        .log("agent", "edit_file", "src/lib.rs", true, None)
        .unwrap();
    logger
        .log(
            "user",
            "approval",
            "cargo test",
            true,
            Some("user approved"),
        )
        .unwrap();
    logger
        .log(
            "agent",
            "bash",
            "dangerous",
            false,
            Some("critical blocked command"),
        )
        .unwrap();

    let stats = logger.compute_stats().unwrap();
    assert_eq!(stats.total_events, 5);
    assert_eq!(stats.safety_blocks, 1);
    assert_eq!(stats.critical_events, 1);
    assert_eq!(stats.approvals, 1);
    assert_eq!(stats.tool_calls, 4); // read_file, read_file, edit_file, bash

    // Verify top tools breakdown
    assert!(!stats.top_tools.is_empty());
    assert_eq!(stats.top_tools[0].name, "read_file");
    assert_eq!(stats.top_tools[0].count, 2);

    let rendered = stats.render_terminal();
    assert!(rendered.contains("Total Events:        5"));
    assert!(rendered.contains("read_file"));
}

#[test]
fn test_audit_disk_fallback_parsing() {
    let temp_dir = tempdir().unwrap();
    let logs_dir = temp_dir.path().join(".agent").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    let log_path = logs_dir.join("audit.log");

    // Write a mixed audit log with JSON lines and text format
    let disk_content = r#"{"id":null,"actor":"agent","action":"read_file","target":"src/main.rs","approved":true,"reason":null,"created_at":"2026-10-19T09:15:00Z"}
{"id":null,"actor":"agent","action":"edit_file","target":"src/lib.rs","approved":true,"reason":"Inserted lines","created_at":"2026-10-19T09:15:01Z"}
[2026-10-19T09:15:02Z] [WARN] safety_block: path="../../../etc/passwd" blocked by sandbox
[2026-10-19T09:15:03Z] [INFO] tool_call: bash command="cargo test" exit=0
"#;
    fs::write(&log_path, disk_content).unwrap();

    let filter = AuditFilter::default();
    let res = AuditLogger::parse_disk_log(&log_path, &filter).unwrap();

    assert_eq!(res.total, 4);
    assert_eq!(res.filtered, 4);

    // Verify ordering: most recent first
    assert_eq!(res.events[0].action, "tool_call");
    assert_eq!(res.events[1].action, "safety_block");
    assert!(!res.events[1].approved);
    assert_eq!(res.events[2].action, "edit_file");
    assert_eq!(res.events[3].action, "read_file");

    // Compute stats from parsed disk log
    let stats = AuditLogger::stats_from_entries(&res.events);
    assert_eq!(stats.total_events, 4);
    assert_eq!(stats.safety_blocks, 1);
}

#[test]
fn test_audit_secret_redaction() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();
    let db_path = agent_dir.join("memory.sqlite");
    let _db = Database::open(&db_path).unwrap();

    let logger = AuditLogger::new(Some(temp_dir.path()));

    // Log with embedded API key
    logger.log(
        "agent",
        "bash",
        "curl https://api.openrouter.ai -H 'Authorization: Bearer sk-or-v1-abcdef1234567890abcdef1234567890'",
        true,
        Some("Calling with sk-or-v1-abcdef1234567890abcdef1234567890"),
    ).unwrap();

    let res = logger.query(&AuditFilter::default()).unwrap();
    assert_eq!(res.filtered, 1);

    let entry = &res.events[0];
    assert!(!entry
        .target
        .contains("sk-or-v1-abcdef1234567890abcdef1234567890"));
    assert!(entry.target.contains("[REDACTED_API_KEY]"));
    assert!(entry
        .reason
        .as_ref()
        .unwrap()
        .contains("[REDACTED_API_KEY]"));
}

#[test]
fn test_audit_json_serialization_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join(".agent");
    fs::create_dir_all(&agent_dir).unwrap();
    let db_path = agent_dir.join("memory.sqlite");
    let _db = Database::open(&db_path).unwrap();

    let logger = AuditLogger::new(Some(temp_dir.path()));
    logger
        .log("agent", "read_file", "src/main.rs", true, None)
        .unwrap();

    let query_res = logger.query(&AuditFilter::default()).unwrap();
    let json_str = serde_json::to_string_pretty(&query_res).unwrap();
    let deserialized: kaioken::safety::audit::AuditQueryResult =
        serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.total, 1);
    assert_eq!(deserialized.filtered, 1);
    assert_eq!(deserialized.events[0].action, "read_file");

    let stats = logger.compute_stats().unwrap();
    let stats_json = serde_json::to_string_pretty(&stats).unwrap();
    let deserialized_stats: kaioken::safety::audit::AuditStats =
        serde_json::from_str(&stats_json).unwrap();
    assert_eq!(deserialized_stats.total_events, 1);
}
