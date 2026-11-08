// src/renderer/lib/graph-types.ts

export type GraphNodeKind =
  | "file"
  | "module"
  | "function"
  | "class"
  | "tool";

export interface GraphNodeDef {
  id: string;
  label: string;
  kind: GraphNodeKind;
  filePath?: string;
  description?: string;
}

export type GraphEdgeKind =
  | "imports"
  | "calls"
  | "uses"
  | "emits"
  | "guards"
  | "resolves";

export interface GraphEdgeDef {
  id: string;
  source: string;
  target: string;
  kind: GraphEdgeKind;
}
