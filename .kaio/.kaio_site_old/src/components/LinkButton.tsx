import * as React from "react"
import { Link } from "react-router-dom"
import { cn } from "@/lib/utils"

export interface LinkButtonProps {
  to?: string
  href?: string
  variant?: "default" | "outline" | "secondary" | "ghost" | "link" | "primary"
  size?: "default" | "sm" | "lg" | "icon"
  className?: string
  children: React.ReactNode
}

export default function LinkButton({
  to,
  href,
  variant = "default",
  size = "default",
  className,
  children,
}: LinkButtonProps) {
  const base =
    "inline-flex items-center justify-center font-mono text-xs font-medium transition-all duration-200 cursor-pointer disabled:pointer-events-none disabled:opacity-50 select-none active:scale-[0.98]"

  const variants = {
    default: "bg-kai-orange text-black font-semibold hover:bg-[#ffa033] shadow-[0_0_20px_rgba(255,135,0,0.2)]",
    primary: "bg-kai-orange text-black font-semibold hover:bg-[#ffa033] shadow-[0_0_20px_rgba(255,135,0,0.2)]",
    outline: "border border-white/15 bg-white/5 text-white hover:bg-white/10 hover:border-white/25",
    secondary: "bg-[#181920] border border-white/10 text-kai-text hover:text-white hover:bg-[#20222c]",
    ghost: "text-kai-muted hover:text-white hover:bg-white/5",
    link: "text-kai-orange underline-offset-4 hover:underline",
  }

  const sizes = {
    default: "h-9 px-4 py-2 rounded-md",
    sm: "h-8 px-3 rounded text-[11px]",
    lg: "h-11 px-6 rounded-md text-sm",
    icon: "size-9 rounded-md",
  }

  const classes = cn(
    base,
    variants[variant as keyof typeof variants] || variants.default,
    sizes[size as keyof typeof sizes] || sizes.default,
    className
  )

  if (to) {
    return (
      <Link to={to} className={classes}>
        {children}
      </Link>
    )
  }

  return (
    <a href={href} target="_blank" rel="noopener noreferrer" className={classes}>
      {children}
    </a>
  )
}
