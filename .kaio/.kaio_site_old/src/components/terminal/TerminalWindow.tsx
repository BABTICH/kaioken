import * as React from "react"
import { cn } from "@/lib/utils"

export interface TerminalWindowProps {
  title?: string
  status?: string
  className?: string
  bodyClassName?: string
  meta?: React.ReactNode
  children: React.ReactNode
  accentColor?: string
}

export default function TerminalWindow({
  title = "kaioken — tui",
  status = "ready",
  className,
  bodyClassName,
  meta,
  children,
  accentColor = "#ff8700",
}: TerminalWindowProps) {
  return (
    <div
      className={cn(
        "relative flex flex-col overflow-hidden rounded-md border border-white/10 bg-[#08080a] shadow-2xl transition-all duration-300",
        className
      )}
    >
      {/* Title bar */}
      <div className="flex h-9 items-center justify-between border-b border-white/10 bg-[#0f1015] px-3.5 select-none">
        {/* Window controls */}
        <div className="flex items-center gap-1.5">
          <span className="size-2.5 rounded-full bg-[#ff5f57] opacity-80" />
          <span className="size-2.5 rounded-full bg-[#febc2e] opacity-80" />
          <span className="size-2.5 rounded-full bg-[#28c840] opacity-80" />
          <span className="ml-2 font-mono text-[11px] text-kai-muted">{title}</span>
        </div>

        {/* Right controls / meta / status */}
        <div className="flex items-center gap-3">
          {meta && <div className="font-mono text-[11px] text-kai-dim">{meta}</div>}
          {status && (
            <div className="flex items-center gap-1.5 font-mono text-[10px] text-kai-muted">
              <span
                className="size-1.5 rounded-full animate-pulse"
                style={{ backgroundColor: accentColor }}
              />
              <span className="uppercase tracking-wider">{status}</span>
            </div>
          )}
        </div>
      </div>

      {/* Terminal content */}
      <div className={cn("relative flex-1 p-4 font-mono text-[13px] leading-relaxed text-kai-text", bodyClassName)}>
        {children}
      </div>
    </div>
  )
}
