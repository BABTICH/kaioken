// src/renderer/hooks/useAgentSession.ts
import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import {
  createSession,
  refreshSessions,
  restoreLastActiveSession,
  restoreSession,
  sendTestMessage,
  sessionStore,
  stopActiveSession
} from "@renderer/stores/session";

export function useAgentSession() {
  const state = useStore(sessionStore);

  useEffect(() => {
    void (async () => {
      await refreshSessions();
      await restoreLastActiveSession();
    })();
  }, []);

  return {
    ...state,
    createSession,
    restoreSession,
    sendTestMessage,
    stopActiveSession
  };
}
