use crate::config::PermissionMode;
use crate::safety::audit::AuditLogEntry;
use serde::{Deserialize, Serialize};

/// Policy tier for tool execution approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyTier {
    ReadOnly,
    WritePrompt,
    Dangerous,
}

impl std::fmt::Display for PolicyTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyTier::ReadOnly => write!(f, "READ_ONLY"),
            PolicyTier::WritePrompt => write!(f, "WRITE_PROMPT"),
            PolicyTier::Dangerous => write!(f, "DANGEROUS"),
        }
    }
}

/// A pending tool execution proposal from the agent needing review or approval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolProposal {
    pub id: String,
    pub tool_name: String,
    pub target: String,
    pub policy_tier: PolicyTier,
    pub details: String,
    pub diff: Option<String>,
    pub proposed_at: String,
}

impl ToolProposal {
    pub fn new_bash(id: impl Into<String>, command: impl Into<String>, is_dangerous: bool) -> Self {
        let cmd = command.into();
        Self {
            id: id.into(),
            tool_name: "bash".to_string(),
            target: cmd.clone(),
            policy_tier: if is_dangerous {
                PolicyTier::Dangerous
            } else {
                PolicyTier::WritePrompt
            },
            details: format!("Execute shell command: {}", cmd),
            diff: None,
            proposed_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn new_edit(
        id: impl Into<String>,
        path: impl Into<String>,
        diff: impl Into<String>,
    ) -> Self {
        let p = path.into();
        Self {
            id: id.into(),
            tool_name: "edit_file".to_string(),
            target: p.clone(),
            policy_tier: PolicyTier::WritePrompt,
            details: format!("Modify file contents in {}", p),
            diff: Some(diff.into()),
            proposed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// JSON-RPC 2.0 styled request sent from desktop UI to daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method", content = "params")]
pub enum IpcRequest {
    Ping,
    GetStatus,
    GetMemoryStats,
    GetMemoryGraph {
        filter: Option<String>,
        limit: Option<usize>,
    },
    GetReviewDiff {
        cached: bool,
    },
    GetAuditLogs {
        limit: usize,
    },
    GetDocsHealth,
    StartAgentTask {
        task: String,
        mode: Option<PermissionMode>,
    },
    ApproveToolCall {
        proposal_id: String,
    },
    DenyToolCall {
        proposal_id: String,
        reason: Option<String>,
    },
    CancelTask {
        task_id: String,
    },
    RegenerateDocs {
        stale_only: bool,
    },
    ExecuteTerminalCommand {
        command: String,
    },
    ReadFile {
        path: String,
    },
    SaveFile {
        path: String,
        content: String,
    },
    ListRepoFiles,
    BrowserFetch {
        url: String,
        method: String,
        headers: Vec<(String, String)>,
        body: Option<String>,
    },
    RunScan,
    RunDoctor,
    GetFailureCards,
    ResolveFailureCard {
        id: i64,
        fix_summary: String,
    },
    GetGitHookStatus,
    SetGitHook {
        install: bool,
    },
    TestApiKey {
        provider: String,
        key: String,
    },
    SaveConfigSettings {
        settings: serde_json::Value,
    },
}

/// Response returned from daemon to desktop UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "data")]
pub enum IpcResponse {
    Ok(serde_json::Value),
    Error { code: i32, message: String },
}

impl IpcResponse {
    pub fn success(data: impl Serialize) -> Self {
        match serde_json::to_value(data) {
            Ok(val) => IpcResponse::Ok(val),
            Err(e) => IpcResponse::Error {
                code: -32603,
                message: format!("Serialization error: {}", e),
            },
        }
    }

    pub fn err(code: i32, message: impl Into<String>) -> Self {
        IpcResponse::Error {
            code,
            message: message.into(),
        }
    }
}

/// Streaming event notifications pushed from daemon to desktop UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", content = "payload")]
pub enum IpcEvent {
    AgentThought {
        task_id: String,
        thought: String,
        timestamp: String,
    },
    ToolProposed(ToolProposal),
    ToolExecutionResult {
        proposal_id: String,
        success: bool,
        output: String,
    },
    FileChanged {
        path: String,
        change_type: String,
    },
    MemoryUpdated {
        total_files: usize,
        total_symbols: usize,
        total_edges: usize,
    },
    AuditLogAppended(Box<AuditLogEntry>),
    TaskCompleted {
        task_id: String,
        success: bool,
        summary: String,
    },
    Heartbeat {
        timestamp: String,
    },
}

/// Top-level wire envelope for messages exchanged over named pipe or socket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum WireMessage {
    Request { id: u64, request: IpcRequest },
    Response { id: u64, response: IpcResponse },
    Event { event: IpcEvent },
}
