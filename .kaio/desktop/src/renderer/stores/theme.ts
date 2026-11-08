// src/renderer/stores/theme.ts
import { atom } from "nanostores";
import { updateTerminalsTheme } from "@renderer/lib/terminal/session-manager";

export type ThemeMode = "dark" | "light" | "system";

const STORAGE_KEY = "kaioken-theme-mode";

function getSystemTheme(): "dark" | "light" {
  if (typeof window !== "undefined" && window.matchMedia) {
    return window.matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark";
  }
  return "dark";
}

function getInitialTheme(): ThemeMode {
  if (typeof window !== "undefined" && window.localStorage) {
    const saved = localStorage.getItem(STORAGE_KEY) as ThemeMode | null;
    if (saved === "dark" || saved === "light" || saved === "system") {
      return saved;
    }
  }
  return "dark";
}

export const themeMode = atom<ThemeMode>(getInitialTheme());
export const resolvedTheme = atom<"dark" | "light">("dark");

function applyTheme(resolved: "dark" | "light"): void {
  resolvedTheme.set(resolved);
  if (typeof document !== "undefined") {
    document.documentElement.setAttribute("data-theme", resolved);
    document.documentElement.style.colorScheme = resolved;
  }
  updateTerminalsTheme(resolved);
}

export function initTheme(): void {
  const currentMode = themeMode.get();
  const resolved = currentMode === "system" ? getSystemTheme() : currentMode;
  applyTheme(resolved);

  if (typeof window !== "undefined" && window.matchMedia) {
    const media = window.matchMedia("(prefers-color-scheme: light)");
    media.addEventListener("change", (e) => {
      if (themeMode.get() === "system") {
        applyTheme(e.matches ? "light" : "dark");
      }
    });
  }
}

export function setThemeMode(mode: ThemeMode): void {
  themeMode.set(mode);
  if (typeof window !== "undefined" && window.localStorage) {
    localStorage.setItem(STORAGE_KEY, mode);
  }
  const resolved = mode === "system" ? getSystemTheme() : mode;
  applyTheme(resolved);
}

export function toggleTheme(): void {
  const current = resolvedTheme.get();
  const next = current === "dark" ? "light" : "dark";
  setThemeMode(next);
}
