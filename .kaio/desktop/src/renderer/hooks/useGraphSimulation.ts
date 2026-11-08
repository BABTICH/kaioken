// src/renderer/hooks/useGraphSimulation.ts
import { useCallback, useEffect, useRef, useState } from "react";
import type {
  GraphEdgeDef,
  GraphNodeDef
} from "@renderer/lib/graph-types";

export const GRAPH_WIDTH = 1000;
export const GRAPH_HEIGHT = 700;

export const GRAPH_CENTER_X = GRAPH_WIDTH / 2;
export const GRAPH_CENTER_Y = GRAPH_HEIGHT / 2;

const REPULSION = 6800;
const LINK_DISTANCE = 135;
const LINK_STRENGTH = 0.09;
const CENTER_STRENGTH = 0.018;
const DAMPING = 0.82;
const ALPHA_DECAY = 0.988;
const ALPHA_MIN = 0.003;

export interface SimNode {
  id: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  fx: number | null;
  fy: number | null;
}

export type GraphPositions = Record<
  string,
  {
    x: number;
    y: number;
  }
>;

export interface GraphSimulationResult {
  positions: GraphPositions;
  dragStart: (id: string, worldX: number, worldY: number) => void;
  dragMove: (id: string, worldX: number, worldY: number) => void;
  dragEnd: (id: string) => void;
  reheat: (alpha?: number) => void;
}

export function useGraphSimulation(
  nodes: GraphNodeDef[],
  edges: GraphEdgeDef[],
  layoutKey: number
): GraphSimulationResult {
  const [positions, setPositions] = useState<GraphPositions>({});
  const simNodesRef = useRef<Map<string, SimNode>>(new Map());
  const alphaRef = useRef(1);
  const runningRef = useRef(false);
  const draggedNodesRef = useRef<Set<string>>(new Set());
  const frameIdRef = useRef<number>(0);
  const edgesRef = useRef<GraphEdgeDef[]>(edges);
  edgesRef.current = edges;

  const startLoop = useCallback(() => {
    if (runningRef.current) return;
    runningRef.current = true;

    const tick = () => {
      if (!runningRef.current) return;

      const simNodes = simNodesRef.current;
      const list = [...simNodes.values()];
      const alpha = alphaRef.current;
      const currentEdges = edgesRef.current;

      // 1. Repulsion between all node pairs
      for (let i = 0; i < list.length; i++) {
        const a = list[i];
        for (let j = i + 1; j < list.length; j++) {
          const b = list[j];
          let dx = b.x - a.x;
          let dy = b.y - a.y;
          let dist2 = dx * dx + dy * dy;

          if (dist2 < 1) {
            dx = (Math.random() - 0.5) * 2;
            dy = (Math.random() - 0.5) * 2;
            dist2 = 1;
          }

          const dist = Math.sqrt(dist2);
          const force = (REPULSION / dist2) * alpha;
          const fx = (dx / dist) * force;
          const fy = (dy / dist) * force;

          a.vx -= fx;
          a.vy -= fy;
          b.vx += fx;
          b.vy += fy;
        }
      }

      // 2. Spring force along edges
      for (const edge of currentEdges) {
        const source = simNodes.get(edge.source);
        const target = simNodes.get(edge.target);
        if (!source || !target) continue;

        const dx = target.x - source.x;
        const dy = target.y - source.y;
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const force = (dist - LINK_DISTANCE) * LINK_STRENGTH * alpha;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;

        source.vx += fx;
        source.vy += fy;
        target.vx -= fx;
        target.vy -= fy;
      }

      // 3. Centering gravity, damping, and integration
      for (const node of list) {
        node.vx += (GRAPH_CENTER_X - node.x) * CENTER_STRENGTH * alpha;
        node.vy += (GRAPH_CENTER_Y - node.y) * CENTER_STRENGTH * alpha;

        node.vx *= DAMPING;
        node.vy *= DAMPING;

        if (node.fx !== null) {
          node.x = node.fx;
          node.y = node.fy!;
          node.vx = 0;
          node.vy = 0;
        } else {
          node.x += node.vx;
          node.y += node.vy;
        }
      }

      // Cool alpha down if no nodes are actively being dragged
      if (draggedNodesRef.current.size === 0) {
        alphaRef.current *= ALPHA_DECAY;
      }

      // Publish positions
      const next: GraphPositions = {};
      for (const node of list) {
        next[node.id] = { x: node.x, y: node.y };
      }
      setPositions(next);

      // Keep running if warm or if dragging
      if (alphaRef.current > ALPHA_MIN || draggedNodesRef.current.size > 0) {
        frameIdRef.current = requestAnimationFrame(tick);
      } else {
        runningRef.current = false;
      }
    };

    frameIdRef.current = requestAnimationFrame(tick);
  }, []);

  const reheat = useCallback(
    (alpha = 0.4) => {
      alphaRef.current = Math.max(alphaRef.current, alpha);
      startLoop();
    },
    [startLoop]
  );

  const dragStart = useCallback(
    (id: string, worldX: number, worldY: number) => {
      const node = simNodesRef.current.get(id);
      if (!node) return;
      node.fx = worldX;
      node.fy = worldY;
      node.x = worldX;
      node.y = worldY;
      node.vx = 0;
      node.vy = 0;
      draggedNodesRef.current.add(id);
      reheat(0.35);
    },
    [reheat]
  );

  const dragMove = useCallback(
    (id: string, worldX: number, worldY: number) => {
      const node = simNodesRef.current.get(id);
      if (!node) return;
      node.fx = worldX;
      node.fy = worldY;
      node.x = worldX;
      node.y = worldY;
      node.vx = 0;
      node.vy = 0;
      reheat(0.35);
    },
    [reheat]
  );

  const dragEnd = useCallback(
    (id: string) => {
      const node = simNodesRef.current.get(id);
      if (node) {
        node.fx = null;
        node.fy = null;
      }
      draggedNodesRef.current.delete(id);
      reheat(0.25);
    },
    [reheat]
  );

  // Initialize or re-layout
  useEffect(() => {
    if (nodes.length === 0) {
      simNodesRef.current.clear();
      setPositions({});
      return;
    }

    const simNodes = simNodesRef.current;
    const isRelayout = layoutKey > 0;

    nodes.forEach((node, index) => {
      const existing = simNodes.get(node.id);
      if (existing && !isRelayout) {
        return;
      }

      const angle = (index / nodes.length) * Math.PI * 2;
      const radius = 170 + (index % 5) * 35;
      simNodes.set(node.id, {
        id: node.id,
        x: GRAPH_CENTER_X + Math.cos(angle) * radius + (Math.random() - 0.5) * 20,
        y: GRAPH_CENTER_Y + Math.sin(angle) * radius + (Math.random() - 0.5) * 20,
        vx: (Math.random() - 0.5) * 2,
        vy: (Math.random() - 0.5) * 2,
        fx: null,
        fy: null
      });
    });

    // Clean up stale nodes
    for (const id of simNodes.keys()) {
      if (!nodes.some((n) => n.id === id)) {
        simNodes.delete(id);
      }
    }

    // Publish initial placement immediately
    const initialPos: GraphPositions = {};
    for (const n of simNodes.values()) {
      initialPos[n.id] = { x: n.x, y: n.y };
    }
    setPositions(initialPos);

    // Warm up simulation
    alphaRef.current = 1;
    startLoop();

    return () => {
      cancelAnimationFrame(frameIdRef.current);
      runningRef.current = false;
    };
  }, [nodes, layoutKey, startLoop]);

  return {
    positions,
    dragStart,
    dragMove,
    dragEnd,
    reheat
  };
}
