#![forbid(unsafe_code)]

pub const BOUNDARY_NAME: &str = "acm-os-domain";

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
    pub fn new(contest: CodeforcesContestIdentity, index: impl Into<String>) -> Result<Self, IdentityError> {
        let index = index.into();
        let valid = !index.is_empty()
            && index.len() <= 8
            && index.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
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
                Unstarted
                | UpsolvePending
                | Learning
                | WaitingColdStart
                | Relearning
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

    fn checked_add_days(self, days: u64) -> Result<Self, InvalidLocalDate> {
        self.0
            .checked_add_days(chrono::Days::new(days))
            .map(Self)
            .ok_or(InvalidLocalDate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidLocalDate;

pub struct ReviewSchedulingEngine;

impl ReviewSchedulingEngine {
    pub const SCHEDULE_RULE_VERSION: u32 = 1;
    pub const FIRST_COLD_START_DAYS: u64 = 3;

    pub fn first_cold_start_due(marked_understood_on: LocalDate) -> Result<LocalDate, InvalidLocalDate> {
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
        if facts.total_submissions == 1
            && facts.first_submission_result != facts.final_result
        {
            return Err(ReviewFactsError::SingleSubmissionContradiction);
        }
        let full_solution_used = highest_help_level == Some(ReviewHelpLevel::FullSolution)
            || facts.external_help == ExternalHelpLevel::FullSolution;
        let solving_help_used = highest_help_level.is_some()
            || facts.external_help != ExternalHelpLevel::None
            || facts.debug_independence == DebugIndependence::UsedSolvingHelp;
        let judgement = if !facts.final_ac || full_solution_used {
            ReviewJudgement::Fail
        } else if solving_help_used
            || !facts.idea_independent
            || !facts.implementation_independent
        {
            ReviewJudgement::Partial
        } else {
            ReviewJudgement::Mastered
        };
        let mut evidence_codes = vec![if facts.final_ac { "final_ac" } else { "no_final_ac" }];
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
    Advance { next_stage: u32, next_due: LocalDate },
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
        if !matches!(status, LearningStatus::WaitingColdStart | LearningStatus::LongTermReview) {
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
            let stopped = ProblemLifecycleEngine::decide(
                status,
                ProblemLifecycleAction::StopLearning,
            )
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
            let deleted = ProblemLifecycleEngine::decide(
                status,
                ProblemLifecycleAction::DeletePersonalNote,
            )
            .expect("personal note deletion exits lifecycle");
            assert_eq!(deleted.next_status, LearningStatus::Unstarted);
            assert_eq!(deleted.review_cycle, ReviewCycleDirective::CancelActive);
        }
    }

    #[test]
    fn illegal_problem_lifecycle_transitions_are_explicit_errors() {
        for (status, action) in [
            (LearningStatus::Unstarted, ProblemLifecycleAction::StartLearning),
            (LearningStatus::UpsolvePending, ProblemLifecycleAction::MarkUnderstood),
            (LearningStatus::Learning, ProblemLifecycleAction::JoinUpsolve),
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
        assert!(ProblemLifecycleEngine::available_actions(LearningStatus::LongTermReview).is_empty());
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

        let early = ReviewEligibilityEngine::decide(
            LearningStatus::WaitingColdStart,
            due,
            before_due,
        )
        .expect("early check");
        assert_eq!(early.attempt_type, ReviewAttemptType::EarlyCheck);
        assert!(early.started_early);
        assert_eq!(early.scheduled_due_local_date, due);

        let first = ReviewEligibilityEngine::decide(
            LearningStatus::WaitingColdStart,
            due,
            on_due,
        )
        .expect("first cold start");
        assert_eq!(first.attempt_type, ReviewAttemptType::FirstColdStart);
        assert!(!first.started_early);

        let long_term = ReviewEligibilityEngine::decide(
            LearningStatus::LongTermReview,
            due,
            overdue,
        )
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
}
