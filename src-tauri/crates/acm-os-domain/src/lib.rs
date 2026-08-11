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
}
