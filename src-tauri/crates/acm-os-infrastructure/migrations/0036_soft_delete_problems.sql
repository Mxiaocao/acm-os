-- Keep identities and all learning/contest relations intact.
ALTER TABLE problems ADD COLUMN deleted_at_utc TEXT;

UPDATE app_metadata SET schema_generation = 36 WHERE singleton = 1;
