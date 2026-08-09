import { invoke } from "@tauri-apps/api/core";

export interface FoundationReadyStatusDto {
  status: "ready";
  core: string;
}

export type FoundationStatus =
  | { state: "checking" }
  | { state: "ready"; foundation: FoundationReadyStatusDto }
  | { state: "unavailable" };

export function getFoundationStatus(): Promise<FoundationReadyStatusDto> {
  return invoke<FoundationReadyStatusDto>("foundation_status");
}
