use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::config::PermissionMode;
use crate::error::Result;
use crate::safety::approval::{ApprovalHandler, ApprovalRequest, AutoApprovalHandler, InteractiveCliApprovalHandler};
use crate::safety::audit::AuditLogger;
use crate::safety::command_guard::CommandGuard;
use crate::safety::path_guard::PathGuard;
use crate::safety::redact::RedactionEngine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allowed,
    SimulatedDryRun,
    Denied(String),
}

pub struct SafetyPolicy {
    pub mode: PermissionMode,
    pub dry_run: bool,
    pub yes: bool,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
    pub approval_handler: Arc<dyn ApprovalHandler>,
    pub command_guard: Arc<CommandGuard>,
    pub audit_logger: AuditLogger,
    pub redactor: RedactionEngine,
    pub repo_root: PathBuf,
}

impl SafetyPolicy {
    pub fn new(
        repo_root: PathBuf,
        mode: PermissionMode,
        dry_run: bool,
        yes: bool,
        allowed_commands: Vec<String>,
        blocked_commands: Vec<String>,
        approval_handler: Option<Arc<dyn ApprovalHandler>>,
        audit_logger: Option<AuditLogger>,
    ) -> Self {
        let handler: Arc<dyn ApprovalHandler> = if let Some(h) = approval_handler {
            h
        } else if yes {
            Arc::new(AutoApprovalHandler)
        } else {
            Arc::new(InteractiveCliApprovalHandler)
        };

        let logger = audit_logger.unwrap_or_else(|| AuditLogger::new(Some(&repo_root)));
        let cmd_guard = Arc::new(CommandGuard::new(&blocked_commands));

        Self {
            mode,
            dry_run,
            yes,
            allowed_commands,
            blocked_commands,
            approval_handler: handler,
            command_guard: cmd_guard,
            audit_logger: logger,
            redactor: RedactionEngine::new(),
            repo_root,
        }
    }

    /// Evaluates whether an agent action is permitted under the current mode and flags.
    pub fn check_action(&self, action: &str, target: &str, details: &str) -> Result<PermissionDecision> {
        let decision = match action {
            // Read actions: permitted across all modes unless target is a secret file
            "read_file" | "grep" | "glob" => {
                if action == "read_file" {
                    let target_path = Path::new(target);
                    if let Err(e) = PathGuard::validate_path(target_path, &self.repo_root, false) {
                        PermissionDecision::Denied(e.to_string())
                    } else {
                        PermissionDecision::Allowed
                    }
                } else {
                    PermissionDecision::Allowed
                }
            }

            // Mutation actions: create_file, edit_file
            "create_file" | "edit_file" => {
                // First validate target path with PathGuard
                let target_path = Path::new(target);
                if let Err(e) = PathGuard::validate_path(target_path, &self.repo_root, true) {
                    PermissionDecision::Denied(e.to_string())
                } else if self.mode == PermissionMode::ReadOnly {
                    PermissionDecision::Denied(format!(
                        "File mutation ('{}') is strictly forbidden under 'read-only' permission mode.",
                        action
                    ))
                } else if self.dry_run {
                    PermissionDecision::SimulatedDryRun
                } else if self.mode == PermissionMode::EditWithApproval {
                    if self.yes {
                        PermissionDecision::Allowed
                    } else {
                        let req = ApprovalRequest::new(action, target, details);
                        match self.approval_handler.request_approval(&req)? {
                            true => PermissionDecision::Allowed,
                            false => PermissionDecision::Denied(format!(
                                "Action '{}' on '{}' was denied by approval policy or user rejection.",
                                action, target
                            )),
                        }
                    }
                } else {
                    // AutoEdit, SafeBash, FullAuto
                    PermissionDecision::Allowed
                }
            }

            // Terminal execution
            "bash" => {
                if self.mode == PermissionMode::ReadOnly {
                    PermissionDecision::Denied(
                        "Shell command execution ('bash') is strictly forbidden under 'read-only' permission mode.".to_string(),
                    )
                } else if let Some(reason) = self.command_guard.is_blocked(target) {
                    PermissionDecision::Denied(format!(
                        "Command execution blocked by safety blocklist: {}",
                        reason
                    ))
                } else if self.dry_run {
                    PermissionDecision::SimulatedDryRun
                } else {
                    match self.mode {
                        PermissionMode::FullAuto => PermissionDecision::Allowed,
                        PermissionMode::SafeBash => {
                            if self.is_command_in_allowlist(target) {
                                PermissionDecision::Allowed
                            } else if self.yes {
                                PermissionDecision::Allowed
                            } else {
                                let req = ApprovalRequest::new(action, target, details);
                                match self.approval_handler.request_approval(&req)? {
                                    true => PermissionDecision::Allowed,
                                    false => PermissionDecision::Denied(format!(
                                        "Command '{}' is outside allowlist and was denied by approval policy.",
                                        target
                                    )),
                                }
                            }
                        }
                        PermissionMode::EditWithApproval | PermissionMode::AutoEdit => {
                            if self.yes {
                                PermissionDecision::Allowed
                            } else {
                                let req = ApprovalRequest::new(action, target, details);
                                match self.approval_handler.request_approval(&req)? {
                                    true => PermissionDecision::Allowed,
                                    false => PermissionDecision::Denied(format!(
                                        "Command '{}' was denied by approval policy or user rejection.",
                                        target
                                    )),
                                }
                            }
                        }
                        PermissionMode::ReadOnly => unreachable!(),
                    }
                }
            }

            _ => PermissionDecision::Allowed,
        };

        // Record decision in audit logger
        let approved = matches!(decision, PermissionDecision::Allowed | PermissionDecision::SimulatedDryRun);
        let reason = match &decision {
            PermissionDecision::Allowed => None,
            PermissionDecision::SimulatedDryRun => Some("Dry-run simulation mode active"),
            PermissionDecision::Denied(r) => Some(r.as_str()),
        };
        let _ = self.audit_logger.log("agent", action, target, approved, reason);

        Ok(decision)
    }

    /// Checks if a command matches an allowlisted prefix or regex
    pub fn is_command_in_allowlist(&self, command: &str) -> bool {
        let trimmed = command.trim();
        for allowed in &self.allowed_commands {
            let allowed_trimmed = allowed.trim();
            if trimmed == allowed_trimmed || trimmed.starts_with(&format!("{} ", allowed_trimmed)) {
                return true;
            }
        }
        false
    }
}
