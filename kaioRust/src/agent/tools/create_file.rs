use crate::error::Result;
use super::Tool;

pub struct CreateFileTool;

impl Tool for CreateFileTool {
    fn name(&self) -> &'static str {
        "create_file"
    }

    fn description(&self) -> &'static str {
        "Create a new file in the workspace"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("create_file stub".to_string())
    }
}
