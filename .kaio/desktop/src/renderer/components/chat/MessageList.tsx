// src/renderer/components/chat/MessageList.tsx
import { useEffect, useRef } from "react";
import { useStore } from "@nanostores/react";
import { messageStore } from "@renderer/stores/messages";
import { useMessageVirtualization } from "@renderer/hooks/useMessageVirtualization";
import { MessageRow } from "./MessageRow";

interface MeasuredMessageRowProps {
  messageId: string;
  onHeight: (id: string, height: number) => void;
}

function MeasuredMessageRow({
  messageId,
  onHeight
}: MeasuredMessageRowProps) {
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const el = ref.current;

    if (!el) {
      return;
    }

    const observer = new ResizeObserver(() => {
      onHeight(messageId, el.offsetHeight);
    });

    observer.observe(el);

    return () => {
      observer.disconnect();
    };
  }, [messageId, onHeight]);

  return (
    <div ref={ref} className="virtual-message">
      <MessageRow messageId={messageId} />
    </div>
  );
}

export function MessageList() {
  const state = useStore(messageStore);

  const {
    containerRef,
    visibleIds,
    topSpacerHeight,
    bottomSpacerHeight,
    setHeight
  } = useMessageVirtualization(state.orderedIds);

  if (state.orderedIds.length === 0) {
    return (
      <div className="message-list-empty">
        <div className="term">
          <div className="term-bar">
            <span>Agent Stream</span>

            <span className="term-status">
              <i />
              IDLE
            </span>
          </div>

          <pre className="term-body">
            {`No messages yet.

Type below to create a session and send the first message.`}
          </pre>
        </div>
      </div>
    );
  }

  return (
    <div ref={containerRef} className="message-list hud-scrollbar">
      <div style={{ height: topSpacerHeight }} aria-hidden />

      {visibleIds.map((messageId) => (
        <MeasuredMessageRow
          key={messageId}
          messageId={messageId}
          onHeight={setHeight}
        />
      ))}

      <div style={{ height: bottomSpacerHeight }} aria-hidden />
    </div>
  );
}
