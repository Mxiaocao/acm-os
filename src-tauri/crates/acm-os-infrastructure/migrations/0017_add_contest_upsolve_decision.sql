ALTER TABLE contest_problems
ADD COLUMN upsolve_decision TEXT NOT NULL DEFAULT 'undecided'
CHECK (upsolve_decision IN ('planned', 'not_planned', 'undecided'));

UPDATE app_metadata
SET schema_generation = 17
WHERE singleton = 1;
