// src/engine/entry.ts
import type {
  RpcNotification,
  RpcRequest,
  RpcResponse
} from "@shared/types/engine";
import type {
  AgentMessageInput,
  ConnectSessionSchemaInput,
  CreateAgentSessionInput,
  DebugStreamOptions,
  EventAck
} from "@shared/types/agent";
import { DebugStreamManager } from "./streaming/debug-stream";
import { MockSessionService } from "./session/session-service";

interface ParentPortLike {
  on(
    event: "message",
    listener: (message: { data: unknown }) => void
  ): void;
  postMessage(value: unknown): void;
}

type ConnectParams = {
  sessionId: string;
  afterCursor?: string;
};

const rawParentPort = (
  process as unknown as {
    parentPort?: ParentPortLike;
  }
).parentPort;

if (!rawParentPort) {
  throw new Error("[KaiopiEngine] Utility process parentPort is unavailable");
}

const parentPort: ParentPortLike = rawParentPort;

const startedAt = Date.now();

let shuttingDown = false;

function respond(
  id: string,
  result?: unknown,
  error?: RpcResponse["error"]
): void {
  const response: RpcResponse = {
    jsonrpc: "2.0",
    id,
    result,
    error
  };

  parentPort.postMessage(response);
}

function notify(method: string, params?: unknown): void {
  const notification: RpcNotification = {
    jsonrpc: "2.0",
    method,
    params
  };

  parentPort.postMessage(notification);
}

const debugStreams = new DebugStreamManager((envelope) => {
  notify("agent.events", envelope);
});

const sessionService = new MockSessionService(
  (envelope) => {
    notify("agent.events", envelope);
  },
  (approvalRequest) => {
    notify("tools.requestApproval", approvalRequest);
  }
);

async function handleRequest(request: RpcRequest): Promise<void> {
  try {
    switch (request.method) {
      case "engine.ping": {
        respond(request.id, {
          pong: Date.now()
        });
        return;
      }

      case "engine.status": {
        respond(request.id, {
          state: "ready",
          pid: process.pid,
          uptimeMs: Date.now() - startedAt,
          startedAt,
          version: "0.1.0"
        });
        return;
      }

      case "engine.shutdown": {
        if (shuttingDown) {
          respond(request.id, {
            ok: true,
            alreadyShuttingDown: true
          });
          return;
        }

        shuttingDown = true;

        debugStreams.stopAll();

        await sessionService.shutdown();

        respond(request.id, {
          ok: true
        });

        setTimeout(() => {
          process.exit(0);
        }, 100);

        return;
      }

      case "agent.createSession": {
        const params = request.params as
          | (CreateAgentSessionInput & {
              provider?: {
                baseUrl?: string;
                apiKey?: string;
              };
            })
          | undefined;

        if (
          !params ||
          typeof params.workspaceRoot !== "string" ||
          typeof params.providerId !== "string" ||
          typeof params.modelId !== "string"
        ) {
          respond(request.id, undefined, {
            code: -32602,
            message: "Invalid agent.createSession params"
          });
          return;
        }

        const hasProviderSecret = Boolean(params.provider?.apiKey);

        console.log(
          `[KaiopiEngine] createSession provider=${params.providerId} hasSecret=${hasProviderSecret}`
        );

        respond(
          request.id,
          sessionService.create(params, {
            hasProviderSecret
          })
        );

        return;
      }

      case "agent.listSessions": {
        respond(request.id, sessionService.list());
        return;
      }

      case "agent.restoreSession": {
        const params = request.params as { sessionId?: string } | undefined;

        if (!params || typeof params.sessionId !== "string") {
          respond(request.id, undefined, {
            code: -32602,
            message: "Invalid agent.restoreSession params"
          });
          return;
        }

        respond(request.id, sessionService.restore(params.sessionId));
        return;
      }

      case "agent.sendMessage": {
        const params = request.params as AgentMessageInput | undefined;

        if (
          !params ||
          typeof params.sessionId !== "string" ||
          typeof params.text !== "string"
        ) {
          respond(request.id, undefined, {
            code: -32602,
            message: "Invalid agent.sendMessage params"
          });
          return;
        }

        respond(request.id, sessionService.sendMessage(params));
        return;
      }

      case "agent.stopSession": {
        const params = request.params as
          | { sessionId?: string; reason?: string }
          | undefined;

        if (!params || typeof params.sessionId !== "string") {
          respond(request.id, undefined, {
            code: -32602,
            message: "Invalid agent.stopSession params"
          });
          return;
        }

        sessionService.stop(params.sessionId, params.reason);

        respond(request.id, {
          ok: true
        });

        return;
      }

      case "agent.connect": {
        const params = request.params as ConnectParams | undefined;

        if (!params || typeof params.sessionId !== "string") {
          respond(request.id, undefined, {
            code: -32602,
            message: "Invalid agent.connect params"
          });
          return;
        }

        respond(
          request.id,
          sessionService.connect(params.sessionId, params.afterCursor)
        );

        return;
      }

      case "agent.debug.start": {
        const params = request.params as DebugStreamOptions | undefined;

        if (!params || typeof params.sessionId !== "string") {
          respond(request.id, undefined, {
            code: -32602,
            message: "Invalid agent.debug.start params"
          });
          return;
        }

        debugStreams.start(params);

        respond(request.id, {
          sessionId: params.sessionId
        });

        return;
      }

      case "agent.debug.stop": {
        const params = request.params as { sessionId?: string } | undefined;

        if (!params || typeof params.sessionId !== "string") {
          respond(request.id, undefined, {
            code: -32602,
            message: "Invalid agent.debug.stop params"
          });
          return;
        }

        debugStreams.stop(params.sessionId);

        respond(request.id, {
          ok: true
        });

        return;
      }

      default: {
        respond(request.id, undefined, {
          code: -32601,
          message: `Method not found: ${request.method}`
        });
      }
    }
  } catch (error) {
    respond(request.id, undefined, {
      code: -32603,
      message: error instanceof Error ? error.message : String(error)
    });
  }
}

function handleNotification(notification: RpcNotification): void {
  if (notification.method === "agent.ackEvents") {
    const params = notification.params as EventAck | undefined;

    if (params && typeof params.sessionId === "string") {
      debugStreams.ack(params);
    }

    return;
  }

  if (notification.method === "tools.approvalResolved") {
    const params = notification.params as
      | {
          requestId?: string;
          decision?: string;
        }
      | undefined;

    if (
      params &&
      typeof params.requestId === "string" &&
      typeof params.decision === "string"
    ) {
      sessionService.resolveApproval({
        requestId: params.requestId,
        decision: params.decision
      });
    }
  }
}

parentPort.on("message", (messageEvent) => {
  const message = messageEvent.data as
    | RpcRequest
    | RpcNotification
    | undefined;

  if (!message || message.jsonrpc !== "2.0") {
    return;
  }

  if (typeof message.method !== "string") {
    return;
  }

  if ("id" in message && typeof message.id === "string") {
    void handleRequest(message as RpcRequest);
    return;
  }

  handleNotification(message as RpcNotification);
});

process.on("uncaughtException", (error) => {
  console.error("[KaiopiEngine] Uncaught exception", error);
});

process.on("unhandledRejection", (reason) => {
  console.error("[KaiopiEngine] Unhandled rejection", reason);
});

async function boot(): Promise<void> {
  try {
    await sessionService.init();
  } catch (error) {
    console.error("[KaiopiEngine] Session initialization failed", error);
  }

  notify("engine.ready", {
    pid: process.pid,
    startedAt
  });
}

setImmediate(() => {
  void boot();
});
