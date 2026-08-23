CREATE TABLE problem_completion_occurrences (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    semantic_kind TEXT NOT NULL CHECK (semantic_kind IN ('learning_completion', 'contest_personal_solve')),
    recorded_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX problem_completion_occurrences_by_problem
    ON problem_completion_occurrences (problem_id, recorded_at_utc, id);

UPDATE app_metadata
SET schema_generation = 28
WHERE singleton = 1;
