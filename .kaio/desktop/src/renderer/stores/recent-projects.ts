// src/renderer/stores/recent-projects.ts
import { atom } from "nanostores";

export interface RecentProject {
  path: string;
  name: string;
  lastOpened: number;
}

const STORAGE_KEY = "kaioken_recent_projects";
const STARTUP_KEY = "kaioken_show_welcome_on_startup";

function loadFromStorage(): RecentProject[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      return parsed.filter(
        (item) => typeof item?.path === "string" && typeof item?.name === "string"
      );
    }
  } catch (error) {
    console.error("[recentProjects] Failed to load from storage", error);
  }
  return [];
}

function saveToStorage(projects: RecentProject[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(projects));
  } catch (error) {
    console.error("[recentProjects] Failed to save to storage", error);
  }
}

export const recentProjectsStore = atom<RecentProject[]>(loadFromStorage());

export const showWelcomeOnStartup = atom<boolean>(
  typeof localStorage !== "undefined"
    ? localStorage.getItem(STARTUP_KEY) !== "false"
    : true
);

export function setShowWelcomeOnStartup(show: boolean): void {
  showWelcomeOnStartup.set(show);
  try {
    // Always persist explicit value ("true" or "false") so the auto-dismiss
    // logic in ui.ts can detect whether the user explicitly opted back in.
    localStorage.setItem(STARTUP_KEY, show ? "true" : "false");
  } catch (error) {
    console.error("[recentProjects] Failed to save startup preference", error);
  }
}

export function addRecentProject(dirPath: string, name?: string): void {
  if (!dirPath) return;

  const current = recentProjectsStore.get();
  const folderName =
    name ||
    dirPath.split(/[\\/]/).filter(Boolean).pop() ||
    dirPath;

  const updated: RecentProject[] = [
    {
      path: dirPath,
      name: folderName,
      lastOpened: Date.now()
    },
    ...current.filter((item) => item.path.toLowerCase() !== dirPath.toLowerCase())
  ].slice(0, 10);

  recentProjectsStore.set(updated);
  saveToStorage(updated);
}

export function removeRecentProject(dirPath: string): void {
  const current = recentProjectsStore.get();
  const updated = current.filter(
    (item) => item.path.toLowerCase() !== dirPath.toLowerCase()
  );
  recentProjectsStore.set(updated);
  saveToStorage(updated);
}

export function clearRecentProjects(): void {
  recentProjectsStore.set([]);
  saveToStorage([]);
}
