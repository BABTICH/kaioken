import { app, session, shell } from "electron";

export function assertTrustedSender(
  event: Electron.IpcMainEvent | Electron.IpcMainInvokeEvent
): void {
  // Ensure the IPC call comes from the main window and not a rogue webview
  const sender = event.sender;
  const senderUrl = sender.getURL();

  if (senderUrl.startsWith("devtools://")) {
    throw new Error("UNAUTHORIZED_DEVTOOLS_IPC");
  }

  // In production, ensure it's strictly the local file or authorized origin
  if (
    app.isPackaged &&
    !senderUrl.startsWith("file://") &&
    !senderUrl.startsWith("kaioken://")
  ) {
    throw new Error("UNAUTHORIZED_ORIGIN");
  }
}

export function setupSecurity(): void {
  // 1. Disable WebSecurity bypasses (default in Electron, but explicitly enforced)
  app.on("web-contents-created", (_event, contents) => {
    contents.on("will-attach-webview", (event) => {
      // Prevent rogue webviews
      event.preventDefault();
    });

    contents.on("will-navigate", (event, url) => {
      const targetUrl = url || (event as { url?: string }).url || "";
      const currentUrl = contents.getURL();

      // Allow reload or dev server navigation in development
      if (!app.isPackaged) {
        try {
          const parsed = new URL(targetUrl);
          if (
            parsed.hostname === "localhost" ||
            parsed.hostname === "127.0.0.1"
          ) {
            return;
          }
        } catch {
          // ignore url parse failure
        }
      }

      // Allow in-app reload if same URL
      if (targetUrl && targetUrl === currentUrl) {
        return;
      }

      console.warn("[Security] Blocked navigation attempt to:", targetUrl);
      event.preventDefault();

      if (targetUrl.startsWith("http://") || targetUrl.startsWith("https://")) {
        void shell.openExternal(targetUrl);
      }
    });

    contents.setWindowOpenHandler((details) => {
      if (details.url.startsWith("http://") || details.url.startsWith("https://")) {
        void shell.openExternal(details.url);
      }
      return { action: "deny" };
    });
  });

  // 2. Inject Strict CSP (development allows unsafe-inline/unsafe-eval for Vite React Refresh preamble and HMR)
  const isDev = !app.isPackaged;
  const scriptSrc = isDev
    ? "script-src 'self' 'unsafe-inline' 'unsafe-eval' blob:"
    : "script-src 'self'";

  const cspDirectives = [
    "default-src 'self'",
    scriptSrc,
    // Allow unsafe-inline for Tailwind/CSS-in-JS if needed, and Google Fonts
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
    "font-src 'self' https://fonts.gstatic.com data:",
    "img-src 'self' data: blob:",
    "worker-src 'self' blob:",
    // Allow local WS for Vite HMR in dev, and specific LLM provider APIs in prod
    "connect-src 'self' ws: wss: http://localhost:* http://127.0.0.1:* https://api.openai.com https://api.anthropic.com http://localhost:11434",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'none'"
  ].join("; ");

  session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
    callback({
      responseHeaders: {
        ...details.responseHeaders,
        "Content-Security-Policy": [cspDirectives]
      }
    });
  });

  console.log("[Security] CSP and WebPreferences hardened.");
}
