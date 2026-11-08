import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Link } from "react-router-dom";
import { AlertTriangle, ArrowLeft, ArrowRight, Info, Lightbulb } from "lucide-react";
import { DOC_ORDER } from "@/data/docs-nav";
import { useDocChrome } from "@/lib/doc-chrome";
import { cn } from "@/lib/utils";
/* Small typographic primitives so each doc page stays readable as JSX. */
export function H2({ id, children }) {
    return (_jsxs("h2", { id: id, className: "scroll-mt-24 pt-10 font-mono text-xl font-bold text-foreground first:pt-0", children: [_jsx("span", { className: "mr-2 text-kai-orange", "aria-hidden": true, children: "##" }), children] }));
}
export function H3({ children }) {
    return (_jsx("h3", { className: "pt-6 font-mono text-[15px] font-bold text-kai-amber", children: children }));
}
export function P({ children }) {
    return (_jsx("p", { className: "pt-4 font-sans text-[15px] leading-[1.75] text-muted-foreground", children: children }));
}
export function UL({ children }) {
    return _jsx("ul", { className: "space-y-2 pt-4", children: children });
}
export function LI({ children }) {
    return (_jsxs("li", { className: "flex gap-2.5 font-sans text-[15px] leading-[1.7] text-muted-foreground", children: [_jsx("span", { className: "mt-[7px] size-1 shrink-0 rounded-full bg-kai-orange", "aria-hidden": true }), _jsx("span", { className: "min-w-0", children: children })] }));
}
/** Numbered steps, rendered with a terminal gutter instead of list markers. */
export function Steps({ items }) {
    return (_jsx("ol", { className: "space-y-3 pt-4", children: items.map((item, i) => (_jsxs("li", { className: "flex gap-3", children: [_jsx("span", { className: "mt-px shrink-0 font-mono text-[12px] font-bold text-kai-orange", children: String(i + 1).padStart(2, "0") }), _jsx("span", { className: "min-w-0 font-sans text-[15px] leading-[1.7] text-muted-foreground", children: item })] }, i))) }));
}
export function C({ children }) {
    return (_jsx("code", { className: "rounded-sm bg-kai-panel px-1.5 py-0.5 font-mono text-[13px] text-kai-amber", children: children }));
}
const CALLOUT = {
    note: { icon: Info, ring: "border-kai-blue/35 bg-kai-blue/[0.06]", text: "text-kai-blue" },
    tip: { icon: Lightbulb, ring: "border-kai-green/35 bg-kai-green/[0.06]", text: "text-kai-green" },
    warn: {
        icon: AlertTriangle,
        ring: "border-kai-amber/40 bg-kai-amber/[0.06]",
        text: "text-kai-amber",
    },
};
export function Callout({ kind = "note", title, children, }) {
    const style = CALLOUT[kind];
    const Glyph = style.icon;
    return (_jsxs("div", { className: cn("mt-6 rounded-sm border p-4", style.ring), children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx(Glyph, { className: cn("size-3.5", style.text) }), _jsx("span", { className: cn("font-mono text-[12.5px] font-bold", style.text), children: title })] }), _jsx("div", { className: "mt-2 font-sans text-[14px] leading-[1.7] text-muted-foreground", children: children })] }));
}
/** Wraps a doc page: title block, body, and prev/next derived from DOC_ORDER. */
export function DocPage({ title, lead, children, }) {
    const { bare } = useDocChrome();
    const index = DOC_ORDER.findIndex((d) => d.label === title);
    const prev = index > 0 ? DOC_ORDER[index - 1] : undefined;
    const next = index >= 0 && index < DOC_ORDER.length - 1 ? DOC_ORDER[index + 1] : undefined;
    // The phone site frames the document itself — title, back button and
    // prev/next all live in its screen, so here only the prose is wanted.
    if (bare)
        return _jsx("div", { className: "min-w-0", children: children });
    return (_jsxs("article", { className: "min-w-0 pb-16", children: [_jsxs("header", { className: "border-b border-border pb-6", children: [_jsxs("p", { className: "font-mono text-[11px] tracking-[0.25em] text-kai-dim uppercase", children: [_jsx("span", { className: "text-kai-orange", children: "\u258E" }), " docs"] }), _jsx("h1", { className: "mt-3 font-mono text-3xl font-bold tracking-tight text-foreground", children: title }), _jsx("p", { className: "mt-3 max-w-2xl font-sans text-[15px] leading-relaxed text-muted-foreground", children: lead })] }), _jsx("div", { className: "pt-2", children: children }), (prev || next) && (_jsxs("nav", { className: "mt-14 grid gap-3 border-t border-border pt-6 sm:grid-cols-2", children: [prev ? (_jsxs(Link, { to: prev.to, className: "group rounded-sm border border-border bg-card p-4 transition-colors hover:border-kai-orange/45", children: [_jsxs("span", { className: "flex items-center gap-1.5 font-mono text-[11px] text-kai-dim", children: [_jsx(ArrowLeft, { className: "size-3" }), "previous"] }), _jsx("span", { className: "mt-1 block font-mono text-[14px] font-bold text-foreground transition-colors group-hover:text-kai-orange", children: prev.label })] })) : (_jsx("span", {})), next ? (_jsxs(Link, { to: next.to, className: "group rounded-sm border border-border bg-card p-4 text-right transition-colors hover:border-kai-orange/45 sm:col-start-2", children: [_jsxs("span", { className: "flex items-center justify-end gap-1.5 font-mono text-[11px] text-kai-dim", children: ["next", _jsx(ArrowRight, { className: "size-3" })] }), _jsx("span", { className: "mt-1 block font-mono text-[14px] font-bold text-foreground transition-colors group-hover:text-kai-orange", children: next.label })] })) : null] }))] }));
}
