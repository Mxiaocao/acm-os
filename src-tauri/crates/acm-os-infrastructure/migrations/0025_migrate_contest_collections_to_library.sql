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

INSERT INTO contest_series (family_id, display_name)
SELECT id, '暑期多校' FROM contest_families WHERE display_name IN ('杭电', '牛客');

INSERT INTO contest_series (family_id, display_name)
SELECT id, '春季赛' FROM contest_families WHERE display_name = '杭电';

INSERT INTO contest_series (family_id, display_name)
SELECT id, series_name
FROM contest_families
CROSS JOIN (
    SELECT '省赛' AS series_name
    UNION ALL SELECT '区域赛'
    UNION ALL SELECT '邀请赛'
)
WHERE display_name = 'XCPC';

CREATE TABLE _contest_collection_migration_map (
    collection_id INTEGER PRIMARY KEY,
    family_id INTEGER NOT NULL,
    series_id INTEGER,
    fixed_year INTEGER,
    derive_year INTEGER NOT NULL CHECK (derive_year IN (0, 1))
);

INSERT INTO _contest_collection_migration_map (
    collection_id,
    family_id,
    series_id,
    fixed_year,
    derive_year
)
SELECT
    collections.id,
    families.id,
    series.id,
    CASE collections.collection_key
        WHEN 'hdu_multi_2026' THEN 2026
        WHEN 'nowcoder_multi_2026' THEN 2026
        WHEN 'hdu_spring_2026' THEN 2026
        ELSE NULL
    END,
    CASE collections.collection_key
        WHEN 'codeforces' THEN 1
        WHEN 'weekly' THEN 1
        WHEN 'xcpc_provincial' THEN 1
        WHEN 'xcpc_regional' THEN 1
        ELSE 0
    END
FROM contest_collections AS collections
JOIN contest_families AS families
  ON families.display_name = CASE collections.collection_key
      WHEN 'hdu_multi_2026' THEN '杭电'
      WHEN 'nowcoder_multi_2026' THEN '牛客'
      WHEN 'hdu_spring_2026' THEN '杭电'
      WHEN 'codeforces' THEN 'Codeforces'
      WHEN 'weekly' THEN '周赛'
      WHEN 'xcpc_provincial' THEN 'XCPC'
      WHEN 'xcpc_regional' THEN 'XCPC'
  END
LEFT JOIN contest_series AS series
  ON series.family_id = families.id
 AND series.display_name = CASE collections.collection_key
      WHEN 'hdu_multi_2026' THEN '暑期多校'
      WHEN 'nowcoder_multi_2026' THEN '暑期多校'
      WHEN 'hdu_spring_2026' THEN '春季赛'
      WHEN 'xcpc_provincial' THEN '省赛'
      WHEN 'xcpc_regional' THEN '区域赛'
  END
WHERE collections.collection_key IN (
    'hdu_multi_2026',
    'nowcoder_multi_2026',
    'hdu_spring_2026',
    'codeforces',
    'weekly',
    'xcpc_provincial',
    'xcpc_regional'
);

INSERT INTO contest_families (display_name)
SELECT
    CASE
        WHEN length(trim(display_name)) > 0
            THEN trim(display_name) || ' [' || collection_key || '#' || id || ']'
        ELSE collection_key || ' [' || collection_key || '#' || id || ']'
    END
FROM contest_collections
WHERE collection_key NOT IN (
    'hdu_multi_2026',
    'nowcoder_multi_2026',
    'hdu_spring_2026',
    'codeforces',
    'weekly',
    'xcpc_provincial',
    'xcpc_regional'
)
ORDER BY sort_order, id;

INSERT INTO _contest_collection_migration_map (
    collection_id,
    family_id,
    series_id,
    fixed_year,
    derive_year
)
SELECT
    collections.id,
    families.id,
    NULL,
    NULL,
    0
FROM contest_collections AS collections
JOIN contest_families AS families
  ON families.display_name = CASE
      WHEN length(trim(collections.display_name)) > 0
          THEN trim(collections.display_name) || ' [' || collections.collection_key || '#' || collections.id || ']'
      ELSE collections.collection_key || ' [' || collections.collection_key || '#' || collections.id || ']'
  END
WHERE collections.collection_key NOT IN (
    'hdu_multi_2026',
    'nowcoder_multi_2026',
    'hdu_spring_2026',
    'codeforces',
    'weekly',
    'xcpc_provincial',
    'xcpc_regional'
);

INSERT INTO contest_placements (
    contest_id,
    family_id,
    series_id,
    year,
    ordinal
)
SELECT
    memberships.contest_id,
    mapping.family_id,
    mapping.series_id,
    CASE
        WHEN mapping.fixed_year IS NOT NULL THEN mapping.fixed_year
        WHEN mapping.derive_year = 1
         AND contests.starts_at_utc GLOB '[0-9][0-9][0-9][0-9]-*'
         AND CAST(substr(contests.starts_at_utc, 1, 4) AS INTEGER) > 0
            THEN CAST(substr(contests.starts_at_utc, 1, 4) AS INTEGER)
        ELSE NULL
    END,
    memberships.ordinal
FROM contest_collection_memberships AS memberships
JOIN _contest_collection_migration_map AS mapping
  ON mapping.collection_id = memberships.collection_id
JOIN contests ON contests.id = memberships.contest_id;

CREATE TABLE _contest_collection_migration_guard (
    verified INTEGER NOT NULL CHECK (verified = 1)
);

INSERT INTO _contest_collection_migration_guard (verified)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM _contest_collection_migration_map)
           = (SELECT COUNT(*) FROM contest_collections)
     AND (SELECT COUNT(*) FROM contest_placements)
           = (SELECT COUNT(*) FROM contest_collection_memberships)
        THEN 1
    ELSE 0
END;

DROP TABLE _contest_collection_migration_guard;
DROP TABLE _contest_collection_migration_map;
DROP TABLE contest_collection_memberships;
DROP TABLE contest_collections;

UPDATE app_metadata
SET schema_generation = 25
WHERE singleton = 1;
