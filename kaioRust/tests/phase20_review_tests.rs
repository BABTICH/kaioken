use std::fs;
use tempfile::tempdir;

use kaioken::memory::{EdgeRecord, FileRecord, MemoryEngine, SymbolRecord};
use kaioken::review::{
    BreakingChange, DiffAnalyzer, ImpactAnalyzer, ReviewArgs, ReviewEngine, ReviewFileSummary,
    RiskAssessment, RiskFactor, RiskLevel,
};

#[test]
fn test_review_no_changes() {
    let memory = MemoryEngine::open_in_memory().expect("open memory");
    let dir = tempdir().expect("tempdir");
    let args = ReviewArgs::default();

    // When there are no changes or empty diff
    let report = ReviewEngine::review(dir.path(), &memory, &args).expect("review");
    assert!(report.changed_files.is_empty());
    assert!(report.breaking_changes.is_empty());
    assert_eq!(report.risk.level, RiskLevel::Low);

    let term = report.render_terminal();
    assert!(term.contains("No changes detected to review"));
}

#[test]
fn test_review_simple_edit() {
    let memory = MemoryEngine::open_in_memory().expect("open memory");
    let dir = tempdir().expect("tempdir");

    // File in memory
    let file_rec = FileRecord {
        id: None,
        path: "src/utils.rs".to_string(),
        language: Some("rust".to_string()),
        size: 500,
        source_hash: None,
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let fid = memory.upsert_file(&file_rec).expect("upsert file");

    let symbols = vec![SymbolRecord {
        id: None,
        file_id: fid,
        name: "internal_helper".to_string(),
        kind: "fn".to_string(),
        line_start: 10,
        line_end: 20,
        visibility: Some("priv".to_string()),
        signature: Some("fn internal_helper() -> i32".to_string()),
        summary: None,
        parent: None,
    }];
    memory
        .insert_symbols(fid, &symbols)
        .expect("insert symbols");

    // Unified diff modifying lines inside internal_helper
    let diff_text = r#"diff --git a/src/utils.rs b/src/utils.rs
index 1234567..89abcdef 100644
--- a/src/utils.rs
+++ b/src/utils.rs
@@ -12,3 +12,3 @@ fn internal_helper() -> i32 {
-    let x = 1;
+    let x = 2;
     x + 1
"#;

    let parsed = DiffAnalyzer::parse_unified_diff(diff_text);
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(parsed.files[0].path, "src/utils.rs");
    assert_eq!(parsed.files[0].additions, 1);
    assert_eq!(parsed.files[0].deletions, 1);

    let (affected, breaking) =
        DiffAnalyzer::match_symbols(dir.path(), &parsed, &memory).expect("match symbols");
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].symbol.name, "internal_helper");
    assert!(!affected[0].is_breaking);
    assert!(
        breaking.is_empty(),
        "Private function edit should not be breaking"
    );
}

