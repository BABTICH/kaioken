// src/renderer/lib/event-batcher.ts
import type { AgentEvent, AgentEventEnvelope } from "@shared/types/agent";
import {
  applyAgentEvents,
  trackEnvelope
} from "@renderer/stores/agent-events";

let queue: AgentEvent[] = [];
let scheduled = false;

export function ingestAgentEnvelope(
  envelope: AgentEventEnvelope
): void {
  trackEnvelope(envelope);

  for (const event of envelope.events) {
    queue.push(event);
  }

  if (!scheduled) {
    scheduled = true;
    requestAnimationFrame(flush);
  }
}

function flush(): void {
  scheduled = false;

  const events = queue;
  queue = [];

  if (events.length === 0) {
    return;
  }

  applyAgentEvents(events);
}
