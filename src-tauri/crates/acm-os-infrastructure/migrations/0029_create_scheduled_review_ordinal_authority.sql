CREATE UNIQUE INDEX review_attempts_id_problem_unique
ON review_attempts(id, problem_id);

CREATE TABLE scheduled_review_ordinal_states (
    problem_id INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE RESTRICT,
    historical_baseline INTEGER NOT NULL
        CHECK (typeof(historical_baseline) = 'integer' AND historical_baseline >= 0),
    last_allocated INTEGER NOT NULL
        CHECK (
            typeof(last_allocated) = 'integer'
            AND last_allocated >= historical_baseline
        )
);

CREATE TABLE scheduled_review_ordinal_facts (
    review_attempt_id TEXT PRIMARY KEY,
    problem_id INTEGER NOT NULL,
    ordinal INTEGER NOT NULL
        CHECK (typeof(ordinal) = 'integer' AND ordinal > 0),
    recorded_at_utc TEXT NOT NULL
        DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (problem_id, ordinal),
    FOREIGN KEY (problem_id) REFERENCES problems(id) ON DELETE RESTRICT,
    FOREIGN KEY (review_attempt_id, problem_id)
        REFERENCES review_attempts(id, problem_id) ON DELETE RESTRICT
);

INSERT INTO scheduled_review_ordinal_states (
    problem_id,
    historical_baseline,
    last_allocated
)
SELECT
    problem_id,
    COUNT(*),
    COUNT(*)
FROM review_attempts
WHERE attempt_status = 'completed'
  AND attempt_type IN ('first_cold_start', 'long_term_review')
GROUP BY problem_id
HAVING COUNT(*) > 0;

CREATE TRIGGER scheduled_review_ordinal_facts_no_update
BEFORE UPDATE ON scheduled_review_ordinal_facts
BEGIN
    SELECT RAISE(ABORT, 'scheduled review ordinal facts are immutable');
END;

CREATE TRIGGER scheduled_review_ordinal_facts_no_delete
BEFORE DELETE ON scheduled_review_ordinal_facts
BEGIN
    SELECT RAISE(ABORT, 'scheduled review ordinal facts are append-only');
END;

CREATE TRIGGER scheduled_review_ordinal_states_no_delete
BEFORE DELETE ON scheduled_review_ordinal_states
BEGIN
    SELECT RAISE(ABORT, 'scheduled review ordinal state is durable');
END;

CREATE TRIGGER scheduled_review_ordinal_baseline_immutable
BEFORE UPDATE OF historical_baseline ON scheduled_review_ordinal_states
WHEN NEW.historical_baseline != OLD.historical_baseline
BEGIN
    SELECT RAISE(ABORT, 'scheduled review historical baseline is immutable');
END;

CREATE TRIGGER scheduled_review_ordinal_last_monotonic
BEFORE UPDATE OF last_allocated ON scheduled_review_ordinal_states
WHEN NEW.last_allocated != OLD.last_allocated + 1
BEGIN
    SELECT RAISE(ABORT, 'scheduled review ordinal allocation must advance by one');
END;

CREATE TRIGGER scheduled_review_ordinal_fact_insert_guard
BEFORE INSERT ON scheduled_review_ordinal_facts
WHEN NOT EXISTS (
    SELECT 1
    FROM review_attempts AS attempt
    JOIN scheduled_review_ordinal_states AS state
      ON state.problem_id = attempt.problem_id
    WHERE attempt.id = NEW.review_attempt_id
      AND attempt.problem_id = NEW.problem_id
      AND attempt.attempt_status = 'in_progress'
      AND attempt.attempt_type IN ('first_cold_start', 'long_term_review')
      AND NEW.ordinal = state.last_allocated
      AND state.last_allocated = state.historical_baseline + 1 + (
          SELECT COUNT(*)
          FROM scheduled_review_ordinal_facts AS existing
          WHERE existing.problem_id = NEW.problem_id
      )
)
BEGIN
    SELECT RAISE(ABORT, 'invalid scheduled review ordinal fact');
END;

CREATE TRIGGER scheduled_review_completion_requires_ordinal
BEFORE UPDATE OF attempt_status ON review_attempts
WHEN OLD.attempt_status = 'in_progress'
 AND NEW.attempt_status = 'completed'
 AND OLD.attempt_type IN ('first_cold_start', 'long_term_review')
 AND NOT EXISTS (
     SELECT 1
     FROM scheduled_review_ordinal_facts AS fact
     WHERE fact.review_attempt_id = OLD.id
       AND fact.problem_id = OLD.problem_id
 )
BEGIN
    SELECT RAISE(ABORT, 'scheduled review completion requires ordinal authority');
END;

CREATE TRIGGER scheduled_review_ordinal_attempt_identity_immutable
BEFORE UPDATE OF id, problem_id, attempt_type ON review_attempts
WHEN EXISTS (
    SELECT 1
    FROM scheduled_review_ordinal_facts AS fact
    WHERE fact.review_attempt_id = OLD.id
)
BEGIN
    SELECT RAISE(ABORT, 'ordinal source attempt identity is immutable');
END;

CREATE TRIGGER scheduled_review_ordinal_attempt_must_complete
BEFORE UPDATE OF attempt_status ON review_attempts
WHEN EXISTS (
    SELECT 1
    FROM scheduled_review_ordinal_facts AS fact
    WHERE fact.review_attempt_id = OLD.id
)
AND NEW.attempt_status != 'completed'
BEGIN
    SELECT RAISE(ABORT, 'ordinal source attempt must complete');
END;

CREATE TRIGGER completed_scheduled_review_no_update
BEFORE UPDATE ON review_attempts
WHEN OLD.attempt_status = 'completed'
 AND OLD.attempt_type IN ('first_cold_start', 'long_term_review')
BEGIN
    SELECT RAISE(ABORT, 'completed scheduled review is immutable');
END;

CREATE TRIGGER completed_scheduled_review_no_delete
BEFORE DELETE ON review_attempts
WHEN OLD.attempt_status = 'completed'
 AND OLD.attempt_type IN ('first_cold_start', 'long_term_review')
BEGIN
    SELECT RAISE(ABORT, 'completed scheduled review is durable');
END;

UPDATE app_metadata
SET schema_generation = 29
WHERE singleton = 1;
