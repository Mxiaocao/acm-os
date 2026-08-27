CREATE TABLE custom_rewards (
    custom_reward_id TEXT PRIMARY KEY CHECK (length(custom_reward_id) = 36),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    coin_cost INTEGER NOT NULL CHECK (
        typeof(coin_cost) = 'integer'
        AND coin_cost >= 1
        AND coin_cost <= 9007199254740991
    ),
    status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
    created_at_utc TEXT NOT NULL CHECK (length(created_at_utc) = 24),
    updated_at_utc TEXT NOT NULL CHECK (length(updated_at_utc) = 24)
);

CREATE INDEX custom_rewards_by_status_updated
    ON custom_rewards (status, updated_at_utc, custom_reward_id);

UPDATE app_metadata
SET schema_generation = 32
WHERE singleton = 1;
