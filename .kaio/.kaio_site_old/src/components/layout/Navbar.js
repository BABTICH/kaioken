import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { NavLink, Link, useLocation } from "react-router-dom";
import { Menu, X } from "lucide-react";
import { GithubMark } from "@/components/ui/BrandIcons";
import { GITHUB_URL, VERSION } from "@/data/content";
import Button from "@/components/ui/Button";
import { cn } from "@/lib/utils";
const NAV_ROUTES = [
    { label: "Home", to: "/", end: true },
    { label: "Desktop", to: "/desktop" },
    { label: "Docs", to: "/docs" },
    { label: "Preview", to: "/preview" },
    { label: "Showcase", to: "/showcase" },
    { label: "Roadmap", to: "/next" },
];
const prefetchRoute = (to) => {
    switch (to) {
        case "/desktop":
            import("@/pages/Desktop");
            break;
        case "/docs":
            import("@/pages/docs/DocsLayout");
            import("@/pages/docs/DocsIndex");
            break;
        case "/preview":
            import("@/pages/preview/PreviewLayout");
            import("@/pages/preview/PreviewIndex");
            break;
        case "/showcase":
            import("@/pages/Showcase");
            break;
        case "/next":
            import("@/pages/Next");
            break;
    }
};
export default function Navbar() {
    const [scrolled, setScrolled] = React.useState(false);
    const [mobileMenuOpen, setMobileMenuOpen] = React.useState(false);
    const { pathname } = useLocation();
    React.useEffect(() => {
        setMobileMenuOpen(false);
    }, [pathname]);
    React.useEffect(() => {
        const handleScroll = () => {
            setScrolled(window.scrollY > 20);
        };
        window.addEventListener("scroll", handleScroll, { passive: true });
        return () => window.removeEventListener("scroll", handleScroll);
    }, []);
    return (_jsxs("header", { className: cn("sticky top-0 z-40 w-full transition-all duration-300", scrolled
            ? "border-b border-white/10 bg-[#050507]/85 backdrop-blur-2xl shadow-xl"
            : "border-b border-transparent bg-transparent"), children: [_jsxs("div", { className: "mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6", children: [_jsxs(Link, { to: "/", className: "flex items-center gap-2.5 group", children: [_jsx("img", { src: "/logo.svg", alt: "Kaioken", className: "size-6 transition-transform duration-300 group-hover:scale-110" }), _jsx("span", { className: "font-mono text-base font-bold tracking-tight text-white", children: "kaioken" }), _jsx("span", { className: "rounded-full border border-kai-orange/40 bg-kai-orange/10 px-2 py-0.5 font-mono text-[10px] font-semibold text-kai-orange", children: VERSION })] }), _jsx("nav", { className: "hidden items-center gap-1 md:flex", children: NAV_ROUTES.map((route) => (_jsx(NavLink, { to: route.to, end: route.end, onMouseEnter: () => prefetchRoute(route.to), onFocus: () => prefetchRoute(route.to), className: ({ isActive }) => cn("rounded-full px-3.5 py-1.5 font-mono text-[12.5px] transition-all duration-200", isActive
                                ? "bg-white/10 text-white font-semibold shadow-inner"
                                : "text-kai-muted hover:bg-white/5 hover:text-white"), children: route.label }, route.to))) }), _jsxs("div", { className: "hidden items-center gap-3 sm:flex", children: [_jsxs("a", { href: GITHUB_URL, target: "_blank", rel: "noopener noreferrer", className: "flex items-center gap-1.5 rounded-full border border-white/10 bg-white/5 px-3.5 py-1.5 font-mono text-xs text-kai-text transition-colors hover:border-white/20 hover:text-white", children: [_jsx(GithubMark, { className: "size-3.5" }), _jsx("span", { children: "GitHub" })] }), _jsx(Link, { to: "/docs/install", children: _jsx(Button, { size: "sm", variant: "primary", pill: true, children: "Get Started" }) })] }), _jsx("button", { onClick: () => setMobileMenuOpen(!mobileMenuOpen), className: "flex size-9 items-center justify-center rounded-md text-kai-muted hover:text-white md:hidden cursor-pointer", "aria-label": "Toggle menu", children: mobileMenuOpen ? _jsx(X, { className: "size-5" }) : _jsx(Menu, { className: "size-5" }) })] }), mobileMenuOpen && (_jsx("div", { className: "border-b border-white/10 bg-[#08080a] px-4 py-5 md:hidden", children: _jsxs("nav", { className: "flex flex-col gap-2", children: [NAV_ROUTES.map((route) => (_jsx(NavLink, { to: route.to, end: route.end, onClick: () => setMobileMenuOpen(false), className: ({ isActive }) => cn("rounded-md px-3 py-2 font-mono text-sm transition-colors", isActive
                                ? "bg-kai-orange/15 text-kai-orange font-semibold"
                                : "text-kai-muted hover:bg-white/5 hover:text-white"), children: route.label }, route.to))), _jsxs("div", { className: "mt-3 flex flex-col gap-2 pt-3 border-t border-white/10", children: [_jsxs("a", { href: GITHUB_URL, target: "_blank", rel: "noopener noreferrer", className: "flex items-center justify-center gap-2 rounded-md border border-white/10 py-2 font-mono text-xs text-white", children: [_jsx(GithubMark, { className: "size-4" }), _jsx("span", { children: "GitHub Repository" })] }), _jsx(Link, { to: "/docs/install", onClick: () => setMobileMenuOpen(false), children: _jsx(Button, { size: "md", variant: "primary", className: "w-full", children: "Get Started" }) })] })] }) }))] }));
}
