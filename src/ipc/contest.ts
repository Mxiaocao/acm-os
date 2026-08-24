import { invoke } from "@tauri-apps/api/core";

export interface ContestShelfItemDto {
  contestId: number;
  title: string;
  importStatus: "incomplete" | "complete";
  problemCount: number;
  missingSnapshotCount: number;
  archived: boolean;
}

export interface ContestLibraryFamilyDto {
  familyId: number;
  displayName: string;
}

export interface ContestLibrarySeriesDto {
  seriesId: number;
  familyId: number;
  displayName: string;
}

export interface ContestLibraryPlacementDto {
  placementId: number;
  familyId: number;
  familyName: string;
  seriesId: number | null;
  seriesName: string | null;
  year: number | null;
  ordinal: number | null;
}

export type ContestLibrarySeriesFilterDto =
  | { kind: "any" }
  | { kind: "unassigned" }
  | { kind: "exact"; seriesId: number };

export type ContestLibraryYearFilterDto =
  | { kind: "any" }
  | { kind: "unassigned" }
  | { kind: "exact"; year: number };

export type ContestLibraryScopeDto =
  | { kind: "all" }
  | {
      kind: "family";
      familyId: number;
      series: ContestLibrarySeriesFilterDto;
      year: ContestLibraryYearFilterDto;
    };

export type ContestLibraryArchiveFilterDto = "all" | "active" | "archived";

export interface ContestLibraryListInput {
  scope: ContestLibraryScopeDto;
  archive: ContestLibraryArchiveFilterDto;
}

export interface ContestDetailDto {
  contestId: number;
  title: string;
  sourceUrl: string;
  contestDate: string | null;
  importStatus: "incomplete" | "complete";
  factsStatus: "pending" | "completed";
  problems: ContestProblemDetailItemDto[];
  corrections: ContestCorrectionEventDto[];
  aiAnalysis: ContestAiAnalysisDto | null;
  archived: boolean;
}

export interface ContestAiAnalysisDto { rawText: string; parseStatus: "complete" | "partial" | "failed"; parsedProjectionJson: string; updatedAtUtc: string; }
export interface ContestAiAnalysisPreviewDto { rawText: string; parseStatus: "complete" | "partial" | "failed"; parsedProjectionJson: string; }

export interface ContestCorrectionEventDto { correctionId: string; problemIndex: string; field: "finalContestResult" | "upsolveDecision"; oldValue: string; newValue: string; correctedAtUtc: string; }

export type ContestFinalResultDto = "unknown" | "notAttempted" | "accepted" | "wrongAnswer" | "timeLimitExceeded" | "memoryLimitExceeded" | "runtimeError" | "compilationError" | "otherFailed";
export type ContestUpsolveDecisionDto = "planned" | "notPlanned" | "undecided";
export interface ContestProblemDetailItemDto extends LightweightProblemItemDto { finalContestResult: ContestFinalResultDto | null; upsolveDecision: ContestUpsolveDecisionDto; liveLearningStatus: LearningStatusDto; }

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

