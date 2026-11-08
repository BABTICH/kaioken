import { invokeIpc } from "./invoke";
import type { EngineApi } from "@shared/types/ipc";
import type { EngineSupervisorSnapshot } from "@shared/types/engine";

export const engineApi: EngineApi = {
  getState: () => invokeIpc<EngineSupervisorSnapshot>("engine:getState")
};
