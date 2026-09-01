CREATE TABLE contest_years (
    id INTEGER PRIMARY KEY CHECK (id > 0),
    value INTEGER NOT NULL UNIQUE CHECK (typeof(value) = 'integer' AND value > 0)
);

INSERT INTO contest_years (value)
SELECT DISTINCT value
FROM (
    SELECT CASE
        WHEN length(c.starts_at_utc) = 20
         AND c.starts_at_utc GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z'
         AND CAST(substr(c.starts_at_utc, 1, 4) AS INTEGER) > 0
         AND strftime('%Y-%m-%dT%H:%M:%SZ', c.starts_at_utc, '+0 seconds') = c.starts_at_utc
            THEN CAST(substr(c.starts_at_utc, 1, 4) AS INTEGER)
        WHEN typeof(p.year) = 'integer' AND p.year > 0 THEN p.year
        ELSE NULL
    END AS value
    FROM contest_placements p
    JOIN contests c ON c.id = p.contest_id
)
WHERE value IS NOT NULL;

ALTER TABLE contest_placements
    ADD COLUMN year_id INTEGER REFERENCES contest_years(id) ON DELETE RESTRICT;

UPDATE contest_placements
SET year_id = (
    SELECT y.id
    FROM contest_years y
    JOIN contests c ON c.id = contest_placements.contest_id
    WHERE y.value = CASE
        WHEN length(c.starts_at_utc) = 20
         AND c.starts_at_utc GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z'
         AND CAST(substr(c.starts_at_utc, 1, 4) AS INTEGER) > 0
         AND strftime('%Y-%m-%dT%H:%M:%SZ', c.starts_at_utc, '+0 seconds') = c.starts_at_utc
            THEN CAST(substr(c.starts_at_utc, 1, 4) AS INTEGER)
        WHEN typeof(contest_placements.year) = 'integer' AND contest_placements.year > 0
            THEN contest_placements.year
        ELSE NULL
    END
);

DROP INDEX contest_placements_unique_identity;

CREATE UNIQUE INDEX contest_placements_unique_identity
ON contest_placements (
    contest_id,
    family_id,
    COALESCE(series_id, 0),
    COALESCE(year_id, 0),
    COALESCE(ordinal, 0)
);

CREATE INDEX contest_placements_by_year ON contest_placements(year_id);

UPDATE app_metadata
SET schema_generation = 34
WHERE singleton = 1;
