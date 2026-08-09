import { invoke } from "@tauri-apps/api/core";

export type StartupRecoveryReasonCode =
  | "app_data_unavailable"
  | "database_unavailable"
  | "migration_ledger_invalid"
  | "unsupported_schema"
  | "migration_failed"
  | "integrity_check_failed"
  | "pre_migration_backup_failed";

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
