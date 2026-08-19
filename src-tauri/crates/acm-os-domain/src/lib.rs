#![forbid(unsafe_code)]

pub const BOUNDARY_NAME: &str = "acm-os-domain";

mod identity;

pub use identity::{
    ContestIdentity, ExternalContestKey, GenericIdentityError, PlatformKey, ProblemIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeforcesContestIdentity {
    contest_id: u64,
}

impl CodeforcesContestIdentity {
    pub fn new(contest_id: u64) -> Result<Self, IdentityError> {
        if contest_id == 0 {
            return Err(IdentityError::InvalidContestId);
        }
        Ok(Self { contest_id })
    }

    pub const fn contest_id(&self) -> u64 {
        self.contest_id
    }

    pub const fn platform(&self) -> &'static str {
        "codeforces"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeforcesProblemIdentity {
    contest: CodeforcesContestIdentity,
    index: String,
}

impl CodeforcesProblemIdentity {
    pub fn new(
        contest: CodeforcesContestIdentity,
        index: impl Into<String>,
    ) -> Result<Self, IdentityError> {
        let index = index.into();
        let valid = !index.is_empty()
            && index.len() <= 8
            && index
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
        if !valid {
            return Err(IdentityError::InvalidProblemIndex);
        }
        Ok(Self { contest, index })
    }

    pub fn contest(&self) -> &CodeforcesContestIdentity {
        &self.contest
    }

    pub fn index(&self) -> &str {
        &self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {
    InvalidContestId,
    InvalidProblemIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningStatus {
    Unstarted,
    UpsolvePending,
    Learning,
    WaitingColdStart,
    Relearning,
    LongTermReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemLifecycleAction {
    JoinUpsolve,
    StartLearning,
    ReturnToPending,
    MarkUnderstood,
    WithdrawUnderstood,
    StartRelearning,
    StopLearning,
    DeletePersonalNote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewCycleDirective {
    None,
    StartFirstColdStart,
    CancelActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProblemLifecycleDecision {
    pub previous_status: LearningStatus,
    pub next_status: LearningStatus,
    pub review_cycle: ReviewCycleDirective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidProblemLifecycleTransition {
    pub status: LearningStatus,
    pub action: ProblemLifecycleAction,
}

pub struct ProblemLifecycleEngine;

impl ProblemLifecycleEngine {
    pub fn available_actions(status: LearningStatus) -> &'static [ProblemLifecycleAction] {
        use LearningStatus::{
            Learning, LongTermReview, Relearning, Unstarted, UpsolvePending, WaitingColdStart,
        };
        use ProblemLifecycleAction::{
            JoinUpsolve, MarkUnderstood, ReturnToPending, StartLearning, StartRelearning,
            StopLearning, WithdrawUnderstood,
        };

        match status {
            Unstarted => &[JoinUpsolve],
            UpsolvePending => &[StartLearning, StopLearning],
            Learning => &[MarkUnderstood, ReturnToPending, StopLearning],
            WaitingColdStart => &[WithdrawUnderstood, StopLearning],
            Relearning => &[StartRelearning, StopLearning],
            LongTermReview => &[],
        }
    }

    pub fn decide(
        status: LearningStatus,
        action: ProblemLifecycleAction,
    ) -> Result<ProblemLifecycleDecision, InvalidProblemLifecycleTransition> {
        use LearningStatus::{
            Learning, LongTermReview, Relearning, Unstarted, UpsolvePending, WaitingColdStart,
        };
        use ProblemLifecycleAction::{
            DeletePersonalNote, JoinUpsolve, MarkUnderstood, ReturnToPending, StartLearning,
            StartRelearning, StopLearning, WithdrawUnderstood,
        };
        use ReviewCycleDirective::{CancelActive, None, StartFirstColdStart};

        let (next_status, review_cycle) = match (status, action) {
            (Unstarted, JoinUpsolve) => (UpsolvePending, None),
            (UpsolvePending, StartLearning) => (Learning, None),
            (Learning, ReturnToPending) => (UpsolvePending, None),
            (Learning, MarkUnderstood) => (WaitingColdStart, StartFirstColdStart),
            (WaitingColdStart, WithdrawUnderstood) => (Learning, CancelActive),
            (Relearning, StartRelearning) => (Learning, None),
            (UpsolvePending | Learning | Relearning, StopLearning) => (Unstarted, None),
            (WaitingColdStart, StopLearning) => (Unstarted, CancelActive),
            (
                Unstarted | UpsolvePending | Learning | WaitingColdStart | Relearning
                | LongTermReview,
                DeletePersonalNote,
            ) => (Unstarted, CancelActive),
            _ => return Err(InvalidProblemLifecycleTransition { status, action }),
        };

        Ok(ProblemLifecycleDecision {
            previous_status: status,
            next_status,
            review_cycle,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalDate(chrono::NaiveDate);

impl LocalDate {
    pub fn parse_iso(value: &str) -> Result<Self, InvalidLocalDate> {
        chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| InvalidLocalDate)
    }

    pub fn to_iso_string(self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }

    pub fn iso_weekday_number(self) -> u8 {
        use chrono::Datelike;
        self.0.weekday().number_from_monday() as u8
    }

    fn checked_add_days(self, days: u64) -> Result<Self, InvalidLocalDate> {
        self.0
            .checked_add_days(chrono::Days::new(days))
            .map(Self)
            .ok_or(InvalidLocalDate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidLocalDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnowledgeUnderstandingLevel {
    NotLearned,
    Vague,
    Basic,
    Proficient,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgeUnderstandingDecision {
    pub current: KnowledgeUnderstandingLevel,
    pub historical_highest: KnowledgeUnderstandingLevel,
    pub first_reached_highest_on: LocalDate,
}

pub fn confirm_knowledge_understanding(
    previous_highest: Option<(KnowledgeUnderstandingLevel, LocalDate)>,
    selected: KnowledgeUnderstandingLevel,
    confirmed_on: LocalDate,
) -> KnowledgeUnderstandingDecision {
    match previous_highest {
        Some((highest, first_on)) if highest >= selected => KnowledgeUnderstandingDecision {
            current: selected,
            historical_highest: highest,
            first_reached_highest_on: first_on,
        },
        _ => KnowledgeUnderstandingDecision {
            current: selected,
            historical_highest: selected,
            first_reached_highest_on: confirmed_on,
        },
    }
}

pub struct ReviewSchedulingEngine;

impl ReviewSchedulingEngine {
    pub const SCHEDULE_RULE_VERSION: u32 = 1;
    pub const FIRST_COLD_START_DAYS: u64 = 3;

    pub fn first_cold_start_due(
        marked_understood_on: LocalDate,
    ) -> Result<LocalDate, InvalidLocalDate> {
        marked_understood_on.checked_add_days(Self::FIRST_COLD_START_DAYS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewAttemptType {
    FirstColdStart,
    LongTermReview,
    EarlyCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewHelpLevel {
    PrerequisiteNames,
    Hints,
    PrerequisiteContent,
    OldIdeaOrCode,
    FullSolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionResult {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    CompilationError,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugIndependence {
    NotNeeded,
    Independent,
    UsedSolvingHelp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalHelpLevel {
    None,
    SolvingHint,
    FullSolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewJudgement {
    Fail,
    Partial,
    Mastered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCompletionFacts {
    pub final_ac: bool,
    pub first_submission_result: SubmissionResult,
    pub final_result: SubmissionResult,
    pub total_submissions: u32,
    pub idea_independent: bool,
    pub implementation_independent: bool,
    pub debug_independence: DebugIndependence,
    pub external_help: ExternalHelpLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewJudgementDecision {
    pub judgement: ReviewJudgement,
    pub evidence_codes: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewFactsError {
    SubmissionCountMissing,
    FinalResultContradiction,
    SingleSubmissionContradiction,
}

pub struct ReviewJudgementEngine;

impl ReviewJudgementEngine {
    pub fn judge(
        facts: &ReviewCompletionFacts,
        highest_help_level: Option<ReviewHelpLevel>,
    ) -> Result<ReviewJudgementDecision, ReviewFactsError> {
        if facts.total_submissions == 0 {
            return Err(ReviewFactsError::SubmissionCountMissing);
        }
        if facts.final_ac != (facts.final_result == SubmissionResult::Accepted) {
            return Err(ReviewFactsError::FinalResultContradiction);
        }
        if facts.total_submissions == 1 && facts.first_submission_result != facts.final_result {
            return Err(ReviewFactsError::SingleSubmissionContradiction);
        }
        let full_solution_used = highest_help_level == Some(ReviewHelpLevel::FullSolution)
            || facts.external_help == ExternalHelpLevel::FullSolution;
        let solving_help_used = highest_help_level.is_some()
            || facts.external_help != ExternalHelpLevel::None
            || facts.debug_independence == DebugIndependence::UsedSolvingHelp;
        let judgement = if !facts.final_ac || full_solution_used {
            ReviewJudgement::Fail
        } else if solving_help_used || !facts.idea_independent || !facts.implementation_independent
        {
            ReviewJudgement::Partial
        } else {
            ReviewJudgement::Mastered
        };
        let mut evidence_codes = vec![if facts.final_ac {
            "final_ac"
        } else {
            "no_final_ac"
        }];
        if let Some(level) = highest_help_level {
            evidence_codes.push(match level {
                ReviewHelpLevel::PrerequisiteNames => "controlled_help_l1",
                ReviewHelpLevel::Hints => "controlled_help_l2",
                ReviewHelpLevel::PrerequisiteContent => "controlled_help_l3",
                ReviewHelpLevel::OldIdeaOrCode => "controlled_help_l4",
                ReviewHelpLevel::FullSolution => "controlled_help_l5",
            });
        }
        if facts.external_help != ExternalHelpLevel::None {
            evidence_codes.push(match facts.external_help {
                ExternalHelpLevel::None => unreachable!(),
                ExternalHelpLevel::SolvingHint => "external_solving_hint",
                ExternalHelpLevel::FullSolution => "external_full_solution",
            });
        }
        if !facts.idea_independent {
            evidence_codes.push("idea_not_independent");
        }
        if !facts.implementation_independent {
            evidence_codes.push("implementation_not_independent");
        }
        match facts.debug_independence {
            DebugIndependence::NotNeeded => evidence_codes.push("debug_not_needed"),
            DebugIndependence::Independent => evidence_codes.push("debug_independent"),
            DebugIndependence::UsedSolvingHelp => evidence_codes.push("debug_solving_help"),
        }
        Ok(ReviewJudgementDecision {
            judgement,
            evidence_codes,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewCycleCompletion {
    Keep,
    Advance {
        next_stage: u32,
        next_due: LocalDate,
    },
    Suspend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewCompletionDecision {
    pub next_learning_status: LearningStatus,
    pub cycle: ReviewCycleCompletion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidReviewCompletion;

impl ReviewSchedulingEngine {
    pub const REVIEW_INTERVAL_DAYS: [u64; 7] = [3, 10, 30, 75, 150, 240, 240];

    pub fn complete_review(
        status: LearningStatus,
        attempt_type: ReviewAttemptType,
        judgement: ReviewJudgement,
        current_stage: u32,
        completed_on: LocalDate,
    ) -> Result<ReviewCompletionDecision, InvalidReviewCompletion> {
        if !matches!(
            status,
            LearningStatus::WaitingColdStart | LearningStatus::LongTermReview
        ) {
            return Err(InvalidReviewCompletion);
        }
        if current_stage > 6
            || (status == LearningStatus::WaitingColdStart && current_stage != 0)
            || (status == LearningStatus::LongTermReview && current_stage == 0)
        {
            return Err(InvalidReviewCompletion);
        }
        if judgement != ReviewJudgement::Mastered {
            return Ok(ReviewCompletionDecision {
                next_learning_status: LearningStatus::Relearning,
                cycle: ReviewCycleCompletion::Suspend,
            });
        }
        if attempt_type == ReviewAttemptType::EarlyCheck {
            return Ok(ReviewCompletionDecision {
                next_learning_status: status,
                cycle: ReviewCycleCompletion::Keep,
            });
        }
        let expected_type = match status {
            LearningStatus::WaitingColdStart => ReviewAttemptType::FirstColdStart,
            LearningStatus::LongTermReview => ReviewAttemptType::LongTermReview,
            _ => unreachable!(),
        };
        if attempt_type != expected_type {
            return Err(InvalidReviewCompletion);
        }
        let next_stage = current_stage.saturating_add(1).min(6);
        let next_due = completed_on
            .checked_add_days(Self::REVIEW_INTERVAL_DAYS[next_stage as usize])
            .map_err(|_| InvalidReviewCompletion)?;
        Ok(ReviewCompletionDecision {
            next_learning_status: LearningStatus::LongTermReview,
            cycle: ReviewCycleCompletion::Advance {
                next_stage,
                next_due,
            },
        })
    }
}

impl ReviewHelpLevel {
    pub const fn number(self) -> u8 {
        match self {
            Self::PrerequisiteNames => 1,
            Self::Hints => 2,
            Self::PrerequisiteContent => 3,
            Self::OldIdeaOrCode => 4,
            Self::FullSolution => 5,
        }
    }

    pub const fn from_number(number: u8) -> Option<Self> {
        match number {
            1 => Some(Self::PrerequisiteNames),
            2 => Some(Self::Hints),
            3 => Some(Self::PrerequisiteContent),
            4 => Some(Self::OldIdeaOrCode),
            5 => Some(Self::FullSolution),
            _ => None,
        }
    }

    pub const fn consequence_code(self) -> &'static str {
        match self {
            Self::FullSolution => "fail_only",
            Self::PrerequisiteNames
            | Self::Hints
            | Self::PrerequisiteContent
            | Self::OldIdeaOrCode => "partial_at_best",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewEligibilityDecision {
    pub attempt_type: ReviewAttemptType,
    pub scheduled_due_local_date: LocalDate,
    pub started_early: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewNotEligible {
    pub status: LearningStatus,
}

pub struct ReviewEligibilityEngine;

impl ReviewEligibilityEngine {
    pub const JUDGEMENT_RULE_VERSION: u32 = 1;

    pub fn decide(
        status: LearningStatus,
        scheduled_due_local_date: LocalDate,
        today: LocalDate,
    ) -> Result<ReviewEligibilityDecision, ReviewNotEligible> {
        let scheduled_attempt_type = match status {
            LearningStatus::WaitingColdStart => ReviewAttemptType::FirstColdStart,
            LearningStatus::LongTermReview => ReviewAttemptType::LongTermReview,
            _ => return Err(ReviewNotEligible { status }),
        };
        let started_early = today < scheduled_due_local_date;
        Ok(ReviewEligibilityDecision {
            attempt_type: if started_early {
                ReviewAttemptType::EarlyCheck
            } else {
                scheduled_attempt_type
            },
            scheduled_due_local_date,
            started_early,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TodayInProgressReview<'a> {
    pub attempt_id: &'a str,
    pub scheduled_due_local_date: LocalDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TodayCandidateInput<'a> {
    pub problem_id: &'a str,
    pub learning_status: LearningStatus,
    pub learning_status_since: LocalDate,
    pub pinned: bool,
    pub active_review_due: Option<LocalDate>,
    pub in_progress_review: Option<TodayInProgressReview<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodayCandidateLane {
    CarryIn,
    Review,
    Study,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodayCandidateReason {
    ContinueReview,
    ContinueLearning,
    DueFirstColdStart,
    DueLongTermReview,
    Relearn,
    Upsolve,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayCandidate {
    pub problem_id: String,
    pub review_attempt_id: Option<String>,
    pub lane: TodayCandidateLane,
    pub reason: TodayCandidateReason,
    pub planning_cost_minutes: u32,
    pub pinned: bool,
    pub learning_status_since: LocalDate,
    pub scheduled_due_local_date: Option<LocalDate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodayCandidateError {
    EmptyProblemId,
    EmptyReviewAttemptId {
        problem_id: String,
    },
    DuplicateProblem {
        problem_id: String,
    },
    MissingActiveReviewDue {
        problem_id: String,
        status: LearningStatus,
    },
    InProgressReviewLifecycleMismatch {
        problem_id: String,
        status: LearningStatus,
    },
}

pub struct TodayCandidateBuilder;

impl TodayCandidateBuilder {
    pub const REVIEW_PLANNING_COST_MINUTES: u32 = 30;
    pub const STUDY_PLANNING_COST_MINUTES: u32 = 60;

    pub fn build(
        today: LocalDate,
        inputs: &[TodayCandidateInput<'_>],
    ) -> Result<Vec<TodayCandidate>, TodayCandidateError> {
        let mut seen_problem_ids = std::collections::HashSet::with_capacity(inputs.len());
        let mut candidates = Vec::with_capacity(inputs.len());

        for input in inputs {
            if input.problem_id.is_empty() {
                return Err(TodayCandidateError::EmptyProblemId);
            }
            if !seen_problem_ids.insert(input.problem_id) {
                return Err(TodayCandidateError::DuplicateProblem {
                    problem_id: input.problem_id.to_owned(),
                });
            }
            if let Some(candidate) = Self::build_one(today, *input)? {
                candidates.push(candidate);
            }
        }

        Ok(candidates)
    }

    fn build_one(
        today: LocalDate,
        input: TodayCandidateInput<'_>,
    ) -> Result<Option<TodayCandidate>, TodayCandidateError> {
        if let Some(review) = input.in_progress_review {
            if review.attempt_id.is_empty() {
                return Err(TodayCandidateError::EmptyReviewAttemptId {
                    problem_id: input.problem_id.to_owned(),
                });
            }
            if !matches!(
                input.learning_status,
                LearningStatus::WaitingColdStart | LearningStatus::LongTermReview
            ) {
                return Err(TodayCandidateError::InProgressReviewLifecycleMismatch {
                    problem_id: input.problem_id.to_owned(),
                    status: input.learning_status,
                });
            }
            return Ok(Some(TodayCandidate {
                problem_id: input.problem_id.to_owned(),
                review_attempt_id: Some(review.attempt_id.to_owned()),
                lane: TodayCandidateLane::CarryIn,
                reason: TodayCandidateReason::ContinueReview,
                planning_cost_minutes: Self::REVIEW_PLANNING_COST_MINUTES,
                pinned: input.pinned,
                learning_status_since: input.learning_status_since,
                scheduled_due_local_date: Some(review.scheduled_due_local_date),
            }));
        }

        let candidate = match input.learning_status {
            LearningStatus::Unstarted => None,
            LearningStatus::Learning => Some((
                TodayCandidateLane::CarryIn,
                TodayCandidateReason::ContinueLearning,
                Self::STUDY_PLANNING_COST_MINUTES,
                None,
            )),
            LearningStatus::Relearning => Some((
                TodayCandidateLane::Study,
                TodayCandidateReason::Relearn,
                Self::STUDY_PLANNING_COST_MINUTES,
                None,
            )),
            LearningStatus::UpsolvePending => Some((
                TodayCandidateLane::Study,
                TodayCandidateReason::Upsolve,
                Self::STUDY_PLANNING_COST_MINUTES,
                None,
            )),
            status @ (LearningStatus::WaitingColdStart | LearningStatus::LongTermReview) => {
                let due = input.active_review_due.ok_or_else(|| {
                    TodayCandidateError::MissingActiveReviewDue {
                        problem_id: input.problem_id.to_owned(),
                        status,
                    }
                })?;
                if due > today {
                    None
                } else {
                    Some((
                        TodayCandidateLane::Review,
                        if status == LearningStatus::WaitingColdStart {
                            TodayCandidateReason::DueFirstColdStart
                        } else {
                            TodayCandidateReason::DueLongTermReview
                        },
                        Self::REVIEW_PLANNING_COST_MINUTES,
                        Some(due),
                    ))
                }
            }
        };

        Ok(candidate.map(
            |(lane, reason, planning_cost_minutes, due)| TodayCandidate {
                problem_id: input.problem_id.to_owned(),
                review_attempt_id: None,
                lane,
                reason,
                planning_cost_minutes,
                pinned: input.pinned,
                learning_status_since: input.learning_status_since,
                scheduled_due_local_date: due,
            },
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayOrderedCandidates {
    pub carry_in: Vec<TodayCandidate>,
    pub review: Vec<TodayCandidate>,
    pub study: Vec<TodayCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodayCandidateOrderingError {
    LaneReasonMismatch { problem_id: String },
    MissingReviewDue { problem_id: String },
}

pub struct TodayCandidateOrderingEngine;

impl TodayCandidateOrderingEngine {
    pub fn order(
        candidates: &[TodayCandidate],
    ) -> Result<TodayOrderedCandidates, TodayCandidateOrderingError> {
        let mut ordered = TodayOrderedCandidates {
            carry_in: Vec::new(),
            review: Vec::new(),
            study: Vec::new(),
        };

        for candidate in candidates {
            let reason_matches_lane = matches!(
                (candidate.lane, candidate.reason),
                (
                    TodayCandidateLane::CarryIn,
                    TodayCandidateReason::ContinueReview | TodayCandidateReason::ContinueLearning
                ) | (
                    TodayCandidateLane::Review,
                    TodayCandidateReason::DueFirstColdStart
                        | TodayCandidateReason::DueLongTermReview
                ) | (
                    TodayCandidateLane::Study,
                    TodayCandidateReason::Relearn | TodayCandidateReason::Upsolve
                )
            );
            if !reason_matches_lane {
                return Err(TodayCandidateOrderingError::LaneReasonMismatch {
                    problem_id: candidate.problem_id.clone(),
                });
            }
            if candidate.lane == TodayCandidateLane::Review
                && candidate.scheduled_due_local_date.is_none()
            {
                return Err(TodayCandidateOrderingError::MissingReviewDue {
                    problem_id: candidate.problem_id.clone(),
                });
            }
            match candidate.lane {
                TodayCandidateLane::CarryIn => ordered.carry_in.push(candidate.clone()),
                TodayCandidateLane::Review => ordered.review.push(candidate.clone()),
                TodayCandidateLane::Study => ordered.study.push(candidate.clone()),
            }
        }

        ordered
            .carry_in
            .sort_by(|left, right| left.problem_id.cmp(&right.problem_id));
        ordered.review.sort_by(|left, right| {
            left.scheduled_due_local_date
                .cmp(&right.scheduled_due_local_date)
                .then_with(|| right.pinned.cmp(&left.pinned))
                .then_with(|| {
                    review_reason_rank(left.reason).cmp(&review_reason_rank(right.reason))
                })
                .then_with(|| left.problem_id.cmp(&right.problem_id))
        });
        ordered.study.sort_by(|left, right| {
            study_reason_rank(left.reason)
                .cmp(&study_reason_rank(right.reason))
                .then_with(|| right.pinned.cmp(&left.pinned))
                .then_with(|| left.learning_status_since.cmp(&right.learning_status_since))
                .then_with(|| left.problem_id.cmp(&right.problem_id))
        });

        Ok(ordered)
    }
}

fn review_reason_rank(reason: TodayCandidateReason) -> u8 {
    match reason {
        TodayCandidateReason::DueFirstColdStart => 0,
        TodayCandidateReason::DueLongTermReview => 1,
        _ => 2,
    }
}

fn study_reason_rank(reason: TodayCandidateReason) -> u8 {
    match reason {
        TodayCandidateReason::Relearn => 0,
        TodayCandidateReason::Upsolve => 1,
        _ => 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TodayPlanningCapacity {
    pub can_fit_review: bool,
    pub can_fit_study: bool,
    pub can_fit_both: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodayLaneRequirement {
    None,
    Review,
    Study,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TodayAntiStarvationDecision {
    pub required_lanes: TodayLaneRequirement,
    pub next_review_only_streak: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodayAntiStarvationError {
    InvalidCapacity,
    InvalidReviewOnlyStreak,
    StudyRequiredButCannotFit,
}

pub struct TodayAntiStarvationEngine;

impl TodayAntiStarvationEngine {
    pub const MAX_CONSECUTIVE_REVIEW_ONLY_DAYS_WITH_STUDY_BACKLOG: u8 = 2;

    pub fn decide(
        has_review_backlog: bool,
        has_study_backlog: bool,
        capacity: TodayPlanningCapacity,
        consecutive_review_only_days_with_study_backlog: u8,
    ) -> Result<TodayAntiStarvationDecision, TodayAntiStarvationError> {
        if capacity.can_fit_both && !(capacity.can_fit_review && capacity.can_fit_study) {
            return Err(TodayAntiStarvationError::InvalidCapacity);
        }
        if consecutive_review_only_days_with_study_backlog
            > Self::MAX_CONSECUTIVE_REVIEW_ONLY_DAYS_WITH_STUDY_BACKLOG
        {
            return Err(TodayAntiStarvationError::InvalidReviewOnlyStreak);
        }
        if has_review_backlog
            && has_study_backlog
            && consecutive_review_only_days_with_study_backlog
                == Self::MAX_CONSECUTIVE_REVIEW_ONLY_DAYS_WITH_STUDY_BACKLOG
            && !capacity.can_fit_study
        {
            return Err(TodayAntiStarvationError::StudyRequiredButCannotFit);
        }

        let required_lanes = match (has_review_backlog, has_study_backlog) {
            (false, false) => TodayLaneRequirement::None,
            (true, false) if capacity.can_fit_review => TodayLaneRequirement::Review,
            (false, true) if capacity.can_fit_study => TodayLaneRequirement::Study,
            (true, true) if capacity.can_fit_both => TodayLaneRequirement::Both,
            (true, true)
                if capacity.can_fit_study
                    && consecutive_review_only_days_with_study_backlog
                        >= Self::MAX_CONSECUTIVE_REVIEW_ONLY_DAYS_WITH_STUDY_BACKLOG =>
            {
                TodayLaneRequirement::Study
            }
            (true, true) if capacity.can_fit_review => TodayLaneRequirement::Review,
            (true, true) if capacity.can_fit_study => TodayLaneRequirement::Study,
            _ => TodayLaneRequirement::None,
        };

        let next_review_only_streak = match required_lanes {
            TodayLaneRequirement::Review if has_study_backlog => {
                consecutive_review_only_days_with_study_backlog + 1
            }
            TodayLaneRequirement::Both | TodayLaneRequirement::Study => 0,
            TodayLaneRequirement::None | TodayLaneRequirement::Review => 0,
        };

        Ok(TodayAntiStarvationDecision {
            required_lanes,
            next_review_only_streak,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayPlanDraft {
    pub entries: Vec<TodayCandidate>,
    pub budget_minutes: u32,
    pub planned_minutes: u32,
    pub over_budget_minutes: u32,
    pub unplanned_review_count: usize,
    pub unplanned_study_count: usize,
    pub next_review_only_streak: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodayPlannerError {
    InvalidPlanningCost { problem_id: String },
    LaneMismatch { problem_id: String },
    DuplicateProblem { problem_id: String },
    RequiredReviewUnavailable,
    RequiredStudyUnavailable,
    RequiredLanesDoNotFit,
}

pub struct TodayPlanner;

impl TodayPlanner {
    pub fn plan_generated(
        ordered: &TodayOrderedCandidates,
        budget_minutes: u32,
        consecutive_review_only_days_with_study_backlog: u8,
    ) -> Result<TodayPlanDraft, TodayPlannerError> {
        Self::validate(ordered)?;
        let carry_in_minutes = Self::total_minutes(&ordered.carry_in)?;
        let remaining_minutes = budget_minutes.saturating_sub(carry_in_minutes);
        let first_review_cost = ordered
            .review
            .first()
            .map(|candidate| candidate.planning_cost_minutes);
        let first_study_cost = ordered
            .study
            .first()
            .map(|candidate| candidate.planning_cost_minutes);
        let capacity = TodayPlanningCapacity {
            can_fit_review: first_review_cost.is_some_and(|cost| cost <= remaining_minutes),
            can_fit_study: first_study_cost.is_some_and(|cost| cost <= remaining_minutes),
            can_fit_both: first_review_cost
                .zip(first_study_cost)
                .and_then(|(review, study)| review.checked_add(study))
                .is_some_and(|cost| cost <= remaining_minutes),
        };
        let anti_starvation = TodayAntiStarvationEngine::decide(
            !ordered.review.is_empty(),
            !ordered.study.is_empty(),
            capacity,
            consecutive_review_only_days_with_study_backlog,
        )
        .map_err(|_| TodayPlannerError::RequiredLanesDoNotFit)?;
        let mut draft = Self::plan(ordered, budget_minutes, anti_starvation.required_lanes)?;
        draft.next_review_only_streak = anti_starvation.next_review_only_streak;
        Ok(draft)
    }

    pub fn plan(
        ordered: &TodayOrderedCandidates,
        budget_minutes: u32,
        required_lanes: TodayLaneRequirement,
    ) -> Result<TodayPlanDraft, TodayPlannerError> {
        Self::validate(ordered)?;

        let mut entries = ordered.carry_in.clone();
        let carry_in_minutes = Self::total_minutes(&entries)?;
        let mut remaining_minutes = budget_minutes.saturating_sub(carry_in_minutes);
        let mut review_index = 0;
        let mut study_index = 0;

        match required_lanes {
            TodayLaneRequirement::None => {}
            TodayLaneRequirement::Review => {
                let review = ordered
                    .review
                    .first()
                    .ok_or(TodayPlannerError::RequiredReviewUnavailable)?;
                if review.planning_cost_minutes > remaining_minutes {
                    return Err(TodayPlannerError::RequiredLanesDoNotFit);
                }
                entries.push(review.clone());
                remaining_minutes -= review.planning_cost_minutes;
                review_index = 1;
            }
            TodayLaneRequirement::Study => {
                let study = ordered
                    .study
                    .first()
                    .ok_or(TodayPlannerError::RequiredStudyUnavailable)?;
                if study.planning_cost_minutes > remaining_minutes {
                    return Err(TodayPlannerError::RequiredLanesDoNotFit);
                }
                entries.push(study.clone());
                remaining_minutes -= study.planning_cost_minutes;
                study_index = 1;
            }
            TodayLaneRequirement::Both => {
                let review = ordered
                    .review
                    .first()
                    .ok_or(TodayPlannerError::RequiredReviewUnavailable)?;
                let study = ordered
                    .study
                    .first()
                    .ok_or(TodayPlannerError::RequiredStudyUnavailable)?;
                let required_minutes = review
                    .planning_cost_minutes
                    .checked_add(study.planning_cost_minutes)
                    .ok_or(TodayPlannerError::RequiredLanesDoNotFit)?;
                if required_minutes > remaining_minutes {
                    return Err(TodayPlannerError::RequiredLanesDoNotFit);
                }
                entries.push(review.clone());
                entries.push(study.clone());
                remaining_minutes -= required_minutes;
                review_index = 1;
                study_index = 1;
            }
        }

        while let Some(review) = ordered.review.get(review_index) {
            if review.planning_cost_minutes > remaining_minutes {
                break;
            }
            entries.push(review.clone());
            remaining_minutes -= review.planning_cost_minutes;
            review_index += 1;
        }
        while let Some(study) = ordered.study.get(study_index) {
            if study.planning_cost_minutes > remaining_minutes {
                break;
            }
            entries.push(study.clone());
            remaining_minutes -= study.planning_cost_minutes;
            study_index += 1;
        }

        let planned_minutes = Self::total_minutes(&entries)?;
        Ok(TodayPlanDraft {
            entries,
            budget_minutes,
            planned_minutes,
            over_budget_minutes: planned_minutes.saturating_sub(budget_minutes),
            unplanned_review_count: ordered.review.len() - review_index,
            unplanned_study_count: ordered.study.len() - study_index,
            next_review_only_streak: 0,
        })
    }

    fn validate(ordered: &TodayOrderedCandidates) -> Result<(), TodayPlannerError> {
        let mut seen_problem_ids = std::collections::HashSet::new();
        for (expected_lane, candidates) in [
            (TodayCandidateLane::CarryIn, &ordered.carry_in),
            (TodayCandidateLane::Review, &ordered.review),
            (TodayCandidateLane::Study, &ordered.study),
        ] {
            for candidate in candidates {
                if candidate.lane != expected_lane {
                    return Err(TodayPlannerError::LaneMismatch {
                        problem_id: candidate.problem_id.clone(),
                    });
                }
                let expected_cost = match candidate.reason {
                    TodayCandidateReason::ContinueReview
                    | TodayCandidateReason::DueFirstColdStart
                    | TodayCandidateReason::DueLongTermReview => {
                        TodayCandidateBuilder::REVIEW_PLANNING_COST_MINUTES
                    }
                    TodayCandidateReason::ContinueLearning
                    | TodayCandidateReason::Relearn
                    | TodayCandidateReason::Upsolve => {
                        TodayCandidateBuilder::STUDY_PLANNING_COST_MINUTES
                    }
                };
                if candidate.planning_cost_minutes != expected_cost {
                    return Err(TodayPlannerError::InvalidPlanningCost {
                        problem_id: candidate.problem_id.clone(),
                    });
                }
                if !seen_problem_ids.insert(candidate.problem_id.as_str()) {
                    return Err(TodayPlannerError::DuplicateProblem {
                        problem_id: candidate.problem_id.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn total_minutes(candidates: &[TodayCandidate]) -> Result<u32, TodayPlannerError> {
        candidates.iter().try_fold(0_u32, |total, candidate| {
            total.checked_add(candidate.planning_cost_minutes).ok_or(
                TodayPlannerError::InvalidPlanningCost {
                    problem_id: candidate.problem_id.clone(),
                },
            )
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProblemMasteryEvidence {
    pub recalls_problem: bool,
    pub multiple_solutions_clear: bool,
    pub knowledge_understood: bool,
    pub implementation_fluent: bool,
    pub can_adapt_or_create: bool,
    pub transfer_solved_independently: bool,
}

impl ProblemMasteryEvidence {
    pub const fn achieved_count(self) -> u8 {
        self.recalls_problem as u8
            + self.multiple_solutions_clear as u8
            + self.knowledge_understood as u8
            + self.implementation_fluent as u8
            + self.can_adapt_or_create as u8
            + self.transfer_solved_independently as u8
    }

    pub const fn is_thoroughly_digested(self) -> bool {
        self.achieved_count() == 6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codeforces_strong_identity_requires_a_positive_contest_id_and_canonical_index() {
        assert_eq!(
            CodeforcesContestIdentity::new(0),
            Err(IdentityError::InvalidContestId)
        );
        let contest = CodeforcesContestIdentity::new(1979).expect("contest identity");
        assert_eq!(contest.platform(), "codeforces");
        assert_eq!(contest.contest_id(), 1979);
        assert_eq!(
            CodeforcesProblemIdentity::new(contest.clone(), "a"),
            Err(IdentityError::InvalidProblemIndex)
        );
        assert_eq!(
            CodeforcesProblemIdentity::new(contest.clone(), "A/1"),
            Err(IdentityError::InvalidProblemIndex)
        );
        let problem = CodeforcesProblemIdentity::new(contest, "A1").expect("problem identity");
        assert_eq!(problem.index(), "A1");
        assert_eq!(problem.contest().contest_id(), 1979);
    }

    #[test]
    fn problem_lifecycle_follows_the_frozen_upsolve_path() {
        let joined = ProblemLifecycleEngine::decide(
            LearningStatus::Unstarted,
            ProblemLifecycleAction::JoinUpsolve,
        )
        .expect("join upsolve");
        assert_eq!(joined.next_status, LearningStatus::UpsolvePending);
        assert_eq!(joined.review_cycle, ReviewCycleDirective::None);

        let started = ProblemLifecycleEngine::decide(
            joined.next_status,
            ProblemLifecycleAction::StartLearning,
        )
        .expect("start learning");
        assert_eq!(started.next_status, LearningStatus::Learning);

        let understood = ProblemLifecycleEngine::decide(
            started.next_status,
            ProblemLifecycleAction::MarkUnderstood,
        )
        .expect("mark understood");
        assert_eq!(understood.next_status, LearningStatus::WaitingColdStart);
        assert_eq!(
            understood.review_cycle,
            ReviewCycleDirective::StartFirstColdStart
        );
    }

    #[test]
    fn problem_lifecycle_supports_frozen_retreat_stop_and_relearn_actions() {
        assert_eq!(
            ProblemLifecycleEngine::decide(
                LearningStatus::Learning,
                ProblemLifecycleAction::ReturnToPending,
            )
            .expect("return to pending")
            .next_status,
            LearningStatus::UpsolvePending
        );
        let withdrawn = ProblemLifecycleEngine::decide(
            LearningStatus::WaitingColdStart,
            ProblemLifecycleAction::WithdrawUnderstood,
        )
        .expect("withdraw understood");
        assert_eq!(withdrawn.next_status, LearningStatus::Learning);
        assert_eq!(withdrawn.review_cycle, ReviewCycleDirective::CancelActive);
        assert_eq!(
            ProblemLifecycleEngine::decide(
                LearningStatus::Relearning,
                ProblemLifecycleAction::StartRelearning,
            )
            .expect("start relearning")
            .next_status,
            LearningStatus::Learning
        );

        for status in [
            LearningStatus::UpsolvePending,
            LearningStatus::Learning,
            LearningStatus::WaitingColdStart,
            LearningStatus::Relearning,
        ] {
            let stopped =
                ProblemLifecycleEngine::decide(status, ProblemLifecycleAction::StopLearning)
                    .expect("eligible status can stop learning");
            assert_eq!(stopped.next_status, LearningStatus::Unstarted);
        }
    }

    #[test]
    fn delete_personal_note_exits_any_lifecycle_and_cancels_active_scheduling() {
        for status in [
            LearningStatus::Unstarted,
            LearningStatus::UpsolvePending,
            LearningStatus::Learning,
            LearningStatus::WaitingColdStart,
            LearningStatus::Relearning,
            LearningStatus::LongTermReview,
        ] {
            let deleted =
                ProblemLifecycleEngine::decide(status, ProblemLifecycleAction::DeletePersonalNote)
                    .expect("personal note deletion exits lifecycle");
            assert_eq!(deleted.next_status, LearningStatus::Unstarted);
            assert_eq!(deleted.review_cycle, ReviewCycleDirective::CancelActive);
        }
    }

    #[test]
    fn illegal_problem_lifecycle_transitions_are_explicit_errors() {
        for (status, action) in [
            (
                LearningStatus::Unstarted,
                ProblemLifecycleAction::StartLearning,
            ),
            (
                LearningStatus::UpsolvePending,
                ProblemLifecycleAction::MarkUnderstood,
            ),
            (
                LearningStatus::Learning,
                ProblemLifecycleAction::JoinUpsolve,
            ),
            (
                LearningStatus::WaitingColdStart,
                ProblemLifecycleAction::StartLearning,
            ),
            (
                LearningStatus::LongTermReview,
                ProblemLifecycleAction::StopLearning,
            ),
        ] {
            assert_eq!(
                ProblemLifecycleEngine::decide(status, action),
                Err(InvalidProblemLifecycleTransition { status, action })
            );
        }
    }

    #[test]
    fn available_actions_match_the_problem_header_contract() {
        assert_eq!(
            ProblemLifecycleEngine::available_actions(LearningStatus::Learning),
            &[
                ProblemLifecycleAction::MarkUnderstood,
                ProblemLifecycleAction::ReturnToPending,
                ProblemLifecycleAction::StopLearning,
            ]
        );
        assert!(
            ProblemLifecycleEngine::available_actions(LearningStatus::LongTermReview).is_empty()
        );
    }

    #[test]
    fn first_cold_start_due_is_three_local_calendar_days_later() {
        let end_of_month = LocalDate::parse_iso("2026-08-30").expect("valid local date");
        assert_eq!(
            ReviewSchedulingEngine::first_cold_start_due(end_of_month)
                .expect("first due")
                .to_iso_string(),
            "2026-09-02"
        );

        let leap_year = LocalDate::parse_iso("2028-02-27").expect("valid local date");
        assert_eq!(
            ReviewSchedulingEngine::first_cold_start_due(leap_year)
                .expect("first due")
                .to_iso_string(),
            "2028-03-01"
        );
        assert_eq!(LocalDate::parse_iso("2026-02-30"), Err(InvalidLocalDate));
    }

    #[test]
    fn review_eligibility_distinguishes_due_and_early_attempts() {
        let due = LocalDate::parse_iso("2026-08-14").expect("due");
        let before_due = LocalDate::parse_iso("2026-08-13").expect("before due");
        let on_due = LocalDate::parse_iso("2026-08-14").expect("on due");
        let overdue = LocalDate::parse_iso("2026-08-20").expect("overdue");

        let early =
            ReviewEligibilityEngine::decide(LearningStatus::WaitingColdStart, due, before_due)
                .expect("early check");
        assert_eq!(early.attempt_type, ReviewAttemptType::EarlyCheck);
        assert!(early.started_early);
        assert_eq!(early.scheduled_due_local_date, due);

        let first = ReviewEligibilityEngine::decide(LearningStatus::WaitingColdStart, due, on_due)
            .expect("first cold start");
        assert_eq!(first.attempt_type, ReviewAttemptType::FirstColdStart);
        assert!(!first.started_early);

        let long_term =
            ReviewEligibilityEngine::decide(LearningStatus::LongTermReview, due, overdue)
                .expect("long-term review");
        assert_eq!(long_term.attempt_type, ReviewAttemptType::LongTermReview);
        assert!(!long_term.started_early);
    }

    #[test]
    fn review_eligibility_rejects_non_review_lifecycle_states() {
        let date = LocalDate::parse_iso("2026-08-14").expect("date");
        for status in [
            LearningStatus::Unstarted,
            LearningStatus::UpsolvePending,
            LearningStatus::Learning,
            LearningStatus::Relearning,
        ] {
            assert_eq!(
                ReviewEligibilityEngine::decide(status, date, date),
                Err(ReviewNotEligible { status })
            );
        }
    }

    fn independent_ac_facts() -> ReviewCompletionFacts {
        ReviewCompletionFacts {
            final_ac: true,
            first_submission_result: SubmissionResult::WrongAnswer,
            final_result: SubmissionResult::Accepted,
            total_submissions: 2,
            idea_independent: true,
            implementation_independent: true,
            debug_independence: DebugIndependence::Independent,
            external_help: ExternalHelpLevel::None,
        }
    }

    #[test]
    fn judgement_is_derived_from_facts_and_immutable_help_evidence() {
        let facts = independent_ac_facts();
        assert_eq!(
            ReviewJudgementEngine::judge(&facts, None)
                .expect("independent AC")
                .judgement,
            ReviewJudgement::Mastered
        );
        assert_eq!(
            ReviewJudgementEngine::judge(&facts, Some(ReviewHelpLevel::Hints))
                .expect("hint-assisted AC")
                .judgement,
            ReviewJudgement::Partial
        );
        assert_eq!(
            ReviewJudgementEngine::judge(&facts, Some(ReviewHelpLevel::FullSolution))
                .expect("solution-assisted AC")
                .judgement,
            ReviewJudgement::Fail
        );
        let mut no_ac = facts;
        no_ac.final_ac = false;
        no_ac.final_result = SubmissionResult::WrongAnswer;
        assert_eq!(
            ReviewJudgementEngine::judge(&no_ac, None)
                .expect("no final AC")
                .judgement,
            ReviewJudgement::Fail
        );
    }

    #[test]
    fn judgement_rejects_contradictory_submission_facts() {
        let mut facts = independent_ac_facts();
        facts.total_submissions = 0;
        assert_eq!(
            ReviewJudgementEngine::judge(&facts, None),
            Err(ReviewFactsError::SubmissionCountMissing)
        );
        facts.total_submissions = 1;
        assert_eq!(
            ReviewJudgementEngine::judge(&facts, None),
            Err(ReviewFactsError::SingleSubmissionContradiction)
        );
        facts.first_submission_result = SubmissionResult::Accepted;
        facts.final_ac = false;
        assert_eq!(
            ReviewJudgementEngine::judge(&facts, None),
            Err(ReviewFactsError::FinalResultContradiction)
        );
    }

    #[test]
    fn review_completion_advances_due_keeps_early_mastery_and_suspends_failure() {
        let completed = LocalDate::parse_iso("2026-08-14").expect("date");
        assert_eq!(
            ReviewSchedulingEngine::complete_review(
                LearningStatus::WaitingColdStart,
                ReviewAttemptType::FirstColdStart,
                ReviewJudgement::Mastered,
                0,
                completed,
            )
            .expect("first pass"),
            ReviewCompletionDecision {
                next_learning_status: LearningStatus::LongTermReview,
                cycle: ReviewCycleCompletion::Advance {
                    next_stage: 1,
                    next_due: LocalDate::parse_iso("2026-08-24").expect("next due"),
                },
            }
        );
        assert_eq!(
            ReviewSchedulingEngine::complete_review(
                LearningStatus::LongTermReview,
                ReviewAttemptType::EarlyCheck,
                ReviewJudgement::Mastered,
                3,
                completed,
            )
            .expect("early pass")
            .cycle,
            ReviewCycleCompletion::Keep
        );
        assert_eq!(
            ReviewSchedulingEngine::complete_review(
                LearningStatus::LongTermReview,
                ReviewAttemptType::LongTermReview,
                ReviewJudgement::Partial,
                3,
                completed,
            )
            .expect("regression"),
            ReviewCompletionDecision {
                next_learning_status: LearningStatus::Relearning,
                cycle: ReviewCycleCompletion::Suspend,
            }
        );
    }

    #[test]
    fn long_term_schedule_uses_the_frozen_interval_sequence_and_caps_at_240_days() {
        let completed = LocalDate::parse_iso("2026-08-14").expect("date");
        let expected = [10_u64, 30, 75, 150, 240, 240, 240];
        for (stage, days) in expected.into_iter().enumerate() {
            let status = if stage == 0 {
                LearningStatus::WaitingColdStart
            } else {
                LearningStatus::LongTermReview
            };
            let attempt_type = if stage == 0 {
                ReviewAttemptType::FirstColdStart
            } else {
                ReviewAttemptType::LongTermReview
            };
            let decision = ReviewSchedulingEngine::complete_review(
                status,
                attempt_type,
                ReviewJudgement::Mastered,
                stage as u32,
                completed,
            )
            .expect("scheduled pass");
            let ReviewCycleCompletion::Advance { next_due, .. } = decision.cycle else {
                panic!("pass must advance");
            };
            assert_eq!(
                next_due,
                completed.checked_add_days(days).expect("expected due")
            );
        }
    }

    #[test]
    fn thoroughly_digested_requires_all_six_real_evidence_items() {
        let five = ProblemMasteryEvidence {
            recalls_problem: true,
            multiple_solutions_clear: true,
            knowledge_understood: true,
            implementation_fluent: true,
            can_adapt_or_create: true,
            transfer_solved_independently: false,
        };
        assert_eq!(five.achieved_count(), 5);
        assert!(!five.is_thoroughly_digested());
        assert!(ProblemMasteryEvidence {
            transfer_solved_independently: true,
            ..five
        }
        .is_thoroughly_digested());
    }

    fn today_input<'a>(
        problem_id: &'a str,
        learning_status: LearningStatus,
        learning_status_since: LocalDate,
    ) -> TodayCandidateInput<'a> {
        TodayCandidateInput {
            problem_id,
            learning_status,
            learning_status_since,
            pinned: false,
            active_review_due: None,
            in_progress_review: None,
        }
    }

    #[test]
    fn today_candidates_cover_only_the_frozen_legal_lanes() {
        let today = LocalDate::parse_iso("2026-08-12").expect("today");
        let earlier = LocalDate::parse_iso("2026-08-01").expect("earlier");
        let due = LocalDate::parse_iso("2026-08-10").expect("due");
        let future = LocalDate::parse_iso("2026-08-13").expect("future");

        let mut waiting = today_input("waiting", LearningStatus::WaitingColdStart, earlier);
        waiting.active_review_due = Some(due);
        let mut long_term = today_input("long-term", LearningStatus::LongTermReview, earlier);
        long_term.active_review_due = Some(today);
        let mut future_review = today_input("future", LearningStatus::LongTermReview, earlier);
        future_review.active_review_due = Some(future);

        let candidates = TodayCandidateBuilder::build(
            today,
            &[
                today_input("unstarted", LearningStatus::Unstarted, earlier),
                today_input("learning", LearningStatus::Learning, earlier),
                today_input("relearning", LearningStatus::Relearning, earlier),
                today_input("pending", LearningStatus::UpsolvePending, earlier),
                waiting,
                long_term,
                future_review,
            ],
        )
        .expect("legal candidates");

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| (
                    candidate.problem_id.as_str(),
                    candidate.lane,
                    candidate.reason,
                    candidate.planning_cost_minutes,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "learning",
                    TodayCandidateLane::CarryIn,
                    TodayCandidateReason::ContinueLearning,
                    60,
                ),
                (
                    "relearning",
                    TodayCandidateLane::Study,
                    TodayCandidateReason::Relearn,
                    60,
                ),
                (
                    "pending",
                    TodayCandidateLane::Study,
                    TodayCandidateReason::Upsolve,
                    60,
                ),
                (
                    "waiting",
                    TodayCandidateLane::Review,
                    TodayCandidateReason::DueFirstColdStart,
                    30,
                ),
                (
                    "long-term",
                    TodayCandidateLane::Review,
                    TodayCandidateReason::DueLongTermReview,
                    30,
                ),
            ]
        );
        assert_eq!(candidates[3].scheduled_due_local_date, Some(due));
        assert_eq!(candidates[4].scheduled_due_local_date, Some(today));
        assert!(candidates
            .iter()
            .all(|candidate| candidate.review_attempt_id.is_none()));
    }

    #[test]
    fn today_candidate_in_progress_review_is_one_authoritative_carry_in() {
        let today = LocalDate::parse_iso("2026-08-12").expect("today");
        let due = LocalDate::parse_iso("2026-08-10").expect("due");
        let mut input = today_input("problem-1", LearningStatus::WaitingColdStart, due);
        input.pinned = true;
        input.active_review_due = Some(due);
        input.in_progress_review = Some(TodayInProgressReview {
            attempt_id: "attempt-1",
            scheduled_due_local_date: due,
        });

        assert_eq!(
            TodayCandidateBuilder::build(today, &[input]).expect("carry-in"),
            vec![TodayCandidate {
                problem_id: "problem-1".to_owned(),
                review_attempt_id: Some("attempt-1".to_owned()),
                lane: TodayCandidateLane::CarryIn,
                reason: TodayCandidateReason::ContinueReview,
                planning_cost_minutes: 30,
                pinned: true,
                learning_status_since: due,
                scheduled_due_local_date: Some(due),
            }]
        );
    }

    #[test]
    fn today_candidate_builder_rejects_missing_or_mismatched_review_facts() {
        let today = LocalDate::parse_iso("2026-08-12").expect("today");
        let missing_due = today_input("missing", LearningStatus::WaitingColdStart, today);
        assert_eq!(
            TodayCandidateBuilder::build(today, &[missing_due]),
            Err(TodayCandidateError::MissingActiveReviewDue {
                problem_id: "missing".to_owned(),
                status: LearningStatus::WaitingColdStart,
            })
        );

        let mut mismatch = today_input("mismatch", LearningStatus::Learning, today);
        mismatch.in_progress_review = Some(TodayInProgressReview {
            attempt_id: "attempt-1",
            scheduled_due_local_date: today,
        });
        assert_eq!(
            TodayCandidateBuilder::build(today, &[mismatch]),
            Err(TodayCandidateError::InProgressReviewLifecycleMismatch {
                problem_id: "mismatch".to_owned(),
                status: LearningStatus::Learning,
            })
        );
    }

    #[test]
    fn today_candidate_builder_rejects_duplicate_problem_inputs() {
        let today = LocalDate::parse_iso("2026-08-12").expect("today");
        assert_eq!(
            TodayCandidateBuilder::build(
                today,
                &[
                    today_input("problem-1", LearningStatus::Learning, today),
                    today_input("problem-1", LearningStatus::UpsolvePending, today),
                ],
            ),
            Err(TodayCandidateError::DuplicateProblem {
                problem_id: "problem-1".to_owned(),
            })
        );
    }

    #[test]
    fn today_candidate_builder_rejects_empty_authoritative_ids() {
        let today = LocalDate::parse_iso("2026-08-12").expect("today");
        assert_eq!(
            TodayCandidateBuilder::build(
                today,
                &[today_input("", LearningStatus::Learning, today)],
            ),
            Err(TodayCandidateError::EmptyProblemId)
        );

        let mut input = today_input("problem-1", LearningStatus::LongTermReview, today);
        input.in_progress_review = Some(TodayInProgressReview {
            attempt_id: "",
            scheduled_due_local_date: today,
        });
        assert_eq!(
            TodayCandidateBuilder::build(today, &[input]),
            Err(TodayCandidateError::EmptyReviewAttemptId {
                problem_id: "problem-1".to_owned(),
            })
        );
    }

    #[test]
    fn today_candidate_semantics_do_not_depend_on_input_order() {
        let today = LocalDate::parse_iso("2026-08-12").expect("today");
        let earlier = LocalDate::parse_iso("2026-08-01").expect("earlier");
        let first = today_input("a", LearningStatus::Learning, earlier);
        let second = today_input("b", LearningStatus::Relearning, earlier);

        let mut forward = TodayCandidateBuilder::build(today, &[first, second]).expect("forward");
        let mut reverse = TodayCandidateBuilder::build(today, &[second, first]).expect("reverse");
        forward.sort_by(|left, right| left.problem_id.cmp(&right.problem_id));
        reverse.sort_by(|left, right| left.problem_id.cmp(&right.problem_id));
        assert_eq!(forward, reverse);
    }

    fn candidate(
        problem_id: &str,
        lane: TodayCandidateLane,
        reason: TodayCandidateReason,
        pinned: bool,
        since: &str,
        due: Option<&str>,
    ) -> TodayCandidate {
        TodayCandidate {
            problem_id: problem_id.to_owned(),
            review_attempt_id: None,
            lane,
            reason,
            planning_cost_minutes: match reason {
                TodayCandidateReason::ContinueReview
                | TodayCandidateReason::DueFirstColdStart
                | TodayCandidateReason::DueLongTermReview => 30,
                TodayCandidateReason::ContinueLearning
                | TodayCandidateReason::Relearn
                | TodayCandidateReason::Upsolve => 60,
            },
            pinned,
            learning_status_since: LocalDate::parse_iso(since).expect("since"),
            scheduled_due_local_date: due.map(|value| LocalDate::parse_iso(value).expect("due")),
        }
    }

    #[test]
    fn today_ordering_follows_the_frozen_lane_rules_and_stable_tie_breaks() {
        let candidates = vec![
            candidate(
                "review-later",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                true,
                "2026-08-01",
                Some("2026-08-10"),
            ),
            candidate(
                "review-overdue",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueLongTermReview,
                false,
                "2026-08-01",
                Some("2026-08-01"),
            ),
            candidate(
                "review-first",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                false,
                "2026-08-01",
                Some("2026-08-05"),
            ),
            candidate(
                "review-pinned-long",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueLongTermReview,
                true,
                "2026-08-01",
                Some("2026-08-05"),
            ),
            candidate(
                "review-pinned-first",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                true,
                "2026-08-01",
                Some("2026-08-05"),
            ),
            candidate(
                "study-pending-old",
                TodayCandidateLane::Study,
                TodayCandidateReason::Upsolve,
                true,
                "2026-07-01",
                None,
            ),
            candidate(
                "study-relearn-unpinned",
                TodayCandidateLane::Study,
                TodayCandidateReason::Relearn,
                false,
                "2026-07-01",
                None,
            ),
            candidate(
                "study-relearn-new",
                TodayCandidateLane::Study,
                TodayCandidateReason::Relearn,
                true,
                "2026-08-01",
                None,
            ),
            candidate(
                "study-relearn-old-b",
                TodayCandidateLane::Study,
                TodayCandidateReason::Relearn,
                true,
                "2026-07-01",
                None,
            ),
            candidate(
                "study-relearn-old-a",
                TodayCandidateLane::Study,
                TodayCandidateReason::Relearn,
                true,
                "2026-07-01",
                None,
            ),
            candidate(
                "carry-z",
                TodayCandidateLane::CarryIn,
                TodayCandidateReason::ContinueLearning,
                false,
                "2026-08-01",
                None,
            ),
            candidate(
                "carry-a",
                TodayCandidateLane::CarryIn,
                TodayCandidateReason::ContinueReview,
                false,
                "2026-08-01",
                Some("2026-08-01"),
            ),
        ];

        let ordered = TodayCandidateOrderingEngine::order(&candidates).expect("valid candidates");
        assert_eq!(
            ordered
                .carry_in
                .iter()
                .map(|item| item.problem_id.as_str())
                .collect::<Vec<_>>(),
            vec!["carry-a", "carry-z"]
        );
        assert_eq!(
            ordered
                .review
                .iter()
                .map(|item| item.problem_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "review-overdue",
                "review-pinned-first",
                "review-pinned-long",
                "review-first",
                "review-later",
            ]
        );
        assert_eq!(
            ordered
                .study
                .iter()
                .map(|item| item.problem_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "study-relearn-old-a",
                "study-relearn-old-b",
                "study-relearn-new",
                "study-relearn-unpinned",
                "study-pending-old",
            ]
        );
    }

    #[test]
    fn today_ordering_is_deterministic_for_every_input_permutation() {
        let a = candidate(
            "a",
            TodayCandidateLane::Review,
            TodayCandidateReason::DueFirstColdStart,
            false,
            "2026-08-01",
            Some("2026-08-01"),
        );
        let b = candidate(
            "b",
            TodayCandidateLane::Study,
            TodayCandidateReason::Relearn,
            false,
            "2026-08-01",
            None,
        );
        let c = candidate(
            "c",
            TodayCandidateLane::CarryIn,
            TodayCandidateReason::ContinueLearning,
            false,
            "2026-08-01",
            None,
        );
        let expected = TodayCandidateOrderingEngine::order(&[a.clone(), b.clone(), c.clone()])
            .expect("expected order");
        for permutation in [
            vec![a.clone(), c.clone(), b.clone()],
            vec![b.clone(), a.clone(), c.clone()],
            vec![b.clone(), c.clone(), a.clone()],
            vec![c.clone(), a.clone(), b.clone()],
            vec![c, b, a],
        ] {
            assert_eq!(
                TodayCandidateOrderingEngine::order(&permutation).expect("permutation order"),
                expected
            );
        }
    }

    #[test]
    fn today_ordering_rejects_malformed_candidate_contracts() {
        let mismatched = candidate(
            "mismatched",
            TodayCandidateLane::Study,
            TodayCandidateReason::DueFirstColdStart,
            false,
            "2026-08-01",
            Some("2026-08-01"),
        );
        assert_eq!(
            TodayCandidateOrderingEngine::order(&[mismatched]),
            Err(TodayCandidateOrderingError::LaneReasonMismatch {
                problem_id: "mismatched".to_owned(),
            })
        );

        let missing_due = candidate(
            "missing-due",
            TodayCandidateLane::Review,
            TodayCandidateReason::DueLongTermReview,
            false,
            "2026-08-01",
            None,
        );
        assert_eq!(
            TodayCandidateOrderingEngine::order(&[missing_due]),
            Err(TodayCandidateOrderingError::MissingReviewDue {
                problem_id: "missing-due".to_owned(),
            })
        );
    }

    #[test]
    fn today_anti_starvation_requires_both_lanes_when_both_fit() {
        assert_eq!(
            TodayAntiStarvationEngine::decide(
                true,
                true,
                TodayPlanningCapacity {
                    can_fit_review: true,
                    can_fit_study: true,
                    can_fit_both: true,
                },
                1,
            ),
            Ok(TodayAntiStarvationDecision {
                required_lanes: TodayLaneRequirement::Both,
                next_review_only_streak: 0,
            })
        );
    }

    #[test]
    fn today_anti_starvation_allows_two_review_only_days_then_requires_study() {
        let review_only_capacity = TodayPlanningCapacity {
            can_fit_review: true,
            can_fit_study: true,
            can_fit_both: false,
        };
        for (streak, expected_next) in [(0, 1), (1, 2)] {
            assert_eq!(
                TodayAntiStarvationEngine::decide(true, true, review_only_capacity, streak,),
                Ok(TodayAntiStarvationDecision {
                    required_lanes: TodayLaneRequirement::Review,
                    next_review_only_streak: expected_next,
                })
            );
        }
        assert_eq!(
            TodayAntiStarvationEngine::decide(true, true, review_only_capacity, 2),
            Ok(TodayAntiStarvationDecision {
                required_lanes: TodayLaneRequirement::Study,
                next_review_only_streak: 0,
            })
        );
    }

    #[test]
    fn today_anti_starvation_does_not_accrue_debt_without_study_backlog() {
        assert_eq!(
            TodayAntiStarvationEngine::decide(
                true,
                false,
                TodayPlanningCapacity {
                    can_fit_review: true,
                    can_fit_study: false,
                    can_fit_both: false,
                },
                2,
            ),
            Ok(TodayAntiStarvationDecision {
                required_lanes: TodayLaneRequirement::Review,
                next_review_only_streak: 0,
            })
        );
    }

    #[test]
    fn today_anti_starvation_rejects_impossible_or_corrupt_inputs() {
        assert_eq!(
            TodayAntiStarvationEngine::decide(
                true,
                true,
                TodayPlanningCapacity {
                    can_fit_review: true,
                    can_fit_study: false,
                    can_fit_both: true,
                },
                0,
            ),
            Err(TodayAntiStarvationError::InvalidCapacity)
        );
        assert_eq!(
            TodayAntiStarvationEngine::decide(
                true,
                true,
                TodayPlanningCapacity {
                    can_fit_review: true,
                    can_fit_study: true,
                    can_fit_both: false,
                },
                3,
            ),
            Err(TodayAntiStarvationError::InvalidReviewOnlyStreak)
        );
        assert_eq!(
            TodayAntiStarvationEngine::decide(
                true,
                true,
                TodayPlanningCapacity {
                    can_fit_review: true,
                    can_fit_study: false,
                    can_fit_both: false,
                },
                2,
            ),
            Err(TodayAntiStarvationError::StudyRequiredButCannotFit)
        );
    }

    fn ordered_candidates(
        carry_in: Vec<TodayCandidate>,
        review: Vec<TodayCandidate>,
        study: Vec<TodayCandidate>,
    ) -> TodayOrderedCandidates {
        TodayOrderedCandidates {
            carry_in,
            review,
            study,
        }
    }

    #[test]
    fn today_planner_keeps_all_carry_in_before_new_recommendations() {
        let carry_review = candidate(
            "carry-review",
            TodayCandidateLane::CarryIn,
            TodayCandidateReason::ContinueReview,
            false,
            "2026-08-01",
            Some("2026-08-01"),
        );
        let carry_learning = candidate(
            "carry-learning",
            TodayCandidateLane::CarryIn,
            TodayCandidateReason::ContinueLearning,
            false,
            "2026-08-01",
            None,
        );
        let review = candidate(
            "new-review",
            TodayCandidateLane::Review,
            TodayCandidateReason::DueFirstColdStart,
            false,
            "2026-08-01",
            Some("2026-08-01"),
        );
        let ordered = ordered_candidates(vec![carry_review, carry_learning], vec![review], vec![]);

        let draft = TodayPlanner::plan(&ordered, 60, TodayLaneRequirement::None)
            .expect("carry-in remains visible");
        assert_eq!(
            draft
                .entries
                .iter()
                .map(|entry| entry.problem_id.as_str())
                .collect::<Vec<_>>(),
            vec!["carry-review", "carry-learning"]
        );
        assert_eq!(draft.planned_minutes, 90);
        assert_eq!(draft.over_budget_minutes, 30);
        assert_eq!(draft.unplanned_review_count, 1);
    }

    #[test]
    fn today_planner_satisfies_required_lanes_before_review_first_fill() {
        let reviews = ["review-a", "review-b"]
            .map(|problem_id| {
                candidate(
                    problem_id,
                    TodayCandidateLane::Review,
                    TodayCandidateReason::DueLongTermReview,
                    false,
                    "2026-08-01",
                    Some("2026-08-01"),
                )
            })
            .to_vec();
        let studies = ["study-a", "study-b"]
            .map(|problem_id| {
                candidate(
                    problem_id,
                    TodayCandidateLane::Study,
                    TodayCandidateReason::Relearn,
                    false,
                    "2026-08-01",
                    None,
                )
            })
            .to_vec();
        let ordered = ordered_candidates(vec![], reviews, studies);

        let draft = TodayPlanner::plan(&ordered, 120, TodayLaneRequirement::Both)
            .expect("both required lanes fit");
        assert_eq!(
            draft
                .entries
                .iter()
                .map(|entry| entry.problem_id.as_str())
                .collect::<Vec<_>>(),
            vec!["review-a", "study-a", "review-b"]
        );
        assert_eq!(draft.planned_minutes, 120);
        assert_eq!(draft.over_budget_minutes, 0);
        assert_eq!(draft.unplanned_review_count, 0);
        assert_eq!(draft.unplanned_study_count, 1);

        let study_first = TodayPlanner::plan(&ordered, 90, TodayLaneRequirement::Study)
            .expect("study requirement fits");
        assert_eq!(
            study_first
                .entries
                .iter()
                .map(|entry| entry.problem_id.as_str())
                .collect::<Vec<_>>(),
            vec!["study-a", "review-a"]
        );
    }

    #[test]
    fn today_generated_planner_consumes_budget_and_anti_starvation_together() {
        let ordered = ordered_candidates(
            vec![],
            vec![candidate(
                "review",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                false,
                "2026-08-01",
                Some("2026-08-01"),
            )],
            vec![candidate(
                "study",
                TodayCandidateLane::Study,
                TodayCandidateReason::Relearn,
                false,
                "2026-08-01",
                None,
            )],
        );

        let both = TodayPlanner::plan_generated(&ordered, 90, 1).expect("both fit");
        assert_eq!(
            both.entries
                .iter()
                .map(|entry| entry.problem_id.as_str())
                .collect::<Vec<_>>(),
            vec!["review", "study"]
        );
        assert_eq!(both.next_review_only_streak, 0);

        let first_review_only = TodayPlanner::plan_generated(&ordered, 60, 0)
            .expect("first review-only day is allowed");
        assert_eq!(first_review_only.entries[0].problem_id, "review");
        assert_eq!(first_review_only.next_review_only_streak, 1);

        let forced_study = TodayPlanner::plan_generated(&ordered, 60, 2)
            .expect("third constrained day gives Study the slot");
        assert_eq!(forced_study.entries[0].problem_id, "study");
        assert_eq!(forced_study.next_review_only_streak, 0);
    }

    #[test]
    fn today_generated_planner_stops_new_work_when_carry_in_consumes_budget() {
        let ordered = ordered_candidates(
            vec![candidate(
                "carry",
                TodayCandidateLane::CarryIn,
                TodayCandidateReason::ContinueLearning,
                false,
                "2026-08-01",
                None,
            )],
            vec![candidate(
                "review",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                false,
                "2026-08-01",
                Some("2026-08-01"),
            )],
            vec![candidate(
                "study",
                TodayCandidateLane::Study,
                TodayCandidateReason::Relearn,
                false,
                "2026-08-01",
                None,
            )],
        );
        let draft = TodayPlanner::plan_generated(&ordered, 30, 0)
            .expect("real carry-in remains even over budget");
        assert_eq!(draft.entries.len(), 1);
        assert_eq!(draft.entries[0].problem_id, "carry");
        assert_eq!(draft.over_budget_minutes, 30);
        assert_eq!(draft.unplanned_review_count, 1);
        assert_eq!(draft.unplanned_study_count, 1);
        assert_eq!(draft.next_review_only_streak, 0);
    }

    #[test]
    fn today_planner_only_packs_complete_tasks() {
        let ordered = ordered_candidates(
            vec![],
            vec![candidate(
                "review",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                false,
                "2026-08-01",
                Some("2026-08-01"),
            )],
            vec![candidate(
                "study",
                TodayCandidateLane::Study,
                TodayCandidateReason::Upsolve,
                false,
                "2026-08-01",
                None,
            )],
        );

        let draft = TodayPlanner::plan(&ordered, 89, TodayLaneRequirement::None)
            .expect("review fits but study does not");
        assert_eq!(draft.entries.len(), 1);
        assert_eq!(draft.entries[0].problem_id, "review");
        assert_eq!(draft.planned_minutes, 30);
        assert_eq!(draft.unplanned_study_count, 1);

        let no_room = TodayPlanner::plan(&ordered, 29, TodayLaneRequirement::None)
            .expect("no complete task fits");
        assert!(no_room.entries.is_empty());
        assert_eq!(no_room.planned_minutes, 0);
        assert_eq!(no_room.unplanned_review_count, 1);
        assert_eq!(no_room.unplanned_study_count, 1);
    }

    #[test]
    fn today_planner_rejects_unsatisfied_lane_requirements() {
        let review_only = ordered_candidates(
            vec![],
            vec![candidate(
                "review",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                false,
                "2026-08-01",
                Some("2026-08-01"),
            )],
            vec![],
        );
        assert_eq!(
            TodayPlanner::plan(&review_only, 90, TodayLaneRequirement::Study),
            Err(TodayPlannerError::RequiredStudyUnavailable)
        );
        assert_eq!(
            TodayPlanner::plan(&review_only, 29, TodayLaneRequirement::Review),
            Err(TodayPlannerError::RequiredLanesDoNotFit)
        );
        assert_eq!(
            TodayPlanner::plan(&review_only, 90, TodayLaneRequirement::Both),
            Err(TodayPlannerError::RequiredStudyUnavailable)
        );
    }

    #[test]
    fn today_planner_rejects_corrupt_ordered_candidates() {
        let valid = candidate(
            "duplicate",
            TodayCandidateLane::Review,
            TodayCandidateReason::DueFirstColdStart,
            false,
            "2026-08-01",
            Some("2026-08-01"),
        );
        let duplicate = TodayCandidate {
            lane: TodayCandidateLane::Study,
            reason: TodayCandidateReason::Upsolve,
            planning_cost_minutes: 60,
            scheduled_due_local_date: None,
            ..valid.clone()
        };
        assert_eq!(
            TodayPlanner::plan(
                &ordered_candidates(vec![], vec![valid], vec![duplicate]),
                90,
                TodayLaneRequirement::None,
            ),
            Err(TodayPlannerError::DuplicateProblem {
                problem_id: "duplicate".to_owned(),
            })
        );

        let invalid_cost = TodayCandidate {
            planning_cost_minutes: 29,
            ..candidate(
                "invalid-cost",
                TodayCandidateLane::Review,
                TodayCandidateReason::DueFirstColdStart,
                false,
                "2026-08-01",
                Some("2026-08-01"),
            )
        };
        assert_eq!(
            TodayPlanner::plan(
                &ordered_candidates(vec![], vec![invalid_cost], vec![]),
                90,
                TodayLaneRequirement::None,
            ),
            Err(TodayPlannerError::InvalidPlanningCost {
                problem_id: "invalid-cost".to_owned(),
            })
        );

        let wrong_lane = candidate(
            "wrong-lane",
            TodayCandidateLane::Study,
            TodayCandidateReason::Upsolve,
            false,
            "2026-08-01",
            None,
        );
        assert_eq!(
            TodayPlanner::plan(
                &ordered_candidates(vec![], vec![wrong_lane], vec![]),
                90,
                TodayLaneRequirement::None,
            ),
            Err(TodayPlannerError::LaneMismatch {
                problem_id: "wrong-lane".to_owned(),
            })
        );
    }
}
