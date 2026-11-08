import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { FEATURES } from "@/data/features";
import TerminalWindow from "@/components/terminal/TerminalWindow";
import Badge from "@/components/ui/Badge";
import Reveal from "@/components/ui/Reveal";
import { cn } from "@/lib/utils";
export default function FeatureListPreview() {
    const [activeFeature, setActiveFeature] = React.useState(FEATURES[0]);
    return (_jsx("section", { id: "features", className: "py-24 border-b border-white/10 bg-[#050507]", children: _jsxs("div", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: [_jsx(Reveal, { children: _jsxs("div", { className: "mb-14", children: [_jsx(Badge, { variant: "orange", isComment: true, className: "mb-3", children: "ARCHITECTURAL ADVANTAGES" }), _jsxs("h2", { className: "font-sans text-3xl font-bold tracking-tight text-white sm:text-4xl", children: ["Every capability you need.", " ", _jsx("span", { className: "text-kai-orange", children: "None of the slop." })] }), _jsx("p", { className: "mt-3 max-w-xl font-sans text-sm text-kai-muted sm:text-base", children: "Explore the core architectural engines that differentiate Kaioken v2 from surface-level coding assistants." })] }) }), _jsxs("div", { className: "grid gap-10 lg:grid-cols-12 lg:items-start", children: [_jsx("div", { className: "flex flex-col divide-y divide-white/10 border-y border-white/10 lg:col-span-6", children: FEATURES.map((item) => {
                                const isActive = activeFeature.id === item.id;
                                return (_jsxs("div", { onMouseEnter: () => setActiveFeature(item), onClick: () => setActiveFeature(item), className: cn("group relative flex cursor-pointer flex-col p-5 transition-all duration-200", isActive
                                        ? "bg-white/[0.04] border-l-2 border-l-kai-orange"
                                        : "hover:bg-white/[0.02] border-l-2 border-l-transparent"), children: [_jsxs("div", { className: "flex items-center justify-between", children: [_jsxs("div", { className: "flex items-center gap-3", children: [_jsx("span", { className: cn("font-mono text-xs font-bold transition-colors", isActive ? "text-kai-orange" : "text-kai-dim group-hover:text-kai-muted"), children: item.number }), _jsx("h3", { className: cn("font-sans text-base font-semibold transition-colors", isActive ? "text-white" : "text-kai-text group-hover:text-white"), children: item.title })] }), _jsx("span", { className: "rounded border border-white/10 bg-white/5 px-2 py-0.5 font-mono text-[10px] text-kai-muted", children: item.tag })] }), _jsx("p", { className: "mt-2 font-sans text-xs leading-relaxed text-kai-muted", children: item.description })] }, item.id));
                            }) }), _jsxs("div", { className: "sticky top-24 lg:col-span-6", children: [_jsx(TerminalWindow, { title: `kaioken — ${activeFeature.id}`, status: "live preview", className: "min-h-[380px] shadow-2xl border-white/15", children: _jsxs("div", { className: "space-y-2", children: [_jsxs("div", { className: "flex items-center gap-2 text-kai-blue font-semibold", children: [_jsx("span", { children: "\u203A" }), _jsx("span", { children: activeFeature.terminalOutput.command })] }), _jsx("div", { className: "mt-3 space-y-1.5 font-mono text-xs", children: activeFeature.terminalOutput.lines.map((line, idx) => {
                                                    const colorClass = {
                                                        user: "text-kai-blue font-semibold",
                                                        tool: "text-kai-tan",
                                                        toolres: "text-kai-sage",
                                                        add: "text-kai-green",
                                                        del: "text-kai-rose",
                                                        ok: "text-kai-green font-medium",
                                                        dim: "text-kai-dim",
                                                        warn: "text-kai-amber",
                                                    }[line.type];
                                                    return (_jsx("div", { className: cn("leading-relaxed", colorClass), children: line.text }, idx));
                                                }) })] }) }), _jsxs("div", { className: "mt-4 flex items-center justify-between font-mono text-[11px] text-kai-dim", children: [_jsx("span", { children: "Hover or tap items to inspect execution" }), _jsx("span", { className: "text-kai-amber", children: "07 capabilities total" })] })] })] })] }) }));
}
