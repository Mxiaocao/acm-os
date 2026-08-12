#![forbid(unsafe_code)]

pub mod codeforces;

use std::path::{Component, Path};

pub const BOUNDARY_NAME: &str = "acm-os-application";

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
        Ok(Self { manifest, snapshots })
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
    Ok(ContestImportRun { persisted, failed_snapshot_problems })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestShelfItem {
    pub contest: acm_os_domain::CodeforcesContestIdentity,
    pub title: String,
    pub import_status: ContestImportStatus,
    pub problem_count: u32,
    pub missing_snapshot_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestDetail {
    pub contest: acm_os_domain::CodeforcesContestIdentity,
    pub title: String,
    pub source_url: String,
    pub import_status: ContestImportStatus,
    pub problems: Vec<LightweightProblemItem>,
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
    let lifecycle = port
        .load_problem_lifecycle(problem)
        .await
        .map_err(|error| match error {
            ProblemLifecycleError::ProblemNotFound => ReviewAttemptError::ProblemNotFound,
            ProblemLifecycleError::NotPersonal => ReviewAttemptError::NotPersonal,
            ProblemLifecycleError::IntegrityViolation => ReviewAttemptError::IntegrityViolation,
            ProblemLifecycleError::PersistenceUnavailable => {
                ReviewAttemptError::PersistenceUnavailable
            }
            ProblemLifecycleError::InvalidTransition | ProblemLifecycleError::InvalidLocalDate => {
                ReviewAttemptError::NotEligible
            }
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
            ReviewFailureReason::Other(text) if !text.trim().is_empty() && text.len() <= 500 => "other",
            ReviewFailureReason::Other(_) => return Err(ReviewAttemptError::InvalidCompletionFacts),
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
    async fn add_extra_problem_link(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        target: &ExtraProblemLinkTarget,
    ) -> Result<PersonalNoteBinding, PersonalNotePatchError>;
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

pub const INITIAL_PROBLEM_MARKDOWN: &str = "# Problem\n\n## 前置知识\n\n## 题解\n\n### 标准推导\n\n## 额外题目\n";

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
    async fn list_lightweight_problems(&self) -> Result<Vec<LightweightProblemItem>, ContestReadError>;
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
        Self::validated(
            active_vault_path,
            problem_root_path,
            knowledge_root_path,
        )
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
    async fn resolve_directory(
        &self,
        path: &str,
    ) -> Result<String, WorkspacePathResolutionError>;

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

async fn resolve_required_directory<P: WorkspaceConfigurationPort>(
    port: &P,
    field: WorkspacePathField,
    raw_path: &str,
) -> Result<String, WorkspaceConfigurationError> {
    let path = raw_path.trim();
    if path.is_empty() {
        return Err(WorkspaceConfigurationError::PathRequired { field });
    }

    port.resolve_directory(path).await.map_err(|reason| match reason {
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
            vec![problem_slot(contest.clone(), 1, "A"), problem_slot(contest.clone(), 2, "B")],
        )
        .expect("manifest");
        let snapshot = |index: &str| StatementSnapshotDraft {
            problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), index).expect("problem"),
            source_html: format!("<div class=\"problem-statement\">{index}</div>"),
            sanitized_html: format!("<div class=\"problem-statement\">{index}</div>"),
            assets: Vec::new(),
        };
        let plan = ContestImportExecutionPlan::validated(manifest, vec![snapshot("A"), snapshot("B")])
            .expect("execution plan");
        let missing_b = acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "B").expect("problem B");
        assert_eq!(plan.snapshots_for_missing(&[missing_b])[0].problem.index(), "B");

        let foreign = StatementSnapshotDraft {
            problem: acm_os_domain::CodeforcesProblemIdentity::new(contest, "C").expect("problem C"),
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
        let port = RecordingPatchPort { calls: Cell::new(0) };
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");
        for invalid in [
            "",
            " CF-2000-A",
            "CF-2000-A\n",
            "[[CF-2000-A]]",
            "A|alias",
        ] {
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
            select_startup_destination(
                &StartupGateStatus::Ready { schema_version: 2 },
                None,
            ),
            StartupDestination::Recovery {
                reason: StartupRecoveryReason::DatabaseUnavailable,
            }
        );
    }
}
