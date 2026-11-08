import { ArrowRight, CheckCircle2, GitBranch, Layers, ShieldCheck, Terminal } from "lucide-react"
import Badge from "@/components/ui/Badge"
import Reveal from "@/components/ui/Reveal"

const STEPS = [
  {
    step: "01",
    cmd: "kaioken init && scan",
    title: "Inventory & Scope",
    body: "Scans repository AST, respects .gitignore and scope exclusions, and calculates token budget before any model call.",
    icon: Terminal,
    color: "#ff8700",
  },
  {
    step: "02",
    cmd: "kaioken plan",
    title: "Propose modules.yaml",
    body: "The model discovers architectural module boundaries and outputs modules.yaml. Maintainers own and edit the boundaries.",
    icon: Layers,
    color: "#ffaf00",
  },
  {
    step: "03",
    cmd: "kaioken worktree spawn",
    title: "Worktree Isolation",
    body: "Sub-agents fork into isolated temporary git worktrees. Zero risk of dirtying your main workspace or breaking concurrent runs.",
    icon: GitBranch,
    color: "#8c4edd",
  },
  {
    step: "04",
    cmd: "kaioken verify",
    title: "Self-Healing Gate",
    body: "Executes native test/build suites, diagnoses compiler failures automatically, and validates that all tests pass.",
    icon: ShieldCheck,
    color: "#00d787",
  },
  {
    step: "05",
    cmd: "apply patch",
    title: "Diff Approval",
    body: "Unified git diff lands in your repository only after your explicit confirmation. Complete auditability and instant /undo.",
    icon: CheckCircle2,
    color: "#87d7ff",
  },
]

export default function ArchitectureFlow() {
  return (
    <section id="pipeline" className="py-24 border-b border-white/10 bg-[#050507]">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        {/* Header */}
        <Reveal>
          <div className="mb-14 text-center">
            <Badge variant="orange" isComment className="mb-3">
              THE AUTONOMOUS PIPELINE
            </Badge>
            <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl">
              From raw codebase to verified production diffs.
            </h2>
            <p className="mx-auto mt-3 max-w-xl font-sans text-sm text-kai-muted sm:text-base">
              A structured lifecycle that enforces architectural discipline at every stage.
            </p>
          </div>
        </Reveal>

        {/* Step Flow Grid */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
          {STEPS.map((s, idx) => {
            const Icon = s.icon
            return (
              <Reveal key={s.step} delay={idx * 0.08}>
                <div className="group relative flex flex-col justify-between h-full rounded-lg border border-white/10 bg-[#0a0b10] p-5 transition-all duration-300 hover:border-white/20 hover:bg-[#0f1016]">
                  <div>
                    {/* Step number and icon */}
                    <div className="flex items-center justify-between">
                      <span className="font-mono text-xs font-bold text-kai-dim group-hover:text-white transition-colors">
                        {s.step}
                      </span>
                      <Icon className="size-4" style={{ color: s.color }} />
                    </div>

                    <div className="mt-4">
                      <span className="font-mono text-[11px] text-kai-muted bg-white/5 px-2 py-0.5 rounded border border-white/5">
                        {s.cmd}
                      </span>
                      <h3 className="mt-2.5 font-sans text-base font-semibold text-white">
                        {s.title}
                      </h3>
                      <p className="mt-2 font-sans text-xs leading-relaxed text-kai-muted">
                        {s.body}
                      </p>
                    </div>
                  </div>

                  {idx < STEPS.length - 1 && (
                    <div className="mt-4 pt-3 border-t border-white/5 flex items-center justify-end font-mono text-[10px] text-kai-dim">
                      <ArrowRight className="size-3 text-kai-dim group-hover:text-kai-orange group-hover:translate-x-1 transition-all" />
                    </div>
                  )}
                </div>
              </Reveal>
            )
          })}
        </div>
      </div>
    </section>
  )
}
