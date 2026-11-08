// src/renderer/components/editor/MarkdownViewer.tsx
import { useStore } from "@nanostores/react";
import { resolvedTheme } from "@renderer/stores/theme";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import rehypeSanitize from "rehype-sanitize";
import "highlight.js/styles/vs2015.css";

interface MarkdownViewerProps {
  content: string;
}

export function MarkdownViewer({ content }: MarkdownViewerProps) {
  const theme = useStore(resolvedTheme);

  return (
    <div
      className="md-viewer hud-scrollbar"
      style={{
        height: "100%",
        width: "100%",
        overflow: "auto",
        padding: "24px 32px",
        background: theme === "dark" ? "#1e1e1e" : "#ffffff",
        color: theme === "dark" ? "#d4d4d4" : "#1a1a1a",
        fontFamily: "var(--font-sans)",
        lineHeight: "1.7",
        fontSize: "14px"
      }}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeSanitize, rehypeHighlight]}
        components={{
          h1: ({ children }) => (
            <h1
              style={{
                fontSize: "2em",
                fontWeight: 700,
                borderBottom: `1px solid ${theme === "dark" ? "#404040" : "#e0e0e0"}`,
                paddingBottom: "0.3em",
                marginBottom: "0.75em",
                marginTop: "1em",
                color: theme === "dark" ? "#fff" : "#000"
              }}
            >
              {children}
            </h1>
          ),
          h2: ({ children }) => (
            <h2
              style={{
                fontSize: "1.5em",
                fontWeight: 600,
                borderBottom: `1px solid ${theme === "dark" ? "#404040" : "#e0e0e0"}`,
                paddingBottom: "0.2em",
                marginBottom: "0.6em",
                marginTop: "1.2em",
                color: theme === "dark" ? "#f0f0f0" : "#111"
              }}
            >
              {children}
            </h2>
          ),
          h3: ({ children }) => (
            <h3
              style={{
                fontSize: "1.2em",
                fontWeight: 600,
                marginBottom: "0.5em",
                marginTop: "1em",
                color: theme === "dark" ? "#d4d4d4" : "#222"
              }}
            >
              {children}
            </h3>
          ),
          p: ({ children }) => (
            <p style={{ marginBottom: "0.75em", lineHeight: "1.7" }}>{children}</p>
          ),
          a: ({ href, children }) => (
            <a
              href={href}
              style={{ color: theme === "dark" ? "#4fc1ff" : "#0078d4", textDecoration: "underline" }}
              target="_blank"
              rel="noopener noreferrer"
            >
              {children}
            </a>
          ),
          code: ({ inline, className, children }: { inline?: boolean; className?: string; children?: React.ReactNode }) => {
            if (inline) {
              return (
                <code
                  style={{
                    background: theme === "dark" ? "#2d2d2d" : "#f0f0f0",
                    color: theme === "dark" ? "#ce9178" : "#a31515",
                    padding: "1px 6px",
                    borderRadius: "3px",
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.88em"
                  }}
                >
                  {children}
                </code>
              );
            }
            return (
              <code className={className}>
                {children}
              </code>
            );
          },
          pre: ({ children }) => (
            <pre
              style={{
                background: theme === "dark" ? "#1e1e1e" : "#f6f8fa",
                border: `1px solid ${theme === "dark" ? "#404040" : "#e1e4e8"}`,
                borderRadius: "4px",
                padding: "16px",
                overflowX: "auto",
                marginBottom: "1em",
                fontFamily: "var(--font-mono)",
                fontSize: "13px",
                lineHeight: "1.5"
              }}
            >
              {children}
            </pre>
          ),
          blockquote: ({ children }) => (
            <blockquote
              style={{
                borderLeft: `4px solid ${theme === "dark" ? "#404040" : "#ddd"}`,
                paddingLeft: "16px",
                margin: "0 0 0.75em 0",
                color: theme === "dark" ? "#999" : "#666"
              }}
            >
              {children}
            </blockquote>
          ),
          ul: ({ children }) => (
            <ul style={{ paddingLeft: "1.5em", marginBottom: "0.75em" }}>{children}</ul>
          ),
          ol: ({ children }) => (
            <ol style={{ paddingLeft: "1.5em", marginBottom: "0.75em" }}>{children}</ol>
          ),
          li: ({ children }) => (
            <li style={{ marginBottom: "0.25em" }}>{children}</li>
          ),
          table: ({ children }) => (
            <div style={{ overflowX: "auto", marginBottom: "1em" }}>
              <table
                style={{
                  borderCollapse: "collapse",
                  width: "100%",
                  fontFamily: "var(--font-mono)",
                  fontSize: "13px"
                }}
              >
                {children}
              </table>
            </div>
          ),
          th: ({ children }) => (
            <th
              style={{
                border: `1px solid ${theme === "dark" ? "#404040" : "#ddd"}`,
                padding: "6px 12px",
                background: theme === "dark" ? "#2a2a2a" : "#f6f8fa",
                textAlign: "left",
                fontWeight: 600
              }}
            >
              {children}
            </th>
          ),
          td: ({ children }) => (
            <td
              style={{
                border: `1px solid ${theme === "dark" ? "#404040" : "#ddd"}`,
                padding: "6px 12px"
              }}
            >
              {children}
            </td>
          ),
          hr: () => (
            <hr
              style={{
                border: "none",
                borderTop: `1px solid ${theme === "dark" ? "#404040" : "#e0e0e0"}`,
                margin: "1.5em 0"
              }}
            />
          )
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
