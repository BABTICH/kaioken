// src/renderer/components/settings/ThemeSettings.tsx
import { useStore } from "@nanostores/react";
import {
  themeMode,
  resolvedTheme,
  setThemeMode,
  type ThemeMode
} from "@renderer/stores/theme";
import { Sun, Moon, Monitor, Check, Palette } from "lucide-react";

interface ThemeOption {
  id: ThemeMode;
  title: string;
  subtitle: string;
  icon: React.ComponentType<{ size?: number; className?: string }>;
}

const themeOptions: ThemeOption[] = [
  {
    id: "dark",
    title: "Dark Mode",
    subtitle: "Obsidian canvas with luminous red & ember accents",
    icon: Moon
  },
  {
    id: "light",
    title: "Light Mode",
    subtitle: "Clean paper studio with sharp contrast and ink typography",
    icon: Sun
  },
  {
    id: "system",
    title: "System Preference",
    subtitle: "Automatically synchronizes with your operating system",
    icon: Monitor
  }
];

export function ThemeSettings() {
  const currentMode = useStore(themeMode);
  const resolved = useStore(resolvedTheme);

  return (
    <div className="surface-card mb-5">
      <div className="head flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="head-num label flex items-center justify-center">
            <Palette size={15} />
          </span>
          <span className="head-title h3">Workspace Appearance</span>
        </div>
        <span className="head-note">
          ACTIVE: {resolved.toUpperCase()}
        </span>
      </div>

      <div className="space-y-4 p-5">
        <p className="lead">
          Customize the visual aesthetic of your desktop studio. Switching modes takes effect immediately across all surfaces, panels, and terminal sessions.
        </p>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
          {themeOptions.map((opt) => {
            const isSelected = currentMode === opt.id;
            const Icon = opt.icon;

            return (
              <button
                key={opt.id}
                type="button"
                onClick={() => setThemeMode(opt.id)}
                className={`flex flex-col items-start p-4 text-left transition-all ${
                  isSelected ? "border-[var(--accent)] bg-[var(--accent-soft)]" : "border-[var(--rule)] bg-[var(--bg-sunk)] hover:bg-[var(--bg-raise)]"
                }`}
                style={{
                  borderWidth: "var(--hair)",
                  borderStyle: "solid"
                }}
              >
                <div className="flex w-full items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <Icon
                      size={18}
                      className={isSelected ? "text-[var(--accent)]" : "text-[var(--fg-mute)]"}
                    />
                    <span className="font-semibold text-sm text-[var(--fg)]">
                      {opt.title}
                    </span>
                  </div>
                  {isSelected && (
                    <Check size={16} className="text-[var(--accent)] shrink-0" />
                  )}
                </div>

                <p className="text-xs text-[var(--fg-mute)] leading-relaxed">
                  {opt.subtitle}
                </p>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
