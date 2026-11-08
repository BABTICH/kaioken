import { handleIpc } from "./safe-handler";
import { getMainWindow } from "../app/window";
import { VoidInputSchema } from "@shared/schema/ipc.zod";

export function registerWindowIpc(): void {
  handleIpc("window:minimize", VoidInputSchema, () => {
    getMainWindow()?.minimize();
  });

  handleIpc("window:maximize", VoidInputSchema, () => {
    const win = getMainWindow();

    if (!win) {
      return;
    }

    if (win.isMaximized()) {
      win.unmaximize();
    } else {
      win.maximize();
    }
  });

  handleIpc("window:close", VoidInputSchema, () => {
    getMainWindow()?.close();
  });

  handleIpc("window:isMaximized", VoidInputSchema, () => {
    return getMainWindow()?.isMaximized() ?? false;
  });
}
