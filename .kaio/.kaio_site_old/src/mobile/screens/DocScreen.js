import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import * as React from "react";
import { Link, Navigate, useParams } from "react-router-dom";
import { ArrowLeft, ArrowRight } from "lucide-react";
import Icon from "@/components/Icon";
import { Eyebrow, Lead } from "@/mobile/components/primitives";
import { useScreenTitle } from "@/mobile/lib/chrome";
import { DOC_ORDER } from "@/data/docs-nav";
import { DocChromeContext } from "@/lib/doc-chrome";
/**
 * Documentation prose lives in one place — src/pages/docs — and both sites read
 * it from there. What differs is the frame: this screen supplies the header,
 * the back button (in the top bar) and prev/next, and switches off the chrome
 * the desktop layout brings, via DocChromeContext.
 */
const CONTENT = {
    install: React.lazy(() => import("@/pages/docs/Install")),
    tui: React.lazy(() => import("@/pages/docs/Tui")),
    commands: React.lazy(() => import("@/pages/docs/CommandsDoc")),
    agent: React.lazy(() => import("@/pages/docs/Agent")),
    research: React.lazy(() => import("@/pages/docs/Research")),
    impact: React.lazy(() => import("@/pages/docs/Impact")),
    wiki: React.lazy(() => import("@/pages/docs/Wiki")),
    cards: React.lazy(() => import("@/pages/docs/Cards")),
    skills: React.lazy(() => import("@/pages/docs/Skills")),
    config: React.lazy(() => import("@/pages/docs/Config")),
    integrations: React.lazy(() => import("@/pages/docs/Integrations")),
    update: React.lazy(() => import("@/pages/docs/Update")),
    output: React.lazy(() => import("@/pages/docs/OutputDoc")),
};
const BARE = { bare: true };
export default function DocScreen() {
    const { slug = "" } = useParams();
    const index = DOC_ORDER.findIndex((d) => d.to === `/docs/${slug}`);
    const doc = index >= 0 ? DOC_ORDER[index] : undefined;
    const Body = CONTENT[slug];
    useScreenTitle(doc?.label ?? null);
    if (!doc || !Body)
        return _jsx(Navigate, { to: "/docs", replace: true });
    const prev = index > 0 ? DOC_ORDER[index - 1] : undefined;
    const next = index < DOC_ORDER.length - 1 ? DOC_ORDER[index + 1] : undefined;
    return (_jsx(_Fragment, { children: _jsxs("article", { className: "px-4 pt-5 pb-4", children: [_jsxs("header", { className: "border-b border-border pb-5", children: [_jsxs(Eyebrow, { children: ["docs \u00B7 ", index + 1, "/", DOC_ORDER.length] }), _jsxs("h1", { className: "mt-3 flex items-start gap-2.5 font-mono text-[25px] leading-[1.2] font-bold tracking-tight text-foreground", children: [_jsx(Icon, { name: doc.icon, className: "mt-1 size-5 shrink-0 text-kai-orange", "aria-hidden": true }), doc.label] }), _jsx(Lead, { className: "mt-3", children: doc.blurb })] }), _jsx("div", { className: "pt-1", children: _jsx(DocChromeContext.Provider, { value: BARE, children: _jsx(React.Suspense, { fallback: _jsxs("p", { className: "pt-8 font-mono text-[12.5px] text-kai-dim", children: [_jsx("span", { className: "animate-caret text-kai-orange", children: "\u258E" }), " loading\u2026"] }), children: _jsx(Body, {}) }) }) }), _jsxs("nav", { className: "mt-10 space-y-2.5 border-t border-border pt-6", children: [prev ? (_jsxs(Link, { to: prev.to, className: "m-press flex min-h-[60px] items-center gap-3 rounded-md border border-border bg-card px-4 py-3", children: [_jsx(ArrowLeft, { className: "size-4 shrink-0 text-kai-dim", "aria-hidden": true }), _jsxs("span", { className: "min-w-0", children: [_jsx("span", { className: "block font-mono text-[10px] tracking-[0.15em] text-kai-dim uppercase", children: "previous" }), _jsx("span", { className: "mt-0.5 block font-mono text-[13.5px] font-bold text-foreground", children: prev.label })] })] })) : null, next ? (_jsxs(Link, { to: next.to, className: "m-press flex min-h-[60px] items-center gap-3 rounded-md border border-kai-orange/35 bg-kai-orange/[0.07] px-4 py-3", children: [_jsxs("span", { className: "min-w-0 flex-1", children: [_jsx("span", { className: "block font-mono text-[10px] tracking-[0.15em] text-kai-dim uppercase", children: "next" }), _jsx("span", { className: "mt-0.5 block font-mono text-[13.5px] font-bold text-kai-orange", children: next.label })] }), _jsx(ArrowRight, { className: "size-4 shrink-0 text-kai-orange", "aria-hidden": true })] })) : null, _jsx(Link, { to: "/docs", className: "m-press flex min-h-[48px] items-center justify-center rounded-md border border-border px-4 font-mono text-[12.5px] text-muted-foreground", children: "all documentation" })] })] }) }));
}
