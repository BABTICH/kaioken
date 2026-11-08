// src/renderer/components/explorer/FileTreeContextMenu.tsx
import { useStore } from "@nanostores/react";
import {
  explorerStore,
  closeContextMenu,
  openFile
} from "@renderer/stores/explorer";

export function FileTreeContextMenu() {
  const state = useStore(explorerStore);
  const menu = state.contextMenu;

  if (!menu) {
    return null;
  }

  const copyRelative = async () => {
    try {
      await navigator.clipboard.writeText(menu.node.path);
    } catch {
      // Clipboard unavailable.
    }
  };

  const copyAbsolute = async () => {
    const root = state.workspace?.root ?? "";
    const sep = root.includes("\\") ? "\\" : "/";
    const abs = `${root}${sep}${menu.node.path}`;

    try {
      await navigator.clipboard.writeText(abs);
    } catch {
      // Clipboard unavailable.
    }
  };

  return (
    <>
      <div
        className="ctx-overlay"
        onClick={closeContextMenu}
        onContextMenu={(event) => {
          event.preventDefault();
          closeContextMenu();
        }}
      />

      <div
        className="ctx-menu"
        style={{ top: menu.y, left: menu.x }}
      >
        {menu.node.type === "file" && (
          <button
            type="button"
            className="ctx-item"
            onClick={() => {
              void openFile(menu.node.path);
              closeContextMenu();
            }}
          >
            Open
          </button>
        )}

        <button
          type="button"
          className="ctx-item"
          onClick={() => {
            void copyRelative();
            closeContextMenu();
          }}
        >
          Copy Relative Path
        </button>

        <button
          type="button"
          className="ctx-item"
          onClick={() => {
            void copyAbsolute();
            closeContextMenu();
          }}
        >
          Copy Absolute Path
        </button>
      </div>
    </>
  );
}
