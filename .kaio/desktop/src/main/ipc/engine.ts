import { handleIpc } from "./safe-handler";
import { VoidInputSchema } from "@shared/schema/ipc.zod";
import { engineSupervisor } from "../engine/supervisor";

export function registerEngineIpc(): void {
  handleIpc("engine:getState", VoidInputSchema, () => {
    return engineSupervisor.getSnapshot();
  });
}
