// src/renderer/components/approval/ApprovalModal.tsx
import { useEffect, useRef, useState } from "react";
import { useStore } from "@nanostores/react";
import {
  approvalStore,
  approveActiveRequest,
  denyActiveRequest
} from "@renderer/stores/approval";

export function ApprovalModal() {
  const state = useStore(approvalStore);
  const request = state.activeRequest;

  const containerRef = useRef<HTMLDivElement | null>(null);
  const denyRef = useRef<HTMLButtonElement | null>(null);

  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (!request) {
      return;
    }

    setNow(Date.now());
    denyRef.current?.focus();

    const timer = window.setInterval(() => {
      setNow(Date.now());
    }, 250);

    return () => {
      window.clearInterval(timer);
    };
  }, [request?.requestId]);

  useEffect(() => {
    if (!request) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void denyActiveRequest("escape");
        return;
      }

      if (event.key === "Enter") {
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          void approveActiveRequest();
        } else {
          event.preventDefault();
          void denyActiveRequest("enter");
        }
        return;
      }

      if (event.key === "Tab") {
        const root = containerRef.current;

        if (!root) {
          return;
        }

        const focusables = Array.from(
          root.querySelectorAll<HTMLButtonElement>(
            "button:not([disabled])"
          )
        );

        if (focusables.length === 0) {
          return;
        }

        const first = focusables[0];
        const last = focusables[focusables.length - 1];

        if (
          event.shiftKey &&
          document.activeElement === first
        ) {
          event.preventDefault();
          last.focus();
        } else if (
          !event.shiftKey &&
          document.activeElement === last
        ) {
          event.preventDefault();
          first.focus();
        }
      }
    };

    window.addEventListener("keydown", onKeyDown);

    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [request?.requestId]);

  if (!request) {
    return null;
  }

  const deadline = request.createdAt + request.timeoutMs;
  const remaining = Math.max(0, deadline - now);
  const seconds = Math.ceil(remaining / 1000);

  const percent = Math.max(
    0,
    Math.min(100, (remaining / request.timeoutMs) * 100)
  );

  return (
    <div className="approval-backdrop">
      <div
        ref={containerRef}
        className="approval-panel"
        role="dialog"
        aria-modal="true"
      >
        <div className="approval-head">
          <span className="label label--accent">
            Destructive action
          </span>

          <span className="approval-countdown">
            auto-deny {seconds}s
          </span>
        </div>

        <div className="approval-progress">
          <div
            className="approval-progress-fill"
            style={{ width: `${percent}%` }}
          />
        </div>

        <div className="approval-body">
          <div className="approval-title h3">{request.title}</div>

          <p className="approval-desc">{request.description}</p>

          <div className="approval-meta">
            <span className="label">tool</span>
            <span>{request.kind}</span>

            <span className="label">risk</span>
            <span>{request.risk}</span>

            {request.filePath && (
              <>
                <span className="label">file</span>
                <span>{request.filePath}</span>
              </>
            )}

            {request.command && (
              <>
                <span className="label">command</span>
                <span>{request.command}</span>
              </>
            )}

            <span className="label">session</span>
            <span>{request.sessionId}</span>
          </div>

          {state.error && (
            <div
              className="label"
              style={{ color: "var(--accent)" }}
            >
              {state.error}
            </div>
          )}
        </div>

        <div className="approval-actions">
          <button
            ref={denyRef}
            type="button"
            className="btn btn--primary"
            disabled={state.resolving}
            onClick={() => {
              void denyActiveRequest("button");
            }}
          >
            Deny
          </button>

          <button
            type="button"
            className="btn btn--ember"
            disabled={state.resolving}
            onClick={() => {
              void approveActiveRequest();
            }}
          >
            Approve once
          </button>
        </div>
      </div>
    </div>
  );
}
