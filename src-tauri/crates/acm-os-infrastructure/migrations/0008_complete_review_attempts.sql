ALTER TABLE review_attempts ADD COLUMN judgement TEXT
    CHECK (judgement IN ('mastered', 'partial', 'fail'));
ALTER TABLE review_attempts ADD COLUMN completed_local_date TEXT;
ALTER TABLE review_attempts ADD COLUMN final_ac INTEGER CHECK (final_ac IN (0, 1));
ALTER TABLE review_attempts ADD COLUMN first_submission_result TEXT;
ALTER TABLE review_attempts ADD COLUMN final_result TEXT;
ALTER TABLE review_attempts ADD COLUMN total_submissions INTEGER CHECK (total_submissions > 0);
ALTER TABLE review_attempts ADD COLUMN idea_independent INTEGER CHECK (idea_independent IN (0, 1));
ALTER TABLE review_attempts ADD COLUMN implementation_independent INTEGER CHECK (implementation_independent IN (0, 1));
ALTER TABLE review_attempts ADD COLUMN debug_independence TEXT
    CHECK (debug_independence IN ('not_needed', 'independent', 'used_solving_help'));
ALTER TABLE review_attempts ADD COLUMN external_help TEXT
    CHECK (external_help IN ('none', 'solving_hint', 'full_solution'));
ALTER TABLE review_attempts ADD COLUMN evidence_codes_json TEXT;

CREATE TABLE review_failure_reasons (
    review_attempt_id TEXT NOT NULL REFERENCES review_attempts(id) ON DELETE RESTRICT,
    reason_code TEXT NOT NULL CHECK (reason_code IN (
        'no_idea',
        'key_property_blocked',
        'derivation_blocked',
        'cannot_implement',
        'implementation_error',
        'boundary_error',
        'complexity_error',
        'other'
    )),
    other_text TEXT,
    PRIMARY KEY (review_attempt_id, reason_code),
    CHECK (
        (reason_code = 'other' AND other_text IS NOT NULL AND length(trim(other_text)) > 0)
        OR
        (reason_code != 'other' AND other_text IS NULL)
    )
);

CREATE TABLE review_void_events (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    review_attempt_id TEXT NOT NULL UNIQUE REFERENCES review_attempts(id) ON DELETE RESTRICT,
    reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
    voided_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

UPDATE app_metadata
SET schema_generation = 8
WHERE singleton = 1;
