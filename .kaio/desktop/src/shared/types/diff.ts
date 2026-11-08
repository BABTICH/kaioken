// src/shared/types/diff.ts

export type HunkDecision =
  | "pending"
  | "accepted"
  | "rejected";

export type DiffFileStatus =
  | "pending"
  | "approved"
  | "rejected"
  | "modified";

export interface DiffLine {
  id: string;
  type: "context" | "add" | "delete";
  oldLineNumber?: number;
  newLineNumber?: number;
  text: string;
}

export interface DiffHunk {
  id: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  decision: HunkDecision;
  lines: DiffLine[];
}

export interface DiffFile {
  path: string;
  status: DiffFileStatus;
  hunks: DiffHunk[];
}

export interface DiffReview {
  diffRef: string;
  sessionId?: string;
  toolCallId?: string;
  patchChecksum?: string;
  activeFilePath: string;
  createdAt: number;
  files: DiffFile[];
}
