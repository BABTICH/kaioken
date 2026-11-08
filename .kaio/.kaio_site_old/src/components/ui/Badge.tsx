import * as React from "react"
import { cn } from "@/lib/utils"

interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "orange" | "amber" | "green" | "violet" | "dim" | "blue" | "rose"
  isComment?: boolean
  children: React.ReactNode
}

export default function Badge({
  variant = "orange",
  isComment = false,
  className,
  children,
  ...props
}: BadgeProps) {
  const variantStyles = {
    orange: "border-kai-orange/30 bg-kai-orange/10 text-kai-orange",
    amber: "border-kai-amber/30 bg-kai-amber/10 text-kai-amber",
    green: "border-kai-green/30 bg-kai-green/10 text-kai-green",
    violet: "border-kai-violet/40 bg-kai-violet/10 text-[#d8b4fe]",
    blue: "border-kai-blue/30 bg-kai-blue/10 text-kai-blue",
    rose: "border-kai-rose/30 bg-kai-rose/10 text-kai-rose",
    dim: "border-white/10 bg-white/5 text-kai-muted",
  }[variant]

  if (isComment) {
    return (
      <div
        className={cn(
          "inline-flex items-center gap-1.5 font-mono text-[12px] font-medium tracking-wide uppercase",
          variant === "orange" && "text-kai-orange",
          variant === "amber" && "text-kai-amber",
          variant === "violet" && "text-[#c084fc]",
          variant === "green" && "text-kai-green",
          variant === "blue" && "text-kai-blue",
          variant === "rose" && "text-kai-rose",
          variant === "dim" && "text-kai-muted",
          className
        )}
        {...props}
      >
        <span className="opacity-60">//</span>
        <span>{children}</span>
      </div>
    )
  }

  return (
    <div
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full border px-3 py-0.5 font-mono text-[11px] font-medium transition-colors",
        variantStyles,
        className
      )}
      {...props}
    >
      {children}
    </div>
  )
}
