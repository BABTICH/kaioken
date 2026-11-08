// src/renderer/components/welcome/WelcomeSurface.tsx
import { useStore } from "@nanostores/react";
import desktopLogo from "@renderer/assets/logo.png";
import {
  openWorkspace,
  openWorkspacePath,
  workspaceStore
} from "@renderer/stores/explorer";
import {
  recentProjectsStore,
  removeRecentProject,
  showWelcomeOnStartup,
  setShowWelcomeOnStartup
} from "@renderer/stores/recent-projects";
import { setSurface, toggleTerminal } from "@renderer/stores/ui";
import {
  FolderOpen,
  MessageSquare,
  Sparkles,
  Network,
  Terminal,
  Clock,
  X,
  ArrowRight,
  CheckCircle2,
  LayoutGrid,
  CircleDollarSign,
  Sliders,
  Plus
} from "lucide-react";

function formatRelativeTime(timestamp: number): string {
  const diff = Date.now() - timestamp;
  const minutes = Math.floor(diff / 60000);
  if (minutes < 1) return "Just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d ago`;
  const weeks = Math.floor(days / 7);
  return `${weeks}w ago`;
}

interface QuickAction {
  id: string;
  label: string;
  description: string;
  hotkey: string;
  icon: React.ComponentType<{ size?: number; className?: string }>;
  action: () => void;
  accent?: boolean;
}

