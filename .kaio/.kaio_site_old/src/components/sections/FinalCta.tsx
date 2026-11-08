import { ArrowRight } from "lucide-react"
import { GithubMark } from "@/components/ui/BrandIcons"
import { DOCS_URL, GITHUB_URL } from "@/data/content"
import CopyCommandBox from "@/components/terminal/CopyCommandBox"
import Button from "@/components/ui/Button"
import NoiseBlock from "@/components/ui/NoiseBlock"
import Reveal from "@/components/ui/Reveal"

export default function FinalCta() {
  return (
    <section className="relative isolate overflow-hidden py-28 border-b border-white/10 bg-[#050507]">
      {/* Background blueprint grid */}
      <div className="absolute inset-0 blueprint-grid opacity-20 pointer-events-none" />

      {/* Decorative Noise Blocks in corners (Command Code signature style) */}
      <div className="hidden lg:block absolute -left-12 top-1/2 -translate-y-1/2 w-48 h-64 pointer-events-none opacity-40">
        <NoiseBlock accentColor="#8C4EDD" gradientDirection="right" />
      </div>
      <div className="hidden lg:block absolute -right-12 top-1/2 -translate-y-1/2 w-48 h-64 pointer-events-none opacity-40">
        <NoiseBlock accentColor="#ff8700" gradientDirection="left" />
      </div>

      <div className="mx-auto max-w-4xl px-4 sm:px-6 relative z-10 text-center">
        <Reveal>
          <h2 className="font-sans text-3xl font-bold tracking-tight text-white sm:text-5xl md:text-6xl leading-tight">
            Ship 10× faster.{" "}
            <span className="text-kai-orange glow-orange">Zero slop.</span>
          </h2>
          <p className="mx-auto mt-5 max-w-xl font-sans text-sm text-kai-muted sm:text-base leading-relaxed">
            Join engineers who stopped fixing broken AI code and started shipping verified software with whole-repo architectural intelligence.
          </p>
        </Reveal>

        <Reveal delay={0.15} className="mt-8">
          <CopyCommandBox />
        </Reveal>

        <Reveal delay={0.25} className="mt-8 flex flex-wrap items-center justify-center gap-3">
          <a href={DOCS_URL} target="_blank" rel="noopener noreferrer">
            <Button size="lg" variant="primary" pill>
              <span>Read Documentation</span>
              <ArrowRight className="size-4" />
            </Button>
          </a>
          <a href={GITHUB_URL} target="_blank" rel="noopener noreferrer">
            <Button size="lg" variant="outline" pill>
              <GithubMark className="size-4" />
              <span>Star on GitHub</span>
            </Button>
          </a>
        </Reveal>

        <div className="mt-6 font-mono text-[11px] text-kai-dim">
          <span>Free & Open Source under MIT · 0 telemetry or tracking</span>
        </div>
      </div>
    </section>
  )
}
