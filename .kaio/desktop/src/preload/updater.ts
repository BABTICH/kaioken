// src/preload/updater.ts
import { ipcRenderer } from "electron";
import { invokeIpc } from "./invoke";
import type {
  UpdateChannel,
  UpdateState,
  UpdaterApi
} from "@shared/types/updater";

const stateListeners = new Set<(state: UpdateState) => void>();

ipcRenderer.on(
  "updater:state",
  (_event, payload: UpdateState) => {
    for (const listener of stateListeners) {
      listener(payload);
    }
  }
);

export const updaterApi: UpdaterApi = {
  getState(): Promise<UpdateState> {
    return invokeIpc<UpdateState>("updater:getState");
  },

  check(): Promise<void> {
    return invokeIpc<void>("updater:check");
  },

  download(): Promise<void> {
    return invokeIpc<void>("updater:download");
  },

  install(): Promise<void> {
    return invokeIpc<void>("updater:install");
  },

  setChannel(channel: UpdateChannel): Promise<void> {
    return invokeIpc<void>("updater:setChannel", {
      channel
    });
  },

  onState(cb: (state: UpdateState) => void) {
    stateListeners.add(cb);

    return () => {
      stateListeners.delete(cb);
    };
  }
};
