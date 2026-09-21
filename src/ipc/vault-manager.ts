import { invoke } from "@tauri-apps/api/core";
import { setVaultContext } from "./invoke";
export interface VaultContext { vaultId: string; epoch: string }
export interface VaultSummary { id: string; name: string; rootPath: string; databasePath: string; createdAtUtc: string; isCurrent: boolean }
export type VaultErrorCode = "invalid_name" | "root_missing" | "root_nested" | "duplicate_root" | "legacy_protected" | "existing_data" | "vault_not_found" | "vault_unavailable" | "current_vault_delete_forbidden" | "confirmation_mismatch" | "invalid_confirmation_token" | "stale_vault_context" | "no_active_vault" | "registry_unavailable" | "registry_corrupt" | "manager_in_use" | "persistence_unavailable";
export interface VaultError { code: VaultErrorCode }
export type CurrentVaultState = { state: "none" } | { state: "ready"; vault: VaultSummary } | { state: "unavailable"; vault: VaultSummary; reason: VaultErrorCode };
export interface ManagerStatus { vaults: VaultSummary[]; current: CurrentVaultState; context: VaultContext | null }
export interface CreateVaultRequest { name: string; rootPath: string }
export interface VaultRequest { vaultId: string }
export interface DeleteVaultRequest extends VaultRequest { token: string; confirmation: string }
export interface DeletePreview extends VaultRequest { name: string; rootPath: string; retainedDataPath: string; confirmationText: string; token: string }
export const getVaultManagerStatus = () => invoke<ManagerStatus>("vault_manager_status");
export const createVault = (input: CreateVaultRequest) => invoke<VaultSummary>("create_vault", { input });
export async function openVault(input: VaultRequest): Promise<ManagerStatus> {
  const result = await invoke<ManagerStatus>("open_vault", { input });
  setVaultContext(null);
  window.dispatchEvent(new Event("vault-context-changed"));
  return result;
}
export const previewDeleteVault = (input: VaultRequest) => invoke<DeletePreview>("preview_delete_vault", { input });
export const deleteVault = (input: DeleteVaultRequest) => invoke<ManagerStatus>("delete_vault", { input });
export function vaultErrorMessage(error: unknown): string {
  const code = typeof error === "object" && error !== null && "code" in error ? String(error.code) : String(error);
  const messages: Record<string, string> = {
    invalid_name: "请输入 1–80 字的 Vault 名称。", root_missing: "请选择已存在的文件夹。",
    root_nested: "Vault 文件夹不能互相包含。", duplicate_root: "该文件夹已注册。",
    legacy_protected: "该文件夹包含旧版数据或应用数据，请选择其他文件夹。",
    existing_data: "该位置已有管理数据；为保护已有内容，请选择其他文件夹。",
    current_vault_delete_forbidden: "请先打开另一个 Vault，再删除此 Vault。",
    confirmation_mismatch: "确认文本不匹配，请重新预览删除。", invalid_confirmation_token: "删除预览已失效，请重新预览。",
    stale_vault_context: "Vault 已切换，此操作已阻止。", registry_corrupt: "Vault 注册表损坏，已停止数据访问。",
    vault_unavailable: "工作区路径或数据库不可用。", manager_in_use: "另一个应用窗口正在管理工作区，请关闭后重试。",
    vault_not_found: "找不到该工作区，请刷新列表后重试。",
    no_active_vault: "请先打开一个工作区。",
    registry_unavailable: "暂时无法读取工作区列表，请检查文件夹权限后重试。",
    persistence_unavailable: "暂时无法保存更改，请检查磁盘空间和文件夹权限后重试。",
  };
  return messages[code] ?? "操作未完成，请稍后重试。";
}
