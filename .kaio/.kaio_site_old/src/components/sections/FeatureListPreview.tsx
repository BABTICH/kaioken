import * as React from "react"
import { FEATURES, type FeatureItem } from "@/data/features"
import TerminalWindow from "@/components/terminal/TerminalWindow"
import Badge from "@/components/ui/Badge"
import Reveal from "@/components/ui/Reveal"
import { cn } from "@/lib/utils"

export default function FeatureListPreview() {
  const [activeFeature, setActiveFeature] = React.useState<FeatureItem>(FEATURES[0])

  return (
    <section id="features" className="py-24 border-b border-white/10 bg-[#050507]">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        {/* Section Header */}
        <Reveal>
          <div className="mb-14">
            <Badge variant="orange" isComment className="mb-3">
              ARCHITECTURAL ADVANTAGES
            </Badge>
            <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl">
              Every capability you need.{" "}
              <span className="text-kai-orange">None of the slop.</span>
            </h2>
            <p className="mt-3 max-w-xl font-sans text-sm text-kai-muted sm:text-base">
              Explore the core architectural engines that differentiate Kaioken v2 from surface-level coding assistants.
            </p>
          </div>
        </Reveal>

        {/* 2-Column Split: Numbered List on Left, Live Terminal on Right */}
        <div className="grid gap-10 lg:grid-cols-12 lg:items-start">
          {/* Left Column: Numbered List (01 - 07) */}
          <div className="flex flex-col divide-y divide-white/10 border-y border-white/10 lg:col-span-6">
            {FEATURES.map((item) => {
              const isActive = activeFeature.id === item.id
              return (
                <div
                  key={item.id}
                  onMouseEnter={() => setActiveFeature(item)}
                  onClick={() => setActiveFeature(item)}
                  className={cn(
                    "group relative flex cursor-pointer flex-col p-5 transition-all duration-200",
                    isActive
                      ? "bg-white/[0.04] border-l-2 border-l-kai-orange"
                      : "hover:bg-white/[0.02] border-l-2 border-l-transparent"
                  )}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span
                        className={cn(
                          "font-mono text-xs font-bold transition-colors",
                          isActive ? "text-kai-orange" : "text-kai-dim group-hover:text-kai-muted"
                        )}
                      >
                        {item.number}
                      </span>
                      <h3
                        className={cn(
                          "font-sans text-base font-semibold transition-colors",
                          isActive ? "text-white" : "text-kai-text group-hover:text-white"
                        )}
                      >
                        {item.title}
                      </h3>
                    </div>

                    <span className="rounded border border-white/10 bg-white/5 px-2 py-0.5 font-mono text-[10px] text-kai-muted">
                      {item.tag}
                    </span>
                  </div>

                  <p className="mt-2 font-sans text-xs leading-relaxed text-kai-muted">
                    {item.description}
                  </p>
                </div>
              )
            })}
          </div>

          {/* Right Column: Live Terminal Preview Pane */}
          <div className="sticky top-24 lg:col-span-6">
            <TerminalWindow
              title={`kaioken — ${activeFeature.id}`}
              status="live preview"
              className="min-h-[380px] shadow-2xl border-white/15"
            >
              <div className="space-y-2">
                {/* Command prompt */}
                <div className="flex items-center gap-2 text-kai-blue font-semibold">
                  <span>›</span>
                  <span>{activeFeature.terminalOutput.command}</span>
                </div>

                {/* Simulated execution output */}
                <div className="mt-3 space-y-1.5 font-mono text-xs">
                  {activeFeature.terminalOutput.lines.map((line, idx) => {
                    const colorClass = {
                      user: "text-kai-blue font-semibold",
                      tool: "text-kai-tan",
                      toolres: "text-kai-sage",
                      add: "text-kai-green",
                      del: "text-kai-rose",
                      ok: "text-kai-green font-medium",
                      dim: "text-kai-dim",
                      warn: "text-kai-amber",
                    }[line.type]

                    return (
                      <div key={idx} className={cn("leading-relaxed", colorClass)}>
                        {line.text}
                      </div>
                    )
                  })}
                </div>
              </div>
            </TerminalWindow>

            <div className="mt-4 flex items-center justify-between font-mono text-[11px] text-kai-dim">
              <span>Hover or tap items to inspect execution</span>
              <span className="text-kai-amber">07 capabilities total</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
