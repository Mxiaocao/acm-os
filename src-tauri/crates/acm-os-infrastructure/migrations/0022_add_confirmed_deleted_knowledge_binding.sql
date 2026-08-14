ALTER TABLE knowledge_file_bindings RENAME TO knowledge_file_bindings_before_confirmed_delete;

CREATE TABLE knowledge_file_bindings (
    knowledge_node_id TEXT PRIMARY KEY REFERENCES knowledge_nodes(id) ON DELETE RESTRICT,
    vault_relative_path TEXT NOT NULL UNIQUE CHECK (length(vault_relative_path) > 0),
    windows_file_key TEXT,
    content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
    location_state TEXT NOT NULL CHECK (
        location_state IN ('ready', 'location_anomaly', 'confirmed_deleted')
    ),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO knowledge_file_bindings (
    knowledge_node_id,
    vault_relative_path,
    windows_file_key,
    content_digest,
    location_state,
    updated_at_utc
)
SELECT
    knowledge_node_id,
    vault_relative_path,
    windows_file_key,
    content_digest,
    location_state,
    updated_at_utc
FROM knowledge_file_bindings_before_confirmed_delete;

DROP TABLE knowledge_file_bindings_before_confirmed_delete;

UPDATE app_metadata
SET schema_generation = 22
WHERE singleton = 1;
