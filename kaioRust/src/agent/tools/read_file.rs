use crate::error::Result;
use super::Tool;

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read file contents or specific line ranges"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("read_file stub".to_string())
    }
}
