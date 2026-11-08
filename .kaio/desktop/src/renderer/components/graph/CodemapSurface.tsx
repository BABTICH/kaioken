// src/renderer/components/graph/CodemapSurface.tsx
import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import {
  graphStore,
  loadMockGraph,
  reheatGraphLayout
} from "@renderer/stores/graph";
import { GraphCanvas } from "./GraphCanvas";
import { SymbolReferencePanel } from "./SymbolReferencePanel";
import { RotateCw, RefreshCcw } from "lucide-react";

export function CodemapSurface() {
  const graph = useStore(graphStore);

  const nodeCount = graph.nodes.length;

  useEffect(() => {
    if (nodeCount === 0) {
      loadMockGraph();
    }
  }, [nodeCount]);

  return (
    <div className="codemap-surface">
      <div className="codemap-toolbar">
        <span className="label label--accent">Codemap</span>

        <span className="label">nodes {graph.nodes.length}</span>
        <span className="label">edges {graph.edges.length}</span>

        <div style={{ flex: 1 }} />

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          onClick={() => loadMockGraph()}
        >
          <RotateCw size={12} />
          <span>Reload mock</span>
        </button>

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          onClick={() => reheatGraphLayout()}
        >
          <RefreshCcw size={12} />
          <span>Re-layout</span>
        </button>
      </div>

      <div className="codemap-body">
        <div className="codemap-canvas tech-grid-bg">
          <GraphCanvas />
        </div>

        <aside className="codemap-panel">
          <SymbolReferencePanel />
        </aside>
      </div>
    </div>
  );
}
