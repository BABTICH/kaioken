// src/renderer/components/diff/DiffHunkView.tsx
import { useEffect, useRef } from "react";
import { EditorState } from "@codemirror/state";
import {
  Decoration,
  EditorView,
  lineNumbers
} from "@codemirror/view";
import type { DiffHunk } from "@shared/types/diff";

const baseTheme = EditorView.theme(
  {
    "&": {
      backgroundColor: "transparent",
      color: "var(--fg)",
      fontSize: "12px"
    },
    ".cm-content": {
      fontFamily: "var(--font-mono)",
      caretColor: "var(--accent)"
    },
    ".cm-gutters": {
      backgroundColor: "var(--bg-raise)",
      color: "var(--fg-mute)",
      borderRight: "var(--hair) solid var(--rule)",
      paddingLeft: "4px"
    },
    ".cm-lineNumbers .cm-gutterElement": {
      padding: "0 6px",
      minWidth: "28px",
      textAlign: "right"
    },
    ".cm-line": {
      padding: "0 6px"
    },
    ".diff-line-del": {
      backgroundColor: "var(--accent-soft)"
    },
    ".diff-line-add": {
      backgroundColor: "var(--ember-soft)"
    }
  },
  { dark: true }
);

function createDiffEditor(
  parent: HTMLElement,
  lines: string[],
  startLine: number,
  changedLines: number[],
  changedClass: string
): EditorView {
  const decorationExtension = EditorView.decorations.compute(
    ["doc"],
    (state) => {
      const ranges = changedLines
        .filter((ln) => ln >= 1 && ln <= state.doc.lines)
        .map((ln) =>
          Decoration.line({
            class: changedClass
          }).range(state.doc.line(ln).from)
        );

      return Decoration.set(ranges);
    }
  );

  const state = EditorState.create({
    doc: lines.join("\n"),
    extensions: [
      lineNumbers({
        formatNumber: (lineNumber) =>
          String(startLine + lineNumber - 1)
      }),
      EditorState.readOnly.of(true),
      EditorView.editable.of(false),
      baseTheme,
      decorationExtension
    ]
  });

  return new EditorView({
    parent,
    state
  });
}

interface DiffHunkViewProps {
  hunk: DiffHunk;
  filePath: string;
  onAccept: () => void;
  onReject: () => void;
}

export function DiffHunkView({
  hunk,
  filePath,
  onAccept,
  onReject
}: DiffHunkViewProps) {
  const leftRef = useRef<HTMLDivElement | null>(null);
  const rightRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const leftLines: string[] = [];
    const leftChanged: number[] = [];

    const rightLines: string[] = [];
    const rightChanged: number[] = [];

    let leftLineNo = 0;
    let rightLineNo = 0;

    for (const diffLine of hunk.lines) {
      if (diffLine.type !== "add") {
        leftLineNo += 1;
        leftLines.push(diffLine.text);

        if (diffLine.type === "delete") {
          leftChanged.push(leftLineNo);
        }
      }

      if (diffLine.type !== "delete") {
        rightLineNo += 1;
        rightLines.push(diffLine.text);

        if (diffLine.type === "add") {
          rightChanged.push(rightLineNo);
        }
      }
    }

    const leftView = leftRef.current
      ? createDiffEditor(
          leftRef.current,
          leftLines,
          hunk.oldStart,
          leftChanged,
          "diff-line-del"
        )
      : null;

    const rightView = rightRef.current
      ? createDiffEditor(
          rightRef.current,
          rightLines,
          hunk.newStart,
          rightChanged,
          "diff-line-add"
        )
      : null;

    return () => {
      leftView?.destroy();
      rightView?.destroy();
    };
  }, [hunk.id, hunk.lines, hunk.oldStart, hunk.newStart]);

  return (
    <section className={`diff-hunk diff-hunk-${hunk.decision}`}>
      <div className="diff-hunk-head">
        <span className="diff-hunk-range">
          @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},
          {hunk.newLines} @@ {filePath}
        </span>

        <div className="diff-hunk-actions">
          <span className={`diff-hunk-decision ${hunk.decision}`}>
            {hunk.decision}
          </span>

          <button
            type="button"
            className={[
              "btn",
              "btn--sm",
              hunk.decision === "accepted" ? "btn--ember" : ""
            ].join(" ")}
            onClick={onAccept}
          >
            Accept
          </button>

          <button
            type="button"
            className={[
              "btn",
              "btn--sm",
              hunk.decision === "rejected" ? "btn--danger" : ""
            ].join(" ")}
            onClick={onReject}
          >
            Reject
          </button>
        </div>
      </div>

      <div className="diff-hunk-body">
        <div className="diff-hunk-grid">
          <div className="diff-pane">
            <div className="diff-pane-label">Original</div>
            <div ref={leftRef} className="diff-pane-editor" />
          </div>

          <div className="diff-pane">
            <div className="diff-pane-label">Revised</div>
            <div ref={rightRef} className="diff-pane-editor" />
          </div>
        </div>
      </div>
    </section>
  );
}
