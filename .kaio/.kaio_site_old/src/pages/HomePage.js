import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import { Monitor, ArrowRight } from "lucide-react";
import Hero from "@/components/sections/Hero";
import LogoBar from "@/components/sections/LogoBar";
import StatsBar from "@/components/sections/StatsBar";
import FeatureListPreview from "@/components/sections/FeatureListPreview";
import ProblemSolution from "@/components/sections/ProblemSolution";
import ModelMatrix from "@/components/sections/ModelMatrix";
import ModesSection from "@/components/sections/ModesSection";
import ArchitectureFlow from "@/components/sections/ArchitectureFlow";
import Multiplier from "@/components/sections/Multiplier";
import OutputTree from "@/components/sections/OutputTree";
import AppWindow from "@/components/desktop/AppWindow";
import LinkButton from "@/components/LinkButton";
import SectionHeading from "@/components/SectionHeading";
import Reveal from "@/components/Reveal";
import PageBackground from "@/components/PageBackground";
import FaqSection from "@/components/sections/FaqSection";
import FinalCta from "@/components/sections/FinalCta";
export default function HomePage({ isDesktop = true }) {
    return (_jsxs(_Fragment, { children: [_jsx(PageBackground, { variant: "full" }), _jsx(Hero, { isDesktop: isDesktop }), _jsx(LogoBar, {}), _jsx(StatsBar, {}), _jsx(FeatureListPreview, {}), _jsx(ProblemSolution, {}), _jsx(ModelMatrix, {}), _jsx(ModesSection, {}), _jsx(ArchitectureFlow, {}), _jsx(Multiplier, {}), _jsx(OutputTree, {}), _jsx("section", { className: "relative border-t border-white/10 py-20 sm:py-24 bg-[#08080c]/60 backdrop-blur-md", children: _jsxs("div", { className: "mx-auto max-w-6xl px-4 sm:px-6", children: [_jsx(Reveal, { children: _jsx(SectionHeading, { index: "08", eyebrow: "desktop studio", title: _jsxs(_Fragment, { children: ["Same engine,", " ", _jsx("span", { className: "text-kai-orange glow-orange", children: "native desktop surface" })] }), description: "Everything the TUI does \u2014 chat, research, wiki, knowledge cards \u2014 in a multi-pane window with live diff approvals and concurrent task execution.", align: "center" }) }), _jsx("div", { className: "mx-auto mt-10 max-w-4xl", children: _jsx("div", { className: "rounded-lg border border-white/10 bg-[#050507]/90 p-px shadow-2xl", children: _jsx(AppWindow, { size: "sm", start: "chat" }) }) }), _jsx("div", { className: "mt-8 flex flex-wrap justify-center gap-3", children: _jsxs(LinkButton, { to: "/desktop", size: "lg", className: "rounded-md", children: [_jsx(Monitor, { className: "size-4", "data-icon": "inline-start" }), "Explore Desktop Studio", _jsx(ArrowRight, { "data-icon": "inline-end" })] }) })] }) }), _jsx(FaqSection, {}), _jsx(FinalCta, {})] }));
}
