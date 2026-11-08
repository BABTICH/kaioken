// src/renderer/components/shell/NavRail.tsx
import { useStore } from "@nanostores/react";
import {
  activeSurface,
  setSurface,
  terminalOpen,
  toggleTerminal,
  inspectorOpen,
  toggleInspector,
  type PrimarySurface
} from "@renderer/stores/ui";
import {
  MessageSquare,
  Sparkles,
  BookOpen,
  Network,
  LayoutGrid,
  CircleDollarSign,
  Sliders,
  Terminal,
  PanelRight
} from "lucide-react";

interface NavItem {
  id: PrimarySurface;
  label: string;
  icon: React.ComponentType<{ size?: number; className?: string }>;
}

const navItems: NavItem[] = [
  {
    id: "chat",
    label: "Chat",
    icon: MessageSquare
  },
  {
    id: "research",
    label: "Research",
    icon: Sparkles
  },
  {
    id: "wiki",
    label: "Wiki",
    icon: BookOpen
  },
  {
    id: "codemap",
    label: "Codemap",
    icon: Network
  },
  {
    id: "cards",
    label: "Cards",
    icon: LayoutGrid
  },
  {
    id: "cost",
    label: "Cost",
    icon: CircleDollarSign
  },
  {
    id: "settings",
    label: "System",
    icon: Sliders
  }
];

export function NavRail() {
  const active = useStore(activeSurface);
  const isTerminalOpen = useStore(terminalOpen);
  const isInspectorOpen = useStore(inspectorOpen);

  return (
    <nav
      className="hairline-r flex w-[84px] shrink-0 flex-col justify-between p-2 select-none"
      style={{ background: "var(--bg)" }}
    >
      <div className="flex flex-col gap-1">
        <div className="k-label k-label--accent px-1 pb-1 pt-0.5 text-center">Surfaces</div>

        {navItems.map((item) => {
          const isActive = active === item.id;
          const Icon = item.icon;

          return (
            <button
              key={item.id}
              type="button"
              aria-label={item.label}
              onClick={() => setSurface(item.id)}
              className={["nav-btn", isActive ? "is-active" : ""].join(" ")}
            >
              <Icon size={17} className="nav-icon" />
              <span className="nav-label">{item.label}</span>
            </button>
          );
        })}
      </div>

      <div className="flex flex-col gap-1 pt-2">
        <div
          className="w-full my-1"
          style={{ height: "var(--hair)", background: "var(--rule)" }}
        />

        <button
          type="button"
          aria-label="Toggle terminal"
          title="Toggle Terminal (Ctrl+`)"
          onClick={() => toggleTerminal()}
          className={["nav-btn", isTerminalOpen ? "is-active" : ""].join(" ")}
        >
          <Terminal size={16} className="nav-icon" />
          <span className="nav-label">Term</span>
        </button>

        <button
          type="button"
          aria-label="Toggle inspector"
          title="Toggle Inspector (Ctrl+B)"
          onClick={() => toggleInspector()}
          className={["nav-btn", isInspectorOpen ? "is-active" : ""].join(" ")}
        >
          <PanelRight size={16} className="nav-icon" />
          <span className="nav-label">Panel</span>
        </button>
      </div>
    </nav>
  );
}
