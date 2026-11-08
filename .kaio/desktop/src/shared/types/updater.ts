// src/shared/types/updater.ts

export type UpdateChannel =
  | "stable"
  | "beta"
  | "dev";

export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "not-available"
  | "downloading"
  | "downloaded"
  | "installing"
  | "error";

export interface UpdateState {
  status: UpdateStatus;
  channel: UpdateChannel;
  currentVersion: string;
  latestVersion?: string;
  downloadProgress: number;
  error: string | null;
  releaseNotes: string;
  updatedAt: number;
}

export type Unsubscribe = () => void;

export interface UpdaterApi {
  getState(): Promise<UpdateState>;

  check(): Promise<void>;

  download(): Promise<void>;

  install(): Promise<void>;

  setChannel(channel: UpdateChannel): Promise<void>;

  onState(cb: (state: UpdateState) => void): Unsubscribe;
}
