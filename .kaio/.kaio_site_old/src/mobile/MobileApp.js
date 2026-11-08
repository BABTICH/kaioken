import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";
import PageBackground from "@/components/PageBackground";
import TopBar from "@/mobile/components/TopBar";
import TabBar from "@/mobile/components/TabBar";
import { ChromeProvider } from "@/mobile/lib/chrome";
import Home from "@/mobile/screens/Home";
/* Only Home is eager. Every other screen is a tap away, and a phone should not
   download the 71-document manifest or react-markdown to read the landing page. */
const DesktopScreen = React.lazy(() => import("@/mobile/screens/DesktopScreen"));
const DocsIndex = React.lazy(() => import("@/mobile/screens/DocsIndex"));
const DocScreen = React.lazy(() => import("@/mobile/screens/DocScreen"));
const Output = React.lazy(() => import("@/mobile/screens/Output"));
const OutputDoc = React.lazy(() => import("@/mobile/screens/OutputDoc"));
const Showcase = React.lazy(() => import("@/mobile/screens/Showcase"));
const Next = React.lazy(() => import("@/mobile/screens/Next"));
const More = React.lazy(() => import("@/mobile/screens/More"));
/** The same backdrop the desktop site paints, on the same two-tier rule: the
 *  shopfront screens get the full phosphor treatment — character grid, three
 *  drifting blooms, scanlines — and the reading surfaces get the quiet one. */
const FULL_BACKDROP = ["/", "/desktop"];
function RouteBackdrop() {
    const { pathname } = useLocation();
    return _jsx(PageBackground, { variant: FULL_BACKDROP.includes(pathname) ? "full" : "simple" });
}
/** A route change lands at the top — a phone has no scroll position worth
 *  keeping between two different screens. */
function ScrollToTop() {
    const { pathname } = useLocation();
    React.useEffect(() => {
        window.scrollTo({ top: 0, behavior: "instant" });
    }, [pathname]);
    return null;
}
function ScreenFallback() {
    return (_jsxs("p", { className: "px-4 pt-10 font-mono text-[12.5px] text-kai-dim", children: [_jsx("span", { className: "animate-caret text-kai-orange", children: "\u258E" }), " loading\u2026"] }));
}
/**
 * The phone site.
 *
 * A fixed bar top and bottom with one scrolling column between them — the shape
 * of an app, not of a page. Routes mirror the desktop site exactly, so every
 * link that has ever been shared still lands somewhere real.
 */
export default function MobileApp() {
    return (_jsx(ChromeProvider, { children: _jsxs("div", { className: "min-h-screen overflow-x-hidden text-foreground", children: [_jsx(ScrollToTop, {}), _jsx(RouteBackdrop, {}), _jsx(TopBar, {}), _jsx("main", { className: "m-top-inset m-bottom-inset", children: _jsx(React.Suspense, { fallback: _jsx(ScreenFallback, {}), children: _jsxs(Routes, { children: [_jsx(Route, { path: "/", element: _jsx(Home, {}) }), _jsx(Route, { path: "/desktop", element: _jsx(DesktopScreen, {}) }), _jsx(Route, { path: "/docs", element: _jsx(DocsIndex, {}) }), _jsx(Route, { path: "/docs/:slug", element: _jsx(DocScreen, {}) }), _jsx(Route, { path: "/preview", element: _jsx(Output, {}) }), _jsx(Route, { path: "/preview/:section/:doc", element: _jsx(OutputDoc, {}) }), _jsx(Route, { path: "/showcase", element: _jsx(Showcase, {}) }), _jsx(Route, { path: "/next", element: _jsx(Next, {}) }), _jsx(Route, { path: "/more", element: _jsx(More, {}) }), _jsx(Route, { path: "*", element: _jsx(Navigate, { to: "/", replace: true }) })] }) }) }), _jsx(TabBar, {})] }) }));
}
