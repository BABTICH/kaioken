use crate::error::Result;
use super::Tool;

pub struct GlobTool;

impl Tool for GlobTool {
    fn name(&self) -> &'static str {
        "glob_search"
    }

    fn description(&self) -> &'static str {
        "Find file paths matching pattern"
    }

    fn execute(&self, _args: &str) -> Result<String> {
        Ok("glob_search stub".to_string())
    }
}
