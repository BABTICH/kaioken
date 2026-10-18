use std::fs;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use kaioken::docs::{
    DocFrontmatter, DocGenerateOptions, DocGenerator,
    StalenessCascadeEngine, StalenessDetector,
};
use kaioken::memory::{CommitRecord, CommitChangeRecord, DocRecord, FileRecord, MemoryEngine, RawImportRecord};

#[test]
fn test_staleness_cascade_direct_and_one_hop() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();

    // 1. Setup File A (core), File B (service -> core), File C (api -> service)
    let mut file_a = FileRecord::new("src/core.rs", 100);
    file_a.source_hash = Some("hash_a".to_string());
    let id_a = memory.upsert_file(&file_a).unwrap();

    let mut file_b = FileRecord::new("src/service.rs", 200);
    file_b.source_hash = Some("hash_b".to_string());
    let id_b = memory.upsert_file(&file_b).unwrap();

    let mut file_c = FileRecord::new("src/api.rs", 300);
    file_c.source_hash = Some("hash_c".to_string());
    let id_c = memory.upsert_file(&file_c).unwrap();

    // Register dependencies in import_raw:
    // B depends on A (resolved_file_id: Some(id_a))
    let import_b = RawImportRecord {
        id: None,
        file_id: id_b,
        raw_import: "use crate::core;".to_string(),
        resolved_file_id: Some(id_a),
        resolved_symbol_id: None,
        status: "resolved".to_string(),
    };
    let conn = memory.db().connection();
    conn.execute(
        "INSERT INTO import_raw (file_id, raw_import, resolved_file_id, status) VALUES (?1, ?2, ?3, ?4);",
        rusqlite::params![import_b.file_id, import_b.raw_import, import_b.resolved_file_id, import_b.status],
    ).unwrap();

    // C depends on B (resolved_file_id: Some(id_b))
    let import_c = RawImportRecord {
        id: None,
        file_id: id_c,
        raw_import: "use crate::service;".to_string(),
        resolved_file_id: Some(id_b),
        resolved_symbol_id: None,
        status: "resolved".to_string(),
    };
    conn.execute(
        "INSERT INTO import_raw (file_id, raw_import, resolved_file_id, status) VALUES (?1, ?2, ?3, ?4);",
        rusqlite::params![import_c.file_id, import_c.raw_import, import_c.resolved_file_id, import_c.status],
    ).unwrap();

    // Register initial fresh docs for A, B, C
    let doc_a = DocRecord {
        id: None,
        file_id: id_a,
        doc_path: "src/core.md".to_string(),
        doc_level: 0,
        source_hash: Some("hash_a".to_string()),
        status: "fresh".to_string(),
        updated_at: "2026-10-08T10:00:00Z".to_string(),
        stale_reason: None,
    };
    memory.upsert_doc(&doc_a).unwrap();

    let doc_b = DocRecord {
        id: None,
        file_id: id_b,
        doc_path: "src/service.md".to_string(),
        doc_level: 0,
        source_hash: Some("hash_b".to_string()),
        status: "fresh".to_string(),
        updated_at: "2026-10-08T10:00:00Z".to_string(),
        stale_reason: None,
    };
    memory.upsert_doc(&doc_b).unwrap();

    let doc_c = DocRecord {
        id: None,
        file_id: id_c,
        doc_path: "src/api.md".to_string(),
        doc_level: 0,
        source_hash: Some("hash_c".to_string()),
        status: "fresh".to_string(),
        updated_at: "2026-10-08T10:00:00Z".to_string(),
        stale_reason: None,
    };
    memory.upsert_doc(&doc_c).unwrap();

    // Trigger staleness cascade with change to src/core.rs
    let report = StalenessCascadeEngine::cascade_changes(
        &repo_root,
        &["src/core.rs".to_string()],
        &memory,
    ).unwrap();

    assert_eq!(report.directly_stale, vec!["src/core.rs".to_string()]);
    assert_eq!(report.maybe_stale_dependents, vec!["src/service.rs".to_string()]);
    assert_eq!(report.cascade_depth, 1);

    // Verify DB states:
    // A must be stale
    let updated_a = memory.get_doc_by_file_id(id_a).unwrap().unwrap();
    assert_eq!(updated_a.status, "stale");
    assert!(updated_a.stale_reason.unwrap().contains("Direct"));

    // B must be maybe_stale
    let updated_b = memory.get_doc_by_file_id(id_b).unwrap().unwrap();
    assert_eq!(updated_b.status, "maybe_stale");
    assert!(updated_b.stale_reason.unwrap().contains("Direct dependency"));

    // C must remain FRESH (no transitive runaway cascade!)
    let updated_c = memory.get_doc_by_file_id(id_c).unwrap().unwrap();
    assert_eq!(updated_c.status, "fresh");
    assert!(updated_c.stale_reason.is_none());
}

