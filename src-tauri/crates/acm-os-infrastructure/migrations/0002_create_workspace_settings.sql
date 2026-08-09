CREATE TABLE workspace_settings (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    active_vault_path TEXT NOT NULL CHECK (length(active_vault_path) > 0),
    problem_root_path TEXT NOT NULL CHECK (length(problem_root_path) > 0),
    knowledge_root_path TEXT NOT NULL CHECK (length(knowledge_root_path) > 0),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

UPDATE app_metadata
SET schema_generation = 2
WHERE singleton = 1;
