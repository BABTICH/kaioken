// src/renderer/components/wiki/WikiMarkdown.tsx
import { useEffect, useMemo, useRef } from "react";
import { renderKaiokenMarkdown } from "@renderer/lib/markdown";
import { MermaidBlock } from "./MermaidBlock";
import type { WikiOutlineItem } from "@renderer/stores/wiki";

interface WikiMarkdownProps {
  content: string;
  onOutline?: (items: WikiOutlineItem[]) => void;
}

export function WikiMarkdown({
  content,
  onOutline
}: WikiMarkdownProps) {
  const contentRef = useRef<HTMLDivElement | null>(null);

  const nodes = useMemo(() => {
    return renderKaiokenMarkdown(content, {
      headingIds: true,
      renderCodeBlock: (language, code, blockKey) => {
        if (language === "mermaid") {
          return <MermaidBlock key={blockKey} code={code} />;
        }

        return undefined;
      }
    });
  }, [content]);

  useEffect(() => {
    const root = contentRef.current;

    if (!root || !onOutline) {
      return;
    }

    const headings = Array.from(
      root.querySelectorAll("h1, h2, h3, h4")
    );

    const items: WikiOutlineItem[] = headings.map((heading) => ({
      id: heading.id,
      level: Number(heading.tagName.charAt(1)),
      text: heading.textContent ?? ""
    }));

    onOutline(items);
  }, [content, onOutline]);

  return (
    <div ref={contentRef} className="wiki-md md">
      {nodes}
    </div>
  );
}
