import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { Check, Copy, Terminal } from "lucide-react";
import { INSTALL_COMMANDS } from "@/data/content";
import { cn } from "@/lib/utils";
export default function CopyCommandBox() {
    const [activeTab, setActiveTab] = React.useState("curl");
    const [copied, setCopied] = React.useState(false);
    const currentCommand = INSTALL_COMMANDS[activeTab];
    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(currentCommand);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
        catch {
            // ignore clipboard error
        }
    };
    return (_jsxs("div", { className: "mx-auto flex w-full max-w-xl flex-col items-center gap-2", children: [_jsx("div", { className: "flex items-center gap-1 rounded-full border border-white/10 bg-[#0d0e14]/80 p-1 backdrop-blur-md", children: Object.keys(INSTALL_COMMANDS).map((tab) => (_jsx("button", { onClick: () => setActiveTab(tab), className: cn("rounded-full px-3 py-1 font-mono text-[11px] font-medium transition-all cursor-pointer", activeTab === tab
                        ? "bg-white/15 text-white shadow-sm"
                        : "text-kai-muted hover:text-white"), children: tab }, tab))) }), _jsxs("div", { className: "group relative flex w-full items-center justify-between overflow-hidden rounded-full border border-white/15 bg-[#0a0b10] px-4 py-2.5 shadow-xl transition-all duration-300 hover:border-kai-orange/40 hover:shadow-[0_0_25px_rgba(255,135,0,0.15)]", children: [_jsxs("div", { className: "flex items-center gap-2.5 overflow-hidden", children: [_jsx(Terminal, { className: "size-4 shrink-0 text-kai-orange" }), _jsx("span", { className: "select-all truncate font-mono text-[12.5px] text-kai-text sm:text-[13.5px]", children: currentCommand })] }), _jsx("button", { onClick: handleCopy, "aria-label": "Copy install command", className: "ml-3 flex size-7 shrink-0 items-center justify-center rounded-full bg-white/10 text-kai-muted transition-all hover:bg-kai-orange hover:text-black cursor-pointer active:scale-95", children: copied ? _jsx(Check, { className: "size-3.5 text-kai-green" }) : _jsx(Copy, { className: "size-3.5" }) })] }), _jsx("span", { className: "font-mono text-[11px] text-kai-dim", children: copied ? "✓ Copied to clipboard!" : "Self-updating single binary · 0 configuration required" })] }));
}
