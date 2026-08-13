CREATE TABLE knowledge_candidate_records (
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    fingerprint TEXT NOT NULL CHECK (
        length(fingerprint) = 64
        AND fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    target_ref TEXT NOT NULL CHECK (length(target_ref) BETWEEN 1 AND 512),
    disposition TEXT NOT NULL CHECK (
        disposition IN ('pending', 'accepted_intent', 'ignored')
    ),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (problem_id, fingerprint)
);

CREATE INDEX knowledge_candidate_records_by_problem
ON knowledge_candidate_records(problem_id, disposition, fingerprint);

UPDATE app_metadata
SET schema_generation = 15
WHERE singleton = 1;
