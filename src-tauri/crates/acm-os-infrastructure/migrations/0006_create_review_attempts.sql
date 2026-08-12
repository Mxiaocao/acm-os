CREATE TABLE review_attempts (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    review_cycle_id TEXT NOT NULL REFERENCES review_cycles(id) ON DELETE RESTRICT,
    attempt_type TEXT NOT NULL
        CHECK (attempt_type IN ('first_cold_start', 'long_term_review', 'early_check')),
    attempt_status TEXT NOT NULL DEFAULT 'in_progress'
        CHECK (attempt_status IN ('in_progress', 'completed', 'void')),
    scheduled_due_local_date TEXT NOT NULL,
    started_early INTEGER NOT NULL CHECK (started_early IN (0, 1)),
    judgement_rule_version INTEGER NOT NULL CHECK (judgement_rule_version > 0),
    started_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at_utc TEXT,
    CHECK (
        (attempt_status = 'in_progress' AND completed_at_utc IS NULL)
        OR
        (attempt_status != 'in_progress' AND completed_at_utc IS NOT NULL)
    ),
    CHECK (
        (attempt_type = 'early_check' AND started_early = 1)
        OR
        (attempt_type != 'early_check' AND started_early = 0)
    )
);

CREATE UNIQUE INDEX one_in_progress_review_attempt_per_problem
ON review_attempts(problem_id)
WHERE attempt_status = 'in_progress';

UPDATE app_metadata
SET schema_generation = 6
WHERE singleton = 1;
