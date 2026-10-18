use crate::error::Result;
use super::Tool;

pub struct RunTestsTool;

impl Tool for RunTestsTool {
    fn name(&self) -> &'static str {
        "run_tests"
    }

    fn description(&self) -> &'static str {
        "Execute repository test suites and return parsed test outcomes"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("run_tests stub".to_string())
    }
}
