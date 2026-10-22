import { invoke } from "./invoke";
import type { ContestParticipationDto } from "./contest-participation";

export interface LearningCalendarEntryDto {
  date: string;
  problemId: string;
  problemTitle: string;
  kind: "library_added" | "study" | "first_cold_start" | "long_term_review" | "early_check";
  status: "completed" | "scheduled" | "inProgress";
  dueDate: string | null;
  judgement: "mastered" | "partial" | "fail" | null;
}

export interface LearningCalendarDto {
  participations: ContestParticipationDto[];
  today: string;
  entries: LearningCalendarEntryDto[];
}

export const loadLearningCalendar = (year: number, month: number) =>
  invoke<LearningCalendarDto>("learning_calendar", { input: { year, month } });

export interface LibraryCompletionDateDto {
  problemId: string;
  problemTitle: string;
  date: string | null;
}
export const loadLibraryCompletionDates = () => invoke<LibraryCompletionDateDto[]>("library_completion_dates");
export const saveLibraryCompletionDate = (problemId: string, date: string) =>
  invoke<void>("save_library_completion_date", { input: { problemId, date } });
