// src/renderer/components/shell/SidebarPanel.tsx
import { useState, useRef } from "react";
import { useStore } from "@nanostores/react";
import { explorerStore, openWorkspace } from "@renderer/stores/explorer";
import { FileTree } from "@renderer/components/explorer/FileTree";
import { OpenEditors } from "@renderer/components/explorer/OpenEditors";
import { FolderOpen } from "lucide-react";

const DEFAULT_SIDEBAR_WIDTH = 240;
const MIN_SIDEBAR_WIDTH = 140;
const MAX_SIDEBAR_WIDTH = 600;

export function SidebarPanel() {
  const explorer = useStore(explorerStore);

  const [width, setWidth] = useState<number>(() => {
    try {
      const saved = localStorage.getItem("kaioken_sidebar_width");
      if (saved) {
        const parsed = Number.parseInt(saved, 10);
        if (!Number.isNaN(parsed) && parsed >= MIN_SIDEBAR_WIDTH && parsed <= MAX_SIDEBAR_WIDTH) {
          return parsed;
        }
      }
    } catch {
      // ignore
    }
    return DEFAULT_SIDEBAR_WIDTH;
  });

  const [isResizing, setIsResizing] = useState(false);
  const dragRef = useRef<{ startX: number; startWidth: number } | null>(null);
  const widthRef = useRef(width);
  widthRef.current = width;

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    dragRef.current = { startX: e.clientX, startWidth: width };
    setIsResizing(true);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    const delta = e.clientX - dragRef.current.startX;
    const next = Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, dragRef.current.startWidth + delta));
    setWidth(next);
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    try { e.currentTarget.releasePointerCapture(e.pointerId); } catch { /* ignore */ }
    dragRef.current = null;
    setIsResizing(false);
    try { localStorage.setItem("kaioken_sidebar_width", String(widthRef.current)); } catch { /* ignore */ }
  };

  const handleDoubleClick = () => {
    setWidth(DEFAULT_SIDEBAR_WIDTH);
    try { localStorage.setItem("kaioken_sidebar_width", String(DEFAULT_SIDEBAR_WIDTH)); } catch { /* ignore */ }
  };

  return (
    <div
      className="relative flex shrink-0 flex-col select-none"
      style={{
        width: `${width}px`,
        borderRight: "var(--hair) solid var(--rule)",
        background: "var(--bg)",
        transition: isResizing ? "none" : undefined
      }}
    >
      {/* Section header */}
      <div
        className="flex items-center justify-between px-3 py-2"
        style={{
          borderBottom: "var(--hair) solid var(--rule)",
          background: "var(--bg-raise)"
        }}
      >
        <span
          className="k-label k-label--accent"
          style={{ fontSize: "var(--fs-micro)", letterSpacing: "0.14em" }}
        >
          Explorer
        </span>
        {explorer.workspace && (
          <span
            className="mono truncate"
            style={{
              fontSize: "var(--fs-micro)",
              color: "var(--fg-mute)",
              maxWidth: "120px"
            }}
            title={explorer.workspace.root}
          >
            {explorer.workspace.name}
          </span>
        )}
      </div>

      {/* File tree */}
      <div className="hud-scrollbar flex-1 overflow-auto flex flex-col">
        {!explorer.workspace ? (
          <div className="flex flex-col items-center justify-center gap-3 p-4 text-center h-full">
            <FolderOpen size={28} style={{ color: "var(--fg-mute)", opacity: 0.4 }} />
            <div>
              <div
                className="font-medium"
                style={{ fontSize: "var(--fs-xs)", color: "var(--fg-2)" }}
              >
                No folder open
              </div>
              <p
                style={{ fontSize: "var(--fs-micro)", color: "var(--fg-mute)", marginTop: "4px" }}
              >
                Open a folder to see files
              </p>
            </div>
            <button
              type="button"
              className="btn btn--sm inline-flex items-center gap-1.5"
              onClick={() => void openWorkspace()}
            >
              <FolderOpen size={12} />
              <span>Open Folder</span>
            </button>
          </div>
        ) : (
          <>
            <OpenEditors />
            <FileTree />
          </>
        )}
      </div>

      {/* Right resize handle */}
      <div
        className="absolute top-0 bottom-0 right-0 w-1.5 cursor-col-resize z-30 transition-colors"
        style={{
          touchAction: "none",
          background: isResizing ? "var(--accent)" : "transparent"
        }}
        onMouseEnter={(e) => {
          if (!isResizing) (e.currentTarget as HTMLDivElement).style.background = "color-mix(in srgb, var(--accent) 50%, transparent)";
        }}
        onMouseLeave={(e) => {
          if (!isResizing) (e.currentTarget as HTMLDivElement).style.background = "transparent";
        }}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onDoubleClick={handleDoubleClick}
        title="Drag to resize sidebar (double-click to reset)"
      />
    </div>
  );
}