#[test]
fn test_review_signature_change() {
    let memory = MemoryEngine::open_in_memory().expect("open memory");
    let dir = tempdir().expect("tempdir");

    let file_path = dir.path().join("src/parser.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    // New source on disk with altered signature (added param `lang: &str`)
    fs::write(
        &file_path,
        r#"pub fn extract_symbols(path: &str, lang: &str) -> Vec<String> {
    Vec::new()
}
"#,
    )
    .unwrap();

    let file_rec = FileRecord {
        id: None,
        path: "src/parser.rs".to_string(),
        language: Some("rust".to_string()),
        size: 200,
        source_hash: None,
        parse_status: Some("ok".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let fid = memory.upsert_file(&file_rec).expect("upsert file");

    let old_symbols = vec![SymbolRecord {
        id: None,
        file_id: fid,
        name: "extract_symbols".to_string(),
        kind: "fn".to_string(),
        line_start: 1,
        line_end: 5,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn extract_symbols(path: &str) -> Vec<String>".to_string()),
        summary: None,
        parent: None,
    }];
    memory
        .insert_symbols(fid, &old_symbols)
        .expect("insert symbols");

    let diff_text = r#"diff --git a/src/parser.rs b/src/parser.rs
index aaaaaaa..bbbbbbb 100644
--- a/src/parser.rs
+++ b/src/parser.rs
@@ -1,3 +1,3 @@
-pub fn extract_symbols(path: &str) -> Vec<String> {
+pub fn extract_symbols(path: &str, lang: &str) -> Vec<String> {
     Vec::new()
 }
"#;

    let parsed = DiffAnalyzer::parse_unified_diff(diff_text);
    let (affected, breaking) =
        DiffAnalyzer::match_symbols(dir.path(), &parsed, &memory).expect("match symbols");

    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].symbol.name, "extract_symbols");
    assert!(affected[0].is_breaking);

    assert_eq!(breaking.len(), 1);
    assert_eq!(breaking[0].symbol_name, "extract_symbols");
    assert_eq!(
        breaking[0].old_signature,
        "pub fn extract_symbols(path: &str) -> Vec<String>"
    );
    assert!(breaking[0]
        .new_signature
        .as_ref()
        .unwrap()
        .contains("lang: &str"));
    assert!(breaking[0].description.contains("Signature changed"));
}

#[test]
fn test_review_affected_callers() {
    let memory = MemoryEngine::open_in_memory().expect("open memory");

    // File with target symbol
    let file_target = FileRecord {
        id: None,
        path: "src/api.rs".to_string(),
        language: Some("rust".to_string()),
        size: 300,
        source_hash: None,
        parse_status: None,
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let fid_target = memory.upsert_file(&file_target).expect("upsert file");

    let target_syms = vec![SymbolRecord {
        id: None,
        file_id: fid_target,
        name: "execute_query".to_string(),
        kind: "fn".to_string(),
        line_start: 10,
        line_end: 25,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn execute_query()".to_string()),
        summary: None,
        parent: None,
    }];
    memory
        .insert_symbols(fid_target, &target_syms)
        .expect("insert symbols");
    let stored_target = memory.get_symbols_for_file(fid_target).unwrap();
    let target_sym_id = stored_target[0].id.unwrap();

    // File with caller symbol
    let file_caller = FileRecord {
        id: None,
        path: "src/service.rs".to_string(),
        language: Some("rust".to_string()),
        size: 600,
        source_hash: None,
        parse_status: None,
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: None,
        doc_status: None,
    };
    let fid_caller = memory.upsert_file(&file_caller).expect("upsert caller");

    let caller_syms = vec![SymbolRecord {
        id: None,
        file_id: fid_caller,
        name: "handle_request".to_string(),
        kind: "fn".to_string(),
        line_start: 50,
        line_end: 70,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn handle_request()".to_string()),
        summary: None,
        parent: None,
    }];
    memory
        .insert_symbols(fid_caller, &caller_syms)
        .expect("insert caller symbols");
    let stored_caller = memory.get_symbols_for_file(fid_caller).unwrap();
    let caller_sym_id = stored_caller[0].id.unwrap();

    // Insert edge: caller -> target
    let edge = EdgeRecord {
        id: None,
        source_symbol_id: Some(caller_sym_id),
        target_symbol_id: Some(target_sym_id),
        edge_type: "calls".to_string(),
    };
    memory.insert_edge(&edge).expect("insert edge");

    let affected_info = vec![kaioken::review::diff::AffectedSymbolInfo {
        symbol: stored_target[0].clone(),
        file_path: "src/api.rs".to_string(),
        is_breaking: true,
        old_signature: "pub fn execute_query()".to_string(),
        new_signature: None,
        breaking_reason: None,
    }];

    let callers =
        ImpactAnalyzer::find_affected_callers(&affected_info, &memory).expect("find callers");
    assert_eq!(callers.len(), 1);
    assert_eq!(callers[0].caller_file, "src/service.rs");
    assert_eq!(callers[0].caller_symbol.as_deref(), Some("handle_request"));
    assert_eq!(callers[0].caller_line, Some(50));
    assert_eq!(callers[0].target_symbol, "execute_query");
}

#[test]
fn test_review_test_gap() {
    let memory = MemoryEngine::open_in_memory().expect("open memory");
    let dir = tempdir().expect("tempdir");

    let changed = vec![
        ReviewFileSummary {
            path: "src/scanner/languages.rs".to_string(),
            additions: 10,
            deletions: 2,
            change_type: kaioken::scanner::git::ChangeType::Modified,
        },
        ReviewFileSummary {
            path: "tests/some_test.rs".to_string(),
            additions: 5,
            deletions: 0,
            change_type: kaioken::scanner::git::ChangeType::Added,
        },
    ];

    let gaps =
        ImpactAnalyzer::check_test_coverage(dir.path(), &changed, &memory).expect("check coverage");
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].file, "src/scanner/languages.rs");
    assert!(gaps[0].suggestion.contains("languages_tests.rs"));
}

#[test]
fn test_review_risk_scoring() {
    let factors = vec![
        RiskFactor::BreakingSignatureChange {
            symbol: "extract_symbols".to_string(),
            weight: 0.30,
        },
        RiskFactor::PublicApiChange {
            symbol: "Language".to_string(),
            weight: 0.25,
        },
        RiskFactor::NoTestCoverage {
            file: "src/languages.rs".to_string(),
            weight: 0.15,
        },
    ];

    let risk = RiskAssessment::calculate(factors);
    assert_eq!(risk.score, 0.70);
    assert_eq!(risk.level, RiskLevel::High);
    assert_eq!(risk.factors.len(), 3);
}

