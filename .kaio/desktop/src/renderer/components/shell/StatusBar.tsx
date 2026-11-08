// src/renderer/components/shell/StatusBar.tsx
import { useEffect, useState } from "react";
import { useStore } from "@nanostores/react";
import { explorerStore } from "@renderer/stores/explorer";
import { resolvedTheme } from "@renderer/stores/theme";
import { terminalStore } from "@renderer/stores/terminal";
import { activeSurface } from "@renderer/stores/ui";
import {
  GitBranch,
  AlertCircle,
  CheckCircle2,
  ZapOff,
  Zap,
  Sun,
  Moon,
  Terminal
} from "lucide-react";
import { toggleTheme } from "@renderer/stores/theme";
import { toggleTerminal } from "@renderer/stores/ui";

function fileExt(path: string | null): string {
  if (!path) return "";
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  const map: Record<string, string> = {
    ts: "TypeScript",
    tsx: "TypeScript JSX",
    js: "JavaScript",
    jsx: "JavaScript JSX",
    json: "JSON",
    md: "Markdown",
    css: "CSS",
    html: "HTML",
    py: "Python",
    rs: "Rust",
    go: "Go",
    java: "Java",
    sh: "Shell",
    yaml: "YAML",
    yml: "YAML",
    toml: "TOML",
    txt: "Plain Text",
    xml: "XML",
    sql: "SQL"
  };
  return map[ext] ?? (ext ? ext.toUpperCase() : "Plain Text");
}

function fileName(path: string | null): string {
  if (!path) return "";
  return path.split(/[\\/]/).pop() ?? path;
}

export function StatusBar() {
  const explorer = useStore(explorerStore);
  const theme = useStore(resolvedTheme);
  const terminal = useStore(terminalStore);
  const surface = useStore(activeSurface);
  const [branch, setBranch] = useState<string | null>(null);
  const [line, setLine] = useState(1);
  const [col, setCol] = useState(1);

  const activeFile = explorer.activeFilePath;
  const workspace = explorer.workspace;

  // Try to get git branch from workspace info
  useEffect(() => {
    if (workspace?.root) {
      // Try to read from any simple-git or IPC we might have
      // For now show a reasonable default
      setBranch("master");
    } else {
      setBranch(null);
    }
  }, [workspace?.root]);

  const activeSessions = terminal.sessions.filter((s) => s.running).length;

  return (
    <footer
      data-status-bar="true"
      className="flex shrink-0 items-center justify-between select-none"
      style={{
        height: "22px",
        background: "var(--accent)",
        color: "rgba(255,255,255,0.92)",
        fontSize: "11px",
        fontFamily: "var(--font-mono)",
        paddingInline: "4px",
        zIndex: 50
      }}
    >
      {/* Left section */}
      <div className="flex items-center gap-0">
        {/* Branch */}
        {branch && (
          <button
            type="button"
            className="status-item"
            title={`Git branch: ${branch}`}
            style={{ gap: "4px", display: "flex", alignItems: "center", padding: "0 8px", height: "22px", cursor: "pointer", transition: "background 0.1s" }}
            onMouseEnter={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "rgba(0,0,0,0.15)")}
            onMouseLeave={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "transparent")}
          >
            <GitBranch size={12} style={{ opacity: 0.85 }} />
            <span>{branch}</span>
          </button>
        )}

        {/* Workspace name */}
        {workspace && (
          <div
            className="flex items-center gap-1"
            style={{ padding: "0 8px", height: "22px", opacity: 0.85 }}
          >
            <CheckCircle2 size={11} />
            <span>{workspace.name}</span>
          </div>
        )}
      </div>

      {/* Center: surface info */}
      <div style={{ opacity: 0.8, letterSpacing: "0.04em" }}>
        {surface === "welcome"
          ? "WELCOME"
          : surface.toUpperCase()}
      </div>

      {/* Right section */}
      <div className="flex items-center gap-0">
        {/* Active file language */}
        {activeFile && surface !== "welcome" && (
          <div
            style={{
              padding: "0 8px",
              height: "22px",
              display: "flex",
              alignItems: "center",
              opacity: 0.85
            }}
            title={activeFile}
          >
            {fileExt(activeFile)}
          </div>
        )}

        {/* Line / col — would be updated by editor component */}
        {activeFile && surface !== "welcome" && (
          <div
            style={{
              padding: "0 8px",
              height: "22px",
              display: "flex",
              alignItems: "center",
              opacity: 0.85
            }}
          >
            Ln {line}, Col {col}
          </div>
        )}

        {/* Encoding */}
        {activeFile && (
          <div
            style={{
              padding: "0 8px",
              height: "22px",
              display: "flex",
              alignItems: "center",
              opacity: 0.75
            }}
          >
            UTF-8
          </div>
        )}

        {/* Terminal sessions indicator */}
        <button
          type="button"
          onClick={() => toggleTerminal()}
          title={`Terminal (${activeSessions} running)`}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "3px",
            padding: "0 8px",
            height: "22px",
            cursor: "pointer",
            transition: "background 0.1s",
            background: "transparent",
            color: "inherit",
            border: "none"
          }}
          onMouseEnter={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "rgba(0,0,0,0.15)")}
          onMouseLeave={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "transparent")}
        >
          <Terminal size={11} style={{ opacity: 0.85 }} />
          {activeSessions > 0 && <span>{activeSessions}</span>}
        </button>

        {/* Theme toggle */}
        <button
          type="button"
          onClick={() => toggleTheme()}
          title={`Switch to ${theme === "dark" ? "light" : "dark"} mode`}
          aria-label="Toggle theme"
          style={{
            display: "flex",
            alignItems: "center",
            padding: "0 8px",
            height: "22px",
            cursor: "pointer",
            transition: "background 0.1s",
            background: "transparent",
            color: "inherit",
            border: "none"
          }}
          onMouseEnter={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "rgba(0,0,0,0.15)")}
          onMouseLeave={(e) => ((e.currentTarget as HTMLButtonElement).style.background = "transparent")}
        >
          {theme === "dark" ? <Sun size={11} /> : <Moon size={11} />}
        </button>

        {/* Kaioken version */}
        <div
          style={{
            padding: "0 8px",
            height: "22px",
            display: "flex",
            alignItems: "center",
            opacity: 0.7,
            borderLeft: "1px solid rgba(255,255,255,0.2)"
          }}
        >
          Kaioken v0.8.1
        </div>
      </div>
    </footer>
  );
}
