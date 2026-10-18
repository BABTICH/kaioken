use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

use kaioken::agent::tools::bash::BashTool;
use kaioken::agent::tools::create_file::CreateFileTool;
use kaioken::agent::tools::edit_file::EditFileTool;
use kaioken::agent::tools::read_file::ReadFileTool;
use kaioken::agent::tools::{Tool, ToolContext, ToolRegistry};
use kaioken::agent::AgentLoop;
use kaioken::config::{Config, PermissionMode};
use kaioken::llm::{ChatChoice, ChatCompletionResponse, ChatMessage, FunctionCall, MockLlmClient, ToolCall};
use kaioken::memory::MemoryEngine;
use kaioken::safety::approval::{AutoApprovalHandler, DenyApprovalHandler, MockApprovalHandler};
use kaioken::safety::audit::AuditLogger;
use kaioken::safety::redact::RedactionEngine;
use kaioken::safety::{PermissionDecision, SafetyPolicy};

#[tokio::test]
async fn test_permission_mode_read_only() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Create a seed file
    let test_file = root.join("src").join("lib.rs");
    fs::create_dir_all(test_file.parent().unwrap()).unwrap();
    fs::write(&test_file, "pub fn hello() -> &'static str { \"world\" }").unwrap();

    let audit = AuditLogger::new(Some(&root));
    let policy = Arc::new(SafetyPolicy::new(
        root.clone(),
        PermissionMode::ReadOnly,
        false,
        false,
        vec!["cargo check".to_string()],
        vec!["rm -rf".to_string()],
        Some(Arc::new(DenyApprovalHandler)),
        Some(audit.clone()),
    ));

    let ctx = ToolContext::new(root.clone(), policy.command_guard.clone(), audit.clone())
        .with_safety_policy(policy);

    // 1. read_file is allowed in ReadOnly mode
    let read_tool = ReadFileTool;
    let read_res = read_tool
        .execute(r#"{"path":"src/lib.rs"}"#, &ctx)
        .await
        .unwrap();
    assert!(read_res.success);
    assert!(read_res.output.contains("pub fn hello"));

    // 2. create_file is forbidden in ReadOnly mode
    let create_tool = CreateFileTool;
    let create_res = create_tool
        .execute(r#"{"path":"src/new.rs","content":"// new"}"#, &ctx)
        .await
        .unwrap();
    assert!(!create_res.success);
    assert!(create_res.output.contains("strictly forbidden under 'read-only'"));
    assert!(!root.join("src").join("new.rs").exists());

    // 3. edit_file is forbidden in ReadOnly mode
    let edit_tool = EditFileTool;
    let edit_res = edit_tool
        .execute(
            r#"{"path":"src/lib.rs","target_content":"\"world\"","replacement_content":"\"kaioken\""}"#,
            &ctx,
        )
        .await
        .unwrap();
    assert!(!edit_res.success);
    assert!(edit_res.output.contains("strictly forbidden under 'read-only'"));

    // 4. bash is forbidden in ReadOnly mode
    let bash_tool = BashTool;
    let bash_res = bash_tool
        .execute(r#"{"command":"cargo check"}"#, &ctx)
        .await
        .unwrap();
    assert!(!bash_res.success);
    assert!(bash_res.output.contains("strictly forbidden under 'read-only'"));
}

#[tokio::test]
async fn test_permission_mode_edit_with_approval_and_mock_handler() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let test_file = root.join("src").join("app.rs");
    fs::create_dir_all(test_file.parent().unwrap()).unwrap();
    fs::write(&test_file, "const VERSION: &str = \"1.0\";").unwrap();

    let audit = AuditLogger::new(Some(&root));

    // Case A: Mock handler approves first request, denies second request
    let mock_handler = Arc::new(MockApprovalHandler::with_responses(vec![true, false]));
    let policy = Arc::new(SafetyPolicy::new(
        root.clone(),
        PermissionMode::EditWithApproval,
        false,
        false,
        vec!["cargo test".to_string()],
        vec!["rm -rf".to_string()],
        Some(mock_handler),
        Some(audit.clone()),
    ));

    let ctx = ToolContext::new(root.clone(), policy.command_guard.clone(), audit.clone())
        .with_safety_policy(policy);

    let edit_tool = EditFileTool;

    // 1st edit is approved
    let edit_res1 = edit_tool
        .execute(
            r#"{"path":"src/app.rs","target_content":"\"1.0\"","replacement_content":"\"1.1\""}"#,
            &ctx,
        )
        .await
        .unwrap();
    assert!(edit_res1.success);
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "const VERSION: &str = \"1.1\";");

    // 2nd edit is denied
    let edit_res2 = edit_tool
        .execute(
            r#"{"path":"src/app.rs","target_content":"\"1.1\"","replacement_content":"\"2.0\""}"#,
            &ctx,
        )
        .await
        .unwrap();
    assert!(!edit_res2.success);
    assert!(edit_res2.output.contains("denied by approval policy"));
    // File content remains 1.1
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "const VERSION: &str = \"1.1\";");
}

