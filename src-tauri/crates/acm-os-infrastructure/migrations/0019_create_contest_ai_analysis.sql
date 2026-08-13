CREATE TABLE contest_ai_analyses (
    contest_id INTEGER PRIMARY KEY REFERENCES contests(id) ON DELETE CASCADE,
    raw_text TEXT NOT NULL CHECK (length(raw_text) > 0),
    parse_status TEXT NOT NULL CHECK (parse_status IN ('complete', 'partial', 'failed')),
    parsed_projection_json TEXT NOT NULL CHECK (length(parsed_projection_json) > 0),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

UPDATE app_metadata SET schema_generation = 19 WHERE singleton = 1;
