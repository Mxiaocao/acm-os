import { invoke } from "@tauri-apps/api/core";

export interface ContestShelfItemDto {
  contestId: number;
  title: string;
  importStatus: "incomplete" | "complete";
  problemCount: number;
  missingSnapshotCount: number;
}

export interface ContestDetailDto {
  contestId: number;
  title: string;
  sourceUrl: string;
  importStatus: "incomplete" | "complete";
  problems: LightweightProblemItemDto[];
}

export interface LightweightProblemItemDto {
  contestId: number;
  index: string;
  title: string;
  rating: number | null;
  hasStatementSnapshot: boolean;
}

export interface ContestImportRunDto {
  importStatus: "incomplete" | "complete";
  missingSnapshotProblems: string[];
  failedSnapshotProblems: string[];
}

export type StatementReadStateDto =
  | { state: "pending" }
  | { state: "ready"; sanitizedHtml: string };

export interface LightweightProblemDetailDto {
  contestId: number;
  index: string;
  title: string;
  rating: number | null;
  sourceUrl: string;
  statement: StatementReadStateDto;
}

export interface LocalStatementAssetDto {
  localRef: string;
  mediaType: string;
  bytes: number[];
}

export function importCodeforcesContest(contestUrl: string): Promise<ContestImportRunDto> {
  return invoke<ContestImportRunDto>("import_codeforces_contest", {
    input: { contestUrl },
  });
}

export function getContestShelf(): Promise<ContestShelfItemDto[]> {
  return invoke<ContestShelfItemDto[]>("contest_shelf");
}

export function getContestDetail(contestId: number): Promise<ContestDetailDto> {
  return invoke<ContestDetailDto>("contest_detail", { input: { contestId } });
}

export function getLightweightProblems(): Promise<LightweightProblemItemDto[]> {
  return invoke<LightweightProblemItemDto[]>("lightweight_problems");
}

export function getLightweightProblemDetail(contestId: number, index: string): Promise<LightweightProblemDetailDto> {
  return invoke<LightweightProblemDetailDto>("lightweight_problem_detail", { input: { contestId, index } });
}

export function getStatementAssets(contestId: number, index: string): Promise<LocalStatementAssetDto[]> {
  return invoke<LocalStatementAssetDto[]>("statement_assets", { input: { contestId, index } });
}