export interface CanonicalProblemDetailDto {
  problemId: string;
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

export interface PersonalNoteRelocationCandidateDto {
  vaultRelativePath: string;
  occupied: boolean;
}

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

export interface CanonicalReviewHistoryDto {
  problemId: string;
  historicalBestReview: ReviewJudgementDto | null;
  mastery: ProblemMasteryProjectionDto;
  attempts: Array<Omit<ReviewHistoryItemDto, "attempt"> & { attempt: CanonicalReviewAttemptDto }>;
}
export interface CanonicalReviewAttemptDto {
  attemptId: string;
  problemId: string;
  attemptType: "firstColdStart" | "longTermReview" | "earlyCheck";
  scheduledDueLocalDate: string;
  startedEarly: boolean;
  judgementRuleVersion: number;
  startedAtUtc: string;
}

export function listContestLibraryFamilies(): Promise<ContestLibraryFamilyDto[]> {
  return invoke<ContestLibraryFamilyDto[]>("contest_library_list_families");
}

export function createContestLibraryFamily(displayName: string): Promise<ContestLibraryFamilyDto> {
  return invoke<ContestLibraryFamilyDto>("contest_library_create_family", {
    input: { displayName },
  });
}

export function renameContestLibraryFamily(
  familyId: number,
  displayName: string,
): Promise<ContestLibraryFamilyDto> {
  return invoke<ContestLibraryFamilyDto>("contest_library_rename_family", {
    input: { familyId, displayName },
  });
}

export function listContestLibrarySeries(familyId: number): Promise<ContestLibrarySeriesDto[]> {
  return invoke<ContestLibrarySeriesDto[]>("contest_library_list_series", {
    input: { familyId },
  });
}

export function createContestLibrarySeries(
  familyId: number,
  displayName: string,
): Promise<ContestLibrarySeriesDto> {
  return invoke<ContestLibrarySeriesDto>("contest_library_create_series", {
    input: { familyId, displayName },
  });
}

export function renameContestLibrarySeries(
  seriesId: number,
  displayName: string,
): Promise<ContestLibrarySeriesDto> {
  return invoke<ContestLibrarySeriesDto>("contest_library_rename_series", {
    input: { seriesId, displayName },
  });
}

export function listContestLibraryYears(
  familyId: number,
  series: ContestLibrarySeriesFilterDto,
): Promise<Array<number | null>> {
  return invoke<Array<number | null>>("contest_library_list_years", {
    input: { familyId, series },
  });
}

export function listContestLibraryPlacements(
  contestId: number,
): Promise<ContestLibraryPlacementDto[]> {
  return invoke<ContestLibraryPlacementDto[]>("contest_library_list_contest_placements", {
    input: { contestId },
  });
}

export function createContestLibraryPlacement(input: {
  contestId: number;
  familyId: number;
  seriesId: number | null;
  year: number | null;
  ordinal: number | null;
}): Promise<ContestLibraryPlacementDto> {
  return invoke<ContestLibraryPlacementDto>("contest_library_create_placement", { input });
}

export function updateContestLibraryPlacement(input: {
  placementId: number;
  familyId: number;
  seriesId: number | null;
  year: number | null;
  ordinal: number | null;
}): Promise<ContestLibraryPlacementDto> {
  return invoke<ContestLibraryPlacementDto>("contest_library_update_placement", { input });
}

export function removeContestLibraryPlacement(placementId: number): Promise<void> {
  return invoke<void>("contest_library_remove_placement", { input: { placementId } });
}

export function listContestLibraryContests(
  input: ContestLibraryListInput,
): Promise<ContestShelfItemDto[]> {
  return invoke<ContestShelfItemDto[]>("contest_library_list_contests", { input });
}

export function getContestDetail(contestId: number): Promise<ContestDetailDto> {
  return invoke<ContestDetailDto>("contest_detail", { input: { contestId } });
}

export interface ManualProblemInputDto { index: string; title: string; sourceUrl: string; statementText: string; }
export function importManualCodeforcesContest(input: { contestId: number; title: string; sourceUrl: string; startsAtUtc: string | null; problems: ManualProblemInputDto[] }): Promise<ContestImportRunDto> { return invoke<ContestImportRunDto>("import_manual_codeforces_contest", { input }); }

export function completeContestFacts(contestId: number, problems: Array<{ index: string; finalContestResult: ContestFinalResultDto; upsolveDecision: ContestUpsolveDecisionDto }>): Promise<ContestDetailDto> {
  return invoke<ContestDetailDto>("complete_contest_facts", { input: { contestId, problems } });
}

export function correctContestProblemFacts(contestId: number, index: string, finalContestResult: ContestFinalResultDto, upsolveDecision: ContestUpsolveDecisionDto): Promise<ContestDetailDto> {
  return invoke<ContestDetailDto>("correct_contest_problem_facts", { input: { contestId, index, finalContestResult, upsolveDecision } });
}

export interface ContestDeletePreviewDto { contestTitle: string; relationshipCount: number; cleanupProblemCount: number; preservedProblemCount: number; }
export function setContestArchived(contestId: number, archived: boolean): Promise<ContestDetailDto> { return invoke<ContestDetailDto>("set_contest_archived", { input: { contestId, archived } }); }
export function previewDeleteContest(contestId: number): Promise<ContestDeletePreviewDto> { return invoke<ContestDeletePreviewDto>("preview_delete_contest", { input: { contestId } }); }
export function deleteContest(contestId: number): Promise<ContestDeletePreviewDto> { return invoke<ContestDeletePreviewDto>("delete_contest", { input: { contestId } }); }

export function previewContestAiAnalysis(contestId: number, rawText: string): Promise<ContestAiAnalysisPreviewDto> { return invoke<ContestAiAnalysisPreviewDto>("preview_contest_ai_analysis", { input: { contestId, rawText } }); }
export function saveContestAiAnalysis(contestId: number, rawText: string): Promise<ContestDetailDto> { return invoke<ContestDetailDto>("save_contest_ai_analysis", { input: { contestId, rawText } }); }

export function getLightweightProblems(): Promise<LightweightProblemItemDto[]> {
  return invoke<LightweightProblemItemDto[]>("lightweight_problems");
}

export function getLightweightProblemDetail(contestId: number, index: string): Promise<LightweightProblemDetailDto> {
  return invoke<LightweightProblemDetailDto>("lightweight_problem_detail", { input: { contestId, index } });
}

export function getCanonicalProblemDetail(problemId: string): Promise<CanonicalProblemDetailDto> {
  return invoke<CanonicalProblemDetailDto>("lightweight_problem_detail_by_id", { input: { problemId } });
}

export function createPersonalNote(contestId: number, index: string): Promise<PersonalNoteBindingDto> {
  return invoke<PersonalNoteBindingDto>("create_personal_note", { input: { contestId, index } });
}

export function createPersonalNoteById(problemId: string): Promise<PersonalNoteBindingDto> {
  return invoke<PersonalNoteBindingDto>("create_personal_note_by_id", { input: { problemId } });
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

export function transitionProblemLifecycleById(problemId: string, action: ProblemLifecycleActionDto): Promise<ProblemLifecycleStateDto> {
  return invoke<ProblemLifecycleStateDto>("transition_problem_lifecycle_by_id", { input: { problemId, action } });
}

export function startOrResumeReview(contestId: number, index: string): Promise<ReviewAttemptDto> {
  return invoke<ReviewAttemptDto>("start_or_resume_review", { input: { contestId, index } });
}

export function startOrResumeReviewById(problemId: string): Promise<CanonicalReviewAttemptDto> {
  return invoke<CanonicalReviewAttemptDto>("start_or_resume_review_by_id", { input: { problemId } });
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

export function getReviewHistoryById(problemId: string): Promise<CanonicalReviewHistoryDto> {
  return invoke<CanonicalReviewHistoryDto>("review_history_by_id", { input: { problemId } });
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

export function updateProblemMasteryEvidenceById(problemId: string, evidence: ProblemMasteryEvidenceDto): Promise<ProblemMasteryProjectionDto> {
  return invoke<ProblemMasteryProjectionDto>("update_problem_mastery_evidence_by_id", { input: { problemId, evidence } });
}

export function deletePersonalNote(contestId: number, index: string): Promise<ProblemLifecycleStateDto> {
  return invoke<ProblemLifecycleStateDto>("delete_personal_note", {
    input: { contestId, index },
  });
}

export function getPersonalNoteProjection(contestId: number, index: string): Promise<PersonalNoteReadStateDto> {
  return invoke<PersonalNoteReadStateDto>("personal_note_projection", { input: { contestId, index } });
}

export function getPersonalNoteRelocationCandidates(
  contestId: number,
  index: string,
): Promise<PersonalNoteRelocationCandidateDto[]> {
  return invoke<PersonalNoteRelocationCandidateDto[]>("personal_note_relocation_candidates", {
    input: { contestId, index },
  });
}

export function rebindPersonalNote(
  contestId: number,
  index: string,
  vaultRelativePath: string,
): Promise<PersonalNoteBindingDto> {
  return invoke<PersonalNoteBindingDto>("rebind_personal_note", {
    input: { contestId, index, vaultRelativePath },
  });
}

export function confirmPersonalNoteDeleted(
  contestId: number,
  index: string,
): Promise<ProblemLifecycleStateDto> {
  return invoke<ProblemLifecycleStateDto>("confirm_personal_note_deleted", {
    input: { contestId, index },
  });
}

export function openPersonalNoteInObsidian(contestId: number, index: string): Promise<void> {
  return invoke<void>("open_personal_note_in_obsidian", { input: { contestId, index } });
}

export function openOriginalOj(url: string): Promise<void> {
  return invoke<void>("open_original_oj", { input: { url } });
}

export function deletePersonalNoteById(problemId: string): Promise<ProblemLifecycleStateDto> {
  return invoke<ProblemLifecycleStateDto>("delete_personal_note_by_id", { input: { problemId } });
}

export function getPersonalNoteProjectionById(problemId: string): Promise<PersonalNoteReadStateDto> {
  return invoke<PersonalNoteReadStateDto>("personal_note_projection_by_id", { input: { problemId } });
}

export function getPersonalNoteRelocationCandidatesById(problemId: string): Promise<PersonalNoteRelocationCandidateDto[]> {
  return invoke<PersonalNoteRelocationCandidateDto[]>("personal_note_relocation_candidates_by_id", { input: { problemId } });
}

export function rebindPersonalNoteById(problemId: string, vaultRelativePath: string): Promise<PersonalNoteBindingDto> {
  return invoke<PersonalNoteBindingDto>("rebind_personal_note_by_id", { input: { problemId, vaultRelativePath } });
}

export function confirmPersonalNoteDeletedById(problemId: string): Promise<ProblemLifecycleStateDto> {
  return invoke<ProblemLifecycleStateDto>("confirm_personal_note_deleted_by_id", { input: { problemId } });
}

export function openPersonalNoteInObsidianById(problemId: string): Promise<void> {
  return invoke<void>("open_personal_note_in_obsidian_by_id", { input: { problemId } });
}

export function getStatementAssets(contestId: number, index: string): Promise<LocalStatementAssetDto[]> {
  return invoke<LocalStatementAssetDto[]>("statement_assets", { input: { contestId, index } });
}

export function getCanonicalStatementAssets(problemId: string): Promise<LocalStatementAssetDto[]> {
  return invoke<LocalStatementAssetDto[]>("statement_assets_by_id", { input: { problemId } });
}
