use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::TempDir;

use kaioken::config::Config;
use kaioken::memory::{
    DocRecord, EdgeRecord, FailureRecord, FileRecord, MemoryEngine, SymbolRecord,
};
use kaioken::retrieval::{
    BudgetConfig, BudgetManager, ContextPacket, DocLookup, FailureLookup,
    FileContextSnippet, GraphQueryEngine, RetrievalCacheStore, RetrievalEngine,
    RetrievalOptions, SymbolContextSummary,
};
use kaioken::safety::RedactionEngine;

#[test]
fn test_context_packet_assembly_and_serialization() {
    let mut packet = ContextPacket::empty("Fix authentication token validation", 8000);

    packet.relevant_files.push(FileContextSnippet {
        path: "src/auth/session.rs".to_string(),
        language: "rust".to_string(),
        line_start: 10,
        line_end: 50,
        content: "pub fn validate_session(token: &str) -> bool { !token.is_empty() }".to_string(),
        source_hash: "abc123hash".to_string(),
        token_estimate: 25,
    });

    packet.relevant_symbols.push(SymbolContextSummary {
        name: "validate_session".to_string(),
        kind: "function".to_string(),
        file_path: "src/auth/session.rs".to_string(),
        signature: "pub fn validate_session(token: &str) -> bool".to_string(),
        visibility: "public".to_string(),
        line_start: 10,
        line_end: 12,
        token_estimate: 15,
    });

    packet.dependency_graph_subset.direct_callers.push("src/api/auth.rs::login_handler (fn)".to_string());
    packet.dependency_graph_subset.affected_tests.push("tests/auth_tests.rs".to_string());

    packet.recommended_verification_commands.push("cargo test --test auth_tests".to_string());

    // JSON serialization test
    let json = packet.to_json().expect("Serialization to JSON should succeed");
    assert!(json.contains("validate_session"));
    assert!(json.contains("auth_tests.rs"));

    // Deserialization roundtrip
    let deserialized: ContextPacket = serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(deserialized.task_summary, packet.task_summary);
    assert_eq!(deserialized.relevant_files.len(), 1);

    // Markdown rendering test
    let markdown = packet.to_markdown();
    assert!(markdown.contains("# Repository Context Packet"));
    assert!(markdown.contains("Fix authentication token validation"));
    assert!(markdown.contains("validate_session"));
    assert!(markdown.contains("src/auth/session.rs"));
    assert!(markdown.contains("cargo test --test auth_tests"));
}

#[test]
fn test_token_budget_priority_truncation() {
    let mut packet = ContextPacket::empty("Large task", 300);

    // File with 2000 characters (~500 tokens)
    packet.relevant_files.push(FileContextSnippet {
        path: "src/big.rs".to_string(),
        language: "rust".to_string(),
        line_start: 1,
        line_end: 100,
        content: "fn test() {\n".to_string() + &"    println!(\"hello world\");\n".repeat(50) + "}",
        source_hash: "hash1".to_string(),
        token_estimate: 500,
    });

    // Symbols
    for i in 0..10 {
        packet.relevant_symbols.push(SymbolContextSummary {
            name: format!("func_{}", i),
            kind: "function".to_string(),
            file_path: "src/big.rs".to_string(),
            signature: format!("pub fn func_{}() -> i32", i),
            visibility: "public".to_string(),
            line_start: i * 5,
            line_end: i * 5 + 3,
            token_estimate: 20,
        });
    }

    let config = BudgetConfig {
        max_files_tokens: 150,
        max_symbols_tokens: 100,
        max_graph_tokens: 50,
        max_failures_tokens: 50,
        max_docs_adrs_tokens: 50,
        total_budget_limit: 300,
    };

    let budget_manager = BudgetManager::new(config);
    budget_manager.enforce_budget(&mut packet);

    assert!(packet.token_budget_report.truncated);
    assert!(packet.token_budget_report.total_tokens <= 300);
    assert!(packet.relevant_files[0].content.contains("TRUNCATED"));
}

