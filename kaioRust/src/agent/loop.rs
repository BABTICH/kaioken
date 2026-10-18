use crate::error::Result;
use super::tools::ToolRegistry;

pub struct AgentLoop {
    tools: ToolRegistry,
    max_steps: usize,
}

impl AgentLoop {
    pub fn new(tools: ToolRegistry) -> Self {
        Self {
            tools,
            max_steps: 15,
        }
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    pub async fn execute_task(&self, task: &str) -> Result<String> {
        // Stub for Phase 10: ReAct autonomous loop
        let plan = super::planner::Planner::create_plan(task);
        Ok(format!(
            "Executed task stub for: '{}'. Plan steps: {}",
            task,
            plan.steps.len()
        ))
    }
}