#[test]
fn test_commit_recording_triggers_automatic_staleness_cascade() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();

    let file1 = FileRecord::new("src/lib.rs", 150);
    let id1 = memory.upsert_file(&file1).unwrap();

    let file2 = FileRecord::new("src/main.rs", 250);
    let id2 = memory.upsert_file(&file2).unwrap();

    // main.rs imports lib.rs
    let conn = memory.db().connection();
    conn.execute(
        "INSERT INTO import_raw (file_id, raw_import, resolved_file_id, status) VALUES (?1, ?2, ?3, 'resolved');",
        rusqlite::params![id2, "use crate::lib;", id1],
    ).unwrap();

    // Initial docs: fresh
    memory.upsert_doc(&DocRecord {
        id: None,
        file_id: id1,
        doc_path: "src/lib.md".to_string(),
        doc_level: 0,
        source_hash: Some("h1".to_string()),
        status: "fresh".to_string(),
        updated_at: "2026-10-08T10:00:00Z".to_string(),
        stale_reason: None,
    }).unwrap();

    memory.upsert_doc(&DocRecord {
        id: None,
        file_id: id2,
        doc_path: "src/main.md".to_string(),
        doc_level: 0,
        source_hash: Some("h2".to_string()),
        status: "fresh".to_string(),
        updated_at: "2026-10-08T10:00:00Z".to_string(),
        stale_reason: None,
    }).unwrap();

    // Record a commit that modified src/lib.rs
    let commit = CommitRecord {
        id: None,
        hash: "abcdef1234567890".to_string(),
        message: "Refactor core exports".to_string(),
        author: Some("Dev <dev@example.com>".to_string()),
        created_at: Some("2026-10-08T11:00:00Z".to_string()),
        summary: None,
        risk_level: Some("low".to_string()),
    };

    let changes = vec![CommitChangeRecord {
        id: None,
        commit_id: 0,
        file_id: Some(id1),
        symbol_id: None,
        change_type: "modified".to_string(),
    }];

    memory.record_commit(&commit, &changes).unwrap();

    // Verify lib.rs transitioned to stale
    let doc1 = memory.get_doc_by_file_id(id1).unwrap().unwrap();
    assert_eq!(doc1.status, "stale");
    assert!(doc1.stale_reason.unwrap().contains("Modified in commit"));

    // Verify dependent main.rs transitioned to maybe_stale
    let doc2 = memory.get_doc_by_file_id(id2).unwrap().unwrap();
    assert_eq!(doc2.status, "maybe_stale");
    assert!(doc2.stale_reason.unwrap().contains("Dependency 'src/lib.rs' modified in commit"));
}

#[test]
fn test_incremental_doc_generation_stale_only() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let docs_root = repo_root.join(".agent-docs");
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();
    let generator = DocGenerator::new(&repo_root, &docs_root);

    // Create 3 files on disk
    let files_data = [
        ("src/f1.rs", "pub fn f1() {}", "fresh"),
        ("src/f2.rs", "pub fn f2() {}", "stale"),
        ("src/f3.rs", "pub fn f3() {}", "maybe_stale"),
    ];

    for (rel_path, content, initial_status) in &files_data {
        let abs_path = repo_root.join(rel_path);
        fs::write(&abs_path, content).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        let mut file_rec = FileRecord::new(*rel_path, content.len());
        file_rec.source_hash = Some(hash.clone());
        let fid = memory.upsert_file(&file_rec).unwrap();

        // Also write mirrored doc on disk
        let doc_rel_path = format!("{}.md", rel_path.trim_end_matches(".rs"));
        let doc_abs_path = docs_root.join(&doc_rel_path);
        if let Some(p) = doc_abs_path.parent() {
            fs::create_dir_all(p).unwrap();
        }

        let fm = DocFrontmatter {
            source_path: rel_path.to_string(),
            source_hash: hash.clone(),
            doc_level: 0,
            generated_at: "2026-10-08T10:00:00Z".to_string(),
            generator: "deterministic".to_string(),
            status: initial_status.to_string(),
        };
        fs::write(&doc_abs_path, format!("{}# {}\n", fm.to_yaml_header(), rel_path)).unwrap();

        memory.upsert_doc(&DocRecord {
            id: None,
            file_id: fid,
            doc_path: doc_rel_path,
            doc_level: 0,
            source_hash: Some(hash),
            status: initial_status.to_string(),
            updated_at: "2026-10-08T10:00:00Z".to_string(),
            stale_reason: if *initial_status != "fresh" { Some("Stale test".to_string()) } else { None },
        }).unwrap();
    }

    // Run generate_all_docs with stale_only: true
    let options = DocGenerateOptions {
        level: 0,
        force: false,
        stale_only: true,
        no_llm: true,
        limit: None,
    };

    let summary = generator.generate_all_docs(&memory, &options).unwrap();
    assert_eq!(summary.total_processed, 3);
    assert_eq!(summary.skipped, 1); // f1.rs is fresh, skipped
    assert_eq!(summary.updated, 2); // f2.rs and f3.rs refreshed

    // Verify all docs in DB are now fresh
    let status_counts = memory.count_docs_by_status().unwrap();
    assert_eq!(status_counts.get("fresh"), Some(&3));
    assert_eq!(status_counts.get("stale").unwrap_or(&0), &0);
    assert_eq!(status_counts.get("maybe_stale").unwrap_or(&0), &0);
}

