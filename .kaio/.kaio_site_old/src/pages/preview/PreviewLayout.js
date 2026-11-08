import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { NavLink, Outlet, useLocation, useParams } from "react-router-dom";
import { ChevronRight, FolderTree, PanelLeft, X } from "lucide-react";
import { WIKI_SECTIONS, WIKI_STATS } from "@/data/wiki";
import { cn } from "@/lib/utils";
function Tree({ onNavigate }) {
    const { section: activeSection } = useParams();
    // The section containing the open document starts expanded.
    const [open, setOpen] = React.useState(() => activeSection ? { [activeSection]: true } : {});
    React.useEffect(() => {
        if (activeSection)
            setOpen((o) => ({ ...o, [activeSection]: true }));
    }, [activeSection]);
    return (_jsxs("nav", { children: [_jsxs(NavLink, { to: "/preview", end: true, onClick: onNavigate, className: ({ isActive }) => cn("flex items-center gap-2 rounded-sm px-2 py-1.5 font-mono text-[12.5px] transition-colors", isActive ? "bg-kai-orange/10 text-kai-amber" : "text-muted-foreground hover:text-foreground"), children: [_jsx(FolderTree, { className: "size-3.5" }), ".kaioken/wiki"] }), _jsxs("p", { className: "mt-1 px-2 font-mono text-[10.5px] text-kai-dim", children: [WIKI_STATS.sections, " sections \u00B7 ", WIKI_STATS.documents, " documents"] }), _jsx("ul", { className: "mt-4 space-y-0.5", children: WIKI_SECTIONS.map((section) => {
                    const expanded = open[section.slug] ?? false;
                    return (_jsxs("li", { children: [_jsxs("button", { type: "button", onClick: () => setOpen((o) => ({ ...o, [section.slug]: !expanded })), "aria-expanded": expanded, className: cn("flex w-full items-center gap-1.5 rounded-sm px-2 py-1.5 text-left font-mono text-[12px] transition-colors", activeSection === section.slug
                                    ? "text-kai-orange"
                                    : "text-muted-foreground hover:text-foreground"), children: [_jsx(ChevronRight, { className: cn("size-3 shrink-0 transition-transform", expanded && "rotate-90") }), _jsx("span", { className: "min-w-0 flex-1 truncate uppercase", children: section.title }), _jsx("span", { className: "shrink-0 rounded-sm bg-kai-panel px-1 text-[10px] text-kai-dim", children: section.docs.length })] }), expanded ? (_jsx("ul", { className: "mt-0.5 mb-1 ml-[13px] space-y-0.5 border-l border-border", children: section.docs.map((doc) => (_jsx("li", { children: _jsx(NavLink, { to: `/preview/${section.slug}/${doc.slug}`, onClick: onNavigate, className: ({ isActive }) => cn("-ml-px block border-l-2 py-1 pr-1 pl-3 font-mono text-[12px] leading-snug transition-colors", isActive
                                            ? "border-kai-orange bg-kai-orange/[0.07] text-kai-amber"
                                            : "border-transparent text-muted-foreground hover:border-kai-dim hover:text-foreground"), children: doc.title }) }, doc.slug))) })) : null] }, section.slug));
                }) })] }));
}
export default function PreviewLayout() {
    const [open, setOpen] = React.useState(false);
    const { pathname } = useLocation();
    React.useEffect(() => setOpen(false), [pathname]);
    return (_jsxs("div", { className: "mx-auto max-w-[88rem] px-4 pt-24 sm:px-6", children: [_jsxs("button", { type: "button", onClick: () => setOpen((v) => !v), "aria-expanded": open, className: "mb-5 flex items-center gap-2 rounded-sm border border-border bg-card px-3 py-2 font-mono text-[12.5px] text-muted-foreground transition-colors hover:text-foreground lg:hidden", children: [open ? _jsx(X, { className: "size-3.5" }) : _jsx(PanelLeft, { className: "size-3.5" }), open ? "close" : "browse output"] }), open ? (_jsx("div", { className: "mb-8 max-h-[70vh] overflow-y-auto rounded-sm border border-border bg-card p-4 lg:hidden", children: _jsx(Tree, { onNavigate: () => setOpen(false) }) })) : null, _jsxs("div", { className: "grid gap-10 lg:grid-cols-[17rem_1fr] lg:gap-12", children: [_jsx("aside", { className: "hidden lg:block", children: _jsx("div", { className: "sticky top-24 max-h-[calc(100vh-8rem)] overflow-y-auto pr-2 pb-8", children: _jsx(Tree, {}) }) }), _jsx(Outlet, {})] })] }));
}
