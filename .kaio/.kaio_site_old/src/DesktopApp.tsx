import * as React from "react"
import { Routes, Route, Navigate, useLocation } from "react-router-dom"
import Navbar from "@/components/layout/Navbar"
import PageBackground from "@/components/PageBackground"
import Footer from "@/components/layout/Footer"
import HomePage from "@/pages/HomePage"
import DesktopPage from "@/pages/Desktop"
import ShowcasePage from "@/pages/Showcase"
import NextPage from "@/pages/Next"
import DocsLayout from "@/pages/docs/DocsLayout"
import DocsIndex from "@/pages/docs/DocsIndex"
import Install from "@/pages/docs/Install"
import Tui from "@/pages/docs/Tui"
import CommandsDoc from "@/pages/docs/CommandsDoc"
import Agent from "@/pages/docs/Agent"
import Research from "@/pages/docs/Research"
import Impact from "@/pages/docs/Impact"
import Wiki from "@/pages/docs/Wiki"
import Cards from "@/pages/docs/Cards"
import Skills from "@/pages/docs/Skills"
import Update from "@/pages/docs/Update"
import Config from "@/pages/docs/Config"
import Integrations from "@/pages/docs/Integrations"
import OutputDoc from "@/pages/docs/OutputDoc"

const PreviewLayout = React.lazy(() => import("@/pages/preview/PreviewLayout"))
const PreviewIndex = React.lazy(() => import("@/pages/preview/PreviewIndex"))
const PreviewDoc = React.lazy(() => import("@/pages/preview/PreviewDoc"))

const OWN_BACKDROP = ["/", "/desktop"]

function RouteBackdrop() {
  const { pathname } = useLocation()
  if (OWN_BACKDROP.includes(pathname)) return null
  return <PageBackground variant="simple" />
}

function ScrollToTop() {
  const { pathname } = useLocation()
  React.useEffect(() => {
    window.scrollTo(0, 0)
  }, [pathname])
  return null
}

function LoadingFallback() {
  return (
    <div className="mx-auto max-w-6xl px-4 pt-32 sm:px-6">
      <p className="font-mono text-[13px] text-kai-dim">
        <span className="text-kai-orange">▎</span> loading…
      </p>
    </div>
  )
}

export default function DesktopApp() {
  return (
    <div className="relative min-h-screen text-white selection:bg-kai-orange/30 selection:text-white">
      <ScrollToTop />

      {/* Ambient backdrop from original website */}
      <RouteBackdrop />

      {/* Sticky Pill Navbar */}
      <Navbar />

      {/* Main Routed Content */}
      <main className="relative z-10">
        <React.Suspense fallback={<LoadingFallback />}>
          <Routes>
            <Route path="/" element={<HomePage isDesktop={true} />} />
            <Route path="/desktop" element={<DesktopPage />} />
            <Route path="/showcase" element={<ShowcasePage />} />
            <Route path="/next" element={<NextPage />} />

            {/* Docs Subroutes */}
            <Route path="/docs" element={<DocsLayout />}>
              <Route index element={<DocsIndex />} />
              <Route path="install" element={<Install />} />
              <Route path="tui" element={<Tui />} />
              <Route path="commands" element={<CommandsDoc />} />
              <Route path="agent" element={<Agent />} />
              <Route path="research" element={<Research />} />
              <Route path="impact" element={<Impact />} />
              <Route path="wiki" element={<Wiki />} />
              <Route path="cards" element={<Cards />} />
              <Route path="skills" element={<Skills />} />
              <Route path="update" element={<Update />} />
              <Route path="config" element={<Config />} />
              <Route path="integrations" element={<Integrations />} />
              <Route path="output" element={<OutputDoc />} />
              <Route path="*" element={<Navigate to="/docs" replace />} />
            </Route>

            {/* Wiki & Output Interactive Previewer */}
            <Route path="/preview" element={<PreviewLayout />}>
              <Route index element={<PreviewIndex />} />
              <Route path=":section/:doc" element={<PreviewDoc />} />
            </Route>

            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </React.Suspense>
      </main>

      {/* Footer with ASCII art */}
      <Footer />
    </div>
  )
}
