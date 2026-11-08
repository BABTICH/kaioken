// src/renderer/lib/markdown.tsx
import { Fragment, type ReactNode } from "react";

export interface KaiokenMarkdownOptions {
  headingIds?: boolean;
  renderCodeBlock?: (
    language: string,
    code: string,
    key: string
  ) => ReactNode | undefined;
}

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  const nodes: ReactNode[] = [];

  const regex =
    /(`[^`]+`|\*\*[^*]+\*\*|\*[^*]+\*|\[[^\]]+\]\([^)\s]+\))/g;

  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let tokenIndex = 0;

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      nodes.push(text.slice(lastIndex, match.index));
    }

    const token = match[0];
    const key = `${keyPrefix}-${tokenIndex++}`;

    if (token.startsWith("`")) {
      nodes.push(
        <code key={key} className="md-code">
          {token.slice(1, -1)}
        </code>
      );
    } else if (token.startsWith("**")) {
      nodes.push(
        <strong key={key}>
          {renderInline(token.slice(2, -2), `${key}-bold`)}
        </strong>
      );
    } else if (token.startsWith("*")) {
      nodes.push(
        <em key={key}>
          {renderInline(token.slice(1, -1), `${key}-italic`)}
        </em>
      );
    } else if (token.startsWith("[")) {
      const linkMatch = token.match(/^\[([^\]]+)\]\(([^)\s]+)\)$/);

      if (linkMatch) {
        nodes.push(
          <span
            key={key}
            className="md-link"
            title={linkMatch[2]}
          >
            {linkMatch[1]}
          </span>
        );
      } else {
        nodes.push(token);
      }
    } else {
      nodes.push(token);
    }

    lastIndex = match.index + token.length;
  }

  if (lastIndex < text.length) {
    nodes.push(text.slice(lastIndex));
  }

  return nodes;
}

export function renderKaiokenMarkdown(
  text: string,
  options?: KaiokenMarkdownOptions
): ReactNode[] {
  const lines = text.split("\n");
  const blocks: ReactNode[] = [];

  let i = 0;
  let key = 0;

  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trim();

    if (trimmed === "") {
      i++;
      continue;
    }

    if (trimmed.startsWith("```")) {
      const language = trimmed.slice(3).trim();
      const codeLines: string[] = [];

      i++;

      while (
        i < lines.length &&
        !lines[i].trim().startsWith("```")
      ) {
        codeLines.push(lines[i]);
        i++;
      }

      if (i < lines.length) {
        i++;
      }

      const blockKey = key++;
      const code = codeLines.join("\n");

      const customBlock = options?.renderCodeBlock?.(
        language,
        code,
        `md-code-${blockKey}`
      );

      if (customBlock !== undefined) {
        blocks.push(customBlock);
        continue;
      }

      blocks.push(
        <pre
          key={blockKey}
          className="md-pre"
          data-lang={language}
        >
          <code>{code}</code>
        </pre>
      );

      continue;
    }

    const heading = line.match(/^(#{1,6})\s+(.*)$/);

    if (heading) {
      const level = heading[1].length;
      const content = renderInline(heading[2], `heading-${key}`);

      const blockKey = key++;

      const headingId = options?.headingIds
        ? `kb-h-${blockKey}`
        : undefined;

      if (level === 1) {
        blocks.push(
          <h1
            key={blockKey}
            id={headingId}
            className="md-h1"
          >
            {content}
          </h1>
        );
      } else if (level === 2) {
        blocks.push(
          <h2
            key={blockKey}
            id={headingId}
            className="md-h2"
          >
            {content}
          </h2>
        );
      } else if (level === 3) {
        blocks.push(
          <h3
            key={blockKey}
            id={headingId}
            className="md-h3"
          >
            {content}
          </h3>
        );
      } else {
        blocks.push(
          <h4
            key={blockKey}
            id={headingId}
            className="md-h4"
          >
            {content}
          </h4>
        );
      }

      i++;
      continue;
    }

    if (/^\s*---\s*$/.test(line)) {
      const blockKey = key++;

      blocks.push(<hr key={blockKey} className="md-hr" />);
      i++;
      continue;
    }

    if (trimmed.startsWith(">")) {
      const quoteLines: string[] = [];

      while (
        i < lines.length &&
        lines[i].trim().startsWith(">")
      ) {
        quoteLines.push(lines[i].trim().replace(/^>\s?/, ""));
        i++;
      }

      const blockKey = key++;

      blocks.push(
        <blockquote key={blockKey} className="md-blockquote">
          {quoteLines.map((quoteLine, quoteIndex) => (
            <Fragment key={quoteIndex}>
              {renderInline(
                quoteLine,
                `quote-${blockKey}-${quoteIndex}`
              )}
              {quoteIndex < quoteLines.length - 1 && <br />}
            </Fragment>
          ))}
        </blockquote>
      );

      continue;
    }

    if (/^\s*[-*+]\s+/.test(line)) {
      const items: string[] = [];

      while (
        i < lines.length &&
        /^\s*[-*+]\s+/.test(lines[i])
      ) {
        items.push(lines[i].replace(/^\s*[-*+]\s+/, ""));
        i++;
      }

      const blockKey = key++;

      blocks.push(
        <ul key={blockKey} className="md-ul">
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>
              {renderInline(
                item,
                `ul-${blockKey}-${itemIndex}`
              )}
            </li>
          ))}
        </ul>
      );

      continue;
    }

    if (/^\s*\d+\.\s+/.test(line)) {
      const items: string[] = [];

      while (
        i < lines.length &&
        /^\s*\d+\.\s+/.test(lines[i])
      ) {
        items.push(lines[i].replace(/^\s*\d+\.\s+/, ""));
        i++;
      }

      const blockKey = key++;

      blocks.push(
        <ol key={blockKey} className="md-ol">
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>
              {renderInline(
                item,
                `ol-${blockKey}-${itemIndex}`
              )}
            </li>
          ))}
        </ol>
      );

      continue;
    }

    const paragraphLines: string[] = [];

    while (
      i < lines.length &&
      lines[i].trim() !== "" &&
      !lines[i].trim().startsWith("```") &&
      !/^#{1,6}\s+/.test(lines[i]) &&
      !lines[i].trim().startsWith(">") &&
      !/^\s*[-*+]\s+/.test(lines[i]) &&
      !/^\s*\d+\.\s+/.test(lines[i]) &&
      !/^\s*---\s*$/.test(lines[i])
    ) {
      paragraphLines.push(lines[i]);
      i++;
    }

    const blockKey = key++;

    blocks.push(
      <p key={blockKey} className="md-p">
        {paragraphLines.map((paragraphLine, paragraphIndex) => (
          <Fragment key={paragraphIndex}>
            {renderInline(
              paragraphLine,
              `p-${blockKey}-${paragraphIndex}`
            )}
            {paragraphIndex < paragraphLines.length - 1 && <br />}
          </Fragment>
        ))}
      </p>
    );
  }

  return blocks;
}
