import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import Reveal from "@/components/ui/Reveal";
const ECOSYSTEM_STACKS = [
    { name: "Golang", tag: "Go 1.22+" },
    { name: "Rust", tag: "Cargo / Tokio" },
    { name: "TypeScript", tag: "Node / Bun" },
    { name: "Python", tag: "PyO3 / Poetry" },
    { name: "PostgreSQL", tag: "SQL / Migrations" },
    { name: "Docker", tag: "Containers" },
    { name: "Kubernetes", tag: "Infra Manifests" },
];
export default function LogoBar() {
    return (_jsx("section", { className: "border-y border-white/10 bg-[#08080a] py-8", children: _jsx("div", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: _jsx(Reveal, { children: _jsxs("div", { className: "flex flex-col items-center justify-between gap-6 md:flex-row", children: [_jsxs("div", { className: "flex items-center gap-2.5 font-mono text-xs text-kai-muted shrink-0", children: [_jsx("span", { className: "size-1.5 rounded-full bg-kai-orange animate-pulse" }), _jsx("span", { children: "ARCHITECTED FOR SERIOUS CODEBASES" })] }), _jsx("div", { className: "flex flex-wrap items-center justify-center gap-3 md:justify-end", children: ECOSYSTEM_STACKS.map((tech) => (_jsxs("div", { className: "flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3.5 py-1 font-mono text-xs text-kai-text transition-colors hover:border-white/20 hover:text-white", children: [_jsx("span", { className: "font-semibold text-white", children: tech.name }), _jsx("span", { className: "text-[10px] text-kai-dim", children: "\u2022" }), _jsx("span", { className: "text-[10px] text-kai-muted", children: tech.tag })] }, tech.name))) })] }) }) }) }));
}
