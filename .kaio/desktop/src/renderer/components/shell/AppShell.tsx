// src/renderer/components/shell/AppShell.tsx
import { useEffect, useRef } from "react";
import { useStore } from "@nanostores/react";
import { TitleBar } from "./TitleBar";
import { NavRail } from "./NavRail";
import { SidebarPanel } from "./SidebarPanel";
import { InspectorPanel } from "./InspectorPanel";
import { TerminalDrawer } from "./TerminalDrawer";
import { StatusBar } from "./StatusBar";
import { CommandPalette } from "../command-palette/CommandPalette";
import { FileTreeContextMenu } from "@renderer/components/explorer/FileTreeContextMenu";
import { ApprovalModal } from "@renderer/components/approval/ApprovalModal";
import { initApprovalBridge } from "@renderer/stores/approval";
import { ReleaseNotesModal } from "@renderer/components/system/ReleaseNotesModal";
import { initUpdaterBridge } from "@renderer/stores/updater";
import { initDiagnosticsBridge } from "@renderer/stores/diagnostics";
import { PrimarySurfaceRouter } from "@renderer/app/router";
import { useGlobalHotkeys } from "@renderer/hooks/useGlobalHotkeys";
import { useAgentEventStream } from "@renderer/hooks/useAgentEventStream";
import { useTerminalBridge } from "@renderer/hooks/useTerminalBridge";
import { restoreWorkspace } from "@renderer/stores/explorer";
import { terminalOpen } from "@renderer/stores/ui";
import { setTerminalHost } from "@renderer/lib/terminal/session-manager";

export function AppShell() {
  useGlobalHotkeys();
  useAgentEventStream();
  useTerminalBridge();

  const terminalHostRef = useRef<HTMLDivElement | null>(null);
  const isTerminalOpen = useStore(terminalOpen);

  useEffect(() => {
    initApprovalBridge();
    initUpdaterBridge();
    initDiagnosticsBridge();
    setTerminalHost(terminalHostRef.current);
    void restoreWorkspace();
  }, []);

  return (
    <div
      className="tech-grid-bg relative flex h-screen w-screen overflow-hidden flex-col"
      data-app-shell="true"
    >
      <div className="relative z-10 flex h-full w-full flex-col">
        <TitleBar />

        <div className="flex flex-1 overflow-hidden">
          {/* Left nav rail */}
          <NavRail />

          {/* Left sidebar: file explorer (moved from right) */}
          <SidebarPanel />

          {/* Main content area */}
          <main className="flex flex-1 flex-col overflow-hidden">
            <div className="flex flex-1 overflow-hidden">
              <PrimarySurfaceRouter />
            </div>

            {/* Terminal drawer */}
            <div
              style={{
                display: isTerminalOpen ? "block" : "none"
              }}
            >
              <TerminalDrawer />
            </div>
          </main>

          {/* Right inspector panel */}
          <InspectorPanel />
        </div>

        {/* VSCode-style status bar at the very bottom */}
        <StatusBar />
      </div>

      <CommandPalette />
      <FileTreeContextMenu />
      <ApprovalModal />
      <ReleaseNotesModal />

      <div
        ref={terminalHostRef}
        style={{
          display: "none"
        }}
      />
    </div>
  );
}
