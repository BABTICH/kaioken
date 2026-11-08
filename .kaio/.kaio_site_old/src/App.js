import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import * as React from "react";
import { useLayout } from "@/lib/viewport";
import TerminalBootLoader from "@/components/terminal/TerminalBootLoader";
const DesktopApp = React.lazy(() => import("@/DesktopApp"));
const MobileApp = React.lazy(() => import("@/mobile/MobileApp"));
function BootFallback() {
    return (_jsx("div", { className: "flex min-h-screen items-center justify-center bg-[#050507]", children: _jsxs("p", { className: "font-mono text-sm text-kai-muted", children: [_jsx("span", { className: "inline-block size-2 bg-kai-orange animate-pulse mr-2" }), "kaioken v2"] }) }));
}
export default function App() {
    const layout = useLayout();
    const [booted, setBooted] = React.useState(false);
    return (_jsxs(_Fragment, { children: [!booted && (_jsx(TerminalBootLoader, { onComplete: () => setBooted(true) })), _jsx(React.Suspense, { fallback: _jsx(BootFallback, {}), children: layout === "mobile" ? _jsx(MobileApp, {}) : _jsx(DesktopApp, {}) })] }));
}