#[tokio::test]
async fn test_permission_mode_auto_edit_allows_mutations_prompts_bash() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let audit = AuditLogger::new(Some(&root));
    // Mock handler denies bash requests
    let mock_handler = Arc::new(MockApprovalHandler::with_responses(vec![false]));
    let policy = Arc::new(SafetyPolicy::new(
        root.clone(),
        PermissionMode::AutoEdit,
        false,
        false,
        vec!["cargo check".to_string()],
        vec![],
        Some(mock_handler),
        Some(audit.clone()),
    ));

    let ctx = ToolContext::new(root.clone(), policy.command_guard.clone(), audit)
        .with_safety_policy(policy);

    // Create file succeeds automatically without prompt
    let create_tool = CreateFileTool;
    let res = create_tool
        .execute(r#"{"path":"config.json","content":"{}"}"#, &ctx)
        .await
        .unwrap();
    assert!(res.success);
    assert!(root.join("config.json").exists());

    // Bash requires prompt; mock handler returns false -> blocked
    let bash_tool = BashTool;
    let bash_res = bash_tool
        .execute(r#"{"command":"cargo test"}"#, &ctx)
        .await
        .unwrap();
    assert!(!bash_res.success);
    assert!(bash_res.output.contains("denied by approval policy"));
}

#[tokio::test]
async fn test_permission_mode_safe_bash_allowlist_and_blocklist() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let audit = AuditLogger::new(Some(&root));
    let mock_handler = Arc::new(MockApprovalHandler::with_responses(vec![false]));
    let policy = Arc::new(SafetyPolicy::new(
        root.clone(),
        PermissionMode::SafeBash,
        false,
        false,
        vec!["git status".to_string(), "cargo check".to_string()],
        vec!["git reset --hard".to_string()],
        Some(mock_handler),
        Some(audit.clone()),
    ));

    // 1. Allowlisted command is allowed
    let dec_allowed = policy.check_action("bash", "git status", "").unwrap();
    assert_eq!(dec_allowed, PermissionDecision::Allowed);

    // 2. Non-allowlisted command triggers prompt (mock returns false -> denied)
    let dec_prompt = policy.check_action("bash", "npm install", "").unwrap();
    assert!(matches!(dec_prompt, PermissionDecision::Denied(_)));

    // 3. Blocklisted command is unconditionally blocked
    let dec_blocklist = policy.check_action("bash", "git reset --hard HEAD", "").unwrap();
    assert!(matches!(dec_blocklist, PermissionDecision::Denied(msg) if msg.contains("safety blocklist")));
}

#[tokio::test]
async fn test_dry_run_simulation_mode_creates_no_disk_side_effects() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let test_file = root.join("src").join("main.rs");
    fs::create_dir_all(test_file.parent().unwrap()).unwrap();
    fs::write(&test_file, "fn main() { println!(\"old\"); }").unwrap();

    let audit = AuditLogger::new(Some(&root));
    let policy = Arc::new(SafetyPolicy::new(
        root.clone(),
        PermissionMode::FullAuto,
        true, // dry_run = true
        false,
        vec![],
        vec![],
        Some(Arc::new(AutoApprovalHandler)),
        Some(audit.clone()),
    ));

    let ctx = ToolContext::new(root.clone(), policy.command_guard.clone(), audit)
        .with_safety_policy(policy);

    // 1. Dry run create_file
    let create_tool = CreateFileTool;
    let create_res = create_tool
        .execute(r#"{"path":"src/dry_file.rs","content":"pub struct Dry;"}"#, &ctx)
        .await
        .unwrap();
    assert!(create_res.success);
    assert!(create_res.output.contains("[DRY-RUN SIMULATION]"));
    assert!(!root.join("src").join("dry_file.rs").exists());

    // 2. Dry run edit_file
    let edit_tool = EditFileTool;
    let edit_res = edit_tool
        .execute(
            r#"{"path":"src/main.rs","target_content":"\"old\"","replacement_content":"\"new\""}"#,
            &ctx,
        )
        .await
        .unwrap();
    assert!(edit_res.success);
    assert!(edit_res.output.contains("[DRY-RUN SIMULATION]"));
    // Disk content unchanged!
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "fn main() { println!(\"old\"); }");

    // 3. Dry run bash
    let bash_tool = BashTool;
    let bash_res = bash_tool
        .execute(r#"{"command":"cargo check"}"#, &ctx)
        .await
        .unwrap();
    assert!(bash_res.success);
    assert!(bash_res.output.contains("[DRY-RUN SIMULATION]"));
}

