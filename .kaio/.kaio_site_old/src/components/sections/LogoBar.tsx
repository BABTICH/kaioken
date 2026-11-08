import Reveal from "@/components/ui/Reveal"

const ECOSYSTEM_STACKS = [
  { name: "Golang", tag: "Go 1.22+" },
  { name: "Rust", tag: "Cargo / Tokio" },
  { name: "TypeScript", tag: "Node / Bun" },
  { name: "Python", tag: "PyO3 / Poetry" },
  { name: "PostgreSQL", tag: "SQL / Migrations" },
  { name: "Docker", tag: "Containers" },
  { name: "Kubernetes", tag: "Infra Manifests" },
]

export default function LogoBar() {
  return (
    <section className="border-y border-white/10 bg-[#08080a] py-8">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <Reveal>
          <div className="flex flex-col items-center justify-between gap-6 md:flex-row">
            {/* Left label */}
            <div className="flex items-center gap-2.5 font-mono text-xs text-kai-muted shrink-0">
              <span className="size-1.5 rounded-full bg-kai-orange animate-pulse" />
              <span>ARCHITECTED FOR SERIOUS CODEBASES</span>
            </div>

            {/* Right badges */}
            <div className="flex flex-wrap items-center justify-center gap-3 md:justify-end">
              {ECOSYSTEM_STACKS.map((tech) => (
                <div
                  key={tech.name}
                  className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3.5 py-1 font-mono text-xs text-kai-text transition-colors hover:border-white/20 hover:text-white"
                >
                  <span className="font-semibold text-white">{tech.name}</span>
                  <span className="text-[10px] text-kai-dim">•</span>
                  <span className="text-[10px] text-kai-muted">{tech.tag}</span>
                </div>
              ))}
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  )
}
