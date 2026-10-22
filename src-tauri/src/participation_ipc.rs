use acm_os_application::contest_participation::{
    save_participation, ContestParticipation, ContestParticipationPort, ParticipationError,
};
use acm_os_domain::{contest_participation::ParticipationFacts, CodeforcesContestIdentity};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipationDto {
    id: String,
    contest_id: Option<String>,
    contest_title: String,
    kind: String,
    local_date: String,
    local_time: String,
    accepted_count: Option<u32>,
    total_count: Option<u32>,
}
pub fn dto(record: ContestParticipation) -> ParticipationDto {
    ParticipationDto {
        id: record.id,
        contest_id: record.contest_key,
        contest_title: record.contest_title,
        kind: record.facts.kind,
        local_date: record.facts.local_date,
        local_time: record.facts.local_time,
        accepted_count: record.facts.accepted_count,
        total_count: record.facts.total_count,
    }
}
pub fn error_code(error: ParticipationError) -> &'static str {
    match error {
        ParticipationError::Invalid(code) => code,
        ParticipationError::NotFound => "participation_not_found",
        ParticipationError::Duplicate => "participation_duplicate_start",
        ParticipationError::Unavailable => "participation_unavailable",
    }
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContestInput {
    contest_id: u64,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveInput {
    contest_id: u64,
    id: String,
    kind: String,
    local_date: String,
    local_time: String,
    accepted_count: Option<u32>,
    total_count: Option<u32>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteInput {
    contest_id: u64,
    id: String,
}

#[tauri::command]
pub async fn contest_participations(
    manager: tauri::State<'_, crate::vault_ipc::DesktopVaultManager>,
    vault_context: acm_os_infrastructure::vault_manager::VaultContext,
    input: ContestInput,
) -> Result<Vec<ParticipationDto>, &'static str> {
    let manager = manager.inner().as_ref().map_err(|e| e.code())?;
    let database = manager
        .database(&vault_context)
        .await
        .map_err(|e| e.code())?;
    let contest =
        CodeforcesContestIdentity::new(input.contest_id).map_err(|_| "invalid_contest_identity")?;
    database
        .inner()
        .list_contest_participations(&contest)
        .await
        .map(|records| records.into_iter().map(dto).collect())
        .map_err(error_code)
}
#[tauri::command]
pub async fn save_contest_participation(
    manager: tauri::State<'_, crate::vault_ipc::DesktopVaultManager>,
    vault_context: acm_os_infrastructure::vault_manager::VaultContext,
    input: SaveInput,
) -> Result<(), &'static str> {
    let manager = manager.inner().as_ref().map_err(|e| e.code())?;
    let database = manager
        .database(&vault_context)
        .await
        .map_err(|e| e.code())?;
    let contest =
        CodeforcesContestIdentity::new(input.contest_id).map_err(|_| "invalid_contest_identity")?;
    let facts = ParticipationFacts {
        kind: input.kind,
        local_date: input.local_date,
        local_time: input.local_time,
        accepted_count: input.accepted_count,
        total_count: input.total_count,
    };
    save_participation(database.inner(), &contest, &input.id, &facts)
        .await
        .map_err(error_code)
}
#[tauri::command]
pub async fn delete_contest_participation(
    manager: tauri::State<'_, crate::vault_ipc::DesktopVaultManager>,
    vault_context: acm_os_infrastructure::vault_manager::VaultContext,
    input: DeleteInput,
) -> Result<(), &'static str> {
    let manager = manager.inner().as_ref().map_err(|e| e.code())?;
    let database = manager
        .database(&vault_context)
        .await
        .map_err(|e| e.code())?;
    let contest =
        CodeforcesContestIdentity::new(input.contest_id).map_err(|_| "invalid_contest_identity")?;
    database
        .inner()
        .delete_contest_participation(&contest, &input.id)
        .await
        .map_err(error_code)
}
