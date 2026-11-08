import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { NavLink, Outlet, useLocation } from "react-router-dom";
import { PanelLeft, X } from "lucide-react";
import { DOCS_NAV } from "@/data/docs-nav";
import { cn } from "@/lib/utils";
function SidebarNav({ onNavigate }) {
    return (_jsx("nav", { className: "space-y-7", children: DOCS_NAV.map((section) => (_jsxs("div", { children: [_jsx("h2", { className: "font-mono text-[10.5px] tracking-[0.25em] text-kai-dim uppercase", children: section.heading }), _jsx("ul", { className: "mt-3 space-y-0.5 border-l border-border", children: section.links.map((link) => (_jsx("li", { children: _jsx(NavLink, { to: link.to, onClick: onNavigate, className: ({ isActive }) => cn("-ml-px flex border-l-2 py-1.5 pl-3 font-mono text-[13px] transition-colors", isActive
                                ? "border-kai-orange bg-kai-orange/[0.07] text-kai-amber"
                                : "border-transparent text-muted-foreground hover:border-kai-dim hover:text-foreground"), children: link.label }) }, link.to))) })] }, section.heading))) }));
}
export default function DocsLayout() {
    const [open, setOpen] = React.useState(false);
    const { pathname } = useLocation();
    React.useEffect(() => setOpen(false), [pathname]);
    return (_jsxs("div", { className: "mx-auto max-w-6xl px-4 pt-24 sm:px-6", children: [_jsxs("button", { type: "button", onClick: () => setOpen((v) => !v), "aria-expanded": open, className: "mb-5 flex items-center gap-2 rounded-sm border border-border bg-card px-3 py-2 font-mono text-[12.5px] text-muted-foreground transition-colors hover:text-foreground lg:hidden", children: [open ? _jsx(X, { className: "size-3.5" }) : _jsx(PanelLeft, { className: "size-3.5" }), open ? "close" : "browse docs"] }), open ? (_jsx("div", { className: "mb-8 rounded-sm border border-border bg-card p-5 lg:hidden", children: _jsx(SidebarNav, { onNavigate: () => setOpen(false) }) })) : null, _jsxs("div", { className: "grid gap-12 lg:grid-cols-[13rem_1fr] lg:gap-14", children: [_jsx("aside", { className: "hidden lg:block", children: _jsx("div", { className: "sticky top-24 max-h-[calc(100vh-8rem)] overflow-y-auto pb-8", children: _jsx(SidebarNav, {}) }) }), _jsx(Outlet, {})] })] }));
}
