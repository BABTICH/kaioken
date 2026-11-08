import * as React from "react"
import { useLayout } from "@/lib/viewport"
import TerminalBootLoader from "@/components/terminal/TerminalBootLoader"

const DesktopApp = React.lazy(() => import("@/DesktopApp"))
const MobileApp = React.lazy(() => import("@/mobile/MobileApp"))

function BootFallback() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-[#050507]">
      <p className="font-mono text-sm text-kai-muted">
        <span className="inline-block size-2 bg-kai-orange animate-pulse mr-2" />
        kaioken v2
      </p>
    </div>
  )
}

export default function App() {
  const layout = useLayout()
  const [booted, setBooted] = React.useState(false)

  return (
    <>
      {!booted && (
        <TerminalBootLoader onComplete={() => setBooted(true)} />
      )}
      <React.Suspense fallback={<BootFallback />}>
        {layout === "mobile" ? <MobileApp /> : <DesktopApp />}
      </React.Suspense>
    </>
  )
}
