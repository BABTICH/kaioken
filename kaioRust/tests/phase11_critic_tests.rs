use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

use kaioken::agent::AgentLoop;
use kaioken::agent::tools::ToolRegistry;
use kaioken::config::Config;
use kaioken::critic::diagnostics::{CompilerDiagnostic, DiagnosticParser};
use kaioken::critic::engine::{Critic, CriticEngine, CriticReport, MockCritic};
use kaioken::llm::{
    ChatChoice, ChatCompletionResponse, ChatMessage, FunctionCall, MockLlmClient, ToolCall,
};
use kaioken::memory::{FailureRecord, MemoryEngine};

#[test]
fn test_diagnostic_parser_extracts_compiler_errors() {
    let raw_rustc_output = r#"
   Compiling kaioken v0.1.0 (D:\project\kaioken)
error[E0308]: mismatched types
  --> src/calc.rs:42:15
   |
42 |     let x: u32 = "abc";
   |            ---   ^^^^^ expected `u32`, found `&str`
   |            |
   |            expected due to this

warning: unused variable: `unused_var`
  --> src/calc.rs:10:9
   |
10 |     let unused_var = 123;
   |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_unused_var`

error[E0425]: cannot find value `calculate_sum` in this scope
  --> src/main.rs:18:5
   |
18 |     calculate_sum();
   |     ^^^^^^^^^^^^^ not found in this scope

thread 'main' panicked at 'assertion failed: `(left == right)`
  left: `42`,
 right: `100`', tests/unit_tests.rs:55:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
"#;

    let diagnostics = DiagnosticParser::parse(raw_rustc_output);
    assert_eq!(diagnostics.len(), 4);

    // 1. Diagnostic E0308
    assert_eq!(diagnostics[0].error_code.as_deref(), Some("E0308"));
    assert_eq!(diagnostics[0].severity, "error");
    assert_eq!(diagnostics[0].file_path.as_deref(), Some("src/calc.rs"));
    assert_eq!(diagnostics[0].line, Some(42));
    assert_eq!(diagnostics[0].col, Some(15));
    assert!(diagnostics[0].message.contains("mismatched types"));

    // 2. Diagnostic Warning
    assert_eq!(diagnostics[1].error_code, None);
    assert_eq!(diagnostics[1].severity, "warning");
    assert_eq!(diagnostics[1].file_path.as_deref(), Some("src/calc.rs"));
    assert_eq!(diagnostics[1].line, Some(10));
    assert_eq!(diagnostics[1].col, Some(9));
    assert!(diagnostics[1].message.contains("unused variable"));

    // 3. Diagnostic E0425
    assert_eq!(diagnostics[2].error_code.as_deref(), Some("E0425"));
    assert_eq!(diagnostics[2].severity, "error");
    assert_eq!(diagnostics[2].file_path.as_deref(), Some("src/main.rs"));
    assert_eq!(diagnostics[2].line, Some(18));
    assert_eq!(diagnostics[2].col, Some(5));

    // 4. Test Panic
    assert_eq!(diagnostics[3].severity, "panic");
    assert_eq!(diagnostics[3].file_path.as_deref(), Some("tests/unit_tests.rs"));
    assert_eq!(diagnostics[3].line, Some(55));
    assert_eq!(diagnostics[3].col, Some(9));
    assert!(diagnostics[3].message.contains("assertion failed"));

    // Formatted feedback block
    let feedback = DiagnosticParser::format_feedback_block(
        "cargo test",
        101,
        &diagnostics,
        raw_rustc_output,
    );
    assert!(feedback.contains("CRITIC FEEDBACK"));
    assert!(feedback.contains("cargo test"));
    assert!(feedback.contains("[E0308]"));
    assert!(feedback.contains("mismatched types"));
    assert!(feedback.contains("assertion failed"));
}

#[tokio::test]
async fn test_critic_engine_identifies_clean_build() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let clean_cmd = if cfg!(windows) { "cmd.exe /c exit 0" } else { "true" };
    let engine = CriticEngine::new().with_command(clean_cmd);

    let report = engine.evaluate(root).await.unwrap();
    assert!(report.passed);
    assert_eq!(report.exit_code, 0);
    assert!(report.diagnostics.is_empty());
    assert!(report.feedback_message.contains("passed successfully"));
}

#[tokio::test]
async fn test_critic_engine_identifies_broken_build() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let cmd_str = if cfg!(windows) {
        let bat = root.join("check.bat");
        fs::write(
            &bat,
            "@echo error[E0425]: cannot find value `foo_bar` in this scope\r\n@echo   --^> src/lib.rs:14:5\r\n@exit /b 1\r\n",
        ).unwrap();
        "check.bat".to_string()
    } else {
        let sh = root.join("check.sh");
        fs::write(
            &sh,
            "#!/bin/sh\necho \"error[E0425]: cannot find value `foo_bar` in this scope\"\necho \"  --> src/lib.rs:14:5\"\nexit 1\n",
        ).unwrap();
        "sh check.sh".to_string()
    };
    let engine = CriticEngine::new().with_command(cmd_str);

    let report = engine.evaluate(root).await.unwrap();
    assert!(!report.passed);
    assert_eq!(report.exit_code, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].error_code.as_deref(), Some("E0425"));
    assert_eq!(report.diagnostics[0].file_path.as_deref(), Some("src/lib.rs"));
    assert_eq!(report.diagnostics[0].line, Some(14));
    assert_eq!(report.diagnostics[0].col, Some(5));
}

#[test]
fn test_failure_memory_insert_and_resolve_lifecycle() {
    let memory = MemoryEngine::open_in_memory().unwrap();

    let unresolved_before = memory.get_unresolved_failures().unwrap();
    assert_eq!(unresolved_before.len(), 0);

    // 1. Insert an unresolved compiler failure record
    let record = FailureRecord {
        id: None,
        error_type: "compiler".to_string(),
        message: "error[E0308]: mismatched types in src/calc.rs".to_string(),
        file_id: None,
        symbol_id: None,
        fix_summary: None,
        status: Some("unresolved".to_string()),
        resolved_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let failure_id = memory.insert_failure(&record).unwrap();
    assert!(failure_id > 0);

    // 2. Query unresolved failures
    let unresolved = memory.get_unresolved_failures().unwrap();
    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0].id, Some(failure_id));
    assert_eq!(unresolved[0].status.as_deref(), Some("unresolved"));
    assert_eq!(unresolved[0].fix_summary, None);

    // 3. Resolve the failure with a fix summary
    let fix_note = "Changed parameter type from &str to u32 and updated call sites.";
    memory.resolve_failure(failure_id, fix_note).unwrap();

    // 4. Verify no unresolved failures remain
    let unresolved_after = memory.get_unresolved_failures().unwrap();
    assert_eq!(unresolved_after.len(), 0);

    // 5. Verify resolved record details
    let resolved_rec = memory.get_failure_by_id(failure_id).unwrap().expect("Record must exist");
    assert_eq!(resolved_rec.status.as_deref(), Some("resolved"));
    assert_eq!(resolved_rec.fix_summary.as_deref(), Some(fix_note));
    assert!(resolved_rec.resolved_at.is_some());
}

#[tokio::test]
async fn test_critic_blocks_premature_completion_on_compiler_error() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let memory = Arc::new(MemoryEngine::open_in_memory().unwrap());
    let mut config = Config::default_config();
    config.llm.coder_model = "test-mock-model".to_string();

    let tools = ToolRegistry::default_registry();

    // Step 1: Agent edits a file
    let step1_resp = ChatCompletionResponse {
        id: "step-1".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Creating broken file".to_string()),
                Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "create_file".to_string(),
                        arguments: r#"{"path": "src/broken.rs", "content": "fn bad() { 123 }"}"#.to_string(),
                    },
                }]),
            ),
            finish_reason: Some("tool_calls".to_string()),
        }],
        usage: None,
    };

    // Step 2: Agent tries to conclude prematurely without fixing
    let step2_resp = ChatCompletionResponse {
        id: "step-2".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("I am done with the file!".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    // Step 3: Agent tries to conclude again, critic fails again until max retries
    let step3_resp = ChatCompletionResponse {
        id: "step-3".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("I am still claiming I am done.".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    let mock_llm = Arc::new(MockLlmClient::with_responses(vec![
        step1_resp,
        step2_resp,
        step3_resp,
    ]));

    // Critic always fails
    let failing_diag = vec![CompilerDiagnostic {
        file_path: Some("src/broken.rs".to_string()),
        line: Some(1),
        col: Some(16),
        error_code: Some("E0308".to_string()),
        message: "mismatched types".to_string(),
        severity: "error".to_string(),
        snippet: None,
    }];
    let failing_report = CriticReport::failure(
        "cargo check",
        1,
        failing_diag,
        "error[E0308]: mismatched types\n  --> src/broken.rs:1:16".to_string(),
        12,
    );

    let mock_critic = Arc::new(MockCritic::with_reports(vec![
        failing_report.clone(),
        failing_report,
    ]));

    let agent_loop = AgentLoop::new(tools, mock_llm, Some(memory.clone()), root, config)
        .with_critic(mock_critic)
        .with_max_steps(5)
        .with_max_critic_retries(2);

    let summary = agent_loop
        .execute_task("Create broken file and finish", None)
        .await
        .expect("Agent execution should return summary");

    // Because critic never passed, status should be failed
    assert_eq!(summary.status, "failed");
    assert!(summary.critic_iterations >= 2);

    // Verify unresolved failure was recorded in memory
    let unresolved = memory.get_unresolved_failures().unwrap();
    assert!(!unresolved.is_empty());
    assert_eq!(unresolved[0].status.as_deref(), Some("unresolved"));
}

#[tokio::test]
async fn test_autonomous_agent_self_corrects_compiler_error_and_records_fix() {
    // =========================================================================
    // GATE 2 DOD TEST:
    // Autonomous self-correction loop:
    // 1. Agent modifies code (introducing error)
    // 2. Agent attempts to conclude task -> Critic rejects it with diagnostic
    // 3. Failure record stored in SQLite (status = 'unresolved')
    // 4. Feedback injected into conversation
    // 5. Agent inspects feedback, applies fix
    // 6. Agent concludes -> Critic re-evaluates and passes!
    // 7. Failure record marked 'resolved' with fix_summary
    // 8. Task completes successfully
    // =========================================================================
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let memory = Arc::new(MemoryEngine::open_in_memory().unwrap());
    let mut config = Config::default_config();
    config.llm.coder_model = "test-mock-model".to_string();

    let tools = ToolRegistry::default_registry();

    // Step 1: Agent writes initial implementation with a type mismatch error
    let step1_resp = ChatCompletionResponse {
        id: "step-1".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Writing calculator function in src/calc.rs".to_string()),
                Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "create_file".to_string(),
                        arguments: r#"{"path": "src/calc.rs", "content": "pub fn get_multiplier() -> u32 {\n    \"10\"\n}\n"}"#.to_string(),
                    },
                }]),
            ),
            finish_reason: Some("tool_calls".to_string()),
        }],
        usage: None,
    };

    // Step 2: Agent attempts to conclude task prematurely
    let step2_resp = ChatCompletionResponse {
        id: "step-2".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Implementation complete. Returning multiplier.".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    // Step 3: Agent receives critic feedback regarding E0308, and calls edit_file to fix it
    let step3_resp = ChatCompletionResponse {
        id: "step-3".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Critic reported type mismatch E0308: expected u32, found &str. Fixing to return 10.".to_string()),
                Some(vec![ToolCall {
                    id: "call_2".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "edit_file".to_string(),
                        arguments: r#"{"path": "src/calc.rs", "target_content": "\"10\"", "replacement_content": "10"}"#.to_string(),
                    },
                }]),
            ),
            finish_reason: Some("tool_calls".to_string()),
        }],
        usage: None,
    };

    // Step 4: Agent concludes task after applying fix
    let step4_resp = ChatCompletionResponse {
        id: "step-4".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("Self-correction applied and verified: get_multiplier() now returns integer 10.".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    let mock_llm = Arc::new(MockLlmClient::with_responses(vec![
        step1_resp,
        step2_resp,
        step3_resp,
        step4_resp,
    ]));

    // Critic sequence:
    // 1st evaluation -> FAIL with E0308
    // 2nd evaluation -> PASS
    let failing_diag = vec![CompilerDiagnostic {
        file_path: Some("src/calc.rs".to_string()),
        line: Some(2),
        col: Some(5),
        error_code: Some("E0308".to_string()),
        message: "mismatched types: expected `u32`, found `&str`".to_string(),
        severity: "error".to_string(),
        snippet: Some("\"10\"".to_string()),
    }];
    let fail_report = CriticReport::failure(
        "cargo check",
        1,
        failing_diag,
        "error[E0308]: mismatched types\n  --> src/calc.rs:2:5\n   |\n2 |     \"10\"\n   |     ^^^^ expected `u32`, found `&str`".to_string(),
        15,
    );
    let pass_report = CriticReport::success("cargo check", 20);

    let mock_critic = Arc::new(MockCritic::with_reports(vec![
        fail_report,
        pass_report,
    ]));

    let agent_loop = AgentLoop::new(tools, mock_llm, Some(memory.clone()), root.clone(), config)
        .with_critic(mock_critic)
        .with_max_steps(10)
        .with_max_critic_retries(3);

    let summary = agent_loop
        .execute_task("Implement get_multiplier() in src/calc.rs", None)
        .await
        .expect("Agent execution should succeed after self-correction");

    // 1. Verify summary metrics
    assert_eq!(summary.status, "completed");
    assert_eq!(summary.critic_iterations, 1);
    assert!(summary.final_response.contains("Self-correction applied and verified"));

    // 2. Verify file on disk has the corrected code
    let file_content = fs::read_to_string(root.join("src/calc.rs")).unwrap();
    assert!(file_content.contains("pub fn get_multiplier() -> u32"));
    assert!(file_content.contains("10"));
    assert!(!file_content.contains("\"10\""));

    // 3. Verify Gate 2 DOD in SQLite Failure Memory:
    // Unresolved failures count must be 0
    let unresolved = memory.get_unresolved_failures().unwrap();
    assert_eq!(unresolved.len(), 0, "No unresolved failures should remain after successful fix");

    // All recent failures should show status = 'resolved' with fix_summary populated
    let recent = memory.get_recent_failures(10).unwrap();
    assert_eq!(recent.len(), 1, "Exactly one failure record should be tracked");
    let failure_record = &recent[0];
    assert_eq!(failure_record.status.as_deref(), Some("resolved"));
    assert_eq!(failure_record.error_type, "compiler");
    assert!(failure_record.message.contains("cargo check"));
    assert!(failure_record.fix_summary.is_some());
    assert!(failure_record.resolved_at.is_some());
}
