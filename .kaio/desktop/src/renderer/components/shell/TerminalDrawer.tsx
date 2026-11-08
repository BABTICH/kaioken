// src/renderer/components/shell/TerminalDrawer.tsx
import { useState, useRef, useEffect } from "react";
import { useStore } from "@nanostores/react";
import {
  closeActiveTerminal,
  createTerminal,
  setActiveTerminal,
  terminalStore
} from "@renderer/stores/terminal";
import { toggleTerminal } from "@renderer/stores/ui";
import { TerminalPane } from "@renderer/components/terminal/TerminalPane";
import { Terminal as TerminalIcon, Plus, X, ChevronDown } from "lucide-react";

function shellLabel(shell: string): string {
  return shell.split(/[\\/]/).pop() ?? shell;
}

const DEFAULT_TERMINAL_HEIGHT = 288;
const MIN_TERMINAL_HEIGHT = 120;
const MAX_TERMINAL_HEIGHT = 700;

export function TerminalDrawer() {
  const terminal = useStore(terminalStore);

  const [height, setHeight] = useState<number>(() => {
    try {
      const saved = localStorage.getItem("kaioken_terminal_height");
      if (saved) {
        const parsed = Number.parseInt(saved, 10);
        if (
          !Number.isNaN(parsed) &&
          parsed >= MIN_TERMINAL_HEIGHT &&
          parsed <= MAX_TERMINAL_HEIGHT
        ) {
          return parsed;
        }
      }
    } catch {
      // ignore
    }
    return DEFAULT_TERMINAL_HEIGHT;
  });

  const [isResizing, setIsResizing] = useState(false);
  const dragRef = useRef<{ startY: number; startHeight: number } | null>(null);
  const heightRef = useRef(height);
  heightRef.current = height;

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    dragRef.current = {
      startY: e.clientY,
      startHeight: height
    };
    setIsResizing(true);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    const delta = dragRef.current.startY - e.clientY; // dragging up increases height
    const nextHeight = Math.min(
      MAX_TERMINAL_HEIGHT,
      Math.max(MIN_TERMINAL_HEIGHT, dragRef.current.startHeight + delta)
    );
    setHeight(nextHeight);
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
      localStorage.setItem("kaioken_terminal_height", String(heightRef.current));
    } catch {
      // ignore
    }
  };

  const handleDoubleClick = () => {
    setHeight(DEFAULT_TERMINAL_HEIGHT);
    try {
      localStorage.setItem(
        "kaioken_terminal_height",
        String(DEFAULT_TERMINAL_HEIGHT)
      );
    } catch {
      // ignore
    }
  };

  // Auto-launch terminal if drawer is open and no sessions exist
  useEffect(() => {
    if (
      terminal.sessions.length === 0 &&
      !terminal.creating &&
      !terminal.error
    ) {
      void createTerminal();
    }
  }, [terminal.sessions.length, terminal.creating, terminal.error]);

  const active =
    terminal.sessions.find(
      (session) => session.id === terminal.activeId
    ) ?? null;

  return (
    <section
      className="term relative select-none"
      style={{
        height: `${height}px`,
        borderTop: "var(--hair) solid var(--rule)",
        background: "var(--bg-sunk)",
        borderRadius: "var(--r-none)",
        transition: isResizing ? "none" : "height 0.1s ease-out"
      }}
    >
      {/* Top resize handle */}
      <div
        className={`absolute top-0 left-0 right-0 h-2 -translate-y-1/2 cursor-row-resize z-30 transition-colors ${
          isResizing ? "bg-[var(--accent)]" : "hover:bg-[var(--accent)]"
        }`}
        style={{ touchAction: "none" }}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onDoubleClick={handleDoubleClick}
        title="Drag to resize Terminal (double-click to reset)"
      />

      <div className="term-bar">
        <div className="tabs">
          {terminal.sessions.map((session) => {
            const isActive = session.id === terminal.activeId;

            return (
              <button
                key={session.id}
                type="button"
                className={`tab ${isActive ? "on" : ""} inline-flex items-center gap-1.5`}
                onClick={() => setActiveTerminal(session.id)}
              >
                <TerminalIcon size={12} className="opacity-80 shrink-0" />
                <span>{shellLabel(session.shell)}</span>
              </button>
            );
          })}

          <button
            type="button"
            className="tab inline-flex items-center justify-center"
            title="Create new terminal tab"
            aria-label="New terminal"
            disabled={terminal.creating}
            onClick={() => {
              void createTerminal();
            }}
          >
            <Plus size={13} />
          </button>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "var(--s-2)"
          }}
        >
          {active && (
            <span className="term-status">
              <i />
              {active.running ? "RUNNING" : "EXITED"}
            </span>
          )}

          <button
            type="button"
            className="btn btn--sm inline-flex items-center gap-1"
            disabled={terminal.creating}
            onClick={() => {
              void createTerminal();
            }}
          >
            <Plus size={12} />
            <span>New</span>
          </button>

          {active && (
            <button
              type="button"
              className="btn btn--sm inline-flex items-center gap-1"
              onClick={() => {
                void closeActiveTerminal();
              }}
            >
              <X size={12} />
              <span>Close</span>
            </button>
          )}

          <button
            type="button"
            className="btn btn--sm inline-flex items-center justify-center"
            title="Minimize terminal drawer (Ctrl+`)"
            aria-label="Minimize terminal drawer"
            onClick={() => {
              toggleTerminal();
            }}
          >
            <ChevronDown size={14} />
          </button>
        </div>
      </div>

      <div
        style={{
          height: "calc(100% - 35px)"
        }}
      >
        {active ? (
          <TerminalPane sessionId={active.id} />
        ) : (
          <div
            className="mono flex flex-col items-center justify-center h-full gap-3"
            style={{
              padding: "var(--s-4)",
              fontSize: "var(--fs-xs)",
              color: "var(--fg-mute)",
              whiteSpace: "pre-wrap"
            }}
          >
            <div>
              {terminal.error
                ? `Terminal initialization error: ${terminal.error}`
                : "No active terminal session."}
            </div>
            <button
              type="button"
              className="btn btn--accent inline-flex items-center gap-1.5"
              disabled={terminal.creating}
              onClick={() => {
                void createTerminal();
              }}
            >
              <Plus size={14} />
              <span>{terminal.creating ? "Starting..." : "Launch Terminal"}</span>
            </button>
          </div>
        )}
      </div>
    </section>
  );
}
