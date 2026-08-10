CREATE TABLE contests (
    id INTEGER PRIMARY KEY,
    platform TEXT NOT NULL CHECK (platform = 'codeforces'),
    external_contest_key INTEGER NOT NULL CHECK (external_contest_key > 0),
    title TEXT NOT NULL CHECK (length(title) > 0),
    source_url TEXT NOT NULL CHECK (length(source_url) > 0),
    starts_at_utc TEXT,
    import_status TEXT NOT NULL CHECK (import_status IN ('incomplete', 'complete')),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (platform, external_contest_key)
);

CREATE TABLE problems (
    id INTEGER PRIMARY KEY,
    platform TEXT NOT NULL CHECK (platform = 'codeforces'),
    external_contest_key INTEGER NOT NULL CHECK (external_contest_key > 0),
    external_problem_key TEXT NOT NULL CHECK (length(external_problem_key) BETWEEN 1 AND 8),
    title TEXT NOT NULL CHECK (length(title) > 0),
    rating INTEGER CHECK (rating > 0),
    source_url TEXT NOT NULL CHECK (length(source_url) > 0),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (platform, external_contest_key, external_problem_key)
);

CREATE TABLE contest_problems (
    contest_id INTEGER NOT NULL REFERENCES contests(id) ON DELETE RESTRICT,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    ordinal INTEGER NOT NULL CHECK (ordinal > 0),
    import_state TEXT NOT NULL CHECK (import_state IN ('pending_snapshot', 'ready')),
    PRIMARY KEY (contest_id, problem_id),
    UNIQUE (contest_id, ordinal)
);

CREATE TABLE problem_statement_snapshots (
    problem_id INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE RESTRICT,
    source_html TEXT NOT NULL CHECK (length(source_html) > 0),
    sanitized_html TEXT NOT NULL CHECK (length(sanitized_html) > 0),
    captured_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE problem_statement_assets (
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    local_ref TEXT NOT NULL CHECK (length(local_ref) > 0),
    media_type TEXT NOT NULL CHECK (length(media_type) > 0),
    bytes BLOB NOT NULL CHECK (length(bytes) > 0),
    PRIMARY KEY (problem_id, local_ref)
);

UPDATE app_metadata
SET schema_generation = 3
WHERE singleton = 1;