export function WelcomeSurface() {
  const explorer = useStore(workspaceStore);
  const recents = useStore(recentProjectsStore);
  const showOnStartup = useStore(showWelcomeOnStartup);

  const currentRoot = explorer.workspace?.root;

  const handleOpenFolder = async () => {
    await openWorkspace();
    // Navigate away from welcome once a workspace is selected
    if (workspaceStore.get().workspace) {
      setSurface("chat");
    }
  };

  const handleOpenRecent = async (path: string) => {
    await openWorkspacePath(path);
    setSurface("chat");
  };

  const quickActions: QuickAction[] = [
    {
      id: "open",
      label: "Open Folder…",
      description: "Browse and open a local repository or project directory",
      hotkey: "Ctrl+O",
      icon: FolderOpen,
      action: () => void handleOpenFolder(),
      accent: true
    },
    {
      id: "chat",
      label: "New Agent Chat",
      description: "Start a grounded AI coding session with streaming reasoning",
      hotkey: "Ctrl+1",
      icon: MessageSquare,
      action: () => setSurface("chat")
    },
    {
      id: "research",
      label: "Deep Research",
      description: "Multi-hop symbol indexing and semantic codebase queries",
      hotkey: "Ctrl+2",
      icon: Sparkles,
      action: () => setSurface("research")
    },
    {
      id: "codemap",
      label: "Interactive Codemap",
      description: "Force-directed particle graph of modules and dependencies",
      hotkey: "Ctrl+4",
      icon: Network,
      action: () => setSurface("codemap")
    },
    {
      id: "cards",
      label: "Flash Cards",
      description: "AI-generated study cards from your codebase knowledge",
      hotkey: "Ctrl+5",
      icon: LayoutGrid,
      action: () => setSurface("cards")
    },
    {
      id: "terminal",
      label: "Open Terminal",
      description: "Launch an embedded PTY session in your workspace",
      hotkey: "Ctrl+`",
      icon: Terminal,
      action: () => toggleTerminal()
    }
  ];

  return (
    <div
      className="welcome-surface relative flex h-full flex-col overflow-hidden select-none"
      style={{ background: "var(--bg)" }}
    >
      {/* Ambient background gradient */}
      <div
        className="pointer-events-none absolute inset-0 z-0"
        style={{
          background:
            "radial-gradient(ellipse 80% 60% at 50% -10%, color-mix(in srgb, var(--accent) 8%, transparent) 0%, transparent 70%)"
        }}
      />

      <div className="relative z-10 flex h-full flex-col overflow-y-auto">
        {/* ── Hero ─────────────────────────────────────────── */}
        <header className="flex flex-col items-center justify-center px-8 pt-12 pb-8 text-center">
          {/* Logo with glow */}
          <div
            className="relative mb-5"
            style={{
              filter: "drop-shadow(0 0 28px color-mix(in srgb, var(--accent) 40%, transparent))"
            }}
          >
            <img
              src={desktopLogo}
              alt="Kaioken Desktop App"
              className="h-16 w-auto object-contain"
              style={{ animation: "float 4s ease-in-out infinite" }}
            />
          </div>

          <h1
            className="font-bold tracking-tight"
            style={{
              fontSize: "clamp(1.4rem, 3vw, 2rem)",
              color: "var(--fg)",
              letterSpacing: "-0.02em"
            }}
          >
            Kaioken
            <span
              style={{
                marginLeft: "0.45em",
                color: "var(--accent)",
                opacity: 0.85
              }}
            >
              Desktop
            </span>
          </h1>

          <p
            className="mt-2 max-w-sm"
            style={{ fontSize: "var(--fs-sm)", color: "var(--fg-mute)" }}
          >
            Grounded AI agent workspace — deterministic, isolated, verified.
          </p>

          <div className="mt-3 flex flex-wrap items-center justify-center gap-2">
            <span className="badge badge--good font-mono text-[10px] flex items-center gap-1">
              <span
                className="h-1.5 w-1.5 rounded-full"
                style={{ background: "var(--green)", animation: "pulse 2s infinite" }}
              />
              v0.8.1
            </span>
            <span className="badge font-mono text-[10px]">ISOLATED SANDBOX</span>
          </div>
        </header>

        {/* ── Active Workspace Banner ──────────────────────── */}
        {currentRoot && (
          <div className="mx-6 mb-4">
            <div
              className="flex items-center justify-between gap-4 rounded-md px-4 py-3"
              style={{
                border: "1px solid var(--accent)",
                background: "var(--accent-soft)"
              }}
            >
              <div className="flex items-center gap-3 min-w-0">
                <CheckCircle2 size={16} style={{ color: "var(--accent)", flexShrink: 0 }} />
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span
                      className="font-semibold truncate"
                      style={{ fontSize: "var(--fs-sm)", color: "var(--fg)" }}
                    >
                      {explorer.workspace?.name}
                    </span>
                    <span className="badge badge--good text-[9px]">ACTIVE</span>
                  </div>
                  <div
                    className="mono truncate"
                    style={{ fontSize: "var(--fs-xs)", color: "var(--fg-mute)", maxWidth: "36ch" }}
                  >
                    {currentRoot}
                  </div>
                </div>
              </div>
              <button
                type="button"
                className="btn btn--sm inline-flex items-center gap-1.5 shrink-0"
                onClick={() => setSurface("chat")}
              >
                <span>Continue</span>
                <ArrowRight size={13} />
              </button>
            </div>
          </div>
        )}

        {/* ── Two-Column Body ──────────────────────────────── */}
        <div className="flex flex-1 gap-6 overflow-hidden px-6 pb-6 min-h-0">

          {/* ─ Left: Recent Projects ─────────────────────── */}
          <div
            className="flex w-80 shrink-0 flex-col rounded-md"
            style={{
              border: "var(--hair) solid var(--rule)",
              background: "var(--bg-raise)"
            }}
          >
            {/* Header */}
            <div
              className="flex items-center justify-between px-4 py-3"
              style={{ borderBottom: "var(--hair) solid var(--rule)" }}
            >
              <div className="flex items-center gap-2">
                <Clock size={13} style={{ color: "var(--fg-mute)" }} />
                <span
                  className="font-semibold"
                  style={{ fontSize: "var(--fs-xs)", color: "var(--fg)" }}
                >
                  Recent
                </span>
              </div>
              {recents.length > 0 && (
                <span
                  className="mono"
                  style={{ fontSize: "var(--fs-micro)", color: "var(--fg-mute)" }}
                >
                  {recents.length} project{recents.length !== 1 ? "s" : ""}
                </span>
              )}
            </div>

            {/* List */}
            <div className="hud-scrollbar flex-1 overflow-y-auto p-2">
              {recents.length === 0 ? (
                <div className="flex h-full flex-col items-center justify-center gap-3 py-8 text-center">
                  <FolderOpen
                    size={32}
                    style={{ color: "var(--fg-mute)", opacity: 0.4 }}
                  />
                  <div>
                    <div
                      className="font-medium"
                      style={{ fontSize: "var(--fs-sm)", color: "var(--fg-2)" }}
                    >
                      No recent projects
                    </div>
                    <p
                      className="mt-1 max-w-[180px] mx-auto"
                      style={{ fontSize: "var(--fs-xs)", color: "var(--fg-mute)" }}
                    >
                      Open a folder to start tracking workspaces
                    </p>
                  </div>
                  <button
                    type="button"
                    className="btn btn--sm inline-flex items-center gap-1.5"
                    onClick={() => void handleOpenFolder()}
                  >
                    <Plus size={12} />
                    <span>Open Folder</span>
                  </button>
                </div>
              ) : (
                <div className="flex flex-col gap-1">
                  {recents.map((item) => {
                    const isCurrent =
                      currentRoot &&
                      currentRoot.toLowerCase() === item.path.toLowerCase();

                    return (
                      <button
                        key={item.path}
                        type="button"
                        className="group relative w-full rounded text-left transition-all"
                        style={{
                          padding: "8px 10px",
                          border: isCurrent
                            ? "1px solid var(--accent)"
                            : "1px solid transparent",
                          background: isCurrent
                            ? "var(--accent-soft)"
                            : "transparent"
                        }}
                        onMouseEnter={(e) => {
                          if (!isCurrent) {
                            (e.currentTarget as HTMLButtonElement).style.background =
                              "var(--bg-sunk)";
                            (e.currentTarget as HTMLButtonElement).style.borderColor =
                              "var(--rule)";
                          }
                        }}
                        onMouseLeave={(e) => {
                          if (!isCurrent) {
                            (e.currentTarget as HTMLButtonElement).style.background =
                              "transparent";
                            (e.currentTarget as HTMLButtonElement).style.borderColor =
                              "transparent";
                          }
                        }}
                        onClick={() => void handleOpenRecent(item.path)}
                      >
                        <div className="flex items-start justify-between gap-2">
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-1.5">
                              <span
                                className="font-medium truncate"
                                style={{
                                  fontSize: "var(--fs-sm)",
                                  color: isCurrent ? "var(--accent)" : "var(--fg)"
                                }}
                              >
                                {item.name}
                              </span>
                              {isCurrent && (
                                <span className="badge badge--good text-[8px] shrink-0">
                                  NOW
                                </span>
                              )}
                            </div>
                            <div
                              className="mono truncate mt-0.5"
                              style={{
                                fontSize: "var(--fs-micro)",
                                color: "var(--fg-mute)",
                                maxWidth: "220px"
                              }}
                            >
                              {item.path}
                            </div>
                          </div>

                          <div className="flex shrink-0 flex-col items-end gap-1">
                            <span
                              className="mono"
                              style={{
                                fontSize: "var(--fs-micro)",
                                color: "var(--fg-mute)"
                              }}
                            >
                              {formatRelativeTime(item.lastOpened)}
                            </span>
                            <button
                              type="button"
                              aria-label="Remove from recents"
                              className="opacity-0 group-hover:opacity-100 transition-opacity rounded p-0.5"
                              style={{ color: "var(--fg-mute)" }}
                              onMouseEnter={(e) =>
                                ((e.currentTarget as HTMLButtonElement).style.color =
                                  "var(--bad)")
                              }
                              onMouseLeave={(e) =>
                                ((e.currentTarget as HTMLButtonElement).style.color =
                                  "var(--fg-mute)")
                              }
                              onClick={(e) => {
                                e.stopPropagation();
                                removeRecentProject(item.path);
                              }}
                            >
                              <X size={11} />
                            </button>
                          </div>
                        </div>
                      </button>
                    );
                  })}
                </div>
              )}
            </div>

            {/* Footer: open new */}
            <div
              className="p-3"
              style={{ borderTop: "var(--hair) solid var(--rule)" }}
            >
              <button
                type="button"
                className="btn w-full inline-flex items-center justify-center gap-2"
                style={{ background: "var(--accent)", color: "#fff", border: "none" }}
                onClick={() => void openWorkspace()}
              >
                <FolderOpen size={14} />
                <span>Open Folder…</span>
              </button>
            </div>
          </div>

          {/* ─ Right: Quick Actions ──────────────────────── */}
          <div className="flex flex-1 flex-col gap-3 overflow-y-auto min-w-0">
            <div
              className="mb-1"
              style={{ fontSize: "var(--fs-xs)", color: "var(--fg-mute)" }}
            >
              <span
                className="mono"
                style={{ color: "var(--accent)", marginRight: "0.5em" }}
              >
                ▸
              </span>
              Quick actions
            </div>

            {quickActions.map((action) => {
              const Icon = action.icon;
              return (
                <button
                  key={action.id}
                  type="button"
                  className="group flex w-full items-start gap-4 rounded-md p-4 text-left transition-all"
                  style={{
                    border: "var(--hair) solid var(--rule)",
                    background: "var(--bg-raise)"
                  }}
                  onMouseEnter={(e) => {
                    const el = e.currentTarget as HTMLButtonElement;
                    el.style.borderColor = "var(--accent)";
                    el.style.background = "var(--bg-sunk)";
                  }}
                  onMouseLeave={(e) => {
                    const el = e.currentTarget as HTMLButtonElement;
                    el.style.borderColor = "var(--rule)";
                    el.style.background = "var(--bg-raise)";
                  }}
                  onClick={action.action}
                >
                  <div
                    className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md transition-all"
                    style={{
                      border: "var(--hair) solid var(--rule)",
                      background: action.accent
                        ? "var(--accent-soft)"
                        : "var(--bg)",
                      color: action.accent ? "var(--accent)" : "var(--fg-2)"
                    }}
                  >
                    <Icon size={17} />
                  </div>

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between gap-2">
                      <span
                        className="font-semibold group-hover:text-[var(--accent)] transition-colors"
                        style={{ fontSize: "var(--fs-sm)", color: "var(--fg)" }}
                      >
                        {action.label}
                      </span>
                      <kbd
                        className="mono shrink-0 rounded px-1.5 py-0.5"
                        style={{
                          fontSize: "10px",
                          color: "var(--fg-mute)",
                          border: "var(--hair) solid var(--rule)",
                          background: "var(--bg-sunk)"
                        }}
                      >
                        {action.hotkey}
                      </kbd>
                    </div>
                    <p
                      className="mt-0.5 line-clamp-1"
                      style={{ fontSize: "var(--fs-xs)", color: "var(--fg-mute)" }}
                    >
                      {action.description}
                    </p>
                  </div>
                </button>
              );
            })}

            {/* Other surfaces row */}
            <div className="flex gap-2 pt-1">
              {[
                { label: "Cost", id: "cost" as const, icon: CircleDollarSign },
                { label: "System", id: "settings" as const, icon: Sliders }
              ].map(({ label, id, icon: Icon }) => (
                <button
                  key={id}
                  type="button"
                  className="flex flex-1 items-center gap-2 rounded-md p-3 transition-all"
                  style={{
                    border: "var(--hair) solid var(--rule)",
                    background: "var(--bg-raise)",
                    fontSize: "var(--fs-xs)",
                    color: "var(--fg-mute)"
                  }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.color = "var(--fg)";
                    (e.currentTarget as HTMLButtonElement).style.borderColor =
                      "var(--rule-strong)";
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.color = "var(--fg-mute)";
                    (e.currentTarget as HTMLButtonElement).style.borderColor = "var(--rule)";
                  }}
                  onClick={() => setSurface(id)}
                >
                  <Icon size={13} />
                  <span>{label}</span>
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* ── Footer ──────────────────────────────────────── */}
        <footer
          className="flex flex-wrap items-center justify-between gap-3 px-6 py-3"
          style={{
            borderTop: "var(--hair) solid var(--rule)",
            fontSize: "var(--fs-xs)",
            color: "var(--fg-mute)"
          }}
        >
          <label className="flex cursor-pointer items-center gap-2">
            <input
              type="checkbox"
              className="cursor-pointer"
              style={{ accentColor: "var(--accent)" }}
              checked={showOnStartup}
              onChange={(e) => setShowWelcomeOnStartup(e.target.checked)}
            />
            <span>Show on startup</span>
          </label>

          <div className="flex items-center gap-4 mono text-[10px]">
            <span>Ctrl+` Terminal</span>
            <span>Ctrl+B Inspector</span>
            <span>Ctrl+K Palette</span>
            <span
              className="badge font-mono text-[9px]"
              style={{ marginLeft: "4px" }}
            >
              kaioken v0.8.1
            </span>
          </div>
        </footer>
      </div>

      <style>{`
        @keyframes float {
          0%, 100% { transform: translateY(0px); }
          50% { transform: translateY(-6px); }
        }
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
      `}</style>
    </div>
  );
}
