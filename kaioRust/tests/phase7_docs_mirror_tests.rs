use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use kaioken::docs::{
    DocFrontmatter, DocGenerateOptions, DocGenerator, DocStalenessStatus,
    StalenessDetector,
};
use kaioken::memory::{DocRecord, FileRecord, MemoryEngine, SymbolRecord};
use kaioken::scanner::{AstParser, Language};

#[test]
fn test_doc_frontmatter_yaml_roundtrip_and_parsing() {
    let frontmatter = DocFrontmatter {
        source_path: "src/scanner/git.rs".to_string(),
        source_hash: "a1b2c3d4e5f67890123456789abcdef0".to_string(),
        doc_level: 0,
        generated_at: "2026-10-08T12:00:00Z".to_string(),
        generator: "deterministic".to_string(),
        status: "fresh".to_string(),
    };

    let yaml_header = frontmatter.to_yaml_header();
    assert!(yaml_header.starts_with("---\n"));
    assert!(yaml_header.ends_with("---\n\n"));
    assert!(yaml_header.contains("source_path: src/scanner/git.rs"));
    assert!(yaml_header.contains("source_hash: a1b2c3d4e5f67890123456789abcdef0"));
    assert!(yaml_header.contains("doc_level: 0"));
    assert!(yaml_header.contains("generator: deterministic"));
    assert!(yaml_header.contains("status: fresh"));

    let markdown_doc = format!("{}# Documentation: src/scanner/git.rs\n\n## Purpose\nGit tracker.", yaml_header);

    let (parsed_fm, body) = DocFrontmatter::parse_from_markdown(&markdown_doc)
        .expect("Should parse frontmatter successfully");

    assert_eq!(parsed_fm.source_path, frontmatter.source_path);
    assert_eq!(parsed_fm.source_hash, frontmatter.source_hash);
    assert_eq!(parsed_fm.doc_level, frontmatter.doc_level);
    assert_eq!(parsed_fm.generated_at, frontmatter.generated_at);
    assert_eq!(parsed_fm.generator, frontmatter.generator);
    assert_eq!(parsed_fm.status, frontmatter.status);

    assert!(body.starts_with("# Documentation: src/scanner/git.rs"));

    // Ensure non-frontmatter markdown returns None
    assert!(DocFrontmatter::parse_from_markdown("# Just a header").is_none());
    assert!(DocFrontmatter::parse_from_markdown("---\nincomplete yaml").is_none());
}

#[test]
fn test_sqlite_docs_crud_and_upsert() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("memory.db");
    let memory = MemoryEngine::open(&db_path).unwrap();

    // Insert source file record
    let mut file = FileRecord::new("src/main.rs", 450);
    file.language = Some("rust".to_string());
    file.source_hash = Some("hash12345".to_string());
    let file_id = memory.upsert_file(&file).unwrap();

    // Insert doc record
    let doc_record = DocRecord {
        id: None,
        file_id,
        doc_path: "src/main.md".to_string(),
        doc_level: 0,
        source_hash: Some("hash12345".to_string()),
        status: "fresh".to_string(),
        updated_at: "2026-10-08T10:05:00Z".to_string(),
        stale_reason: None,
    };
    let doc_id = memory.upsert_doc(&doc_record).unwrap();
    assert!(doc_id > 0);

    // Retrieve by file_id
    let retrieved_by_file = memory.get_doc_by_file_id(file_id).unwrap().unwrap();
    assert_eq!(retrieved_by_file.doc_path, "src/main.md");
    assert_eq!(retrieved_by_file.status, "fresh");
    assert_eq!(retrieved_by_file.source_hash.as_deref(), Some("hash12345"));

    // Retrieve by doc_path
    let retrieved_by_path = memory.get_doc_by_path("src/main.md").unwrap().unwrap();
    assert_eq!(retrieved_by_path.file_id, file_id);

    // Verify counts
    assert_eq!(memory.count_docs().unwrap(), 1);
    let status_counts = memory.count_docs_by_status().unwrap();
    assert_eq!(status_counts.get("fresh"), Some(&1));

    // Upsert update (conflict on file_id)
    let updated_doc = DocRecord {
        id: None,
        file_id,
        doc_path: "src/main.md".to_string(),
        doc_level: 1,
        source_hash: Some("hash67890".to_string()),
        status: "stale".to_string(),
        updated_at: "2026-10-08T11:00:00Z".to_string(),
        stale_reason: Some("Source file changed".to_string()),
    };
    memory.upsert_doc(&updated_doc).unwrap();

    // Ensure record was updated in place (total count still 1)
    assert_eq!(memory.count_docs().unwrap(), 1);
    let updated_retrieved = memory.get_doc_by_file_id(file_id).unwrap().unwrap();
    assert_eq!(updated_retrieved.doc_level, 1);
    assert_eq!(updated_retrieved.source_hash.as_deref(), Some("hash67890"));
    assert_eq!(updated_retrieved.status, "stale");
    assert_eq!(updated_retrieved.stale_reason, Some("Source file changed".to_string()));

    let updated_status_counts = memory.count_docs_by_status().unwrap();
    assert_eq!(updated_status_counts.get("stale"), Some(&1));
}

