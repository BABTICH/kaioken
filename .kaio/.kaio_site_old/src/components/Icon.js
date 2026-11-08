import { jsx as _jsx } from "react/jsx-runtime";
import { Activity, BookOpen, BookOpenText, Boxes, BrainCircuit, Check, ClipboardCheck, Code2, Cpu, FolderGit2, FolderOpen, GitMerge, Globe, Languages, Layers, Layout, Library, MessageSquare, MessageSquareCode, MonitorSmartphone, Plug, Puzzle, Radar, RefreshCw, Ruler, ScanText, SearchCode, Settings, ShieldCheck, Terminal, Users, Wallet, Waypoints, Wrench, Zap, } from "lucide-react";
/**
 * Content files reference icons by name so the copy stays free of imports.
 * Registering them explicitly keeps tree-shaking working.
 */
const REGISTRY = {
    Activity,
    BookOpen,
    BookOpenText,
    Boxes,
    BrainCircuit,
    Check,
    ClipboardCheck,
    Code2,
    Cpu,
    FolderGit2,
    FolderOpen,
    GitMerge,
    Globe,
    Languages,
    Layers,
    Layout,
    Library,
    MessageSquare,
    MessageSquareCode,
    MonitorSmartphone,
    Plug,
    Puzzle,
    Radar,
    RefreshCw,
    Ruler,
    ScanText,
    SearchCode,
    Settings,
    ShieldCheck,
    Terminal,
    Users,
    Wallet,
    Waypoints,
    Wrench,
    Zap,
};
export default function Icon({ name, ...props }) {
    const Cmp = REGISTRY[name] ?? Terminal;
    return _jsx(Cmp, { ...props });
}