#[test]
fn test_review_json_output() {
    let report = kaioken::review::ReviewReport {
        changed_files: vec![ReviewFileSummary {
            path: "src/main.rs".to_string(),
            additions: 5,
            deletions: 1,
            change_type: kaioken::scanner::git::ChangeType::Modified,
        }],
        breaking_changes: vec![BreakingChange {
            file_path: "src/main.rs".to_string(),
            line_number: 10,
            symbol_name: "run".to_string(),
            old_signature: "pub fn run()".to_string(),
            new_signature: Some("pub fn run(verbose: bool)".to_string()),
            description: "Signature changed".to_string(),
            affected_callers_count: 2,
        }],
        affected_callers: Vec::new(),
        test_gaps: Vec::new(),
        stale_docs: Vec::new(),
        risk: RiskAssessment::calculate(vec![RiskFactor::BreakingSignatureChange {
            symbol: "run".to_string(),
            weight: 0.30,
        }]),
        recommendations: vec!["Update callers".to_string()],
        fix_suggestions: Vec::new(),
    };

    let json_str = serde_json::to_string_pretty(&report).expect("serialize report");
    assert!(json_str.contains("\"path\": \"src/main.rs\""));
    assert!(json_str.contains("\"symbol_name\": \"run\""));
    assert!(json_str.contains("\"score\": 0.3"));
    assert!(json_str.contains("\"level\": \"medium\""));

    let deserialized: kaioken::review::ReviewReport =
        serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(deserialized.changed_files.len(), 1);
    assert_eq!(deserialized.breaking_changes.len(), 1);
    assert_eq!(deserialized.risk.level, RiskLevel::Medium);
}

#[test]
fn test_review_terminal_rendering_complete() {
    let report = kaioken::review::ReviewReport {
        changed_files: vec![
            ReviewFileSummary {
                path: "src/scanner/tree_sitter.rs".to_string(),
                additions: 15,
                deletions: 3,
                change_type: kaioken::scanner::git::ChangeType::Modified,
            },
            ReviewFileSummary {
                path: "src/scanner/languages.rs".to_string(),
                additions: 5,
                deletions: 0,
                change_type: kaioken::scanner::git::ChangeType::Modified,
            },
        ],
        breaking_changes: vec![BreakingChange {
            file_path: "src/scanner/tree_sitter.rs".to_string(),
            line_number: 72,
            symbol_name: "extract_symbols".to_string(),
            old_signature: "fn extract_symbols(path: &Path)".to_string(),
            new_signature: Some("fn extract_symbols(path: &Path, lang: Language)".to_string()),
            description: "Signature changed: added parameter `lang: Language`".to_string(),
            affected_callers_count: 1,
        }],
        affected_callers: vec![kaioken::review::AffectedCaller {
            caller_file: "src/memory/mod.rs".to_string(),
            caller_symbol: Some("sync_inventory".to_string()),
            caller_line: Some(145),
            target_symbol: "extract_symbols".to_string(),
            target_file: "src/scanner/tree_sitter.rs".to_string(),
            edge_type: "calls".to_string(),
        }],
        test_gaps: vec![kaioken::review::TestGap {
            file: "src/scanner/languages.rs".to_string(),
            suggestion: "Create tests/scanner_languages_tests.rs".to_string(),
        }],
        stale_docs: vec![kaioken::review::StaleDocInfo {
            file_path: "src/scanner/tree_sitter.rs".to_string(),
            doc_path: ".agent-docs/src/scanner/tree_sitter.md".to_string(),
            reason: "will become STALE".to_string(),
        }],
        risk: RiskAssessment::calculate(vec![
            RiskFactor::PublicApiChange {
                symbol: "extract_symbols".to_string(),
                weight: 0.25,
            },
            RiskFactor::HighFanOutSymbol {
                symbol: "extract_symbols".to_string(),
                callers: 3,
                weight: 0.20,
            },
            RiskFactor::StaleDocumentation {
                doc_path: "1 doc".to_string(),
                weight: 0.10,
            },
        ]),
        recommendations: vec!["Update callers before committing.".to_string()],
        fix_suggestions: Vec::new(),
    };

    let term = report.render_terminal();
    assert!(term.contains("kaioken review — Change Analysis Report"));
    assert!(term.contains("Changed Files: 2"));
    assert!(term.contains("src/scanner/tree_sitter.rs"));
    assert!(term.contains("Breaking Changes (1)"));
    assert!(term.contains("extract_symbols"));
    assert!(term.contains("Affected callers (1)"));
    assert!(term.contains("src/memory/mod.rs:L145 (sync_inventory)"));
    assert!(term.contains("Test Gaps (1)"));
    assert!(term.contains("Documentation Impact (1)"));
    assert!(term.contains("Score: 0.55 / 1.00"));
    assert!(term.contains("Recommendation: Update callers before committing."));
}
