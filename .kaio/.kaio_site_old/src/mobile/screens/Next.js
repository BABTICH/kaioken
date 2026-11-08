import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import Icon from "@/components/Icon";
import Accordion from "@/mobile/components/Accordion";
import { Eyebrow, Lead, Note, Section, SectionHead, StatGrid, Tag, } from "@/mobile/components/primitives";
import { tone, toneText } from "@/mobile/lib/tone";
import { ARCH_ENABLERS, ROADMAP_CATEGORIES } from "@/data/roadmap";
import { cn } from "@/lib/utils";
const TOTAL_FEATURES = ROADMAP_CATEGORIES.reduce((n, c) => n + c.items.length, 0);
const STATS = [
    { value: String(ROADMAP_CATEGORIES.length), label: "categories" },
    { value: String(TOTAL_FEATURES), label: "features" },
    { value: String(ARCH_ENABLERS.length), label: "enablers" },
    { value: "1", label: "binary" },
];
/** Shipped items carry a check; the rest are still ideas. */
function Done() {
    return (_jsx("span", { className: "ml-1.5 inline-flex size-4 shrink-0 translate-y-px items-center justify-center rounded-full bg-kai-green/15 text-kai-green", children: _jsx(Icon, { name: "Check", className: "size-2.5", strokeWidth: 3, "aria-hidden": true }) }));
}
export default function Next() {
    return (_jsxs(_Fragment, { children: [_jsxs("header", { className: "px-4 pt-6", children: [_jsx(Eyebrow, { children: "next" }), _jsxs("h1", { className: "mt-3 font-mono text-[26px] leading-[1.2] font-bold tracking-tight text-foreground", children: ["What comes ", _jsx("span", { className: "text-kai-orange glow-orange", children: "next" })] }), _jsx(Lead, { className: "mt-3", children: "Advanced agents, richer search, a GUI shell, deeper integrations and extended language support. Every item builds on what already exists \u2014 the agent loop, the tool framework, the codemap index, the wiki engine and the serve layer." }), _jsx(StatGrid, { className: "mt-6", items: STATS }), _jsx("div", { className: "mt-4", children: _jsxs(Note, { tone: "orange", glyph: "\u258E", children: ["These enhancements extend the current architecture \u2014 the", " ", _jsx("span", { className: "font-mono text-foreground", children: "Agent.Run()" }), " tool loop,", " ", _jsx("span", { className: "font-mono text-foreground", children: "codemap.Index" }), " symbol extraction,", " ", _jsx("span", { className: "font-mono text-foreground", children: "wiki.Run()" }), " multi-pass pipeline,", " ", _jsx("span", { className: "font-mono text-foreground", children: "serve.Server" }), " HTTP layer and the", " ", _jsx("span", { className: "font-mono text-foreground", children: "skills" }), " system \u2014 rather than replacing it."] }) })] }), _jsxs(Section, { className: "pt-8", children: [_jsx(SectionHead, { index: "01", eyebrow: "the plan", title: _jsxs(_Fragment, { children: [ROADMAP_CATEGORIES.length, " categories"] }), lead: "Tap a category to read its features." }), _jsx(Accordion, { className: "mt-5", items: ROADMAP_CATEGORIES.map((cat) => {
                            const t = tone(cat.tone);
                            return {
                                id: cat.id,
                                title: cat.title,
                                glyph: _jsx(Icon, { name: cat.icon, className: cn("size-4", toneText[t]), "aria-hidden": true }),
                                meta: _jsx(Tag, { children: cat.items.length }),
                                body: (_jsxs(_Fragment, { children: [_jsx("p", { className: "font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: cat.blurb }), cat.status ? (_jsx("p", { className: "mt-2", children: _jsx(Tag, { className: toneText[t], children: cat.status }) })) : null, _jsx("ul", { className: "mt-3 divide-y divide-border/70 border-t border-border/70", children: cat.items.map((item) => (_jsxs("li", { className: "py-2.5", children: [_jsxs("p", { className: "font-mono text-[12.5px] font-bold text-foreground", children: [item.title, item.done ? _jsx(Done, {}) : null] }), _jsx("p", { className: "mt-1 font-sans text-[12px] leading-[1.55] text-muted-foreground", children: item.body })] }, item.title))) })] })),
                            };
                        }) })] }), _jsxs(Section, { className: "pt-8", children: [_jsx(SectionHead, { index: "02", eyebrow: "foundation", title: _jsxs(_Fragment, { children: ["Architectural ", _jsx("span", { className: "text-kai-amber", children: "enablers" })] }), lead: "Cross-cutting changes that underpin several categories above. Each one extends an existing structure rather than introducing a new system." }), _jsx("ol", { className: "mt-5 divide-y divide-border overflow-hidden rounded-md border border-border", children: ARCH_ENABLERS.map((item, i) => (_jsxs("li", { className: "flex gap-3 bg-card px-4 py-3.5", children: [_jsx("span", { className: "mt-px shrink-0 font-mono text-[11px] text-kai-dim", children: String(i + 1).padStart(2, "0") }), _jsxs("div", { className: "min-w-0", children: [_jsxs("p", { className: "font-mono text-[12.5px] font-bold text-foreground", children: [item.title, item.done ? _jsx(Done, {}) : null] }), _jsx("p", { className: "mt-1 font-sans text-[12px] leading-[1.55] text-muted-foreground", children: item.body })] })] }, item.title))) })] }), _jsxs("section", { className: "relative overflow-hidden border-t border-border px-4 py-12 text-center", children: [_jsx("div", { className: "rule-sweep absolute inset-x-0 top-0 h-px", "aria-hidden": true }), _jsx("p", { className: "font-mono text-[10.5px] tracking-[0.3em] text-kai-dim uppercase", children: "the roadmap is the repo" }), _jsxs("h2", { className: "mt-3.5 font-mono text-[19px] leading-[1.3] font-bold text-balance text-foreground", children: ["Every feature here maps to an", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "existing interface" })] }), _jsx(Lead, { className: "mt-3", children: "The agent's UI interface, the tool schema, the Progress callbacks, the serve routes and the codemap Index \u2014 these are the extension points each enhancement plugs into." })] })] }));
}
