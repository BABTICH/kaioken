use crate::error::Result;
use super::Tool;

pub struct GrepTool;

impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep_search"
    }

    fn description(&self) -> &'static str {
        "Search codebase using ripgrep pattern matching"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("grep_search stub".to_string())
    }
}
