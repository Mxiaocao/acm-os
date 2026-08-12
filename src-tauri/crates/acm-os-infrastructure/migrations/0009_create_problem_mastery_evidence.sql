CREATE TABLE problem_mastery_evidence (
    problem_id TEXT PRIMARY KEY REFERENCES problems(id) ON DELETE RESTRICT,
    recalls_problem INTEGER NOT NULL DEFAULT 0 CHECK (recalls_problem IN (0, 1)),
    multiple_solutions_clear INTEGER NOT NULL DEFAULT 0 CHECK (multiple_solutions_clear IN (0, 1)),
    knowledge_understood INTEGER NOT NULL DEFAULT 0 CHECK (knowledge_understood IN (0, 1)),
    implementation_fluent INTEGER NOT NULL DEFAULT 0 CHECK (implementation_fluent IN (0, 1)),
    can_adapt_or_create INTEGER NOT NULL DEFAULT 0 CHECK (can_adapt_or_create IN (0, 1)),
    transfer_solved_independently INTEGER NOT NULL DEFAULT 0 CHECK (transfer_solved_independently IN (0, 1)),
    historical_thoroughly_digested INTEGER NOT NULL DEFAULT 0
        CHECK (historical_thoroughly_digested IN (0, 1)),
    first_thoroughly_digested_local_date TEXT,
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (
        (historical_thoroughly_digested = 0 AND first_thoroughly_digested_local_date IS NULL)
        OR
        (historical_thoroughly_digested = 1 AND first_thoroughly_digested_local_date IS NOT NULL)
    )
);

INSERT INTO problem_mastery_evidence (problem_id)
SELECT id FROM problems;

UPDATE app_metadata
SET schema_generation = 9
WHERE singleton = 1;
