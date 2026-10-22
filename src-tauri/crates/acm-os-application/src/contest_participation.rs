use acm_os_domain::{
    contest_participation::ParticipationFacts, CodeforcesContestIdentity, LocalDate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestParticipation {
    pub id: String,
    pub contest_key: Option<String>,
    pub contest_title: String,
    pub facts: ParticipationFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipationError {
    Invalid(&'static str),
    NotFound,
    Duplicate,
    Unavailable,
}

#[allow(async_fn_in_trait)]
pub trait ContestParticipationPort {
    async fn list_contest_participations(
        &self,
        contest: &CodeforcesContestIdentity,
    ) -> Result<Vec<ContestParticipation>, ParticipationError>;
    async fn save_contest_participation(
        &self,
        contest: &CodeforcesContestIdentity,
        id: &str,
        facts: &ParticipationFacts,
    ) -> Result<(), ParticipationError>;
    async fn delete_contest_participation(
        &self,
        contest: &CodeforcesContestIdentity,
        id: &str,
    ) -> Result<(), ParticipationError>;
    async fn calendar_contest_participations(
        &self,
        start: LocalDate,
        end: LocalDate,
    ) -> Result<Vec<ContestParticipation>, ParticipationError>;
}

pub async fn save_participation(
    port: &impl ContestParticipationPort,
    contest: &CodeforcesContestIdentity,
    id: &str,
    facts: &ParticipationFacts,
) -> Result<(), ParticipationError> {
    facts.validate().map_err(ParticipationError::Invalid)?;
    port.save_contest_participation(contest, id, facts).await
}
