import { randomUUID } from "node:crypto";
import type {
  RpcNotification,
  RpcRequest,
  RpcResponse
} from "@shared/types/engine";

type NotificationListener = (notification: RpcNotification) => void;

interface PendingRequest {
  method: string;
  timer: NodeJS.Timeout;
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

export class EngineRpcClient {
  private readonly pending = new Map<string, PendingRequest>();

  private readonly notificationListeners = new Set<NotificationListener>();

  constructor(
    private readonly postMessage: (message: unknown) => void,
    private readonly defaultTimeoutMs = 5000
  ) {}

  request<T>(
    method: string,
    params?: unknown,
    timeoutMs?: number
  ): Promise<T> {
    const id = randomUUID();

    const request: RpcRequest = {
      jsonrpc: "2.0",
      id,
      method,
      params
    };

    const promise = new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Engine RPC timeout: ${method}`));
      }, timeoutMs ?? this.defaultTimeoutMs);

      this.pending.set(id, {
        method,
        timer,
        resolve: resolve as (value: unknown) => void,
        reject
      });
    });

    try {
      this.postMessage(request);
    } catch (error) {
      const pending = this.pending.get(id);

      if (pending) {
        clearTimeout(pending.timer);
        this.pending.delete(id);
        pending.reject(
          error instanceof Error ? error : new Error(String(error))
        );
      }
    }

    return promise;
  }

  notify(method: string, params?: unknown): void {
    const notification: RpcNotification = {
      jsonrpc: "2.0",
      method,
      params
    };

    this.postMessage(notification);
  }

  handleMessage(message: unknown): void {
    if (!this.isRpcMessage(message)) {
      return;
    }

    if (this.isResponse(message)) {
      const pending = this.pending.get(message.id);

      if (!pending) {
        return;
      }

      clearTimeout(pending.timer);
      this.pending.delete(message.id);

      if (message.error) {
        pending.reject(new Error(message.error.message));
      } else {
        pending.resolve(message.result);
      }

      return;
    }

    if (this.isNotification(message)) {
      for (const listener of this.notificationListeners) {
        listener(message);
      }
    }
  }

  onNotification(listener: NotificationListener): () => void {
    this.notificationListeners.add(listener);

    return () => {
      this.notificationListeners.delete(listener);
    };
  }

  dispose(): void {
    for (const [id, pending] of this.pending) {
      clearTimeout(pending.timer);
      pending.reject(new Error("Engine RPC client disposed"));
      this.pending.delete(id);
    }

    this.notificationListeners.clear();
  }

  private isRpcMessage(message: unknown): message is Record<string, unknown> {
    return (
      typeof message === "object" &&
      message !== null &&
      (message as Record<string, unknown>).jsonrpc === "2.0"
    );
  }

  private isResponse(
    message: unknown
  ): message is RpcResponse {
    const msg = message as Record<string, unknown>;
    return (
      this.isRpcMessage(message) &&
      typeof msg.id === "string" &&
      typeof msg.method !== "string"
    );
  }

  private isNotification(
    message: unknown
  ): message is RpcNotification {
    const msg = message as Record<string, unknown>;
    return (
      this.isRpcMessage(message) &&
      typeof msg.method === "string" &&
      typeof msg.id !== "string"
    );
  }
}
