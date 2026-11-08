import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { Link } from "react-router-dom";
import { ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
/* ─────────────────────────────────────────────────────────────────────────────
   The phone site's building blocks.

   One gutter (16px), one section rhythm, one card. Everything on the phone is a
   single column, so the primitives here are about vertical rhythm and touch
   targets rather than grids — nothing below is narrower than 44px in the
   direction a thumb has to hit it.
   ───────────────────────────────────────────────────────────────────────────── */
export const GUTTER = "px-4";
/** A page-level band. Sections are separated by a rule, terminal style. */
export function Section({ id, children, className, first = false, }) {
    return (_jsx("section", { id: id, className: cn("m-scroll-offset py-9", GUTTER, !first && "border-t border-border", className), children: children }));
}
/** `▎ 03 · features` — the gutter mark the TUI prints before a heading. */
export function Eyebrow({ index, children, tone = "orange", }) {
    return (_jsxs("p", { className: "flex items-center gap-2 font-mono text-[10.5px] tracking-[0.28em] uppercase", children: [_jsx("span", { className: tone === "orange" ? "text-kai-orange" : "text-kai-dim", "aria-hidden": true, children: "\u258E" }), index ? _jsx("span", { className: "text-kai-amber", children: index }) : null, _jsx("span", { className: "text-kai-dim", children: children })] }));
}
/** Eyebrow + title + lead, the opening of nearly every section. */
export function SectionHead({ index, eyebrow, title, lead, className, }) {
    return (_jsxs("header", { className: className, children: [_jsx(Eyebrow, { index: index, children: eyebrow }), _jsx("h2", { className: "mt-3 font-mono text-[21px] leading-[1.25] font-bold tracking-tight text-balance text-foreground", children: title }), lead ? _jsx(Lead, { className: "mt-3", children: lead }) : null] }));
}
export function Lead({ children, className, }) {
    return (_jsx("p", { className: cn("font-sans text-[14.5px] leading-[1.65] text-muted-foreground", className), children: children }));
}
export function Card({ children, className, ...rest }) {
    return (_jsx("div", { className: cn("rounded-md border border-border bg-card/70 p-4 backdrop-blur-[2px]", className), ...rest, children: children }));
}
/** Small monospace label pill — tones, keys, tags. */
export function Tag({ children, className, }) {
    return (_jsx("span", { className: cn("inline-block rounded-sm border border-border px-1.5 py-px font-mono text-[10px] whitespace-nowrap text-kai-dim", className), children: children }));
}
/** A number and what it counts. Used in 2- and 4-up strips. */
export function StatGrid({ items, columns = 2, className, }) {
    return (_jsx("dl", { className: cn("grid gap-px overflow-hidden rounded-md border border-border bg-border", columns === 3 ? "grid-cols-3" : "grid-cols-2", className), children: items.map((s) => (_jsxs("div", { className: "bg-card px-3 py-3.5 text-center", children: [_jsx("dd", { className: "font-mono text-[22px] leading-none font-bold text-kai-orange", children: s.value }), _jsx("dt", { className: "mt-1.5 font-mono text-[10px] tracking-[0.14em] text-kai-dim uppercase", children: s.label })] }, s.label))) }));
}
/**
 * The workhorse of the phone site: a 56px-tall tappable row. Renders as a
 * router Link, an anchor or a button depending on where it goes, so the
 * accessibility tree always matches the behaviour.
 */
export function ListRow({ to, href, onClick, glyph, title, subtitle, meta, plain, className, }) {
    const body = (_jsxs(_Fragment, { children: [glyph ? _jsx("span", { className: "mt-0.5 shrink-0", children: glyph }) : null, _jsxs("span", { className: "min-w-0 flex-1", children: [_jsx("span", { className: "block font-mono text-[13.5px] leading-snug font-bold text-foreground", children: title }), subtitle ? (_jsx("span", { className: "mt-1 block font-sans text-[12.5px] leading-[1.5] text-muted-foreground", children: subtitle })) : null] }), meta ? _jsx("span", { className: "shrink-0 self-center", children: meta }) : null, !plain ? (_jsx(ChevronRight, { className: "mt-0.5 size-4 shrink-0 self-center text-kai-dim", "aria-hidden": true })) : null] }));
    const classes = cn("m-press flex min-h-[56px] w-full items-start gap-3 bg-card px-4 py-3.5 text-left", className);
    if (to) {
        return (_jsx(Link, { to: to, className: classes, children: body }));
    }
    if (href) {
        return (_jsx("a", { href: href, target: "_blank", rel: "noopener noreferrer", className: classes, children: body }));
    }
    return (_jsx("button", { type: "button", onClick: onClick, className: classes, children: body }));
}
/** Groups rows into one bordered block with hairlines between them. */
export function RowGroup({ children, className, }) {
    return (_jsx("div", { className: cn("divide-y divide-border overflow-hidden rounded-md border border-border", className), children: children }));
}
/* ── rails ────────────────────────────────────────────────────────────────── */
/**
 * A full-bleed horizontal rail of snapping cards. Breaks out of the page
 * gutter so the next card peeks in from the edge — the cue that says "swipe".
 */
export function Rail({ children, className, }) {
    return (_jsxs("div", { className: cn("m-rail -mx-4 scroll-px-4 px-4", className), children: [children, _jsx("span", { className: "w-px shrink-0", "aria-hidden": true })] }));
}
/** A callout: one accent rule, one line of type. */
export function Note({ tone = "amber", glyph = "!", children, }) {
    const ring = {
        amber: "border-kai-amber/30 bg-kai-amber/[0.06] text-kai-amber",
        blue: "border-kai-blue/30 bg-kai-blue/[0.06] text-kai-blue",
        green: "border-kai-green/30 bg-kai-green/[0.06] text-kai-green",
        orange: "border-kai-orange/30 bg-kai-orange/[0.06] text-kai-orange",
    }[tone];
    return (_jsxs("div", { className: cn("flex gap-2.5 rounded-md border p-3.5", ring), children: [_jsx("span", { className: "shrink-0 font-mono text-[12px] font-bold", "aria-hidden": true, children: glyph }), _jsx("p", { className: "font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: children })] }));
}
/** A full-width primary action. Phone buttons are 48px and stack. */
export function Action({ to, href, children, variant = "primary", className, }) {
    const classes = cn("m-press flex h-12 w-full items-center justify-center gap-2 rounded-md font-mono text-[13.5px] font-bold", variant === "primary"
        ? "bg-kai-orange text-[#180c00]"
        : "border border-border bg-card text-foreground", className);
    if (to) {
        return (_jsx(Link, { to: to, className: classes, children: children }));
    }
    return (_jsx("a", { href: href, target: "_blank", rel: "noopener noreferrer", className: classes, children: children }));
}
