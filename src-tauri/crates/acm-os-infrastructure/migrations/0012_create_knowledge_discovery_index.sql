CREATE TABLE knowledge_nodes (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE knowledge_file_bindings (
    knowledge_node_id TEXT PRIMARY KEY REFERENCES knowledge_nodes(id) ON DELETE RESTRICT,
    vault_relative_path TEXT NOT NULL UNIQUE CHECK (length(vault_relative_path) > 0),
    windows_file_key TEXT,
    content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
    location_state TEXT NOT NULL CHECK (location_state IN ('ready', 'location_anomaly')),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE knowledge_discovery_index (
    knowledge_node_id TEXT PRIMARY KEY REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL CHECK (length(display_name) > 0),
    vault_relative_path TEXT NOT NULL UNIQUE CHECK (length(vault_relative_path) > 0),
    content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
    indexed_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX knowledge_discovery_index_by_name
ON knowledge_discovery_index(display_name, knowledge_node_id);

UPDATE app_metadata
SET schema_generation = 12
WHERE singleton = 1;
