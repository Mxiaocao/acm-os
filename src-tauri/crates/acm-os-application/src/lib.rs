#![forbid(unsafe_code)]

pub mod codeforces;

use std::path::{Component, Path};

pub const BOUNDARY_NAME: &str = "acm-os-application";

pub use acm_os_domain::{
    ContestIdentity, ExternalContestKey, GenericIdentityError, PlatformKey, ProblemIdentity,
};

/// The canonical, adapter-validated import contract.  It deliberately has no
/// network or database details: adapters produce it and persistence consumes
/// it after identity validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestImportDraft {
    pub contest: acm_os_domain::CodeforcesContestIdentity,
    pub title: String,
    pub source_url: String,
    pub starts_at_utc: Option<String>,
    pub slots: Vec<ContestProblemSlotDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestProblemSlotDraft {
    pub ordinal: u32,
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub title: String,
    pub rating: Option<u32>,
    pub source_url: String,
}

/// The first successful capture is immutable.  Re-import may fill a missing
/// snapshot, but it must never replace an existing one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementSnapshotDraft {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub source_html: String,
    pub sanitized_html: String,
    pub assets: Vec<StatementAssetDraft>,
}

/// A binary asset captured alongside the first statement snapshot. The
/// renderer only receives the local reference, never the original remote URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementAssetDraft {
    pub local_ref: String,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

/// An adapter has completed all external work before this plan reaches
/// persistence. Keeping this value pure makes it impossible for a SQLite
/// transaction to own an HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestImportExecutionPlan {
    pub manifest: ContestImportDraft,
    pub snapshots: Vec<StatementSnapshotDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualProblemDraft {
    pub index: String,
    pub title: String,
    pub source_url: String,
    pub statement_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualContestError {
    InvalidIdentity,
    InvalidInput,
    DuplicateProblem,
}

pub fn build_manual_codeforces_contest(
    contest_id: u64,
    title: &str,
    source_url: &str,
    starts_at_utc: Option<String>,
    problems: &[ManualProblemDraft],
) -> Result<ContestImportExecutionPlan, ManualContestError> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(contest_id)
        .map_err(|_| ManualContestError::InvalidIdentity)?;
    if title.trim().is_empty() || source_url.trim().is_empty() || problems.is_empty() {
        return Err(ManualContestError::InvalidInput);
    }
    let mut seen = std::collections::HashSet::new();
    let mut slots = Vec::with_capacity(problems.len());
    let mut snapshots = Vec::with_capacity(problems.len());
    for (position, item) in problems.iter().enumerate() {
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            contest.clone(),
            item.index.trim().to_owned(),
        )
        .map_err(|_| ManualContestError::InvalidIdentity)?;
        if !seen.insert(problem.clone()) {
            return Err(ManualContestError::DuplicateProblem);
        }
        if item.title.trim().is_empty()
            || item.source_url.trim().is_empty()
            || item.statement_text.trim().is_empty()
        {
            return Err(ManualContestError::InvalidInput);
        }
        slots.push(ContestProblemSlotDraft {
            ordinal: position as u32 + 1,
            problem: problem.clone(),
            title: item.title.trim().to_owned(),
            rating: None,
            source_url: item.source_url.trim().to_owned(),
        });
        let escaped = item
            .statement_text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
            .replace("\r\n", "\n")
            .replace('\n', "<br>");
        snapshots.push(StatementSnapshotDraft {
            problem,
            source_html: item.statement_text.clone(),
            sanitized_html: format!(
                "<div class=\"problem-statement manual-statement\"><p>{escaped}</p></div>"
            ),
            assets: Vec::new(),
        });
    }
    let manifest = ContestImportDraft::validated(
        contest,
        title.trim().to_owned(),
        source_url.trim().to_owned(),
        starts_at_utc,
        slots,
    )
    .map_err(|_| ManualContestError::InvalidInput)?;
    ContestImportExecutionPlan::validated(manifest, snapshots)
        .map_err(|_| ManualContestError::InvalidInput)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContestImportExecutionError {
    SnapshotOutsideManifest,
    DuplicateSnapshotIdentity,
}

impl ContestImportExecutionPlan {
    pub fn validated(
        manifest: ContestImportDraft,
        snapshots: Vec<StatementSnapshotDraft>,
    ) -> Result<Self, ContestImportExecutionError> {
        let manifest_problems: std::collections::HashSet<_> = manifest
            .slots
            .iter()
            .map(|slot| slot.problem.clone())
            .collect();
        let mut seen = std::collections::HashSet::new();
        for snapshot in &snapshots {
            if !manifest_problems.contains(&snapshot.problem) {
                return Err(ContestImportExecutionError::SnapshotOutsideManifest);
            }
            if !seen.insert(snapshot.problem.clone()) {
                return Err(ContestImportExecutionError::DuplicateSnapshotIdentity);
            }
        }
        Ok(Self {
            manifest,
            snapshots,
        })
    }

