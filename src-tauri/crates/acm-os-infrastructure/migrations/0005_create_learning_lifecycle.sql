CREATE TABLE problem_learning_states (
    problem_id INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE RESTRICT,
    learning_status TEXT NOT NULL DEFAULT 'unstarted'
        CHECK (learning_status IN (
            'unstarted',
            'upsolve_pending',
            'learning',
            'waiting_cold_start',
            'relearning',
            'long_term_review'
        )),
    learning_status_since_utc TEXT NOT NULL
        DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO problem_learning_states (problem_id, learning_status_since_utc)
SELECT id, created_at_utc FROM problems;

CREATE TABLE review_cycles (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    cycle_number INTEGER NOT NULL CHECK (cycle_number > 0),
    cycle_status TEXT NOT NULL
        CHECK (cycle_status IN ('active', 'cancelled', 'suspended', 'completed')),
    stage INTEGER NOT NULL DEFAULT 0 CHECK (stage >= 0),
    schedule_rule_version INTEGER NOT NULL CHECK (schedule_rule_version > 0),
    next_due_local_date TEXT,
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ended_at_utc TEXT,
    UNIQUE (problem_id, cycle_number),
    CHECK (
        (cycle_status = 'active' AND next_due_local_date IS NOT NULL AND ended_at_utc IS NULL)
        OR
        (cycle_status != 'active' AND next_due_local_date IS NULL AND ended_at_utc IS NOT NULL)
    )
);

CREATE UNIQUE INDEX one_active_review_cycle_per_problem
ON review_cycles(problem_id)
WHERE cycle_status = 'active';

UPDATE app_metadata
SET schema_generation = 5
WHERE singleton = 1;
