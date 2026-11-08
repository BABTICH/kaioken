import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { useInView, useReducedMotion } from "@/lib/motion";
import { cn } from "@/lib/utils";
/** Mirrors the TUI's lipgloss styles one-for-one. */
const STYLE = {
    user: "text-kai-blue font-semibold",
    assistant: "text-foreground",
    tool: "text-kai-tan",
    toolres: "text-kai-sage",
    add: "text-kai-green",
    del: "text-kai-rose",
    approve: "text-kai-amber font-semibold",
    ok: "text-kai-green",
    blank: "",
};
/**
 * A phone-width transcript. Every line is under 43 characters, which is what
 * fits at 11px on a 360px screen without a horizontal scroll — the desktop
 * demo's lines are written for a 3xl column and wrap into mush here.
 */
const SCRIPT = [
    { kind: "user", text: "› wrap the validate error", hold: 400 },
    { kind: "blank", text: "" },
    { kind: "tool", text: "● read_knowledge" },
    { kind: "toolres", text: "  ↳ 1 skill matched" },
    { kind: "tool", text: "● read_file  api/handler.go" },
    { kind: "toolres", text: "  ↳ 148 lines", hold: 300 },
    { kind: "blank", text: "" },
    { kind: "assistant", text: "The bare return drops the cause.", hold: 250 },
    { kind: "blank", text: "" },
    { kind: "tool", text: "● edit: api/handler.go" },
    { kind: "del", text: "-   return nil" },
    { kind: "add", text: '+   return fmt.Errorf("%w", err)', hold: 400 },
    { kind: "blank", text: "" },
    { kind: "approve", text: "apply edit?  [y] yes  [n] no", hold: 800 },
    { kind: "ok", text: "✓ applied · 1 file changed" },
];
const LINE_DELAY = 320;
export default function TerminalPeek({ className }) {
    const ref = React.useRef(null);
    const inView = useInView(ref, "-20px");
    const reduced = useReducedMotion();
    const [shown, setShown] = React.useState(0);
    const [done, setDone] = React.useState(false);
    React.useEffect(() => {
        if (reduced) {
            setShown(SCRIPT.length);
            setDone(true);
            return;
        }
        // Nothing types while the card is off-screen: on a phone that is most of
        // the page, and a timer chain running behind the fold is battery for free.
        if (!inView || done)
            return;
        let cancelled = false;
        let timer;
        const advance = (i) => {
            if (cancelled)
                return;
            if (i >= SCRIPT.length) {
                setDone(true);
                return;
            }
            setShown(i + 1);
            timer = setTimeout(() => advance(i + 1), LINE_DELAY + (SCRIPT[i].hold ?? 0));
        };
        timer = setTimeout(() => advance(0), 350);
        return () => {
            cancelled = true;
            clearTimeout(timer);
        };
    }, [inView, reduced, done]);
    return (_jsxs("div", { ref: ref, className: cn("overflow-hidden rounded-md border border-border bg-card panel-glow", className), children: [_jsxs("div", { className: "flex items-center gap-2 border-b border-border bg-kai-panel px-3 py-2", children: [_jsx("span", { className: "size-2 rounded-[1px] bg-kai-red/70", "aria-hidden": true }), _jsx("span", { className: "size-2 rounded-[1px] bg-kai-amber/70", "aria-hidden": true }), _jsx("span", { className: "size-2 rounded-[1px] bg-kai-green/70", "aria-hidden": true }), _jsx("span", { className: "ml-1 truncate font-mono text-[10.5px] text-muted-foreground", children: "kaioken \u2014 tui" })] }), _jsx("div", { className: "crt-scanlines relative", children: _jsxs("div", { className: "m-no-scrollbar min-h-[15.5rem] overflow-x-auto px-3.5 py-3 font-mono text-[11px] leading-[1.55]", "aria-live": "off", children: [SCRIPT.slice(0, shown).map((line, i) => (_jsx("div", { className: cn("animate-rise whitespace-pre", STYLE[line.kind], line.kind === "blank" && "h-[0.75em]"), children: line.text }, i))), done ? (_jsxs("div", { className: "mt-1 flex items-center gap-2", children: [_jsx("span", { className: "text-kai-orange", children: "\u276F" }), _jsx("span", { className: "animate-caret inline-block h-[1.1em] w-[0.6ch] bg-kai-orange" })] })) : null] }) })] }));
}
