-- Membership is independent of contest import and of personal-note identity.
CREATE TABLE problem_library_items (
    problem_id INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE RESTRICT,
    added_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    origin TEXT NOT NULL CHECK (origin IN ('legacy', 'explicit'))
);

-- Preserve everything that was visible before this migration. Historical
-- imports do not contain enough evidence to infer the user's intent.
INSERT INTO problem_library_items (problem_id, added_at_utc, origin)
SELECT id, created_at_utc, 'legacy' FROM problems;

UPDATE app_metadata SET schema_generation = 35 WHERE singleton = 1;
