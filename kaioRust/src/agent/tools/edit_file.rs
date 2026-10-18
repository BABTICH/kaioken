use crate::error::Result;
use super::Tool;

pub struct EditFileTool;

impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Apply search-and-replace modification to a file"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("edit_file stub".to_string())
    }
}
