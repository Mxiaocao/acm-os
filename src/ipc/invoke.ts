import { invoke as tauriInvoke, type InvokeArgs, type InvokeOptions } from "@tauri-apps/api/core";
import type { VaultContext } from "./vault-manager";
// Only binding metadata lives here. All workspace data belongs to mounted React state.
let context: VaultContext | null = null;
export function setVaultContext(next: VaultContext | null) { context = next; }
export function getVaultContext() { return context; }
const unscoped = new Set(["foundation_status", "preview_contest_ai_analysis", "open_original_oj", "desktop_e2e_context", "desktop_e2e_set_date", "desktop_e2e_log", "desktop_e2e_finish", "desktop_e2e_exit"]);
export async function invoke<T>(command: string, args?: InvokeArgs, options?: InvokeOptions): Promise<T> {
  if (unscoped.has(command)) return tauriInvoke<T>(command, args, options);
  const captured = context;
  if (!captured) throw { code: "no_active_vault" };
  const result = await tauriInvoke<T>(command, { ...args, vaultContext: captured }, options);
  if (context?.epoch !== captured.epoch || context?.vaultId !== captured.vaultId) throw { code: "stale_vault_context" };
  return result;
}