#[test]
fn test_secret_redaction_api_keys_and_tokens() {
    let engine = RedactionEngine::new();

    let raw = r#"
        let openai_key = "sk-1234567890abcdef1234567890abcdef";
        let github_token = "ghp_123456789012345678901234567890123456";
        let aws_key = "AKIA1234567890ABCDEF";
        let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.secretpayload";
        let priv_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
    "#;

    let redacted = engine.redact(raw);

    assert!(!redacted.contains("sk-1234567890abcdef"));
    assert!(!redacted.contains("ghp_12345678901234567890"));
    assert!(!redacted.contains("AKIA1234567890ABCDEF"));
    assert!(!redacted.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    assert!(!redacted.contains("MIIEowIBAAKCAQEA"));

    assert!(redacted.contains("[REDACTED_API_KEY]"));
    assert!(redacted.contains("[REDACTED_GITHUB_TOKEN]"));
    assert!(redacted.contains("[REDACTED_AWS_KEY]"));
    assert!(redacted.contains("[REDACTED_BEARER_TOKEN]"));
    assert!(redacted.contains("[REDACTED_PRIVATE_KEY]"));

    // Sensitive files filter
    assert!(RedactionEngine::is_sensitive_file(Path::new(".env")));
    assert!(RedactionEngine::is_sensitive_file(Path::new(".env.production")));
    assert!(RedactionEngine::is_sensitive_file(Path::new("credentials.json")));
    assert!(RedactionEngine::is_sensitive_file(Path::new("id_rsa")));
    assert!(RedactionEngine::is_sensitive_file(Path::new("server.key")));
    assert!(!RedactionEngine::is_sensitive_file(Path::new("src/main.rs")));
    assert!(!RedactionEngine::is_sensitive_file(Path::new("Cargo.toml")));
}

#[test]
fn test_retrieval_cache_hit_miss_and_invalidation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path();
    let db_path = repo_root.join(".agent/memory.sqlite");
    let memory = MemoryEngine::open(&db_path).unwrap();

    let cache_store = RetrievalCacheStore::new(&memory);

    // Create a real file on disk
    let file_path = repo_root.join("src/lib.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

    let content = fs::read(&file_path).unwrap();
    let source_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&content);
        format!("{:x}", hasher.finalize())
    };

    let mut packet = ContextPacket::empty("Test task", 4000);
    packet.relevant_files.push(FileContextSnippet {
        path: "src/lib.rs".to_string(),
        language: "rust".to_string(),
        line_start: 1,
        line_end: 1,
        content: "pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
        source_hash: source_hash.clone(),
        token_estimate: 20,
    });

    let key = RetrievalCacheStore::compute_key("Test task", Some("commit123"), "budget=4000");

    // Put into cache
    cache_store.put(&key, &packet, 600).unwrap();

    // Cache hit
    let cached = cache_store.get(&key, repo_root).unwrap();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().task_summary, "Test task");

    // Modify file on disk -> cache should invalidate
    fs::write(&file_path, "pub fn add(a: i32, b: i32) -> i32 { a + b + 1 }").unwrap();

    let after_mod = cache_store.get(&key, repo_root).unwrap();
    assert!(after_mod.is_none(), "Cache must be invalidated when file on disk changes hash");
}