#[tokio::test]
async fn test_yes_flag_bypasses_interactive_approval() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let audit = AuditLogger::new(Some(&root));
    // DenyApprovalHandler would deny if called, but yes = true bypasses it!
    let policy = Arc::new(SafetyPolicy::new(
        root.clone(),
        PermissionMode::EditWithApproval,
        false,
        true, // yes = true
        vec![],
        vec![],
        Some(Arc::new(DenyApprovalHandler)),
        Some(audit.clone()),
    ));

    let ctx = ToolContext::new(root.clone(), policy.command_guard.clone(), audit)
        .with_safety_policy(policy);

    let create_tool = CreateFileTool;
    let res = create_tool
        .execute(r#"{"path":"test.txt","content":"hello yes"}"#, &ctx)
        .await
        .unwrap();
    assert!(res.success);
    assert!(root.join("test.txt").exists());
}

#[tokio::test]
async fn test_enterprise_secret_redaction_and_sensitive_files() {
    let redactor = RedactionEngine::new();

    let text_with_secrets = "
        OPENROUTER_KEY=sk-or-v1-abcdef1234567890abcdef1234567890
        AWS_SECRET_ACCESS_KEY=1234567890abcdef1234567890abcdef12345678
        GITHUB_TOKEN=ghp_abcdef1234567890abcdef1234567890123456
        standalone_token: ghp_standalone12345678901234567890123456
        AUTHORIZATION=Bearer abcdef1234567890abcdef1234567890.eyJhbGciOiJIUzI1NiJ9
        -----BEGIN RSA PRIVATE KEY-----
        MIIEowIBAAKCAQEA0+
        -----END RSA PRIVATE KEY-----
    ";

    let redacted = redactor.redact(text_with_secrets);
    assert!(!redacted.contains("sk-or-v1-abcdef"));
    assert!(!redacted.contains("ghp_abcdef"));
    assert!(!redacted.contains("ghp_standalone"));
    assert!(redacted.contains("[REDACTED_SECRET]"));
    assert!(redacted.contains("[REDACTED_GITHUB_TOKEN]"));
    assert!(redacted.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(redacted.contains("[REDACTED_BEARER_TOKEN]"));

    assert!(RedactionEngine::is_sensitive_file(std::path::Path::new(".env")));
    assert!(RedactionEngine::is_sensitive_file(std::path::Path::new("credentials.json")));
    assert!(!RedactionEngine::is_sensitive_file(std::path::Path::new("src/main.rs")));
}

#[tokio::test]
async fn test_agent_loop_with_safety_policy_integration() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    let db_path = root.join(".agent").join("memory.sqlite");
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let memory = Arc::new(MemoryEngine::open(&db_path).unwrap());

    let mut config = Config::default_config();
    config.permissions.mode = PermissionMode::ReadOnly;

    let audit = AuditLogger::new(Some(&root));
    let policy = Arc::new(SafetyPolicy::new(
        root.clone(),
        PermissionMode::ReadOnly,
        false,
        false,
        vec![],
        vec![],
        Some(Arc::new(DenyApprovalHandler)),
        Some(audit),
    ));

    let mock_llm = Arc::new(MockLlmClient::new());
    // Simulate assistant trying to call create_file under ReadOnly mode
    let resp1 = ChatCompletionResponse {
        id: "call-1".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("I will create a file.".to_string()),
                Some(vec![ToolCall {
                    id: "fn-1".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "create_file".to_string(),
                        arguments: r#"{"path":"forbidden.rs","content":"pub struct X;"}"#.to_string(),
                    },
                }]),
            ),
            finish_reason: Some("tool_calls".to_string()),
        }],
        usage: None,
    };

    let resp2 = ChatCompletionResponse {
        id: "call-2".to_string(),
        choices: vec![ChatChoice {
            index: Some(0),
            message: ChatMessage::assistant(
                Some("File creation was blocked by policy. Task complete.".to_string()),
                None,
            ),
            finish_reason: Some("stop".to_string()),
        }],
        usage: None,
    };

    mock_llm.push_response(resp1);
    mock_llm.push_response(resp2);

    let tools = ToolRegistry::default_registry();
    let agent_loop = AgentLoop::new(tools, mock_llm, Some(memory), root.clone(), config)
        .with_safety_policy(policy)
        .with_retrieval(false);

    let summary = agent_loop.execute_task("Create forbidden file", None).await.unwrap();

    assert_eq!(summary.status, "completed");
    assert_eq!(summary.tool_calls_count, 1);
    assert!(!root.join("forbidden.rs").exists());
}
