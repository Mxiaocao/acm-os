ALTER TABLE contests ADD COLUMN archived_at_utc TEXT;
UPDATE app_metadata SET schema_generation = 20 WHERE singleton = 1;
