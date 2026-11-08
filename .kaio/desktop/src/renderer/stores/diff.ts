// src/renderer/stores/diff.ts
import { atom } from "nanostores";
import type {
  DiffFile,
  DiffFileStatus,
  DiffHunk,
  DiffLine,
  DiffReview,
  HunkDecision
} from "@shared/types/diff";

export interface DiffState {
  review: DiffReview | null;
  finalPatch: string;
}

const initialState: DiffState = {
  review: null,
  finalPatch: ""
};

export const diffStore = atom<DiffState>(initialState);

function set(patch: Partial<DiffState>): void {
  diffStore.set({
    ...diffStore.get(),
    ...patch
  });
}

function line(
  id: string,
  type: DiffLine["type"],
  oldLineNumber: number | undefined,
  newLineNumber: number | undefined,
  text: string
): DiffLine {
  return {
    id,
    type,
    oldLineNumber,
    newLineNumber,
    text
  };
}

function createMockDiff(): DiffReview {
  const fileOne: DiffFile = {
    path: "src/kaioken/phase.ts",
    status: "pending",
    hunks: [
      {
        id: "file-one-hunk-one",
        oldStart: 1,
        oldLines: 5,
        newStart: 1,
        newLines: 6,
        decision: "pending",
        lines: [
          line(
            "f1h1l1",
            "context",
            1,
            1,
            'import { app } from "electron";'
          ),
          line("f1h1l2", "context", 2, 2, ""),
          line(
            "f1h1l3",
            "delete",
            3,
            undefined,
            'export const phase = "4.3";'
          ),
          line(
            "f1h1l4",
            "add",
            undefined,
            3,
            'export const phase = "5.1";'
          ),
          line(
            "f1h1l5",
            "add",
            undefined,
            4,
            'export const surface = "diff-inspector";'
          ),
          line("f1h1l6", "context", 4, 5, ""),
          line(
            "f1h1l7",
            "context",
            5,
            6,
            "export function boot() {"
          )
        ]
      },
      {
        id: "file-one-hunk-two",
        oldStart: 8,
        oldLines: 3,
        newStart: 9,
        newLines: 3,
        decision: "pending",
        lines: [
          line(
            "f1h2l1",
            "context",
            8,
            9,
            '  console.log("Kaioken booting");'
          ),
          line(
            "f1h2l2",
            "delete",
            9,
            undefined,
            "  return false;"
          ),
          line(
            "f1h2l3",
            "add",
            undefined,
            10,
            "  return true;"
          ),
          line("f1h2l4", "context", 10, 11, "}")
        ]
      }
    ]
  };

  const fileTwo: DiffFile = {
    path: "docs/phase.md",
    status: "pending",
    hunks: [
      {
        id: "file-two-hunk-one",
        oldStart: 1,
        oldLines: 2,
        newStart: 1,
        newLines: 3,
        decision: "pending",
        lines: [
          line(
            "f2h1l1",
            "context",
            1,
            1,
            "# Kaioken Desktop"
          ),
          line(
            "f2h1l2",
            "delete",
            2,
            undefined,
            "- Phase 4.3"
          ),
          line(
            "f2h1l3",
            "add",
            undefined,
            2,
            "- Phase 5.1"
          ),
          line(
            "f2h1l4",
            "add",
            undefined,
            3,
            "- Diff inspector skeleton"
          )
        ]
      }
    ]
  };

  return {
    diffRef: `mock-diff-${Date.now()}`,
    patchChecksum: "mock-checksum",
    activeFilePath: fileOne.path,
    createdAt: Date.now(),
    files: [fileOne, fileTwo]
  };
}

function computeFileStatus(hunks: DiffHunk[]): DiffFileStatus {
  if (hunks.length === 0) {
    return "pending";
  }

  if (hunks.every((hunk) => hunk.decision === "accepted")) {
    return "approved";
  }

  if (hunks.every((hunk) => hunk.decision === "rejected")) {
    return "rejected";
  }

  if (hunks.some((hunk) => hunk.decision === "pending")) {
    return "pending";
  }

  return "modified";
}

function computeFinalPatch(review: DiffReview): string {
  let patch = "";

  for (const file of review.files) {
    const acceptedHunks = file.hunks.filter(
      (hunk) => hunk.decision === "accepted"
    );

    if (acceptedHunks.length === 0) {
      continue;
    }

    patch += `--- a/${file.path}\n`;
    patch += `+++ b/${file.path}\n`;

    for (const hunk of acceptedHunks) {
      patch += `@@ -${hunk.oldStart},${hunk.oldLines} +${hunk.newStart},${hunk.newLines} @@\n`;

      for (const diffLine of hunk.lines) {
        if (diffLine.type === "context") {
          patch += ` ${diffLine.text}\n`;
        } else if (diffLine.type === "delete") {
          patch += `-${diffLine.text}\n`;
        } else if (diffLine.type === "add") {
          patch += `+${diffLine.text}\n`;
        }
      }
    }
  }

  return patch;
}

export function loadMockDiff(): void {
  const review = createMockDiff();

  set({
    review,
    finalPatch: computeFinalPatch(review)
  });
}

export function clearDiff(): void {
  set({
    review: null,
    finalPatch: ""
  });
}

export function setActiveDiffFile(filePath: string): void {
  const state = diffStore.get();

  if (!state.review) {
    return;
  }

  set({
    review: {
      ...state.review,
      activeFilePath: filePath
    }
  });
}

export function setHunkDecision(
  filePath: string,
  hunkId: string,
  decision: HunkDecision
): void {
  const state = diffStore.get();

  if (!state.review) {
    return;
  }

  const files = state.review.files.map((file) => {
    if (file.path !== filePath) {
      return file;
    }

    const hunks = file.hunks.map((hunk) => {
      if (hunk.id !== hunkId) {
        return hunk;
      }

      return {
        ...hunk,
        decision
      };
    });

    return {
      ...file,
      hunks,
      status: computeFileStatus(hunks)
    };
  });

  const review: DiffReview = {
    ...state.review,
    files
  };

  set({
    review,
    finalPatch: computeFinalPatch(review)
  });
}
