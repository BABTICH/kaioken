// src/renderer/lib/message-text-cache.ts

type Listener = () => void;

const texts = new Map<string, string>();
const listeners = new Map<string, Set<Listener>>();

function emit(messageId: string): void {
  const set = listeners.get(messageId);

  if (!set) {
    return;
  }

  for (const listener of set) {
    listener();
  }
}

export function getMessageText(messageId: string): string {
  return texts.get(messageId) ?? "";
}

export function appendMessageText(
  messageId: string,
  delta: string
): void {
  const next = (texts.get(messageId) ?? "") + delta;

  texts.set(messageId, next);
  emit(messageId);
}

export function setMessageText(
  messageId: string,
  text: string
): void {
  texts.set(messageId, text);
  emit(messageId);
}

export function resetMessageText(): void {
  texts.clear();
}

export function subscribeMessageText(
  messageId: string,
  listener: Listener
): () => void {
  let set = listeners.get(messageId);

  if (!set) {
    set = new Set();
    listeners.set(messageId, set);
  }

  set.add(listener);

  return () => {
    set.delete(listener);
  };
}
