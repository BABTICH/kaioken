import type {
  AgentEvent,
  AgentEventEnvelope,
  DebugStreamOptions,
  EventAck
} from "@shared/types/agent";

const DEFAULT_BATCH_WINDOW_MS = 16;
const MAX_BATCH_EVENTS = 256;
const OUTSTANDING_HIGH_WATER_BYTES = 2 * 1024 * 1024;
const OUTSTANDING_LOW_WATER_BYTES = 512 * 1024;

interface DebugSession {
  sessionId: string;
  messageId: string;
  eventsPerSecond: number;
  tokensPerEvent: number;
  startedAt: number;
  durationMs?: number;
  seq: number;
  accumulator: number;
  batchWindowMs: number;
  outstandingBytes: number;
  paused: boolean;
  stopped: boolean;
  totalTokens: number;
  timer?: NodeJS.Timeout;
}

export class DebugStreamManager {
  private readonly sessions = new Map<string, DebugSession>();

  constructor(
    private readonly emitEnvelope: (envelope: AgentEventEnvelope) => void
  ) {}

  start(options: DebugStreamOptions): void {
    this.stop(options.sessionId, "replaced");

    const sessionId = options.sessionId;
    const messageId = `${sessionId}:message:debug`;

    const session: DebugSession = {
      sessionId,
      messageId,
      eventsPerSecond: options.eventsPerSecond ?? 240,
      tokensPerEvent: options.tokensPerEvent ?? 16,
      startedAt: Date.now(),
      durationMs: options.durationMs,
      seq: 0,
      accumulator: 0,
      batchWindowMs: DEFAULT_BATCH_WINDOW_MS,
      outstandingBytes: 0,
      paused: false,
      stopped: false,
      totalTokens: 0
    };

    this.sessions.set(sessionId, session);

    this.emitEvents(session, [
      {
        type: "session.status",
        status: "running",
        reason: "debug-stream"
      },
      {
        type: "message.created",
        messageId,
        role: "assistant",
        createdAt: Date.now()
      }
    ]);

    session.timer = setInterval(() => {
      this.tick(sessionId);
    }, session.batchWindowMs);
  }

  stop(sessionId: string, reason = "manual"): void {
    const session = this.sessions.get(sessionId);

    if (!session || session.stopped) {
      return;
    }

    session.stopped = true;

    if (session.timer) {
      clearInterval(session.timer);
      session.timer = undefined;
    }

    this.emitEvents(
      session,
      [
        {
          type: "message.completed",
          messageId: session.messageId,
          fullText: `[debug-stream] ${session.totalTokens} synthetic tokens`,
          usage: {
            promptTokens: 0,
            completionTokens: session.totalTokens
          }
        },
        {
          type: "session.status",
          status: "stopped",
          reason
        }
      ],
      true
    );

    this.sessions.delete(sessionId);
  }

  stopAll(): void {
    for (const sessionId of [...this.sessions.keys()]) {
      this.stop(sessionId, "shutdown");
    }
  }

  ack(ack: EventAck): void {
    const session = this.sessions.get(ack.sessionId);

    if (!session || session.stopped) {
      return;
    }

    session.outstandingBytes = Math.max(
      0,
      session.outstandingBytes - ack.receivedBytes
    );

    const nextBatchWindow =
      ack.busyLevel === 2 ? 64 : ack.busyLevel === 1 ? 32 : 16;

    if (nextBatchWindow !== session.batchWindowMs) {
      session.batchWindowMs = nextBatchWindow;

      if (session.timer) {
        clearInterval(session.timer);
      }

      session.timer = setInterval(() => {
        this.tick(session.sessionId);
      }, session.batchWindowMs);
    }

    if (
      session.paused &&
      session.outstandingBytes <= OUTSTANDING_LOW_WATER_BYTES
    ) {
      session.paused = false;
    }
  }

  private tick(sessionId: string): void {
    const session = this.sessions.get(sessionId);

    if (!session || session.stopped || session.paused) {
      return;
    }

    if (
      session.durationMs &&
      Date.now() - session.startedAt >= session.durationMs
    ) {
      this.stop(sessionId, "duration-complete");
      return;
    }

    session.accumulator +=
      (session.eventsPerSecond * session.batchWindowMs) / 1000;

    const generated = Math.floor(session.accumulator);
    session.accumulator -= generated;

    const count = Math.min(generated, MAX_BATCH_EVENTS);

    if (generated > count) {
      session.accumulator += generated - count;
    }

    if (count <= 0) {
      return;
    }

    const tokens = count * session.tokensPerEvent;
    session.totalTokens += tokens;

    const deltaText = ".".repeat(Math.min(tokens, 65_536));

    this.emitEvents(session, [
      {
        type: "message.delta",
        messageId: session.messageId,
        deltaText,
        deltaTokens: tokens
      }
    ]);
  }

  private emitEvents(
    session: DebugSession,
    events: AgentEvent[],
    final = false
  ): void {
    session.seq += 1;

    const envelope: AgentEventEnvelope = {
      sessionId: session.sessionId,
      seq: session.seq,
      cursor: `${session.sessionId}:${session.seq}`,
      emittedAt: Date.now(),
      events,
      final
    };

    const bytes = Buffer.byteLength(JSON.stringify(envelope), "utf8");
    session.outstandingBytes += bytes;

    if (session.outstandingBytes > OUTSTANDING_HIGH_WATER_BYTES) {
      session.paused = true;
    }

    this.emitEnvelope(envelope);
  }
}
