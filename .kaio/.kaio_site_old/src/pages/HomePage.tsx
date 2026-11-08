import { Monitor, ArrowRight, Sparkles } from "lucide-react"
import Hero from "@/components/sections/Hero"
import LogoBar from "@/components/sections/LogoBar"
import StatsBar from "@/components/sections/StatsBar"
import FeatureListPreview from "@/components/sections/FeatureListPreview"
import ProblemSolution from "@/components/sections/ProblemSolution"
import ModelMatrix from "@/components/sections/ModelMatrix"
import ModesSection from "@/components/sections/ModesSection"
import ArchitectureFlow from "@/components/sections/ArchitectureFlow"
import Multiplier from "@/components/sections/Multiplier"
import OutputTree from "@/components/sections/OutputTree"
import AppWindow from "@/components/desktop/AppWindow"
import LinkButton from "@/components/LinkButton"
import SectionHeading from "@/components/SectionHeading"
import Reveal from "@/components/Reveal"
import PageBackground from "@/components/PageBackground"
import FaqSection from "@/components/sections/FaqSection"
import FinalCta from "@/components/sections/FinalCta"

export default function HomePage({ isDesktop = true }: { isDesktop?: boolean }) {
  return (
    <>
      <PageBackground variant="full" />
      <Hero isDesktop={isDesktop} />
      <LogoBar />
      <StatsBar />
      <FeatureListPreview />
      <ProblemSolution />
      <ModelMatrix />
      <ModesSection />
      <ArchitectureFlow />
      <Multiplier />
      <OutputTree />

      {/* Desktop App Live Teaser with Interactive AppWindow */}
      <section className="relative border-t border-white/10 py-20 sm:py-24 bg-[#08080c]/60 backdrop-blur-md">
        <div className="mx-auto max-w-6xl px-4 sm:px-6">
          <Reveal>
            <SectionHeading
              index="08"
              eyebrow="desktop studio"
              title={
                <>
                  Same engine,{" "}
                  <span className="text-kai-orange glow-orange">native desktop surface</span>
                </>
              }
              description="Everything the TUI does — chat, research, wiki, knowledge cards — in a multi-pane window with live diff approvals and concurrent task execution."
              align="center"
            />
          </Reveal>

          <div className="mx-auto mt-10 max-w-4xl">
            <div className="rounded-lg border border-white/10 bg-[#050507]/90 p-px shadow-2xl">
              <AppWindow size="sm" start="chat" />
            </div>
          </div>

          <div className="mt-8 flex flex-wrap justify-center gap-3">
            <LinkButton to="/desktop" size="lg" className="rounded-md">
              <Monitor className="size-4" data-icon="inline-start" />
              Explore Desktop Studio
              <ArrowRight data-icon="inline-end" />
            </LinkButton>
          </div>
        </div>
      </section>

      <FaqSection />
      <FinalCta />
    </>
  )
}
