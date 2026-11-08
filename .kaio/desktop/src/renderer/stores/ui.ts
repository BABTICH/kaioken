import { atom } from "nanostores";

export type PrimarySurface =
  | "welcome"
  | "chat"
  | "research"
  | "wiki"
  | "codemap"
  | "cards"
  | "cost"
  | "settings";

export type InspectorTab = "diff" | "files" | "graph" | "mermaid" | "symbols";

const STARTUP_KEY = "kaioken_show_welcome_on_startup";

function getInitialSurface(): PrimarySurface {
  try {
    if (typeof localStorage !== "undefined") {
      const showWelcome = localStorage.getItem(STARTUP_KEY);
      if (showWelcome === "false") {
        return "chat";
      }
    }
  } catch {
    // Ignored
  }
  return "welcome";
}

export const activeSurface = atom<PrimarySurface>(getInitialSurface());

/**
 * Navigate to a surface. When leaving welcome for any other surface the
 * first time in a session, auto-disable "show on startup" so the next
 * app launch lands directly on chat (like VS Code's welcome tab).
 */
export function setSurface(surface: PrimarySurface): void {
  const current = activeSurface.get();
  // User is actively leaving welcome → mark dismissed for future sessions
  if (current === "welcome" && surface !== "welcome") {
    try {
      const stored = localStorage.getItem(STARTUP_KEY);
      // Only auto-dismiss if they haven't explicitly re-enabled it
      if (stored !== "true") {
        localStorage.setItem(STARTUP_KEY, "false");
      }
    } catch {
      // Ignore storage errors
    }
  }
  activeSurface.set(surface);
}

export const inspectorOpen = atom<boolean>(true);

export function toggleInspector(): void {
  inspectorOpen.set(!inspectorOpen.get());
}

export const inspectorTab = atom<InspectorTab>("diff");

export function setInspectorTab(tab: InspectorTab): void {
  inspectorTab.set(tab);
}

export const terminalOpen = atom<boolean>(false);

export function toggleTerminal(): void {
  terminalOpen.set(!terminalOpen.get());
}

export const paletteOpen = atom<boolean>(false);

export function openPalette(): void {
  paletteOpen.set(true);
}

export function closePalette(): void {
  paletteOpen.set(false);
}

export function togglePalette(): void {
  paletteOpen.set(!paletteOpen.get());
}

export function closeOverlays(): void {
  paletteOpen.set(false);
}
