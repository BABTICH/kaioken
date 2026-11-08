import * as React from "react"
import { RefreshCw } from "lucide-react"
import { WITHOUT_KAIOKEN_SCRIPT, WITH_KAIOKEN_SCRIPT } from "@/data/comparison"
import TerminalWindow from "@/components/terminal/TerminalWindow"
import Badge from "@/components/ui/Badge"
import Reveal from "@/components/ui/Reveal"
import { cn } from "@/lib/utils"

export default function ProblemSolution() {
  const [replayKey, setReplayKey] = React.useState(0)

  return (
    <section id="anti-slop" className="py-24 border-b border-white/10 bg-[#060608]">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        {/* Section Header */}
        <Reveal>
          <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 mb-12">
            <div>
              <Badge variant="rose" isComment className="mb-3">
                THE PROBLEM VS THE SOLUTION
              </Badge>
              <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl">
                Stop patching{" "}
                <span className="text-kai-rose line-through opacity-80">AI slop.</span>{" "}
                <span className="text-kai-green">Build with taste.</span>
              </h2>
              <p className="mt-3 max-w-xl font-sans text-sm text-kai-muted sm:text-base">
                Generic coding assistants generate sloppy, out-of-context patches that break builds and dirty your working tree. Kaioken v2 grounds every change in your architecture.
              </p>
            </div>

            <button
              onClick={() => setReplayKey((k) => k + 1)}
              className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2 font-mono text-xs text-kai-text transition-colors hover:border-white/20 hover:text-white cursor-pointer self-start md:self-auto"
            >
              <RefreshCw className="size-3.5 text-kai-orange" />
              <span>Replay Comparison</span>
            </button>
          </div>
        </Reveal>

        {/* Side-by-side terminal comparison */}
        <div key={replayKey} className="grid gap-6 md:grid-cols-2">
          {/* Left: Without Kaioken (Agony) */}
          <Reveal delay={0.1}>
            <div className="flex flex-col h-full">
              <div className="mb-3 flex items-center justify-between font-mono text-xs">
                <span className="font-semibold text-kai-rose">✕ Without Kaioken</span>
                <span className="text-kai-dim">The Generic Agent Experience</span>
              </div>

              <TerminalWindow
                title="conventional-agent"
                status="interrupted"
                accentColor="#ff5f5f"
                className="border-kai-rose/20 bg-[#0d0909] flex-1"
              >
                <div className="space-y-2 text-xs font-mono">
                  {WITHOUT_KAIOKEN_SCRIPT.map((line, idx) => (
                    <div
                      key={idx}
                      className={cn(
                        "leading-relaxed",
                        line.type === "prompt" && "text-kai-blue font-semibold",
                        line.type === "wrong" && "text-kai-rose",
                        line.type === "agent" && "text-kai-muted",
                        line.type === "code" && "text-white/60 bg-black/40 px-2 py-0.5 rounded font-mono"
                      )}
                    >
                      {line.text}
                    </div>
                  ))}
                </div>
              </TerminalWindow>
            </div>
          </Reveal>

          {/* Right: With Kaioken v2 (Flow State) */}
          <Reveal delay={0.2}>
            <div className="flex flex-col h-full">
              <div className="mb-3 flex items-center justify-between font-mono text-xs">
                <span className="font-semibold text-kai-green">✓ With Kaioken v2</span>
                <span className="text-kai-dim">Autonomous Architectural Flow</span>
              </div>

              <TerminalWindow
                title="kaioken-v2-session"
                status="verified pass"
                accentColor="#00d787"
                className="border-kai-green/30 bg-[#080d0a] shadow-[0_0_30px_rgba(0,215,135,0.07)] flex-1"
              >
                <div className="space-y-2 text-xs font-mono">
                  {WITH_KAIOKEN_SCRIPT.map((line, idx) => (
                    <div
                      key={idx}
                      className={cn(
                        "leading-relaxed",
                        line.type === "prompt" && "text-kai-blue font-semibold",
                        line.type === "correct" && "text-kai-green font-medium",
                        line.type === "system" && "text-kai-tan",
                        line.type === "tool" && "text-kai-sage",
                        line.type === "code" && "text-kai-green bg-kai-green/10 px-2 py-0.5 rounded font-mono"
                      )}
                    >
                      {line.text}
                    </div>
                  ))}
                </div>
              </TerminalWindow>
            </div>
          </Reveal>
        </div>
      </div>
    </section>
  )
}
