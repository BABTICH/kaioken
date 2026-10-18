use clap::Args;
use crate::error::Result;

#[derive(Args, Debug)]
pub struct TaskArgs {
    /// Task description or prompt for the autonomous agent
    pub prompt: String,
}

impl TaskArgs {
    pub async fn run(self) -> Result<()> {
        println!("Task execution for '{}' scheduled (Phase 10/11 feature).", self.prompt);
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct PlanArgs {
    /// Task goal to plan without applying changes
    pub prompt: String,
}

impl PlanArgs {
    pub fn run(self) -> Result<()> {
        let plan = crate::agent::Planner::create_plan(&self.prompt);
        println!("Generated Execution Plan for: '{}'\n", plan.goal);
        for step in plan.steps {
            println!("  {}. {}", step.step_number, step.description);
        }
        Ok(())
    }
}
