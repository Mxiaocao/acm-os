ALTER TABLE knowledge_file_bindings RENAME TO knowledge_file_bindings_before_rebuild_decision;

CREATE TABLE knowledge_file_bindings (
    knowledge_node_id TEXT PRIMARY KEY REFERENCES knowledge_nodes(id) ON DELETE RESTRICT,
    vault_relative_path TEXT NOT NULL UNIQUE CHECK (length(vault_relative_path) > 0),
    windows_file_key TEXT,
    content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
    location_state TEXT NOT NULL CHECK (
        location_state IN ('ready', 'location_anomaly', 'confirmed_deleted', 'confirmed_deleted_replaced')
    ),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO knowledge_file_bindings SELECT * FROM knowledge_file_bindings_before_rebuild_decision;
DROP TABLE knowledge_file_bindings_before_rebuild_decision;

UPDATE app_metadata SET schema_generation = 23 WHERE singleton = 1;
