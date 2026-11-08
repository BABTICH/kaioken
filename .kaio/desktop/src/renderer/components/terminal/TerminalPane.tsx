// src/renderer/components/terminal/TerminalPane.tsx
import { useEffect, useRef } from "react";
import {
  attachTerminal,
  detachTerminal,
  fitTerminal
} from "@renderer/lib/terminal/session-manager";

interface TerminalPaneProps {
  sessionId: string;
}

export function TerminalPane({ sessionId }: TerminalPaneProps) {
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const target = ref.current;

    if (!target) {
      return;
    }

    attachTerminal(sessionId, target);

    const observer = new ResizeObserver(() => {
      fitTerminal(sessionId);
    });

    observer.observe(target);

    const timer = setTimeout(() => {
      fitTerminal(sessionId);
    }, 50);

    return () => {
      clearTimeout(timer);
      observer.disconnect();
      detachTerminal(sessionId);
    };
  }, [sessionId]);

  return (
    <div
      ref={ref}
      style={{
        width: "100%",
        height: "100%",
        background: "var(--bg-sunk)"
      }}
    />
  );
}
