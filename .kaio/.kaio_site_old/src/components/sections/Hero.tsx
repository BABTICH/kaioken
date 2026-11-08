import * as React from "react"
import Badge from "@/components/ui/Badge"
import CopyCommandBox from "@/components/terminal/CopyCommandBox"
import TerminalDemo from "@/components/terminal/TerminalDemo"
import Reveal from "@/components/ui/Reveal"

const FaultyTerminal = React.lazy(() => import("@/bits/FaultyTerminal"))

export default function Hero({ isDesktop = true }: { isDesktop?: boolean }) {
  return (
    <section className="relative isolate overflow-hidden pt-12 pb-20 md:pt-16 md:pb-28">
      {/* Background WebGL Shader on Desktop / Scanlines on Mobile */}
      {isDesktop ? (
        <div className="pointer-events-none absolute inset-0 -z-10 opacity-45">
          <React.Suspense fallback={null}>
            <FaultyTerminal
              scale={1.5}
              gridMul={[2, 1]}
              digitSize={1.3}
              timeScale={0.3}
              scanlineIntensity={0.5}
              glitchAmount={0.8}
              flickerAmount={0.5}
              noiseAmp={0.9}
              tint="#ff8700"
              curvature={0.06}
              mouseReact
              mouseStrength={0.3}
              dpr={1}
              brightness={0.75}
              fps={30}
              resolutionScale={0.5}
            />
          </React.Suspense>
        </div>
      ) : (
        <div
          aria-hidden
          className="crt-scanlines pointer-events-none absolute inset-0 -z-10 opacity-25"
        />
      )}

      {/* Gradient mask over the shader */}
      <div className="pointer-events-none absolute inset-0 -z-10 bg-gradient-to-b from-[#050507]/40 via-[#050507]/75 to-[#050507]" />

      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <div className="flex flex-col items-center text-center">
          {/* Top Kicker / Badge */}
          <Reveal>
            <Badge variant="orange" isComment className="mb-6">
              THE AUTONOMOUS ARCHITECTURAL CODING AGENT
            </Badge>
          </Reveal>

          {/* Main Headline */}
          <Reveal delay={0.1}>
            <h1 className="max-w-4xl font-sans text-4xl font-bold tracking-tight text-white sm:text-5xl md:text-6xl lg:text-[68px] leading-[1.08]">
              The coding agent that{" "}
              <span className="text-kai-orange glow-orange">understands how you build.</span>
            </h1>
          </Reveal>

          {/* Subheading */}
          <Reveal delay={0.2}>
            <p className="mx-auto mt-6 max-w-2xl font-sans text-base text-kai-muted sm:text-lg leading-relaxed">
              Kaioken indexes full repository architecture, delegates tasks into{" "}
              <span className="text-kai-text font-medium">isolated git worktrees</span>, and{" "}
              <span className="text-kai-text font-medium">auto-verifies compilation</span>{" "}
              before presenting a single diff.
            </p>
          </Reveal>

          {/* 1-Click Install Command Box */}
          <Reveal delay={0.3} className="mt-8 w-full">
            <CopyCommandBox />
          </Reveal>

          {/* Hero Terminal Demo Showcase */}
          <Reveal delay={0.4} className="mt-12 w-full max-w-4xl">
            <div className="rounded-lg border border-white/15 bg-[#08080c]/90 p-1.5 shadow-[0_0_50px_rgba(0,0,0,0.8)] glow-box-orange backdrop-blur-xl">
              <TerminalDemo />
            </div>
          </Reveal>
        </div>
      </div>
    </section>
  )
}