#[test]
fn test_doc_generator_mirror_path_calculation() {
    let repo_root = PathBuf::from("/repo");
    let docs_root = PathBuf::from("/repo/.agent-docs");
    let generator = DocGenerator::new(&repo_root, &docs_root);

    let src1 = Path::new("src/lib.rs");
    let mirror1 = generator.mirror_path_for_source(src1);
    assert_eq!(mirror1, docs_root.join("src").join("lib.md"));

    let src2 = Path::new("src/scanner/git.rs");
    let mirror2 = generator.mirror_path_for_source(src2);
    assert_eq!(mirror2, docs_root.join("src").join("scanner").join("git.md"));

    let src3 = Path::new("README.md");
    let mirror3 = generator.mirror_path_for_source(src3);
    assert_eq!(mirror3, docs_root.join("README.md"));
}

#[test]
fn test_doc_generator_file_doc_rendering() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let docs_root = repo_root.join(".agent-docs");
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();
    let generator = DocGenerator::new(&repo_root, &docs_root);

    // Create a rust source file
    let code = r#"// Authentication module
use std::collections::HashMap;

/// User session model
pub struct Session {
    pub token: String,
    pub user_id: u64,
}

impl Session {
    pub fn is_valid(&self) -> bool {
        !self.token.is_empty()
    }
}

pub fn create_session(user_id: u64) -> Session {
    Session {
        token: "tok_123".to_string(),
        user_id,
    }
}

fn internal_hash(s: &str) -> u64 {
    42
}
"#;
    let source_rel_path = "src/auth.rs";
    let source_abs_path = repo_root.join(source_rel_path);
    fs::write(&source_abs_path, code).unwrap();

    // Hash source
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let source_hash = format!("{:x}", hasher.finalize());

    // Register file in memory
    let mut file_rec = FileRecord::new(source_rel_path, code.len());
    file_rec.language = Some("rust".to_string());
    file_rec.source_hash = Some(source_hash.clone());
    let file_id = memory.upsert_file(&file_rec).unwrap();
    let file_rec_with_id = memory.get_file_by_id(file_id).unwrap().unwrap();

    // Extract AST symbols and save into memory
    let parsed_ast = AstParser::parse_source(code, Language::Rust).unwrap();
    let sym_records: Vec<SymbolRecord> = parsed_ast
        .symbols
        .into_iter()
        .map(|sym| SymbolRecord {
            id: None,
            file_id,
            name: sym.name,
            kind: sym.kind.as_str().to_string(),
            line_start: sym.line_start,
            line_end: sym.line_end,
            visibility: Some(sym.visibility),
            signature: sym.signature,
            summary: None,
            parent: sym.parent,
        })
        .collect();
    memory.insert_symbols(file_id, &sym_records).unwrap();

    // Save import into memory
    memory.insert_raw_imports(file_id, &parsed_ast.imports_raw).unwrap();

    // Generate documentation
    let options = DocGenerateOptions {
        level: 0,
        force: false,
        stale_only: false,
        no_llm: true,
        limit: None,
    };
    let res = generator.generate_file_doc(&file_rec_with_id, &memory, &options).unwrap();
    assert_eq!(res.file_id, file_id);
    assert_eq!(res.source_path, "src/auth.rs");
    assert_eq!(res.relative_doc_path, "src/auth.md");
    assert_eq!(res.doc_level, 0);
    assert!(res.symbols_count >= 4);
    assert_eq!(res.imports_count, 1);
    assert_eq!(res.status, "generated");

    // Verify mirrored file on disk
    let doc_abs_path = docs_root.join("src").join("auth.md");
    assert!(doc_abs_path.exists());
    let doc_content = fs::read_to_string(&doc_abs_path).unwrap();

    // Verify Frontmatter
    let (fm, body) = DocFrontmatter::parse_from_markdown(&doc_content).unwrap();
    assert_eq!(fm.source_path, "src/auth.rs");
    assert_eq!(fm.source_hash, source_hash);
    assert_eq!(fm.doc_level, 0);
    assert_eq!(fm.generator, "deterministic");
    assert_eq!(fm.status, "fresh");

    // Verify Body sections
    assert!(body.contains("# src/auth.rs"));
    assert!(body.contains("## Purpose"));
    assert!(body.contains("## Public API"));
    assert!(body.contains("Session"));
    assert!(body.contains("create_session"));
    assert!(body.contains("## Internal Symbols"));
    assert!(body.contains("internal_hash"));
    assert!(body.contains("## Dependencies"));
    assert!(body.contains("std::collections::HashMap"));

    // Verify DB entry
    let doc_db = memory.get_doc_by_file_id(file_id).unwrap().unwrap();
    assert_eq!(doc_db.status, "fresh");
    assert_eq!(doc_db.source_hash, Some(source_hash));
}

