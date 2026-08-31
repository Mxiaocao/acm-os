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
  identityConflicts: KnowledgeIdentityConflictDto[];
}

export interface KnowledgeIdentityConflictDto {
  historicalKnowledgeNodeId: string;
  displayName: string;
  candidateVaultRelativePath: string;
}

export interface KnowledgeRelocationCandidateDto {
  vaultRelativePath: string;
  occupied: boolean;
}

export interface KnowledgeUnderstandingDto {
  knowledgeNodeId: string;
  current: KnowledgeUnderstandingLevel;
  historicalHighest: KnowledgeUnderstandingLevel;
  firstReachedHighestOn: string;
}

export interface RelatedKnowledgeProblemDto {
  problemId: string;
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
export interface CanonicalKnowledgeCandidateDto {
  problemId: string;
  fingerprint: string;
  targetRef: string;
  disposition: KnowledgeCandidateDisposition;
  knowledgeNodeId: string | null;
}

export const loadKnowledgeIndex = (query = "") =>
  invoke<KnowledgeIndexDto>("knowledge_index", { input: { query } });

export const loadKnowledgeRelocationCandidates = (knowledgeNodeId: string) =>
  invoke<KnowledgeRelocationCandidateDto[]>("knowledge_relocation_candidates", { input: { knowledgeNodeId } });

export const rebindKnowledgeNode = (knowledgeNodeId: string, vaultRelativePath: string) =>
  invoke<KnowledgeNodeDto>("rebind_knowledge_node", { input: { knowledgeNodeId, vaultRelativePath } });

export const confirmKnowledgeMarkdownDeleted = (knowledgeNodeId: string) =>
  invoke<void>("confirm_knowledge_markdown_deleted", { input: { knowledgeNodeId } });

export const resolveKnowledgeIdentityConflict = (
  historicalKnowledgeNodeId: string,
  candidateVaultRelativePath: string,
  restoreOldIdentity: boolean,
) => invoke<KnowledgeNodeDto>("resolve_knowledge_identity_conflict", {
  input: { historicalKnowledgeNodeId, candidateVaultRelativePath, restoreOldIdentity },
});

export const loadKnowledgeDetail = (knowledgeNodeId: string) =>
  invoke<KnowledgeDetailDto>("knowledge_detail", { input: { knowledgeNodeId } });

export const confirmKnowledgeUnderstanding = (
  knowledgeNodeId: string,
  level: KnowledgeUnderstandingLevel,
) => invoke<KnowledgeUnderstandingDto>("confirm_knowledge_understanding", { input: { knowledgeNodeId, level } });

export const openKnowledgeInObsidian = (knowledgeNodeId: string) =>
  invoke<void>("open_knowledge_in_obsidian", { input: { knowledgeNodeId } });

export const openObsidianGraph = () => invoke<void>("open_obsidian_graph");

export const loadKnowledgeCandidates = (contestId: number, index: string) =>
  invoke<KnowledgeCandidateDto[]>("knowledge_candidates", { input: { contestId, index } });

export const loadKnowledgeCandidatesById = (problemId: string) =>
  invoke<CanonicalKnowledgeCandidateDto[]>("knowledge_candidates_by_id", { input: { problemId } });

export const registerKnowledgeCandidate = (
  contestId: number,
  index: string,
  fingerprint: string,
  targetRef: string,
) => invoke<KnowledgeCandidateDto>("register_knowledge_candidate", {
  input: { contestId, index, fingerprint, targetRef },
});

export const registerKnowledgeCandidateById = (
  problemId: string,
  fingerprint: string,
  targetRef: string,
) => invoke<CanonicalKnowledgeCandidateDto>("register_knowledge_candidate_by_id", {
  input: { problemId, fingerprint, targetRef },
});

export const setKnowledgeCandidateDisposition = (
  contestId: number,
  index: string,
  fingerprint: string,
  disposition: KnowledgeCandidateDisposition,
) => invoke<KnowledgeCandidateDto>("set_knowledge_candidate_disposition", {
  input: { contestId, index, fingerprint, disposition },
});

export const setKnowledgeCandidateDispositionById = (
  problemId: string,
  fingerprint: string,
  disposition: KnowledgeCandidateDisposition,
) => invoke<CanonicalKnowledgeCandidateDto>("set_knowledge_candidate_disposition_by_id", {
  input: { problemId, fingerprint, disposition },
});

export const acceptExistingKnowledgeCandidate = (
  contestId: number,
  index: string,
  fingerprint: string,
  knowledgeNodeId: string,
) => invoke<{ knowledgeNodeId: string; targetRef: string }>("accept_existing_knowledge_candidate", {
  input: { contestId, index, fingerprint, knowledgeNodeId },
});

export const acceptExistingKnowledgeCandidateById = (
  problemId: string,
  fingerprint: string,
  knowledgeNodeId: string,
) => invoke<{ knowledgeNodeId: string; targetRef: string }>("accept_existing_knowledge_candidate_by_id", {
  input: { problemId, fingerprint, knowledgeNodeId },
});
