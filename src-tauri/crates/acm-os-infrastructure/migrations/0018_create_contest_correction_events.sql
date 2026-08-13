CREATE TABLE contest_correction_events (
    id TEXT PRIMARY KEY CHECK (length(id) > 0),
    contest_id INTEGER NOT NULL REFERENCES contests(id) ON DELETE RESTRICT,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    field_name TEXT NOT NULL CHECK (field_name IN ('final_contest_result', 'upsolve_decision')),
    old_value TEXT NOT NULL CHECK (length(old_value) > 0),
    new_value TEXT NOT NULL CHECK (length(new_value) > 0),
    corrected_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX contest_correction_events_by_contest
ON contest_correction_events(contest_id, corrected_at_utc);

UPDATE app_metadata
SET schema_generation = 18
WHERE singleton = 1;
