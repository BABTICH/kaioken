// src/engine/session/session-service.ts
import { randomUUID } from "node:crypto";
import { promises as fs } from "node:fs";
import path from "node:path";
import type {
  AgentConnectionInfo,
  AgentEvent,
  AgentEventEnvelope,
  AgentMessageInput,
  AgentSessionInfo,
  AgentSessionSummary,
  CreateAgentSessionInput,
  SendMessageResult
} from "@shared/types/agent";
import type { ApprovalRequest } from "@shared/types/tools";

const BATCH_WINDOW_MS = 16;
const RESPONSE_CHUNKS = 160;
const TOKENS_PER_CHUNK = 8;

const MOCK_MARKDOWN_CHUNKS = [
  "# Kaioken session report\n\n",
  "Streaming **markdown** with `inline code`.\n\n",
  "- Workspace guard\n",
  "- PTY batching\n",
  "- Tool approval\n\n",
  "```ts\nexport const phase = '4.2';\n",
  "```\n\n",
  "Mock token chunk "
];

const HISTORY_RETAIN = 80;
const COMPACTION_TRIGGER = 160;

const SAVE_DEBOUNCE_MS = 250;

interface SessionCompaction {
  droppedEnvelopes: number;
  lastCompactedSeq: number | null;
  at: number;
}

interface EngineSession {
  sessionId: string;
  info: AgentSessionInfo;
  seq: number;
  cursor: string;
  history: AgentEventEnvelope[];
  activeMessageId?: string;
  activeMessageText?: string;
  activeToolId?: string;
  toolCompleted?: boolean;
  approvalToolId?: string;
  approvalRequestId?: string;
  approvalCompleted?: boolean;
  mockChunkIndex?: number;
  streamTimer?: NodeJS.Timeout;
  remainingChunks: number;
  stopped: boolean;
  compaction?: SessionCompaction;
}

interface PersistedSession {
  info: AgentSessionInfo;
  seq: number;
  cursor: string;
  history: AgentEventEnvelope[];
  compaction?: SessionCompaction;
}

interface PersistedSessionFile {
  version: 1;
  sessions: Record<string, PersistedSession>;
}

export class MockSessionService {
  private readonly sessions = new Map<string, EngineSession>();

  private readonly pendingApprovals = new Map<
    string,
    {
      sessionId: string;
      toolCallId: string;
      messageId: string;
    }
  >();

  private saveTimer: NodeJS.Timeout | undefined;

  private readonly storagePath: string;

  private initialized = false;

  constructor(
    private readonly emitEnvelope: (
      envelope: AgentEventEnvelope
    ) => void,
    private readonly requestApproval?: (
      request: ApprovalRequest
    ) => void
  ) {
    const dir =
      process.env.KAIOKEN_SESSION_DIR ??
      path.join(process.cwd(), ".kaiopi-sessions");

    this.storagePath = path.join(dir, "sessions.json");
  }

  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    this.initialized = true;

