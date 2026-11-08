// src/renderer/components/wiki/MermaidBlock.tsx
import { useEffect, useRef, useState } from "react";
import mermaid from "mermaid";

let initialized = false;
let idCounter = 0;

function ensureMermaid(): void {
  if (initialized) {
    return;
  }

  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    theme: "base",
    themeVariables: {
      darkMode: true,
      background: "#08080a",
      primaryColor: "#101012",
      primaryTextColor: "#f5f5f6",
      primaryBorderColor: "#33333a",
      lineColor: "#ff9500",
      secondaryColor: "#0a0a0b",
      tertiaryColor: "#101012",
      fontFamily:
        "'Geist Mono', ui-monospace, 'SF Mono', Menlo, monospace",
      fontSize: "13px"
    }
  });

  initialized = true;
}

interface MermaidBlockProps {
  code: string;
}

export function MermaidBlock({ code }: MermaidBlockProps) {
  const ref = useRef<HTMLDivElement | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    ensureMermaid();

    let cancelled = false;

    const id = `kaioken-mermaid-${++idCounter}`;

    const render = async () => {
      try {
        const result = await mermaid.render(id, code);

        if (!cancelled && ref.current) {
          ref.current.innerHTML = result.svg;
          setError(null);
        }
      } catch (renderError) {
        if (!cancelled) {
          setError(
            renderError instanceof Error
              ? renderError.message
              : String(renderError)
          );
        }
      }
    };

    void render();

    return () => {
      cancelled = true;
    };
  }, [code]);

  return (
    <div className="mermaid-block">
      {error ? (
        <div className="mermaid-error">
          <div className="label label--accent">Mermaid error</div>
          <pre>{error}</pre>
        </div>
      ) : (
        <div ref={ref} className="mermaid-svg" />
      )}
    </div>
  );
}
