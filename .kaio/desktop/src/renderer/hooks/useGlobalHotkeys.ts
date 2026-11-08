// src/renderer/hooks/useGlobalHotkeys.ts
import { useEffect } from "react";
import {
  closeOverlays,
  setSurface,
  toggleInspector,
  togglePalette,
  toggleTerminal,
  type PrimarySurface
} from "@renderer/stores/ui";

const surfaceByKey: Record<string, PrimarySurface> = {
  "0": "welcome",
  "1": "chat",
  "2": "research",
  "3": "wiki",
  "4": "codemap",
  "5": "cards",
  "6": "cost",
  "7": "settings"
};

export function useGlobalHotkeys(): void {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeOverlays();
        return;
      }

      const mod = event.ctrlKey || event.metaKey;

      if (!mod) {
        return;
      }

      if (event.code === "Backquote") {
        event.preventDefault();
        toggleTerminal();
        return;
      }

      const key = event.key.toLowerCase();

      if (key === "b") {
        event.preventDefault();
        toggleInspector();
        return;
      }

      if (key === "k") {
        event.preventDefault();
        togglePalette();
        return;
      }

      if (key === "p") {
        event.preventDefault();
        togglePalette();
        return;
      }

      const surface = surfaceByKey[event.key];

      if (surface) {
        event.preventDefault();
        setSurface(surface);
      }
    };

    window.addEventListener("keydown", onKeyDown);

    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);
}