    try {
      await fs.mkdir(path.dirname(this.storagePath), {
        recursive: true
      });

      const raw = await fs.readFile(this.storagePath, "utf8");
      const parsed = JSON.parse(raw) as PersistedSessionFile;

      if (parsed?.version === 1 && parsed.sessions) {
        for (const persisted of Object.values(parsed.sessions)) {
          const session: EngineSession = {
            sessionId: persisted.info.sessionId,
            info: persisted.info,
            seq: persisted.seq,
            cursor: persisted.cursor,
            history: persisted.history ?? [],
            remainingChunks: 0,
            stopped: persisted.info.status === "stopped",
            compaction: persisted.compaction
          };

          if (
            session.info.status === "running" ||
            session.info.status === "starting"
          ) {
            session.info.status = "ready";
          }

          this.sessions.set(session.info.sessionId, session);
        }
      }

      console.log(
        `[KaiopiEngine] Session storage loaded: ${this.storagePath}`
      );
    } catch {
      console.log(
        `[KaiopiEngine] No previous session storage found. Starting clean: ${this.storagePath}`
      );
    }
  }

  create(
    input: CreateAgentSessionInput,
    options?: {
      hasProviderSecret?: boolean;
    }
  ): AgentSessionInfo {
    const sessionId = randomUUID();
    const now = Date.now();

    const info: AgentSessionInfo = {
      sessionId,
      workspaceRoot: input.workspaceRoot,
      providerId: input.providerId,
      modelId: input.modelId,
      multiplier: input.multiplier ?? 1,
      status: "ready",
      createdAt: now,
      updatedAt: now,
      hasProviderSecret: options?.hasProviderSecret ?? false
    };

    const session: EngineSession = {
      sessionId,
      info,
      seq: 0,
      cursor: `${sessionId}:0`,
      history: [],
      remainingChunks: 0,
      stopped: false
    };

    this.sessions.set(sessionId, session);

    this.emitEvents(session, [
      {
        type: "session.status",
        status: "ready",
        reason: "session.created"
      }
    ]);

    return this.publicInfo(session);
  }

  list(): AgentSessionSummary[] {
    return [...this.sessions.values()]
      .map((session) => ({
        sessionId: session.info.sessionId,
        title: "Kaiopi session",
        workspaceRoot: session.info.workspaceRoot,
        status: session.info.status,
        updatedAt: session.info.updatedAt
      }))
      .sort((a, b) => b.updatedAt - a.updatedAt);
  }

  restore(sessionId: string): AgentSessionInfo {
    const session = this.getSession(sessionId);

    return this.publicInfo(session);
  }

  connect(
    sessionId: string,
    afterCursor?: string
  ): AgentConnectionInfo {
    const session = this.getSession(sessionId);
    const afterSeq = this.parseCursor(afterCursor);

    const replay = session.history.filter(
      (envelope) => envelope.seq > afterSeq
    );

    return {
      sessionId,
      lastSeq: session.seq,
      cursor: session.cursor,
      transport: "main-brokered",
      replay,
      capabilities: {
        streamingMarkdown: true,
        mermaid: false,
        diffs: false,
        codemap: false,
        terminal: false
      }
    };
  }

  sendMessage(input: AgentMessageInput): SendMessageResult {
    const session = this.getSession(input.sessionId);

    if (session.stopped) {
      throw new Error("SESSION_STOPPED");
    }

    this.stopActiveResponse(session, "replaced", "ready");

    const now = Date.now();
    const userMessageId = randomUUID();
    const assistantMessageId = randomUUID();
    const toolId = randomUUID();

    session.info.status = "running";
    session.info.updatedAt = now;
    session.activeMessageId = assistantMessageId;
    session.activeMessageText = "";
    session.activeToolId = toolId;
    session.toolCompleted = false;
    session.remainingChunks = RESPONSE_CHUNKS;
    session.mockChunkIndex = 0;

    const approvalToolId = randomUUID();
    const approvalRequestId = randomUUID();

    session.approvalToolId = approvalToolId;
    session.approvalRequestId = approvalRequestId;
    session.approvalCompleted = false;

    this.emitEvents(session, [
      {
        type: "message.created",
        messageId: userMessageId,
        role: "user",
        createdAt: now
      },
      {
        type: "message.completed",
        messageId: userMessageId,
        fullText: input.text,
        usage: {
          promptTokens: 0,
          completionTokens: 0
        }
      },
      {
        type: "session.status",
        status: "running",
        reason: "assistant.response"
      },
      {
        type: "message.created",
        messageId: assistantMessageId,
        role: "assistant",
        createdAt: now
      },
      {
        type: "tool.proposed",
        messageId: assistantMessageId,
        tool: {
          id: toolId,
          name: "kaio_symbol_lookup",
          title: "Symbol lookup",
          summary: "Mock grounded symbol search",
          state: "proposed"
        }
      },
      {
        type: "tool.state",
        messageId: assistantMessageId,
        toolId,
        state: "running",
        outputDelta: "query: agent session runtime\n"
      }
    ]);

    session.streamTimer = setInterval(() => {
      this.tick(session.sessionId);
    }, BATCH_WINDOW_MS);

    this.scheduleSave();

    return {
      messageId: assistantMessageId
    };
  }

  stop(sessionId: string, reason?: string): void {
    const session = this.getSession(sessionId);

    if (session.stopped) {
      return;
    }

    this.stopActiveResponse(session, reason ?? "user_stop", "stopped");

    session.stopped = true;
    session.info.status = "stopped";
    session.info.updatedAt = Date.now();

    this.emitEvents(
      session,
      [
        {
          type: "session.status",
          status: "stopped",
          reason: reason ?? "user_stop"
        }
      ],
      true
    );
  }

  stopAll(): void {
    void this.shutdown();
  }

  async shutdown(): Promise<void> {
    for (const session of this.sessions.values()) {
      if (session.streamTimer) {
        clearInterval(session.streamTimer);
        session.streamTimer = undefined;
      }

      session.activeMessageId = undefined;
      session.remainingChunks = 0;

      if (
        session.info.status === "running" ||
        session.info.status === "starting"
      ) {
        session.info.status = "ready";
      }
    }

    await this.saveNow();
  }

  private tick(sessionId: string): void {
    const session = this.sessions.get(sessionId);

    if (!session || !session.activeMessageId) {
      return;
    }

    if (session.remainingChunks <= 0) {
      this.stopActiveResponse(session, "complete", "ready");
      return;
    }

    if (
      session.activeToolId &&
      !session.toolCompleted &&
      session.remainingChunks === Math.floor(RESPONSE_CHUNKS / 2)
    ) {
      session.toolCompleted = true;

      this.emitEvents(session, [
        {
          type: "tool.state",
          messageId: session.activeMessageId,
          toolId: session.activeToolId,
          state: "success",
          result: "3 symbols matched (mock)"
        }
      ]);
    }

    if (
      session.approvalToolId &&
      session.approvalRequestId &&
      !session.approvalCompleted &&
      session.remainingChunks === Math.floor(RESPONSE_CHUNKS / 4)
    ) {
      const requestId = session.approvalRequestId;
      const toolId = session.approvalToolId;
      const messageId = session.activeMessageId;

      this.emitEvents(session, [
        {
          type: "tool.proposed",
          messageId,
          tool: {
            id: toolId,
            name: "write_to_file",
            title: "Write file",
            summary: "Mock destructive file write",
            state: "proposed",
            risk: "destructive"
          }
        },
        {
          type: "tool.state",
          messageId,
          toolId,
          state: "awaiting_approval",
          outputDelta: "Waiting for desktop approval\n"
        }
      ]);

      this.pendingApprovals.set(requestId, {
        sessionId: session.info.sessionId,
        toolCallId: toolId,
        messageId
      });

      if (this.requestApproval) {
        this.requestApproval({
          requestId,
          sessionId: session.info.sessionId,
          toolCallId: toolId,
          messageId,
          kind: "write_to_file",
          risk: "destructive",
          title: "Write file",
          description:
            "Kaiopi wants to write a file inside the workspace. This action requires explicit approval.",
          filePath: "src/kaioken/phase-4-3.ts",
          timeoutMs: 60_000,
          createdAt: Date.now()
        });
      } else {
        this.resolveApproval({
          requestId,
          decision: "deny"
        });
      }
    }

    session.remainingChunks -= 1;

    const chunkIndex = session.mockChunkIndex ?? 0;

    const deltaText =
      MOCK_MARKDOWN_CHUNKS[
        chunkIndex % MOCK_MARKDOWN_CHUNKS.length
      ];

    session.mockChunkIndex = chunkIndex + 1;

    session.activeMessageText =
      (session.activeMessageText ?? "") + deltaText;

    this.emitEvents(session, [
      {
        type: "message.delta",
        messageId: session.activeMessageId,
        deltaText,
        deltaTokens: TOKENS_PER_CHUNK
      }
    ]);

    if (session.remainingChunks <= 0) {
      this.stopActiveResponse(session, "complete", "ready");
    }
  }

  resolveApproval(resolution: {
    requestId: string;
    decision: string;
  }): void {
    const pending = this.pendingApprovals.get(resolution.requestId);

    if (!pending) {
      return;
    }

    this.pendingApprovals.delete(resolution.requestId);

    const session = this.sessions.get(pending.sessionId);

    if (!session) {
      return;
    }

    const approved =
      resolution.decision === "allow_once" ||
      resolution.decision === "allow_session";

    const decisionState = approved
      ? "approved"
      : resolution.decision === "timeout"
        ? "timeout"
        : "denied";

    this.emitEvents(session, [
      {
        type: "tool.state",
        messageId: pending.messageId,
        toolId: pending.toolCallId,
        state: decisionState as never,
        outputDelta: approved
          ? "Approval granted\n"
          : resolution.decision === "timeout"
            ? "Approval timed out\n"
            : "Approval denied\n"
      }
    ]);

    if (approved) {
      this.emitEvents(session, [
        {
          type: "tool.state",
          messageId: pending.messageId,
          toolId: pending.toolCallId,
          state: "running",
          outputDelta: "Writing file...\n"
        },
        {
          type: "tool.state",
          messageId: pending.messageId,
          toolId: pending.toolCallId,
          state: "success",
          result: "src/kaioken/phase-4-3.ts written (mock)"
        }
      ]);
    }

    if (session.approvalRequestId === resolution.requestId) {
      session.approvalCompleted = true;
    }
  }

  private stopActiveResponse(
    session: EngineSession,
    reason: string,
    nextStatus: "ready" | "stopped"
  ): void {
    if (session.streamTimer) {
      clearInterval(session.streamTimer);
      session.streamTimer = undefined;
    }

    if (
      session.activeMessageId &&
      session.activeToolId &&
      !session.toolCompleted
    ) {
      this.emitEvents(session, [
        {
          type: "tool.state",
          messageId: session.activeMessageId,
          toolId: session.activeToolId,
          state: nextStatus === "ready" ? "success" : "cancelled",
          result: reason
        }
      ]);

      session.toolCompleted = true;
    }

    if (session.activeMessageId) {
      const fullText =
        session.activeMessageText ||
        `[mock-response] ${reason}`;

      this.emitEvents(session, [
        {
          type: "message.completed",
          messageId: session.activeMessageId,
          fullText,
          usage: {
            promptTokens: 0,
            completionTokens:
              (RESPONSE_CHUNKS - session.remainingChunks) *
              TOKENS_PER_CHUNK
          }
        }
      ]);
    }

    session.activeMessageId = undefined;
    session.activeMessageText = "";
    session.activeToolId = undefined;
    session.toolCompleted = false;
    session.remainingChunks = 0;

    if (nextStatus === "ready") {
      session.info.status = "ready";
      session.info.updatedAt = Date.now();

      this.emitEvents(session, [
        {
          type: "session.status",
          status: "ready",
          reason
        }
      ]);
    }
  }

  private emitEvents(
    session: EngineSession,
    events: AgentEvent[],
    final = false
  ): void {
    session.seq += 1;
    session.cursor = `${session.info.sessionId}:${session.seq}`;
    session.info.updatedAt = Date.now();

    const envelope: AgentEventEnvelope = {
      sessionId: session.info.sessionId,
      seq: session.seq,
      cursor: session.cursor,
      emittedAt: Date.now(),
      events,
      final
    };

    session.history.push(envelope);

    this.maybeCompact(session);

    this.emitEnvelope(envelope);
    this.scheduleSave();
  }

  private maybeCompact(session: EngineSession): void {
    if (session.history.length <= COMPACTION_TRIGGER) {
      return;
    }

    const dropped = session.history.slice(
      0,
      session.history.length - HISTORY_RETAIN
    );

    const retained = session.history.slice(-HISTORY_RETAIN);

    if (dropped.length === 0) {
      return;
    }

    session.history = retained;

    const lastCompactedSeq =
      dropped[dropped.length - 1]?.seq ??
      session.compaction?.lastCompactedSeq ??
      null;

    session.compaction = {
      droppedEnvelopes:
        (session.compaction?.droppedEnvelopes ?? 0) + dropped.length,
      lastCompactedSeq,
      at: Date.now()
    };

    console.log(
      `[KaiopiEngine] Compacted session=${session.info.sessionId} dropped=${dropped.length} retained=${session.history.length}`
    );
  }

  private scheduleSave(): void {
    if (this.saveTimer) {
      return;
    }

    this.saveTimer = setTimeout(() => {
      this.saveTimer = undefined;
      void this.saveNow();
    }, SAVE_DEBOUNCE_MS);
  }

  private async saveNow(): Promise<void> {
    const file: PersistedSessionFile = {
      version: 1,
      sessions: {}
    };

    for (const session of this.sessions.values()) {
      file.sessions[session.info.sessionId] = {
        info: session.info,
        seq: session.seq,
        cursor: session.cursor,
        history: session.history,
        compaction: session.compaction
      };
    }

    try {
      await fs.mkdir(path.dirname(this.storagePath), {
        recursive: true
      });

      await fs.writeFile(
        this.storagePath,
        JSON.stringify(file, null, 2),
        "utf8"
      );
    } catch (error) {
      console.error("[KaiopiEngine] Failed to save sessions", error);
    }
  }

  private publicInfo(session: EngineSession): AgentSessionInfo {
    return {
      ...session.info,
      historyStats: {
        retainedEnvelopes: session.history.length,
        droppedEnvelopes: session.compaction?.droppedEnvelopes ?? 0,
        lastCompactedSeq: session.compaction?.lastCompactedSeq ?? null
      }
    };
  }

  private getSession(sessionId: string): EngineSession {
    const session = this.sessions.get(sessionId);

    if (!session) {
      throw new Error("SESSION_NOT_FOUND");
    }

    return session;
  }

  private parseCursor(cursor?: string): number {
    if (!cursor) {
      return 0;
    }

    const parts = cursor.split(":");
    const raw = parts[parts.length - 1];
    const parsed = Number.parseInt(raw, 10);

    return Number.isFinite(parsed) ? parsed : 0;
  }
}
