import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import { ArrowRight, Terminal } from "lucide-react";
import AsciiArt from "@/components/AsciiArt";
import GithubMark from "@/components/GithubMark";
import Icon from "@/components/Icon";
import Accordion, { Points } from "@/mobile/components/Accordion";
import Code from "@/mobile/components/Code";
import FitText from "@/mobile/components/FitText";
import { Action, Card, Lead, Note, Section, SectionHead, StatGrid, Tag, } from "@/mobile/components/primitives";
import { tone, toneSurface, toneText } from "@/mobile/lib/tone";
import { BUILD_STEPS, COMPARISON, CURL_PROOF, DESKTOP_ART, DESKTOP_REPO_PATH, DESKTOP_STATS, DISTRIBUTION_NOTE, LAYERS, PLATFORMS, PRINCIPLES, SHORTCUT_GROUPS, SURFACES, } from "@/data/desktop";
import { cn } from "@/lib/utils";
export default function DesktopScreen() {
    return (_jsxs(_Fragment, { children: [_jsx(Hero, {}), _jsx(Surfaces, {}), _jsx(Architecture, {}), _jsx(Honestly, {}), _jsx(Principles, {}), _jsx(Shortcuts, {}), _jsx(Platforms, {}), _jsx(Closing, {})] }));
}
/* ── hero ─────────────────────────────────────────────────────────────────── */
function Hero() {
    return (_jsxs("section", { className: "px-4 pt-6 pb-9", children: [_jsxs("div", { className: "animate-rise", children: [_jsxs("span", { className: "inline-flex items-center gap-2 rounded-sm border border-kai-orange/30 bg-kai-orange/10 px-2.5 py-1 font-mono text-[10.5px] text-kai-amber", children: [_jsx("span", { className: "size-1.5 rounded-full bg-kai-green shadow-[0_0_6px_-1px_var(--kai-green)]", "aria-hidden": true }), "Tauri v2 \u00B7 same binary, same .kaioken/"] }), _jsx("h1", { className: "mt-5", children: _jsx(FitText, { children: _jsx(AsciiArt, { art: DESKTOP_ART, label: "Desktop app", className: "text-[9px] leading-[1.3]" }) }) }), _jsx("div", { className: "mt-2 h-px w-full max-w-[21rem] opacity-40", style: { background: "linear-gradient(90deg, var(--kai-red), var(--kai-orange))" }, "aria-hidden": true }), _jsxs(Lead, { className: "mt-4", children: ["The CLI in a window. Diffs you can read, a wiki you can browse, runs you can watch \u2014 all against the same ", _jsx("code", { className: "font-mono text-kai-amber", children: ".kaioken/" }), " folder the terminal writes."] })] }), _jsx(StatGrid, { className: "mt-7", items: DESKTOP_STATS.map((s) => ({ value: s.value, label: s.label })) }), _jsxs("div", { className: "mt-4 space-y-2.5", children: [_jsxs(Action, { href: DESKTOP_REPO_PATH, children: [_jsx(GithubMark, { className: "size-4" }), "View source"] }), _jsxs(Action, { to: "/docs/install", variant: "outline", children: [_jsx(Terminal, { className: "size-4", "aria-hidden": true }), "Get the CLI first", _jsx(ArrowRight, { className: "size-4", "aria-hidden": true })] })] })] }));
}
/* ── surfaces ─────────────────────────────────────────────────────────────── */
function Surfaces() {
    return (_jsxs(Section, { id: "surfaces", children: [_jsx(SectionHead, { index: "01", eyebrow: "surfaces", title: _jsxs(_Fragment, { children: ["Twelve screens,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "one rail" })] }), lead: "Tap one to read what it actually does." }), _jsx(Accordion, { className: "mt-6", defaultOpen: SURFACES[0].label, items: SURFACES.map((s) => {
                    const t = tone(s.tone);
                    return {
                        id: s.label,
                        title: s.label,
                        glyph: _jsx(Icon, { name: s.icon, className: cn("size-4", toneText[t]), "aria-hidden": true }),
                        meta: s.key ? _jsx(Tag, { children: s.key }) : null,
                        body: (_jsxs(_Fragment, { children: [_jsx("p", { className: cn("font-mono text-[12.5px] font-semibold", toneText[t]), children: s.headline }), _jsx("p", { className: "mt-2 font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: s.body }), _jsx("div", { className: "mt-3 border-t border-border/70 pt-3", children: _jsx(Points, { items: s.points, tone: toneText[t] }) })] })),
                    };
                }) })] }));
}
/* ── architecture ─────────────────────────────────────────────────────────── */
function Architecture() {
    return (_jsxs(Section, { id: "architecture", children: [_jsx(SectionHead, { index: "02", eyebrow: "architecture", title: _jsxs(_Fragment, { children: ["Four layers,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "no rewrite" })] }), lead: "Top is what you look at, bottom is what everything agrees on." }), _jsx("div", { className: "mt-6", children: LAYERS.map((layer, i) => {
                    const t = tone(layer.tone);
                    return (_jsxs("div", { children: [_jsxs("div", { className: cn("rounded-md border p-4", toneSurface[t]), children: [_jsxs("div", { className: "flex items-baseline gap-2.5", children: [_jsx("span", { className: cn("font-mono text-[22px] leading-none font-bold", toneText[t]), children: i + 1 }), _jsxs("div", { className: "min-w-0", children: [_jsx("h3", { className: "font-mono text-[13.5px] font-bold text-foreground", children: layer.title }), _jsx("p", { className: "font-mono text-[10px] text-kai-dim", children: layer.subtitle })] })] }), _jsx("p", { className: "mt-3 font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: layer.detail }), _jsx("div", { className: "mt-3 flex flex-wrap gap-1.5", children: layer.parts.map((p) => (_jsx("span", { className: cn("rounded-sm border px-1.5 py-px font-mono text-[9.5px]", toneSurface[t], toneText[t]), children: p }, p))) })] }), i < LAYERS.length - 1 ? (_jsx("p", { className: "py-1.5 text-center font-mono text-[12px] text-kai-dim", "aria-hidden": true, children: "\u2193" })) : null] }, layer.id));
                }) })] }));
}
/* ── the honest comparison ────────────────────────────────────────────────── */
function WinnerBadge({ winner }) {
    return (_jsx("span", { className: cn("inline-block rounded-sm border px-1.5 py-px font-mono text-[9.5px] whitespace-nowrap", winner === "desktop"
            ? "border-kai-orange/40 bg-kai-orange/10 text-kai-orange"
            : winner === "tui"
                ? "border-kai-green/40 bg-kai-green/10 text-kai-green"
                : "border-border text-kai-dim"), children: winner }));
}
function Honestly() {
    return (_jsxs(Section, { id: "comparison", children: [_jsx(SectionHead, { index: "03", eyebrow: "honestly", title: _jsxs(_Fragment, { children: ["GUI wins \u2014", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "and when it doesn't" })] }), lead: "Two rows go to the terminal, and they are the two that matter on a server." }), _jsx("div", { className: "mt-6 space-y-2.5", children: COMPARISON.map((row) => (_jsxs(Card, { className: "p-3.5", children: [_jsxs("div", { className: "flex items-start justify-between gap-3", children: [_jsx("h3", { className: "font-mono text-[12.5px] leading-snug font-bold text-foreground", children: row.job }), _jsx(WinnerBadge, { winner: row.winner })] }), _jsxs("dl", { className: "mt-2.5 space-y-1.5", children: [_jsxs("div", { className: "flex gap-2.5", children: [_jsx("dt", { className: "w-14 shrink-0 font-mono text-[9.5px] tracking-[0.12em] text-kai-dim uppercase", children: "tui" }), _jsx("dd", { className: cn("font-sans text-[12px] leading-[1.55]", row.winner === "tui" ? "text-foreground" : "text-muted-foreground"), children: row.tui })] }), _jsxs("div", { className: "flex gap-2.5", children: [_jsx("dt", { className: "w-14 shrink-0 font-mono text-[9.5px] tracking-[0.12em] text-kai-dim uppercase", children: "desktop" }), _jsx("dd", { className: cn("font-sans text-[12px] leading-[1.55]", row.winner === "desktop" ? "text-foreground" : "text-muted-foreground"), children: row.desktop })] })] })] }, row.job))) })] }));
}
/* ── principles ───────────────────────────────────────────────────────────── */
function Principles() {
    return (_jsxs(Section, { id: "principles", children: [_jsx(SectionHead, { index: "04", eyebrow: "principles", title: "Built this way" }), _jsx("div", { className: "mt-6 space-y-2.5", children: PRINCIPLES.map((p) => (_jsxs(Card, { className: "p-3.5", children: [_jsxs("div", { className: "flex items-center gap-2.5", children: [_jsx("span", { className: "flex size-7 shrink-0 items-center justify-center rounded-sm border border-kai-orange/25 bg-kai-orange/10", children: _jsx(Icon, { name: p.icon, className: "size-3.5 text-kai-orange", "aria-hidden": true }) }), _jsx("h3", { className: "font-mono text-[12.5px] font-bold text-foreground", children: p.title })] }), _jsx("p", { className: "mt-2.5 font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: p.body })] }, p.title))) })] }));
}
/* ── shortcuts ────────────────────────────────────────────────────────────── */
function Shortcuts() {
    return (_jsxs(Section, { id: "shortcuts", children: [_jsx(SectionHead, { index: "05", eyebrow: "shortcuts", title: "Keyboard-first", lead: "Worth knowing before you open it on a machine with a keyboard." }), _jsx(Accordion, { className: "mt-6", items: SHORTCUT_GROUPS.map((g) => ({
                    id: g.group,
                    title: g.group,
                    meta: _jsx(Tag, { children: g.items.length }),
                    body: (_jsx("div", { className: "divide-y divide-border/70 border-t border-border/70", children: g.items.map((item) => (_jsxs("div", { className: "flex items-center justify-between gap-3 py-2.5", children: [_jsx("span", { className: "font-sans text-[12.5px] text-muted-foreground", children: item.label }), _jsx("kbd", { className: "shrink-0 rounded-sm border border-border bg-background px-1.5 py-0.5 font-mono text-[10px] whitespace-nowrap text-kai-amber", children: item.keys })] }, item.keys))) })),
                })) })] }));
}
/* ── platforms and build ──────────────────────────────────────────────────── */
const PLATFORM_GLYPH = {
    windows: "▤",
    macos: "◍",
    linux: "◆",
};
function Platforms() {
    return (_jsxs(Section, { id: "platforms", children: [_jsx(SectionHead, { index: "06", eyebrow: "platforms", title: _jsxs(_Fragment, { children: [_jsx("span", { className: "text-kai-orange glow-orange", children: "Three platforms" }), ", three commands"] }) }), _jsx("div", { className: "mt-6 space-y-2.5", children: PLATFORMS.map((p) => (_jsxs(Card, { className: "p-3.5", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "font-mono text-[13px] text-kai-dim", "aria-hidden": true, children: PLATFORM_GLYPH[p.id] }), _jsx("h3", { className: "font-mono text-[13px] font-bold text-foreground", children: p.label })] }), _jsx("p", { className: "mt-1.5 font-mono text-[11px] text-kai-amber", children: p.artifacts }), _jsx("p", { className: "mt-1.5 font-sans text-[12px] leading-[1.55] text-muted-foreground", children: p.note })] }, p.id))) }), _jsx("div", { className: "mt-4", children: _jsx(Note, { tone: "amber", children: DISTRIBUTION_NOTE }) }), _jsxs("div", { className: "mt-6 space-y-4", children: [_jsxs("div", { children: [_jsx("p", { className: "mb-2 font-mono text-[10px] tracking-[0.15em] text-kai-dim uppercase", children: "build it yourself" }), _jsx(Code, { code: BUILD_STEPS, title: "build", prompt: true })] }), _jsxs("div", { children: [_jsx("p", { className: "mb-2 font-mono text-[10px] tracking-[0.15em] text-kai-dim uppercase", children: "proof it's just HTTP" }), _jsx(Code, { code: CURL_PROOF, title: "curl", prompt: true })] })] })] }));
}
/* ── closing ──────────────────────────────────────────────────────────────── */
function Closing() {
    return (_jsxs("section", { className: "relative overflow-hidden border-t border-border px-4 py-12 text-center", children: [_jsx("div", { className: "rule-sweep absolute inset-x-0 top-0 h-px", "aria-hidden": true }), _jsxs("h2", { className: "font-mono text-[20px] leading-[1.3] font-bold text-balance text-foreground", children: ["A surface worth ", _jsx("span", { className: "text-kai-orange glow-orange", children: "looking at" })] }), _jsx(Lead, { className: "mt-3", children: "The desktop app is a second window onto the same repository. Start with the binary \u2014 the window comes with it." }), _jsxs("div", { className: "mt-6 space-y-2.5", children: [_jsxs(Action, { to: "/docs/install", children: [_jsx(Terminal, { className: "size-4", "aria-hidden": true }), "Install the CLI", _jsx(ArrowRight, { className: "size-4", "aria-hidden": true })] }), _jsxs(Action, { href: DESKTOP_REPO_PATH, variant: "outline", children: [_jsx(GithubMark, { className: "size-4" }), "Source on GitHub"] })] })] }));
}
