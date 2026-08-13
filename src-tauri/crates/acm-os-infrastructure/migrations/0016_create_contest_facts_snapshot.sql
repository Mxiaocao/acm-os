ALTER TABLE contests
ADD COLUMN facts_status TEXT NOT NULL DEFAULT 'pending'
CHECK (facts_status IN ('pending', 'completed'));

ALTER TABLE contests
ADD COLUMN facts_completed_at_utc TEXT;

ALTER TABLE contest_problems
ADD COLUMN final_contest_result TEXT
CHECK (final_contest_result IN (
    'unknown',
    'not_attempted',
    'accepted',
    'wrong_answer',
    'time_limit_exceeded',
    'memory_limit_exceeded',
    'runtime_error',
    'compilation_error',
    'other_failed'
));

UPDATE app_metadata
SET schema_generation = 16
WHERE singleton = 1;
