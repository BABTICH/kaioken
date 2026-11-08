import * as React from "react"

export default function GridBackground() {
  return (
    <div className="pointer-events-none fixed inset-0 z-0 overflow-hidden" aria-hidden>
      {/* 1. Subtle blueprint grid */}
      <div className="absolute inset-0 blueprint-grid opacity-35" />

      {/* 2. Top-center radial glow spotlight */}
      <div className="absolute -top-40 left-1/2 -translate-x-1/2 size-[650px] rounded-full bg-[radial-gradient(circle,rgba(255,135,0,0.12)_0%,transparent_70%)] blur-3xl" />

      {/* 3. Subtle bottom violet neural glow */}
      <div className="absolute bottom-0 right-1/4 size-[500px] rounded-full bg-[radial-gradient(circle,rgba(140,78,221,0.08)_0%,transparent_70%)] blur-3xl" />

      {/* 4. Edge darkening gradient */}
      <div className="absolute inset-0 bg-gradient-to-b from-transparent via-[#050507]/60 to-[#050507]" />
    </div>
  )
}
