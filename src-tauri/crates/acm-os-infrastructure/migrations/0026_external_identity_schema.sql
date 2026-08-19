CREATE TABLE _external_identity_migration_guard (
    verified INTEGER NOT NULL CHECK (verified = 1)
);

INSERT INTO _external_identity_migration_guard (verified)
SELECT CASE WHEN
    NOT EXISTS (
        SELECT 1 FROM contests
        WHERE platform IS NULL
           OR typeof(platform) != 'text'
           OR platform != 'codeforces'
           OR typeof(external_contest_key) != 'integer'
           OR external_contest_key <= 0
    )
    AND NOT EXISTS (
        SELECT 1 FROM contests
        GROUP BY platform, external_contest_key
        HAVING COUNT(*) != 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM problems
        WHERE platform IS NULL
           OR typeof(platform) != 'text'
           OR platform != 'codeforces'
           OR typeof(external_contest_key) != 'integer'
           OR external_contest_key <= 0
           OR external_problem_key IS NULL
           OR typeof(external_problem_key) != 'text'
           OR length(external_problem_key) = 0
    )
    AND NOT EXISTS (
        SELECT 1 FROM problems
        GROUP BY platform, external_contest_key, external_problem_key
        HAVING COUNT(*) != 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM contest_problems AS relationships
        LEFT JOIN contests AS contests
          ON contests.id = relationships.contest_id
        LEFT JOIN problems AS problems
          ON problems.id = relationships.problem_id
        WHERE contests.id IS NULL
           OR problems.id IS NULL
           OR contests.platform != problems.platform
           OR contests.external_contest_key != problems.external_contest_key
    )
    AND (SELECT COUNT(*) FROM contests)
        = (SELECT COUNT(DISTINCT id) FROM contests)
    AND (SELECT COUNT(*) FROM problems)
        = (SELECT COUNT(DISTINCT id) FROM problems)
THEN 1 ELSE 0 END;

DROP TABLE _external_identity_migration_guard;

CREATE TABLE _contests_schema_26 (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(title) > 0),
    source_url TEXT NOT NULL CHECK (length(source_url) > 0),
    starts_at_utc TEXT,
    import_status TEXT NOT NULL CHECK (import_status IN ('incomplete', 'complete')),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    facts_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (facts_status IN ('pending', 'completed')),
    facts_completed_at_utc TEXT,
    archived_at_utc TEXT
);

INSERT INTO _contests_schema_26 (
    id,
    title,
    source_url,
    starts_at_utc,
    import_status,
    created_at_utc,
    facts_status,
    facts_completed_at_utc,
    archived_at_utc
)
SELECT
    id,
    title,
    source_url,
    starts_at_utc,
    import_status,
    created_at_utc,
    facts_status,
    facts_completed_at_utc,
    archived_at_utc
FROM contests;

CREATE TABLE _problems_schema_26 (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(title) > 0),
    rating INTEGER CHECK (rating > 0),
    source_url TEXT NOT NULL CHECK (length(source_url) > 0),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    identity_type TEXT NOT NULL DEFAULT 'lightweight'
        CHECK (identity_type IN ('lightweight', 'personal'))
);

INSERT INTO _problems_schema_26 (
    id,
    title,
    rating,
    source_url,
    created_at_utc,
    identity_type
)
SELECT
    id,
    title,
    rating,
    source_url,
    created_at_utc,
    identity_type
FROM problems;

CREATE TABLE contest_external_identities (
    contest_id INTEGER NOT NULL,
    platform TEXT NOT NULL,
    external_contest_key TEXT NOT NULL,
    FOREIGN KEY (contest_id) REFERENCES contests(id) ON DELETE RESTRICT,
    UNIQUE (platform, external_contest_key)
);

INSERT INTO contest_external_identities (
    contest_id,
    platform,
    external_contest_key
)
SELECT id, platform, CAST(external_contest_key AS TEXT)
FROM contests;

CREATE TABLE problem_external_identities (
    problem_id INTEGER NOT NULL,
    platform TEXT NOT NULL,
    external_contest_key TEXT NOT NULL,
    external_problem_key TEXT NOT NULL,
    FOREIGN KEY (problem_id) REFERENCES problems(id) ON DELETE RESTRICT,
    UNIQUE (platform, external_contest_key, external_problem_key)
);

INSERT INTO problem_external_identities (
    problem_id,
    platform,
    external_contest_key,
    external_problem_key
)
SELECT
    id,
    platform,
    CAST(external_contest_key AS TEXT),
    external_problem_key
FROM problems;

DROP TABLE contests;
ALTER TABLE _contests_schema_26 RENAME TO contests;
DROP TABLE problems;
ALTER TABLE _problems_schema_26 RENAME TO problems;

CREATE TABLE _external_identity_copy_guard (
    verified INTEGER NOT NULL CHECK (verified = 1)
);

INSERT INTO _external_identity_copy_guard (verified)
SELECT CASE WHEN
    (SELECT COUNT(*) FROM contests)
        = (SELECT COUNT(*) FROM contest_external_identities)
    AND (SELECT COUNT(*) FROM problems)
        = (SELECT COUNT(*) FROM problem_external_identities)
    AND NOT EXISTS (
        SELECT id FROM contests
        EXCEPT
        SELECT contest_id FROM contest_external_identities
    )
    AND NOT EXISTS (
        SELECT id FROM problems
        EXCEPT
        SELECT problem_id FROM problem_external_identities
    )
    AND NOT EXISTS (
        SELECT 1
        FROM contest_problems AS relationships
        LEFT JOIN contests AS contests
          ON contests.id = relationships.contest_id
        LEFT JOIN problems AS problems
          ON problems.id = relationships.problem_id
        WHERE contests.id IS NULL OR problems.id IS NULL
    )
THEN 1 ELSE 0 END;

DROP TABLE _external_identity_copy_guard;

UPDATE app_metadata
SET schema_generation = 26
WHERE singleton = 1;
