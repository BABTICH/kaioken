import * as React from "react"
import { cn } from "@/lib/utils"

interface NoiseBlockProps extends React.HTMLAttributes<HTMLDivElement> {
  accentColor?: string
  gradientDirection?: "left" | "right" | "center"
  className?: string
  children?: React.ReactNode
}

/**
 * Recreates the signature CommandCode.ai 5-layer composite grainy block:
 * 1. Base accent background (#8C4EDD violet or #ff8700 orange)
 * 2. Darkness overlay (40% mix-blend-overlay)
 * 3. Contrast layer (15% mix-blend-overlay)
 * 4. Micro-grain noise texture
 * 5. Directional gradient mask to edge
 */
export default function NoiseBlock({
  accentColor = "#8C4EDD",
  gradientDirection = "right",
  className,
  children,
  ...props
}: NoiseBlockProps) {
  const gradientClass =
    gradientDirection === "left"
      ? "bg-gradient-to-l from-black/90 via-transparent to-black/90"
      : gradientDirection === "center"
      ? "bg-gradient-to-r from-black/85 via-transparent to-black/85"
      : "bg-gradient-to-r from-black/90 via-transparent to-transparent"

  return (
    <div
      className={cn(
        "relative w-full h-full min-h-[48px] overflow-hidden rounded-sm border border-white/10",
        className
      )}
      {...props}
    >
      {/* 1. Solid tint */}
      <div className="absolute inset-0" style={{ backgroundColor: accentColor }} />
      {/* 2. Darkness blend */}
      <div className="absolute inset-0 bg-black/40 mix-blend-overlay pointer-events-none" />
      {/* 3. Midtone contrast blend */}
      <div className="absolute inset-0 bg-black/20 mix-blend-overlay pointer-events-none" />
      {/* 4. Tactile micro-grain overlay */}
      <div className="absolute inset-0 opacity-40 mix-blend-overlay pointer-events-none block-grain" />
      {/* 5. Directional gradient fade */}
      <div className={cn("absolute inset-0 pointer-events-none", gradientClass)} />
      {/* Optional Content */}
      {children && <div className="relative z-10 w-full h-full">{children}</div>}
    </div>
  )
}
