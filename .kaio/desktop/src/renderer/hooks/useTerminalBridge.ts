// src/renderer/hooks/useTerminalBridge.ts
import { useEffect } from "react";
import { writeTerminal } from "@renderer/lib/terminal/session-manager";
import { markTerminalExited } from "@renderer/stores/terminal";

export function useTerminalBridge(): void {
  useEffect(() => {
    const offOutput = window.api.pty.onOutput((output) => {
      writeTerminal(output.sessionId, output.data, () => {
        window.api.pty.ack(
          output.sessionId,
          output.seq,
          output.bytes
        );
      });
    });

    const offExit = window.api.pty.onExit((info) => {
      markTerminalExited(info);
    });

    return () => {
      offOutput();
      offExit();
    };
  }, []);
}
