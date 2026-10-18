use serde::{Deserialize, Serialize};

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
                    tool_hint: Some("memory_query".to_string()),
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
        }
    }
}
