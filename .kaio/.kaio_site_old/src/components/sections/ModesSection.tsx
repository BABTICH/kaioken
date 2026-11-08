import * as React from "react"
import { CheckCircle2, FastForward, GitFork, MessageSquare, Search } from "lucide-react"
import Badge from "@/components/ui/Badge"
import Reveal from "@/components/ui/Reveal"
import { cn } from "@/lib/utils"

interface ModeDetail {
  id: string
  name: string
  shortcut: string
  icon: React.ComponentType<{ className?: string; style?: React.CSSProperties }>
  color: string
  tagline: string
  description: string
  features: string[]
  terminalSnippet: string[]
}

const MODES: ModeDetail[] = [
  {
    id: "interactive",
    name: "Interactive Mode",
    shortcut: "default",
    icon: MessageSquare,
    color: "#ff8700",
    tagline: "Conversational flow with per-edit human approval",
    description:
      "Real-time token streaming with syntax highlighting. Proposes modifications as readable diffs with unified confirmation prompts.",
    features: [
      "Token-by-token streaming markdown rendering",
      "Per-edit user diff approval before write",
      "Inline steering queue while agent is running",
    ],
    terminalSnippet: [
      "› wrap the validate error in handler.go",
      "● proposed edit: internal/api/handler.go",
      "-   return nil",
      "+   return fmt.Errorf(\"validate: %w\", err)",
      "apply edit? [y]es [n]o › y",
      "✓ applied · 1 file changed",
    ],
  },
  {
    id: "worktree",
    name: "Worktree Sandbox",
    shortcut: "/worktree",
    icon: GitFork,
    color: "#8c4edd",
    tagline: "Fire-and-forget sub-agent execution in parallel worktrees",
    description:
      "Spins up an isolated git worktree for sub-agents to complete heavy refactors, run test suites, and report back without locking your main session.",
    features: [
      "Isolated filesystem in temporary worktree",
      "Zero dirty files or merge conflicts on main branch",
      "Resume and inspect any background run with --id",
    ],
    terminalSnippet: [
      "› kaioken worktree spawn \"migrate db schema to v2\"",
      "● git worktree created at .kaioken/worktrees/task-88",
      "● sub-agent executing migration and tests in background...",
      "✓ task finished · unified diff patch generated",
    ],
  },
  {
    id: "plan",
    name: "Plan Mode",
    shortcut: "/plan",
    icon: Search,
    color: "#87d7ff",
    tagline: "Read-only architectural analysis and module boundary mapping",
    description:
      "Analyze large codebases, trace symbol dependencies, and design architectures without executing any file modifications.",
    features: [
      "Strictly read-only sandbox with zero disk modifications",
      "Proposes modules.yaml boundaries for review",
      "Generates implementation plans before coding begins",
    ],
    terminalSnippet: [
      "› kaioken /plan",
      "● analyzing 148 repository files...",
      "● proposed 6 module boundaries in modules.yaml",
      "● no files were modified · plan ready for maintainer review",
    ],
  },
  {
    id: "yolo",
    name: "Auto-Accept / YOLO",
    shortcut: "shift+tab / /yolo",
    icon: FastForward,
    color: "#00d787",
    tagline: "Skip confirmation prompts for trusted high-speed workflows",
    description:
      "Instantly toggle auto-approval mode for routine chores, boilerplate generation, and large multi-file migrations. Toggle back anytime with shift+tab.",
    features: [
      "Instant 1-key toggle between safe & auto-accept",
      "Autonomous verification gate still enforces test passes",
      "Automatic git checkpoint rollback available via /undo",
    ],
    terminalSnippet: [
      "› /yolo",
      "● auto-accept mode ENABLED (shift+tab to toggle back)",
      "● applying edits across 12 files...",
      "✓ 12/12 applied automatically · tests verified green",
    ],
  },
]

export default function ModesSection() {
  const [activeMode, setActiveMode] = React.useState<ModeDetail>(MODES[0])

  return (
    <section id="modes" className="py-24 border-b border-white/10 bg-[#060608]">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        {/* Header */}
        <Reveal>
          <div className="mb-14">
            <Badge variant="amber" isComment className="mb-3">
              FLEXIBLE EXECUTION SURFACES
            </Badge>
            <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl">
              A mode for every moment.
            </h2>
            <p className="mt-3 max-w-xl font-sans text-sm text-kai-muted sm:text-base">
              From conversational pair programming to fire-and-forget background worktree sandboxes, switch seamlessly to match your task.
            </p>
          </div>
        </Reveal>

        {/* Tab Buttons */}
        <div className="flex flex-wrap gap-2 pb-6 border-b border-white/10">
          {MODES.map((m) => {
            const Icon = m.icon
            const isActive = activeMode.id === m.id
            return (
              <button
                key={m.id}
                onClick={() => setActiveMode(m)}
                className={cn(
                  "flex items-center gap-2 rounded-full px-4 py-2 font-mono text-xs transition-all cursor-pointer",
                  isActive
                    ? "bg-white/15 text-white shadow-md border border-white/20"
                    : "bg-white/5 text-kai-muted border border-white/5 hover:border-white/15 hover:text-white"
                )}
              >
                <Icon className="size-3.5" style={{ color: m.color }} />
                <span>{m.name}</span>
                <span className="text-[10px] text-kai-dim">[{m.shortcut}]</span>
              </button>
            )
          })}
        </div>

        {/* Active Mode Card & Terminal Split */}
        <div className="mt-8 grid gap-8 lg:grid-cols-12 items-center">
          {/* Left info */}
          <div className="lg:col-span-6 space-y-5">
            <div>
              <span
                className="font-mono text-xs uppercase tracking-wider font-semibold"
                style={{ color: activeMode.color }}
              >
                {activeMode.shortcut}
              </span>
              <h3 className="mt-1 font-sans text-2xl font-bold text-white">
                {activeMode.tagline}
              </h3>
              <p className="mt-3 font-sans text-sm text-kai-muted leading-relaxed">
                {activeMode.description}
              </p>
            </div>

            <ul className="space-y-2.5 pt-2">
              {activeMode.features.map((feat, i) => (
                <li key={i} className="flex items-center gap-2.5 font-sans text-xs text-kai-text">
                  <CheckCircle2 className="size-4 shrink-0 text-kai-green" />
                  <span>{feat}</span>
                </li>
              ))}
            </ul>
          </div>

          {/* Right terminal mockup */}
          <div className="lg:col-span-6">
            <div className="rounded-md border border-white/10 bg-[#0a0a0f] p-4 font-mono text-xs shadow-xl">
              <div className="flex items-center justify-between pb-3 mb-3 border-b border-white/10 text-kai-dim text-[11px]">
                <span>terminal — {activeMode.id}</span>
                <span style={{ color: activeMode.color }}>ACTIVE</span>
              </div>
              <div className="space-y-1.5 leading-relaxed">
                {activeMode.terminalSnippet.map((line, idx) => (
                  <div
                    key={idx}
                    className={cn(
                      line.startsWith("›") && "text-kai-blue font-semibold",
                      line.startsWith("●") && "text-kai-tan",
                      line.startsWith("-") && "text-kai-rose",
                      line.startsWith("+") && "text-kai-green",
                      line.startsWith("✓") && "text-kai-green font-medium",
                      !line.match(/^[›●\-+✓]/) && "text-kai-muted"
                    )}
                  >
                    {line}
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
