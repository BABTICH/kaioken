// src/renderer/components/explorer/FileTree.tsx
import { useStore } from "@nanostores/react";
import {
  explorerStore,
  selectNode,
  toggleNode,
  openFile,
  openContextMenu
} from "@renderer/stores/explorer";
import {
  ChevronRight,
  ChevronDown,
  Folder,
  FolderOpen,
  FileCode,
  FileText
} from "lucide-react";
import type { WorkspaceTreeNode } from "@shared/types/workspace";

function getFileIcon(filename: string) {
  const ext = filename.split(".").pop()?.toLowerCase();
  if (
    ["ts", "tsx", "js", "jsx", "json", "html", "css", "py", "rs", "go", "sh", "yaml", "yml", "md"].includes(
      ext ?? ""
    )
  ) {
    return <FileCode size={13} className="shrink-0 text-[var(--kai-blue)] opacity-85" />;
  }
  return <FileText size={13} className="shrink-0 text-[var(--fg-mute)] opacity-85" />;
}

interface FileTreeNodeProps {
  node: WorkspaceTreeNode;
  depth: number;
}

function GitBadge({
  status
}: {
  status?: WorkspaceTreeNode["gitStatus"];
}) {
  if (!status) return null;

  const colorMap: Record<string, string> = {
    modified: "var(--ember)",
    untracked: "var(--fg-mute)",
    added: "var(--kai-green)",
    deleted: "var(--accent)",
    renamed: "var(--kai-blue)"
  };

  const labelMap: Record<string, string> = {
    modified: "M",
    untracked: "U",
    added: "A",
    deleted: "D",
    renamed: "R"
  };

  return (
    <span
      className="mono"
      style={{
        fontSize: "var(--fs-micro)",
        color: colorMap[status] ?? "var(--fg-mute)",
        marginLeft: "auto",
        paddingLeft: "var(--s-2)"
      }}
    >
      {labelMap[status] ?? ""}
    </span>
  );
}

function FileTreeNode({ node, depth }: FileTreeNodeProps) {
  const state = useStore(explorerStore);
  const isExpanded = state.expandedNodes.has(node.id);
  const isSelected = state.selectedNodeId === node.id;
  const isDir = node.type === "directory";

  const handleClick = () => {
    if (isDir) {
      toggleNode(node.id);
    } else {
      void openFile(node.path);
    }

    selectNode(node.id);
  };

  const handleContextMenu = (
    event: React.MouseEvent
  ) => {
    event.preventDefault();
    selectNode(node.id);
    openContextMenu(event.clientX, event.clientY, node);
  };

  return (
    <div>
      <button
        type="button"
        onClick={handleClick}
        onContextMenu={handleContextMenu}
        className="mono"
        style={{
          display: "flex",
          alignItems: "center",
          width: "100%",
          padding: "3px var(--s-3)",
          paddingLeft: `calc(var(--s-3) + ${depth * 14}px)`,
          fontSize: "var(--fs-xs)",
          color: isSelected ? "var(--accent)" : "var(--fg-2)",
          background: isSelected
            ? "var(--accent-soft)"
            : "transparent",
          border: "none",
          borderRadius: "var(--r-none)",
          cursor: "pointer",
          transition: "background var(--dur-fast) var(--ease)",
          textAlign: "left"
        }}
        onMouseEnter={(e) => {
          if (!isSelected) {
            e.currentTarget.style.background = "var(--bg-raise)";
          }
        }}
        onMouseLeave={(e) => {
          if (!isSelected) {
            e.currentTarget.style.background = "transparent";
          }
        }}
      >
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "4px",
            marginRight: "6px",
            flexShrink: 0
          }}
        >
          {isDir ? (
            <>
              {isExpanded ? (
                <ChevronDown size={11} className="text-[var(--fg-mute)] shrink-0" />
              ) : (
                <ChevronRight size={11} className="text-[var(--fg-mute)] shrink-0" />
              )}
              {isExpanded ? (
                <FolderOpen size={13} className="text-[var(--accent)] shrink-0" />
              ) : (
                <Folder size={13} className="text-[var(--fg-mute)] shrink-0" />
              )}
            </>
          ) : (
            <>
              <span style={{ width: "11px", display: "inline-block" }} />
              {getFileIcon(node.name)}
            </>
          )}
        </span>

        <span
          style={{
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            flex: 1
          }}
        >
          {node.name}
        </span>

        <GitBadge status={node.gitStatus} />
      </button>

      {isDir && isExpanded && node.children && (
        <div>
          {node.children.map((child) => (
            <FileTreeNode
              key={child.id}
              node={child}
              depth={depth + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function FileTree() {
  const state = useStore(explorerStore);

  if (!state.workspace) {
    return (
      <div
        className="mono"
        style={{
          padding: "var(--s-4)",
          fontSize: "var(--fs-xs)",
          color: "var(--fg-mute)"
        }}
      >
        No workspace open.
      </div>
    );
  }

  if (state.loading && state.tree.length === 0) {
    return (
      <div
        className="mono"
        style={{
          padding: "var(--s-4)",
          fontSize: "var(--fs-xs)",
          color: "var(--fg-mute)"
        }}
      >
        Loading tree...
      </div>
    );
  }

  return (
    <div
      className="hud-scrollbar"
      style={{ overflow: "auto", flex: 1 }}
    >
      {state.tree.map((node) => (
        <FileTreeNode key={node.id} node={node} depth={0} />
      ))}
    </div>
  );
}
