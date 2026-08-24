import { invoke } from "@tauri-apps/api/core";

export type TodayLane = "carryIn" | "review" | "study";
export type TodayReason = "continueReview" | "continueLearning" | "dueFirstColdStart" | "dueLongTermReview" | "relearn" | "upsolve";
export type TodayStatus = "notStarted" | "inProgress" | "completed" | "unavailable";

export interface TodayEntryDto {
  entryId: string; problemId: string; reviewAttemptId: string | null;
  problemTitle: string; problemRating: number | null;
  lane: TodayLane; reason: TodayReason; planningCostMinutes: number;
  position: number; origin: "auto" | "manual"; status: TodayStatus;
}
export interface TodaySnapshotDto {
  planId: string; localDate: string; budgetMinutes: number; plannedMinutes: number;
  overBudgetMinutes: number; reviewOnlyStreak: number; entries: TodayEntryDto[];
}
export interface TodayReplanEntryDto {
  existingEntryId: string | null; problemId: string; reviewAttemptId: string | null;
  lane: TodayLane; reason: TodayReason; planningCostMinutes: number;
  origin: "auto" | "manual"; status: TodayStatus;
}
export interface TodayReplanPreviewDto {
  expectedSnapshot: TodaySnapshotDto; proposedBudgetMinutes: number;
  proposedPlannedMinutes: number; proposedOverBudgetMinutes: number;
  proposedReviewOnlyStreak: number; entries: TodayReplanEntryDto[];
}
export interface TodayExtraSuggestionDto {
  problemId: string; problemTitle: string; problemRating: number | null;
  reviewAttemptId: string | null; lane: TodayLane;
  reason: TodayReason; planningCostMinutes: number;
}
export interface TodayExtraSuggestionsPreviewDto {
  expectedSnapshot: TodaySnapshotDto; remainingBudgetMinutes: number;
  suggestions: TodayExtraSuggestionDto[];
}
export interface WeeklyAcmBudgetDto {
  monday: number | null; tuesday: number | null; wednesday: number | null;
  thursday: number | null; friday: number | null; saturday: number | null; sunday: number | null;
}

export const loadToday = (budgetMinutes: number | null) => invoke<TodaySnapshotDto | null>("today_snapshot", { input: { budgetMinutes } });
export const reorderToday = (planId: string, orderedEntryIds: string[]) => invoke<TodaySnapshotDto>("reorder_today", { input: { planId, orderedEntryIds } });
export const completeTodayEntry = (planId: string, entryId: string) => invoke<TodaySnapshotDto>("complete_today_entry", { input: { planId, entryId } });
export const previewTodayReplan = (budgetMinutes: number) => invoke<TodayReplanPreviewDto>("preview_today_replan", { input: { budgetMinutes } });
export const applyTodayReplan = (preview: TodayReplanPreviewDto) => invoke<TodaySnapshotDto>("apply_today_replan", { preview });
export const loadTodayExtraSuggestions = () => invoke<TodayExtraSuggestionsPreviewDto>("today_extra_suggestions");
export const acceptTodayExtraSuggestion = (preview: TodayExtraSuggestionsPreviewDto, problemId: string) => invoke<TodaySnapshotDto>("accept_today_extra_suggestion", { input: { preview, problemId } });
export const loadWeeklyAcmBudget = () => invoke<WeeklyAcmBudgetDto>("weekly_acm_budget");
export const saveWeeklyAcmBudget = (schedule: WeeklyAcmBudgetDto) => invoke<WeeklyAcmBudgetDto>("save_weekly_acm_budget", { schedule });
