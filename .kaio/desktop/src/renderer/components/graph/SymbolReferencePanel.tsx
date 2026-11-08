// src/renderer/components/graph/SymbolReferencePanel.tsx
import { useStore } from "@nanostores/react";
import { graphStore, selectGraphNode } from "@renderer/stores/graph";
import type {
  GraphEdgeDef,
  GraphNodeDef
} from "@renderer/lib/graph-types";

interface ReferenceListProps {
  title: string;
  edges: GraphEdgeDef[];
  nodesById: Map<string, GraphNodeDef>;
  direction: "out" | "in";
}

function ReferenceList({
  title,
  edges,
  nodesById,
  direction
}: ReferenceListProps) {
  if (edges.length === 0) {
    return (
      <div>
        <div className="label">{title}</div>

        <div
          className="mono"
          style={{
            fontSize: "var(--fs-xs)",
            color: "var(--fg-mute)"
          }}
        >
          No {title.toLowerCase()} references.
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="label">{title}</div>

      <div className="codemap-ref-list">
        {edges.map((edge) => {
          const otherId =
            direction === "out" ? edge.target : edge.source;

          const other = nodesById.get(otherId);

          return (
            <button
              key={edge.id}
              type="button"
              className="codemap-ref"
              onClick={() => selectGraphNode(otherId)}
            >
              <span>{other?.label ?? otherId}</span>

              <span className="codemap-ref-kind">{edge.kind}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

export function SymbolReferencePanel() {
  const state = useStore(graphStore);

  const selectedNode = state.nodes.find(
    (node) => node.id === state.selectedNodeId
  );

  if (!selectedNode) {
    return (
      <div className="codemap-empty">
        <p className="lead">Select a symbol node.</p>

        <p
          className="mono"
          style={{
            fontSize: "var(--fs-xs)",
            color: "var(--fg-mute)"
          }}
        >
          References and relationships appear here.
        </p>
      </div>
    );
  }

  const nodesById = new Map(
    state.nodes.map((node) => [node.id, node])
  );

  const outgoing = state.edges.filter(
    (edge) => edge.source === selectedNode.id
  );

  const incoming = state.edges.filter(
    (edge) => edge.target === selectedNode.id
  );

  return (
    <>
      <div className="codemap-panel-head">
        <span className="label label--accent">Symbol</span>

        <span className="label">{selectedNode.kind}</span>
      </div>

      <div className="codemap-panel-body hud-scrollbar">
        <div className="codemap-symbol-title h3">
          {selectedNode.label}
        </div>

        {selectedNode.filePath && (
          <div className="mono codemap-meta">
            {selectedNode.filePath}
          </div>
        )}

        {selectedNode.description && (
          <p className="lead">{selectedNode.description}</p>
        )}

        <ReferenceList
          title="Outgoing"
          edges={outgoing}
          nodesById={nodesById}
          direction="out"
        />

        <ReferenceList
          title="Incoming"
          edges={incoming}
          nodesById={nodesById}
          direction="in"
        />
      </div>
    </>
  );
}
