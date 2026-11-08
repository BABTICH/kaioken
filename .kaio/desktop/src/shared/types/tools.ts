// src/shared/types/tools.ts

export type Unsubscribe = () => void;

export type DestructiveToolKind =
  | "write_to_file"
  | "apply_patch"
  | "run_command"
  | "git_operation";

export type ApprovalRiskLevel = "mutate" | "destructive";

export type ApprovalDecision =
  | "allow_once"
  | "allow_session"
  | "deny"
  | "modify"
  | "timeout";

export interface ApprovalRequest {
  requestId: string;
  sessionId: string;
  toolCallId: string;
  messageId?: string;
  kind: DestructiveToolKind;
  risk: ApprovalRiskLevel;
  title: string;
  description: string;
  command?: string;
  filePath?: string;
  diffRef?: string;
  timeoutMs: number;
  createdAt: number;
}

export interface RendererApprovalResolution {
  requestId: string;
  decision: "allow_once" | "deny";
  reason?: string;
}

export interface ApprovalResolution {
  requestId: string;
  decision: ApprovalDecision;
  reason?: string;
}

export interface ApprovalResolvedEvent {
  requestId: string;
  decision: ApprovalDecision;
}

export interface AuditEntry {
  id: string;
  at: number;
  sessionId?: string;
  toolCallId?: string;
  kind?: DestructiveToolKind;
  decision: ApprovalDecision;
  summary: string;
}

export interface ToolsApi {
  onApprovalRequest(
    cb: (request: ApprovalRequest) => void
  ): Unsubscribe;

  onApprovalResolved(
    cb: (event: ApprovalResolvedEvent) => void
  ): Unsubscribe;

  resolveApproval(
    resolution: RendererApprovalResolution
  ): Promise<void>;

  getAuditLog(limit?: number): Promise<AuditEntry[]>;
}
