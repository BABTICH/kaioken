use std::sync::Arc;
use std::thread;
use tempfile::tempdir;
use kaioken::memory::{
    EdgeRecord, FileRecord, MemoryEngine, SymbolRecord,
};
use kaioken::scanner::{
    DiscoveredFile, ExtractedSymbol, InventorySummary, Language, ParseStatus, SymbolKind,
};

#[test]
fn test_sqlite_schema_init_and_pragmas() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("memory.sqlite");

    let engine = MemoryEngine::open(&db_path).unwrap();
    let conn = engine.db().connection();

    // Verify WAL mode
    let journal_mode: String = conn.query_row("PRAGMA journal_mode;", [], |r| r.get(0)).unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");

    // Verify foreign keys enabled
    let foreign_keys: i32 = conn.query_row("PRAGMA foreign_keys;", [], |r| r.get(0)).unwrap();
    assert_eq!(foreign_keys, 1);

    // Verify busy timeout
    let busy_timeout: i32 = conn.query_row("PRAGMA busy_timeout;", [], |r| r.get(0)).unwrap();
    assert_eq!(busy_timeout, 5000);
}

#[test]
fn test_sqlite_file_and_symbol_crud() {
    let engine = MemoryEngine::open_in_memory().unwrap();

    let mut file_rec = FileRecord::new("src/auth.rs", 1024);
    file_rec.language = Some("rust".to_string());
    file_rec.source_hash = Some("abcdef123456".to_string());

    let file_id = engine.upsert_file(&file_rec).unwrap();
    assert!(file_id > 0);

    let retrieved = engine.get_file_by_path("src/auth.rs").unwrap().unwrap();
    assert_eq!(retrieved.path, "src/auth.rs");
    assert_eq!(retrieved.size, 1024);
    assert_eq!(retrieved.source_hash.as_deref(), Some("abcdef123456"));

    // Insert symbols
    let sym1 = SymbolRecord {
        id: None,
        file_id,
        name: "login".to_string(),
        kind: "function".to_string(),
        line_start: 10,
        line_end: 25,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn login()".to_string()),
        summary: None,
        parent: None,
    };
    let sym2 = SymbolRecord {
        id: None,
        file_id,
        name: "Session".to_string(),
        kind: "struct".to_string(),
        line_start: 30,
        line_end: 45,
        visibility: Some("pub".to_string()),
        signature: Some("pub struct Session".to_string()),
        summary: None,
        parent: None,
    };

    engine.insert_symbols(file_id, &[sym1, sym2]).unwrap();
    assert_eq!(engine.count_symbols().unwrap(), 2);

    let symbols = engine.get_symbols_for_file(file_id).unwrap();
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].name, "login");
    assert_eq!(symbols[1].name, "Session");

    // Search by name
    let found = engine.find_symbols_by_name("login").unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].file_id, file_id);
}

#[test]
fn test_sqlite_foreign_key_cascade_deletion() {
    let engine = MemoryEngine::open_in_memory().unwrap();

    let file_rec = FileRecord::new("src/delete_me.rs", 500);
    let file_id = engine.upsert_file(&file_rec).unwrap();

    let sym = SymbolRecord::new(file_id, "temporary", "function", 1, 10);
    engine.insert_symbols(file_id, &[sym]).unwrap();
    engine.insert_raw_imports(file_id, &["use std::io;".to_string()]).unwrap();

    assert_eq!(engine.count_files().unwrap(), 1);
    assert_eq!(engine.count_symbols().unwrap(), 1);
    assert_eq!(engine.count_raw_imports().unwrap(), 1);

    // Delete file
    engine.delete_file("src/delete_me.rs").unwrap();

    // Verify cascade deleted associated symbols and imports
    assert_eq!(engine.count_files().unwrap(), 0);
    assert_eq!(engine.count_symbols().unwrap(), 0, "Symbols must be cascade deleted");
    assert_eq!(engine.count_raw_imports().unwrap(), 0, "Imports must be cascade deleted");
}

