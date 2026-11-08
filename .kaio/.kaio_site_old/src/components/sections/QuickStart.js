import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { ArrowRight } from "lucide-react";
import GithubMark from "@/components/GithubMark";
import CodeBlock from "@/components/CodeBlock";
import SectionHeading from "@/components/SectionHeading";
import LinkButton from "@/components/LinkButton";
import { GITHUB_URL, QUICK_START } from "@/data/content";
const REQUIREMENTS = [
    { label: "Go", value: "≥ 1.24" },
    { label: "execution engine", value: "Local hardware weights, unified gateway, …" },
    { label: "a git repo", value: "for incremental updates" },
];
export default function QuickStart() {
    return (_jsx("section", { id: "quick-start", className: "relative border-t border-border py-20 sm:py-24", children: _jsx("div", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: _jsxs("div", { className: "grid gap-10 lg:grid-cols-[1fr_1.25fr] lg:gap-14", children: [_jsxs("div", { className: "lg:sticky lg:top-24 lg:self-start", children: [_jsx(SectionHeading, { index: "08", eyebrow: "quick start", title: "Running in one build", description: "One Go build, one environment variable, and you are in the TUI. Bare kaioken launches it; every subcommand exists too, for CI." }), _jsx("dl", { className: "mt-8 space-y-3", children: REQUIREMENTS.map((r) => (_jsxs("div", { className: "flex items-baseline gap-3 font-mono text-[12.5px]", children: [_jsx("dt", { className: "w-28 shrink-0 text-kai-amber", children: r.label }), _jsx("dd", { className: "text-muted-foreground", children: r.value })] }, r.label))) }), _jsxs("div", { className: "mt-8 flex flex-wrap gap-3", children: [_jsxs(LinkButton, { to: "/docs/install", children: ["Install guide", _jsx(ArrowRight, { "data-icon": "inline-end" })] }), _jsxs(LinkButton, { href: GITHUB_URL, variant: "outline", children: [_jsx(GithubMark, { "data-icon": "inline-start" }), "Source"] })] })] }), _jsxs("div", { className: "space-y-4", children: [_jsx(CodeBlock, { title: "powershell", code: QUICK_START }), _jsxs("p", { className: "font-mono text-[11.5px] leading-relaxed text-kai-dim", children: [_jsx("span", { className: "text-kai-green", children: "\u2713" }), " a model id ending in", " ", _jsx("span", { className: "text-kai-green", children: ":free" }), " caps parallelism at 2 \u2014 those tiers rate-limit hard, and four parallel calls mostly buys 429s"] })] })] }) }) }));
}
