import { Link } from "react-router-dom"
import { MessageSquare } from "lucide-react"
import { GithubMark, XMark } from "@/components/ui/BrandIcons"
import {
  ASCII_LOGO,
  BUILDER_NAME,
  DISCORD_URL,
  GITHUB_URL,
  VERSION,
  X_URL,
} from "@/data/content"

export default function Footer() {
  return (
    <footer className="relative z-10 border-t border-white/10 bg-[#050507] pt-16 pb-12 font-mono text-xs text-kai-muted">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        {/* Giant ASCII Logo */}
        <div className="mb-12 overflow-x-auto text-center">
          <pre className="inline-block text-[9px] sm:text-[11px] md:text-[12px] font-bold leading-tight text-kai-orange select-none opacity-80">
            {ASCII_LOGO}
          </pre>
        </div>

        {/* 4-column link grid */}
        <div className="grid grid-cols-2 gap-8 sm:grid-cols-4 pb-12 border-b border-white/10">
          <div>
            <span className="font-bold text-white uppercase tracking-wider">Product</span>
            <ul className="mt-4 space-y-2.5">
              <li><Link to="/desktop" className="hover:text-white transition-colors">Desktop Studio</Link></li>
              <li><Link to="/showcase" className="hover:text-white transition-colors">CLI Showcase</Link></li>
              <li><Link to="/next" className="hover:text-white transition-colors">Roadmap & Next</Link></li>
              <li><a href="/#features" className="hover:text-white transition-colors">Git Worktrees</a></li>
              <li><a href="/#anti-slop" className="hover:text-white transition-colors">Anti-Slop Engine</a></li>
            </ul>
          </div>

          <div>
            <span className="font-bold text-white uppercase tracking-wider">Resources</span>
            <ul className="mt-4 space-y-2.5">
              <li><Link to="/docs" className="hover:text-white transition-colors">Documentation</Link></li>
              <li><Link to="/docs/install" className="hover:text-white transition-colors">Installation</Link></li>
              <li><Link to="/docs/commands" className="hover:text-white transition-colors">CLI Reference</Link></li>
              <li><Link to="/docs/agent" className="hover:text-white transition-colors">Agent Modes</Link></li>
              <li><a href="/#faq" className="hover:text-white transition-colors">FAQ</a></li>
            </ul>
          </div>

          <div>
            <span className="font-bold text-white uppercase tracking-wider">Engines</span>
            <ul className="mt-4 space-y-2.5">
              <li><a href="/#engines" className="hover:text-white transition-colors">Worktree Sandboxes</a></li>
              <li><a href="/#engines" className="hover:text-white transition-colors">Verification Gates</a></li>
              <li><a href="/#engines" className="hover:text-white transition-colors">AST Call Graph</a></li>
              <li><a href="/#engines" className="hover:text-white transition-colors">Deep Wiki Engine</a></li>
              <li><a href="/#engines" className="hover:text-white transition-colors">Air-Gapped Offline</a></li>
            </ul>
          </div>

          <div>
            <span className="font-bold text-white uppercase tracking-wider">Community</span>
            <ul className="mt-4 space-y-2.5">
              <li><a href={GITHUB_URL} target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors">GitHub Repo</a></li>
              <li><a href={DISCORD_URL} target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors">Discord Server</a></li>
              <li><a href={X_URL} target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors">Twitter / X</a></li>
              <li><Link to="/next" className="hover:text-white transition-colors">Roadmap ({VERSION})</Link></li>
            </ul>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="mt-8 flex flex-col items-center justify-between gap-4 sm:flex-row text-[11px] text-kai-dim">
          <div>
            <span>Built with precision by </span>
            <span className="font-bold text-white">{BUILDER_NAME}</span>
            <span> · Open Source MIT</span>
          </div>

          <div className="flex items-center gap-4 text-kai-muted">
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noopener noreferrer"
              aria-label="GitHub"
              className="hover:text-white transition-colors"
            >
              <GithubMark className="size-4" />
            </a>
            <a
              href={DISCORD_URL}
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Discord"
              className="hover:text-white transition-colors"
            >
              <MessageSquare className="size-4" />
            </a>
            <a
              href={X_URL}
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Twitter / X"
              className="hover:text-white transition-colors"
            >
              <XMark className="size-4" />
            </a>
          </div>
        </div>
      </div>
    </footer>
  )
}
