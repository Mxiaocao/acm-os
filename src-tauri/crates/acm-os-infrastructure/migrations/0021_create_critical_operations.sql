CREATE TABLE critical_operations (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    operation_kind TEXT NOT NULL CHECK (operation_kind IN ('markdown_system_fact')),
    object_type TEXT NOT NULL CHECK (length(object_type) > 0),
    object_id TEXT NOT NULL CHECK (length(object_id) > 0),
    binding_id INTEGER REFERENCES file_bindings(id) ON DELETE RESTRICT,
    pre_content_digest TEXT NOT NULL CHECK (length(pre_content_digest) = 64),
    postcondition_json TEXT NOT NULL CHECK (length(postcondition_json) > 0),
    operation_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (operation_status IN ('pending', 'needs_recovery', 'completed', 'abandoned')),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    resolved_at_utc TEXT,
    CHECK (
        (operation_status IN ('pending', 'needs_recovery') AND resolved_at_utc IS NULL)
        OR
        (operation_status IN ('completed', 'abandoned') AND resolved_at_utc IS NOT NULL)
    )
);

CREATE INDEX critical_operations_by_status
ON critical_operations(operation_status, created_at_utc);

UPDATE app_metadata
SET schema_generation = 21
WHERE singleton = 1;
