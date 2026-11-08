// src/renderer/components/command-palette/CommandPalette.tsx
import { useEffect, useState } from "react";
import { Command } from "cmdk";
import { useStore } from "@nanostores/react";
import { paletteOpen, closePalette } from "@renderer/stores/ui";
import { openFile } from "@renderer/stores/explorer";
import type { WorkspaceSearchResult } from "@shared/types/workspace";
import { FileCode } from "lucide-react";

function fileBaseName(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

export function CommandPalette() {
  const open = useStore(paletteOpen);

  const [query, setQuery] = useState("");
  const [results, setResults] = useState<WorkspaceSearchResult[]>([]);
  const [searching, setSearching] = useState(false);

  useEffect(() => {
    if (open) {
      setQuery("");
      setResults([]);
      setSearching(false);
    }
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    setSearching(true);

    const t = window.setTimeout(async () => {
      try {
        const r = await window.api.workspace.search(query, {
          limit: 30
        });
        setResults(r);
      } catch {
        setResults([]);
      } finally {
        setSearching(false);
      }
    }, 120);

    return () => window.clearTimeout(t);
  }, [query, open]);

  if (!open) {
    return null;
  }

  const handleSelect = (path: string) => {
    void openFile(path);
    closePalette();
  };

  return (
    <div
      className="command-backdrop flex items-start justify-center pt-24"
      onClick={closePalette}
    >
      <div
        className="command-panel"
        style={{ padding: 0, overflow: "hidden" }}
        onClick={(event) => event.stopPropagation()}
      >
        <Command label="Quick Open" className="quickopen">
          <Command.Input
            autoFocus
            value={query}
            onValueChange={setQuery}
            placeholder="Search files by name…"
            className="quickopen-input"
          />

          <Command.List className="quickopen-list hud-scrollbar">
            {searching && query && (
              <div className="quickopen-empty">Searching…</div>
            )}

            {!searching && query && results.length === 0 && (
              <Command.Empty className="quickopen-empty">
                No files match.
              </Command.Empty>
            )}

            {!query && results.length === 0 && (
              <div className="quickopen-empty">
                Type to search workspace files.
              </div>
            )}

            <Command.Group>
              {results.map((r) => (
                <Command.Item
                  key={r.path}
                  value={r.path}
                  onSelect={() => handleSelect(r.path)}
                  className="quickopen-item"
                >
                  <div className="flex items-center gap-1.5 flex-1 min-w-0">
                    <FileCode size={13} className="shrink-0 text-[var(--kai-blue)] opacity-85" />
                    <span className="quickopen-item-name">
                      {fileBaseName(r.path)}
                    </span>
                  </div>
                  <span className="quickopen-item-path">{r.path}</span>
                </Command.Item>
              ))}
            </Command.Group>
          </Command.List>
        </Command>
      </div>
    </div>
  );
}
