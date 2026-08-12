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
  lifecycle: ProblemLifecycleStateDto;
  reviewAction: "startReview" | "earlyCheck" | "continueReview" | null;
}

export type LearningStatusDto =
  | "unstarted"
  | "upsolvePending"
  | "learning"
  | "waitingColdStart"
  | "relearning"
  | "longTermReview";

export type ProblemLifecycleActionDto =
  | "joinUpsolve"
  | "startLearning"
  | "returnToPending"
  | "markUnderstood"
  | "withdrawUnderstood"
  | "startRelearning"
  | "stopLearning";

export interface ProblemLifecycleStateDto {
  learningStatus: LearningStatusDto;
  learningStatusSinceUtc: string;
  nextReviewDueLocalDate: string | null;
  availableActions: ProblemLifecycleActionDto[];
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

export interface ReviewAttemptDto {
  attemptId: string;
  contestId: number;
  index: string;
  attemptType: "firstColdStart" | "longTermReview" | "earlyCheck";
  scheduledDueLocalDate: string;
  startedEarly: boolean;
  judgementRuleVersion: number;
  startedAtUtc: string;
}

export interface ReviewFocusDto {
  attempt: ReviewAttemptDto;
  title: string;
  sourceUrl: string;
  statementSanitizedHtml: string;
  statementAssets: LocalStatementAssetDto[];
}

export type ReviewHelpLevel = 1 | 2 | 3 | 4 | 5;

export interface ReviewHelpItemDto {
  level: ReviewHelpLevel;
  consequence: "partial_at_best" | "fail_only";
  available: boolean;
  revealedAtUtc: string | null;
}

export interface ReviewHelpDrawerDto {
  attemptId: string;
  items: ReviewHelpItemDto[];
}

export interface RevealedReviewHelpDto {
  eventId: string;
  attemptId: string;
  level: ReviewHelpLevel;
  consequence: "partial_at_best" | "fail_only";
  title: string;
  contentMarkdown: string;
  sourceDigest: string;
  revealedAtUtc: string;
}

export type SubmissionResultDto =
  | "accepted"
  | "wrongAnswer"
  | "timeLimitExceeded"
  | "memoryLimitExceeded"
  | "runtimeError"
  | "compilationError"
  | "other";
export type ReviewJudgementDto = "mastered" | "partial" | "fail";
export type ReviewFailureReasonCodeDto =
  | "noIdea"
  | "keyPropertyBlocked"
  | "derivationBlocked"
  | "cannotImplement"
  | "implementationError"
  | "boundaryError"
  | "complexityError"
  | "other";

export interface ReviewFailureReasonDto {
  code: ReviewFailureReasonCodeDto;
  otherText: string | null;
}

export interface CompleteReviewInputDto {
  attemptId: string;
  finalAc: boolean;
  firstSubmissionResult: SubmissionResultDto;
  firstSubmissionOther: string | null;
  finalResult: SubmissionResultDto;
  finalResultOther: string | null;
  totalSubmissions: number;
  ideaIndependent: boolean;
  implementationIndependent: boolean;
  debugIndependence: "notNeeded" | "independent" | "usedSolvingHelp";
  externalHelp: "none" | "solvingHint" | "fullSolution";
  failureReasons: ReviewFailureReasonDto[];
}

export interface CompletedReviewAttemptDto {
  attempt: ReviewAttemptDto;
  judgement: ReviewJudgementDto;
  evidenceCodes: string[];
  failureReasons: ReviewFailureReasonDto[];
  completedAtUtc: string;
  completedLocalDate: string;
  lifecycle: ProblemLifecycleStateDto;
}

export interface ReviewCompletionFactsDto {
  finalAc: boolean;
  firstSubmissionResult: string;
  finalResult: string;
  totalSubmissions: number;
  ideaIndependent: boolean;
  implementationIndependent: boolean;
  debugIndependence: "notNeeded" | "independent" | "usedSolvingHelp";
  externalHelp: "none" | "solvingHint" | "fullSolution";
}

export interface ReviewHistoryItemDto {
  attempt: ReviewAttemptDto;
  status: "inProgress" | "completed" | "void";
  judgement: ReviewJudgementDto | null;
  completionFacts: ReviewCompletionFactsDto | null;
  evidenceCodes: string[];
  failureReasons: ReviewFailureReasonDto[];
  helpLevels: ReviewHelpLevel[];
  completedAtUtc: string | null;
  completedLocalDate: string | null;
  voidReason: string | null;
  voidedAtUtc: string | null;
}

export interface ReviewHistoryDto {
  contestId: number;
  index: string;
  historicalBestReview: ReviewJudgementDto | null;
  mastery: ProblemMasteryProjectionDto;
  attempts: ReviewHistoryItemDto[];
}

export interface ProblemMasteryEvidenceDto {
  recallsProblem: boolean;
  multipleSolutionsClear: boolean;
  knowledgeUnderstood: boolean;
  implementationFluent: boolean;
  canAdaptOrCreate: boolean;
  transferSolvedIndependently: boolean;
}

export interface ProblemMasteryProjectionDto {
  current: ProblemMasteryEvidenceDto;
  historicalThoroughlyDigested: boolean;
  firstThoroughlyDigestedLocalDate: string | null;
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

export function transitionProblemLifecycle(
  contestId: number,
  index: string,
  action: ProblemLifecycleActionDto,
): Promise<ProblemLifecycleStateDto> {
  return invoke<ProblemLifecycleStateDto>("transition_problem_lifecycle", {
    input: { contestId, index, action },
  });
}

export function startOrResumeReview(contestId: number, index: string): Promise<ReviewAttemptDto> {
  return invoke<ReviewAttemptDto>("start_or_resume_review", { input: { contestId, index } });
}

export function getReviewFocus(attemptId: string): Promise<ReviewFocusDto> {
  return invoke<ReviewFocusDto>("review_focus", { input: { attemptId } });
}

export function getReviewHelpDrawer(attemptId: string): Promise<ReviewHelpDrawerDto> {
  return invoke<ReviewHelpDrawerDto>("review_help_drawer", { input: { attemptId } });
}

export function revealReviewHelp(
  attemptId: string,
  level: ReviewHelpLevel,
  impactAcknowledged: boolean,
): Promise<RevealedReviewHelpDto> {
  return invoke<RevealedReviewHelpDto>("reveal_review_help", {
    input: { attemptId, level, impactAcknowledged },
  });
}

export function completeReview(input: CompleteReviewInputDto): Promise<CompletedReviewAttemptDto> {
  return invoke<CompletedReviewAttemptDto>("complete_review", { input });
}

export function voidReview(attemptId: string, reason: string): Promise<ReviewHistoryItemDto> {
  return invoke<ReviewHistoryItemDto>("void_review", { input: { attemptId, reason } });
}

export function getReviewAttemptHistory(attemptId: string): Promise<ReviewHistoryItemDto> {
  return invoke<ReviewHistoryItemDto>("review_attempt_history", { input: { attemptId } });
}

export function getReviewHistory(contestId: number, index: string): Promise<ReviewHistoryDto> {
  return invoke<ReviewHistoryDto>("review_history", { input: { contestId, index } });
}

export function updateProblemMasteryEvidence(
  contestId: number,
  index: string,
  evidence: ProblemMasteryEvidenceDto,
): Promise<ProblemMasteryProjectionDto> {
  return invoke<ProblemMasteryProjectionDto>("update_problem_mastery_evidence", {
    input: { contestId, index, evidence },
  });
}

export function deletePersonalNote(contestId: number, index: string): Promise<ProblemLifecycleStateDto> {
  return invoke<ProblemLifecycleStateDto>("delete_personal_note", {
    input: { contestId, index },
  });
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
