import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { CHANGELOG } from "@/data/changelog";
import Badge from "@/components/ui/Badge";
import Reveal from "@/components/ui/Reveal";
import { cn } from "@/lib/utils";
export default function ChangelogSection() {
    return (_jsx("section", { id: "changelog", className: "py-24 border-b border-white/10 bg-[#060608]", children: _jsxs("div", { className: "mx-auto max-w-4xl px-4 sm:px-6", children: [_jsx(Reveal, { children: _jsxs("div", { className: "mb-14", children: [_jsx(Badge, { variant: "blue", isComment: true, className: "mb-3", children: "CONTINUOUS DELIVERY" }), _jsxs("div", { className: "flex flex-wrap items-baseline gap-4", children: [_jsx("h2", { className: "font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl", children: "Changelog" }), _jsx("span", { className: "font-mono text-2xl font-bold text-kai-blue glow-orange", children: "v2.0.0" })] }), _jsx("p", { className: "mt-3 max-w-xl font-sans text-sm text-kai-muted", children: "Recent engine advancements, performance optimizations, and provider integrations." })] }) }), _jsx("div", { className: "space-y-10", children: CHANGELOG.map((release) => (_jsx(Reveal, { children: _jsxs("div", { className: "rounded-lg border border-white/10 bg-[#0a0b10] p-6", children: [_jsx("div", { className: "flex items-center justify-between border-b border-white/10 pb-4 mb-6", children: _jsxs("div", { className: "flex items-center gap-3", children: [_jsx("span", { className: "font-mono text-base font-bold text-white", children: release.version }), _jsx("span", { className: "rounded-full border border-kai-blue/30 bg-kai-blue/10 px-2.5 py-0.5 font-mono text-[10px] text-kai-blue", children: release.date })] }) }), _jsx("div", { className: "space-y-4", children: release.highlights.map((h, i) => {
                                        const tagStyle = {
                                            FEAT: "border-kai-green/30 bg-kai-green/10 text-kai-green",
                                            MODELS: "border-[#c084fc]/30 bg-[#c084fc]/10 text-[#d8b4fe]",
                                            PERF: "border-kai-orange/30 bg-kai-orange/10 text-kai-orange",
                                            FIX: "border-kai-amber/30 bg-kai-amber/10 text-kai-amber",
                                        }[h.tag];
                                        return (_jsxs("div", { className: "flex flex-col sm:flex-row sm:items-baseline gap-2 sm:gap-4", children: [_jsx("span", { className: cn("inline-block w-fit rounded border px-2 py-0.5 font-mono text-[10px] font-bold uppercase shrink-0", tagStyle), children: h.tag }), _jsxs("div", { children: [_jsx("h4", { className: "font-sans text-sm font-semibold text-white", children: h.title }), _jsx("p", { className: "mt-0.5 font-sans text-xs text-kai-muted leading-relaxed", children: h.description })] })] }, i));
                                    }) })] }) }, release.version))) })] }) }));
}
