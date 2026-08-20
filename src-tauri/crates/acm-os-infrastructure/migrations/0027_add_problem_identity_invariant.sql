CREATE UNIQUE INDEX problem_external_identities_one_key_per_problem_contest
ON problem_external_identities (
    problem_id,
    platform,
    external_contest_key
);

UPDATE app_metadata
SET schema_generation = 27
WHERE singleton = 1;
