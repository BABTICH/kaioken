// src/renderer/components/shell/InspectorPanel.tsx
import { useState, useRef } from "react";
import { useStore } from "@nanostores/react";
import {
  inspectorOpen,
  inspectorTab,
  setInspectorTab,
  toggleInspector,
  type InspectorTab
} from "@renderer/stores/ui";
import { DiffInspector } from "@renderer/components/diff/DiffInspector";
import {
  GitCompare,
  Network,
  FileCode,
  Hash,
  X
} from "lucide-react";

interface TabDef {
  id: InspectorTab;
  label: string;
  icon: React.ComponentType<{ size?: number; className?: string }>;
}

const tabs: TabDef[] = [
  { id: "diff", label: "Diff", icon: GitCompare },
  { id: "graph", label: "Graph", icon: Network },
  { id: "mermaid", label: "MD", icon: FileCode },
  { id: "symbols", label: "Sym", icon: Hash }
];

const DEFAULT_INSPECTOR_WIDTH = 320;
const MIN_INSPECTOR_WIDTH = 220;
const MAX_INSPECTOR_WIDTH = 750;

export function InspectorPanel() {
  const open = useStore(inspectorOpen);
  const activeTab = useStore(inspectorTab);

  const [width, setWidth] = useState<number>(() => {
    try {
      const saved = localStorage.getItem("kaioken_inspector_width");
      if (saved) {
        const parsed = Number.parseInt(saved, 10);
        if (
          !Number.isNaN(parsed) &&
          parsed >= MIN_INSPECTOR_WIDTH &&
          parsed <= MAX_INSPECTOR_WIDTH
        ) {
          return parsed;
        }
      }
    } catch {
      // ignore
    }
    return DEFAULT_INSPECTOR_WIDTH;
  });

  const [isResizing, setIsResizing] = useState(false);
  const dragRef = useRef<{ startX: number; startWidth: number } | null>(null);
  const widthRef = useRef(width);
  widthRef.current = width;

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    dragRef.current = {
      startX: e.clientX,
      startWidth: width
    };
    setIsResizing(true);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    const delta = dragRef.current.startX - e.clientX; // dragging left increases width
    const nextWidth = Math.min(
      MAX_INSPECTOR_WIDTH,
      Math.max(MIN_INSPECTOR_WIDTH, dragRef.current.startWidth + delta)
    );
    setWidth(nextWidth);
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    try {
      e.currentTarget.releasePointerCapture(e.pointerId);
    } catch {
      // ignore
    }
    dragRef.current = null;
    setIsResizing(false);
    try {
      localStorage.setItem("kaioken_inspector_width", String(widthRef.current));
    } catch {
      // ignore
    }
  };

  const handleDoubleClick = () => {
    setWidth(DEFAULT_INSPECTOR_WIDTH);
    try {
      localStorage.setItem(
        "kaioken_inspector_width",
        String(DEFAULT_INSPECTOR_WIDTH)
      );
    } catch {
      // ignore
    }
  };

  if (!open) {
    return null;
  }

  return (
    <aside
      className="relative flex shrink-0 flex-col select-none"
      style={{
        width: `${width}px`,
        borderLeft: "var(--hair) solid var(--rule)",
        background: "var(--bg)",
        transition: isResizing ? "none" : "width 0.1s ease-out"
      }}
    >
      {/* Horizontal resize handle on left border */}
      <div
        className={`absolute top-0 bottom-0 left-0 w-2 -translate-x-1/2 cursor-col-resize z-30 transition-colors ${
          isResizing ? "bg-[var(--accent)]" : "hover:bg-[var(--accent)]"
        }`}
        style={{ touchAction: "none" }}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onDoubleClick={handleDoubleClick}
        title="Drag to resize Inspector (double-click to reset)"
      />

      <div
        className="flex items-center justify-between px-3 py-2"
        style={{ borderBottom: "var(--hair) solid var(--rule)" }}
      >
        <div className="flex items-center gap-2">
          <span className="label label--accent">Inspector</span>
          <span
            className="mono"
            style={{
              fontSize: "var(--fs-micro)",
              color: "var(--fg-mute)"
            }}
          >
            CTRL+B
          </span>
        </div>

        <button
          type="button"
          onClick={() => toggleInspector()}
          className="k-btn k-btn--icon text-[var(--fg-mute)] hover:text-[var(--fg)]"
          title="Close Inspector (Ctrl+B)"
          aria-label="Close Inspector"
          style={{ width: "24px", height: "24px", border: "none" }}
        >
          <X size={14} />
        </button>
      </div>

      <div
        className="flex gap-1 p-2"
        style={{ borderBottom: "var(--hair) solid var(--rule)" }}
      >
        {tabs.map((tab) => {
          const isActive = activeTab === tab.id;
          const Icon = tab.icon;

          return (
            <button
              key={tab.id}
              type="button"
              onClick={() => setInspectorTab(tab.id)}
              className={`tab ${isActive ? "on" : ""} inline-flex items-center gap-1.5`}
            >
              <Icon size={12} className="shrink-0" />
              <span>{tab.label}</span>
            </button>
          );
        })}
      </div>

      <div className="hud-scrollbar flex-1 overflow-auto flex flex-col">
        {activeTab === "diff" ? (
          <DiffInspector />
        ) : (
          <div
            className="mono"
            style={{
              padding: "var(--s-4)",
              fontSize: "var(--fs-xs)",
              color: "var(--fg-mute)"
            }}
          >
            Inspector module offline.
            <br />
            {activeTab.toUpperCase()} arrives in Phase 5.
          </div>
        )}
      </div>
    </aside>
  );
}
