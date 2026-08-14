import { invoke } from "@tauri-apps/api/core";

export type WorkspacePathField = "active_vault" | "problem_root" | "knowledge_root";

export type WorkspaceConfigurationErrorCode =
  | "path_required"
  | "path_unavailable"
  | "path_not_directory"
  | "root_outside_vault"
  | "roots_overlap"
  | "already_configured"
  | "persistence_unavailable";

export type WorkspaceConfigurationErrorDto = {
  code: WorkspaceConfigurationErrorCode;
  field: WorkspacePathField | null;
};

export type WorkspaceStatusDto =
  | {
      state: "unconfigured";
      activeVaultPath: null;
      problemRootPath: null;
      knowledgeRootPath: null;
    }
  | {
      state: "configured";
      activeVaultPath: string;
      problemRootPath: string;
      knowledgeRootPath: string;
    };

export type WorkspaceConfigurationDraft = {
  activeVaultPath: string;
  problemRootPath: string;
  knowledgeRootPath: string;
};

export function getWorkspaceStatus(): Promise<WorkspaceStatusDto> {
  return invoke<WorkspaceStatusDto>("workspace_status");
}

export interface ManualBackupPreviewDto {
  schemaVersion: number;
  backupDirectory: string;
  filenamePrefix: string;
}

export interface ManualBackupResultDto {
  path: string;
  schemaVersion: number;
}

export interface BackupInventoryEntryDto {
  path: string;
  category: string;
  sizeBytes: number;
  integrityVerified: boolean;
  retention: "protected" | "keep" | "prune_candidate";
}

export interface BackupInventoryDto {
  entries: BackupInventoryEntryDto[];
  dailyKeep: number;
  weeklyKeep: number;
}

export interface SystemRestoreCandidatePreviewDto {
  sourcePath: string;
  schemaVersion: number;
  supportedSchemaVersion: number;
  migrationRequired: boolean;
  restoresSystemFacts: true;
  overwritesMarkdown: false;
}

export interface RestoreIntentPreparationDto {
  stagingPath: string;
  preRestoreSnapshotPath: string;
  candidate: SystemRestoreCandidatePreviewDto;
}

export const previewManualBackup = () => invoke<ManualBackupPreviewDto>("preview_manual_backup");
export const createManualBackup = () => invoke<ManualBackupResultDto>("create_manual_backup");
export const createWeeklyBackup = () => invoke<ManualBackupResultDto>("create_weekly_backup");
export interface BackupRetentionPreviewDto {
  protectedPaths: string[];
  pruneCandidatePaths: string[];
  dailyKeep: number;
  weeklyKeep: number;
}
export const previewBackupRetention = () =>
  invoke<BackupRetentionPreviewDto>("preview_backup_retention");
export const applyBackupRetention = (paths: string[]) =>
  invoke<number>("apply_backup_retention", { input: { paths } });
export const loadBackupInventory = () => invoke<BackupInventoryDto>("backup_inventory");
export const previewSystemRestoreCandidate = (sourcePath: string) =>
  invoke<SystemRestoreCandidatePreviewDto>("preview_system_restore_candidate", {
    input: { sourcePath },
  });
export const prepareSystemRestore = (sourcePath: string) =>
  invoke<RestoreIntentPreparationDto>("prepare_system_restore", {
    input: { sourcePath },
  });
export const restartForPendingRestore = () => invoke<void>("restart_for_pending_restore");

export function configureWorkspace(
  draft: WorkspaceConfigurationDraft,
): Promise<WorkspaceStatusDto> {
  return invoke<WorkspaceStatusDto>("configure_workspace", { draft });
}

export function describeWorkspaceError(cause: unknown): string {
  const error = parseWorkspaceConfigurationError(cause);
  if (error) {
    switch (error.code) {
      case "path_required":
        return "Choose a folder.";
      case "path_unavailable":
        return "This folder does not exist or cannot be accessed.";
      case "path_not_directory":
        return "This path must point to a folder.";
      case "root_outside_vault":
        return "This folder must be inside the Active Vault.";
      case "roots_overlap":
        return "Problem Notes Root and Knowledge Root must not be equal or contain one another.";
      case "already_configured":
        return "The workspace was already configured. Reload the application to view it.";
      case "persistence_unavailable":
        return "The workspace configuration database is unavailable.";
    }
  }
  return cause instanceof Error ? cause.message : String(cause);
}

export function parseWorkspaceConfigurationError(
  cause: unknown,
): WorkspaceConfigurationErrorDto | null {
  if (typeof cause !== "object" || cause === null) return null;
  const candidate = cause as Partial<WorkspaceConfigurationErrorDto>;
  const codes: readonly WorkspaceConfigurationErrorCode[] = [
    "path_required",
    "path_unavailable",
    "path_not_directory",
    "root_outside_vault",
    "roots_overlap",
    "already_configured",
    "persistence_unavailable",
  ];
  const fields: readonly WorkspacePathField[] = [
    "active_vault",
    "problem_root",
    "knowledge_root",
  ];
  if (
    typeof candidate.code !== "string" ||
    !codes.includes(candidate.code as WorkspaceConfigurationErrorCode) ||
    (candidate.field !== null &&
      (typeof candidate.field !== "string" ||
        !fields.includes(candidate.field as WorkspacePathField)))
  ) {
    return null;
  }
  return candidate as WorkspaceConfigurationErrorDto;
}
