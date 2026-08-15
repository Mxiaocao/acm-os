CREATE TABLE contest_families (
    id INTEGER PRIMARY KEY CHECK (id > 0),
    display_name TEXT NOT NULL COLLATE NOCASE
        CHECK (
            length(display_name) > 0
            AND display_name = trim(display_name)
        ),
    UNIQUE (display_name)
);

CREATE TABLE contest_series (
    id INTEGER PRIMARY KEY CHECK (id > 0),
    family_id INTEGER NOT NULL,
    display_name TEXT NOT NULL COLLATE NOCASE
        CHECK (
            length(display_name) > 0
            AND display_name = trim(display_name)
        ),
    FOREIGN KEY (family_id)
        REFERENCES contest_families(id)
        ON DELETE RESTRICT,
    UNIQUE (family_id, display_name),
    UNIQUE (id, family_id)
);

CREATE TABLE contest_placements (
    id INTEGER PRIMARY KEY CHECK (id > 0),
    contest_id INTEGER NOT NULL,
    family_id INTEGER NOT NULL,
    series_id INTEGER,
    year INTEGER CHECK (
        year IS NULL OR (typeof(year) = 'integer' AND year > 0)
    ),
    ordinal INTEGER CHECK (
        ordinal IS NULL OR (typeof(ordinal) = 'integer' AND ordinal > 0)
    ),
    FOREIGN KEY (contest_id)
        REFERENCES contests(id)
        ON DELETE RESTRICT,
    FOREIGN KEY (family_id)
        REFERENCES contest_families(id)
        ON DELETE RESTRICT,
    FOREIGN KEY (series_id, family_id)
        REFERENCES contest_series(id, family_id)
        ON DELETE RESTRICT
);

CREATE UNIQUE INDEX contest_placements_unique_identity
ON contest_placements (
    contest_id,
    family_id,
    COALESCE(series_id, 0),
    COALESCE(year, 0),
    COALESCE(ordinal, 0)
);

CREATE INDEX contest_placements_by_path
ON contest_placements(family_id, series_id, year, ordinal, contest_id);

INSERT INTO contest_families (display_name)
VALUES ('杭电'), ('牛客'), ('Codeforces'), ('XCPC'), ('周赛');

UPDATE app_metadata
SET schema_generation = 24
WHERE singleton = 1;
