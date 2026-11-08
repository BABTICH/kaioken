import { jsx as _jsx } from "react/jsx-runtime";
import { cn } from "@/lib/utils";
export default function Button({ variant = "primary", size = "md", pill = true, className, children, ...props }) {
    const sizeStyles = {
        sm: "px-3.5 py-1.5 text-xs",
        md: "px-5 py-2.5 text-sm",
        lg: "px-7 py-3 text-base font-semibold",
    }[size];
    const variantStyles = {
        primary: "bg-kai-orange text-black font-semibold hover:bg-[#ffa033] shadow-[0_0_20px_rgba(255,135,0,0.3)] active:scale-[0.98]",
        secondary: "bg-kai-panel text-kai-text border border-white/10 hover:border-white/20 hover:bg-[#1a1c25] active:scale-[0.98]",
        outline: "bg-transparent text-kai-text border border-white/15 hover:border-white/30 hover:bg-white/5 active:scale-[0.98]",
        ghost: "bg-transparent text-kai-muted hover:text-kai-text hover:bg-white/5",
        violet: "bg-kai-violet text-white font-medium hover:bg-[#9d63f0] shadow-[0_0_20px_rgba(140,78,221,0.35)] active:scale-[0.98]",
    }[variant];
    return (_jsx("button", { className: cn("inline-flex items-center justify-center gap-2 transition-all duration-200 cursor-pointer select-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-kai-orange/60 disabled:opacity-50 disabled:pointer-events-none", pill ? "rounded-full" : "rounded-sm", sizeStyles, variantStyles, className), ...props, children: children }));
}
