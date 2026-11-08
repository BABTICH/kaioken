// src/renderer/components/editor/EditorSurface.tsx
import { useState } from "react";
import { useStore } from "@nanostores/react";
import {
  explorerStore,
  closeFile,
  setActiveFile,
  openWorkspace
} from "@renderer/stores/explorer";
import { CodeEditor } from "./CodeEditor";
import { MarkdownViewer } from "./MarkdownViewer";
import { FileCode, X, FolderOpen, FileText, Eye, Code2 } from "lucide-react";

function fileBaseName(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

function isMarkdown(filename: string): boolean {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";
  return ext === "md" || ext === "mdx";
}

function getFileIcon(filename: string) {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";
  const codeExts = ["ts", "tsx", "js", "jsx", "json", "html", "css", "py", "rs", "go", "sh", "yaml", "yml"];
  if (codeExts.includes(ext)) {
    return <FileCode size={13} className="shrink-0" style={{ color: "var(--kai-blue)", opacity: 0.85 }} />;
  }
  return <FileText size={13} className="shrink-0" style={{ color: "var(--fg-mute)", opacity: 0.85 }} />;
}

type PreviewMode = "source" | "preview";

export function EditorSurface() {
  const explorer = useStore(explorerStore);
  const [mdMode, setMdMode] = useState<PreviewMode>("preview");

  const { openFiles, activeFilePath, openFileContents, workspace } = explorer;

  if (!workspace) {
    return (
      <div
        className="flex h-full flex-col items-center justify-center gap-5"
        style={{ background: "#1e1e1e", color: "#858585" }}
      >
        <FolderOpen size={48} style={{ opacity: 0.3 }} />
        <div className="text-center">
          <div style={{ fontSize: "18px", color: "#d4d4d4", fontWeight: 500, marginBottom: "8px" }}>
            No folder opened
          </div>
          <div style={{ fontSize: "13px", marginBottom: "20px" }}>
            Open a folder to start editing files
          </div>
          <button
            type="button"
            className="btn inline-flex items-center gap-2"
            style={{
              background: "var(--accent)",
              color: "#fff",
              border: "none",
              padding: "8px 16px",
              borderRadius: "2px",
              fontSize: "13px",
              fontFamily: "var(--font-mono)"
            }}
            onClick={() => void openWorkspace()}
          >
            <FolderOpen size={14} />
            Open Folder…
          </button>
        </div>
        <div style={{ fontSize: "11px", opacity: 0.5, fontFamily: "var(--font-mono)" }}>
          Ctrl+O to open a folder
        </div>
      </div>
    );
  }

  if (openFiles.length === 0) {
    return (
      <div
        className="flex h-full flex-col items-center justify-center gap-4"
        style={{ background: "#1e1e1e", color: "#858585" }}
      >
        <FileText size={40} style={{ opacity: 0.25 }} />
        <div className="text-center">
          <div style={{ fontSize: "16px", color: "#d4d4d4", fontWeight: 500, marginBottom: "6px" }}>
            {workspace.name}
          </div>
          <div style={{ fontSize: "12px" }}>
            Select a file from the explorer to open it
          </div>
        </div>
      </div>
    );
  }

  const activeContent = activeFilePath ? (openFileContents[activeFilePath] ?? "") : "";
  const activeFilename = activeFilePath ? fileBaseName(activeFilePath) : "";
  const isMd = activeFilePath ? isMarkdown(activeFilePath) : false;

  return (
    <div
      className="editor-surface flex h-full flex-col"
      style={{ background: "#1e1e1e" }}
    >
      {/* ─ Tab bar ────────────────────────────────────────── */}
      <div
        className="flex items-end overflow-x-auto"
        style={{
          background: "#252526",
          borderBottom: "1px solid #1e1e1e",
          minHeight: "35px",
          flexShrink: 0,
          scrollbarWidth: "none"
        }}
      >
        {openFiles.map((p) => {
          const isActive = p === activeFilePath;
          const name = fileBaseName(p);
          return (
            <button
              key={p}
              type="button"
              onClick={() => setActiveFile(p)}
              className="editor-tab"
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "6px",
                height: "35px",
                padding: "0 12px 0 10px",
                borderRight: "1px solid #1e1e1e",
                background: isActive ? "#1e1e1e" : "#2d2d2d",
                color: isActive ? "#ffffff" : "#adadad",
                fontSize: "13px",
                fontFamily: "var(--font-mono)",
                whiteSpace: "nowrap",
                cursor: "pointer",
                border: "none",
                borderBottom: isActive ? "1px solid var(--accent)" : "1px solid transparent",
                flexShrink: 0,
                transition: "none",
                position: "relative"
              }}
            >
              {getFileIcon(name)}
              <span style={{ maxWidth: "160px", overflow: "hidden", textOverflow: "ellipsis" }}>{name}</span>
              <span
                role="button"
                tabIndex={0}
                aria-label={`Close ${name}`}
                onClick={(e) => {
                  e.stopPropagation();
                  closeFile(p);
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.stopPropagation();
                    closeFile(p);
                  }
                }}
                style={{
                  marginLeft: "4px",
                  color: "#adadad",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  width: "16px",
                  height: "16px",
                  borderRadius: "3px",
                  cursor: "pointer"
                }}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLSpanElement).style.background = "rgba(255,255,255,0.15)";
                  (e.currentTarget as HTMLSpanElement).style.color = "#fff";
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLSpanElement).style.background = "transparent";
                  (e.currentTarget as HTMLSpanElement).style.color = "#adadad";
                }}
              >
                <X size={12} />
              </span>
            </button>
          );
        })}
      </div>

      {/* ─ Breadcrumb / toolbar ───────────────────────────── */}
      {activeFilePath && (
        <div
          className="flex items-center justify-between"
          style={{
            background: "#1e1e1e",
            borderBottom: "1px solid #333",
            padding: "3px 12px",
            fontSize: "11px",
            fontFamily: "var(--font-mono)",
            color: "#858585",
            flexShrink: 0
          }}
        >
          <span className="truncate">{activeFilePath.replace(/\\/g, " › ")}</span>
          {isMd && (
            <div className="flex items-center gap-1">
              <button
                type="button"
                onClick={() => setMdMode("source")}
                title="View source"
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "4px",
                  padding: "2px 8px",
                  background: mdMode === "source" ? "#094771" : "transparent",
                  color: mdMode === "source" ? "#fff" : "#858585",
                  border: "none",
                  borderRadius: "2px",
                  fontSize: "11px",
                  fontFamily: "var(--font-mono)",
                  cursor: "pointer"
                }}
              >
                <Code2 size={11} />
                Source
              </button>
              <button
                type="button"
                onClick={() => setMdMode("preview")}
                title="Render markdown"
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "4px",
                  padding: "2px 8px",
                  background: mdMode === "preview" ? "#094771" : "transparent",
                  color: mdMode === "preview" ? "#fff" : "#858585",
                  border: "none",
                  borderRadius: "2px",
                  fontSize: "11px",
                  fontFamily: "var(--font-mono)",
                  cursor: "pointer"
                }}
              >
                <Eye size={11} />
                Preview
              </button>
            </div>
          )}
        </div>
      )}

      {/* ─ Editor / Preview area ──────────────────────────── */}
      <div className="flex-1 overflow-hidden">
        {activeFilePath && (
          isMd && mdMode === "preview" ? (
            <MarkdownViewer content={activeContent} />
          ) : (
            <CodeEditor
              key={activeFilePath}
              content={activeContent}
              filename={activeFilename}
              readOnly={true}
            />
          )
        )}
      </div>
    </div>
  );
}