#[test]
fn test_graph_impact_callers_and_affected_tests() {
    let memory = MemoryEngine::open_in_memory().unwrap();

    // Insert files
    let lib_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/core.rs".to_string(),
        language: Some("rust".to_string()),
        size: 100,
        source_hash: Some("h1".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    let caller_file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/service.rs".to_string(),
        language: Some("rust".to_string()),
        size: 100,
        source_hash: Some("h2".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    let test_file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "tests/core_test.rs".to_string(),
        language: Some("rust".to_string()),
        size: 100,
        source_hash: Some("h3".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    // Insert symbols
    let target_sym = SymbolRecord {
        id: None,
        file_id: lib_id,
        name: "calculate_tax".to_string(),
        kind: "function".to_string(),
        line_start: 10,
        line_end: 20,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn calculate_tax(amount: f64) -> f64".to_string()),
        summary: None,
        parent: None,
    };
    memory.insert_symbols(lib_id, &[target_sym]).unwrap();
    let target_sym_id = memory.find_symbols_by_name("calculate_tax").unwrap()[0].id.unwrap();

    let caller_sym = SymbolRecord {
        id: None,
        file_id: caller_file_id,
        name: "process_checkout".to_string(),
        kind: "function".to_string(),
        line_start: 30,
        line_end: 50,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn process_checkout()".to_string()),
        summary: None,
        parent: None,
    };
    memory.insert_symbols(caller_file_id, &[caller_sym]).unwrap();
    let caller_sym_id = memory.find_symbols_by_name("process_checkout").unwrap()[0].id.unwrap();

    let test_sym = SymbolRecord {
        id: None,
        file_id: test_file_id,
        name: "test_calculate_tax".to_string(),
        kind: "function".to_string(),
        line_start: 5,
        line_end: 15,
        visibility: None,
        signature: Some("fn test_calculate_tax()".to_string()),
        summary: None,
        parent: None,
    };
    memory.insert_symbols(test_file_id, &[test_sym]).unwrap();
    let test_sym_id = memory.find_symbols_by_name("test_calculate_tax").unwrap()[0].id.unwrap();

    // Insert edges (caller -> target, test -> target)
    memory.insert_edge(&EdgeRecord {
        id: None,
        source_symbol_id: Some(caller_sym_id),
        target_symbol_id: Some(target_sym_id),
        edge_type: "calls".to_string(),
    }).unwrap();

    memory.insert_edge(&EdgeRecord {
        id: None,
        source_symbol_id: Some(test_sym_id),
        target_symbol_id: Some(target_sym_id),
        edge_type: "calls".to_string(),
    }).unwrap();

    let graph_engine = GraphQueryEngine::new(&memory);
    let subset = graph_engine.query_subgraph(&[target_sym_id], &[lib_id], 1).unwrap();

    assert!(subset.direct_callers.iter().any(|c| c.contains("process_checkout")));
    assert!(subset.affected_tests.iter().any(|t| t.contains("tests/core_test.rs")));
}

#[test]
fn test_failure_memory_lookup_and_warnings() {
    let memory = MemoryEngine::open_in_memory().unwrap();

    let file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/auth/token.rs".to_string(),
        language: Some("rust".to_string()),
        size: 50,
        source_hash: Some("hash".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    let failure = FailureRecord {
        id: None,
        error_type: "compiler".to_string(),
        message: "cannot borrow `token` as mutable, as it is also borrowed as immutable".to_string(),
        file_id: Some(file_id),
        symbol_id: None,
        fix_summary: Some("Cloned token prior to entering verification block".to_string()),
        status: Some("resolved".to_string()),
        resolved_at: Some("2026-10-18T12:00:00Z".to_string()),
        created_at: "2026-10-18T10:00:00Z".to_string(),
    };

    memory.insert_failure(&failure).unwrap();

    let lookup = FailureLookup::new(&memory);
    let results = lookup.find_failures(&[file_id], &[], "token borrowing error", 5).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].error_type, "compiler");
    assert!(results[0].message.contains("cannot borrow `token`"));
    assert!(results[0].fix_summary.as_ref().unwrap().contains("Cloned token"));
}

#[test]
fn test_doc_staleness_warning_generation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path();
    let memory = MemoryEngine::open_in_memory().unwrap();

    let file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/payment.rs".to_string(),
        language: Some("rust".to_string()),
        size: 80,
        source_hash: Some("current_hash_xyz".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: Some(".agent-docs/src/payment.md".to_string()),
        doc_status: Some("stale".to_string()),
    }).unwrap();

    memory.upsert_doc(&DocRecord {
        id: None,
        file_id,
        doc_path: ".agent-docs/src/payment.md".to_string(),
        doc_level: 0,
        source_hash: Some("outdated_hash_123".to_string()),
        status: "stale".to_string(),
        updated_at: "2026-10-18T00:00:00Z".to_string(),
        stale_reason: Some("Signature modified in commit def456".to_string()),
    }).unwrap();

    let doc_lookup = DocLookup::new(&memory, repo_root);
    let warnings = doc_lookup.check_doc_staleness(&[file_id]).unwrap();

    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].file_path, "src/payment.rs");
    assert_eq!(warnings[0].status, "stale");
    assert!(warnings[0].reason.contains("Signature modified"));
}

#[test]
fn test_retrieval_engine_performance_under_200ms() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path();
    let db_path = repo_root.join(".agent/memory.sqlite");
    let memory = MemoryEngine::open(&db_path).unwrap();
    let config = Config::default_config();

    // Create a few source files on disk and in memory
    for i in 0..15 {
        let rel_path = format!("src/module_{}.rs", i);
        let full_path = repo_root.join(&rel_path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, format!("pub fn compute_{}(x: i32) -> i32 {{ x * 2 }}", i)).unwrap();

        let fid = memory.upsert_file(&FileRecord {
            id: None,
            path: rel_path.clone(),
            language: Some("rust".to_string()),
            size: 50,
            source_hash: Some(format!("hash_{}", i)),
            parse_status: Some("ok".to_string()),
            last_commit: None,
            last_modified: None,
            summary: None,
            risk_level: None,
            doc_path: None,
            doc_status: None,
        }).unwrap();

        memory.insert_symbols(fid, &[SymbolRecord {
            id: None,
            file_id: fid,
            name: format!("compute_{}", i),
            kind: "function".to_string(),
            line_start: 1,
            line_end: 1,
            visibility: Some("pub".to_string()),
            signature: Some(format!("pub fn compute_{}(x: i32) -> i32", i)),
            summary: None,
            parent: None,
        }]).unwrap();
    }

    let engine = RetrievalEngine::new(&memory, &config, repo_root);
    let opts = RetrievalOptions {
        budget_tokens: Some(4000),
        no_cache: true,
        max_depth: 1,
        include_commits: false,
    };

    let start = Instant::now();
    let packet = engine.retrieve_context("Fix issue in compute_5 and module_5.rs", &opts).unwrap();
    let duration = start.elapsed();

    assert!(duration.as_millis() < 200, "Retrieval latency must be < 200ms (was {}ms)", duration.as_millis());
    assert!(packet.relevant_files.iter().any(|f| f.path.contains("module_5")));
    assert!(packet.relevant_symbols.iter().any(|s| s.name == "compute_5"));
}
