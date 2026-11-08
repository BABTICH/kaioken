// src/renderer/app/router.tsx
import { useStore } from "@nanostores/react";
import { activeSurface } from "@renderer/stores/ui";
import { SurfacePlaceholder } from "@renderer/components/surfaces/SurfacePlaceholder";
import { ChatSurface } from "@renderer/components/chat/ChatSurface";
import { SettingsSurface } from "@renderer/components/settings/SettingsSurface";
import { CodemapSurface } from "@renderer/components/graph/CodemapSurface";
import { WikiSurface } from "@renderer/components/wiki/WikiSurface";
import { CardsSurface } from "@renderer/components/cards/CardsSurface";
import { CostSurface } from "@renderer/components/cost/CostSurface";
import { ResearchSurface } from "@renderer/components/research/ResearchSurface";
import { WelcomeSurface } from "@renderer/components/welcome/WelcomeSurface";

export function PrimarySurfaceRouter() {
  const surface = useStore(activeSurface);

  switch (surface) {
    case "welcome": {
      return <WelcomeSurface />;
    }

    case "chat": {
      return <ChatSurface />;
    }

    case "research": {
      return <ResearchSurface />;
    }

    case "wiki": {
      return <WikiSurface />;
    }

    case "codemap": {
      return <CodemapSurface />;
    }

    case "cards": {
      return <CardsSurface />;
    }

    case "cost": {
      return <CostSurface />;
    }

    case "settings": {
      return <SettingsSurface />;
    }

    default: {
      return (
        <SurfacePlaceholder
          index="ERR"
          title="Unknown Surface"
          phase="ERROR"
          description="The requested navigation surface does not exist."
        />
      );
    }
  }
}
