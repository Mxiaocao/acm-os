import { invoke } from "@tauri-apps/api/core";

export interface SystemHealthSnapshotDto {
  startupState: "ready" | "recoveryRequired";
  schemaVersion: number | null;
  pendingCriticalOperationCount: number;
  backupFileCount: number;
  pendingRestoreIntent: boolean;
  rollbackIntegrityVerified: boolean | null;
}

export function getSystemHealthSnapshot(): Promise<SystemHealthSnapshotDto> {
  return invoke<SystemHealthSnapshotDto>("system_health_snapshot");
}

export type StartupRecoveryReasonCode =
  | "app_data_unavailable"
  | "database_unavailable"
  | "migration_ledger_invalid"
  | "unsupported_schema"
  | "migration_failed"
  | "integrity_check_failed"
  | "pre_migration_backup_failed"
  | "unresolved_critical_operation";

export type StartupStatusDto =
  | {
      state: "ready";
      schemaVersion: number;
      recoveryReason: null;
      supportedSchemaVersion: number;
      foundSchemaVersion: null;
    }
  | {
      state: "recoveryRequired";
      schemaVersion: null;
      recoveryReason: StartupRecoveryReasonCode;
      supportedSchemaVersion: number | null;
      foundSchemaVersion: number | null;
    };

export function getStartupStatus(): Promise<StartupStatusDto> {
  return invoke<StartupStatusDto>("startup_status");
}

export interface RestoreDiagnosticsDto {
  pendingIntent: boolean;
  rollbackArtifactPath: string | null;
  rollbackIntegrityVerified: boolean | null;
  startupState: "ready" | "recoveryRequired";
  currentSchemaVersion: number | null;
}

export function getRestoreDiagnostics(): Promise<RestoreDiagnosticsDto> {
  return invoke<RestoreDiagnosticsDto>("restore_diagnostics");
}

export function confirmRestoreRollbackCleanup(rollbackArtifactPath: string): Promise<void> {
  return invoke<void>("confirm_restore_rollback_cleanup", { rollbackArtifactPath });
}

export interface PostRestoreRebuildPreviewDto {
  problemBindingCount: number;
  knowledgeBindingCount: number;
  derivedRelationCount: number;
  revalidatesBindings: true;
  rebuildsDerivedKnowledge: true;
  overwritesMarkdown: false;
}

export function previewPostRestoreRebuild(): Promise<PostRestoreRebuildPreviewDto> {
  return invoke<PostRestoreRebuildPreviewDto>("preview_post_restore_rebuild");
}

export interface PostRestoreBindingAnomalyDto {
  problemId: number;
  vaultRelativePath: string;
  reason: "location_anomaly" | "vault_unavailable" | "invalid_binding";
}

export interface PostRestoreProblemBindingValidationDto {
  totalCount: number;
  readyCount: number;
  anomalies: PostRestoreBindingAnomalyDto[];
}

export function validatePostRestoreProblemBindings(): Promise<PostRestoreProblemBindingValidationDto> {
  return invoke<PostRestoreProblemBindingValidationDto>("validate_post_restore_problem_bindings");
}

export interface PostRestoreKnowledgeBindingValidationDto {
  totalCount: number;
  readyCount: number;
  confirmedDeletedCount: number;
  anomalies: Array<{
    knowledgeNodeId: string;
    vaultRelativePath: string;
    reason: "location_anomaly";
  }>;
}

export function validatePostRestoreKnowledgeBindings(): Promise<PostRestoreKnowledgeBindingValidationDto> {
  return invoke<PostRestoreKnowledgeBindingValidationDto>("validate_post_restore_knowledge_bindings");
}

export interface PostRestoreRebuildPreconditionCheckDto {
  eligible: boolean;
  blockers: string[];
  problemBindingAnomalyCount: number;
  knowledgeBindingAnomalyCount: number;
}

export function checkPostRestoreRebuildPreconditions(): Promise<PostRestoreRebuildPreconditionCheckDto> {
  return invoke<PostRestoreRebuildPreconditionCheckDto>("check_post_restore_rebuild_preconditions");
}

export interface PostRestoreRebuildApplyResultDto {
  knowledgeNodeCount: number;
  relationCount: number;
  locationAnomalyCount: number;
}

export function applyPostRestoreRebuild(): Promise<PostRestoreRebuildApplyResultDto> {
  return invoke<PostRestoreRebuildApplyResultDto>("apply_post_restore_rebuild");
}

export interface DiagnosticExportPreviewDto {
  outputDirectory: string;
  sections: string[];
  privacyExclusions: string[];
  createsFiles: false;
}

export function previewDiagnosticExport(): Promise<DiagnosticExportPreviewDto> {
  return invoke<DiagnosticExportPreviewDto>("preview_diagnostic_export");
}

export interface DiagnosticExportResultDto {
  path: string;
  sections: string[];
}

export function createDiagnosticExport(): Promise<DiagnosticExportResultDto> {
  return invoke<DiagnosticExportResultDto>("create_diagnostic_export");
}
