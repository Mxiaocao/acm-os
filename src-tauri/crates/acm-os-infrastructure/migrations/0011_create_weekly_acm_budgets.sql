CREATE TABLE weekly_acm_budgets (
    weekday INTEGER PRIMARY KEY CHECK (weekday BETWEEN 1 AND 7),
    budget_minutes INTEGER NOT NULL CHECK (budget_minutes >= 0),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

UPDATE app_metadata
SET schema_generation = 11
WHERE singleton = 1;
