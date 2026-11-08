import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { Plus } from "lucide-react";
import { cn } from "@/lib/utils";
/**
 * The phone site's answer to dense reference content.
 *
 * Twelve surfaces, thirty-odd commands and six engineering notes are all
 * scannable as headers and unreadable as a wall — so they open one at a time.
 * Single-open on purpose: with a bottom tab bar eating 56px, two expanded
 * panels means the second one is already off-screen when it opens.
 */
export default function Accordion({ items, defaultOpen, className, }) {
    const [open, setOpen] = React.useState(defaultOpen ?? null);
    return (_jsx("div", { className: cn("divide-y divide-border overflow-hidden rounded-md border border-border", className), children: items.map((item) => {
            const isOpen = open === item.id;
            return (_jsxs("div", { className: isOpen ? "bg-kai-panel/40" : "bg-card", children: [_jsxs("button", { type: "button", onClick: () => setOpen(isOpen ? null : item.id), "aria-expanded": isOpen, "aria-controls": `acc-${item.id}`, className: "m-press flex min-h-[52px] w-full items-center gap-3 px-4 py-3 text-left", children: [item.glyph ? _jsx("span", { className: "shrink-0", children: item.glyph }) : null, _jsx("span", { className: cn("min-w-0 flex-1 font-mono text-[13.5px] leading-snug font-bold", isOpen ? "text-kai-orange" : "text-foreground"), children: item.title }), item.meta ? _jsx("span", { className: "shrink-0", children: item.meta }) : null, _jsx(Plus, { className: cn("size-4 shrink-0 text-kai-dim transition-transform duration-200", isOpen && "rotate-45 text-kai-orange"), "aria-hidden": true })] }), isOpen ? (_jsx("div", { id: `acc-${item.id}`, className: "m-expand px-4 pb-4", children: item.body })) : null] }, item.id));
        }) }));
}
/** The bullet list that fills most accordion bodies. */
export function Points({ items, tone = "text-kai-orange" }) {
    return (_jsx("ul", { className: "space-y-2", children: items.map((p) => (_jsxs("li", { className: "flex gap-2.5", children: [_jsx("span", { className: cn("mt-px shrink-0 font-mono text-[11px]", tone), "aria-hidden": true, children: "\u25B8" }), _jsx("span", { className: "font-sans text-[12.5px] leading-[1.6] text-muted-foreground", children: p })] }, p))) }));
}
