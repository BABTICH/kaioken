// src/renderer/stores/release-notes.ts
import { atom } from "nanostores";

export interface ReleaseNotesState {
  open: boolean;
}

export const releaseNotesStore = atom<ReleaseNotesState>({
  open: false
});

export function openReleaseNotes(): void {
  releaseNotesStore.set({
    open: true
  });
}

export function closeReleaseNotes(): void {
  releaseNotesStore.set({
    open: false
  });
}

export function toggleReleaseNotes(): void {
  releaseNotesStore.set({
    open: !releaseNotesStore.get().open
  });
}
