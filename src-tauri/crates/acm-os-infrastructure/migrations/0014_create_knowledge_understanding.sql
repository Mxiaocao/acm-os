CREATE TABLE knowledge_understanding_states (
    knowledge_node_id TEXT PRIMARY KEY REFERENCES knowledge_nodes(id) ON DELETE RESTRICT,
    current_level TEXT NOT NULL CHECK (current_level IN ('not_learned', 'vague', 'basic', 'proficient', 'deep')),
    historical_highest_level TEXT NOT NULL CHECK (historical_highest_level IN ('not_learned', 'vague', 'basic', 'proficient', 'deep')),
    first_reached_highest_local_date TEXT NOT NULL,
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

UPDATE app_metadata
SET schema_generation = 14
WHERE singleton = 1;
