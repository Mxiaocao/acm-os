import { invoke } from "./invoke";

export interface ContestParticipationDto {
  id: string;
  contestId: string | null;
  contestTitle: string;
  kind: "live" | "virtual";
  localDate: string;
  localTime: string;
  acceptedCount: number | null;
  totalCount: number | null;
}
export type ParticipationInput = Pick<ContestParticipationDto, "id" | "kind" | "localDate" | "localTime" | "acceptedCount" | "totalCount">;
export const listContestParticipations = (contestId: number) => invoke<ContestParticipationDto[]>("contest_participations", { input: { contestId } });
export const saveContestParticipation = (contestId: number, record: ParticipationInput) => invoke<void>("save_contest_participation", { input: { contestId, ...record } });
export const deleteContestParticipation = (contestId: number, id: string) => invoke<void>("delete_contest_participation", { input: { contestId, id } });
export const participationKindLabel = (kind: ContestParticipationDto["kind"]) => kind === "live" ? "现场" : "VP";
export const participationScore = (record: ContestParticipationDto) => record.acceptedCount === null
  ? "赛内AC未记录" : `赛内AC ${record.acceptedCount}${record.totalCount === null ? "" : "/" + record.totalCount}`;
