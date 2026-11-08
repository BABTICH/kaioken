import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeSlug from "rehype-slug";
import { Link } from "react-router-dom";
import Mermaid from "./Mermaid";
/** Matches the slug rule in scripts/gen-wiki-manifest.mjs. */
const slug = (s) => s
    .toLowerCase()
    .replace(/\.md$/, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
/**
 * Kaioken rewrites cross-chapter mentions into relative .md links. Turn those
 * into routes inside the preview instead of dead file links.
 */
function resolveHref(href, sectionDir) {
    if (/^https?:/i.test(href))
        return { external: href };
    if (href.startsWith("#"))
        return { external: href };
    const clean = href.replace(/^\.\//, "");
    if (!clean.endsWith(".md"))
        return { external: href };
    const parts = clean.split("/");
    if (parts.length === 1) {
        // sibling document inside the same section
        return { to: `/preview/${slug(sectionDir)}/${slug(parts[0])}` };
    }
    const [dir, file] = parts.slice(-2);
    return { to: `/preview/${slug(dir)}/${slug(file)}` };
}
export default function Markdown({ children, sectionDir = "", }) {
    const components = React.useMemo(() => ({
        h1: ({ children }) => (_jsx("h1", { className: "mt-0 font-mono text-2xl font-bold tracking-tight text-foreground sm:text-3xl", children: children })),
        h2: ({ children, ...props }) => (_jsx("h2", { ...props, className: "scroll-mt-24 border-b border-border pt-10 pb-2 font-mono text-xl font-bold text-kai-orange", children: children })),
        h3: ({ children, ...props }) => (_jsx("h3", { ...props, className: "scroll-mt-24 pt-7 font-mono text-[16px] font-bold text-kai-amber", children: children })),
        h4: ({ children, ...props }) => (_jsx("h4", { ...props, className: "scroll-mt-24 pt-5 font-mono text-[14px] font-bold text-foreground", children: children })),
        p: ({ children }) => (_jsx("p", { className: "pt-4 font-sans text-[15px] leading-[1.75] text-muted-foreground", children: children })),
        ul: ({ children }) => _jsx("ul", { className: "mt-4 space-y-2 pl-1", children: children }),
        ol: ({ children }) => (_jsx("ol", { className: "mt-4 list-decimal space-y-2 pl-5 marker:font-mono marker:text-kai-orange", children: children })),
        li: ({ children }) => (_jsx("li", { className: "font-sans text-[15px] leading-[1.7] text-muted-foreground", children: children })),
        strong: ({ children }) => (_jsx("strong", { className: "font-semibold text-foreground", children: children })),
        em: ({ children }) => _jsx("em", { className: "text-kai-tan italic", children: children }),
        hr: () => _jsx("hr", { className: "my-10 border-border" }),
        blockquote: ({ children }) => (_jsx("blockquote", { className: "mt-5 border-l-2 border-kai-orange/50 bg-kai-panel/40 py-1 pl-4", children: children })),
        a: ({ href, children }) => {
            if (!href)
                return _jsx("span", { children: children });
            const r = resolveHref(href, sectionDir);
            if (r.to) {
                return (_jsx(Link, { to: r.to, className: "text-kai-orange underline decoration-kai-orange/40 underline-offset-2 transition-colors hover:text-kai-amber", children: children }));
            }
            const isAnchor = r.external?.startsWith("#");
            return (_jsx("a", { href: r.external, ...(isAnchor ? {} : { target: "_blank", rel: "noopener noreferrer" }), className: "text-kai-orange underline decoration-kai-orange/40 underline-offset-2 transition-colors hover:text-kai-amber", children: children }));
        },
        table: ({ children }) => (_jsx("div", { className: "mt-5 overflow-x-auto rounded-sm border border-border", children: _jsx("table", { className: "w-full border-collapse text-left", children: children }) })),
        thead: ({ children }) => _jsx("thead", { className: "bg-kai-panel", children: children }),
        th: ({ children }) => (_jsx("th", { className: "border-b border-border px-4 py-2.5 font-mono text-[11px] tracking-wider text-kai-amber uppercase", children: children })),
        td: ({ children }) => (_jsx("td", { className: "border-b border-border px-4 py-2.5 align-top font-sans text-[13.5px] text-muted-foreground last:border-r-0", children: children })),
        pre: ({ children }) => _jsx(_Fragment, { children: children }),
        code: ({ className, children, ...props }) => {
            const text = String(children ?? "");
            const lang = /language-(\w+)/.exec(className ?? "")?.[1];
            // Inline code has no language class and no trailing newline.
            if (!lang && !text.includes("\n")) {
                return (_jsx("code", { className: "rounded-sm bg-kai-panel px-1.5 py-0.5 font-mono text-[13px] text-kai-amber", ...props, children: children }));
            }
            if (lang === "mermaid")
                return _jsx(Mermaid, { chart: text.trimEnd() });
            return (_jsxs("div", { className: "mt-5 overflow-hidden rounded-sm border border-border bg-card", children: [_jsxs("div", { className: "flex items-center gap-2 border-b border-border bg-kai-panel px-3 py-1.5", children: [_jsx("span", { className: "size-2 rounded-[1px] bg-kai-orange/60" }), _jsx("span", { className: "font-mono text-[10.5px] tracking-wider text-kai-dim uppercase", children: lang ?? "text" })] }), _jsx("pre", { className: "overflow-x-auto p-4", children: _jsx("code", { className: "font-mono text-[12.5px] leading-relaxed text-foreground", children: text.replace(/\n$/, "") }) })] }));
        },
    }), [sectionDir]);
    return (_jsx(ReactMarkdown, { remarkPlugins: [remarkGfm], rehypePlugins: [rehypeSlug], components: components, children: children }));
}
