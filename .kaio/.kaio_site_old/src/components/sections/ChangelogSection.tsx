import { CHANGELOG } from "@/data/changelog"
import Badge from "@/components/ui/Badge"
import Reveal from "@/components/ui/Reveal"
import { cn } from "@/lib/utils"

export default function ChangelogSection() {
  return (
    <section id="changelog" className="py-24 border-b border-white/10 bg-[#060608]">
      <div className="mx-auto max-w-4xl px-4 sm:px-6">
        {/* Header */}
        <Reveal>
          <div className="mb-14">
            <Badge variant="blue" isComment className="mb-3">
              CONTINUOUS DELIVERY
            </Badge>
            <div className="flex flex-wrap items-baseline gap-4">
              <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl">
                Changelog
              </h2>
              <span className="font-mono text-2xl font-bold text-kai-blue glow-orange">
                v2.0.0
              </span>
            </div>
            <p className="mt-3 max-w-xl font-sans text-sm text-kai-muted">
              Recent engine advancements, performance optimizations, and provider integrations.
            </p>
          </div>
        </Reveal>

        {/* Changelog Entries */}
        <div className="space-y-10">
          {CHANGELOG.map((release) => (
            <Reveal key={release.version}>
              <div className="rounded-lg border border-white/10 bg-[#0a0b10] p-6">
                <div className="flex items-center justify-between border-b border-white/10 pb-4 mb-6">
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-base font-bold text-white">
                      {release.version}
                    </span>
                    <span className="rounded-full border border-kai-blue/30 bg-kai-blue/10 px-2.5 py-0.5 font-mono text-[10px] text-kai-blue">
                      {release.date}
                    </span>
                  </div>
                </div>

                <div className="space-y-4">
                  {release.highlights.map((h, i) => {
                    const tagStyle = {
                      FEAT: "border-kai-green/30 bg-kai-green/10 text-kai-green",
                      MODELS: "border-[#c084fc]/30 bg-[#c084fc]/10 text-[#d8b4fe]",
                      PERF: "border-kai-orange/30 bg-kai-orange/10 text-kai-orange",
                      FIX: "border-kai-amber/30 bg-kai-amber/10 text-kai-amber",
                    }[h.tag]

                    return (
                      <div key={i} className="flex flex-col sm:flex-row sm:items-baseline gap-2 sm:gap-4">
                        <span
                          className={cn(
                            "inline-block w-fit rounded border px-2 py-0.5 font-mono text-[10px] font-bold uppercase shrink-0",
                            tagStyle
                          )}
                        >
                          {h.tag}
                        </span>
                        <div>
                          <h4 className="font-sans text-sm font-semibold text-white">
                            {h.title}
                          </h4>
                          <p className="mt-0.5 font-sans text-xs text-kai-muted leading-relaxed">
                            {h.description}
                          </p>
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  )
}
