// src/shared/types/pty.ts

export type Unsubscribe = () => void;

export interface PtyCreateOptions {
  cols: number;
  rows: number;
  cwd?: string;
  shell?: string;
  profile?: "default" | "build" | "test" | "git";
}

export interface PtyCreateResult {
  sessionId: string;
  pid: number;
  shell: string;
  cwd: string;
}

export interface PtyOutput {
  sessionId: string;
  seq: number;
  data: string;
  bytes: number;
}

export interface PtyExit {
  sessionId: string;
  exitCode: number;
  signal?: string;
}

export interface PtyApi {
  create(opts: PtyCreateOptions): Promise<PtyCreateResult>;

  write(sessionId: string, data: string): void;

  resize(
    sessionId: string,
    cols: number,
    rows: number
  ): Promise<void>;

  close(sessionId: string): Promise<void>;

  onOutput(cb: (output: PtyOutput) => void): Unsubscribe;

  onExit(cb: (info: PtyExit) => void): Unsubscribe;

  ack(sessionId: string, seq: number, bytes: number): void;
}