    /// Selects only the snapshots still missing after manifest persistence.
    /// Existing first captures are never scheduled for replacement.
    pub fn snapshots_for_missing(
        &self,
        missing: &[acm_os_domain::CodeforcesProblemIdentity],
    ) -> Vec<&StatementSnapshotDraft> {
        self.snapshots
            .iter()
            .filter(|snapshot| missing.contains(&snapshot.problem))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestImportStatus {
    Incomplete,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedContestImport {
    pub status: ContestImportStatus,
    pub missing_snapshot_problems: Vec<acm_os_domain::CodeforcesProblemIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestImportPersistenceError {
    Unavailable,
    ManifestConflict,
}

#[allow(async_fn_in_trait)]
pub trait ContestImportPort {
    /// Persists the first manifest for a contest. A later call must preserve
    /// that manifest rather than silently accepting remote structural drift.
    async fn persist_manifest(
        &self,
        draft: &ContestImportDraft,
    ) -> Result<PersistedContestImport, ContestImportPersistenceError>;

    /// Inserts a first snapshot only if it is currently missing, returning the
    /// contest's recalculated completion state.
    async fn persist_first_snapshot(
        &self,
        snapshot: &StatementSnapshotDraft,
    ) -> Result<PersistedContestImport, ContestImportPersistenceError>;
}

#[allow(async_fn_in_trait)]
pub trait ContestImportSource {
    /// Fetches and validates a full ordered manifest before System Facts are
    /// changed. Implementations own all network authority.
    async fn fetch_manifest(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
    ) -> Result<ContestImportDraft, ContestImportSourceError>;

    /// Fetches one missing first snapshot. This is deliberately separate from
    /// manifest fetch so partial retry never re-downloads completed items.
    async fn fetch_snapshot(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<StatementSnapshotDraft, ContestImportSourceError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContestImportSourceError {
    Unavailable,
    InvalidRemoteData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestImportRun {
    pub persisted: PersistedContestImport,
    pub failed_snapshot_problems: Vec<acm_os_domain::CodeforcesProblemIdentity>,
}

/// Coordinates an adapter and persistence without granting network authority
/// to the application. Every database call is complete before the next remote
/// request begins; partial item failures preserve already captured snapshots.
pub async fn import_codeforces_contest<P: ContestImportPort, S: ContestImportSource>(
    persistence: &P,
    source: &S,
    contest: acm_os_domain::CodeforcesContestIdentity,
) -> Result<ContestImportRun, ContestImportSourceError> {
    let manifest = source.fetch_manifest(&contest).await?;
    let mut persisted = persistence
        .persist_manifest(&manifest)
        .await
        .map_err(|_| ContestImportSourceError::Unavailable)?;
    let mut failed_snapshot_problems = Vec::new();

    for problem in persisted.missing_snapshot_problems.clone() {
        match source.fetch_snapshot(&problem).await {
            Ok(snapshot) => {
                persisted = persistence
                    .persist_first_snapshot(&snapshot)
                    .await
                    .map_err(|_| ContestImportSourceError::Unavailable)?;
            }
            Err(_) => failed_snapshot_problems.push(problem),
        }
    }
    Ok(ContestImportRun {
        persisted,
        failed_snapshot_problems,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestShelfItem {
    pub contest: acm_os_domain::CodeforcesContestIdentity,
    pub title: String,
    pub import_status: ContestImportStatus,
    pub problem_count: u32,
    pub missing_snapshot_count: u32,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestDetail {
    pub contest: acm_os_domain::CodeforcesContestIdentity,
    pub title: String,
    pub source_url: String,
    pub contest_date: Option<String>,
    pub import_status: ContestImportStatus,
    pub facts_status: ContestFactsStatus,
    pub problems: Vec<ContestProblemDetailItem>,
    pub corrections: Vec<ContestCorrectionEvent>,
    pub ai_analysis: Option<ContestAiAnalysis>,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestDeletePreview {
    pub contest_title: String,
    pub relationship_count: u32,
    pub cleanup_problem_count: u32,
    pub preserved_problem_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestManagementError {
    Unavailable,
    NotFound,
}

#[allow(async_fn_in_trait)]
pub trait ContestManagementPort {
    async fn set_contest_archived(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
        archived: bool,
    ) -> Result<ContestDetail, ContestManagementError>;
    async fn preview_delete_contest(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
    ) -> Result<ContestDeletePreview, ContestManagementError>;
    async fn delete_contest(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
    ) -> Result<ContestDeletePreview, ContestManagementError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestAiAnalysis {
    pub raw_text: String,
    pub parse_status: ContestAiParseStatus,
    pub parsed_projection_json: String,
    pub updated_at_utc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestAiParseStatus {
    Complete,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestAiAnalysisPreview {
    pub raw_text: String,
    pub parse_status: ContestAiParseStatus,
    pub parsed_projection_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestAiAnalysisError {
    Unavailable,
    NotFound,
    Empty,
    Invalid,
}

pub fn preview_contest_ai_analysis(
    raw_text: &str,
) -> Result<ContestAiAnalysisPreview, ContestAiAnalysisError> {
    if raw_text.trim().is_empty() {
        return Err(ContestAiAnalysisError::Empty);
    }
    let has_title = raw_text
        .lines()
        .any(|line| line.trim() == "# Contest AI Analysis");
    if !has_title {
        return Ok(ContestAiAnalysisPreview {
            raw_text: raw_text.to_owned(),
            parse_status: ContestAiParseStatus::Failed,
            parsed_projection_json: "{}".to_owned(),
        });
    }
    let has_overall = raw_text.lines().any(|line| line.trim() == "## Overall");
    let problem_count = raw_text
        .lines()
        .filter(|line| line.trim().starts_with("## Problem "))
        .count();
    let status = if has_overall && problem_count > 0 {
        ContestAiParseStatus::Complete
    } else {
        ContestAiParseStatus::Partial
    };
    let projection = format!(
        r#"{{"overall":{},"problemCount":{}}}"#,
        has_overall, problem_count
    );
    Ok(ContestAiAnalysisPreview {
        raw_text: raw_text.to_owned(),
        parse_status: status,
        parsed_projection_json: projection,
    })
}

#[allow(async_fn_in_trait)]
pub trait ContestAiAnalysisPort {
    async fn preview_contest_ai_analysis(
        &self,
        raw_text: &str,
    ) -> Result<ContestAiAnalysisPreview, ContestAiAnalysisError>;
    async fn save_contest_ai_analysis(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
        preview: &ContestAiAnalysisPreview,
    ) -> Result<ContestDetail, ContestAiAnalysisError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestCorrectionField {
    FinalContestResult,
    UpsolveDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestCorrectionEvent {
    pub correction_id: String,
    pub problem_index: String,
    pub field: ContestCorrectionField,
    pub old_value: String,
    pub new_value: String,
    pub corrected_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestProblemCorrectionInput {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub final_contest_result: ContestFinalResult,
    pub upsolve_decision: ContestUpsolveDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestCorrectionError {
    Unavailable,
    NotFound,
    FactsNotCompleted,
    NoChange,
}

#[allow(async_fn_in_trait)]
pub trait ContestCorrectionPort {
    async fn correct_contest_problem_facts(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
        correction: &ContestProblemCorrectionInput,
    ) -> Result<ContestDetail, ContestCorrectionError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestFactsStatus {
    Pending,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestFinalResult {
    Unknown,
    NotAttempted,
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    CompilationError,
    OtherFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestUpsolveDecision {
    Planned,
    NotPlanned,
    Undecided,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestProblemDetailItem {
    pub problem: LightweightProblemItem,
    pub final_contest_result: Option<ContestFinalResult>,
    pub upsolve_decision: ContestUpsolveDecision,
    pub live_learning_status: acm_os_domain::LearningStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestProblemFactInput {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub final_contest_result: ContestFinalResult,
    pub upsolve_decision: ContestUpsolveDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestFactsError {
    Unavailable,
    NotFound,
    ImportIncomplete,
    ContestDateMissing,
    ProblemSetMismatch,
    AlreadyCompleted,
}

#[allow(async_fn_in_trait)]
pub trait ContestFactsPort {
    async fn complete_contest_facts(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
        problems: &[ContestProblemFactInput],
    ) -> Result<ContestDetail, ContestFactsError>;
}

pub fn validate_contest_facts_input(
    contest: &acm_os_domain::CodeforcesContestIdentity,
    contest_date: Option<&str>,
    problems: &[ContestProblemFactInput],
) -> Result<(), ContestFactsError> {
    if contest_date.is_none() {
        return Err(ContestFactsError::ContestDateMissing);
    }
    if problems.is_empty() {
        return Err(ContestFactsError::ProblemSetMismatch);
    }
    let mut seen = std::collections::HashSet::new();
    for item in problems {
        if item.problem.contest() != contest || !seen.insert(item.problem.index().to_owned()) {
            return Err(ContestFactsError::ProblemSetMismatch);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightweightProblemItem {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub title: String,
    pub rating: Option<u32>,
    pub has_statement_snapshot: bool,
    pub identity_type: ProblemIdentityType,
}

/// Read-only M1 detail. Source HTML stays archival-only in Infrastructure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightweightProblemDetail {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub title: String,
    pub rating: Option<u32>,
    pub source_url: String,
    pub statement: StatementReadState,
    pub identity_type: ProblemIdentityType,
    pub personal_note: Option<PersonalNoteBinding>,
    pub lifecycle: ProblemLifecycleState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemIdentityType {
    Lightweight,
    Personal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveReviewCycle {
    pub cycle_number: u32,
    pub stage: u32,
    pub schedule_rule_version: u32,
    pub next_due_local_date: acm_os_domain::LocalDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemLifecycleState {
    pub identity_type: ProblemIdentityType,
    pub learning_status: acm_os_domain::LearningStatus,
    pub learning_status_since_utc: String,
    pub active_review_cycle: Option<ActiveReviewCycle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemLifecycleError {
    ProblemNotFound,
    NotPersonal,
    InvalidTransition,
    InvalidLocalDate,
    IntegrityViolation,
    PersistenceUnavailable,
}

impl ProblemLifecycleError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProblemNotFound => "problem_not_found",
            Self::NotPersonal => "problem_not_personal",
            Self::InvalidTransition => "invalid_lifecycle_transition",
            Self::InvalidLocalDate => "invalid_local_date",
            Self::IntegrityViolation => "lifecycle_integrity_violation",
            Self::PersistenceUnavailable => "lifecycle_persistence_unavailable",
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ProblemLifecyclePort {
    async fn load_problem_lifecycle(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<ProblemLifecycleState, ProblemLifecycleError>;

    async fn commit_problem_lifecycle_decision(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        decision: acm_os_domain::ProblemLifecycleDecision,
        first_due: Option<acm_os_domain::LocalDate>,
    ) -> Result<ProblemLifecycleState, ProblemLifecycleError>;
}

pub async fn transition_problem_lifecycle<P: ProblemLifecyclePort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    action: acm_os_domain::ProblemLifecycleAction,
    today: acm_os_domain::LocalDate,
) -> Result<ProblemLifecycleState, ProblemLifecycleError> {
    if action == acm_os_domain::ProblemLifecycleAction::DeletePersonalNote {
        return Err(ProblemLifecycleError::InvalidTransition);
    }
    let current = port.load_problem_lifecycle(problem).await?;
    if current.identity_type != ProblemIdentityType::Personal {
        return Err(ProblemLifecycleError::NotPersonal);
    }
    let decision = acm_os_domain::ProblemLifecycleEngine::decide(current.learning_status, action)
        .map_err(|_| ProblemLifecycleError::InvalidTransition)?;
    let first_due = match decision.review_cycle {
        acm_os_domain::ReviewCycleDirective::StartFirstColdStart => Some(
            acm_os_domain::ReviewSchedulingEngine::first_cold_start_due(today)
                .map_err(|_| ProblemLifecycleError::InvalidLocalDate)?,
        ),
        acm_os_domain::ReviewCycleDirective::None
        | acm_os_domain::ReviewCycleDirective::CancelActive => None,
    };
    port.commit_problem_lifecycle_decision(problem, decision, first_due)
        .await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewAttempt {
    pub attempt_id: String,
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub attempt_type: acm_os_domain::ReviewAttemptType,
    pub scheduled_due_local_date: acm_os_domain::LocalDate,
    pub started_early: bool,
    pub judgement_rule_version: u32,
    pub started_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewFocusView {
    pub attempt: ReviewAttempt,
    pub title: String,
    pub source_url: String,
    pub statement_sanitized_html: String,
    pub statement_assets: Vec<LocalStatementAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewHelpItem {
    pub level: acm_os_domain::ReviewHelpLevel,
    pub available: bool,
    pub revealed_at_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewHelpDrawerView {
    pub attempt_id: String,
    pub items: Vec<ReviewHelpItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealedReviewHelp {
    pub event_id: String,
    pub attempt_id: String,
    pub level: acm_os_domain::ReviewHelpLevel,
    pub title: String,
    pub content_markdown: String,
    pub source_digest: String,
    pub revealed_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionFact {
    pub result: acm_os_domain::SubmissionResult,
    pub other_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCompletionInput {
    pub final_ac: bool,
    pub first_submission: SubmissionFact,
    pub final_submission: SubmissionFact,
    pub total_submissions: u32,
    pub idea_independent: bool,
    pub implementation_independent: bool,
    pub debug_independence: acm_os_domain::DebugIndependence,
    pub external_help: acm_os_domain::ExternalHelpLevel,
    pub failure_reasons: Vec<ReviewFailureReason>,
}

impl ReviewCompletionInput {
    pub fn domain_facts(&self) -> acm_os_domain::ReviewCompletionFacts {
        acm_os_domain::ReviewCompletionFacts {
            final_ac: self.final_ac,
            first_submission_result: self.first_submission.result,
            final_result: self.final_submission.result,
            total_submissions: self.total_submissions,
            idea_independent: self.idea_independent,
            implementation_independent: self.implementation_independent,
            debug_independence: self.debug_independence,
            external_help: self.external_help,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewFailureReason {
    NoIdea,
    KeyPropertyBlocked,
    DerivationBlocked,
    CannotImplement,
    ImplementationError,
    BoundaryError,
    ComplexityError,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCompletionContext {
    pub attempt: ReviewAttempt,
    pub learning_status: acm_os_domain::LearningStatus,
    pub current_stage: u32,
    pub highest_help_level: Option<acm_os_domain::ReviewHelpLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedReviewAttempt {
    pub attempt: ReviewAttempt,
    pub judgement: acm_os_domain::ReviewJudgement,
    pub evidence_codes: Vec<String>,
    pub failure_reasons: Vec<ReviewFailureReason>,
    pub completed_at_utc: String,
    pub completed_local_date: acm_os_domain::LocalDate,
    pub lifecycle: ProblemLifecycleState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewAttemptStatus {
    InProgress,
    Completed,
    Void,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewHistoryItem {
    pub attempt: ReviewAttempt,
    pub status: ReviewAttemptStatus,
    pub judgement: Option<acm_os_domain::ReviewJudgement>,
    pub completion_input: Option<ReviewCompletionInput>,
    pub evidence_codes: Vec<String>,
    pub failure_reasons: Vec<ReviewFailureReason>,
    pub help_levels: Vec<acm_os_domain::ReviewHelpLevel>,
    pub completed_at_utc: Option<String>,
    pub completed_local_date: Option<acm_os_domain::LocalDate>,
    pub void_reason: Option<String>,
    pub voided_at_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewHistoryView {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub historical_best_review: Option<acm_os_domain::ReviewJudgement>,
    pub mastery: ProblemMasteryProjection,
    pub attempts: Vec<ReviewHistoryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemMasteryProjection {
    pub current: acm_os_domain::ProblemMasteryEvidence,
    pub historical_thoroughly_digested: bool,
    pub first_thoroughly_digested_local_date: Option<acm_os_domain::LocalDate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewAttemptError {
    ProblemNotFound,
    AttemptNotFound,
    NotPersonal,
    NotEligible,
    ScheduleMissing,
    StatementMissing,
    HelpContentUnavailable,
    HelpConfirmationRequired,
    NoteUnavailable,
    InvalidMarkdown,
    InvalidCompletionFacts,
    FailureReasonRequired,
    AttemptAlreadyFinished,
    InvalidVoidReason,
    IntegrityViolation,
    PersistenceUnavailable,
}

impl ReviewAttemptError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProblemNotFound => "problem_not_found",
            Self::AttemptNotFound => "review_attempt_not_found",
            Self::NotPersonal => "problem_not_personal",
            Self::NotEligible => "review_not_eligible",
            Self::ScheduleMissing => "review_schedule_missing",
            Self::StatementMissing => "review_statement_missing",
            Self::HelpContentUnavailable => "review_help_content_unavailable",
            Self::HelpConfirmationRequired => "review_help_confirmation_required",
            Self::NoteUnavailable => "review_note_unavailable",
            Self::InvalidMarkdown => "review_note_invalid_utf8",
            Self::InvalidCompletionFacts => "review_completion_facts_invalid",
            Self::FailureReasonRequired => "review_failure_reason_required",
            Self::AttemptAlreadyFinished => "review_attempt_already_finished",
            Self::InvalidVoidReason => "review_void_reason_invalid",
            Self::IntegrityViolation => "review_integrity_violation",
            Self::PersistenceUnavailable => "review_persistence_unavailable",
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ReviewAttemptPort {
    async fn load_in_progress_review_attempt(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<Option<ReviewAttempt>, ReviewAttemptError>;

    async fn create_or_resume_review_attempt(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        eligibility: acm_os_domain::ReviewEligibilityDecision,
    ) -> Result<ReviewAttempt, ReviewAttemptError>;

    async fn load_review_focus(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewFocusView, ReviewAttemptError>;

    async fn load_review_help_drawer(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewHelpDrawerView, ReviewAttemptError>;

    async fn reveal_review_help(
        &self,
        attempt_id: &str,
        level: acm_os_domain::ReviewHelpLevel,
        impact_acknowledged: bool,
    ) -> Result<RevealedReviewHelp, ReviewAttemptError>;

    async fn load_review_completion_context(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewCompletionContext, ReviewAttemptError>;

    async fn commit_review_completion(
        &self,
        context: &ReviewCompletionContext,
        input: &ReviewCompletionInput,
        judgement: &acm_os_domain::ReviewJudgementDecision,
        scheduling: acm_os_domain::ReviewCompletionDecision,
        completed_on: acm_os_domain::LocalDate,
    ) -> Result<CompletedReviewAttempt, ReviewAttemptError>;

    async fn void_review_attempt(
        &self,
        attempt_id: &str,
        reason: &str,
    ) -> Result<ReviewHistoryItem, ReviewAttemptError>;

    async fn load_review_attempt_history_item(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewHistoryItem, ReviewAttemptError>;

    async fn load_review_history(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<ReviewHistoryView, ReviewAttemptError>;

    async fn update_problem_mastery_evidence(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        evidence: acm_os_domain::ProblemMasteryEvidence,
        confirmed_on: acm_os_domain::LocalDate,
    ) -> Result<ProblemMasteryProjection, ReviewAttemptError>;
}

pub async fn start_or_resume_review<P: ProblemLifecyclePort + ReviewAttemptPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    today: acm_os_domain::LocalDate,
) -> Result<ReviewAttempt, ReviewAttemptError> {
    if let Some(attempt) = port.load_in_progress_review_attempt(problem).await? {
        return Ok(attempt);
    }
    let lifecycle =
        port.load_problem_lifecycle(problem)
            .await
            .map_err(|error| match error {
                ProblemLifecycleError::ProblemNotFound => ReviewAttemptError::ProblemNotFound,
                ProblemLifecycleError::NotPersonal => ReviewAttemptError::NotPersonal,
                ProblemLifecycleError::IntegrityViolation => ReviewAttemptError::IntegrityViolation,
                ProblemLifecycleError::PersistenceUnavailable => {
                    ReviewAttemptError::PersistenceUnavailable
                }
                ProblemLifecycleError::InvalidTransition
                | ProblemLifecycleError::InvalidLocalDate => ReviewAttemptError::NotEligible,
            })?;
    if lifecycle.identity_type != ProblemIdentityType::Personal {
        return Err(ReviewAttemptError::NotPersonal);
    }
    let cycle = lifecycle
        .active_review_cycle
        .ok_or(ReviewAttemptError::ScheduleMissing)?;
    let eligibility = acm_os_domain::ReviewEligibilityEngine::decide(
        lifecycle.learning_status,
        cycle.next_due_local_date,
        today,
    )
    .map_err(|_| ReviewAttemptError::NotEligible)?;
    port.create_or_resume_review_attempt(problem, eligibility)
        .await
}

pub async fn review_focus<P: ReviewAttemptPort>(
    port: &P,
    attempt_id: &str,
) -> Result<ReviewFocusView, ReviewAttemptError> {
    port.load_review_focus(attempt_id).await
}

pub async fn review_help_drawer<P: ReviewAttemptPort>(
    port: &P,
    attempt_id: &str,
) -> Result<ReviewHelpDrawerView, ReviewAttemptError> {
    port.load_review_help_drawer(attempt_id).await
}

pub async fn reveal_review_help<P: ReviewAttemptPort>(
    port: &P,
    attempt_id: &str,
    level: acm_os_domain::ReviewHelpLevel,
    impact_acknowledged: bool,
) -> Result<RevealedReviewHelp, ReviewAttemptError> {
    port.reveal_review_help(attempt_id, level, impact_acknowledged)
        .await
}

pub async fn complete_review<P: ReviewAttemptPort>(
    port: &P,
    attempt_id: &str,
    input: ReviewCompletionInput,
    completed_on: acm_os_domain::LocalDate,
) -> Result<CompletedReviewAttempt, ReviewAttemptError> {
    validate_completion_text(&input)?;
    let context = port.load_review_completion_context(attempt_id).await?;
    let judgement = acm_os_domain::ReviewJudgementEngine::judge(
        &input.domain_facts(),
        context.highest_help_level,
    )
    .map_err(|_| ReviewAttemptError::InvalidCompletionFacts)?;
    if judgement.judgement != acm_os_domain::ReviewJudgement::Mastered
        && input.failure_reasons.is_empty()
    {
        return Err(ReviewAttemptError::FailureReasonRequired);
    }
    if judgement.judgement == acm_os_domain::ReviewJudgement::Mastered
        && !input.failure_reasons.is_empty()
    {
        return Err(ReviewAttemptError::InvalidCompletionFacts);
    }
    let scheduling = acm_os_domain::ReviewSchedulingEngine::complete_review(
        context.learning_status,
        context.attempt.attempt_type,
        judgement.judgement,
        context.current_stage,
        completed_on,
    )
    .map_err(|_| ReviewAttemptError::IntegrityViolation)?;
    port.commit_review_completion(&context, &input, &judgement, scheduling, completed_on)
        .await
}

fn validate_completion_text(input: &ReviewCompletionInput) -> Result<(), ReviewAttemptError> {
    for submission in [&input.first_submission, &input.final_submission] {
        match (submission.result, submission.other_text.as_deref()) {
            (acm_os_domain::SubmissionResult::Other, Some(text))
                if !text.trim().is_empty() && text.len() <= 120 => {}
            (acm_os_domain::SubmissionResult::Other, _) => {
                return Err(ReviewAttemptError::InvalidCompletionFacts);
            }
            (_, None) => {}
            (_, Some(_)) => return Err(ReviewAttemptError::InvalidCompletionFacts),
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for reason in &input.failure_reasons {
        let key = match reason {
            ReviewFailureReason::NoIdea => "no_idea",
            ReviewFailureReason::KeyPropertyBlocked => "key_property_blocked",
            ReviewFailureReason::DerivationBlocked => "derivation_blocked",
            ReviewFailureReason::CannotImplement => "cannot_implement",
            ReviewFailureReason::ImplementationError => "implementation_error",
            ReviewFailureReason::BoundaryError => "boundary_error",
            ReviewFailureReason::ComplexityError => "complexity_error",
            ReviewFailureReason::Other(text) if !text.trim().is_empty() && text.len() <= 500 => {
                "other"
            }
            ReviewFailureReason::Other(_) => {
                return Err(ReviewAttemptError::InvalidCompletionFacts)
            }
        };
        if !seen.insert(key) {
            return Err(ReviewAttemptError::InvalidCompletionFacts);
        }
    }
    Ok(())
}

pub async fn void_review<P: ReviewAttemptPort>(
    port: &P,
    attempt_id: &str,
    reason: &str,
) -> Result<ReviewHistoryItem, ReviewAttemptError> {
    if reason.trim().is_empty() || reason.len() > 500 {
        return Err(ReviewAttemptError::InvalidVoidReason);
    }
    port.void_review_attempt(attempt_id, reason.trim()).await
}

pub async fn review_attempt_history_item<P: ReviewAttemptPort>(
    port: &P,
    attempt_id: &str,
) -> Result<ReviewHistoryItem, ReviewAttemptError> {
    port.load_review_attempt_history_item(attempt_id).await
}

pub async fn review_history<P: ReviewAttemptPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<ReviewHistoryView, ReviewAttemptError> {
    port.load_review_history(problem).await
}

pub async fn update_problem_mastery_evidence<P: ReviewAttemptPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    evidence: acm_os_domain::ProblemMasteryEvidence,
    confirmed_on: acm_os_domain::LocalDate,
) -> Result<ProblemMasteryProjection, ReviewAttemptError> {
    port.update_problem_mastery_evidence(problem, evidence, confirmed_on)
        .await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayGenerationCandidate {
    pub problem_id: String,
    pub contest_id: u64,
    pub problem_index: String,
    pub problem_title: String,
    pub learning_status: acm_os_domain::LearningStatus,
    pub learning_status_since: acm_os_domain::LocalDate,
    pub pinned: bool,
    pub active_review_due: Option<acm_os_domain::LocalDate>,
    pub in_progress_review_attempt_id: Option<String>,
    pub in_progress_review_due: Option<acm_os_domain::LocalDate>,
    pub available_for_today: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayGenerationContext {
    pub candidates: Vec<TodayGenerationCandidate>,
    pub prior_review_only_streak: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodaySnapshotEntry {
    pub entry_id: String,
    pub problem_id: String,
    pub contest_id: u64,
    pub problem_index: String,
    pub problem_title: String,
    pub review_attempt_id: Option<String>,
    pub lane: acm_os_domain::TodayCandidateLane,
    pub reason: acm_os_domain::TodayCandidateReason,
    pub planning_cost_minutes: u32,
    pub position: u32,
    pub origin: TodayEntryOrigin,
    pub status: TodayEntryStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodayEntryOrigin {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodayEntryStatus {
    NotStarted,
    InProgress,
    Completed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodaySnapshot {
    pub plan_id: String,
    pub local_date: acm_os_domain::LocalDate,
    pub budget_minutes: u32,
    pub planned_minutes: u32,
    pub over_budget_minutes: u32,
    pub review_only_streak: u8,
    pub entries: Vec<TodaySnapshotEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayReplanEntry {
    pub existing_entry_id: Option<String>,
    pub problem_id: String,
    pub review_attempt_id: Option<String>,
    pub lane: acm_os_domain::TodayCandidateLane,
    pub reason: acm_os_domain::TodayCandidateReason,
    pub planning_cost_minutes: u32,
    pub origin: TodayEntryOrigin,
    pub status: TodayEntryStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayReplanPreview {
    pub expected_snapshot: TodaySnapshot,
    pub proposed_budget_minutes: u32,
    pub proposed_planned_minutes: u32,
    pub proposed_over_budget_minutes: u32,
    pub proposed_review_only_streak: u8,
    pub entries: Vec<TodayReplanEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayExtraSuggestion {
    pub problem_id: String,
    pub contest_id: u64,
    pub problem_index: String,
    pub problem_title: String,
    pub review_attempt_id: Option<String>,
    pub lane: acm_os_domain::TodayCandidateLane,
    pub reason: acm_os_domain::TodayCandidateReason,
    pub planning_cost_minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayExtraSuggestionsPreview {
    pub expected_snapshot: TodaySnapshot,
    pub remaining_budget_minutes: u32,
    pub suggestions: Vec<TodayExtraSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklyAcmBudgetSchedule {
    pub monday: Option<u32>,
    pub tuesday: Option<u32>,
    pub wednesday: Option<u32>,
    pub thursday: Option<u32>,
    pub friday: Option<u32>,
    pub saturday: Option<u32>,
    pub sunday: Option<u32>,
}

impl WeeklyAcmBudgetSchedule {
    pub fn budget_for_iso_weekday(&self, weekday: u8) -> Option<u32> {
        match weekday {
            1 => self.monday,
            2 => self.tuesday,
            3 => self.wednesday,
            4 => self.thursday,
            5 => self.friday,
            6 => self.saturday,
            7 => self.sunday,
            _ => None,
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait WeeklyAcmBudgetPort {
    async fn load_weekly_acm_budget(&self) -> Result<WeeklyAcmBudgetSchedule, TodaySnapshotError>;

    async fn save_weekly_acm_budget(
        &self,
        schedule: &WeeklyAcmBudgetSchedule,
    ) -> Result<WeeklyAcmBudgetSchedule, TodaySnapshotError>;
}

pub async fn load_weekly_acm_budget<P: WeeklyAcmBudgetPort>(
    port: &P,
) -> Result<WeeklyAcmBudgetSchedule, TodaySnapshotError> {
    port.load_weekly_acm_budget().await
}

pub async fn save_weekly_acm_budget<P: WeeklyAcmBudgetPort>(
    port: &P,
    schedule: &WeeklyAcmBudgetSchedule,
) -> Result<WeeklyAcmBudgetSchedule, TodaySnapshotError> {
    port.save_weekly_acm_budget(schedule).await
}

pub fn weekly_acm_budget_for_date(
    schedule: &WeeklyAcmBudgetSchedule,
    local_date: acm_os_domain::LocalDate,
) -> Option<u32> {
    schedule.budget_for_iso_weekday(local_date.iso_weekday_number())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodaySnapshotError {
    PersistenceUnavailable,
    IntegrityViolation,
    InvalidReorder,
    InvalidTodayDone,
    InvalidExtraSuggestion,
    StaleExtraSuggestions,
    StaleReplanPreview,
    Candidate(acm_os_domain::TodayCandidateError),
    Ordering(acm_os_domain::TodayCandidateOrderingError),
    Planning(acm_os_domain::TodayPlannerError),
}

#[allow(async_fn_in_trait)]
pub trait TodaySnapshotPort {
    async fn load_today_snapshot(
        &self,
        local_date: acm_os_domain::LocalDate,
    ) -> Result<Option<TodaySnapshot>, TodaySnapshotError>;

    async fn load_today_generation_context(
        &self,
        local_date: acm_os_domain::LocalDate,
    ) -> Result<TodayGenerationContext, TodaySnapshotError>;

    /// Projects authoritative work started or completed outside Today into
    /// the stable snapshot without regenerating ordinary recommendations.
    async fn reconcile_today_snapshot(
        &self,
        local_date: acm_os_domain::LocalDate,
    ) -> Result<TodaySnapshot, TodaySnapshotError>;

    async fn reorder_today_snapshot(
        &self,
        plan_id: &str,
        ordered_entry_ids: &[String],
    ) -> Result<TodaySnapshot, TodaySnapshotError>;

    async fn complete_today_entry(
        &self,
        plan_id: &str,
        entry_id: &str,
    ) -> Result<TodaySnapshot, TodaySnapshotError>;

    async fn add_manual_today_entry(
        &self,
        expected_snapshot: &TodaySnapshot,
        suggestion: &TodayExtraSuggestion,
    ) -> Result<TodaySnapshot, TodaySnapshotError>;

    async fn apply_today_replan(
        &self,
        preview: &TodayReplanPreview,
    ) -> Result<TodaySnapshot, TodaySnapshotError>;

    /// Atomically creates a complete Plan + Entries snapshot. If another
    /// caller won the same-date race, implementations return the winner.
    async fn create_or_load_today_snapshot(
        &self,
        local_date: acm_os_domain::LocalDate,
        draft: &acm_os_domain::TodayPlanDraft,
    ) -> Result<TodaySnapshot, TodaySnapshotError>;
}

pub async fn preview_today_replan<P: TodaySnapshotPort>(
    port: &P,
    local_date: acm_os_domain::LocalDate,
    proposed_budget_minutes: u32,
) -> Result<TodayReplanPreview, TodaySnapshotError> {
    let snapshot = port
        .load_today_snapshot(local_date)
        .await?
        .ok_or(TodaySnapshotError::IntegrityViolation)?;
    let context = port.load_today_generation_context(local_date).await?;
    let protected = snapshot
        .entries
        .iter()
        .filter(|entry| {
            entry.origin == TodayEntryOrigin::Manual || entry.status != TodayEntryStatus::NotStarted
        })
        .cloned()
        .collect::<Vec<_>>();
    let protected_problem_ids = protected
        .iter()
        .map(|entry| entry.problem_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let candidates = build_today_candidates(
        local_date,
        &context
            .candidates
            .iter()
            .filter(|candidate| !protected_problem_ids.contains(candidate.problem_id.as_str()))
            .cloned()
            .collect::<Vec<_>>(),
    )?;
    let ordered = acm_os_domain::TodayCandidateOrderingEngine::order(&candidates)
        .map_err(TodaySnapshotError::Ordering)?;
    let protected_minutes = protected.iter().try_fold(0_u32, |total, entry| {
        total
            .checked_add(entry.planning_cost_minutes)
            .ok_or(TodaySnapshotError::IntegrityViolation)
    })?;
    let remaining_budget = proposed_budget_minutes.saturating_sub(protected_minutes);
    let draft = acm_os_domain::TodayPlanner::plan_generated(
        &ordered,
        remaining_budget,
        context.prior_review_only_streak,
    )
    .map_err(TodaySnapshotError::Planning)?;
    let proposed_planned_minutes = protected_minutes
        .checked_add(draft.planned_minutes)
        .ok_or(TodaySnapshotError::IntegrityViolation)?;
    let mut entries = protected
        .into_iter()
        .map(|entry| TodayReplanEntry {
            existing_entry_id: Some(entry.entry_id),
            problem_id: entry.problem_id,
            review_attempt_id: entry.review_attempt_id,
            lane: entry.lane,
            reason: entry.reason,
            planning_cost_minutes: entry.planning_cost_minutes,
            origin: entry.origin,
            status: entry.status,
        })
        .collect::<Vec<_>>();
    entries.extend(draft.entries.into_iter().map(|entry| TodayReplanEntry {
        existing_entry_id: None,
        problem_id: entry.problem_id,
        review_attempt_id: entry.review_attempt_id,
        lane: entry.lane,
        reason: entry.reason,
        planning_cost_minutes: entry.planning_cost_minutes,
        origin: TodayEntryOrigin::Auto,
        status: if entry.lane == acm_os_domain::TodayCandidateLane::CarryIn {
            TodayEntryStatus::InProgress
        } else {
            TodayEntryStatus::NotStarted
        },
    }));
    Ok(TodayReplanPreview {
        expected_snapshot: snapshot,
        proposed_budget_minutes,
        proposed_planned_minutes,
        proposed_over_budget_minutes: proposed_planned_minutes
            .saturating_sub(proposed_budget_minutes),
        proposed_review_only_streak: draft.next_review_only_streak,
        entries,
    })
}

pub async fn apply_today_replan<P: TodaySnapshotPort>(
    port: &P,
    preview: &TodayReplanPreview,
) -> Result<TodaySnapshot, TodaySnapshotError> {
    let authoritative = preview_today_replan(
        port,
        preview.expected_snapshot.local_date,
        preview.proposed_budget_minutes,
    )
    .await?;
    if authoritative != *preview {
        return Err(TodaySnapshotError::StaleReplanPreview);
    }
    port.apply_today_replan(preview).await
}

pub async fn reorder_today_snapshot<P: TodaySnapshotPort>(
    port: &P,
    plan_id: &str,
    ordered_entry_ids: &[String],
) -> Result<TodaySnapshot, TodaySnapshotError> {
    if plan_id.is_empty() {
        return Err(TodaySnapshotError::InvalidReorder);
    }
    let unique = ordered_entry_ids
        .iter()
        .collect::<std::collections::HashSet<_>>();
    if unique.len() != ordered_entry_ids.len() || ordered_entry_ids.iter().any(String::is_empty) {
        return Err(TodaySnapshotError::InvalidReorder);
    }
    port.reorder_today_snapshot(plan_id, ordered_entry_ids)
        .await
}

pub async fn complete_today_entry<P: TodaySnapshotPort>(
    port: &P,
    plan_id: &str,
    entry_id: &str,
) -> Result<TodaySnapshot, TodaySnapshotError> {
    if plan_id.is_empty() || entry_id.is_empty() {
        return Err(TodaySnapshotError::InvalidTodayDone);
    }
    port.complete_today_entry(plan_id, entry_id).await
}

pub async fn preview_today_extra_suggestions<P: TodaySnapshotPort>(
    port: &P,
    local_date: acm_os_domain::LocalDate,
) -> Result<TodayExtraSuggestionsPreview, TodaySnapshotError> {
    let snapshot = port.reconcile_today_snapshot(local_date).await?;
    let remaining_budget_minutes = snapshot
        .budget_minutes
        .saturating_sub(snapshot.planned_minutes);
    if snapshot
        .entries
        .iter()
        .any(|entry| entry.status != TodayEntryStatus::Completed)
        || remaining_budget_minutes == 0
    {
        return Ok(TodayExtraSuggestionsPreview {
            expected_snapshot: snapshot,
            remaining_budget_minutes,
            suggestions: Vec::new(),
        });
    }

    let existing_problem_ids = snapshot
        .entries
        .iter()
        .map(|entry| entry.problem_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let context = port.load_today_generation_context(local_date).await?;
    let candidates = build_today_candidates(
        local_date,
        &context
            .candidates
            .iter()
            .filter(|candidate| {
                candidate.available_for_today
                    && !existing_problem_ids.contains(candidate.problem_id.as_str())
            })
            .cloned()
            .collect::<Vec<_>>(),
    )?;
    let ordered = acm_os_domain::TodayCandidateOrderingEngine::order(&candidates)
        .map_err(TodaySnapshotError::Ordering)?;
    let candidate_details = context
        .candidates
        .iter()
        .map(|candidate| (candidate.problem_id.as_str(), candidate))
        .collect::<std::collections::HashMap<_, _>>();
    let suggestions = ordered
        .carry_in
        .into_iter()
        .chain(ordered.review)
        .chain(ordered.study)
        .filter(|candidate| candidate.planning_cost_minutes <= remaining_budget_minutes)
        .map(|candidate| {
            let details = candidate_details
                .get(candidate.problem_id.as_str())
                .ok_or(TodaySnapshotError::IntegrityViolation)?;
            Ok(TodayExtraSuggestion {
                problem_id: candidate.problem_id,
                contest_id: details.contest_id,
                problem_index: details.problem_index.clone(),
                problem_title: details.problem_title.clone(),
                review_attempt_id: candidate.review_attempt_id,
                lane: candidate.lane,
                reason: candidate.reason,
                planning_cost_minutes: candidate.planning_cost_minutes,
            })
        })
        .collect::<Result<Vec<_>, TodaySnapshotError>>()?;
    Ok(TodayExtraSuggestionsPreview {
        expected_snapshot: snapshot,
        remaining_budget_minutes,
        suggestions,
    })
}

pub async fn accept_today_extra_suggestion<P: TodaySnapshotPort>(
    port: &P,
    preview: &TodayExtraSuggestionsPreview,
    problem_id: &str,
) -> Result<TodaySnapshot, TodaySnapshotError> {
    if problem_id.is_empty() {
        return Err(TodaySnapshotError::InvalidExtraSuggestion);
    }
    let current =
        preview_today_extra_suggestions(port, preview.expected_snapshot.local_date).await?;
    if current.expected_snapshot != preview.expected_snapshot {
        return Err(TodaySnapshotError::StaleExtraSuggestions);
    }
    let suggestion = current
        .suggestions
        .iter()
        .find(|suggestion| suggestion.problem_id == problem_id)
        .ok_or(TodaySnapshotError::InvalidExtraSuggestion)?;
    port.add_manual_today_entry(&current.expected_snapshot, suggestion)
        .await
}

pub async fn load_or_generate_today_snapshot<P: TodaySnapshotPort>(
    port: &P,
    local_date: acm_os_domain::LocalDate,
    budget_minutes: u32,
) -> Result<TodaySnapshot, TodaySnapshotError> {
    if let Some(existing) = port.load_today_snapshot(local_date).await? {
        return port.reconcile_today_snapshot(existing.local_date).await;
    }

    let context = port.load_today_generation_context(local_date).await?;
    let candidates = build_today_candidates(local_date, &context.candidates)?;
    let ordered = acm_os_domain::TodayCandidateOrderingEngine::order(&candidates)
        .map_err(TodaySnapshotError::Ordering)?;
    let draft = acm_os_domain::TodayPlanner::plan_generated(
        &ordered,
        budget_minutes,
        context.prior_review_only_streak,
    )
    .map_err(TodaySnapshotError::Planning)?;
    port.create_or_load_today_snapshot(local_date, &draft).await
}

fn build_today_candidates(
    local_date: acm_os_domain::LocalDate,
    candidates: &[TodayGenerationCandidate],
) -> Result<Vec<acm_os_domain::TodayCandidate>, TodaySnapshotError> {
    let inputs = candidates
        .iter()
        .map(|candidate| {
            let in_progress_review = match (
                candidate.in_progress_review_attempt_id.as_deref(),
                candidate.in_progress_review_due,
            ) {
                (None, None) => Ok(None),
                (Some(attempt_id), Some(scheduled_due_local_date)) => {
                    Ok(Some(acm_os_domain::TodayInProgressReview {
                        attempt_id,
                        scheduled_due_local_date,
                    }))
                }
                _ => Err(TodaySnapshotError::IntegrityViolation),
            }?;
            Ok(acm_os_domain::TodayCandidateInput {
                problem_id: &candidate.problem_id,
                learning_status: candidate.learning_status,
                learning_status_since: candidate.learning_status_since,
                pinned: candidate.pinned,
                active_review_due: candidate.active_review_due,
                in_progress_review,
            })
        })
        .collect::<Result<Vec<_>, TodaySnapshotError>>()?;
    acm_os_domain::TodayCandidateBuilder::build(local_date, &inputs)
        .map_err(TodaySnapshotError::Candidate)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalNoteBinding {
    pub vault_relative_path: String,
    pub content_digest: String,
    pub windows_file_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemMarkdownProjection {
    pub content_digest: String,
    pub known_sections: Vec<KnownMarkdownSection>,
    pub solution_routes: Vec<SolutionRoute>,
    pub warnings: Vec<MarkdownParseWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownMarkdownSection {
    pub name: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionRoute {
    pub name: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownParseWarning {
    DuplicateKnownSection { name: String, count: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalNoteReadError {
    ProblemNotFound,
    NotPersonal,
    BindingUnavailable,
    FileReadFailed,
    InvalidUtf8,
    PersistenceUnavailable,
}

impl PersonalNoteReadError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProblemNotFound => "problem_not_found",
            Self::NotPersonal => "problem_not_personal",
            Self::BindingUnavailable => "note_binding_unavailable",
            Self::FileReadFailed => "note_read_failed",
            Self::InvalidUtf8 => "note_invalid_utf8",
            Self::PersistenceUnavailable => "note_persistence_unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalNoteReadState {
    Ready {
        binding: PersonalNoteBinding,
        projection: ProblemMarkdownProjection,
        relocated: bool,
    },
    LocationAnomaly {
        binding: PersonalNoteBinding,
    },
    VaultUnavailable {
        binding: PersonalNoteBinding,
    },
}

#[allow(async_fn_in_trait)]
pub trait PersonalNoteReadPort {
    async fn read_personal_note_projection(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<PersonalNoteReadState, PersonalNoteReadError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalNoteRelocationCandidate {
    pub vault_relative_path: String,
    pub occupied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalNoteBindingRepairError {
    ProblemNotFound,
    NotPersonal,
    LocationAnomalyRequired,
    VaultUnavailable,
    CandidateUnavailable,
    CandidateOccupied,
    ReviewInProgress,
    IntegrityViolation,
    PersistenceUnavailable,
}

impl PersonalNoteBindingRepairError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProblemNotFound => "problem_not_found",
            Self::NotPersonal => "problem_not_personal",
            Self::LocationAnomalyRequired => "note_location_anomaly_required",
            Self::VaultUnavailable => "vault_unavailable",
            Self::CandidateUnavailable => "note_relocation_candidate_unavailable",
            Self::CandidateOccupied => "note_relocation_candidate_occupied",
            Self::ReviewInProgress => "review_in_progress",
            Self::IntegrityViolation => "note_delete_integrity_violation",
            Self::PersistenceUnavailable => "note_persistence_unavailable",
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait PersonalNoteBindingRepairPort {
    async fn personal_note_relocation_candidates(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<Vec<PersonalNoteRelocationCandidate>, PersonalNoteBindingRepairError>;

    async fn rebind_personal_note(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        vault_relative_path: &str,
    ) -> Result<PersonalNoteBinding, PersonalNoteBindingRepairError>;

    async fn confirm_personal_note_deleted(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<ProblemLifecycleState, PersonalNoteBindingRepairError>;
}

pub async fn personal_note_relocation_candidates<P: PersonalNoteBindingRepairPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<Vec<PersonalNoteRelocationCandidate>, PersonalNoteBindingRepairError> {
    port.personal_note_relocation_candidates(problem).await
}

pub async fn rebind_personal_note<P: PersonalNoteBindingRepairPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    vault_relative_path: impl AsRef<str>,
) -> Result<PersonalNoteBinding, PersonalNoteBindingRepairError> {
    port.rebind_personal_note(problem, vault_relative_path.as_ref())
        .await
}

pub async fn confirm_personal_note_deleted<P: PersonalNoteBindingRepairPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<ProblemLifecycleState, PersonalNoteBindingRepairError> {
    port.confirm_personal_note_deleted(problem).await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraProblemLinkTarget(String);

impl ExtraProblemLinkTarget {
    pub fn parse(value: impl Into<String>) -> Result<Self, PersonalNotePatchError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 512
            || value.trim() != value
            || value.chars().any(char::is_control)
            || value.contains("[[")
            || value.contains("]]")
            || value.contains('|')
        {
            return Err(PersonalNotePatchError::InvalidLinkTarget);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrerequisiteLinkTarget(String);

impl PrerequisiteLinkTarget {
    pub fn parse(value: impl Into<String>) -> Result<Self, PersonalNotePatchError> {
        ExtraProblemLinkTarget::parse(value).map(|value| Self(value.0))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalNotePatchError {
    InvalidLinkTarget,
    ProblemNotFound,
    NotPersonal,
    BindingUnavailable,
    LocationAnomaly,
    VaultUnavailable,
    InvalidUtf8,
    TargetSectionMissing,
    TargetSectionAmbiguous,
    LinkAlreadyPresent,
    ConcurrentModification,
    RecoveryCopyFailed,
    WriteFailed,
    VerificationFailed,
    PersistenceUnavailable,
}

impl PersonalNotePatchError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidLinkTarget => "invalid_extra_problem_link_target",
            Self::ProblemNotFound => "problem_not_found",
            Self::NotPersonal => "problem_not_personal",
            Self::BindingUnavailable => "note_binding_unavailable",
            Self::LocationAnomaly => "note_location_anomaly",
            Self::VaultUnavailable => "vault_unavailable",
            Self::InvalidUtf8 => "note_invalid_utf8",
            Self::TargetSectionMissing => "markdown_target_section_missing",
            Self::TargetSectionAmbiguous => "markdown_target_section_ambiguous",
            Self::LinkAlreadyPresent => "extra_problem_link_already_present",
            Self::ConcurrentModification => "markdown_concurrent_modification",
            Self::RecoveryCopyFailed => "markdown_recovery_copy_failed",
            Self::WriteFailed => "markdown_write_failed",
            Self::VerificationFailed => "markdown_verification_failed",
            Self::PersistenceUnavailable => "note_persistence_unavailable",
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait PersonalNotePatchPort {
    async fn add_prerequisite_link(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        target: &PrerequisiteLinkTarget,
    ) -> Result<PersonalNoteBinding, PersonalNotePatchError>;

    async fn add_extra_problem_link(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        target: &ExtraProblemLinkTarget,
    ) -> Result<PersonalNoteBinding, PersonalNotePatchError>;
}

pub async fn add_prerequisite_link<P: PersonalNotePatchPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    target: impl Into<String>,
) -> Result<PersonalNoteBinding, PersonalNotePatchError> {
    let target = PrerequisiteLinkTarget::parse(target)?;
    port.add_prerequisite_link(problem, &target).await
}

pub async fn add_extra_problem_link<P: PersonalNotePatchPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    target: impl Into<String>,
) -> Result<PersonalNoteBinding, PersonalNotePatchError> {
    let target = ExtraProblemLinkTarget::parse(target)?;
    port.add_extra_problem_link(problem, &target).await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalNoteCreationContext {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub existing_binding: Option<PersonalNoteBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedPersonalNoteFile {
    pub vault_relative_path: String,
    pub content_digest: String,
    pub windows_file_key: Option<String>,
}

impl From<CreatedPersonalNoteFile> for PersonalNoteBinding {
    fn from(value: CreatedPersonalNoteFile) -> Self {
        Self {
            vault_relative_path: value.vault_relative_path,
            content_digest: value.content_digest,
            windows_file_key: value.windows_file_key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalNoteError {
    ProblemNotFound,
    WorkspaceUnavailable,
    TargetAlreadyExists,
    FileWriteFailed,
    FileVerificationFailed,
    PersistenceUnavailable,
    CompensationFailed,
}

impl PersonalNoteError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProblemNotFound => "problem_not_found",
            Self::WorkspaceUnavailable => "workspace_unavailable",
            Self::TargetAlreadyExists => "note_target_exists",
            Self::FileWriteFailed => "note_write_failed",
            Self::FileVerificationFailed => "note_verification_failed",
            Self::PersistenceUnavailable => "note_persistence_unavailable",
            Self::CompensationFailed => "note_compensation_failed",
        }
    }
}

pub const INITIAL_PROBLEM_MARKDOWN: &str =
    "# Problem\n\n## 前置知识\n\n## 题解\n\n### 标准推导\n\n## 额外题目\n";

#[allow(async_fn_in_trait)]
pub trait PersonalNotePort {
    async fn personal_note_creation_context(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<PersonalNoteCreationContext, PersonalNoteError>;

    async fn create_personal_note_file(
        &self,
        context: &PersonalNoteCreationContext,
        markdown: &[u8],
    ) -> Result<CreatedPersonalNoteFile, PersonalNoteError>;

    async fn commit_personal_note_binding(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        file: &CreatedPersonalNoteFile,
    ) -> Result<PersonalNoteBinding, PersonalNoteError>;

    async fn discard_created_personal_note(
        &self,
        file: &CreatedPersonalNoteFile,
    ) -> Result<(), PersonalNoteError>;
}

pub async fn create_personal_note<P: PersonalNotePort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<PersonalNoteBinding, PersonalNoteError> {
    let context = port.personal_note_creation_context(problem).await?;
    if let Some(binding) = context.existing_binding {
        return Ok(binding);
    }

    let file = port
        .create_personal_note_file(&context, INITIAL_PROBLEM_MARKDOWN.as_bytes())
        .await?;
    match port.commit_personal_note_binding(problem, &file).await {
        Ok(binding) => Ok(binding),
        Err(error) => match port.discard_created_personal_note(&file).await {
            Ok(()) => Err(error),
            Err(_) => Err(PersonalNoteError::CompensationFailed),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPersonalNoteDeletion {
    pub vault_relative_path: String,
    pub content_digest: String,
    pub recovery_copy_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalNoteDeletionError {
    ProblemNotFound,
    NotPersonal,
    BindingUnavailable,
    LocationAnomaly,
    VaultUnavailable,
    ConcurrentModification,
    ReviewInProgress,
    RecoveryCopyFailed,
    FileDeleteFailed,
    PersistenceUnavailable,
    CompensationFailed,
    IntegrityViolation,
}

impl PersonalNoteDeletionError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProblemNotFound => "problem_not_found",
            Self::NotPersonal => "problem_not_personal",
            Self::BindingUnavailable => "note_binding_unavailable",
            Self::LocationAnomaly => "note_location_anomaly",
            Self::VaultUnavailable => "vault_unavailable",
            Self::ConcurrentModification => "markdown_concurrent_modification",
            Self::ReviewInProgress => "review_in_progress",
            Self::RecoveryCopyFailed => "markdown_recovery_copy_failed",
            Self::FileDeleteFailed => "note_delete_failed",
            Self::PersistenceUnavailable => "note_persistence_unavailable",
            Self::CompensationFailed => "note_delete_compensation_failed",
            Self::IntegrityViolation => "note_delete_integrity_violation",
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait PersonalNoteDeletionPort {
    async fn prepare_personal_note_deletion(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<PreparedPersonalNoteDeletion, PersonalNoteDeletionError>;

    async fn commit_personal_note_deletion(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        prepared: &PreparedPersonalNoteDeletion,
    ) -> Result<ProblemLifecycleState, PersonalNoteDeletionError>;

    async fn restore_deleted_personal_note(
        &self,
        prepared: &PreparedPersonalNoteDeletion,
    ) -> Result<(), PersonalNoteDeletionError>;
}

pub async fn delete_personal_note<P: PersonalNoteDeletionPort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<ProblemLifecycleState, PersonalNoteDeletionError> {
    let prepared = port.prepare_personal_note_deletion(problem).await?;
    match port.commit_personal_note_deletion(problem, &prepared).await {
        Ok(state) => Ok(state),
        Err(error) => match port.restore_deleted_personal_note(&prepared).await {
            Ok(()) => Err(error),
            Err(_) => Err(PersonalNoteDeletionError::CompensationFailed),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementReadState {
    Pending,
    Ready { sanitized_html: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalStatementAsset {
    pub local_ref: String,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestReadError {
    Unavailable,
    NotFound,
}

#[allow(async_fn_in_trait)]
pub trait ContestReadPort {
    async fn list_contests(&self) -> Result<Vec<ContestShelfItem>, ContestReadError>;
    async fn contest_detail(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
    ) -> Result<ContestDetail, ContestReadError>;
    async fn list_lightweight_problems(
        &self,
    ) -> Result<Vec<LightweightProblemItem>, ContestReadError>;
    async fn lightweight_problem_detail(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<LightweightProblemDetail, ContestReadError>;
    async fn statement_assets(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<Vec<LocalStatementAsset>, ContestReadError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContestImportContractError {
    TitleRequired,
    SourceUrlRequired,
    EmptyManifest,
    NonContiguousOrdinal,
    SlotContestMismatch,
    DuplicateProblemIdentity,
}

impl ContestImportDraft {
    pub fn validated(
        contest: acm_os_domain::CodeforcesContestIdentity,
        title: String,
        source_url: String,
        starts_at_utc: Option<String>,
        slots: Vec<ContestProblemSlotDraft>,
    ) -> Result<Self, ContestImportContractError> {
        if title.trim().is_empty() {
            return Err(ContestImportContractError::TitleRequired);
        }
        if source_url.trim().is_empty() {
            return Err(ContestImportContractError::SourceUrlRequired);
        }
        if slots.is_empty() {
            return Err(ContestImportContractError::EmptyManifest);
        }

        let mut seen = std::collections::HashSet::new();
        for (position, slot) in slots.iter().enumerate() {
            if slot.ordinal != position as u32 + 1 {
                return Err(ContestImportContractError::NonContiguousOrdinal);
            }
            if slot.problem.contest() != &contest {
                return Err(ContestImportContractError::SlotContestMismatch);
            }
            if !seen.insert(slot.problem.clone()) {
                return Err(ContestImportContractError::DuplicateProblemIdentity);
            }
        }

        Ok(Self {
            contest,
            title,
            source_url,
            starts_at_utc,
            slots,
        })
    }
}

pub struct FoundationStatus {
    pub status: &'static str,
    pub core: &'static str,
}

pub fn foundation_status() -> FoundationStatus {
    FoundationStatus {
        status: "ready",
        core: acm_os_domain::BOUNDARY_NAME,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupGateStatus {
    Ready { schema_version: i64 },
    RecoveryRequired { reason: StartupRecoveryReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupDestination {
    Recovery { reason: StartupRecoveryReason },
    Setup,
    Normal,
}

pub fn select_startup_destination(
    startup: &StartupGateStatus,
    workspace: Option<&WorkspaceConfigurationStatus>,
) -> StartupDestination {
    match startup {
        StartupGateStatus::RecoveryRequired { reason } => StartupDestination::Recovery {
            reason: reason.clone(),
        },
        StartupGateStatus::Ready { .. } => match workspace {
            Some(WorkspaceConfigurationStatus::Unconfigured) => StartupDestination::Setup,
            Some(WorkspaceConfigurationStatus::Configured(_)) => StartupDestination::Normal,
            None => StartupDestination::Recovery {
                reason: StartupRecoveryReason::DatabaseUnavailable,
            },
        },
    }
}

pub struct StartupStatusQuery {
    status: StartupGateStatus,
}

impl StartupStatusQuery {
    pub fn new(status: StartupGateStatus) -> Self {
        Self { status }
    }

    pub fn execute(&self) -> &StartupGateStatus {
        &self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupRecoveryReason {
    AppDataUnavailable,
    DatabaseUnavailable,
    MigrationLedgerInvalid,
    UnsupportedSchema { found: i64, supported: i64 },
    MigrationFailed,
    IntegrityCheckFailed,
    PreMigrationBackupFailed,
    UnresolvedCriticalOperation,
    RestoreIntentInvalid,
    RestoreFailed,
    RestoreIntentCleanupFailed,
}

impl StartupRecoveryReason {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::AppDataUnavailable => "app_data_unavailable",
            Self::DatabaseUnavailable => "database_unavailable",
            Self::MigrationLedgerInvalid => "migration_ledger_invalid",
            Self::UnsupportedSchema { .. } => "unsupported_schema",
            Self::MigrationFailed => "migration_failed",
            Self::IntegrityCheckFailed => "integrity_check_failed",
            Self::PreMigrationBackupFailed => "pre_migration_backup_failed",
            Self::UnresolvedCriticalOperation => "unresolved_critical_operation",
            Self::RestoreIntentInvalid => "restore_intent_invalid",
            Self::RestoreFailed => "restore_failed",
            Self::RestoreIntentCleanupFailed => "restore_intent_cleanup_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePathField {
    ActiveVault,
    ProblemRoot,
    KnowledgeRoot,
}

impl WorkspacePathField {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ActiveVault => "active_vault",
            Self::ProblemRoot => "problem_root",
            Self::KnowledgeRoot => "knowledge_root",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceConfigurationDraft {
    pub active_vault_path: String,
    pub problem_root_path: String,
    pub knowledge_root_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceConfiguration {
    active_vault_path: String,
    problem_root_path: String,
    knowledge_root_path: String,
}

impl WorkspaceConfiguration {
    pub fn from_resolved(
        active_vault_path: String,
        problem_root_path: String,
        knowledge_root_path: String,
    ) -> Result<Self, WorkspaceConfigurationError> {
        Self::validated(active_vault_path, problem_root_path, knowledge_root_path)
    }

    pub fn active_vault_path(&self) -> &str {
        &self.active_vault_path
    }

    pub fn problem_root_path(&self) -> &str {
        &self.problem_root_path
    }

    pub fn knowledge_root_path(&self) -> &str {
        &self.knowledge_root_path
    }

    fn validated(
        active_vault_path: String,
        problem_root_path: String,
        knowledge_root_path: String,
    ) -> Result<Self, WorkspaceConfigurationError> {
        let vault = Path::new(&active_vault_path);
        let problem = Path::new(&problem_root_path);
        let knowledge = Path::new(&knowledge_root_path);

        validate_resolved_path(vault, WorkspacePathField::ActiveVault)?;
        validate_resolved_path(problem, WorkspacePathField::ProblemRoot)?;
        validate_resolved_path(knowledge, WorkspacePathField::KnowledgeRoot)?;

        if !is_strict_descendant(problem, vault) {
            return Err(WorkspaceConfigurationError::RootOutsideVault {
                field: WorkspacePathField::ProblemRoot,
            });
        }
        if !is_strict_descendant(knowledge, vault) {
            return Err(WorkspaceConfigurationError::RootOutsideVault {
                field: WorkspacePathField::KnowledgeRoot,
            });
        }
        if problem.starts_with(knowledge) || knowledge.starts_with(problem) {
            return Err(WorkspaceConfigurationError::RootsOverlap);
        }

        Ok(Self {
            active_vault_path,
            problem_root_path,
            knowledge_root_path,
        })
    }
}

fn validate_resolved_path(
    path: &Path,
    field: WorkspacePathField,
) -> Result<(), WorkspaceConfigurationError> {
    let is_normalized = path.is_absolute()
        && !path.as_os_str().to_string_lossy().contains('\0')
        && path.components().all(|component| {
            matches!(
                component,
                Component::Prefix(_) | Component::RootDir | Component::Normal(_)
            )
        });
    if is_normalized {
        Ok(())
    } else {
        Err(WorkspaceConfigurationError::PathUnavailable { field })
    }
}

fn is_strict_descendant(path: &Path, parent: &Path) -> bool {
    path.strip_prefix(parent)
        .is_ok_and(|relative| relative.components().next().is_some())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceConfigurationStatus {
    Unconfigured,
    Configured(WorkspaceConfiguration),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceConfigurationError {
    PathRequired { field: WorkspacePathField },
    PathUnavailable { field: WorkspacePathField },
    PathNotDirectory { field: WorkspacePathField },
    RootOutsideVault { field: WorkspacePathField },
    RootsOverlap,
    AlreadyConfigured,
    PersistenceUnavailable,
}

impl WorkspaceConfigurationError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::PathRequired { .. } => "path_required",
            Self::PathUnavailable { .. } => "path_unavailable",
            Self::PathNotDirectory { .. } => "path_not_directory",
            Self::RootOutsideVault { .. } => "root_outside_vault",
            Self::RootsOverlap => "roots_overlap",
            Self::AlreadyConfigured => "already_configured",
            Self::PersistenceUnavailable => "persistence_unavailable",
        }
    }

    pub const fn field(&self) -> Option<WorkspacePathField> {
        match self {
            Self::PathRequired { field }
            | Self::PathUnavailable { field }
            | Self::PathNotDirectory { field }
            | Self::RootOutsideVault { field } => Some(*field),
            Self::RootsOverlap | Self::AlreadyConfigured | Self::PersistenceUnavailable => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePathResolutionError {
    Unavailable,
    NotDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePersistenceError {
    AlreadyConfigured,
    Unavailable,
}

#[allow(async_fn_in_trait)]
pub trait WorkspaceConfigurationPort {
    async fn resolve_directory(&self, path: &str) -> Result<String, WorkspacePathResolutionError>;

    async fn load_workspace_configuration(
        &self,
    ) -> Result<Option<WorkspaceConfiguration>, WorkspacePersistenceError>;

    async fn insert_workspace_configuration(
        &self,
        configuration: &WorkspaceConfiguration,
    ) -> Result<(), WorkspacePersistenceError>;
}

pub async fn query_workspace_configuration<P: WorkspaceConfigurationPort>(
    port: &P,
) -> Result<WorkspaceConfigurationStatus, WorkspaceConfigurationError> {
    port.load_workspace_configuration()
        .await
        .map(|configuration| match configuration {
            Some(configuration) => WorkspaceConfigurationStatus::Configured(configuration),
            None => WorkspaceConfigurationStatus::Unconfigured,
        })
        .map_err(|_| WorkspaceConfigurationError::PersistenceUnavailable)
}

pub async fn configure_workspace<P: WorkspaceConfigurationPort>(
    port: &P,
    draft: WorkspaceConfigurationDraft,
) -> Result<WorkspaceConfiguration, WorkspaceConfigurationError> {
    if port
        .load_workspace_configuration()
        .await
        .map_err(map_persistence_error)?
        .is_some()
    {
        return Err(WorkspaceConfigurationError::AlreadyConfigured);
    }

    let active_vault_path = resolve_required_directory(
        port,
        WorkspacePathField::ActiveVault,
        &draft.active_vault_path,
    )
    .await?;
    let problem_root_path = resolve_required_directory(
        port,
        WorkspacePathField::ProblemRoot,
        &draft.problem_root_path,
    )
    .await?;
    let knowledge_root_path = resolve_required_directory(
        port,
        WorkspacePathField::KnowledgeRoot,
        &draft.knowledge_root_path,
    )
    .await?;

    let configuration = WorkspaceConfiguration::from_resolved(
        active_vault_path,
        problem_root_path,
        knowledge_root_path,
    )?;
    port.insert_workspace_configuration(&configuration)
        .await
        .map_err(map_persistence_error)?;
    Ok(configuration)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualBackupPreview {
    pub schema_version: i64,
    pub backup_directory: String,
    pub filename_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualBackupResult {
    pub path: String,
    pub schema_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInventoryEntry {
    pub path: String,
    pub category: String,
    pub size_bytes: u64,
    pub integrity_verified: bool,
    pub retention: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInventory {
    pub entries: Vec<BackupInventoryEntry>,
    pub daily_keep: u32,
    pub weekly_keep: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRestoreCandidatePreview {
    pub source_path: String,
    pub schema_version: i64,
    pub supported_schema_version: i64,
    pub migration_required: bool,
    pub restores_system_facts: bool,
    pub overwrites_markdown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreRestoreSnapshotResult {
    pub path: String,
    pub schema_version: i64,
    pub candidate: SystemRestoreCandidatePreview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreIntentPreparationResult {
    pub staging_path: String,
    pub pre_restore_snapshot_path: String,
    pub candidate: SystemRestoreCandidatePreview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRestoreRebuildPreview {
    pub problem_binding_count: u64,
    pub knowledge_binding_count: u64,
    pub derived_relation_count: u64,
    pub revalidates_bindings: bool,
    pub rebuilds_derived_knowledge: bool,
    pub overwrites_markdown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRestoreBindingAnomaly {
    pub problem_id: i64,
    pub vault_relative_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRestoreProblemBindingValidation {
    pub total_count: u64,
    pub ready_count: u64,
    pub anomalies: Vec<PostRestoreBindingAnomaly>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRestoreKnowledgeBindingAnomaly {
    pub knowledge_node_id: String,
    pub vault_relative_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRestoreKnowledgeBindingValidation {
    pub total_count: u64,
    pub ready_count: u64,
    pub confirmed_deleted_count: u64,
    pub anomalies: Vec<PostRestoreKnowledgeBindingAnomaly>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRestoreRebuildPreconditionCheck {
    pub eligible: bool,
    pub blockers: Vec<String>,
    pub problem_binding_anomaly_count: u64,
    pub knowledge_binding_anomaly_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRestoreRebuildApplyResult {
    pub knowledge_node_count: u64,
    pub relation_count: u64,
    pub location_anomaly_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticExportPreview {
    pub output_directory: String,
    pub sections: Vec<String>,
    pub privacy_exclusions: Vec<String>,
    pub creates_files: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticExportResult {
    pub path: String,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRetentionPreview {
    pub protected_paths: Vec<String>,
    pub prune_candidate_paths: Vec<String>,
    pub daily_keep: u32,
    pub weekly_keep: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualBackupError {
    PersistenceUnavailable,
    BackupFailed,
    IntegrityViolation,
    RestoreCandidateUnavailable,
    RestoreCandidateOutsideBackupArea,
    RestoreCandidateNotPublished,
    RestoreCandidateSchemaUnsupported,
    PreRestoreBackupFailed,
    RestoreIntentPending,
    RestoreIntentWriteFailed,
    RestoreStagingFailed,
}

impl ManualBackupError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::BackupFailed => "manual_backup_failed",
            Self::IntegrityViolation => "integrity_violation",
            Self::RestoreCandidateUnavailable => "restore_candidate_unavailable",
            Self::RestoreCandidateOutsideBackupArea => "restore_candidate_outside_backup_area",
            Self::RestoreCandidateNotPublished => "restore_candidate_not_published",
            Self::RestoreCandidateSchemaUnsupported => "restore_candidate_schema_unsupported",
            Self::PreRestoreBackupFailed => "pre_restore_backup_failed",
            Self::RestoreIntentPending => "restore_intent_pending",
            Self::RestoreIntentWriteFailed => "restore_intent_write_failed",
            Self::RestoreStagingFailed => "restore_staging_failed",
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ManualBackupPort {
    async fn preview_manual_backup(&self) -> Result<ManualBackupPreview, ManualBackupError>;
    async fn create_manual_backup(&self) -> Result<ManualBackupResult, ManualBackupError>;
    async fn backup_inventory(&self) -> Result<BackupInventory, ManualBackupError>;
    async fn preview_system_restore_candidate(
        &self,
        source_path: String,
    ) -> Result<SystemRestoreCandidatePreview, ManualBackupError>;
    async fn create_pre_restore_snapshot(
        &self,
        source_path: String,
    ) -> Result<PreRestoreSnapshotResult, ManualBackupError>;
    async fn prepare_restore_intent(
        &self,
        source_path: String,
    ) -> Result<RestoreIntentPreparationResult, ManualBackupError>;
    async fn preview_post_restore_rebuild(
        &self,
    ) -> Result<PostRestoreRebuildPreview, ManualBackupError>;
    async fn validate_post_restore_problem_bindings(
        &self,
    ) -> Result<PostRestoreProblemBindingValidation, ManualBackupError>;
    async fn validate_post_restore_knowledge_bindings(
        &self,
    ) -> Result<PostRestoreKnowledgeBindingValidation, ManualBackupError>;
    async fn check_post_restore_rebuild_preconditions(
        &self,
    ) -> Result<PostRestoreRebuildPreconditionCheck, ManualBackupError>;
    async fn apply_post_restore_rebuild(
        &self,
    ) -> Result<PostRestoreRebuildApplyResult, ManualBackupError>;
    async fn preview_diagnostic_export(&self)
        -> Result<DiagnosticExportPreview, ManualBackupError>;
    async fn create_diagnostic_export(&self) -> Result<DiagnosticExportResult, ManualBackupError>;
    async fn create_weekly_backup(&self) -> Result<ManualBackupResult, ManualBackupError>;
    async fn preview_backup_retention(&self) -> Result<BackupRetentionPreview, ManualBackupError>;
    async fn apply_backup_retention(&self, paths: Vec<String>) -> Result<u64, ManualBackupError>;
}

pub async fn preview_manual_backup<P: ManualBackupPort>(
    port: &P,
) -> Result<ManualBackupPreview, ManualBackupError> {
    port.preview_manual_backup().await
}

pub async fn create_manual_backup<P: ManualBackupPort>(
    port: &P,
) -> Result<ManualBackupResult, ManualBackupError> {
    port.create_manual_backup().await
}

pub async fn backup_inventory<P: ManualBackupPort>(
    port: &P,
) -> Result<BackupInventory, ManualBackupError> {
    port.backup_inventory().await
}

pub async fn preview_system_restore_candidate<P: ManualBackupPort>(
    port: &P,
    source_path: String,
) -> Result<SystemRestoreCandidatePreview, ManualBackupError> {
    port.preview_system_restore_candidate(source_path).await
}

pub async fn create_pre_restore_snapshot<P: ManualBackupPort>(
    port: &P,
    source_path: String,
) -> Result<PreRestoreSnapshotResult, ManualBackupError> {
    port.create_pre_restore_snapshot(source_path).await
}

pub async fn prepare_restore_intent<P: ManualBackupPort>(
    port: &P,
    source_path: String,
) -> Result<RestoreIntentPreparationResult, ManualBackupError> {
    port.prepare_restore_intent(source_path).await
}

pub async fn preview_post_restore_rebuild<P: ManualBackupPort>(
    port: &P,
) -> Result<PostRestoreRebuildPreview, ManualBackupError> {
    port.preview_post_restore_rebuild().await
}

pub async fn validate_post_restore_problem_bindings<P: ManualBackupPort>(
    port: &P,
) -> Result<PostRestoreProblemBindingValidation, ManualBackupError> {
    port.validate_post_restore_problem_bindings().await
}

pub async fn validate_post_restore_knowledge_bindings<P: ManualBackupPort>(
    port: &P,
) -> Result<PostRestoreKnowledgeBindingValidation, ManualBackupError> {
    port.validate_post_restore_knowledge_bindings().await
}

pub async fn check_post_restore_rebuild_preconditions<P: ManualBackupPort>(
    port: &P,
) -> Result<PostRestoreRebuildPreconditionCheck, ManualBackupError> {
    port.check_post_restore_rebuild_preconditions().await
}

pub async fn apply_post_restore_rebuild<P: ManualBackupPort>(
    port: &P,
) -> Result<PostRestoreRebuildApplyResult, ManualBackupError> {
    port.apply_post_restore_rebuild().await
}

pub async fn preview_diagnostic_export<P: ManualBackupPort>(
    port: &P,
) -> Result<DiagnosticExportPreview, ManualBackupError> {
    port.preview_diagnostic_export().await
}

pub async fn create_diagnostic_export<P: ManualBackupPort>(
    port: &P,
) -> Result<DiagnosticExportResult, ManualBackupError> {
    port.create_diagnostic_export().await
}

pub async fn create_weekly_backup<P: ManualBackupPort>(
    port: &P,
) -> Result<ManualBackupResult, ManualBackupError> {
    port.create_weekly_backup().await
}

pub async fn preview_backup_retention<P: ManualBackupPort>(
    port: &P,
) -> Result<BackupRetentionPreview, ManualBackupError> {
    port.preview_backup_retention().await
}

pub async fn apply_backup_retention<P: ManualBackupPort>(
    port: &P,
    paths: Vec<String>,
) -> Result<u64, ManualBackupError> {
    port.apply_backup_retention(paths).await
}

async fn resolve_required_directory<P: WorkspaceConfigurationPort>(
    port: &P,
    field: WorkspacePathField,
    raw_path: &str,
) -> Result<String, WorkspaceConfigurationError> {
    let path = raw_path.trim();
    if path.is_empty() {
        return Err(WorkspaceConfigurationError::PathRequired { field });
    }

    port.resolve_directory(path)
        .await
        .map_err(|reason| match reason {
            WorkspacePathResolutionError::Unavailable => {
                WorkspaceConfigurationError::PathUnavailable { field }
            }
            WorkspacePathResolutionError::NotDirectory => {
                WorkspaceConfigurationError::PathNotDirectory { field }
            }
        })
}

fn map_persistence_error(error: WorkspacePersistenceError) -> WorkspaceConfigurationError {
    match error {
        WorkspacePersistenceError::AlreadyConfigured => {
            WorkspaceConfigurationError::AlreadyConfigured
        }
        WorkspacePersistenceError::Unavailable => {
            WorkspaceConfigurationError::PersistenceUnavailable
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeLocationState {
    Ready,
    LocationAnomaly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeNodeProjection {
    pub knowledge_node_id: String,
    pub display_name: String,
    pub vault_relative_path: String,
    pub content_digest: String,
    pub windows_file_key: Option<String>,
    pub location_state: KnowledgeLocationState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeIndexProjection {
    pub nodes: Vec<KnowledgeNodeProjection>,
    pub location_anomalies: Vec<KnowledgeNodeProjection>,
    pub identity_conflicts: Vec<KnowledgeIdentityConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeIdentityConflict {
    pub historical_knowledge_node_id: String,
    pub display_name: String,
    pub candidate_vault_relative_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeIndexError {
    WorkspaceUnavailable,
    KnowledgeRootUnavailable,
    KnowledgeNodeNotFound,
    PersistenceUnavailable,
    IntegrityViolation,
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeIndexPort {
    async fn rebuild_knowledge_index(
        &self,
    ) -> Result<KnowledgeIndexProjection, KnowledgeIndexError>;

    async fn search_knowledge_index(
        &self,
        query: &str,
    ) -> Result<Vec<KnowledgeNodeProjection>, KnowledgeIndexError>;
}

pub async fn rebuild_knowledge_index<P: KnowledgeIndexPort>(
    port: &P,
) -> Result<KnowledgeIndexProjection, KnowledgeIndexError> {
    port.rebuild_knowledge_index().await
}

pub async fn search_knowledge_index<P: KnowledgeIndexPort>(
    port: &P,
    query: &str,
) -> Result<Vec<KnowledgeNodeProjection>, KnowledgeIndexError> {
    port.search_knowledge_index(query.trim()).await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeRelocationCandidate {
    pub vault_relative_path: String,
    pub occupied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeBindingRepairError {
    WorkspaceUnavailable,
    VaultUnavailable,
    KnowledgeNodeNotFound,
    LocationAnomalyRequired,
    CandidateUnavailable,
    CandidateOccupied,
    PersistenceUnavailable,
    IntegrityViolation,
    IdentityConflictRequired,
}

impl KnowledgeBindingRepairError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::WorkspaceUnavailable => "workspace_unavailable",
            Self::VaultUnavailable => "vault_unavailable",
            Self::KnowledgeNodeNotFound => "knowledge_node_not_found",
            Self::LocationAnomalyRequired => "knowledge_location_anomaly_required",
            Self::CandidateUnavailable => "knowledge_relocation_candidate_unavailable",
            Self::CandidateOccupied => "knowledge_relocation_candidate_occupied",
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::IntegrityViolation => "integrity_violation",
            Self::IdentityConflictRequired => "knowledge_identity_conflict_required",
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeBindingRepairPort {
    async fn knowledge_relocation_candidates(
        &self,
        knowledge_node_id: &str,
    ) -> Result<Vec<KnowledgeRelocationCandidate>, KnowledgeBindingRepairError>;

    async fn rebind_knowledge_node(
        &self,
        knowledge_node_id: &str,
        vault_relative_path: &str,
    ) -> Result<KnowledgeNodeProjection, KnowledgeBindingRepairError>;

    async fn confirm_knowledge_markdown_deleted(
        &self,
        knowledge_node_id: &str,
    ) -> Result<(), KnowledgeBindingRepairError>;

    async fn resolve_knowledge_identity_conflict(
        &self,
        historical_knowledge_node_id: &str,
        candidate_vault_relative_path: &str,
        restore_old_identity: bool,
    ) -> Result<KnowledgeNodeProjection, KnowledgeBindingRepairError>;
}

pub async fn knowledge_relocation_candidates<P: KnowledgeBindingRepairPort>(
    port: &P,
    knowledge_node_id: &str,
) -> Result<Vec<KnowledgeRelocationCandidate>, KnowledgeBindingRepairError> {
    let knowledge_node_id = knowledge_node_id.trim();
    if knowledge_node_id.is_empty() {
        return Err(KnowledgeBindingRepairError::IntegrityViolation);
    }
    port.knowledge_relocation_candidates(knowledge_node_id)
        .await
}

pub async fn rebind_knowledge_node<P: KnowledgeBindingRepairPort>(
    port: &P,
    knowledge_node_id: &str,
    vault_relative_path: impl AsRef<str>,
) -> Result<KnowledgeNodeProjection, KnowledgeBindingRepairError> {
    let knowledge_node_id = knowledge_node_id.trim();
    let vault_relative_path = vault_relative_path.as_ref().trim();
    if knowledge_node_id.is_empty() || vault_relative_path.is_empty() {
        return Err(KnowledgeBindingRepairError::IntegrityViolation);
    }
    port.rebind_knowledge_node(knowledge_node_id, vault_relative_path)
        .await
}

pub async fn confirm_knowledge_markdown_deleted<P: KnowledgeBindingRepairPort>(
    port: &P,
    knowledge_node_id: &str,
) -> Result<(), KnowledgeBindingRepairError> {
    let knowledge_node_id = knowledge_node_id.trim();
    if knowledge_node_id.is_empty() {
        return Err(KnowledgeBindingRepairError::IntegrityViolation);
    }
    port.confirm_knowledge_markdown_deleted(knowledge_node_id)
        .await
}

pub async fn resolve_knowledge_identity_conflict<P: KnowledgeBindingRepairPort>(
    port: &P,
    historical_knowledge_node_id: &str,
    candidate_vault_relative_path: &str,
    restore_old_identity: bool,
) -> Result<KnowledgeNodeProjection, KnowledgeBindingRepairError> {
    if historical_knowledge_node_id.trim().is_empty()
        || candidate_vault_relative_path.trim().is_empty()
    {
        return Err(KnowledgeBindingRepairError::IntegrityViolation);
    }
    port.resolve_knowledge_identity_conflict(
        historical_knowledge_node_id.trim(),
        candidate_vault_relative_path.trim(),
        restore_old_identity,
    )
    .await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeLinkResolution {
    Resolved,
    Unresolved,
    Ambiguous,
    NonKnowledgeTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeLinkProjection {
    pub source_kind: String,
    pub source_id: String,
    pub target_ref: String,
    pub target_knowledge_node_id: Option<String>,
    pub resolution: KnowledgeLinkResolution,
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeRelationPort {
    async fn rebuild_knowledge_relations(
        &self,
    ) -> Result<Vec<KnowledgeLinkProjection>, KnowledgeIndexError>;
}

pub async fn rebuild_knowledge_relations<P: KnowledgeRelationPort>(
    port: &P,
) -> Result<Vec<KnowledgeLinkProjection>, KnowledgeIndexError> {
    port.rebuild_knowledge_relations().await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeUnderstandingProjection {
    pub knowledge_node_id: String,
    pub current: acm_os_domain::KnowledgeUnderstandingLevel,
    pub historical_highest: acm_os_domain::KnowledgeUnderstandingLevel,
    pub first_reached_highest_on: acm_os_domain::LocalDate,
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeUnderstandingPort {
    async fn confirm_knowledge_understanding(
        &self,
        knowledge_node_id: &str,
        selected: acm_os_domain::KnowledgeUnderstandingLevel,
        confirmed_on: acm_os_domain::LocalDate,
    ) -> Result<KnowledgeUnderstandingProjection, KnowledgeIndexError>;
}

pub async fn confirm_knowledge_understanding<P: KnowledgeUnderstandingPort>(
    port: &P,
    knowledge_node_id: &str,
    selected: acm_os_domain::KnowledgeUnderstandingLevel,
    confirmed_on: acm_os_domain::LocalDate,
) -> Result<KnowledgeUnderstandingProjection, KnowledgeIndexError> {
    if knowledge_node_id.trim().is_empty() {
        return Err(KnowledgeIndexError::IntegrityViolation);
    }
    port.confirm_knowledge_understanding(knowledge_node_id, selected, confirmed_on)
        .await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedKnowledgeProblemProjection {
    pub problem_id: String,
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeDetailProjection {
    pub node: KnowledgeNodeProjection,
    pub understanding: Option<KnowledgeUnderstandingProjection>,
    pub incoming: Vec<KnowledgeNodeProjection>,
    pub outgoing: Vec<KnowledgeNodeProjection>,
    pub related_problems: Vec<RelatedKnowledgeProblemProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeReevaluationSuggestion {
    pub knowledge_node_id: String,
    pub should_suggest: bool,
    pub qualifying_problem_count: u32,
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeReevaluationPort {
    async fn load_knowledge_reevaluation_suggestion(
        &self,
        knowledge_node_id: &str,
    ) -> Result<KnowledgeReevaluationSuggestion, KnowledgeIndexError>;
}

pub async fn load_knowledge_reevaluation_suggestion<P: KnowledgeReevaluationPort>(
    port: &P,
    knowledge_node_id: &str,
) -> Result<KnowledgeReevaluationSuggestion, KnowledgeIndexError> {
    let id = knowledge_node_id.trim();
    if id.is_empty() {
        return Err(KnowledgeIndexError::IntegrityViolation);
    }
    port.load_knowledge_reevaluation_suggestion(id).await
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeDetailPort {
    async fn load_knowledge_detail(
        &self,
        knowledge_node_id: &str,
    ) -> Result<KnowledgeDetailProjection, KnowledgeIndexError>;
}

pub async fn load_knowledge_detail<P: KnowledgeDetailPort>(
    port: &P,
    knowledge_node_id: &str,
) -> Result<KnowledgeDetailProjection, KnowledgeIndexError> {
    let knowledge_node_id = knowledge_node_id.trim();
    if knowledge_node_id.is_empty() {
        return Err(KnowledgeIndexError::IntegrityViolation);
    }
    port.load_knowledge_detail(knowledge_node_id).await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeCandidateDisposition {
    Pending,
    AcceptedIntent,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeCandidateProjection {
    pub problem: acm_os_domain::CodeforcesProblemIdentity,
    pub fingerprint: String,
    pub target_ref: String,
    pub disposition: KnowledgeCandidateDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedKnowledgeCandidateProjection {
    pub knowledge_node_id: String,
    pub target_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeCandidateError {
    ProblemNotFound,
    NotPersonal,
    CandidateNotFound,
    InvalidFingerprint,
    InvalidTarget,
    PersistenceUnavailable,
    IntegrityViolation,
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeCandidatePort {
    async fn list_knowledge_candidates(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<Vec<KnowledgeCandidateProjection>, KnowledgeCandidateError>;

    async fn register_knowledge_candidate(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        fingerprint: &str,
        target_ref: &str,
    ) -> Result<KnowledgeCandidateProjection, KnowledgeCandidateError>;

    async fn set_knowledge_candidate_disposition(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        fingerprint: &str,
        disposition: KnowledgeCandidateDisposition,
    ) -> Result<KnowledgeCandidateProjection, KnowledgeCandidateError>;

    async fn accept_existing_knowledge_candidate(
        &self,
        problem: &acm_os_domain::ProblemIdentity,
        fingerprint: &str,
        knowledge_node_id: &str,
    ) -> Result<AcceptedKnowledgeCandidateProjection, KnowledgeCandidateError>;
}

pub async fn accept_existing_knowledge_candidate<P: KnowledgeCandidatePort>(
    port: &P,
    problem: &acm_os_domain::ProblemIdentity,
    fingerprint: &str,
    knowledge_node_id: &str,
) -> Result<AcceptedKnowledgeCandidateProjection, KnowledgeCandidateError> {
    let fingerprint = normalize_candidate_fingerprint(fingerprint)?;
    let knowledge_node_id = knowledge_node_id.trim();
    if knowledge_node_id.is_empty() {
        return Err(KnowledgeCandidateError::IntegrityViolation);
    }
    port.accept_existing_knowledge_candidate(problem, &fingerprint, knowledge_node_id)
        .await
}

pub async fn list_knowledge_candidates<P: KnowledgeCandidatePort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<Vec<KnowledgeCandidateProjection>, KnowledgeCandidateError> {
    port.list_knowledge_candidates(problem).await
}

pub async fn register_knowledge_candidate<P: KnowledgeCandidatePort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    fingerprint: &str,
    target_ref: &str,
) -> Result<KnowledgeCandidateProjection, KnowledgeCandidateError> {
    let fingerprint = normalize_candidate_fingerprint(fingerprint)?;
    let target_ref = normalize_candidate_target(target_ref)?;
    port.register_knowledge_candidate(problem, &fingerprint, &target_ref)
        .await
}

pub async fn set_knowledge_candidate_disposition<P: KnowledgeCandidatePort>(
    port: &P,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    fingerprint: &str,
    disposition: KnowledgeCandidateDisposition,
) -> Result<KnowledgeCandidateProjection, KnowledgeCandidateError> {
    let fingerprint = normalize_candidate_fingerprint(fingerprint)?;
    port.set_knowledge_candidate_disposition(problem, &fingerprint, disposition)
        .await
}

fn normalize_candidate_fingerprint(fingerprint: &str) -> Result<String, KnowledgeCandidateError> {
    let fingerprint = fingerprint.trim().to_ascii_lowercase();
    if fingerprint.len() != 64 || !fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(KnowledgeCandidateError::InvalidFingerprint);
    }
    Ok(fingerprint)
}

fn normalize_candidate_target(target_ref: &str) -> Result<String, KnowledgeCandidateError> {
    let target_ref = target_ref.trim();
    if target_ref.is_empty()
        || target_ref.len() > 512
        || target_ref.contains(['\r', '\n'])
        || target_ref.contains("[[")
        || target_ref.contains("]]")
    {
        return Err(KnowledgeCandidateError::InvalidTarget);
    }
    Ok(target_ref.to_owned())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    use super::*;

    fn run_ready<F: Future>(future: F) -> F::Output {
        let mut future = Box::pin(future);
        let mut context = Context::from_waker(Waker::noop());
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("test future unexpectedly yielded"),
        }
    }

    struct FailingPersonalNoteCommit {
        discarded: Cell<bool>,
    }

    struct FailingPersonalNoteDeletionCommit {
        restored: Cell<bool>,
    }

    impl PersonalNoteDeletionPort for FailingPersonalNoteDeletionCommit {
        async fn prepare_personal_note_deletion(
            &self,
            _problem: &acm_os_domain::CodeforcesProblemIdentity,
        ) -> Result<PreparedPersonalNoteDeletion, PersonalNoteDeletionError> {
            Ok(PreparedPersonalNoteDeletion {
                vault_relative_path: "Problems/CF-1979-A.md".to_owned(),
                content_digest: "0".repeat(64),
                recovery_copy_path: "private/recovery.md".to_owned(),
            })
        }

        async fn commit_personal_note_deletion(
            &self,
            _problem: &acm_os_domain::CodeforcesProblemIdentity,
            _prepared: &PreparedPersonalNoteDeletion,
        ) -> Result<ProblemLifecycleState, PersonalNoteDeletionError> {
            Err(PersonalNoteDeletionError::PersistenceUnavailable)
        }

        async fn restore_deleted_personal_note(
            &self,
            _prepared: &PreparedPersonalNoteDeletion,
        ) -> Result<(), PersonalNoteDeletionError> {
            self.restored.set(true);
            Ok(())
        }
    }

    impl PersonalNotePort for FailingPersonalNoteCommit {
        async fn personal_note_creation_context(
            &self,
            problem: &acm_os_domain::CodeforcesProblemIdentity,
        ) -> Result<PersonalNoteCreationContext, PersonalNoteError> {
            Ok(PersonalNoteCreationContext {
                problem: problem.clone(),
                existing_binding: None,
            })
        }

        async fn create_personal_note_file(
            &self,
            _context: &PersonalNoteCreationContext,
            markdown: &[u8],
        ) -> Result<CreatedPersonalNoteFile, PersonalNoteError> {
            assert_eq!(markdown, INITIAL_PROBLEM_MARKDOWN.as_bytes());
            Ok(CreatedPersonalNoteFile {
                vault_relative_path: "Problems/CF-1979-A.md".to_owned(),
                content_digest: "0".repeat(64),
                windows_file_key: None,
            })
        }

        async fn commit_personal_note_binding(
            &self,
            _problem: &acm_os_domain::CodeforcesProblemIdentity,
            _file: &CreatedPersonalNoteFile,
        ) -> Result<PersonalNoteBinding, PersonalNoteError> {
            Err(PersonalNoteError::PersistenceUnavailable)
        }

        async fn discard_created_personal_note(
            &self,
            _file: &CreatedPersonalNoteFile,
        ) -> Result<(), PersonalNoteError> {
            self.discarded.set(true);
            Ok(())
        }
    }

    fn contest_identity() -> acm_os_domain::CodeforcesContestIdentity {
        acm_os_domain::CodeforcesContestIdentity::new(1979).expect("valid contest")
    }

    #[test]
    fn contest_facts_require_date_and_exact_unique_problem_membership() {
        let contest = contest_identity();
        let facts = vec![ContestProblemFactInput {
            problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "A")
                .expect("problem"),
            final_contest_result: ContestFinalResult::Unknown,
            upsolve_decision: ContestUpsolveDecision::Undecided,
        }];
        assert_eq!(
            validate_contest_facts_input(&contest, None, &facts),
            Err(ContestFactsError::ContestDateMissing)
        );
        assert_eq!(
            validate_contest_facts_input(&contest, Some("2026-08-13"), &facts),
            Ok(())
        );
        let duplicated = vec![facts[0].clone(), facts[0].clone()];
        assert_eq!(
            validate_contest_facts_input(&contest, Some("2026-08-13"), &duplicated),
            Err(ContestFactsError::ProblemSetMismatch)
        );
    }

    fn problem_slot(
        contest: acm_os_domain::CodeforcesContestIdentity,
        ordinal: u32,
        index: &str,
    ) -> ContestProblemSlotDraft {
        ContestProblemSlotDraft {
            ordinal,
            problem: acm_os_domain::CodeforcesProblemIdentity::new(contest, index)
                .expect("valid problem"),
            title: format!("Problem {index}"),
            rating: Some(800),
            source_url: format!("https://codeforces.com/contest/1979/problem/{index}"),
        }
    }

    #[test]
    fn import_manifest_requires_a_stable_complete_ordered_identity_list() {
        let contest = contest_identity();
        let valid = ContestImportDraft::validated(
            contest.clone(),
            "Codeforces Round".to_owned(),
            "https://codeforces.com/contest/1979".to_owned(),
            None,
            vec![
                problem_slot(contest.clone(), 1, "A"),
                problem_slot(contest.clone(), 2, "B"),
            ],
        )
        .expect("complete manifest");
        assert_eq!(valid.slots.len(), 2);

        assert_eq!(
            ContestImportDraft::validated(
                contest.clone(),
                "Contest".to_owned(),
                "https://codeforces.com/contest/1979".to_owned(),
                None,
                vec![],
            ),
            Err(ContestImportContractError::EmptyManifest)
        );
        assert_eq!(
            ContestImportDraft::validated(
                contest.clone(),
                "Contest".to_owned(),
                "https://codeforces.com/contest/1979".to_owned(),
                None,
                vec![problem_slot(contest.clone(), 2, "A")],
            ),
            Err(ContestImportContractError::NonContiguousOrdinal)
        );
        assert_eq!(
            ContestImportDraft::validated(
                contest.clone(),
                "Contest".to_owned(),
                "https://codeforces.com/contest/1979".to_owned(),
                None,
                vec![
                    problem_slot(contest.clone(), 1, "A"),
                    problem_slot(contest, 2, "A"),
                ],
            ),
            Err(ContestImportContractError::DuplicateProblemIdentity)
        );
    }

    #[test]
    fn import_execution_plan_only_retries_missing_snapshot_identities() {
        let contest = contest_identity();
        let manifest = ContestImportDraft::validated(
            contest.clone(),
            "Codeforces Round".to_owned(),
            "https://codeforces.com/contest/1979".to_owned(),
            None,
            vec![
                problem_slot(contest.clone(), 1, "A"),
                problem_slot(contest.clone(), 2, "B"),
            ],
        )
        .expect("manifest");
        let snapshot = |index: &str| StatementSnapshotDraft {
            problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), index)
                .expect("problem"),
            source_html: format!("<div class=\"problem-statement\">{index}</div>"),
            sanitized_html: format!("<div class=\"problem-statement\">{index}</div>"),
            assets: Vec::new(),
        };
        let plan =
            ContestImportExecutionPlan::validated(manifest, vec![snapshot("A"), snapshot("B")])
                .expect("execution plan");
        let missing_b =
            acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "B").expect("problem B");
        assert_eq!(
            plan.snapshots_for_missing(&[missing_b])[0].problem.index(),
            "B"
        );

        let foreign = StatementSnapshotDraft {
            problem: acm_os_domain::CodeforcesProblemIdentity::new(contest, "C")
                .expect("problem C"),
            source_html: "<div class=\"problem-statement\">C</div>".to_owned(),
            sanitized_html: "<div class=\"problem-statement\">C</div>".to_owned(),
            assets: Vec::new(),
        };
        assert_eq!(
            ContestImportExecutionPlan::validated(plan.manifest.clone(), vec![foreign]),
            Err(ContestImportExecutionError::SnapshotOutsideManifest)
        );
    }

    #[test]
    fn personal_note_creation_compensates_a_failed_binding_commit() {
        let port = FailingPersonalNoteCommit {
            discarded: Cell::new(false),
        };
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest_identity(), "A")
            .expect("problem");
        assert_eq!(
            run_ready(create_personal_note(&port, &problem)),
            Err(PersonalNoteError::PersistenceUnavailable)
        );
        assert!(port.discarded.get());
    }

    #[test]
    fn personal_note_deletion_restores_the_file_when_system_fact_commit_fails() {
        let port = FailingPersonalNoteDeletionCommit {
            restored: Cell::new(false),
        };
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest_identity(), "A")
            .expect("problem");
        assert_eq!(
            run_ready(delete_personal_note(&port, &problem)),
            Err(PersonalNoteDeletionError::PersistenceUnavailable)
        );
        assert!(port.restored.get());
    }

    fn test_path(windows: &str, unix: &str) -> String {
        if cfg!(windows) {
            windows.to_owned()
        } else {
            unix.to_owned()
        }
    }

    #[test]
    fn resolved_configuration_rejects_relative_paths() {
        let error = WorkspaceConfiguration::from_resolved(
            "Vault".to_owned(),
            test_path("C:\\Vault\\Problems", "/Vault/Problems"),
            test_path("C:\\Vault\\Knowledge", "/Vault/Knowledge"),
        )
        .expect_err("resolved Vault must be absolute");
        assert_eq!(
            error,
            WorkspaceConfigurationError::PathUnavailable {
                field: WorkspacePathField::ActiveVault,
            }
        );
    }

    #[test]
    fn resolved_configuration_rejects_parent_components() {
        let error = WorkspaceConfiguration::from_resolved(
            test_path("C:\\Vault", "/Vault"),
            test_path("C:\\Vault\\Problems", "/Vault/Problems"),
            test_path(
                "C:\\Vault\\Problems\\..\\..\\Outside",
                "/Vault/Problems/../../Outside",
            ),
        )
        .expect_err("resolved paths cannot contain parent traversal");
        assert_eq!(
            error,
            WorkspaceConfigurationError::PathUnavailable {
                field: WorkspacePathField::KnowledgeRoot,
            }
        );
    }

    #[test]
    fn startup_destination_blocks_recovery_before_workspace_routing() {
        let destination = select_startup_destination(
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            },
            Some(&WorkspaceConfigurationStatus::Unconfigured),
        );
        assert_eq!(
            destination,
            StartupDestination::Recovery {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[test]
    fn startup_destination_routes_workspace_states() {
        assert_eq!(
            select_startup_destination(
                &StartupGateStatus::Ready { schema_version: 2 },
                Some(&WorkspaceConfigurationStatus::Unconfigured),
            ),
            StartupDestination::Setup
        );
        let configured = WorkspaceConfigurationStatus::Configured(
            WorkspaceConfiguration::from_resolved(
                test_path("C:\\Vault", "/Vault"),
                test_path("C:\\Vault\\Problems", "/Vault/Problems"),
                test_path("C:\\Vault\\Knowledge", "/Vault/Knowledge"),
            )
            .expect("configured workspace"),
        );
        assert_eq!(
            select_startup_destination(
                &StartupGateStatus::Ready { schema_version: 2 },
                Some(&configured),
            ),
            StartupDestination::Normal
        );
    }

    struct RecordingPatchPort {
        calls: Cell<u32>,
    }

    impl PersonalNotePatchPort for RecordingPatchPort {
        async fn add_prerequisite_link(
            &self,
            _problem: &acm_os_domain::CodeforcesProblemIdentity,
            _target: &PrerequisiteLinkTarget,
        ) -> Result<PersonalNoteBinding, PersonalNotePatchError> {
            Err(PersonalNotePatchError::PersistenceUnavailable)
        }

        async fn add_extra_problem_link(
            &self,
            _problem: &acm_os_domain::CodeforcesProblemIdentity,
            target: &ExtraProblemLinkTarget,
        ) -> Result<PersonalNoteBinding, PersonalNotePatchError> {
            assert_eq!(target.as_str(), "CF-2000-A");
            self.calls.set(self.calls.get() + 1);
            Ok(PersonalNoteBinding {
                vault_relative_path: "Problems/CF-1979-A.md".to_owned(),
                content_digest: "0".repeat(64),
                windows_file_key: None,
            })
        }
    }

    #[test]
    fn extra_problem_command_validates_semantics_before_calling_the_port() {
        let port = RecordingPatchPort {
            calls: Cell::new(0),
        };
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");
        for invalid in ["", " CF-2000-A", "CF-2000-A\n", "[[CF-2000-A]]", "A|alias"] {
            assert_eq!(
                run_ready(add_extra_problem_link(&port, &problem, invalid)),
                Err(PersonalNotePatchError::InvalidLinkTarget)
            );
        }
        assert_eq!(port.calls.get(), 0);
        run_ready(add_extra_problem_link(&port, &problem, "CF-2000-A"))
            .expect("valid semantic command");
        assert_eq!(port.calls.get(), 1);
    }

    #[test]
    fn missing_workspace_query_result_fails_closed() {
        assert_eq!(
            select_startup_destination(&StartupGateStatus::Ready { schema_version: 2 }, None,),
            StartupDestination::Recovery {
                reason: StartupRecoveryReason::DatabaseUnavailable,
            }
        );
    }

    #[test]
    fn contest_ai_analysis_preview_distinguishes_complete_partial_and_failed() {
        let complete = preview_contest_ai_analysis("# Contest AI Analysis\n\n## Overall\n\n### Overall Difficulty\nHard\n\n## Problem A\n\n### Analysis\nMissed invariant").expect("complete preview");
        assert_eq!(complete.parse_status, ContestAiParseStatus::Complete);
        assert_eq!(
            complete.parsed_projection_json,
            r#"{"overall":true,"problemCount":1}"#
        );
        assert_eq!(
            preview_contest_ai_analysis("# Contest AI Analysis\n\n## Overall\nText")
                .expect("partial")
                .parse_status,
            ContestAiParseStatus::Partial
        );
        assert_eq!(
            preview_contest_ai_analysis("arbitrary response")
                .expect("failed")
                .parse_status,
            ContestAiParseStatus::Failed
        );
        assert_eq!(
            preview_contest_ai_analysis("  "),
            Err(ContestAiAnalysisError::Empty)
        );
    }

    #[test]
    fn manual_contest_builder_uses_canonical_identity_and_escapes_statement_text() {
        let problem = ManualProblemDraft {
            index: "A".to_owned(),
            title: "Manual A".to_owned(),
            source_url: "https://codeforces.com/contest/1979/problem/A".to_owned(),
            statement_text: "x < y && <script>alert(1)</script>".to_owned(),
        };
        let plan = build_manual_codeforces_contest(
            1979,
            "Manual Round",
            "https://codeforces.com/contest/1979",
            Some("2026-08-13T00:00:00Z".to_owned()),
            std::slice::from_ref(&problem),
        )
        .expect("manual plan");
        assert_eq!(plan.manifest.contest.contest_id(), 1979);
        assert_eq!(plan.snapshots[0].problem.index(), "A");
        assert!(plan.snapshots[0].sanitized_html.contains("&lt;script&gt;"));
        assert!(!plan.snapshots[0].sanitized_html.contains("<script>"));
        assert_eq!(
            build_manual_codeforces_contest(
                1979,
                "Manual Round",
                "https://codeforces.com/contest/1979",
                None,
                &[problem.clone(), problem]
            ),
            Err(ManualContestError::DuplicateProblem)
        );
    }
}
