use crate::error::Result;
use super::Tool;

pub struct GitTool;

impl Tool for GitTool {
    fn name(&self) -> &'static str {
        "git"
    }

    fn description(&self) -> &'static str {
        "Execute git status, diff, or log operations"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("git tool stub".to_string())
    }
}
