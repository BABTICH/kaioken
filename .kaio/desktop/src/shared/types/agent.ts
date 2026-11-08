// src/shared/types/agent.ts

export type Unsubscribe = () => void;

export type MultiplierLevel =
  | 1
  | 2
  | 3
  | 4
  | 5
  | 6
  | 7
  | 8
  | 9
  | 10;

export type SessionStatus =
  | "starting"
  | "ready"
  | "running"
  | "waiting_approval"
  | "stopped"
  | "error";

export interface SessionStatusEvent {
  type: "session.status";
  status: SessionStatus;
  reason?: string;
}

export interface MessageCreatedEvent {
  type: "message.created";
  messageId: string;
  role: "user" | "assistant" | "system" | "tool";
  createdAt: number;
}

export interface MessageDeltaEvent {
  type: "message.delta";
  messageId: string;
  deltaText: string;
  deltaTokens?: number;
  finishReason?: "stop" | "length" | "tool_calls" | "error";
}

export interface MessageCompletedEvent {
  type: "message.completed";
  messageId: string;
  fullText: string;
  usage?: {
    promptTokens: number;
    completionTokens: number;
    estimatedUsd?: number;
  };
}

export type ToolRiskLevel =
  | "read"
  | "mutate"
  | "destructive";

export type ToolCallState =
  | "proposed"
  | "awaiting_approval"
  | "approved"
  | "running"
  | "success"
  | "failure"
  | "cancelled"
  | "denied"
  | "timeout";

export interface ToolCallSummary {
  id: string;
  name: string;
  title: string;
  summary?: string;
  state: ToolCallState;
  risk?: ToolRiskLevel;
}

export interface ToolProposedEvent {
  type: "tool.proposed";
  messageId: string;
  tool: ToolCallSummary;
}

export interface ToolStateEvent {
  type: "tool.state";
  messageId: string;
  toolId: string;
  state: ToolCallState;
  outputDelta?: string;
  result?: string;
  error?: string;
}

export type AgentEvent =
  | SessionStatusEvent
  | MessageCreatedEvent
  | MessageDeltaEvent
  | MessageCompletedEvent
  | ToolProposedEvent
  | ToolStateEvent;

export interface AgentEventEnvelope {
  sessionId: string;
  seq: number;
  cursor: string;
  emittedAt: number;
  events: AgentEvent[];
  final?: boolean;
}

export interface EventAck {
  sessionId: string;
  lastSeq: number;
  receivedBytes: number;
  busyLevel: 0 | 1 | 2;
}

export interface CreateAgentSessionInput {
  workspaceRoot: string;
  providerId: string;
  modelId: string;
  multiplier?: MultiplierLevel;
}

export interface ProviderEngineConfig {
  providerId: string;
  baseUrl?: string;
  apiKey?: string;
}

export interface AgentSessionHistoryStats {
  retainedEnvelopes: number;
  droppedEnvelopes: number;
  lastCompactedSeq: number | null;
}

export interface AgentSessionInfo {
  sessionId: string;
  workspaceRoot: string;
  providerId: string;
  modelId: string;
  multiplier: MultiplierLevel;
  status: SessionStatus;
  createdAt: number;
  updatedAt: number;
  hasProviderSecret?: boolean;
  historyStats?: AgentSessionHistoryStats;
}

export interface AgentSessionSummary {
  sessionId: string;
  title?: string;
  workspaceRoot: string;
  status: SessionStatus;
  updatedAt: number;
}

export interface AgentMessageInput {
  sessionId: string;
  text: string;
}

export interface SendMessageResult {
  messageId: string;
}

export interface AgentConnectionInfo {
  sessionId: string;
  lastSeq: number;
  cursor: string;
  transport: "main-brokered";
  replay: AgentEventEnvelope[];
  capabilities: {
    streamingMarkdown: boolean;
    mermaid: boolean;
    diffs: boolean;
    codemap: boolean;
    terminal: boolean;
  };
}

export interface ConnectSessionSchemaInput {
  sessionId: string;
  afterCursor?: string;
}

export interface DebugStreamOptions {
  sessionId: string;
  eventsPerSecond?: number;
  tokensPerEvent?: number;
  durationMs?: number;
}

export interface StartDebugStreamResult {
  sessionId: string;
}

export interface AgentApi {
  createSession(
    input: CreateAgentSessionInput
  ): Promise<AgentSessionInfo>;

  listSessions(): Promise<AgentSessionSummary[]>;

  restoreSession(sessionId: string): Promise<AgentSessionInfo>;

  sendMessage(input: AgentMessageInput): Promise<SendMessageResult>;

  stopSession(sessionId: string, reason?: string): Promise<void>;

  connect(
    sessionId: string,
    afterCursor?: string
  ): Promise<AgentConnectionInfo>;

  startDebugStream(
    options: DebugStreamOptions
  ): Promise<StartDebugStreamResult>;

  stopDebugStream(sessionId: string): Promise<void>;

  ackEvents(ack: EventAck): void;

  onEvent(cb: (envelope: AgentEventEnvelope) => void): Unsubscribe;
}

export type AgentStreamApi = AgentApi;
