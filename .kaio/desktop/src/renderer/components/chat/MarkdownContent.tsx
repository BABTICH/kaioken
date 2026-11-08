// src/renderer/components/chat/MarkdownContent.tsx
import { memo, useDeferredValue } from "react";
import { renderKaiokenMarkdown } from "@renderer/lib/markdown";

interface MarkdownContentProps {
  text: string;
  streaming?: boolean;
}

export const MarkdownContent = memo(function MarkdownContent({
  text,
  streaming = false
}: MarkdownContentProps) {
  const deferredText = useDeferredValue(text);

  return (
    <div className="md">
      {renderKaiokenMarkdown(deferredText)}

      {streaming && <span className="md-caret">▊</span>}
    </div>
  );
});
