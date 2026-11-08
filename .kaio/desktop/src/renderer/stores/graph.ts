// src/renderer/stores/graph.ts
import { atom } from "nanostores";
import type {
  GraphEdgeDef,
  GraphNodeDef
} from "@renderer/lib/graph-types";

export interface GraphState {
  nodes: GraphNodeDef[];
  edges: GraphEdgeDef[];
  selectedNodeId: string | null;
  layoutKey: number;
}

const initialState: GraphState = {
  nodes: [],
  edges: [],
  selectedNodeId: null,
  layoutKey: 0
};

export const graphStore = atom<GraphState>(initialState);

function set(patch: Partial<GraphState>): void {
  graphStore.set({
    ...graphStore.get(),
    ...patch
  });
}

function createMockGraph(): {
  nodes: GraphNodeDef[];
  edges: GraphEdgeDef[];
} {
  const nodes: GraphNodeDef[] = [
    {
      id: "app-shell",
      label: "AppShell",
      kind: "module",
      filePath: "src/renderer/components/shell/AppShell.tsx",
      description: "Desktop application shell."
    },
    {
      id: "session-store",
      label: "sessionStore",
      kind: "module",
      filePath: "src/renderer/stores/session.ts",
      description: "Renderer session lifecycle store."
    },
    {
      id: "engine-supervisor",
      label: "EngineSupervisor",
      kind: "class",
      filePath: "src/main/engine/supervisor.ts",
      description: "Supervises the Kaiopi utility process."
    },
    {
      id: "kaiopi-engine",
      label: "Kaiopi Engine",
      kind: "module",
      filePath: "src/engine/entry.ts",
      description: "Utility-process agent runtime."
    },
    {
      id: "agent-events",
      label: "agent-events",
      kind: "module",
      filePath: "src/renderer/stores/agent-events.ts",
      description: "Applies streamed agent envelopes."
    },
    {
      id: "message-store",
      label: "messageStore",
      kind: "module",
      filePath: "src/renderer/stores/messages.ts",
      description: "Conversation message metadata."
    },
    {
      id: "tool-store",
      label: "toolStore",
      kind: "module",
      filePath: "src/renderer/stores/tools.ts",
      description: "Tool-call state machine."
    },
    {
      id: "approval-gateway",
      label: "ApprovalGateway",
      kind: "module",
      filePath: "src/main/tools/approval-gateway.ts",
      description: "Destructive action approval gateway."
    },
    {
      id: "pty-manager",
      label: "PtyManager",
      kind: "module",
      filePath: "src/main/pty/manager.ts",
      description: "Terminal PTY supervisor."
    },
    {
      id: "workspace-guard",
      label: "WorkspaceGuard",
      kind: "module",
      filePath: "src/main/workspace/guard.ts",
      description: "Path sandbox and trusted root guard."
    },
    {
      id: "diff-store",
      label: "diffStore",
      kind: "module",
      filePath: "src/renderer/stores/diff.ts",
      description: "Diff review and hunk decisions."
    },
    {
      id: "graph-store",
      label: "graphStore",
      kind: "module",
      filePath: "src/renderer/stores/graph.ts",
      description: "Codemap graph state."
    },
    {
      id: "codemap-surface",
      label: "CodemapSurface",
      kind: "function",
      filePath: "src/renderer/components/graph/CodemapSurface.tsx",
      description: "Codemap visualization surface."
    }
  ];

  const edges: GraphEdgeDef[] = [
    {
      id: "app-shell-session",
      source: "app-shell",
      target: "session-store",
      kind: "uses"
    },
    {
      id: "session-supervisor",
      source: "session-store",
      target: "engine-supervisor",
      kind: "calls"
    },
    {
      id: "supervisor-engine",
      source: "engine-supervisor",
      target: "kaiopi-engine",
      kind: "calls"
    },
    {
      id: "engine-agent-events",
      source: "kaiopi-engine",
      target: "agent-events",
      kind: "emits"
    },
    {
      id: "agent-events-message",
      source: "agent-events",
      target: "message-store",
      kind: "uses"
    },
    {
      id: "agent-events-tool",
      source: "agent-events",
      target: "tool-store",
      kind: "uses"
    },
    {
      id: "tool-approval",
      source: "tool-store",
      target: "approval-gateway",
      kind: "uses"
    },
    {
      id: "approval-engine",
      source: "approval-gateway",
      target: "kaiopi-engine",
      kind: "resolves"
    },
    {
      id: "pty-guard",
      source: "pty-manager",
      target: "workspace-guard",
      kind: "guards"
    },
    {
      id: "app-shell-pty",
      source: "app-shell",
      target: "pty-manager",
      kind: "uses"
    },
    {
      id: "codemap-graph-store",
      source: "codemap-surface",
      target: "graph-store",
      kind: "uses"
    },
    {
      id: "graph-store-mock",
      source: "graph-store",
      target: "kaiopi-engine",
      kind: "uses"
    },
    {
      id: "diff-store-engine",
      source: "diff-store",
      target: "kaiopi-engine",
      kind: "uses"
    }
  ];

  return {
    nodes,
    edges
  };
}

export function loadMockGraph(): void {
  const graph = createMockGraph();
  const state = graphStore.get();

  set({
    nodes: graph.nodes,
    edges: graph.edges,
    selectedNodeId: null,
    layoutKey: state.layoutKey + 1
  });
}

export function selectGraphNode(nodeId: string | null): void {
  set({
    selectedNodeId: nodeId
  });
}

export function reheatGraphLayout(): void {
  const state = graphStore.get();

  set({
    layoutKey: state.layoutKey + 1
  });
}
