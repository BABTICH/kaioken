import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { ArrowRight, Monitor, Terminal } from "lucide-react";
import AsciiArt from "@/components/AsciiArt";
import GithubMark from "@/components/GithubMark";
import Icon from "@/components/Icon";
import Accordion, { Points } from "@/mobile/components/Accordion";
import Code from "@/mobile/components/Code";
import FitText from "@/mobile/components/FitText";
import TerminalPeek from "@/mobile/components/TerminalPeek";
import { Action, Card, Eyebrow, Lead, Note, Rail, Section, SectionHead, Tag, } from "@/mobile/components/primitives";
import { tone, toneSurface, toneText } from "@/mobile/lib/tone";
import { ASCII_LOGO, COMMAND_GROUPS, DESIGN_DECISIONS, FEATURES, GITHUB_URL, MULTIPLIER, OUTPUT_TREE, PIPELINE, PROVIDERS, QUALITY, QUICK_START, RESEARCH, RESEARCH_EXAMPLE, RESEARCH_NOTES, ROADMAP, } from "@/data/content";
import { cn } from "@/lib/utils";
export default function Home() {
    return (_jsxs(_Fragment, { children: [_jsx(Hero, {}), _jsx(Features, {}), _jsx(Pipeline, {}), _jsx(MultiplierDial, {}), _jsx(Commands, {}), _jsx(Quality, {}), _jsx(Research, {}), _jsx(OutputTree, {}), _jsx(QuickStart, {}), _jsx(DesktopTeaser, {}), _jsx(Decisions, {}), _jsx(Closing, {})] }));
}
/* ── hero ─────────────────────────────────────────────────────────────────── */
function Hero() {
    return (_jsxs("section", { className: "px-4 pt-6 pb-9", children: [_jsxs("div", { className: "animate-rise text-center", children: [_jsx(FitText, { children: _jsx(AsciiArt, { art: ASCII_LOGO, label: "kaioken", className: "text-[10px] leading-[1.3]" }) }), _jsx("div", { className: "mx-auto mt-1 h-px w-full max-w-[22rem] opacity-40", style: { background: "linear-gradient(90deg, var(--kai-red), var(--kai-orange))" }, "aria-hidden": true }), _jsx("p", { className: "mt-6 inline-block rounded-sm border border-kai-orange/35 bg-kai-orange/10 px-2.5 py-1 font-mono text-[10.5px] text-kai-amber", children: "single Go binary \u00B7 TUI \u00B7 provider-agnostic" }), _jsxs("h1", { className: "mt-5 font-mono text-[27px] leading-[1.2] font-bold tracking-tight text-balance text-foreground", children: ["A terminal AI", _jsx("br", {}), "coding assistant", _jsx("br", {}), _jsx("span", { className: "text-kai-orange glow-orange", children: "+ knowledge engine" })] }), _jsx(Lead, { className: "mt-4", children: "One binary, two faces. A chat agent that edits your repo behind diff approval \u2014 and an engine that turns the codebase into deep wikis, knowledge cards and skills your agent loads before it starts working." })] }), _jsxs("div", { className: "animate-rise mt-7 space-y-2.5", style: { animationDelay: "0.1s" }, children: [_jsxs(Action, { to: "/docs/install", children: ["Get started", _jsx(ArrowRight, { className: "size-4", "aria-hidden": true })] }), _jsxs(Action, { href: GITHUB_URL, variant: "outline", children: [_jsx(GithubMark, { className: "size-4" }), "View source"] })] }), _jsx(TerminalPeek, { className: "animate-rise mt-8" }), _jsxs("p", { className: "mt-4 text-center font-mono text-[10.5px] leading-[1.9] text-balance text-kai-dim", children: [PROVIDERS.join(" · "), " \u00B7 ~20 providers"] })] }));
}
/* ── features ─────────────────────────────────────────────────────────────── */
function Features() {
    return (_jsxs(Section, { id: "features", children: [_jsx(SectionHead, { index: "01", eyebrow: "what it does", title: _jsxs(_Fragment, { children: ["Two faces,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "one binary" })] }), lead: "Swipe through the six things it ships with." }), _jsx(Rail, { className: "mt-6", children: FEATURES.map((f) => {
                    const t = tone(f.tone);
                    return (_jsxs("article", { className: cn("flex w-[82vw] max-w-[20rem] flex-col rounded-md border p-4", toneSurface[t]), children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx(Icon, { name: f.icon, className: cn("size-4 shrink-0", toneText[t]), "aria-hidden": true }), _jsx("h3", { className: "font-mono text-[14px] font-bold text-foreground", children: f.title })] }), _jsx("p", { className: "mt-2.5 font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: f.description }), _jsx("div", { className: "mt-3.5 border-t border-border/70 pt-3.5", children: _jsx(Points, { items: f.highlights, tone: toneText[t] }) })] }, f.title));
                }) }), _jsxs("p", { className: "mt-3 font-mono text-[10.5px] text-kai-dim", children: ["\u2190 swipe \u00B7 ", FEATURES.length, " cards"] })] }));
}
/* ── pipeline ─────────────────────────────────────────────────────────────── */
function Pipeline() {
    return (_jsxs(Section, { id: "pipeline", children: [_jsx(SectionHead, { index: "02", eyebrow: "the pipeline", title: _jsxs(_Fragment, { children: ["Six commands,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "one of them yours" })] }), lead: "Nothing expensive runs before you have seen what it plans to do." }), _jsx("ol", { className: "mt-6", children: PIPELINE.map((step, i) => (_jsxs("li", { className: "relative flex gap-3.5 pb-5 last:pb-0", children: [i < PIPELINE.length - 1 ? (_jsx("span", { className: "absolute top-7 bottom-0 left-[13px] w-px bg-border", "aria-hidden": true })) : null, _jsx("span", { className: cn("relative z-10 flex size-7 shrink-0 items-center justify-center rounded-sm border font-mono text-[11px] font-bold", step.human
                                ? "border-kai-amber/50 bg-kai-amber/15 text-kai-amber"
                                : "border-border bg-card text-kai-dim"), children: String(i + 1).padStart(2, "0") }), _jsxs("div", { className: "min-w-0 flex-1 pt-0.5", children: [_jsxs("div", { className: "flex flex-wrap items-center gap-2", children: [_jsx("code", { className: "font-mono text-[12.5px] font-bold text-kai-orange", children: step.cmd }), step.human ? (_jsx(Tag, { className: "border-kai-amber/40 text-kai-amber", children: "your call" })) : null] }), _jsx("p", { className: "mt-0.5 font-mono text-[12px] text-foreground", children: step.title }), _jsx("p", { className: "mt-1.5 font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: step.body })] })] }, step.cmd))) })] }));
}
/* ── deep research ───────────────────────────────────────────────────────────── */
function Research() {
    return (_jsxs(Section, { id: "research", children: [_jsx(SectionHead, { index: "06", eyebrow: "deep research", title: _jsxs(_Fragment, { children: ["Ask the open web,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "get a cited answer" })] }), lead: "One question in, a sourced report out. A cheap router decides whether it needs a quick loop or a team of parallel agents." }), _jsx("ol", { className: "mt-6", children: RESEARCH.map((step, i) => (_jsxs("li", { className: "relative flex gap-3.5 pb-5 last:pb-0", children: [i < RESEARCH.length - 1 ? (_jsx("span", { className: "absolute top-7 bottom-0 left-[13px] w-px bg-border", "aria-hidden": true })) : null, _jsx("span", { className: cn("relative z-10 flex size-7 shrink-0 items-center justify-center rounded-sm border font-mono text-[11px] font-bold", step.tone === "amber"
                                ? "border-kai-amber/50 bg-kai-amber/15 text-kai-amber"
                                : "border-border bg-card text-kai-orange"), children: String(i + 1).padStart(2, "0") }), _jsxs("div", { className: "min-w-0 flex-1 pt-0.5", children: [_jsx("p", { className: "font-mono text-[12px] text-foreground", children: step.title }), _jsx("p", { className: "mt-1.5 font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: step.body })] })] }, step.title))) }), _jsx(Code, { className: "mt-6", title: "powershell", code: RESEARCH_EXAMPLE }), _jsx("ul", { className: "mt-4 space-y-2.5", children: RESEARCH_NOTES.map((note) => (_jsxs("li", { className: "flex gap-2.5", children: [_jsx("span", { className: "mt-px shrink-0 font-mono text-[12px] text-kai-green", "aria-hidden": true, children: "\u2713" }), _jsx("span", { className: "font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: note })] }, note))) })] }));
}
/* ── multiplier ───────────────────────────────────────────────────────── */
function MultiplierDial() {
    const defaultIndex = Math.max(0, MULTIPLIER.findIndex((m) => m.isDefault));
    const [active, setActive] = React.useState(defaultIndex);
    const level = MULTIPLIER[active];
    return (_jsxs(Section, { id: "multiplier", children: [_jsx(SectionHead, { index: "03", eyebrow: "the multiplier", title: "\u00D7N buys passes, not padding", lead: "Each level roughly doubles the calls per document \u2014 which is what a power-multiplier metaphor ought to mean." }), _jsx("div", { role: "tablist", "aria-label": "Wiki depth", className: "mt-6 flex gap-px overflow-hidden rounded-md border border-border bg-border", children: MULTIPLIER.map((m, i) => (_jsx("button", { type: "button", role: "tab", "aria-selected": i === active, onClick: () => setActive(i), className: cn("h-11 flex-1 font-mono text-[13px] font-bold transition-colors", i === active
                        ? "bg-kai-orange text-[#180c00]"
                        : "bg-card text-kai-dim"), children: m.level }, m.level))) }), _jsxs(Card, { className: "mt-3", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "font-mono text-[17px] font-bold text-kai-orange", children: level.level }), level.isDefault ? (_jsx(Tag, { className: "border-kai-orange/40 text-kai-orange", children: "default" })) : null] }), _jsx("p", { className: "mt-2 font-sans text-[13.5px] leading-[1.6] text-muted-foreground", children: level.behavior })] }), _jsxs("p", { className: "mt-3 font-mono text-[11px] leading-relaxed text-kai-dim", children: [_jsx("span", { className: "text-kai-green", children: "\u2713" }), " the estimate names the passes before the run starts, and asks for confirmation past a threshold"] })] }));
}
/* ── engineering notes ────────────────────────────────────────────────────── */
function Quality() {
    return (_jsxs(Section, { id: "quality", children: [_jsx(SectionHead, { index: "05", eyebrow: "why it holds up", title: _jsxs(_Fragment, { children: ["Six things that make the output", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "trustworthy" })] }) }), _jsx(Accordion, { className: "mt-6", defaultOpen: QUALITY[0].claim, items: QUALITY.map((q) => ({
                    id: q.claim,
                    title: q.claim,
                    glyph: _jsx(Icon, { name: q.icon, className: "size-4 text-kai-orange", "aria-hidden": true }),
                    body: (_jsx("p", { className: "font-sans text-[12.5px] leading-[1.65] text-muted-foreground", children: q.body })),
                })) })] }));
}
/* ── commands ─────────────────────────────────────────────────────────────── */
function Commands() {
    const total = COMMAND_GROUPS.reduce((n, g) => n + g.commands.length, 0);
    return (_jsxs(Section, { id: "commands", children: [_jsx(SectionHead, { index: "04", eyebrow: "slash commands", title: _jsxs(_Fragment, { children: [total, " commands, four groups"] }), lead: "Everything the TUI answers to. Tap a group to open it." }), _jsx(Accordion, { className: "mt-6", items: COMMAND_GROUPS.map((g) => ({
                    id: g.id,
                    title: g.label,
                    glyph: _jsx(Icon, { name: g.icon, className: "size-4 text-kai-orange", "aria-hidden": true }),
                    meta: _jsx(Tag, { children: g.commands.length }),
                    body: (_jsxs(_Fragment, { children: [_jsx("p", { className: "font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: g.blurb }), _jsx("ul", { className: "mt-3 divide-y divide-border/70 border-t border-border/70", children: g.commands.map((c) => (_jsxs("li", { className: "py-2.5", children: [_jsxs("p", { className: "font-mono text-[12.5px]", children: [_jsx("span", { className: "font-bold text-kai-orange", children: c.name }), c.args ? _jsx("span", { className: "ml-1.5 text-kai-dim", children: c.args }) : null] }), _jsx("p", { className: "mt-1 font-sans text-[12px] leading-[1.55] text-muted-foreground", children: c.summary })] }, c.name))) })] })),
                })) }), _jsxs(Action, { to: "/docs/commands", variant: "outline", className: "mt-4", children: ["Full command reference", _jsx(ArrowRight, { className: "size-4", "aria-hidden": true })] })] }));
}
/* ── output tree ──────────────────────────────────────────────────────────── */
const KIND_COLOR = {
    dir: "text-kai-orange",
    edit: "text-kai-amber",
    file: "text-muted-foreground",
};
function OutputTree() {
    return (_jsxs(Section, { id: "output", children: [_jsx(SectionHead, { index: "07", eyebrow: "what lands on disk", title: _jsxs(_Fragment, { children: ["One folder,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "plain files" })] }), lead: "No database, no lock-in \u2014 markdown and YAML you can read, diff and commit." }), _jsxs("div", { className: "mt-6 overflow-hidden rounded-md border border-border bg-card", children: [_jsxs("div", { className: "flex items-center gap-2 border-b border-border bg-kai-panel px-3 py-2", children: [_jsx("span", { className: "size-2 rounded-[1px] bg-kai-orange/60", "aria-hidden": true }), _jsx("span", { className: "font-mono text-[10.5px] tracking-[0.15em] text-kai-dim uppercase", children: ".kaioken/" })] }), _jsx("div", { className: "m-no-scrollbar overflow-x-auto px-3 py-3", children: _jsx("ul", { className: "min-w-max font-mono text-[11.5px] leading-[1.9]", children: OUTPUT_TREE.map((node, i) => (_jsxs("li", { className: "whitespace-pre", style: { paddingLeft: `${node.depth * 1.1}rem` }, children: [_jsxs("span", { className: KIND_COLOR[node.kind] ?? "text-muted-foreground", children: [node.kind === "dir" ? "▸ " : "  ", node.name] }), node.note ? (_jsxs("span", { className: "text-kai-dim", children: ["  · ", node.note] })) : null] }, `${node.name}-${i}`))) }) })] }), _jsx(Note, { tone: "amber", glyph: "!", children: "The two files marked in amber are meant to be edited between passes \u2014 module boundaries and the wiki outline are judgment calls the maintainer should own." })] }));
}
/* ── quick start ──────────────────────────────────────────────────────────── */
function QuickStart() {
    return (_jsxs(Section, { id: "quickstart", children: [_jsx(SectionHead, { index: "08", eyebrow: "quick start", title: "From clone to wiki", lead: "The scan is free, the plan is a file you can edit, and the cost estimate prints before anything expensive runs." }), _jsx(Code, { className: "mt-6", title: "powershell", code: QUICK_START })] }));
}
/* ── desktop teaser ───────────────────────────────────────────────────────── */
function DesktopTeaser() {
    return (_jsxs(Section, { id: "desktop", children: [_jsx(SectionHead, { index: "09", eyebrow: "desktop", title: _jsxs(_Fragment, { children: ["Same engine,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "new surface" })] }), lead: "Everything the TUI does, in a window \u2014 with diff approval you can read and runs you can watch concurrently." }), _jsx("div", { className: "mt-6 grid grid-cols-2 gap-2", children: [
                    { v: "12", l: "surfaces" },
                    { v: "1", l: "extra process" },
                    { v: "0", l: "telemetry" },
                    { v: "3", l: "platforms" },
                ].map((s) => (_jsxs("div", { className: "rounded-md border border-border bg-card px-3 py-3.5", children: [_jsx("p", { className: "font-mono text-[20px] leading-none font-bold text-kai-orange", children: s.v }), _jsx("p", { className: "mt-1.5 font-mono text-[10px] tracking-[0.12em] text-kai-dim uppercase", children: s.l })] }, s.l))) }), _jsxs(Action, { to: "/desktop", variant: "outline", className: "mt-4", children: [_jsx(Monitor, { className: "size-4", "aria-hidden": true }), "Explore the desktop app", _jsx(ArrowRight, { className: "size-4", "aria-hidden": true })] })] }));
}
/* ── design decisions + roadmap ───────────────────────────────────────────── */
function Decisions() {
    return (_jsxs(Section, { id: "decisions", children: [_jsx(SectionHead, { index: "10", eyebrow: "design decisions", title: "Choices worth stating" }), _jsx("dl", { className: "mt-6 space-y-4", children: DESIGN_DECISIONS.map((d) => (_jsxs("div", { className: "border-l-2 border-kai-orange/40 pl-3.5", children: [_jsx("dt", { className: "font-mono text-[12.5px] font-bold text-foreground", children: d.title }), _jsx("dd", { className: "mt-1 font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: d.body })] }, d.title))) }), _jsxs("div", { className: "mt-8", children: [_jsx(Eyebrow, { tone: "dim", children: "roadmap \u00B7 not yet built" }), _jsx("ul", { className: "mt-3.5 space-y-2.5", children: ROADMAP.map((item) => (_jsxs("li", { className: "flex gap-2.5", children: [_jsx("span", { className: "mt-px shrink-0 font-mono text-[12px] text-kai-dim", "aria-hidden": true, children: "\u2610" }), _jsx("span", { className: "font-mono text-[12px] leading-[1.55] text-muted-foreground", children: item })] }, item))) }), _jsxs(Action, { to: "/next", variant: "outline", className: "mt-5", children: ["The full roadmap", _jsx(ArrowRight, { className: "size-4", "aria-hidden": true })] })] })] }));
}
/* ── closing ──────────────────────────────────────────────────────────────── */
function Closing() {
    return (_jsxs("section", { className: "relative overflow-hidden border-t border-border px-4 py-12 text-center", children: [_jsx("div", { className: "rule-sweep absolute inset-x-0 top-0 h-px", "aria-hidden": true }), _jsx("p", { className: "font-mono text-[10.5px] tracking-[0.3em] text-kai-dim uppercase", children: "ready when you are" }), _jsxs("h2", { className: "mt-3.5 font-mono text-[20px] leading-[1.3] font-bold text-balance text-foreground", children: ["Point it at a repo and", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "read what comes back" })] }), _jsx(Lead, { className: "mt-3", children: "The scan is free, the plan is a file you can edit, and the cost estimate prints before anything expensive runs." }), _jsxs("div", { className: "mt-6 space-y-2.5", children: [_jsxs(Action, { to: "/docs/install", children: [_jsx(Terminal, { className: "size-4", "aria-hidden": true }), "Start here"] }), _jsxs(Action, { to: "/preview", variant: "outline", children: ["Read what it generated", _jsx(ArrowRight, { className: "size-4", "aria-hidden": true })] })] })] }));
}
