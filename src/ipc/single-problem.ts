import { invoke } from "./invoke";

export interface IndependentProblemDto {
  importedAtUtc?: string | null;
  statementHtml?: string | null;
  problemId: string; title: string; sourceUrl: string; rating: number | null; hasStatementSnapshot: boolean;
}
export interface SingleProblemInput {
  completionDate: string;
  mode: "manual" | "codeforces"; requestId: string; title: string; sourceUrl: string;
  rating: number | null; statementText: string;
}
export const importSingleProblem = (input: SingleProblemInput) => invoke<{ problemId: string }>("import_single_problem", { input });
export const getIndependentProblems = () => invoke<IndependentProblemDto[]>("independent_problems");
