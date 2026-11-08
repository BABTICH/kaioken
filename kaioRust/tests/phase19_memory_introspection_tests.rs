use kaioken::memory::{
    normalize_file_path, risk_rank, CommitChangeRecord, CommitRecord, DocRecord, EdgeRecord,
    FailureRecord, FileExplanation, FileRecord, MemoryEngine, SymbolRecord,
};
use std::path::Path;

#[test]
fn test_explain_full_file() {
    let memory = MemoryEngine::open_in_memory().expect("open in-memory db");

    // 1. Insert file
    let file_rec = FileRecord {
        id: None,
        path: "src/scanner/tree_sitter.rs".to_string(),
        language: Some("rust".to_string()),
        size: 13792,
        source_hash: Some("a1b2c3d4e5f67890abcdef1234567890".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: Some("def5678".to_string()),
        last_modified: Some("2026-10-18T15:30:00Z".to_string()),
        summary: Some("Tree-sitter parser implementation".to_string()),
        risk_level: Some("low".to_string()),
        doc_path: Some(".agent-docs/src/scanner/tree_sitter.md".to_string()),
        doc_status: Some("current".to_string()),
    };
    let file_id = memory.upsert_file(&file_rec).expect("upsert file");

    // Insert target file for edges
    let target_file_rec = FileRecord {
        id: None,
        path: "src/error.rs".to_string(),
        language: Some("rust".to_string()),
        size: 2048,
        source_hash: Some("0987654321fedcba0987654321fedcba".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let target_file_id = memory
        .upsert_file(&target_file_rec)
        .expect("upsert target file");

    // 2. Insert symbols
    let symbols = vec![
        SymbolRecord {
            id: None,
            file_id,
            name: "SymbolKind".to_string(),
            kind: "enum".to_string(),
            line_start: 9,
            line_end: 22,
            visibility: Some("pub".to_string()),
            signature: Some("pub enum SymbolKind".to_string()),
            summary: Some("Kinds of AST symbols".to_string()),
            parent: None,
        },
        SymbolRecord {
            id: None,
            file_id,
            name: "as_str".to_string(),
            kind: "fn".to_string(),
            line_start: 25,
            line_end: 38,
            visibility: Some("pub".to_string()),
            signature: Some("pub fn as_str(&self) -> &str".to_string()),
            summary: None,
            parent: Some("SymbolKind".to_string()),
        },
        SymbolRecord {
            id: None,
            file_id,
            name: "extract_symbols".to_string(),
            kind: "fn".to_string(),
            line_start: 72,
            line_end: 150,
            visibility: Some("pub".to_string()),
            signature: Some(
                "pub fn extract_symbols(source: &str) -> Vec<ExtractedSymbol>".to_string(),
            ),
            summary: Some("Extract symbols from source code".to_string()),
            parent: None,
        },
    ];
    memory
        .insert_symbols(file_id, &symbols)
        .expect("insert symbols");
    let stored_syms = memory
        .get_symbols_for_file(file_id)
        .expect("get stored symbols");
    let extract_sym_id = stored_syms
        .iter()
        .find(|s| s.name == "extract_symbols")
        .unwrap()
        .id
        .unwrap();

    let target_symbols = vec![SymbolRecord {
        id: None,
        file_id: target_file_id,
        name: "KaiokenError".to_string(),
        kind: "enum".to_string(),
        line_start: 5,
        line_end: 30,
        visibility: Some("pub".to_string()),
        signature: Some("pub enum KaiokenError".to_string()),
        summary: None,
        parent: None,
    }];
    memory
        .insert_symbols(target_file_id, &target_symbols)
        .expect("insert target symbols");
    let target_syms = memory
        .get_symbols_for_file(target_file_id)
        .expect("get target symbols");
    let err_sym_id = target_syms[0].id.unwrap();

    // 3. Raw Imports
    let imports = vec![
        "crate::error::KaiokenError".to_string(),
        "tree_sitter".to_string(),
    ];
    memory
        .insert_raw_imports(file_id, &imports)
        .expect("insert raw imports");

    // 4. Edges
    let edge = EdgeRecord {
        id: None,
        source_symbol_id: Some(extract_sym_id),
        target_symbol_id: Some(err_sym_id),
        edge_type: "imports".to_string(),
    };
    memory.insert_edge(&edge).expect("insert edge");

    // 5. Documentation
    let doc = DocRecord {
        id: None,
        file_id,
        doc_path: ".agent-docs/src/scanner/tree_sitter.md".to_string(),
        doc_level: 0,
        source_hash: Some("a1b2c3d4e5f67890abcdef1234567890".to_string()),
        status: "current".to_string(),
        updated_at: "2026-10-18T15:30:00Z".to_string(),
        stale_reason: None,
    };
    memory.upsert_doc(&doc).expect("upsert doc");

    // 6. Commits
    let commit = CommitRecord {
        id: None,
        hash: "def567890abcdef".to_string(),
        message: "Fix partial parse recovery".to_string(),
        author: Some("developer@example.com".to_string()),
        created_at: Some("2026-10-17T12:15:00Z".to_string()),
        summary: Some("Fixed type mismatch in visibility detection".to_string()),
        risk_level: Some("medium".to_string()),
    };
    let change = CommitChangeRecord {
        id: None,
        commit_id: 0,
        file_id: Some(file_id),
        symbol_id: Some(extract_sym_id),
        change_type: "modified".to_string(),
    };
    memory
        .record_commit(&commit, &[change])
        .expect("record commit");

    // 7. Failures
    let failure = FailureRecord {
        id: None,
        error_type: "compiler".to_string(),
        message: "E0308: mismatched types in detect_visibility".to_string(),
        file_id: Some(file_id),
        symbol_id: Some(extract_sym_id),
        fix_summary: Some("Fixed in def5678".to_string()),
        status: Some("resolved".to_string()),
        resolved_at: Some("2026-10-17T12:15:00Z".to_string()),
        created_at: "2026-10-17T10:00:00Z".to_string(),
    };
    memory.insert_failure(&failure).expect("insert failure");

    // Test explain_file
    let explanation = memory
        .explain_file("src/scanner/tree_sitter.rs")
        .expect("explain_file");

    assert_eq!(explanation.file.path, "src/scanner/tree_sitter.rs");
    assert_eq!(explanation.file.language.as_deref(), Some("rust"));
    assert_eq!(explanation.symbols.len(), 3);
    assert_eq!(explanation.outgoing_edges.len(), 1);
    assert_eq!(explanation.raw_imports.len(), 2);
    assert!(explanation.documentation.is_some());
    assert_eq!(explanation.recent_commits.len(), 1);
    assert_eq!(explanation.recent_commits[0].hash, "def567890abcdef");
    assert_eq!(explanation.failures.len(), 1);
    assert_eq!(explanation.failures[0].status.as_deref(), Some("resolved"));

    // Verify terminal rendering
    let term = explanation.render_terminal();
    assert!(term.contains("Memory Explain — src/scanner/tree_sitter.rs"));
    assert!(term.contains("File Metadata"));
    assert!(term.contains("Rust"));
    assert!(term.contains("Symbols (3 extracted)"));
    assert!(term.contains("SymbolKind"));
    assert!(term.contains("as_str"));
    assert!(term.contains("Dependencies"));
    assert!(term.contains("Documentation"));
    assert!(term.contains("Recent Commits"));
    assert!(term.contains("Failure Memory"));
    assert!(term.contains("E0308"));
}

#[test]
fn test_explain_missing_file() {
    let memory = MemoryEngine::open_in_memory().expect("open in-memory db");
    let err = memory.explain_file("src/does_not_exist.rs");
    assert!(err.is_err());
    let err_msg = err.unwrap_err().to_string();
    assert!(err_msg.contains("not found in memory"));
}

#[test]
fn test_explain_no_edges() {
    let memory = MemoryEngine::open_in_memory().expect("open in-memory db");

    let file_rec = FileRecord {
        id: None,
        path: "src/isolated.rs".to_string(),
        language: Some("rust".to_string()),
        size: 512,
        source_hash: Some("11223344".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let file_id = memory.upsert_file(&file_rec).expect("upsert file");

    let symbols = vec![SymbolRecord {
        id: None,
        file_id,
        name: "solo_func".to_string(),
        kind: "fn".to_string(),
        line_start: 1,
        line_end: 10,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn solo_func()".to_string()),
        summary: None,
        parent: None,
    }];
    memory
        .insert_symbols(file_id, &symbols)
        .expect("insert symbols");

    let explanation = memory
        .explain_file("src/isolated.rs")
        .expect("explain_file isolated");
    assert_eq!(explanation.file.path, "src/isolated.rs");
    assert_eq!(explanation.symbols.len(), 1);
    assert!(explanation.outgoing_edges.is_empty());
    assert!(explanation.incoming_edges.is_empty());
    assert!(explanation.documentation.is_none());
    assert!(explanation.recent_commits.is_empty());
    assert!(explanation.failures.is_empty());

    let term = explanation.render_terminal();
    assert!(term.contains("src/isolated.rs"));
    assert!(term.contains("No outgoing dependencies recorded"));
    assert!(term.contains("No incoming dependents recorded"));
    assert!(term.contains("No documentation generated yet"));
    assert!(term.contains("No recent commits touching this file"));
    assert!(term.contains("No failures recorded for this file"));
}

#[test]
fn test_recent_default() {
    let memory = MemoryEngine::open_in_memory().expect("open in-memory db");

    for i in 1..=15 {
        let commit = CommitRecord {
            id: None,
            hash: format!("hash_{:04}", i),
            message: format!("Commit message {}", i),
            author: Some("tester@example.com".to_string()),
            created_at: Some(format!("2026-10-18T{:02}:00:00Z", i)),
            summary: Some(format!("Summary of changes for commit {}", i)),
            risk_level: Some(if i % 3 == 0 {
                "high".to_string()
            } else {
                "low".to_string()
            }),
        };
        memory.record_commit(&commit, &[]).expect("record commit");
    }

    let result = memory
        .get_recent_commits_filtered(10, None, None)
        .expect("recent commits");
    assert_eq!(result.total_commits, 15);
    assert_eq!(result.shown_commits, 10);
    assert_eq!(result.commits.len(), 10);

    // Latest commit should be first (hash_0015)
    assert_eq!(result.commits[0].commit.hash, "hash_0015");
    assert_eq!(result.commits[9].commit.hash, "hash_0006");

    let term = result.render_terminal();
    assert!(term.contains("Recent Commit Memories — Last 10"));
    assert!(term.contains("Showing 10 of 15 total commit memories"));
}

#[test]
fn test_recent_min_risk() {
    let memory = MemoryEngine::open_in_memory().expect("open in-memory db");

    let risks = ["low", "medium", "high", "critical", "low", "medium", "high"];
    for (idx, &risk) in risks.iter().enumerate() {
        let commit = CommitRecord {
            id: None,
            hash: format!("commit_{}", idx),
            message: format!("Commit with risk {}", risk),
            author: Some("author@test.com".to_string()),
            created_at: Some(format!("2026-10-18T1{:02}:00:00Z", idx)),
            summary: None,
            risk_level: Some(risk.to_string()),
        };
        memory.record_commit(&commit, &[]).expect("record commit");
    }

    // Filter by high (should return high and critical)
    let res_high = memory
        .get_recent_commits_filtered(10, Some("high"), None)
        .expect("filter high");
    for detail in &res_high.commits {
        let r = detail.commit.risk_level.as_deref().unwrap_or("low");
        assert!(risk_rank(r) >= risk_rank("high"));
    }
    assert_eq!(res_high.commits.len(), 3); // idx 2(high), 3(critical), 6(high)

    // Filter by medium (should return medium, high, critical)
    let res_med = memory
        .get_recent_commits_filtered(10, Some("medium"), None)
        .expect("filter medium");
    assert_eq!(res_med.commits.len(), 5);
}

#[test]
fn test_recent_file_filter() {
    let memory = MemoryEngine::open_in_memory().expect("open in-memory db");

    let file_a = FileRecord {
        id: None,
        path: "src/module_a.rs".to_string(),
        language: Some("rust".to_string()),
        size: 100,
        source_hash: None,
        parse_status: None,
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let fid_a = memory.upsert_file(&file_a).expect("upsert file a");

    let file_b = FileRecord {
        id: None,
        path: "src/module_b.rs".to_string(),
        language: Some("rust".to_string()),
        size: 200,
        source_hash: None,
        parse_status: None,
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let fid_b = memory.upsert_file(&file_b).expect("upsert file b");

    // Commit 1 touches A
    let c1 = CommitRecord {
        id: None,
        hash: "c1111".to_string(),
        message: "Change file A".to_string(),
        author: None,
        created_at: None,
        summary: None,
        risk_level: Some("low".to_string()),
    };
    let ch1 = CommitChangeRecord {
        id: None,
        commit_id: 0,
        file_id: Some(fid_a),
        symbol_id: None,
        change_type: "modified".to_string(),
    };
    memory.record_commit(&c1, &[ch1]).expect("record c1");

    // Commit 2 touches B
    let c2 = CommitRecord {
        id: None,
        hash: "c2222".to_string(),
        message: "Change file B".to_string(),
        author: None,
        created_at: None,
        summary: None,
        risk_level: Some("medium".to_string()),
    };
    let ch2 = CommitChangeRecord {
        id: None,
        commit_id: 0,
        file_id: Some(fid_b),
        symbol_id: None,
        change_type: "modified".to_string(),
    };
    memory.record_commit(&c2, &[ch2]).expect("record c2");

    // Commit 3 touches A
    let c3 = CommitRecord {
        id: None,
        hash: "c3333".to_string(),
        message: "Refactor file A".to_string(),
        author: None,
        created_at: None,
        summary: None,
        risk_level: Some("high".to_string()),
    };
    let ch3 = CommitChangeRecord {
        id: None,
        commit_id: 0,
        file_id: Some(fid_a),
        symbol_id: None,
        change_type: "modified".to_string(),
    };
    memory.record_commit(&c3, &[ch3]).expect("record c3");

    // Filter by module_a.rs
    let res = memory
        .get_recent_commits_filtered(10, None, Some("src/module_a.rs"))
        .expect("filter file a");
    assert_eq!(res.commits.len(), 2);
    assert_eq!(res.commits[0].commit.hash, "c3333");
    assert_eq!(res.commits[1].commit.hash, "c1111");
}

#[test]
fn test_explain_json_serialization_roundtrip() {
    let memory = MemoryEngine::open_in_memory().expect("open in-memory db");

    let file_rec = FileRecord {
        id: None,
        path: "src/sample.rs".to_string(),
        language: Some("rust".to_string()),
        size: 1024,
        source_hash: Some("abcd1234efgh5678".to_string()),
        parse_status: Some("ok".to_string()),
        last_commit: Some("c123".to_string()),
        last_modified: Some("2026-10-18T10:00:00Z".to_string()),
        summary: Some("Sample file".to_string()),
        risk_level: Some("low".to_string()),
        doc_path: Some(".agent-docs/src/sample.md".to_string()),
        doc_status: Some("fresh".to_string()),
    };
    let file_id = memory.upsert_file(&file_rec).expect("upsert file");

    let symbols = vec![SymbolRecord {
        id: None,
        file_id,
        name: "hello".to_string(),
        kind: "fn".to_string(),
        line_start: 1,
        line_end: 5,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn hello()".to_string()),
        summary: None,
        parent: None,
    }];
    memory
        .insert_symbols(file_id, &symbols)
        .expect("insert symbols");

    let explanation = memory.explain_file("src/sample.rs").expect("explain_file");

    let json_str = serde_json::to_string(&explanation).expect("serialize to json");
    assert!(json_str.contains("\"path\":\"src/sample.rs\""));
    assert!(json_str.contains("\"name\":\"hello\""));

    let deserialized: FileExplanation = serde_json::from_str(&json_str).expect("deserialize json");
    assert_eq!(deserialized.file.path, explanation.file.path);
    assert_eq!(deserialized.symbols.len(), explanation.symbols.len());
    assert_eq!(deserialized.symbols[0].name, "hello");
}

#[test]
fn test_normalize_file_path() {
    let root = Path::new("d:/project/my_repo");
    assert_eq!(
        normalize_file_path("d:/project/my_repo/src/main.rs", root),
        "src/main.rs"
    );
    assert_eq!(
        normalize_file_path("./src/scanner/parser.rs", root),
        "src/scanner/parser.rs"
    );
    assert_eq!(
        normalize_file_path(r"src\cli\memory.rs", root),
        "src/cli/memory.rs"
    );
}
