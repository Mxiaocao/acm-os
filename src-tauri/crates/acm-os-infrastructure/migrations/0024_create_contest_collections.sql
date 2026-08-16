CREATE TABLE contest_collections (
    id INTEGER PRIMARY KEY,
    collection_key TEXT NOT NULL UNIQUE CHECK (length(collection_key) > 0),
    display_name TEXT NOT NULL CHECK (length(display_name) > 0),
    sort_order INTEGER NOT NULL CHECK (sort_order > 0),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE contest_collection_memberships (
    collection_id INTEGER NOT NULL REFERENCES contest_collections(id) ON DELETE CASCADE,
    contest_id INTEGER NOT NULL REFERENCES contests(id) ON DELETE CASCADE,
    ordinal INTEGER CHECK (ordinal IS NULL OR ordinal > 0),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (collection_id, contest_id),
    UNIQUE (collection_id, ordinal)
);

CREATE INDEX idx_contest_collection_memberships_contest
    ON contest_collection_memberships (contest_id);

CREATE INDEX idx_contest_collection_memberships_order
    ON contest_collection_memberships (collection_id, ordinal);

INSERT INTO contest_collections (collection_key, display_name, sort_order) VALUES
    ('hdu_multi_2026', '26 杭电多校', 10),
    ('nowcoder_multi_2026', '26 牛客多校', 20),
    ('hdu_spring_2026', '26 杭电春', 30),
    ('codeforces', 'Codeforces', 40),
    ('weekly', '周赛', 50),
    ('xcpc_provincial', 'XCPC 省赛', 60),
    ('xcpc_regional', 'XCPC 区域赛', 70);

INSERT INTO contest_collection_memberships (collection_id, contest_id)
SELECT collections.id, contests.id
FROM contest_collections AS collections
JOIN contests ON contests.platform = 'codeforces'
WHERE collections.collection_key = 'codeforces';

UPDATE app_metadata
SET schema_generation = 24
WHERE singleton = 1;
