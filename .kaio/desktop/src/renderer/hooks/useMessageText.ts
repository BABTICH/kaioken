// src/renderer/hooks/useMessageText.ts
import { useSyncExternalStore } from "react";
import {
  getMessageText,
  subscribeMessageText
} from "@renderer/lib/message-text-cache";

export function useMessageText(messageId: string): string {
  return useSyncExternalStore(
    (listener) => subscribeMessageText(messageId, listener),
    () => getMessageText(messageId)
  );
}
