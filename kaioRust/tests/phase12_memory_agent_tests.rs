use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

use kaioken::agent::AgentLoop;
use kaioken::agent::planner::Planner;
use kaioken::agent::tools::ToolRegistry;
use kaioken::config::Config;
use kaioken::critic::engine::{CriticReport, MockCritic};
use kaioken::llm::{
    ChatChoice, ChatCompletionResponse, ChatMessage, FunctionCall, MockLlmClient, ToolCall,
};
use kaioken::memory::{DocRecord, FailureRecord, FileRecord, MemoryEngine, SymbolRecord};
use kaioken::retrieval::{
    ContextPacket, FailureMemorySnippet, FileContextSnippet, SymbolContextSummary,
};

#[test]
fn test_planner_cites_memory_symbols_and_past_failures() {
    let mut packet = ContextPacket::empty("Implement tax calculation", 8000);

    packet.relevant_symbols.push(SymbolContextSummary {
        name: "calculate_tax".to_string(),
        kind: "function".to_string(),
        file_path: "src/billing.rs".to_string(),
        signature: "pub fn calculate_tax(amount: f64) -> f64".to_string(),
        visibility: "pub".to_string(),
        line_start: 10,
        line_end: 25,
        token_estimate: 50,
    });

    packet.relevant_files.push(FileContextSnippet {
        path: "src/billing.rs".to_string(),
        language: "rust".to_string(),
        line_start: 1,
        line_end: 30,
        content: "pub fn calculate_tax(amount: f64) -> f64 { amount * 0.2 }\n".to_string(),
        source_hash: "hash123".to_string(),
        token_estimate: 60,
    });

    packet.related_failures.push(FailureMemorySnippet {
        error_type: "compiler".to_string(),
        message: "error[E0308]: mismatched types: expected f64, found i32".to_string(),
        file_path: Some("src/billing.rs".to_string()),
        symbol_name: Some("calculate_tax".to_string()),
        fix_summary: Some("Cast integer rate explicitly to f64".to_string()),
        created_at: "2026-10-18T10:00:00Z".to_string(),
    });

    packet.recommended_verification_commands.push("cargo test --test billing_tests".to_string());

    let plan = Planner::create_plan_with_context("Implement tax calculation with discount", &packet);

    // 1. Verify citations
    assert_eq!(plan.cited_symbols, vec!["calculate_tax"]);
    assert_eq!(plan.cited_files, vec!["src/billing.rs"]);
    assert_eq!(plan.cited_failures.len(), 1);
    assert!(plan.cited_failures[0].contains("E0308"));
    assert!(plan.cited_failures[0].contains("Cast integer rate"));

    // 2. Verify steps
    assert!(plan.steps.len() >= 4);
    assert!(plan.steps[0].description.contains("past repository failure"));
    assert!(plan.steps[1].description.contains("src/billing.rs"));
    assert!(plan.steps.last().unwrap().description.contains("cargo test --test billing_tests"));

    // 3. Verify Markdown formatting
    let md = plan.format_markdown();
    assert!(md.contains("Past Failure Cautions"));
    assert!(md.contains("Relevant Memory Symbols"));
    assert!(md.contains("`calculate_tax`"));
    assert!(md.contains("Target Files"));
    assert!(md.contains("Recommended Verification"));
}

