use super::DatabaseRuntime;
use acm_os_application::single_problem::{IndependentProblemItem, SingleProblemDraft, SingleProblemError, SingleProblemPort};

impl SingleProblemPort for DatabaseRuntime {
    async fn import_single_problem(&self, draft: &SingleProblemDraft) -> Result<i64, SingleProblemError> {
        let pool = self._pool.as_ref().ok_or(SingleProblemError::Unavailable)?;
        let today = crate::current_local_date().map_err(|_| SingleProblemError::Unavailable)?;
        self.ensure_daily_backup(today).await.map_err(|_| SingleProblemError::Unavailable)?;
        let mut tx = pool.begin().await.map_err(|_| SingleProblemError::Unavailable)?;
        let existing: Option<i64> = sqlx::query_scalar("SELECT problem_id FROM problem_external_identities WHERE platform = ?1 AND external_contest_key = ?2 AND external_problem_key = ?3")
            .bind(&draft.platform).bind(&draft.scope).bind(&draft.key).fetch_optional(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
        let id = if let Some(id) = existing { id } else {
            let result = sqlx::query("INSERT INTO problems (title, source_url, rating) VALUES (?1, ?2, ?3)")
                .bind(&draft.title).bind(&draft.source_url).bind(draft.rating.map(i64::from)).execute(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
            let id = result.last_insert_rowid();
            sqlx::query("INSERT INTO problem_external_identities (problem_id, platform, external_contest_key, external_problem_key) VALUES (?1, ?2, ?3, ?4)")
                .bind(id).bind(&draft.platform).bind(&draft.scope).bind(&draft.key).execute(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
            sqlx::query("INSERT INTO problem_learning_states (problem_id) VALUES (?1)").bind(id).execute(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
            id
        };
        sqlx::query("UPDATE problems SET deleted_at_utc = NULL WHERE id = ?1").bind(id).execute(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
        sqlx::query("INSERT OR IGNORE INTO problem_library_items (problem_id, origin) VALUES (?1, 'explicit')").bind(id).execute(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
        let captured = sqlx::query("INSERT OR IGNORE INTO problem_statement_snapshots (problem_id, source_html, sanitized_html) VALUES (?1, ?2, ?3)")
            .bind(id).bind(&draft.source_html).bind(&draft.sanitized_html).execute(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
        if captured.rows_affected() > 0 {
            for asset in &draft.assets {
                sqlx::query("INSERT INTO problem_statement_assets (problem_id, local_ref, media_type, bytes) VALUES (?1, ?2, ?3, ?4)")
                    .bind(id).bind(&asset.local_ref).bind(&asset.media_type).bind(&asset.bytes).execute(&mut *tx).await.map_err(|_| SingleProblemError::Unavailable)?;
            }
        }
        // No contest, placement or contest_problems row is created here.
        tx.commit().await.map_err(|_| SingleProblemError::Unavailable)?;
        Ok(id)
    }

    async fn independent_problems(&self) -> Result<Vec<IndependentProblemItem>, SingleProblemError> {
        let pool = self._pool.as_ref().ok_or(SingleProblemError::Unavailable)?;
        let rows: Vec<(i64, String, String, Option<i64>, bool, String, Option<String>)> = sqlx::query_as(
            "SELECT p.id, p.title, p.source_url, p.rating, EXISTS(SELECT 1 FROM problem_statement_snapshots s WHERE s.problem_id = p.id), p.created_at_utc, ss.sanitized_html \
             FROM problems p JOIN problem_library_items li ON li.problem_id = p.id LEFT JOIN problem_statement_snapshots ss ON ss.problem_id = p.id \
             WHERE p.deleted_at_utc IS NULL AND NOT EXISTS(SELECT 1 FROM problem_external_identities pi WHERE pi.problem_id = p.id AND pi.platform = 'codeforces') \
             ORDER BY li.added_at_utc DESC, p.id DESC")
            .fetch_all(pool).await.map_err(|_| SingleProblemError::Unavailable)?;
        rows.into_iter().map(|(problem_id, title, source_url, rating, has_statement_snapshot, imported_at_utc, statement_html)| Ok(IndependentProblemItem {
            imported_at_utc, statement_html, problem_id, title, source_url, rating: rating.map(u32::try_from).transpose().map_err(|_| SingleProblemError::Unavailable)?, has_statement_snapshot
        })).collect()
    }
}
