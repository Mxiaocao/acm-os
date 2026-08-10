ALTER TABLE problems
ADD COLUMN identity_type TEXT NOT NULL DEFAULT 'lightweight'
    CHECK (identity_type IN ('lightweight', 'personal'));

CREATE TABLE file_bindings (
    id INTEGER PRIMARY KEY,
    problem_id INTEGER NOT NULL UNIQUE REFERENCES problems(id) ON DELETE RESTRICT,
    vault_relative_path TEXT NOT NULL UNIQUE CHECK (length(vault_relative_path) > 0),
    windows_file_key TEXT,
    content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
    binding_state TEXT NOT NULL DEFAULT 'linked'
        CHECK (binding_state IN (
            'linked',
            'location_anomaly',
            'external_source_unavailable',
            'confirmed_deleted'
        )),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

UPDATE app_metadata
SET schema_generation = 4
WHERE singleton = 1;
