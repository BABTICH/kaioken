import { useEffect, useState } from "react";
import { useStore } from "@nanostores/react";
import { resolvedTheme, toggleTheme } from "@renderer/stores/theme";
import { Minus, Square, Copy, X, Sun, Moon, Flame } from "lucide-react";

interface TitleBarButtonProps {
  onClick: () => void;
  danger?: boolean;
  title?: string;
  ariaLabel?: string;
  children: React.ReactNode;
}

function TitleBarButton({
  children,
  onClick,
  danger = false,
  title,
  ariaLabel
}: TitleBarButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={title}
      aria-label={ariaLabel ?? title}
      className={[
        "k-btn",
        "k-btn--icon",
        "titlebar-no-drag",
        danger ? "k-btn--danger" : ""
      ].join(" ")}
    >
      {children}
    </button>
  );
}

export function TitleBar() {
  const [maximized, setMaximized] = useState(false);
  const theme = useStore(resolvedTheme);

  useEffect(() => {
    window.api.window
      .isMaximized()
      .then(setMaximized)
      .catch(() => setMaximized(false));
  }, []);

  const handleMaximize = async () => {
    try {
      await window.api.window.maximize();
      const nextMaximized = await window.api.window.isMaximized();
      setMaximized(nextMaximized);
    } catch (error) {
      console.error("[TitleBar] Failed to toggle maximize", error);
    }
  };

  return (
    <header
      className={[
        "titlebar-drag",
        "hairline-b",
        "flex h-11 items-center justify-between px-3 select-none"
      ].join(" ")}
      style={{
        background: "color-mix(in srgb, var(--bg) 92%, transparent)",
        backdropFilter: "blur(12px)"
      }}
    >
      <div className="flex items-center gap-3">
        <div className="k-brand flex items-center gap-1.5">
          <Flame size={14} className="text-[var(--accent)] shrink-0" />
          <span>KAIOKEN</span>
        </div>

        <div className="k-brand-sub">Desktop Studio</div>
      </div>

      <div className="flex items-center gap-1">
        <TitleBarButton
          title={`Switch to ${theme === "dark" ? "light" : "dark"} mode`}
          ariaLabel="Toggle theme"
          onClick={() => toggleTheme()}
        >
          {theme === "light" ? <Moon size={14} /> : <Sun size={14} />}
        </TitleBarButton>

        <TitleBarButton
          title="Minimize"
          ariaLabel="Minimize window"
          onClick={() => {
            void window.api.window.minimize();
          }}
        >
          <Minus size={13} />
        </TitleBarButton>

        <TitleBarButton
          title={maximized ? "Restore" : "Maximize"}
          ariaLabel={maximized ? "Restore window" : "Maximize window"}
          onClick={() => {
            void handleMaximize();
          }}
        >
          {maximized ? <Copy size={11} /> : <Square size={11} />}
        </TitleBarButton>

        <TitleBarButton
          title="Close"
          ariaLabel="Close window"
          danger
          onClick={() => {
            void window.api.window.close();
          }}
        >
          <X size={14} />
        </TitleBarButton>
      </div>
    </header>
  );
}
