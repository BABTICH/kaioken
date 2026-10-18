use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub action: String,
    pub target: String,
    pub details: String,
}

impl ApprovalRequest {
    pub fn new(action: impl Into<String>, target: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            target: target.into(),
            details: details.into(),
        }
    }
}

pub trait ApprovalHandler: Send + Sync {
    fn request_approval(&self, req: &ApprovalRequest) -> Result<bool>;
}

/// Automatically approves all requests. Used with `--yes` or in CI pipelines.
pub struct AutoApprovalHandler;

impl ApprovalHandler for AutoApprovalHandler {
    fn request_approval(&self, _req: &ApprovalRequest) -> Result<bool> {
        Ok(true)
    }
}

/// Automatically denies all requests requiring approval.
pub struct DenyApprovalHandler;

impl ApprovalHandler for DenyApprovalHandler {
    fn request_approval(&self, _req: &ApprovalRequest) -> Result<bool> {
        Ok(false)
    }
}

/// Interactive terminal approval handler that prompts the user on stdout/stdin.
pub struct InteractiveCliApprovalHandler;

impl ApprovalHandler for InteractiveCliApprovalHandler {
    fn request_approval(&self, req: &ApprovalRequest) -> Result<bool> {
        println!("\n⚠️  KAIOKEN Safety Approval Required:");
        println!("   Action:  {}", req.action);
        println!("   Target:  {}", req.target);
        if !req.details.is_empty() {
            println!("   Details: {}", req.details);
        }
        print!("   Approve this action? [y/N]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) if n > 0 => {
                let trimmed = input.trim().to_lowercase();
                Ok(trimmed == "y" || trimmed == "yes")
            }
            _ => Ok(false),
        }
    }
}

/// Mock handler for deterministic unit testing.
pub struct MockApprovalHandler {
    responses: Arc<Mutex<Vec<bool>>>,
    default_response: bool,
}

impl MockApprovalHandler {
    pub fn new(responses: Vec<bool>, default_response: bool) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            default_response,
        }
    }

    pub fn with_responses(responses: Vec<bool>) -> Self {
        Self::new(responses, false)
    }
}

impl ApprovalHandler for MockApprovalHandler {
    fn request_approval(&self, _req: &ApprovalRequest) -> Result<bool> {
        let mut list = self.responses.lock().unwrap();
        if !list.is_empty() {
            Ok(list.remove(0))
        } else {
            Ok(self.default_response)
        }
    }
}
