use serde::{Deserialize, Serialize};
use crate::retrieval::ContextPacket;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_number: usize,
    pub description: String,
    pub tool_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub cited_symbols: Vec<String>,
    pub cited_failures: Vec<String>,
    pub cited_files: Vec<String>,
    pub recommended_checks: Vec<String>,
}

impl TaskPlan {
    pub fn format_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("### Execution Plan: {}\n\n", self.goal));

        if !self.cited_failures.is_empty() {
            out.push_str("#### 🚨 Past Failure Cautions\n");
            for f in &self.cited_failures {
                out.push_str(&format!("- {}\n", f));
            }
            out.push('\n');
        }

        if !self.cited_symbols.is_empty() {
            out.push_str("#### 🧩 Relevant Memory Symbols\n");
            for sym in &self.cited_symbols {
                out.push_str(&format!("- `{}`\n", sym));
            }
            out.push('\n');
        }

        if !self.cited_files.is_empty() {
            out.push_str("#### 📄 Target Files\n");
            for file in &self.cited_files {
                out.push_str(&format!("- `{}`\n", file));
            }
            out.push('\n');
        }

        out.push_str("#### 📋 Steps\n");
        for step in &self.steps {
            let hint = match &step.tool_hint {
                Some(h) => format!(" (Tool: `{}`)", h),
                None => String::new(),
            };
            out.push_str(&format!("{}. {}{}\n", step.step_number, step.description, hint));
        }

        if !self.recommended_checks.is_empty() {
            out.push_str("\n#### 🧪 Recommended Verification\n");
            for check in &self.recommended_checks {
                out.push_str(&format!("- `{}`\n", check));
            }
            out.push('\n');
        }

        out
    }
}

pub struct Planner;

impl Planner {
    pub fn create_plan(goal: &str) -> TaskPlan {
        TaskPlan {
            goal: goal.to_string(),
            steps: vec![
                PlanStep {
                    step_number: 1,
                    description: "Retrieve relevant files and symbols from memory".to_string(),
                    tool_hint: Some("read_file".to_string()),
                },
                PlanStep {
                    step_number: 2,
                    description: "Inspect code and understand current state".to_string(),
                    tool_hint: Some("read_file".to_string()),
                },
                PlanStep {
                    step_number: 3,
                    description: "Execute edits and verify with deterministic compiler/test checks".to_string(),
                    tool_hint: Some("edit_file".to_string()),
                },
            ],
            cited_symbols: Vec::new(),
            cited_failures: Vec::new(),
            cited_files: Vec::new(),
            recommended_checks: Vec::new(),
        }
    }

    pub fn create_plan_with_context(goal: &str, context: &ContextPacket) -> TaskPlan {
        let mut steps = Vec::new();
        let mut step_num = 1;

        let cited_symbols: Vec<String> = context.relevant_symbols.iter().map(|s| s.name.clone()).collect();
        let cited_files: Vec<String> = context.relevant_files.iter().map(|f| f.path.clone()).collect();
        let cited_failures: Vec<String> = context.related_failures.iter().map(|f| {
            let loc = f.file_path.as_deref().unwrap_or("workspace");
            let fix = f.fix_summary.as_deref().unwrap_or("No previous fix recorded");
            format!("[{}] on `{}`: {} (Past Fix: {})", f.error_type, loc, f.message, fix)
        }).collect();

        // Step 1: Past failure cautions if any exist in context
        if !context.related_failures.is_empty() {
            steps.push(PlanStep {
                step_number: step_num,
                description: format!(
                    "Review {} known past repository failure(s) to avoid repeating past regressions.",
                    context.related_failures.len()
                ),
                tool_hint: None,
            });
            step_num += 1;
        }

        // Step 2: Inspect relevant files and symbols
        if !cited_files.is_empty() {
            steps.push(PlanStep {
                step_number: step_num,
                description: format!("Read and inspect target file(s): {}", cited_files.join(", ")),
                tool_hint: Some("read_file".to_string()),
            });
            step_num += 1;
        } else {
            steps.push(PlanStep {
                step_number: step_num,
                description: "Inspect target code and relevant dependencies".to_string(),
                tool_hint: Some("read_file".to_string()),
            });
            step_num += 1;
        }

        // Step 3: Implement modifications
        steps.push(PlanStep {
            step_number: step_num,
            description: format!("Apply implementation edits for: {}", goal),
            tool_hint: Some("edit_file".to_string()),
        });
        step_num += 1;

        // Step 4: Verification
        let verification_desc = if !context.recommended_verification_commands.is_empty() {
            format!("Run verification commands: {}", context.recommended_verification_commands.join(" && "))
        } else {
            "Run deterministic compiler and test verification".to_string()
        };
        steps.push(PlanStep {
            step_number: step_num,
            description: verification_desc,
            tool_hint: Some("critic".to_string()),
        });

        TaskPlan {
            goal: goal.to_string(),
            steps,
            cited_symbols,
            cited_failures,
            cited_files,
            recommended_checks: context.recommended_verification_commands.clone(),
        }
    }
}
