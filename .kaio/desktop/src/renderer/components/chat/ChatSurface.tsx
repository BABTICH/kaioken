// src/renderer/components/chat/ChatSurface.tsx
import { useState } from "react";
import { useAgentSession } from "@renderer/hooks/useAgentSession";
import { MessageList } from "./MessageList";
import { Plus, Square, Send } from "lucide-react";

export function ChatSurface() {
  const session = useAgentSession();

  const [providerId, setProviderId] = useState("mock");
  const [modelId, setModelId] = useState("mock-stream");
  const [draft, setDraft] = useState("");

  const busy = session.connectionState === "connecting";

  const send = async () => {
    const text = draft.trim();

    if (!text || busy) {
      return;
    }

    setDraft("");

    if (
      !session.activeSession ||
      session.activeSession.status === "stopped"
    ) {
      await session.createSession({
        providerId,
        modelId
      });
    }

    await session.sendTestMessage(text);
  };

  return (
    <div className="chat-surface">
      <div className="chat-topbar">
        <span className="label label--accent">Session</span>

        <input
          className="input chat-provider-input"
          value={providerId}
          placeholder="provider"
          onChange={(event) => setProviderId(event.target.value)}
        />

        <input
          className="input chat-model-input"
          value={modelId}
          placeholder="model"
          onChange={(event) => setModelId(event.target.value)}
        />

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          disabled={busy}
          onClick={() => {
            void session.createSession({
              providerId,
              modelId
            });
          }}
        >
          <Plus size={12} />
          <span>New</span>
        </button>

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          disabled={
            !session.activeSession ||
            session.activeSession.status === "stopped"
          }
          onClick={() => {
            void session.stopActiveSession();
          }}
        >
          <Square size={11} />
          <span>Stop</span>
        </button>

        <span className="label">{session.connectionState}</span>

        {session.error && (
          <span
            className="label"
            style={{ color: "var(--accent)" }}
          >
            {session.error}
          </span>
        )}
      </div>

      <MessageList />

      <div className="composer">
        <input
          className="composer-input"
          value={draft}
          placeholder="Send a message to the Kaiopi agent..."
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              void send();
            }
          }}
        />

        <button
          type="button"
          className="btn btn--primary inline-flex items-center gap-1.5"
          disabled={draft.trim().length === 0 || busy}
          onClick={() => {
            void send();
          }}
        >
          <Send size={13} />
          <span>Send</span>
        </button>
      </div>
    </div>
  );
}
