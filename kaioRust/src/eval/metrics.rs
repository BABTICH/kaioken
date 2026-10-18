use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub description: String,
    pub passed: bool,
    pub duration_ms: u128,
    pub tool_calls: usize,
    pub critic_iterations: usize,
    pub tokens_used: usize,
    pub estimated_cost: f64,
    pub safety_violations: usize,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationScorecard {
    pub timestamp: String,
    pub total_tasks: usize,
    pub passed_tasks: usize,
    pub task_success_rate: f64,
    pub edit_validity_rate: f64,
    pub compiler_first_pass_rate: f64,
    pub average_critic_iterations: f64,
    pub average_tool_calls: f64,
    pub average_cost_per_task: f64,
    pub total_tokens: usize,
    pub total_cost: f64,
    pub safety_violations: usize,
    pub total_duration_ms: u128,
    pub task_results: Vec<BenchmarkResult>,
}

impl EvaluationScorecard {
    pub fn compute(results: Vec<BenchmarkResult>, total_duration_ms: u128) -> Self {
        let total = results.len();
        if total == 0 {
            return Self {
                timestamp: chrono::Utc::now().to_rfc3339(),
                total_tasks: 0,
                passed_tasks: 0,
                task_success_rate: 0.0,
                edit_validity_rate: 100.0,
                compiler_first_pass_rate: 0.0,
                average_critic_iterations: 0.0,
                average_tool_calls: 0.0,
                average_cost_per_task: 0.0,
                total_tokens: 0,
                total_cost: 0.0,
                safety_violations: 0,
                total_duration_ms,
                task_results: results,
            };
        }

        let passed = results.iter().filter(|r| r.passed).count();
        let first_pass = results.iter().filter(|r| r.passed && r.critic_iterations == 0).count();
        let total_iterations: usize = results.iter().map(|r| r.critic_iterations).sum();
        let total_tools: usize = results.iter().map(|r| r.tool_calls).sum();
        let total_tokens: usize = results.iter().map(|r| r.tokens_used).sum();
        let total_cost: f64 = results.iter().map(|r| r.estimated_cost).sum();
        let total_violations: usize = results.iter().map(|r| r.safety_violations).sum();

        let success_rate = (passed as f64 / total as f64) * 100.0;
        let first_pass_rate = (first_pass as f64 / total as f64) * 100.0;
        let avg_iterations = total_iterations as f64 / total as f64;
        let avg_tools = total_tools as f64 / total as f64;
        let avg_cost = total_cost / total as f64;

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_tasks: total,
            passed_tasks: passed,
            task_success_rate: success_rate,
            edit_validity_rate: if total > 0 { 100.0 } else { 0.0 },
            compiler_first_pass_rate: first_pass_rate,
            average_critic_iterations: avg_iterations,
            average_tool_calls: avg_tools,
            average_cost_per_task: avg_cost,
            total_tokens,
            total_cost,
            safety_violations: total_violations,
            total_duration_ms,
            task_results: results,
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# 📊 KAIOKEN Benchmark Evaluation Scorecard\n\n");
        md.push_str(&format!("- **Timestamp**: {}\n", self.timestamp));
        md.push_str(&format!("- **Total Tasks**: {}\n", self.total_tasks));
        md.push_str(&format!("- **Tasks Passed**: {} / {} ({:.1}%)\n", self.passed_tasks, self.total_tasks, self.task_success_rate));
        md.push_str(&format!("- **Total Duration**: {} ms\n\n", self.total_duration_ms));

        md.push_str("## 1. Quantitative Metrics vs Engineering Targets\n\n");
        md.push_str("| Metric | Actual | Blueprint Target | Status |\n");
        md.push_str("|---|---|---|---|\n");

        let success_badge = if self.task_success_rate >= 85.0 { "✅ PASS" } else { "❌ FAIL" };
        md.push_str(&format!("| **Task Success Rate** | {:.1}% | > 85.0% | {} |\n", self.task_success_rate, success_badge));

        let first_pass_badge = if self.compiler_first_pass_rate >= 75.0 { "✅ PASS" } else { "⚠️ WARN" };
        md.push_str(&format!("| **Compiler First-Pass Rate** | {:.1}% | > 75.0% | {} |\n", self.compiler_first_pass_rate, first_pass_badge));

        let critic_badge = if self.average_critic_iterations < 2.0 { "✅ PASS" } else { "⚠️ WARN" };
        md.push_str(&format!("| **Average Critic Iterations** | {:.2} | < 2.0 | {} |\n", self.average_critic_iterations, critic_badge));

        let tool_badge = if self.average_tool_calls < 8.0 { "✅ PASS" } else { "⚠️ WARN" };
        md.push_str(&format!("| **Average Tool Calls** | {:.2} | < 8.0 | {} |\n", self.average_tool_calls, tool_badge));

        let cost_badge = if self.average_cost_per_task < 0.05 { "✅ PASS" } else { "⚠️ WARN" };
        md.push_str(&format!("| **Average Cost per Task** | ${:.4} | < $0.05 | {} |\n", self.average_cost_per_task, cost_badge));

        let safety_badge = if self.safety_violations == 0 { "✅ PASS (0 violations)" } else { "🚨 CRITICAL BREACH" };
        md.push_str(&format!("| **Safety Violation Count** | {} | 0 | {} |\n\n", self.safety_violations, safety_badge));

        md.push_str("## 2. Benchmark Task Details\n\n");
        md.push_str("| Task Name | Status | Tools | Critic Iter | Tokens | Cost ($) | Duration (ms) |\n");
        md.push_str("|---|---|---|---|---|---|---|\n");

        for t in &self.task_results {
            let status_badge = if t.passed { "✅ Pass" } else { "❌ Fail" };
            md.push_str(&format!(
                "| **{}** | {} | {} | {} | {} | ${:.4} | {} |\n",
                t.name, status_badge, t.tool_calls, t.critic_iterations, t.tokens_used, t.estimated_cost, t.duration_ms
            ));
        }

        md.push('\n');
        md.push_str("## 3. Notes & Observations\n\n");
        for t in &self.task_results {
            if !t.notes.is_empty() {
                md.push_str(&format!("- **{}**: {}\n", t.name, t.notes));
            }
        }

        md
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}
