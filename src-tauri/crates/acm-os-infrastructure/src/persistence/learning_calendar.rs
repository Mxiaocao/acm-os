use super::DatabaseRuntime;
use acm_os_application::learning_calendar::{LearningCalendarEntry, LearningCalendarPort};
use acm_os_application::TodaySnapshotError;
use acm_os_domain::LocalDate;

impl LearningCalendarPort for DatabaseRuntime {
    async fn load_learning_calendar(
        &self,
        start: LocalDate,
        end: LocalDate,
        today: LocalDate,
    ) -> Result<Vec<LearningCalendarEntry>, TodaySnapshotError> {
        let pool = self._pool.as_ref().ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        // One read statement gives a consistent snapshot across all three sources.
        // DISTINCT combines study completions in a plan and cycle creation for
        // the same problem/day. Review completions use their recorded local date.
        let rows: Vec<(String, i64, String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
            r#"
            WITH study_days AS (
                SELECT tp.local_date AS date, e.problem_id
                FROM today_plan_entries e JOIN today_plans tp ON tp.id = e.today_plan_id
                WHERE e.entry_status = 'completed'
                  AND e.reason IN ('continue_learning', 'relearn', 'upsolve')
                UNION
                SELECT date(rc.created_at_utc, 'localtime'), rc.problem_id FROM review_cycles rc
            ), events AS (
                SELECT s.date, p.id AS problem_id, p.title AS problem_title,
                       'study' AS kind, 'completed' AS status,
                       NULL AS due_date, NULL AS judgement
                FROM study_days s JOIN problems p ON p.id = s.problem_id
                UNION ALL
                SELECT ra.completed_local_date, p.id, p.title,
                       ra.attempt_type, 'completed', ra.scheduled_due_local_date, ra.judgement
                FROM review_attempts ra JOIN problems p ON p.id = ra.problem_id
                WHERE ra.attempt_status = 'completed'
                UNION ALL
                SELECT CASE WHEN rc.next_due_local_date < ?3 THEN ?3 ELSE rc.next_due_local_date END,
                       p.id, p.title,
                       CASE WHEN pls.learning_status = 'waiting_cold_start' THEN 'first_cold_start' ELSE 'long_term_review' END,
                       CASE WHEN EXISTS (SELECT 1 FROM review_attempts ra WHERE ra.problem_id = p.id AND ra.attempt_status = 'in_progress')
                            THEN 'inProgress' ELSE 'scheduled' END,
                       rc.next_due_local_date, NULL
                FROM review_cycles rc
                JOIN problems p ON p.id = rc.problem_id
                JOIN problem_learning_states pls ON pls.problem_id = p.id
                WHERE rc.cycle_status = 'active' AND p.deleted_at_utc IS NULL
                  AND p.identity_type = 'personal'
            )
            SELECT date, problem_id, problem_title, kind, status, due_date, judgement
            FROM events WHERE date >= ?1 AND date < ?2
            ORDER BY date, status, problem_id, kind
            "#,
        )
        .bind(start.to_iso_string())
        .bind(end.to_iso_string())
        .bind(today.to_iso_string())
        .fetch_all(pool)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        Ok(rows.into_iter().map(|(date, problem_id, problem_title, kind, status, due_date, judgement)| {
            LearningCalendarEntry { date, problem_id, problem_title, kind, status, due_date, judgement }
        }).collect())
    }
}
