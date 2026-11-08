import { jsx as _jsx } from "react/jsx-runtime";
import { cn } from "@/lib/utils";
/* Renders ASCII art with the site's signature left-to-right red→orange
   gradient (#ff0000 → #ff8600). Box-drawing characters drop to a 40%
   shade so the block fills lead — the same treatment as the hero logo. */
const BOX_CHARS = new Set(["╗", "║", "╔", "╝", "═", "╚"]);
function charColor(ch, x, width) {
    const g = Math.round((134 * x) / (width - 1));
    return BOX_CHARS.has(ch) ? `rgb(102, ${Math.round(g * 0.4)}, 0)` : `rgb(255, ${g}, 0)`;
}
export default function AsciiArt({ art, label, className, }) {
    const raw = art.split("\n");
    const width = Math.max(...raw.map((l) => [...l].length));
    const lines = raw.map((l) => l.padEnd(width, " "));
    return (_jsx("pre", { "aria-label": label, className: cn("inline-block text-left font-mono leading-[1.35] font-extrabold", className), children: lines.map((line, y) => (_jsx("span", { className: "block", children: [...line].map((ch, x) => (_jsx("span", { style: { color: charColor(ch, x, width) }, children: ch }, x))) }, y))) }));
}
