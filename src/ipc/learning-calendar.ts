import { invoke } from "./invoke";

export interface LearningCalendarEntryDto {
  date: string;
  problemId: string;
  problemTitle: string;
  kind: "study" | "first_cold_start" | "long_term_review" | "early_check";
  status: "completed" | "scheduled" | "inProgress";
  dueDate: string | null;
  judgement: "mastered" | "partial" | "fail" | null;
}

export interface LearningCalendarDto {
  today: string;
  entries: LearningCalendarEntryDto[];
}

export const loadLearningCalendar = (year: number, month: number) =>
  invoke<LearningCalendarDto>("learning_calendar", { input: { year, month } });
