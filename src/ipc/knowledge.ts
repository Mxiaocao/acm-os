import { invoke } from "@tauri-apps/api/core";

export type KnowledgeLocationState = "ready" | "locationAnomaly";
export type KnowledgeUnderstandingLevel = "notLearned" | "vague" | "basic" | "proficient" | "deep";

export interface KnowledgeNodeDto {
  knowledgeNodeId: string;
  displayName: string;
  vaultRelativePath: string;
  contentDigest: string;
  locationState: KnowledgeLocationState;
}

export interface KnowledgeIndexDto {
  nodes: KnowledgeNodeDto[];
  locationAnomalies: KnowledgeNodeDto[];
}

export interface KnowledgeUnderstandingDto {
  knowledgeNodeId: string;
  current: KnowledgeUnderstandingLevel;
  historicalHighest: KnowledgeUnderstandingLevel;
  firstReachedHighestOn: string;
}

export interface RelatedKnowledgeProblemDto {
  problemId: string;
  contestId: number;
  problemIndex: string;
  title: string;
}

export interface KnowledgeDetailDto {
  node: KnowledgeNodeDto;
  understanding: KnowledgeUnderstandingDto | null;
  incoming: KnowledgeNodeDto[];
  outgoing: KnowledgeNodeDto[];
  relatedProblems: RelatedKnowledgeProblemDto[];
}
export interface KnowledgeReevaluationSuggestionDto { knowledgeNodeId: string; shouldSuggest: boolean; qualifyingProblemCount: number; }
export const loadKnowledgeReevaluationSuggestion = (knowledgeNodeId: string) => invoke<KnowledgeReevaluationSuggestionDto>("knowledge_reevaluation_suggestion", { input: { knowledgeNodeId } });

export type KnowledgeCandidateDisposition = "pending" | "acceptedIntent" | "ignored";

export interface KnowledgeCandidateDto {
  contestId: number;
  problemIndex: string;
  fingerprint: string;
  targetRef: string;
  disposition: KnowledgeCandidateDisposition;
  knowledgeNodeId: string | null;
}

export const loadKnowledgeIndex = (query = "") =>
  invoke<KnowledgeIndexDto>("knowledge_index", { input: { query } });

export const loadKnowledgeDetail = (knowledgeNodeId: string) =>
  invoke<KnowledgeDetailDto>("knowledge_detail", { input: { knowledgeNodeId } });

export const confirmKnowledgeUnderstanding = (
  knowledgeNodeId: string,
  level: KnowledgeUnderstandingLevel,
) => invoke<KnowledgeUnderstandingDto>("confirm_knowledge_understanding", { input: { knowledgeNodeId, level } });

export const openKnowledgeInObsidian = (knowledgeNodeId: string) =>
  invoke<void>("open_knowledge_in_obsidian", { input: { knowledgeNodeId } });

export const loadKnowledgeCandidates = (contestId: number, index: string) =>
  invoke<KnowledgeCandidateDto[]>("knowledge_candidates", { input: { contestId, index } });

export const registerKnowledgeCandidate = (
  contestId: number,
  index: string,
  fingerprint: string,
  targetRef: string,
) => invoke<KnowledgeCandidateDto>("register_knowledge_candidate", {
  input: { contestId, index, fingerprint, targetRef },
});

export const setKnowledgeCandidateDisposition = (
  contestId: number,
  index: string,
  fingerprint: string,
  disposition: KnowledgeCandidateDisposition,
) => invoke<KnowledgeCandidateDto>("set_knowledge_candidate_disposition", {
  input: { contestId, index, fingerprint, disposition },
});

export const acceptExistingKnowledgeCandidate = (
  contestId: number,
  index: string,
  fingerprint: string,
  knowledgeNodeId: string,
) => invoke<{ knowledgeNodeId: string; targetRef: string }>("accept_existing_knowledge_candidate", {
  input: { contestId, index, fingerprint, knowledgeNodeId },
});