#[tokio::test]
async fn test_agent_retrieves_context_packet_before_react_loop() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create real file in workspace
    let math_file = root.join("src/math.rs");
    fs::create_dir_all(math_file.parent().unwrap()).unwrap();
    fs::write(&math_file, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();

    let memory = Arc::new(MemoryEngine::open_in_memory().unwrap());
    let mut config = Config::default_config();
    config.llm.coder_model = "test-mock-model".to_string();

    // Populate memory with file and symbol
    let file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/math.rs".to_string(),
        language: Some("rust".to_string()),
        size: 45,
        source_hash: Some("hash_initial".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: Some("Math arithmetic functions".to_string()),
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    memory.insert_symbols(file_id, &[SymbolRecord {
        id: None,
        file_id,
        name: "add".to_string(),
        kind: "function".to_string(),
        line_start: 1,
        line_end: 1,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn add(a: i32, b: i32) -> i32".to_string()),
        summary: None,
        parent: None,
    }]).unwrap();

    let tools = ToolRegistry::default_registry();

    // Mock response concluding immediately
    let resp = ChatCompletionResponse {
        id: "step-1".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Inspected math.rs context and task is complete.".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    let mock_llm = Arc::new(MockLlmClient::with_responses(vec![resp]));
    let agent_loop = AgentLoop::new(tools, mock_llm.clone(), Some(memory.clone()), root, config)
        .with_max_steps(5);

    let summary = agent_loop
        .execute_task("Inspect add function in src/math.rs", None)
        .await
        .expect("Agent execution should succeed");

    assert_eq!(summary.status, "completed");
    assert!(summary.context_retrieved, "Memory context packet must be retrieved prior to loop");
}

#[tokio::test]
async fn test_agent_plan_explains_and_cites_past_failure() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let calc_file = root.join("src/calc.rs");
    fs::create_dir_all(calc_file.parent().unwrap()).unwrap();
    fs::write(&calc_file, "pub fn mul(a: u32, b: u32) -> u64 { (a * b) as u64 }\n").unwrap();

    let memory = Arc::new(MemoryEngine::open_in_memory().unwrap());
    let mut config = Config::default_config();
    config.llm.coder_model = "test-mock-model".to_string();

    let file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/calc.rs".to_string(),
        language: Some("rust".to_string()),
        size: 55,
        source_hash: Some("hash_calc".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: Some("Calculator module".to_string()),
        risk_level: None,
        doc_path: None,
        doc_status: None,
    }).unwrap();

    // Record past failure on this file
    memory.insert_failure(&FailureRecord {
        id: None,
        error_type: "compiler".to_string(),
        message: "error[E0308]: mismatched types: expected u64, found u32".to_string(),
        file_id: Some(file_id),
        symbol_id: None,
        fix_summary: Some("Cast multiplication result explicitly to u64 before returning".to_string()),
        status: Some("resolved".to_string()),
        resolved_at: Some("2026-10-18T12:00:00Z".to_string()),
        created_at: "2026-10-18T11:00:00Z".to_string(),
    }).unwrap();

    let tools = ToolRegistry::default_registry();

    let resp = ChatCompletionResponse {
        id: "step-1".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Noted past failure E0308 regarding u64 casting. Verified calculation.".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    let mock_llm = Arc::new(MockLlmClient::with_responses(vec![resp]));
    let agent_loop = AgentLoop::new(tools, mock_llm, Some(memory.clone()), root, config)
        .with_max_steps(5);

    let summary = agent_loop
        .execute_task("Review mul in src/calc.rs", None)
        .await
        .expect("Task should complete");

    assert_eq!(summary.status, "completed");
    assert!(summary.context_retrieved);
    assert!(summary.final_response.contains("E0308"));
}

#[tokio::test]
async fn test_post_task_memory_and_doc_staleness_synchronization() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let util_file = root.join("src/util.rs");
    fs::create_dir_all(util_file.parent().unwrap()).unwrap();
    fs::write(&util_file, "pub fn is_empty(s: &str) -> bool { false }\n").unwrap();

    let memory = Arc::new(MemoryEngine::open_in_memory().unwrap());
    let mut config = Config::default_config();
    config.llm.coder_model = "test-mock-model".to_string();

    let file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/util.rs".to_string(),
        language: Some("rust".to_string()),
        size: 42,
        source_hash: Some("old_hash_12345".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: None,
        risk_level: None,
        doc_path: Some(".agent-docs/src/util.md".to_string()),
        doc_status: Some("synced".to_string()),
    }).unwrap();

    memory.upsert_doc(&DocRecord {
        id: None,
        file_id,
        doc_path: ".agent-docs/src/util.md".to_string(),
        source_hash: Some("old_hash_12345".to_string()),
        doc_level: 0,
        status: "synced".to_string(),
        updated_at: "2026-10-18T00:00:00Z".to_string(),
        stale_reason: None,
    }).unwrap();

    let tools = ToolRegistry::default_registry();

    // Step 1: Agent calls edit_file on src/util.rs
    let step1_resp = ChatCompletionResponse {
        id: "step-1".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Fixing is_empty implementation".to_string()),
                Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "edit_file".to_string(),
                        arguments: r#"{"path": "src/util.rs", "target_content": "false", "replacement_content": "s.is_empty()"}"#.to_string(),
                    },
                }]),
            ),
            finish_reason: Some("tool_calls".to_string()),
        }],
        usage: None,
    };

    // Step 2: Agent concludes
    let step2_resp = ChatCompletionResponse {
        id: "step-2".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Updated is_empty to correctly check s.is_empty().".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    let mock_llm = Arc::new(MockLlmClient::with_responses(vec![step1_resp, step2_resp]));
    let agent_loop = AgentLoop::new(tools, mock_llm, Some(memory.clone()), root.clone(), config)
        .with_max_steps(5);

    let summary = agent_loop
        .execute_task("Fix is_empty in src/util.rs", None)
        .await
        .expect("Execution should succeed");

    assert_eq!(summary.status, "completed");
    assert!(summary.touched_files.contains(&"src/util.rs".to_string()));

    // Verify documentation staleness cascade was triggered:
    let doc = memory.get_doc_by_file_id(file_id).unwrap().expect("Doc must exist");
    assert_eq!(doc.status, "stale", "Doc status must cascade to 'stale' after agent modifications");

    // Verify file hash in files table was synchronized to new on-disk hash:
    let file_rec = memory.get_file_by_path("src/util.rs").unwrap().expect("File must exist");
    assert_ne!(file_rec.source_hash.as_deref(), Some("old_hash_12345"), "File hash must be updated to match edited content");
}

