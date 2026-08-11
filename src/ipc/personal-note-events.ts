import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function onPersonalNoteInvalidated(callback: () => void): Promise<UnlistenFn> {
  return listen("personal-note-invalidated", callback);
}
