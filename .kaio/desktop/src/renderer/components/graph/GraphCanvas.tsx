// src/renderer/components/graph/GraphCanvas.tsx
import { useCallback, useRef, useState } from "react";
import { useStore } from "@nanostores/react";
import { graphStore, selectGraphNode } from "@renderer/stores/graph";
import {
  GRAPH_WIDTH,
  GRAPH_HEIGHT,
  useGraphSimulation
} from "@renderer/hooks/useGraphSimulation";
import { ZoomIn, ZoomOut, Maximize2, Move } from "lucide-react";

export function GraphCanvas() {
  const state = useStore(graphStore);
  const svgRef = useRef<SVGSVGElement | null>(null);

  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const [isPanning, setIsPanning] = useState(false);
  const [draggingId, setDraggingId] = useState<string | null>(null);

  const dragRef = useRef<{
    id: string;
    startX: number;
    startY: number;
    moved: boolean;
  } | null>(null);

  const panRef = useRef<{
    startX: number;
    startY: number;
    panStart: { x: number; y: number };
    moved: boolean;
  } | null>(null);

  const { positions, dragStart, dragMove, dragEnd } = useGraphSimulation(
    state.nodes,
    state.edges,
    state.layoutKey
  );

  const getWorldCoords = useCallback(
    (clientX: number, clientY: number) => {
      if (!svgRef.current) return { x: 500, y: 350 };
      const rect = svgRef.current.getBoundingClientRect();
      const svgX = ((clientX - rect.left) / rect.width) * GRAPH_WIDTH;
      const svgY = ((clientY - rect.top) / rect.height) * GRAPH_HEIGHT;
      return {
        x: (svgX - pan.x) / zoom,
        y: (svgY - pan.y) / zoom
      };
    },
    [pan, zoom]
  );

  // Node Drag Handler (Initiation)
  const handleNodePointerDown = (
    e: React.PointerEvent<SVGGElement>,
    nodeId: string
  ) => {
    e.stopPropagation();

    if (svgRef.current) {
      try {
        svgRef.current.setPointerCapture(e.pointerId);
      } catch {
        // Ignored
      }
    }

    const world = getWorldCoords(e.clientX, e.clientY);
    dragRef.current = {
      id: nodeId,
      startX: e.clientX,
      startY: e.clientY,
      moved: false
    };

    setDraggingId(nodeId);
    dragStart(nodeId, world.x, world.y);
  };

  // Canvas Pan Handler (Initiation)
  const handleCanvasPointerDown = (e: React.PointerEvent<SVGSVGElement>) => {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | SVGElement | null;
    if (target && target.closest(".graph-node")) {
      return;
    }

    if (svgRef.current) {
      try {
        svgRef.current.setPointerCapture(e.pointerId);
      } catch {
        // Ignored
      }
    }

    panRef.current = {
      startX: e.clientX,
      startY: e.clientY,
      panStart: { ...pan },
      moved: false
    };
    setIsPanning(true);
  };

  // Combined Pointer Move
  const handlePointerMove = (e: React.PointerEvent<SVGSVGElement>) => {
    if (dragRef.current) {
      const dx = e.clientX - dragRef.current.startX;
      const dy = e.clientY - dragRef.current.startY;
      if (dx * dx + dy * dy > 9) {
        dragRef.current.moved = true;
      }

      const world = getWorldCoords(e.clientX, e.clientY);
      dragMove(dragRef.current.id, world.x, world.y);
      return;
    }

    if (panRef.current && svgRef.current) {
      const rect = svgRef.current.getBoundingClientRect();
      const dx = e.clientX - panRef.current.startX;
      const dy = e.clientY - panRef.current.startY;

      if (dx * dx + dy * dy > 9) {
        panRef.current.moved = true;
      }

      const svgDx = (dx / rect.width) * GRAPH_WIDTH;
      const svgDy = (dy / rect.height) * GRAPH_HEIGHT;

      setPan({
        x: panRef.current.panStart.x + svgDx,
        y: panRef.current.panStart.y + svgDy
      });
    }
  };

  // Combined Pointer Up
  const handlePointerUp = (e: React.PointerEvent<SVGSVGElement>) => {
    if (svgRef.current) {
      try {
        svgRef.current.releasePointerCapture(e.pointerId);
      } catch {
        // Ignored
      }
    }

    if (dragRef.current) {
      const { id, moved } = dragRef.current;
      dragEnd(id);
      if (!moved) {
        selectGraphNode(id);
      }
      dragRef.current = null;
      setDraggingId(null);
      return;
    }

    if (panRef.current) {
      if (!panRef.current.moved) {
        selectGraphNode(null);
      }
      panRef.current = null;
      setIsPanning(false);
    }
  };

  // Zoom Handler
  const handleWheel = (e: React.WheelEvent<SVGSVGElement>) => {
    e.preventDefault();
    if (!svgRef.current) return;

    const rect = svgRef.current.getBoundingClientRect();
    const svgX = ((e.clientX - rect.left) / rect.width) * GRAPH_WIDTH;
    const svgY = ((e.clientY - rect.top) / rect.height) * GRAPH_HEIGHT;

    const factor = Math.pow(1.0015, -e.deltaY);
    const nextZoom = Math.min(3.5, Math.max(0.35, zoom * factor));
    const scale = nextZoom / zoom;

    const nextPanX = svgX - (svgX - pan.x) * scale;
    const nextPanY = svgY - (svgY - pan.y) * scale;

    setZoom(nextZoom);
    setPan({ x: nextPanX, y: nextPanY });
  };

  const handleDoubleClick = () => {
    setPan({ x: 0, y: 0 });
    setZoom(1);
  };

  const zoomIn = () => {
    const nextZoom = Math.min(3.5, zoom * 1.25);
    const scale = nextZoom / zoom;
    setPan((prev) => ({
      x: GRAPH_WIDTH / 2 - (GRAPH_WIDTH / 2 - prev.x) * scale,
      y: GRAPH_HEIGHT / 2 - (GRAPH_HEIGHT / 2 - prev.y) * scale
    }));
    setZoom(nextZoom);
  };

  const zoomOut = () => {
    const nextZoom = Math.max(0.35, zoom / 1.25);
    const scale = nextZoom / zoom;
    setPan((prev) => ({
      x: GRAPH_WIDTH / 2 - (GRAPH_WIDTH / 2 - prev.x) * scale,
      y: GRAPH_HEIGHT / 2 - (GRAPH_HEIGHT / 2 - prev.y) * scale
    }));
    setZoom(nextZoom);
  };

  const resetView = () => {
    setPan({ x: 0, y: 0 });
    setZoom(1);
  };

  if (state.nodes.length === 0) {
    return (
      <div className="codemap-empty">
        <p className="lead">No graph loaded.</p>
      </div>
    );
  }

  return (
    <div className="relative h-full w-full select-none overflow-hidden">
      <svg
        ref={svgRef}
        className={[
          "graph-svg",
          isPanning ? "is-panning" : ""
        ].join(" ")}
        viewBox={`0 0 ${GRAPH_WIDTH} ${GRAPH_HEIGHT}`}
        role="img"
        aria-label="Kaioken symbol graph"
        onPointerDown={handleCanvasPointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        onWheel={handleWheel}
        onDoubleClick={handleDoubleClick}
      >
        <g transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}>
          {/* Edges */}
          {state.edges.map((edge) => {
            const source = positions[edge.source];
            const target = positions[edge.target];

            if (!source || !target) {
              return null;
            }

            const isActive =
              state.selectedNodeId !== null &&
              (edge.source === state.selectedNodeId ||
                edge.target === state.selectedNodeId);

            return (
              <line
                key={edge.id}
                x1={source.x}
                y1={source.y}
                x2={target.x}
                y2={target.y}
                className={[
                  "graph-edge",
                  isActive ? "is-active" : ""
                ].join(" ")}
              />
            );
          })}

          {/* Nodes (Particles) */}
          {state.nodes.map((node) => {
            const position = positions[node.id];

            if (!position) {
              return null;
            }

            const isSelected = node.id === state.selectedNodeId;
            const isDragging = node.id === draggingId;

            return (
              <g
                key={node.id}
                transform={`translate(${position.x}, ${position.y})`}
                className={[
                  "graph-node",
                  `graph-node-${node.kind}`,
                  isSelected ? "is-selected" : "",
                  isDragging ? "is-dragging" : ""
                ].join(" ")}
                onPointerDown={(e) => handleNodePointerDown(e, node.id)}
                onClick={(e) => {
                  e.stopPropagation();
                  selectGraphNode(node.id);
                }}
              >
                {/* Node Box */}
                <rect
                  x={-8}
                  y={-8}
                  width={16}
                  height={16}
                  rx={2}
                  className="graph-node-box"
                />

                {/* Node Label */}
                <text
                  y={-14}
                  textAnchor="middle"
                  className="graph-node-label"
                >
                  {node.label}
                </text>
              </g>
            );
          })}
        </g>
      </svg>

      {/* Interactive HUD / Navigation Controls */}
      <div className="pointer-events-none absolute bottom-4 left-4 flex items-center gap-1.5">
        <div className="pointer-events-auto flex items-center gap-1 rounded border border-[var(--rule)] bg-[var(--bg-raise)] p-1 backdrop-blur shadow-sm">
          <button
            type="button"
            className="k-btn k-btn--icon h-7 w-7 text-xs"
            onClick={zoomIn}
            title="Zoom In"
            aria-label="Zoom in"
          >
            <ZoomIn size={13} />
          </button>

          <button
            type="button"
            className="k-btn k-btn--icon h-7 w-7 text-xs"
            onClick={zoomOut}
            title="Zoom Out"
            aria-label="Zoom out"
          >
            <ZoomOut size={13} />
          </button>

          <button
            type="button"
            className="k-btn k-btn--icon h-7 w-7 text-xs"
            onClick={resetView}
            title="Reset Pan & Zoom (Double Click)"
            aria-label="Reset view"
          >
            <Maximize2 size={13} />
          </button>

          <span className="mono px-1.5 text-[10px] text-[var(--fg-mute)]">
            {Math.round(zoom * 100)}%
          </span>
        </div>

        <div className="hidden sm:flex items-center gap-1 rounded border border-[var(--rule)] bg-[color-mix(in_srgb,var(--bg)_85%,transparent)] px-2 py-1 text-[11px] text-[var(--fg-mute)]">
          <Move size={11} className="text-[var(--accent)]" />
          <span>Drag particles to move &bull; Scroll to zoom</span>
        </div>
      </div>
    </div>
  );
}
