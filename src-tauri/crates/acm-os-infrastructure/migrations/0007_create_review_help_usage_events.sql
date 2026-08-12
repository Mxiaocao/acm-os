CREATE TABLE review_help_usage_events (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    review_attempt_id TEXT NOT NULL REFERENCES review_attempts(id) ON DELETE RESTRICT,
    help_level INTEGER NOT NULL CHECK (help_level BETWEEN 1 AND 5),
    source_digest TEXT NOT NULL CHECK (length(source_digest) = 64),
    revealed_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

UPDATE app_metadata
SET schema_generation = 7
WHERE singleton = 1;
