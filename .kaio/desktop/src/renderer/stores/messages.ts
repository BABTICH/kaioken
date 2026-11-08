// src/renderer/stores/messages.ts
import { atom } from "nanostores";
import type { AgentEvent } from "@shared/types/agent";
import {
  appendMessageText,
  resetMessageText,
  setMessageText
} from "@renderer/lib/message-text-cache";

export interface MessageMeta {
  id: string;
  role: "user" | "assistant" | "system" | "tool";
  createdAt: number;
  status: "streaming" | "complete";
  toolIds: string[];
}

export interface MessagesState {
  orderedIds: string[];
  byId: Record<string, MessageMeta>;
  activeMessageId: string | null;
}

const initialState: MessagesState = {
  orderedIds: [],
  byId: {},
  activeMessageId: null
};

export const messageStore = atom<MessagesState>(initialState);

function set(patch: Partial<MessagesState>): void {
  messageStore.set({
    ...messageStore.get(),
    ...patch
  });
}

export function resetMessages(): void {
  messageStore.set({
    orderedIds: [],
    byId: {},
    activeMessageId: null
  });

  resetMessageText();
}

function ensureMessage(
  messageId: string,
  role: MessageMeta["role"],
  createdAt: number
): void {
  const state = messageStore.get();

  if (state.byId[messageId]) {
    return;
  }

  const message: MessageMeta = {
    id: messageId,
    role,
    createdAt,
    status: "streaming",
    toolIds: []
  };

  set({
    byId: {
      ...state.byId,
      [messageId]: message
    },
    orderedIds: [...state.orderedIds, messageId],
    activeMessageId: messageId
  });
}

function patchMessage(
  messageId: string,
  patch: Partial<MessageMeta>
): void {
  const state = messageStore.get();
  const existing = state.byId[messageId];

  if (!existing) {
    return;
  }

  set({
    byId: {
      ...state.byId,
      [messageId]: {
        ...existing,
        ...patch
      }
    }
  });
}

function addToolToMessage(
  messageId: string,
  toolId: string
): void {
  ensureMessage(messageId, "assistant", Date.now());

  const state = messageStore.get();
  const message = state.byId[messageId];

  if (!message) {
    return;
  }

  if (message.toolIds.includes(toolId)) {
    return;
  }

  patchMessage(messageId, {
    toolIds: [...message.toolIds, toolId]
  });
}

export function applyMessageEvent(event: AgentEvent): void {
  switch (event.type) {
    case "message.created": {
      ensureMessage(event.messageId, event.role, event.createdAt);
      return;
    }

    case "message.delta": {
      ensureMessage(event.messageId, "assistant", Date.now());
      appendMessageText(event.messageId, event.deltaText);
      return;
    }

    case "message.completed": {
      ensureMessage(event.messageId, "assistant", Date.now());
      setMessageText(event.messageId, event.fullText);
      patchMessage(event.messageId, {
        status: "complete"
      });
      return;
    }

    case "tool.proposed": {
      addToolToMessage(event.messageId, event.tool.id);
      return;
    }

    default: {
      return;
    }
  }
}
