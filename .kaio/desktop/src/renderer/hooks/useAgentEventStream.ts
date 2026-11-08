// src/renderer/hooks/useAgentEventStream.ts
import { useEffect } from "react";
import { ingestAgentEnvelope } from "@renderer/lib/event-batcher";

export function useAgentEventStream(): void {
  useEffect(() => {
    const unsubscribe = window.api.agent.onEvent(ingestAgentEnvelope);

    return () => {
      unsubscribe();
    };
  }, []);
}
