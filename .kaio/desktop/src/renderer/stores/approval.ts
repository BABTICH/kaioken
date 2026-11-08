// src/renderer/stores/approval.ts
import { atom } from "nanostores";
import type { ApprovalRequest } from "@shared/types/tools";

export interface ApprovalState {
  activeRequest: ApprovalRequest | null;
  resolving: boolean;
  error: string | null;
}

const initialState: ApprovalState = {
  activeRequest: null,
  resolving: false,
  error: null
};

export const approvalStore = atom<ApprovalState>(initialState);

let initialized = false;

function set(patch: Partial<ApprovalState>): void {
  approvalStore.set({
    ...approvalStore.get(),
    ...patch
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function initApprovalBridge(): void {
  if (initialized) {
    return;
  }

  initialized = true;

  window.api.tools.onApprovalRequest((request) => {
    set({
      activeRequest: request,
      resolving: false,
      error: null
    });
  });

  window.api.tools.onApprovalResolved((resolved) => {
    const current = approvalStore.get();

    if (
      current.activeRequest &&
      current.activeRequest.requestId === resolved.requestId
    ) {
      set({
        activeRequest: null,
        resolving: false,
        error: null
      });
    }
  });
}

async function resolveActiveRequest(
  decision: "allow_once" | "deny",
  reason?: string
): Promise<void> {
  const current = approvalStore.get();
  const request = current.activeRequest;

  if (!request || current.resolving) {
    return;
  }

  set({
    resolving: true,
    error: null
  });

  try {
    await window.api.tools.resolveApproval({
      requestId: request.requestId,
      decision,
      reason
    });

    set({
      activeRequest: null,
      resolving: false,
      error: null
    });
  } catch (error) {
    set({
      resolving: false,
      error: errorMessage(error)
    });
  }
}

export async function approveActiveRequest(): Promise<void> {
  await resolveActiveRequest("allow_once", "user approved");
}

export async function denyActiveRequest(reason?: string): Promise<void> {
  await resolveActiveRequest("deny", reason ?? "user denied");
}
