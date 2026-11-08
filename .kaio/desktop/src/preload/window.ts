import { invokeIpc } from "./invoke";
import type { WindowApi } from "@shared/types/ipc";

export const windowApi: WindowApi = {
  minimize: () => invokeIpc<void>("window:minimize"),
  maximize: () => invokeIpc<void>("window:maximize"),
  close: () => invokeIpc<void>("window:close"),
  isMaximized: () => invokeIpc<boolean>("window:isMaximized")
};
