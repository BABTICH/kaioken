// src/renderer/components/chat/ToolCard.tsx
import { useMemo, useState } from "react";
import { useStore } from "@nanostores/react";
import { toolStore } from "@renderer/stores/tools";
import { loadMockDiff } from "@renderer/stores/diff";
import {
  inspectorOpen,
  setInspectorTab
} from "@renderer/stores/ui";

interface ToolCardProps {
  toolId: string;
}

const DIFF_TOOL_STATES = [
  "awaiting_approval",
  "approved",
  "running",
  "success"
];

export function ToolCard({ toolId }: ToolCardProps) {
  const state = useStore(toolStore);
  const tool = state.byId[toolId];

  const [expanded, setExpanded] = useState(false);

  const outputLineCount = useMemo(() => {
    if (!tool) {
      return 0;
    }

    return tool.output.split("\n").length;
  }, [tool]);

  if (!tool) {
    return null;
  }

  const collapsible =
    outputLineCount > 6 || tool.output.length > 500;

  const collapsed = collapsible && !expanded;

  const stateLabel = tool.state.split("_").join(" ");

  const showDiffAction =
    tool.name === "write_to_file" &&
    DIFF_TOOL_STATES.includes(tool.state);

  const openDiff = () => {
    loadMockDiff();
    setInspectorTab("diff");
    inspectorOpen.set(true);
  };

  return (
    <div className="tool-card">
      <div className="tool-head">
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "var(--s-2)"
          }}
        >
          <span className="label label--accent">{tool.name}</span>

          {tool.risk === "destructive" && (
            <span className="tool-risk">destructive</span>
          )}
        </div>

        <span className={`tool-state tool-state-${tool.state}`}>
          {stateLabel}
        </span>
      </div>

      <div className="tool-body">
        <div className="tool-title">{tool.title}</div>

        {tool.summary && (
          <div className="tool-summary">{tool.summary}</div>
        )}

        {tool.output.length > 0 && (
          <div
            className={[
              "tool-output-wrap",
              collapsed ? "tool-output--collapsed" : ""
            ].join(" ")}
          >
            <pre className="tool-output">{tool.output}</pre>

            {collapsed && <div className="tool-output-fade" />}
          </div>
        )}

        <div className="tool-actions">
          {collapsible && (
            <button
              type="button"
              className="btn btn--sm tool-expand"
              onClick={() => setExpanded((current) => !current)}
            >
              {expanded ? "Collapse output" : "Expand output"}
            </button>
          )}

          {showDiffAction && (
            <button
              type="button"
              className="btn btn--sm"
              onClick={openDiff}
            >
              Inspect diff
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
