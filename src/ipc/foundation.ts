import { invoke } from "@tauri-apps/api/core";

export interface FoundationStatusDto {
  status: "ready";
  core: string;
}

export function getFoundationStatus(): Promise<FoundationStatusDto> {
  return invoke<FoundationStatusDto>("foundation_status");
}
