CREATE TABLE contest_participations (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    contest_id INTEGER REFERENCES contests(id) ON DELETE SET NULL,
    contest_title TEXT NOT NULL CHECK (length(trim(contest_title)) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('live', 'virtual')),
    local_date TEXT NOT NULL CHECK (length(local_date) = 10 AND date(local_date, '+0 days') IS NOT NULL AND date(local_date, '+0 days') = local_date),
    local_time TEXT NOT NULL CHECK (length(local_time) = 5 AND local_time GLOB '[0-2][0-9]:[0-5][0-9]' AND local_time <= '23:59'),
    accepted_count INTEGER CHECK (accepted_count IS NULL OR (typeof(accepted_count) = 'integer' AND accepted_count >= 0)),
    total_count INTEGER CHECK (total_count IS NULL OR (typeof(total_count) = 'integer' AND total_count > 0)),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (accepted_count IS NULL OR total_count IS NULL OR accepted_count <= total_count),
    UNIQUE (contest_id, local_date, local_time)
);

UPDATE app_metadata SET schema_generation = 37 WHERE singleton = 1;
