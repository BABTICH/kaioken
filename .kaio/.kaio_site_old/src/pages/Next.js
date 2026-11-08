import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import Icon from "@/components/Icon";
import SectionHeading from "@/components/SectionHeading";
import { ROADMAP_CATEGORIES, ARCH_ENABLERS } from "@/data/roadmap";
import { cn } from "@/lib/utils";
/* ── tone styles ─────────────────────────────────────────────────────────── */
const TONE = {
    orange: {
        icon: "text-kai-orange",
        rule: "bg-kai-orange",
        badge: "border-kai-orange/40 text-kai-orange",
        hover: "hover:border-kai-orange/45",
    },
    amber: {
        icon: "text-kai-amber",
        rule: "bg-kai-amber",
        badge: "border-kai-amber/40 text-kai-amber",
        hover: "hover:border-kai-amber/45",
    },
    blue: {
        icon: "text-kai-blue",
        rule: "bg-kai-blue",
        badge: "border-kai-blue/40 text-kai-blue",
        hover: "hover:border-kai-blue/40",
    },
    green: {
        icon: "text-kai-green",
        rule: "bg-kai-green",
        badge: "border-kai-green/40 text-kai-green",
        hover: "hover:border-kai-green/40",
    },
};
/* ── category section ────────────────────────────────────────────────────── */
function CategorySection({ cat }) {
    const tone = TONE[cat.tone];
    return (_jsx("section", { className: "border-t border-border py-16 sm:py-20", children: _jsxs("div", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: [_jsxs("div", { className: "flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between", children: [_jsxs("div", { className: "flex items-start gap-4", children: [_jsx("span", { className: "mt-0.5 font-mono text-[11px] tracking-[0.25em] text-kai-dim", children: cat.index }), _jsxs("div", { children: [_jsxs("div", { className: "flex flex-wrap items-center gap-2.5", children: [_jsx(Icon, { name: cat.icon, className: cn("size-4.5", tone.icon) }), _jsx("h3", { className: "font-mono text-lg font-bold tracking-tight text-foreground sm:text-xl", children: cat.title }), cat.status && (_jsx("span", { className: cn("rounded-sm border px-2 py-0.5 font-mono text-[10px] tracking-wider uppercase", tone.badge), children: cat.status }))] }), _jsx("p", { className: "mt-2 max-w-2xl font-sans text-[13.5px] leading-relaxed text-muted-foreground", children: cat.blurb })] })] }), _jsxs("span", { className: cn("hidden shrink-0 rounded-sm border px-2 py-0.5 font-mono text-[10px] tracking-wider uppercase sm:block", tone.badge), children: [cat.items.length, " features"] })] }), _jsx("div", { className: "mt-8 grid gap-px overflow-hidden rounded-sm border border-border bg-border sm:grid-cols-2 lg:grid-cols-3", children: cat.items.map((item) => (_jsxs("article", { className: "group relative glass p-5 transition-all duration-300 hover:brightness-110", children: [_jsx("span", { className: cn("absolute inset-y-0 left-0 w-[2px] origin-top scale-y-0 transition-transform duration-300 group-hover:scale-y-100", tone.rule), "aria-hidden": true }), _jsxs("div", { className: "flex items-start justify-between gap-2", children: [_jsx("h4", { className: "font-mono text-[13.5px] font-bold text-foreground", children: item.title }), item.done && (_jsx("span", { className: "flex size-4.5 shrink-0 items-center justify-center rounded-full bg-kai-green/15 text-kai-green", title: "Already shipped", children: _jsx(Icon, { name: "Check", className: "size-3", strokeWidth: 3 }) }))] }), _jsx("p", { className: "mt-2 font-sans text-[13px] leading-relaxed text-muted-foreground", children: item.body })] }, item.title))) })] }) }));
}
/* ── page ────────────────────────────────────────────────────────────────── */
const TOTAL_FEATURES = ROADMAP_CATEGORIES.reduce((n, c) => n + c.items.length, 0);
const SUMMARY_STATS = [
    { value: String(ROADMAP_CATEGORIES.length), label: "categories" },
    { value: String(TOTAL_FEATURES), label: "features" },
    { value: String(ARCH_ENABLERS.length), label: "enablers" },
    { value: "1", label: "binary" },
];
export default function Next() {
    return (_jsxs("div", { className: "pt-24", children: [_jsxs("section", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: [_jsx(SectionHeading, { eyebrow: "next", title: _jsxs(_Fragment, { children: ["What comes ", _jsx("span", { className: "text-kai-orange glow-orange", children: "next" })] }), description: "A next-generation interface for Kaioken \u2014 advanced agents, richer search, a GUI shell, deeper integrations, and extended language support. Every item builds on the existing architecture: the agent loop, tool framework, codemap index, wiki engine, and serve layer." }), _jsx("dl", { className: "mt-10 grid grid-cols-2 gap-px overflow-hidden rounded-sm border border-border bg-border sm:grid-cols-4", children: SUMMARY_STATS.map((s) => (_jsxs("div", { className: "glass px-5 py-5", children: [_jsx("dt", { className: "font-mono text-[10.5px] tracking-[0.2em] text-kai-dim uppercase", children: s.label }), _jsx("dd", { className: "mt-1 font-mono text-2xl font-bold text-kai-orange", children: s.value })] }, s.label))) }), _jsx("div", { className: "mt-6 rounded-sm p-5 glass", children: _jsxs("p", { className: "font-mono text-[12.5px] leading-relaxed text-kai-dim", children: [_jsx("span", { className: "text-kai-orange", children: "\u258E" }), " These enhancements extend the current architecture \u2014 the", " ", _jsx("span", { className: "text-foreground", children: "Agent.Run()" }), " tool loop,", " ", _jsx("span", { className: "text-foreground", children: "codemap.Index" }), " symbol extraction,", " ", _jsx("span", { className: "text-foreground", children: "wiki.Run()" }), " multi-pass pipeline,", " ", _jsx("span", { className: "text-foreground", children: "serve.Server" }), " HTTP layer, and the", " ", _jsx("span", { className: "text-foreground", children: "skills" }), " system \u2014 rather than replacing it."] }) })] }), _jsx("div", { className: "mt-16", children: ROADMAP_CATEGORIES.map((cat) => (_jsx(CategorySection, { cat: cat }, cat.id))) }), _jsx("section", { className: "border-t border-border py-16 sm:py-20", children: _jsxs("div", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: [_jsx(SectionHeading, { eyebrow: "foundation", title: _jsxs(_Fragment, { children: ["Architectural ", _jsx("span", { className: "text-kai-amber", children: "enablers" })] }), description: "Cross-cutting changes that underpin multiple categories above. Each one extends an existing structure rather than introducing a new system." }), _jsx("ul", { className: "mt-8 space-y-0 overflow-hidden rounded-sm border border-border", children: ARCH_ENABLERS.map((item, i) => (_jsxs("li", { className: "group flex gap-4 border-b border-white/[0.06] glass px-5 py-4 transition-all duration-200 last:border-b-0", children: [_jsx("span", { className: "mt-px shrink-0 font-mono text-[11px] text-kai-dim", children: String(i + 1).padStart(2, "0") }), _jsxs("div", { className: "min-w-0 flex-1", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx("p", { className: "font-mono text-[13.5px] font-bold text-foreground", children: item.title }), item.done && (_jsx("span", { className: "flex size-4 shrink-0 items-center justify-center rounded-full bg-kai-green/15 text-kai-green", title: "Already shipped", children: _jsx(Icon, { name: "Check", className: "size-2.5", strokeWidth: 3 }) }))] }), _jsx("p", { className: "mt-1 font-sans text-[13px] leading-relaxed text-muted-foreground", children: item.body })] })] }, item.title))) })] }) }), _jsxs("section", { className: "relative overflow-hidden border-t border-border", children: [_jsx("div", { className: "rule-sweep absolute inset-x-0 top-0 h-px", "aria-hidden": true }), _jsxs("div", { className: "mx-auto max-w-6xl px-4 py-16 text-center sm:px-6 sm:py-20", children: [_jsx("p", { className: "font-mono text-[11px] tracking-[0.3em] text-kai-dim uppercase", children: "the roadmap is the repo" }), _jsxs("h2", { className: "mx-auto mt-4 max-w-2xl text-balance font-mono text-xl font-bold text-foreground sm:text-2xl", children: ["Every feature here maps to an", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "existing interface" })] }), _jsx("p", { className: "mx-auto mt-4 max-w-xl font-sans text-[14px] leading-relaxed text-muted-foreground", children: "The agent's UI interface, the tool schema, the Progress callbacks, the serve routes, and the codemap Index \u2014 these are the extension points each enhancement plugs into." })] })] })] }));
}
