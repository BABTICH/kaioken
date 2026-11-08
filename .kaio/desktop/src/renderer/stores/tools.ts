// src/renderer/stores/tools.ts
import { atom } from "nanostores";
import type {
  AgentEvent,
  ToolCallState,
  ToolRiskLevel
} from "@shared/types/agent";

export interface ToolCallVM {
  id: string;
  messageId: string;
  name: string;
  title: string;
  summary?: string;
  risk?: ToolRiskLevel;
  state: ToolCallState;
  output: string;
  startedAt: number;
  endedAt?: number;
}

export interface ToolsState {
  byId: Record<string, ToolCallVM>;
}

const initialState: ToolsState = {
  byId: {}
};

export const toolStore = atom<ToolsState>(initialState);

export function resetTools(): void {
  toolStore.set({ byId: {} });
}

export function applyToolEvent(event: AgentEvent): void {
  if (event.type === "tool.proposed") {
    const state = toolStore.get();
    const tool = event.tool;

    toolStore.set({
      byId: {
        ...state.byId,
        [tool.id]: {
          id: tool.id,
          messageId: event.messageId,
          name: tool.name,
          title: tool.title,
          summary: tool.summary,
          risk: tool.risk,
          state: tool.state,
          output: "",
          startedAt: Date.now()
        }
      }
    });

    return;
  }

  if (event.type === "tool.state") {
    const state = toolStore.get();
    const existing = state.byId[event.toolId];

    if (!existing) {
      return;
    }

    let output = existing.output;

    if (event.outputDelta) {
      output += event.outputDelta;
    }

    if (event.result) {
      output += `${output.length > 0 ? "\n" : ""}${event.result}`;
    }

    if (event.error) {
      output += `${output.length > 0 ? "\n" : ""}${event.error}`;
    }

    const terminal =
      event.state === "success" ||
      event.state === "failure" ||
      event.state === "cancelled" ||
      event.state === "denied" ||
      event.state === "timeout";

    const endedAt = terminal ? Date.now() : existing.endedAt;

    toolStore.set({
      byId: {
        ...state.byId,
        [event.toolId]: {
          ...existing,
          state: event.state,
          output,
          endedAt
        }
      }
    });
  }
}
