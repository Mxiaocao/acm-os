import { invoke } from "@tauri-apps/api/core";
import type { StartupRecoveryReasonCode } from "./startup";
import type { WorkspaceStatusDto } from "./workspace";

export type AppShellStatusDto =
  | {
      state: "recovery";
      recoveryReason: StartupRecoveryReasonCode;
      supportedSchemaVersion: number | null;
      foundSchemaVersion: number | null;
      workspace: null;
    }
  | {
      state: "setup";
      recoveryReason: null;
      supportedSchemaVersion: null;
      foundSchemaVersion: null;
      workspace: Extract<WorkspaceStatusDto, { state: "unconfigured" }>;
    }
  | {
      state: "normal";
      recoveryReason: null;
      supportedSchemaVersion: null;
      foundSchemaVersion: null;
      workspace: Extract<WorkspaceStatusDto, { state: "configured" }>;
    };

export function getAppShellStatus(): Promise<AppShellStatusDto> {
  return invoke<AppShellStatusDto>("app_shell_status");
}
