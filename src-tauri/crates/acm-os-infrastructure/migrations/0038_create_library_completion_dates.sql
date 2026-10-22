CREATE TABLE library_completion_dates (
    problem_id INTEGER PRIMARY KEY REFERENCES problem_library_items(problem_id) ON DELETE CASCADE,
    local_date TEXT NOT NULL CHECK (length(local_date) = 10 AND date(local_date, '+0 days') IS NOT NULL AND date(local_date, '+0 days') = local_date)
);

UPDATE app_metadata SET schema_generation = 38 WHERE singleton = 1;