#[test]
fn test_sqlite_dependency_edges() {
    let engine = MemoryEngine::open_in_memory().unwrap();

    let edge = EdgeRecord {
        id: None,
        source_symbol_id: Some(10),
        target_symbol_id: Some(20),
        edge_type: "calls".to_string(),
    };

    let edge_id = engine.insert_edge(&edge).unwrap();
    assert!(edge_id > 0);

    let outgoing = engine.get_edges_for_source(10).unwrap();
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].target_symbol_id, Some(20));
    assert_eq!(outgoing[0].edge_type, "calls");

    let incoming = engine.get_edges_for_target(20).unwrap();
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].source_symbol_id, Some(10));
}

#[test]
fn test_sqlite_sync_inventory_bridge() {
    let engine = MemoryEngine::open_in_memory().unwrap();

    let file1 = DiscoveredFile {
        path: std::path::PathBuf::from("src/main.rs"),
        relative_path: std::path::PathBuf::from("src/main.rs"),
        language: Language::Rust,
        size: 800,
        file_hash: Some("hash1".to_string()),
        parse_status: ParseStatus::Ok,
        last_modified: Some("2026-10-08T19:00:00Z".to_string()),
        symbols: vec![
            ExtractedSymbol {
                name: "main".to_string(),
                kind: SymbolKind::Function,
                line_start: 1,
                line_end: 10,
                visibility: "pub".to_string(),
                signature: Some("pub fn main()".to_string()),
                parent: None,
            },
        ],
        imports_raw: vec!["use std::path::Path;".to_string()],
        imports: vec!["std::path::Path".to_string()],
        exports: vec!["main".to_string()],
    };

    let file2 = DiscoveredFile {
        path: std::path::PathBuf::from("README.md"),
        relative_path: std::path::PathBuf::from("README.md"),
        language: Language::Markdown,
        size: 300,
        file_hash: Some("hash2".to_string()),
        parse_status: ParseStatus::Ok,
        last_modified: Some("2026-10-08T19:00:00Z".to_string()),
        symbols: Vec::new(),
        imports_raw: Vec::new(),
        imports: Vec::new(),
        exports: Vec::new(),
    };

    let summary = InventorySummary {
        total_files: 2,
        total_symbols: 1,
        total_size_bytes: 1100,
        files_by_language: Default::default(),
        files_by_status: Default::default(),
        files: vec![file1, file2],
    };

    let stats = engine.sync_inventory(&summary).unwrap();
    assert_eq!(stats.files_synced, 2);
    assert_eq!(stats.symbols_synced, 1);
    assert_eq!(stats.imports_synced, 1);

    assert_eq!(engine.count_files().unwrap(), 2);
    assert_eq!(engine.count_symbols().unwrap(), 1);
    assert_eq!(engine.count_raw_imports().unwrap(), 1);

    // Re-running sync updates existing records idempotently
    let stats2 = engine.sync_inventory(&summary).unwrap();
    assert_eq!(stats2.files_synced, 2);
    assert_eq!(engine.count_files().unwrap(), 2, "File count must remain 2 after re-sync");
    assert_eq!(engine.count_symbols().unwrap(), 1, "Symbol count must remain 1 after re-sync");
}

#[test]
fn test_sqlite_concurrency_readers_and_writer() {
    let temp = tempdir().unwrap();
    let db_path = Arc::new(temp.path().join("concurrency.sqlite"));

    // Initialize DB
    {
        let engine = MemoryEngine::open(&db_path).unwrap();
        let file = FileRecord::new("init.rs", 100);
        engine.upsert_file(&file).unwrap();
    }

    let mut handles = Vec::new();

    // Spawn 4 concurrent readers
    for i in 0..4 {
        let path = Arc::clone(&db_path);
        handles.push(thread::spawn(move || {
            let engine = MemoryEngine::open(&path).unwrap();
            for _ in 0..20 {
                let count = engine.count_files().unwrap();
                assert!(count >= 1);
            }
            i
        }));
    }

    // Spawn 2 concurrent writers
    for i in 0..2 {
        let path = Arc::clone(&db_path);
        handles.push(thread::spawn(move || {
            let engine = MemoryEngine::open(&path).unwrap();
            for step in 0..10 {
                let file_path = format!("worker_{}_step_{}.rs", i, step);
                let file = FileRecord::new(&file_path, 200 + step);
                engine.upsert_file(&file).unwrap();
            }
            i + 100
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_engine = MemoryEngine::open(&db_path).unwrap();
    let final_count = final_engine.count_files().unwrap();
    assert_eq!(final_count, 1 + (2 * 10), "Must contain initial file plus all writer insertions");
}
