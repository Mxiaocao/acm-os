CREATE TABLE app_metadata (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    schema_generation INTEGER NOT NULL CHECK (schema_generation > 0),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO app_metadata (singleton, schema_generation)
VALUES (1, 1);
