import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { Routes, Route, Navigate, useLocation } from "react-router-dom";
import Navbar from "@/components/layout/Navbar";
import PageBackground from "@/components/PageBackground";
import Footer from "@/components/layout/Footer";
import HomePage from "@/pages/HomePage";
import DesktopPage from "@/pages/Desktop";
import ShowcasePage from "@/pages/Showcase";
import NextPage from "@/pages/Next";
import DocsLayout from "@/pages/docs/DocsLayout";
import DocsIndex from "@/pages/docs/DocsIndex";
import Install from "@/pages/docs/Install";
import Tui from "@/pages/docs/Tui";
import CommandsDoc from "@/pages/docs/CommandsDoc";
import Agent from "@/pages/docs/Agent";
import Research from "@/pages/docs/Research";
import Impact from "@/pages/docs/Impact";
import Wiki from "@/pages/docs/Wiki";
import Cards from "@/pages/docs/Cards";
import Skills from "@/pages/docs/Skills";
import Update from "@/pages/docs/Update";
import Config from "@/pages/docs/Config";
import Integrations from "@/pages/docs/Integrations";
import OutputDoc from "@/pages/docs/OutputDoc";
const PreviewLayout = React.lazy(() => import("@/pages/preview/PreviewLayout"));
const PreviewIndex = React.lazy(() => import("@/pages/preview/PreviewIndex"));
const PreviewDoc = React.lazy(() => import("@/pages/preview/PreviewDoc"));
const OWN_BACKDROP = ["/", "/desktop"];
function RouteBackdrop() {
    const { pathname } = useLocation();
    if (OWN_BACKDROP.includes(pathname))
        return null;
    return _jsx(PageBackground, { variant: "simple" });
}
function ScrollToTop() {
    const { pathname } = useLocation();
    React.useEffect(() => {
        window.scrollTo(0, 0);
    }, [pathname]);
    return null;
}
function LoadingFallback() {
    return (_jsx("div", { className: "mx-auto max-w-6xl px-4 pt-32 sm:px-6", children: _jsxs("p", { className: "font-mono text-[13px] text-kai-dim", children: [_jsx("span", { className: "text-kai-orange", children: "\u258E" }), " loading\u2026"] }) }));
}
export default function DesktopApp() {
    return (_jsxs("div", { className: "relative min-h-screen text-white selection:bg-kai-orange/30 selection:text-white", children: [_jsx(ScrollToTop, {}), _jsx(RouteBackdrop, {}), _jsx(Navbar, {}), _jsx("main", { className: "relative z-10", children: _jsx(React.Suspense, { fallback: _jsx(LoadingFallback, {}), children: _jsxs(Routes, { children: [_jsx(Route, { path: "/", element: _jsx(HomePage, { isDesktop: true }) }), _jsx(Route, { path: "/desktop", element: _jsx(DesktopPage, {}) }), _jsx(Route, { path: "/showcase", element: _jsx(ShowcasePage, {}) }), _jsx(Route, { path: "/next", element: _jsx(NextPage, {}) }), _jsxs(Route, { path: "/docs", element: _jsx(DocsLayout, {}), children: [_jsx(Route, { index: true, element: _jsx(DocsIndex, {}) }), _jsx(Route, { path: "install", element: _jsx(Install, {}) }), _jsx(Route, { path: "tui", element: _jsx(Tui, {}) }), _jsx(Route, { path: "commands", element: _jsx(CommandsDoc, {}) }), _jsx(Route, { path: "agent", element: _jsx(Agent, {}) }), _jsx(Route, { path: "research", element: _jsx(Research, {}) }), _jsx(Route, { path: "impact", element: _jsx(Impact, {}) }), _jsx(Route, { path: "wiki", element: _jsx(Wiki, {}) }), _jsx(Route, { path: "cards", element: _jsx(Cards, {}) }), _jsx(Route, { path: "skills", element: _jsx(Skills, {}) }), _jsx(Route, { path: "update", element: _jsx(Update, {}) }), _jsx(Route, { path: "config", element: _jsx(Config, {}) }), _jsx(Route, { path: "integrations", element: _jsx(Integrations, {}) }), _jsx(Route, { path: "output", element: _jsx(OutputDoc, {}) }), _jsx(Route, { path: "*", element: _jsx(Navigate, { to: "/docs", replace: true }) })] }), _jsxs(Route, { path: "/preview", element: _jsx(PreviewLayout, {}), children: [_jsx(Route, { index: true, element: _jsx(PreviewIndex, {}) }), _jsx(Route, { path: ":section/:doc", element: _jsx(PreviewDoc, {}) })] }), _jsx(Route, { path: "*", element: _jsx(Navigate, { to: "/", replace: true }) })] }) }) }), _jsx(Footer, {})] }));
}
