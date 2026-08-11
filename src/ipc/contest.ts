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
  identityType: "lightweight" | "personal";
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
  identityType: "lightweight" | "personal";
  personalNote: PersonalNoteBindingDto | null;
}

export interface PersonalNoteBindingDto {
  vaultRelativePath: string;
}

export interface ProblemMarkdownProjectionDto {
  contentDigest: string;
  knownSections: KnownMarkdownSectionDto[];
  solutionRoutes: SolutionRouteDto[];
  warnings: MarkdownParseWarningDto[];
}

export interface KnownMarkdownSectionDto {
  name: string;
  startOffset: number;
  endOffset: number;
}

export interface SolutionRouteDto {
  name: string;
  startOffset: number;
  endOffset: number;
}

export interface MarkdownParseWarningDto {
  code: "duplicate_known_section";
  name: string;
  count: number;
}

export type PersonalNoteReadStateDto =
  | {
      state: "ready";
      vaultRelativePath: string;
      relocated: boolean;
      projection: ProblemMarkdownProjectionDto;
    }
  | { state: "locationAnomaly"; lastKnownPath: string }
  | { state: "vaultUnavailable"; lastKnownPath: string };

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

export function createPersonalNote(contestId: number, index: string): Promise<PersonalNoteBindingDto> {
  return invoke<PersonalNoteBindingDto>("create_personal_note", { input: { contestId, index } });
}

export function getPersonalNoteProjection(contestId: number, index: string): Promise<PersonalNoteReadStateDto> {
  return invoke<PersonalNoteReadStateDto>("personal_note_projection", { input: { contestId, index } });
}

export function openPersonalNoteInObsidian(contestId: number, index: string): Promise<void> {
  return invoke<void>("open_personal_note_in_obsidian", { input: { contestId, index } });
}

export function getStatementAssets(contestId: number, index: string): Promise<LocalStatementAssetDto[]> {
  return invoke<LocalStatementAssetDto[]>("statement_assets", { input: { contestId, index } });
}
