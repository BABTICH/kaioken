use std::fs;
use tempfile::TempDir;

use kaioken::cli::eval::{handle_eval_command, EvalCommands, EvalRunArgs};
use kaioken::eval::metrics::{BenchmarkResult, EvaluationScorecard};
use kaioken::eval::{BenchmarkRegistry, EvaluationRunner};

#[test]
fn test_evaluation_scorecard_metric_calculations() {
    let results = vec![
        BenchmarkResult {
            name: "task_1".to_string(),
            description: "First task".to_string(),
            passed: true,
            duration_ms: 100,
            tool_calls: 3,
            critic_iterations: 0,
            tokens_used: 200,
            estimated_cost: 0.0004,
            safety_violations: 0,
            notes: "Clean pass".to_string(),
        },
        BenchmarkResult {
            name: "task_2".to_string(),
            description: "Second task".to_string(),
            passed: true,
            duration_ms: 250,
            tool_calls: 5,
            critic_iterations: 1,
            tokens_used: 400,
            estimated_cost: 0.0008,
            safety_violations: 0,
            notes: "Corrected on 1st iteration".to_string(),
        },
        BenchmarkResult {
            name: "task_3".to_string(),
            description: "Third task".to_string(),
            passed: false,
            duration_ms: 300,
            tool_calls: 4,
            critic_iterations: 2,
            tokens_used: 600,
            estimated_cost: 0.0012,
            safety_violations: 0,
            notes: "Failed verification".to_string(),
        },
    ];

    let scorecard = EvaluationScorecard::compute(results, 650);

    assert_eq!(scorecard.total_tasks, 3);
    assert_eq!(scorecard.passed_tasks, 2);
    // Success rate: 2 / 3 = 66.666...%
    assert!((scorecard.task_success_rate - 66.66).abs() < 1.0);
    // First pass: 1 / 3 = 33.333...%
    assert!((scorecard.compiler_first_pass_rate - 33.33).abs() < 1.0);
    // Avg critic iterations: (0 + 1 + 2) / 3 = 1.0
    assert_eq!(scorecard.average_critic_iterations, 1.0);
    // Avg tool calls: (3 + 5 + 4) / 3 = 4.0
    assert_eq!(scorecard.average_tool_calls, 4.0);
    // Total tokens: 1200
    assert_eq!(scorecard.total_tokens, 1200);
    // Safety violations: 0
    assert_eq!(scorecard.safety_violations, 0);
}

#[tokio::test]
async fn test_evaluation_runner_executes_all_8_benchmarks() {
    let runner = EvaluationRunner::new();

    let scorecard = runner.run_all(None, None).await.unwrap();

    assert_eq!(scorecard.total_tasks, 8);
    assert_eq!(scorecard.passed_tasks, 8);
    assert_eq!(scorecard.task_success_rate, 100.0);
    assert_eq!(scorecard.safety_violations, 0);
    // Targets met
    assert!(scorecard.average_critic_iterations < 2.0);
    assert!(scorecard.average_tool_calls < 8.0);
    assert!(scorecard.average_cost_per_task < 0.05);

    // Verify task names
    let names: Vec<String> = scorecard.task_results.iter().map(|r| r.name.clone()).collect();
    assert!(names.contains(&"add_function".to_string()));
    assert!(names.contains(&"fix_compiler_error".to_string()));
    assert!(names.contains(&"fix_broken_test".to_string()));
    assert!(names.contains(&"cross_file_symbol_rename".to_string()));
    assert!(names.contains(&"add_unit_test".to_string()));
    assert!(names.contains(&"module_refactor".to_string()));
    assert!(names.contains(&"sync_mirrored_docs".to_string()));
    assert!(names.contains(&"impact_analysis".to_string()));
}

#[tokio::test]
async fn test_evaluation_runner_filtering() {
    let runner = EvaluationRunner::new();

    // Filter only rename task
    let scorecard = runner.run_all(Some("rename"), None).await.unwrap();

    assert_eq!(scorecard.total_tasks, 1);
    assert_eq!(scorecard.passed_tasks, 1);
    assert_eq!(scorecard.task_results[0].name, "cross_file_symbol_rename");
}

#[test]
fn test_scorecard_markdown_and_json_rendering() {
    let registry = BenchmarkRegistry::new();
    let tasks = registry.list();
    assert_eq!(tasks.len(), 8);

    let dummy_result = BenchmarkResult {
        name: "test_task".to_string(),
        description: "A test benchmark task".to_string(),
        passed: true,
        duration_ms: 45,
        tool_calls: 2,
        critic_iterations: 0,
        tokens_used: 150,
        estimated_cost: 0.0003,
        safety_violations: 0,
        notes: "Verified cleanly".to_string(),
    };

    let scorecard = EvaluationScorecard::compute(vec![dummy_result], 45);

    // Markdown rendering
    let md = scorecard.to_markdown();
    assert!(md.contains("# 📊 KAIOKEN Benchmark Evaluation Scorecard"));
    assert!(md.contains("| **Task Success Rate** | 100.0% | > 85.0% | ✅ PASS |"));
    assert!(md.contains("| **Safety Violation Count** | 0 | 0 | ✅ PASS (0 violations) |"));
    assert!(md.contains("test_task"));

    // JSON rendering & roundtrip
    let json_str = scorecard.to_json().unwrap();
    let parsed: EvaluationScorecard = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed.total_tasks, 1);
    assert_eq!(parsed.passed_tasks, 1);
    assert_eq!(parsed.task_results[0].name, "test_task");
}

#[tokio::test]
async fn test_cli_eval_run_command_file_output() {
    let temp = TempDir::new().unwrap();
    let report_file = temp.path().join("custom_eval_report.md");

    let cmd = EvalCommands::Run(EvalRunArgs {
        filter: Some("add_function".to_string()),
        output: Some(report_file.clone()),
        json: false,
    });

    handle_eval_command(cmd).await.unwrap();

    assert!(report_file.exists());
    let content = fs::read_to_string(&report_file).unwrap();
    assert!(content.contains("KAIOKEN Benchmark Evaluation Scorecard"));
    assert!(content.contains("add_function"));
}
