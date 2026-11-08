// src/renderer/components/diff/DiffInspector.tsx
import { useStore } from "@nanostores/react";
import {
  clearDiff,
  diffStore,
  loadMockDiff,
  setActiveDiffFile,
  setHunkDecision
} from "@renderer/stores/diff";
import { DiffHunkView } from "./DiffHunkView";
import { FileCode, X, Play } from "lucide-react";

export function DiffInspector() {
  const state = useStore(diffStore);
  const review = state.review;

  if (!review) {
    return (
      <div className="diff-empty">
        <p className="lead">No active diff review.</p>

        <button
          type="button"
          className="btn btn--primary btn--sm inline-flex items-center gap-1"
          onClick={() => loadMockDiff()}
        >
          <Play size={12} />
          <span>Load mock diff</span>
        </button>
      </div>
    );
  }

  const activeFile =
    review.files.find((file) => file.path === review.activeFilePath) ??
    review.files[0];

  if (!activeFile) {
    return (
      <div className="diff-empty">
        <p className="lead">Diff review has no files.</p>
      </div>
    );
  }

  return (
    <div className="diff-inspector">
      <div className="diff-toolbar">
        <div className="tabs">
          {review.files.map((file) => {
            const isActive = file.path === activeFile.path;

            return (
              <button
                key={file.path}
                type="button"
                className={`tab ${isActive ? "on" : ""} inline-flex items-center gap-1.5`}
                onClick={() => setActiveDiffFile(file.path)}
              >
                <FileCode size={12} className="shrink-0" />
                <span>{file.path.split(/[\\/]/).pop()}</span>
              </button>
            );
          })}
        </div>

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          onClick={() => clearDiff()}
        >
          <X size={12} />
          <span>Clear</span>
        </button>
      </div>

      <div className="diff-file-status">
        <span className="label">file</span>
        <span className="mono">{activeFile.path}</span>

        <span className="label">status</span>
        <span className="mono">{activeFile.status}</span>
      </div>

      {activeFile.hunks.map((hunk) => (
        <DiffHunkView
          key={`${activeFile.path}:${hunk.id}`}
          hunk={hunk}
          filePath={activeFile.path}
          onAccept={() =>
            setHunkDecision(activeFile.path, hunk.id, "accepted")
          }
          onReject={() =>
            setHunkDecision(activeFile.path, hunk.id, "rejected")
          }
        />
      ))}

      <div className="diff-patch">
        <div className="diff-patch-head">
          <span className="label label--accent">Final patch</span>

          <span className="label">
            {state.finalPatch
              ? `${state.finalPatch.split("\n").length} lines`
              : "empty"}
          </span>
        </div>

        <pre className="hud-scrollbar">
          {state.finalPatch || "# No accepted hunks"}
        </pre>
      </div>
    </div>
  );
}