#[test]
fn test_doc_generation_budget_limit() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let docs_root = repo_root.join(".agent-docs");
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();
    let generator = DocGenerator::new(&repo_root, &docs_root);

    // Create 4 files with no docs yet
    for i in 1..=4 {
        let path = format!("src/chunk_{}.rs", i);
        let content = format!("pub fn chunk_{}() {{}}", i);
        fs::write(repo_root.join(&path), &content).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        let mut file_rec = FileRecord::new(&path, content.len());
        file_rec.source_hash = Some(hash);
        memory.upsert_file(&file_rec).unwrap();
    }

    // Generate with limit = 2
    let options = DocGenerateOptions {
        level: 0,
        force: false,
        stale_only: false,
        no_llm: true,
        limit: Some(2),
    };

    let summary = generator.generate_all_docs(&memory, &options).unwrap();
    assert_eq!(summary.generated, 2);
    assert_eq!(memory.count_docs().unwrap(), 2);
}

#[test]
fn test_staleness_detector_verify_and_clean_orphaned_docs() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let docs_root = repo_root.join(".agent-docs");
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();
    let generator = DocGenerator::new(&repo_root, &docs_root);

    let src_path = "src/temp_deleted.rs";
    let abs_src = repo_root.join(src_path);
    fs::write(&abs_src, "pub fn temp() {}").unwrap();

    let mut file_rec = FileRecord::new(src_path, 16);
    file_rec.source_hash = Some("h_temp".to_string());
    let fid = memory.upsert_file(&file_rec).unwrap();
    let file_rec_with_id = memory.get_file_by_id(fid).unwrap().unwrap();

    generator.generate_file_doc(&file_rec_with_id, &memory, &DocGenerateOptions::default()).unwrap();

    let doc_abs_path = docs_root.join("src/temp_deleted.md");
    assert!(doc_abs_path.exists());
    assert_eq!(memory.count_docs().unwrap(), 1);

    // Delete source file on disk (creates orphan)
    fs::remove_file(&abs_src).unwrap();

    // 1. Audit without clean
    let audit_dry = StalenessDetector::audit_and_clean(&repo_root, &docs_root, &memory, false).unwrap();
    assert_eq!(audit_dry.orphaned_docs, 1);
    assert_eq!(audit_dry.cleaned_docs, 0);
    assert!(doc_abs_path.exists()); // Still exists

    // 2. Audit with clean = true
    let audit_clean = StalenessDetector::audit_and_clean(&repo_root, &docs_root, &memory, true).unwrap();
    assert_eq!(audit_clean.orphaned_docs, 1);
    assert_eq!(audit_clean.cleaned_docs, 1);

    // Verify orphaned file removed from disk and deleted from DB
    assert!(!doc_abs_path.exists());
    assert_eq!(memory.count_docs().unwrap(), 0);
}

#[test]
fn test_used_by_population_in_rendered_documentation() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let docs_root = repo_root.join(".agent-docs");
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();
    let generator = DocGenerator::new(&repo_root, &docs_root);

    // 1. Setup src/db.rs
    let db_code = "pub struct Connection;";
    let abs_db = repo_root.join("src/db.rs");
    fs::write(&abs_db, db_code).unwrap();

    let db_file = FileRecord::new("src/db.rs", db_code.len());
    let id_db = memory.upsert_file(&db_file).unwrap();
    let db_file_with_id = memory.get_file_by_id(id_db).unwrap().unwrap();

    // 2. Setup src/user.rs that depends on src/db.rs
    let user_code = "use crate::db::Connection;";
    let abs_user = repo_root.join("src/user.rs");
    fs::write(&abs_user, user_code).unwrap();

    let user_file = FileRecord::new("src/user.rs", user_code.len());
    let id_user = memory.upsert_file(&user_file).unwrap();

    // Register import dependency
    let conn = memory.db().connection();
    conn.execute(
        "INSERT INTO import_raw (file_id, raw_import, resolved_file_id, status) VALUES (?1, ?2, ?3, 'resolved');",
        rusqlite::params![id_user, "use crate::db::Connection;", id_db],
    ).unwrap();

    // Generate doc for src/db.rs
    let res = generator.generate_file_doc(&db_file_with_id, &memory, &DocGenerateOptions::default()).unwrap();
    assert_eq!(res.relative_doc_path, "src/db.md");

    // Read generated markdown
    let doc_content = fs::read_to_string(docs_root.join("src/db.md")).unwrap();
    assert!(doc_content.contains("## Used By"));
    assert!(doc_content.contains("- `src/user.rs`"));
}
