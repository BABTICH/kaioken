// src/renderer/components/chat/MessageRow.tsx
import { useStore } from "@nanostores/react";
import { messageStore } from "@renderer/stores/messages";
import { useMessageText } from "@renderer/hooks/useMessageText";
import { ToolCard } from "./ToolCard";
import { MarkdownContent } from "./MarkdownContent";

interface MessageRowProps {
  messageId: string;
}

export function MessageRow({ messageId }: MessageRowProps) {
  const state = useStore(messageStore);
  const meta = state.byId[messageId];
  const text = useMessageText(messageId);

  if (!meta) {
    return null;
  }

  return (
    <article
      className={[
        "msg",
        `msg-${meta.role}`,
        meta.status === "streaming" ? "msg-streaming" : ""
      ].join(" ")}
    >
      <div className="msg-head">
        <span className="label label--accent">{meta.role}</span>

        <span className="label">
          {new Date(meta.createdAt).toLocaleTimeString()}
        </span>
      </div>

      <div className="msg-body">
        {text.length > 0 ? (
          <MarkdownContent
            text={text}
            streaming={meta.status === "streaming"}
          />
        ) : meta.status === "streaming" ? (
          <MarkdownContent text="" streaming />
        ) : (
          "(empty)"
        )}
      </div>

      {meta.toolIds.length > 0 && (
        <div className="msg-tools">
          {meta.toolIds.map((toolId) => (
            <ToolCard key={toolId} toolId={toolId} />
          ))}
        </div>
      )}
    </article>
  );
}
