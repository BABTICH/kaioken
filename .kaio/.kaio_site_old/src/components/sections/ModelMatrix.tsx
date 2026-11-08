import { Cpu, ShieldCheck, Sparkles, Terminal } from "lucide-react"
import { SUPPORTED_MODELS } from "@/data/models"
import Badge from "@/components/ui/Badge"
import NoiseBlock from "@/components/ui/NoiseBlock"
import Reveal from "@/components/ui/Reveal"

export default function ModelMatrix() {
  return (
    <section id="engines" className="py-24 border-b border-white/10 bg-[#050507]">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        {/* Header */}
        <Reveal>
          <div className="mb-14 text-center">
            <Badge variant="violet" isComment className="mb-3">
              ARCHITECTURAL FOUNDATIONS
            </Badge>
            <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl">
              Engineered for serious codebases.{" "}
              <span className="text-[#c084fc]">100% private execution.</span>
            </h2>
            <p className="mx-auto mt-3 max-w-xl font-sans text-sm text-kai-muted sm:text-base">
              Built from the ground up for software maintainers. Connect to any reasoning infrastructure or execute completely air-gapped on local hardware.
            </p>
          </div>
        </Reveal>

        {/* Bento Grid */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {SUPPORTED_MODELS.map((item, idx) => (
            <Reveal key={item.name} delay={idx * 0.05}>
              <div className="group relative flex flex-col justify-between overflow-hidden rounded-lg border border-white/10 bg-[#0b0c10] p-6 transition-all duration-300 hover:border-white/20 hover:bg-[#101117]">
                <div>
                  <div className="flex items-center justify-between">
                    <span className="font-mono text-xs font-bold text-white uppercase tracking-wider">
                      {item.name}
                    </span>
                    <span className="rounded-full border border-white/10 bg-white/5 px-2.5 py-0.5 font-mono text-[10px] text-kai-amber">
                      {item.badge}
                    </span>
                  </div>

                  <h3 className="mt-3 font-sans text-lg font-semibold text-white">
                    {item.model}
                  </h3>

                  <p className="mt-2 font-sans text-xs leading-relaxed text-kai-muted">
                    {item.description}
                  </p>
                </div>

                <div className="mt-6 flex items-center gap-2 pt-4 border-t border-white/5 font-mono text-[11px] text-kai-dim">
                  {item.type === "local" ? (
                    <>
                      <Cpu className="size-3.5 text-kai-green" />
                      <span>Air-gapped offline execution</span>
                    </>
                  ) : item.type === "reasoning" ? (
                    <>
                      <Sparkles className="size-3.5 text-[#c084fc]" />
                      <span>Autonomous self-healing loop</span>
                    </>
                  ) : (
                    <>
                      <ShieldCheck className="size-3.5 text-kai-orange" />
                      <span>Whole-repo architectural grounding</span>
                    </>
                  )}
                </div>
              </div>
            </Reveal>
          ))}
        </div>

        {/* Highlight callout block with CommandCode-style noise tile */}
        <Reveal delay={0.3} className="mt-8">
          <div className="grid gap-4 md:grid-cols-12 items-center rounded-lg border border-white/10 bg-[#090a0f] p-6">
            <div className="md:col-span-8">
              <div className="flex items-center gap-2 font-mono text-xs text-kai-orange">
                <Terminal className="size-4" />
                <span>UNIFIED EXECUTION CONTROL</span>
              </div>
              <h4 className="mt-2 font-sans text-lg font-semibold text-white">
                Seamlessly tune reasoning depth and permissions on the fly
              </h4>
              <p className="mt-1 font-sans text-xs text-kai-muted">
                Run <code className="text-kai-orange">/mode</code> to switch permission boundaries or set reasoning effort with <code className="text-kai-amber">/thinking high</code> instantly.
              </p>
            </div>

            <div className="h-24 md:col-span-4">
              <NoiseBlock accentColor="#8C4EDD" gradientDirection="center">
                <div className="flex h-full items-center justify-center p-4 text-center font-mono text-xs font-bold text-white tracking-widest uppercase">
                  <span>Autonomous Architectural Core</span>
                </div>
              </NoiseBlock>
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  )
}
