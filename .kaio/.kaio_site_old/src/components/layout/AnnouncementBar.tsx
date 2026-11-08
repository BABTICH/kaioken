import { ArrowRight, Sparkles } from "lucide-react"
import { GITHUB_URL } from "@/data/content"

export default function AnnouncementBar() {
  return (
    <div className="relative z-50 flex h-10 w-full items-center justify-between border-b border-white/10 bg-[#08080a] px-4 font-mono text-[11px] text-kai-muted sm:px-8">
      {/* Left: Release Announcement */}
      <a
        href={GITHUB_URL}
        target="_blank"
        rel="noopener noreferrer"
        className="group flex items-center gap-2 text-kai-text transition-colors hover:text-white"
      >
        <span className="flex size-4 items-center justify-center rounded-full bg-kai-orange/20 text-kai-orange">
          <Sparkles className="size-2.5" />
        </span>
        <span className="font-semibold text-white">Kaioken v2 is here:</span>
        <span className="hidden text-kai-muted group-hover:underline sm:inline">
          Multi-agent git worktree sandboxes & autonomous Go verification gates
        </span>
        <ArrowRight className="size-3 transition-transform group-hover:translate-x-0.5 text-kai-orange" />
      </a>

      {/* Right: Architectural Capabilities */}
      <div className="hidden items-center gap-3 text-[11px] lg:flex">
        <span className="font-semibold uppercase tracking-wider text-kai-amber">
          Architectural Core
        </span>
        <span className="text-kai-text">Worktree Sandboxes</span>
        <span className="text-kai-dim">•</span>
        <span className="text-kai-text">Verification Gates</span>
        <span className="text-kai-dim">•</span>
        <span className="text-kai-text">AST Symbol Graph</span>
        <span className="text-kai-dim">•</span>
        <span className="text-kai-text">Air-Gapped Offline</span>
      </div>
    </div>
  )
}
