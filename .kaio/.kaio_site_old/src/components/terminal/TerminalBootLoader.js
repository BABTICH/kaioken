import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { cn } from "@/lib/utils";
const BOOT_STEPS = [
    { text: "KAIOKEN ARCHITECTURAL ENGINE v2.4.0 (x86_64-pc-windows)", delay: 30 },
    { text: "Kernel bootstrap: memory arenas initialized (64MB locked)", delay: 40 },
    { text: "Loading AST symbol graph & package call trees", status: "DONE", statusColor: "text-kai-green", delay: 50 },
    { text: "Initializing git worktree sandboxes & delegation pool", status: "READY", statusColor: "text-kai-green", delay: 50 },
    { text: "Mounting autonomous diagnostic & verification gates", status: "ARMED", statusColor: "text-kai-green", delay: 60 },
    { text: "Configuring air-gapped local hardware runners & gateways", status: "SYNCED", statusColor: "text-kai-green", delay: 60 },
    { text: "Warming semantic memory cache & vector indexing", status: "CACHED", statusColor: "text-kai-green", delay: 70 },
    { text: "Prefetching site bundles: Desktop, Docs, Showcase, Preview", status: "100%", statusColor: "text-kai-amber", delay: 70 },
    { text: "Compiling WebGL viewport CRT & phosphor scanline shader", status: "ONLINE", statusColor: "text-kai-green", delay: 70 },
    { text: "System verification passed. Launching workspace interface...", status: "EXEC", statusColor: "text-kai-orange", delay: 80 },
];
export default function TerminalBootLoader({ onComplete, }) {
    const [currentLineIndex, setCurrentLineIndex] = React.useState(0);
    const [progress, setProgress] = React.useState(10);
    const [isFading, setIsFading] = React.useState(false);
    // Aggressively prefetch all route bundles in parallel with boot animation
    React.useEffect(() => {
        try {
            import("@/pages/Desktop");
            import("@/pages/Showcase");
            import("@/pages/Next");
            import("@/pages/docs/DocsLayout");
            import("@/pages/docs/DocsIndex");
            import("@/pages/docs/Install");
            import("@/pages/docs/Tui");
            import("@/pages/docs/CommandsDoc");
            import("@/pages/preview/PreviewLayout");
        }
        catch {
            // Background prefetching is opportunistic
        }
    }, []);
    // Fast sequential line progression
    React.useEffect(() => {
        if (currentLineIndex < BOOT_STEPS.length) {
            const step = BOOT_STEPS[currentLineIndex];
            const timer = setTimeout(() => {
                setCurrentLineIndex((prev) => prev + 1);
                setProgress(Math.round(((currentLineIndex + 1) / BOOT_STEPS.length) * 100));
            }, step.delay);
            return () => clearTimeout(timer);
        }
        else {
            // Finished all lines: quick pause before fade-out
            const finishTimer = setTimeout(() => {
                setIsFading(true);
                const exitTimer = setTimeout(onComplete, 350);
                return () => clearTimeout(exitTimer);
            }, 140);
            return () => clearTimeout(finishTimer);
        }
    }, [currentLineIndex, onComplete]);
    // Skip on keyboard interaction or click
    React.useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.key === "Escape" || e.key === "Enter" || e.key === " ") {
                setIsFading(true);
                setTimeout(onComplete, 150);
            }
        };
        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [onComplete]);
    const handleSkip = () => {
        setIsFading(true);
        setTimeout(onComplete, 150);
    };
    return (_jsx("div", { onClick: handleSkip, className: cn("fixed inset-0 z-50 flex flex-col items-center justify-center bg-[#050507] p-4 font-mono text-white transition-opacity duration-300 cursor-pointer select-none", isFading ? "opacity-0 pointer-events-none" : "opacity-100"), style: {
            backgroundImage: `
          radial-gradient(ellipse 70% 60% at 50% 50%, rgba(255, 135, 0, 0.08), transparent),
          repeating-linear-gradient(0deg, rgba(0,0,0,0.2) 0px, rgba(0,0,0,0.2) 1px, transparent 1px, transparent 2px)
        `,
        }, children: _jsxs("div", { className: "w-full max-w-2xl rounded-lg border border-kai-orange/40 bg-[#09090d]/95 p-5 shadow-[0_0_50px_rgba(255,135,0,0.15)] backdrop-blur-2xl", children: [_jsxs("div", { className: "mb-4 flex items-center justify-between border-b border-white/10 pb-3", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "size-3 rounded-full bg-red-500/80 inline-block" }), _jsx("span", { className: "size-3 rounded-full bg-yellow-500/80 inline-block" }), _jsx("span", { className: "size-3 rounded-full bg-green-500/80 inline-block" }), _jsx("span", { className: "ml-2 font-mono text-xs text-kai-muted", children: "kaioken-v2-bootloader.exe" })] }), _jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "size-2 rounded-full bg-kai-green animate-pulse" }), _jsx("span", { className: "text-[11px] text-kai-dim uppercase tracking-wider", children: "fast_boot" })] })] }), _jsxs("div", { className: "space-y-1.5 min-h-[220px] text-xs", children: [BOOT_STEPS.slice(0, currentLineIndex).map((step, idx) => (_jsxs("div", { className: "flex items-center justify-between gap-3 text-left font-mono leading-relaxed", children: [_jsxs("div", { className: "flex items-center gap-2 truncate", children: [_jsx("span", { className: "text-kai-orange select-none", children: "\u203A" }), _jsx("span", { className: idx === currentLineIndex - 1 ? "text-white font-medium" : "text-kai-muted", children: step.text })] }), step.status && (_jsxs("span", { className: cn("shrink-0 rounded bg-white/5 px-1.5 py-0.5 text-[10px] font-bold", step.statusColor || "text-kai-dim"), children: ["[", step.status, "]"] }))] }, idx))), currentLineIndex < BOOT_STEPS.length && (_jsxs("div", { className: "flex items-center gap-2 text-kai-orange", children: [_jsx("span", { className: "animate-pulse font-bold", children: "\u203A" }), _jsx("span", { className: "inline-block h-3.5 w-2 bg-kai-orange animate-pulse" })] }))] }), _jsxs("div", { className: "mt-5 border-t border-white/10 pt-4", children: [_jsxs("div", { className: "flex items-center justify-between text-[11px] text-kai-muted mb-1.5", children: [_jsx("span", { className: "font-mono", children: "PRE-CACHING ROUTE ENGINE" }), _jsxs("span", { className: "font-bold font-mono text-kai-orange", children: [progress, "%"] })] }), _jsx("div", { className: "h-1.5 w-full overflow-hidden rounded-full bg-white/10", children: _jsx("div", { className: "h-full bg-gradient-to-r from-kai-orange via-kai-amber to-kai-green transition-all duration-75", style: { width: `${progress}%` } }) })] }), _jsxs("div", { className: "mt-4 flex items-center justify-between text-[10px] text-kai-dim", children: [_jsx("span", { children: "INSTANT CACHE & ASYNC ROUTING ACTIVATED" }), _jsxs("span", { className: "hover:text-kai-amber transition-colors", children: ["Click anywhere or press ", _jsx("kbd", { className: "rounded border border-white/20 px-1", children: "ESC" }), " to skip"] })] })] }) }));
}
