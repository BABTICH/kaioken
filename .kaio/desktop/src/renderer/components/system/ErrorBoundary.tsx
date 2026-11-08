// src/renderer/components/system/ErrorBoundary.tsx
import React, { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null
    };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error, errorInfo: null };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    this.setState({
      error,
      errorInfo
    });

    // Report to main process
    if (window.api?.crash) {
      window.api.crash.record({
        message: error.message,
        stack: error.stack,
        componentStack: errorInfo.componentStack ?? undefined
      });
    }
  }

  handleReload = (): void => {
    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      const { error, errorInfo } = this.state;

      return (
        <div
          style={{
            height: "100vh",
            display: "flex",
            flexDirection: "column",
            background: "var(--bg-sunk)",
            padding: "var(--s-6)",
            overflow: "hidden"
          }}
        >
          <div className="term" style={{ flex: 1, display: "flex", flexDirection: "column" }}>
            <div className="term-bar">
              <span>Kaioken Desktop Studio — Fatal Exception</span>
              <span className="term-status" style={{ color: "var(--accent)" }}>
                <i style={{ background: "var(--accent)" }} />
                CRASHED
              </span>
            </div>

            <pre className="term-body hud-scrollbar" style={{ flex: 1, overflow: "auto" }}>
              <span className="tl">
                <span className="t-pr">kaioken@desktop</span>
                <span className="t-mute">:</span>
                <span className="t-cmd">~</span>
                <span className="t-mute">$ </span>
                <span className="t-cmd">renderer --diagnose</span>
              </span>

              <span className="tl tl-bad">
                FATAL: Uncaught exception in renderer process.
              </span>

              <span className="tl-gap" />

              <span className="tl">
                <span className="t-k-bad">ERROR:</span>{" "}
                <span className="t-v">{error?.message ?? "Unknown error"}</span>
              </span>

              {error?.stack && (
                <span className="tl" style={{ color: "var(--fg-mute)" }}>
                  {error.stack.split("\n").map((line, i) => (
                    <React.Fragment key={i}>
                      {line}
                      {"\n"}
                    </React.Fragment>
                  ))}
                </span>
              )}

              {errorInfo?.componentStack && (
                <>
                  <span className="tl-gap" />
                  <span className="tl">
                    <span className="t-k">COMPONENT TRACE:</span>
                  </span>
                  <span className="tl" style={{ color: "var(--fg-mute)" }}>
                    {errorInfo.componentStack}
                  </span>
                </>
              )}
            </pre>
          </div>

          <div style={{ marginTop: "var(--s-4)", display: "flex", gap: "var(--s-3)" }}>
            <button className="btn btn--primary" onClick={this.handleReload}>
              Restart Renderer
            </button>
            <button
              className="btn"
              onClick={() => window.api.window.close()}
            >
              Exit Application
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