#[tokio::test]
async fn test_gate3_end_to_end_memory_augmented_agent_loop() {
    // =========================================================================
    // GATE 3 DOD TEST:
    // Full End-to-End Loop:
    // 1. Workspace contains source file, docs, past failure record.
    // 2. Pre-planning retrieval acquires context packet from SQLite memory.
    // 3. Agent inspects context, acknowledges past failure and symbols in plan.
    // 4. Agent executes edits cleanly.
    // 5. Critic evaluates and passes.
    // 6. Post-task updates agent_runs, tool_calls, doc staleness, and file hashes.
    // =========================================================================
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let source_file = root.join("src/service.rs");
    fs::create_dir_all(source_file.parent().unwrap()).unwrap();
    fs::write(&source_file, "pub fn get_status() -> &str {\n    \"pending\"\n}\n").unwrap();

    let memory = Arc::new(MemoryEngine::open_in_memory().unwrap());
    let mut config = Config::default_config();
    config.llm.coder_model = "test-mock-model".to_string();

    let file_id = memory.upsert_file(&FileRecord {
        id: None,
        path: "src/service.rs".to_string(),
        language: Some("rust".to_string()),
        size: 50,
        source_hash: Some("initial_service_hash".to_string()),
        parse_status: Some("parsed".to_string()),
        last_commit: None,
        last_modified: None,
        summary: Some("Core service module".to_string()),
        risk_level: None,
        doc_path: Some(".agent-docs/src/service.md".to_string()),
        doc_status: Some("synced".to_string()),
    }).unwrap();

    memory.upsert_doc(&DocRecord {
        id: None,
        file_id,
        doc_path: ".agent-docs/src/service.md".to_string(),
        source_hash: Some("initial_service_hash".to_string()),
        doc_level: 0,
        status: "synced".to_string(),
        updated_at: "2026-10-18T00:00:00Z".to_string(),
        stale_reason: None,
    }).unwrap();

    memory.insert_symbols(file_id, &[SymbolRecord {
        id: None,
        file_id,
        name: "get_status".to_string(),
        kind: "function".to_string(),
        line_start: 1,
        line_end: 3,
        visibility: Some("pub".to_string()),
        signature: Some("pub fn get_status() -> &str".to_string()),
        summary: None,
        parent: None,
    }]).unwrap();

    memory.insert_failure(&FailureRecord {
        id: None,
        error_type: "compiler".to_string(),
        message: "error[E0308]: mismatched types: expected &str, found String".to_string(),
        file_id: Some(file_id),
        symbol_id: None,
        fix_summary: Some("Return static string literal instead of allocated String".to_string()),
        status: Some("resolved".to_string()),
        resolved_at: Some("2026-10-18T10:00:00Z".to_string()),
        created_at: "2026-10-18T09:00:00Z".to_string(),
    }).unwrap();

    let tools = ToolRegistry::default_registry();

    // Step 1: Agent acknowledges context (symbols, past failure) and calls edit_file
    let step1_resp = ChatCompletionResponse {
        id: "step-1".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Context indicates past E0308 failure when returning allocated String. Keeping &str return type and updating status to 'ready'.".to_string()),
                Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "edit_file".to_string(),
                        arguments: r#"{"path": "src/service.rs", "target_content": "\"pending\"", "replacement_content": "\"ready\""}"#.to_string(),
                    },
                }]),
            ),
            finish_reason: Some("tool_calls".to_string()),
        }],
        usage: None,
    };

    // Step 2: Agent concludes task
    let step2_resp = ChatCompletionResponse {
        id: "step-2".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Status updated to 'ready' in src/service.rs adhering to &str signature.".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    let mock_llm = Arc::new(MockLlmClient::with_responses(vec![step1_resp, step2_resp]));
    let mock_critic = Arc::new(MockCritic::with_reports(vec![
        CriticReport::success("cargo check", 15),
    ]));

    let agent_loop = AgentLoop::new(tools, mock_llm, Some(memory.clone()), root.clone(), config)
        .with_critic(mock_critic)
        .with_max_steps(5);

    let summary = agent_loop
        .execute_task("Update service status in src/service.rs to ready", None)
        .await
        .expect("Task must succeed");

    // 1. Verify Agent Summary Metrics
    assert_eq!(summary.status, "completed");
    assert!(summary.context_retrieved, "Gate 3 requires context packet pre-retrieval");
    assert_eq!(summary.tool_calls_count, 1);
    assert_eq!(summary.touched_files, vec!["src/service.rs"]);

    // 2. Verify File on Disk
    let updated_content = fs::read_to_string(&source_file).unwrap();
    assert!(updated_content.contains("\"ready\""));

    // 3. Verify SQLite Memory Updates
    // A. agent_runs updated
    let calls = memory.get_tool_calls_for_run(summary.run_id).unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool_name, "edit_file");
    assert_eq!(calls[0].status, "success");

    // B. Documentation Staleness Cascaded
    let doc = memory.get_doc_by_file_id(file_id).unwrap().unwrap();
    assert_eq!(doc.status, "stale");

    // C. File hash synchronized
    let file = memory.get_file_by_path("src/service.rs").unwrap().unwrap();
    assert_ne!(file.source_hash.as_deref(), Some("initial_service_hash"));
}
