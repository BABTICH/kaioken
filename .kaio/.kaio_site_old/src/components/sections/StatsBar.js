import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import Reveal from "@/components/Reveal";
import { cn } from "@/lib/utils";
const STATS = [
    { value: "1", label: "Go binary", sub: "single executable", accent: "text-kai-orange" },
    { value: "100%", label: "Private & Local", sub: "Air-gapped · Local Hardware · Gateways", accent: "text-kai-amber" },
    { value: "∞", label: "context depth", sub: "deep wiki + knowledge cards", accent: "text-kai-green" },
    { value: "MIT", label: "open source", sub: "no cloud lock-in", accent: "text-kai-blue" },
    { value: "0", label: "dependencies", sub: "for the binary itself", accent: "text-kai-tan" },
];
export default function StatsBar() {
    return (_jsxs("section", { className: "relative border-t border-border section-alt py-10 sm:py-12", children: [_jsx("div", { className: "rule-sweep absolute inset-x-0 top-0 h-px", "aria-hidden": true }), _jsx("div", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: _jsx(Reveal, { children: _jsx("div", { className: "grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-5", children: STATS.map((s, i) => (_jsxs("div", { className: cn("group flex flex-col items-center gap-1 rounded-md px-4 py-5 text-center glass", "lift cursor-default"), style: { transitionDelay: `${i * 50}ms` }, children: [_jsx("span", { className: cn("font-mono text-3xl font-bold tracking-tight", s.accent), children: s.value }), _jsx("span", { className: "font-mono text-[13px] font-semibold text-foreground", children: s.label }), _jsx("span", { className: "font-sans text-[11px] leading-snug text-muted-foreground", children: s.sub })] }, s.label))) }) }) })] }));
}
