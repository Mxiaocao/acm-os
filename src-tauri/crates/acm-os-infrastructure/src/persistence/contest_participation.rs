use super::{normalize_schema_sql, validate_table_columns, DatabaseRuntime, StartupRecoveryReason};
use acm_os_application::contest_participation::{
    ContestParticipation, ContestParticipationPort, ParticipationError,
};
use acm_os_domain::{
    contest_participation::ParticipationFacts, CodeforcesContestIdentity, LocalDate,
};
use sqlx::SqlitePool;

type Row = (
    String,
    Option<String>,
    String,
    String,
    String,
    String,
    Option<u32>,
    Option<u32>,
);
// Resolve a single display identity without joining aliases into extra rows.
// The recorded local start date remains stable when the machine timezone changes.
const SELECT: &str = "SELECT cp.id, (SELECT MIN(external_contest_key) FROM contest_external_identities WHERE contest_id = c.id AND platform = 'codeforces'), COALESCE(c.title, cp.contest_title), cp.kind, cp.local_date, cp.local_time, cp.accepted_count, cp.total_count FROM contest_participations cp LEFT JOIN contests c ON c.id = cp.contest_id WHERE (?1 IS NULL OR cp.contest_id = (SELECT contest_id FROM contest_external_identities WHERE platform = 'codeforces' AND external_contest_key = ?1)) AND cp.local_date >= ?2 AND cp.local_date < ?3 ORDER BY cp.local_date, cp.local_time, cp.id";
fn record(row: Row) -> ContestParticipation {
    let (id, contest_key, contest_title, kind, local_date, local_time, accepted_count, total_count) =
        row;
    ContestParticipation {
        id,
        contest_key,
        contest_title,
        facts: ParticipationFacts {
            kind,
            local_date,
            local_time,
            accepted_count,
            total_count,
        },
    }
}
fn error(error: sqlx::Error) -> ParticipationError {
    if error
        .as_database_error()
        .is_some_and(|e| e.is_unique_violation())
    {
        ParticipationError::Duplicate
    } else {
        ParticipationError::Unavailable
    }
}

impl ContestParticipationPort for DatabaseRuntime {
    async fn list_contest_participations(
        &self,
        contest: &CodeforcesContestIdentity,
    ) -> Result<Vec<ContestParticipation>, ParticipationError> {
        let pool = self._pool.as_ref().ok_or(ParticipationError::Unavailable)?;
        sqlx::query_as::<_, Row>(SELECT)
            .bind(contest.contest_id().to_string())
            .bind("0001-01-01")
            .bind("9999-01-01")
            .fetch_all(pool)
            .await
            .map(|rows| rows.into_iter().map(record).collect())
            .map_err(error)
    }
    async fn save_contest_participation(
        &self,
        contest: &CodeforcesContestIdentity,
        id: &str,
        facts: &ParticipationFacts,
    ) -> Result<(), ParticipationError> {
        facts.validate().map_err(ParticipationError::Invalid)?;
        uuid::Uuid::parse_str(id)
            .map_err(|_| ParticipationError::Invalid("invalid_participation_id"))?;
        let pool = self._pool.as_ref().ok_or(ParticipationError::Unavailable)?;
        self.ensure_daily_backup(
            crate::current_local_date().map_err(|_| ParticipationError::Unavailable)?,
        )
        .await
        .map_err(|_| ParticipationError::Unavailable)?;
        let mut tx = pool.begin().await.map_err(error)?;
        let (contest_id, title): (i64, String) = sqlx::query_as("SELECT c.id, c.title FROM contests c JOIN contest_external_identities ci ON ci.contest_id = c.id WHERE ci.platform = 'codeforces' AND ci.external_contest_key = ?1")
            .bind(contest.contest_id().to_string()).fetch_optional(&mut *tx).await.map_err(error)?.ok_or(ParticipationError::NotFound)?;
        let result = sqlx::query("INSERT INTO contest_participations (id, contest_id, contest_title, kind, local_date, local_time, accepted_count, total_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) ON CONFLICT(id) DO UPDATE SET kind = excluded.kind, local_date = excluded.local_date, local_time = excluded.local_time, accepted_count = excluded.accepted_count, total_count = excluded.total_count WHERE contest_participations.contest_id = excluded.contest_id")
            .bind(id).bind(contest_id).bind(title).bind(&facts.kind).bind(&facts.local_date).bind(&facts.local_time).bind(facts.accepted_count).bind(facts.total_count)
            .execute(&mut *tx).await.map_err(error)?;
        if result.rows_affected() != 1 {
            return Err(ParticipationError::NotFound);
        }
        tx.commit().await.map_err(error)
    }
    async fn delete_contest_participation(
        &self,
        contest: &CodeforcesContestIdentity,
        id: &str,
    ) -> Result<(), ParticipationError> {
        let pool = self._pool.as_ref().ok_or(ParticipationError::Unavailable)?;
        self.ensure_daily_backup(
            crate::current_local_date().map_err(|_| ParticipationError::Unavailable)?,
        )
        .await
        .map_err(|_| ParticipationError::Unavailable)?;
        sqlx::query("DELETE FROM contest_participations WHERE id = ?1 AND contest_id = (SELECT contest_id FROM contest_external_identities WHERE platform = 'codeforces' AND external_contest_key = ?2)")
            .bind(id).bind(contest.contest_id().to_string()).execute(pool).await.map_err(error)?;
        Ok(())
    }
    async fn calendar_contest_participations(
        &self,
        start: LocalDate,
        end: LocalDate,
    ) -> Result<Vec<ContestParticipation>, ParticipationError> {
        let pool = self._pool.as_ref().ok_or(ParticipationError::Unavailable)?;
        sqlx::query_as::<_, Row>(SELECT)
            .bind(Option::<String>::None)
            .bind(start.to_iso_string())
            .bind(end.to_iso_string())
            .fetch_all(pool)
            .await
            .map(|rows| rows.into_iter().map(record).collect())
            .map_err(error)
    }
}

pub(super) async fn validate_contract(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "contest_participations",
        &[
            "id",
            "contest_id",
            "contest_title",
            "kind",
            "local_date",
            "local_time",
            "accepted_count",
            "total_count",
            "created_at_utc",
        ],
    )
    .await?;
    let actual: String = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'contest_participations'",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    let expected = include_str!("../../migrations/0037_create_contest_participations.sql")
        .split(';')
        .next()
        .unwrap();
    if normalize_schema_sql(&actual) != normalize_schema_sql(expected) {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }
    Ok(())
}
