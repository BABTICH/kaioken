use crate::error::Result;
use super::Tool;

pub struct BashTool;

impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn description(&self) -> &'static str {
        "Execute a terminal command within the workspace sandbox"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("bash stub".to_string())
    }
}