#[test]
fn test_doc_staleness_detector_hash_mismatch_and_missing() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let docs_root = repo_root.join(".agent-docs");
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();
    let generator = DocGenerator::new(&repo_root, &docs_root);

    let initial_code = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
    let source_rel_path = "src/math.rs";
    let source_abs_path = repo_root.join(source_rel_path);
    fs::write(&source_abs_path, initial_code).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(initial_code.as_bytes());
    let initial_hash = format!("{:x}", hasher.finalize());

    let mut file_rec = FileRecord::new(source_rel_path, initial_code.len());
    file_rec.language = Some("rust".to_string());
    file_rec.source_hash = Some(initial_hash.clone());
    let file_id = memory.upsert_file(&file_rec).unwrap();
    let file_rec_with_id = memory.get_file_by_id(file_id).unwrap().unwrap();

    let options = DocGenerateOptions {
        level: 0,
        force: false,
        stale_only: false,
        no_llm: true,
        limit: None,
    };
    generator.generate_file_doc(&file_rec_with_id, &memory, &options).unwrap();

    let doc_record = memory.get_doc_by_file_id(file_id).unwrap().unwrap();

    // 1. Initially fresh
    let check1 = StalenessDetector::verify_doc(&doc_record, &repo_root, &docs_root, &memory).unwrap();
    assert!(check1.is_valid);
    assert_eq!(check1.status, DocStalenessStatus::Fresh);

    // 2. Modify source file content (triggers HashMismatch)
    let modified_code = "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub fn sub(a: i32, b: i32) -> i32 { a - b }";
    fs::write(&source_abs_path, modified_code).unwrap();

    let check2 = StalenessDetector::verify_doc(&doc_record, &repo_root, &docs_root, &memory).unwrap();
    assert!(!check2.is_valid);
    match check2.status {
        DocStalenessStatus::HashMismatch { ref doc_hash, ref source_hash } => {
            assert_eq!(doc_hash, &initial_hash);
            assert_ne!(source_hash, &initial_hash);
        }
        other => panic!("Expected HashMismatch, got {:?}", other),
    }

    // 3. Delete doc markdown file on disk (triggers MissingDocFile)
    let doc_abs_path = docs_root.join(&doc_record.doc_path);
    fs::remove_file(&doc_abs_path).unwrap();

    let check3 = StalenessDetector::verify_doc(&doc_record, &repo_root, &docs_root, &memory).unwrap();
    assert!(!check3.is_valid);
    assert_eq!(check3.status, DocStalenessStatus::MissingDocFile);

    // 4. Delete source file (triggers MissingSource)
    fs::remove_file(&source_abs_path).unwrap();

    let check4 = StalenessDetector::verify_doc(&doc_record, &repo_root, &docs_root, &memory).unwrap();
    assert!(!check4.is_valid);
    assert_eq!(check4.status, DocStalenessStatus::MissingSource);
}

#[test]
fn test_batch_doc_generation_and_audit() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path().to_path_buf();
    let docs_root = repo_root.join(".agent-docs");
    let db_path = repo_root.join(".agent/memory.db");
    fs::create_dir_all(repo_root.join(".agent")).unwrap();
    fs::create_dir_all(repo_root.join("src")).unwrap();

    let memory = MemoryEngine::open(&db_path).unwrap();
    let generator = DocGenerator::new(&repo_root, &docs_root);

    // Create 3 files
    for i in 1..=3 {
        let path = format!("src/module_{}.rs", i);
        let content = format!("pub fn run_{}() -> usize {{ {} }}", i, i);
        let abs = repo_root.join(&path);
        fs::write(&abs, &content).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        let mut file_rec = FileRecord::new(&path, content.len());
        file_rec.language = Some("rust".to_string());
        file_rec.source_hash = Some(hash);
        memory.upsert_file(&file_rec).unwrap();
    }

    let options = DocGenerateOptions {
        level: 0,
        force: false,
        stale_only: false,
        no_llm: true,
        limit: None,
    };

    // First batch generation
    let summary1 = generator.generate_all_docs(&memory, &options).unwrap();
    assert_eq!(summary1.total_processed, 3);
    assert_eq!(summary1.generated, 3);
    assert_eq!(summary1.skipped, 0);

    // Audit report
    let audit1 = StalenessDetector::audit_all(&repo_root, &docs_root, &memory).unwrap();
    assert_eq!(audit1.total_docs, 3);
    assert_eq!(audit1.valid_docs, 3);
    assert_eq!(audit1.stale_docs, 0);
    assert_eq!(audit1.orphaned_docs, 0);

    // Second batch generation without changes: all 3 should be skipped as fresh
    let summary2 = generator.generate_all_docs(&memory, &options).unwrap();
    assert_eq!(summary2.total_processed, 3);
    assert_eq!(summary2.generated, 0);
    assert_eq!(summary2.skipped, 3);

    // Modify module_2.rs and re-audit
    let modified = "pub fn run_2() -> usize { 999 }";
    fs::write(repo_root.join("src/module_2.rs"), modified).unwrap();

    let audit2 = StalenessDetector::audit_all(&repo_root, &docs_root, &memory).unwrap();
    assert_eq!(audit2.total_docs, 3);
    assert_eq!(audit2.valid_docs, 2);
    assert_eq!(audit2.stale_docs, 1);
}
