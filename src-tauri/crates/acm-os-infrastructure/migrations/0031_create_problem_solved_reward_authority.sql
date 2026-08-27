CREATE TABLE reward_problem_completion_evaluations (
    problem_completion_occurrence_id TEXT PRIMARY KEY
        REFERENCES problem_completion_occurrences(id) ON DELETE RESTRICT,
    problem_id INTEGER NOT NULL
        REFERENCES problems(id) ON DELETE RESTRICT,
    qualification TEXT NOT NULL
        CHECK (qualification IN ('qualifying', 'non_qualifying')),
    reason_code TEXT NOT NULL
        CHECK (
            (qualification = 'qualifying' AND reason_code = 'explicitly_accepted_digested')
            OR
            (qualification = 'non_qualifying' AND reason_code IN (
                'mechanical_reentry', 'insufficient_digestion', 'other'
            ))
        ),
    policy_key TEXT NOT NULL
        CHECK (policy_key = 'problem_solved_v1'),
    policy_version INTEGER NOT NULL
        CHECK (policy_version = 1),
    evaluated_at_utc TEXT NOT NULL
        CHECK (
            length(evaluated_at_utc) = 24
            AND substr(evaluated_at_utc, 5, 1) = '-'
            AND substr(evaluated_at_utc, 8, 1) = '-'
            AND substr(evaluated_at_utc, 11, 1) = 'T'
            AND substr(evaluated_at_utc, 14, 1) = ':'
            AND substr(evaluated_at_utc, 17, 1) = ':'
            AND substr(evaluated_at_utc, 20, 1) = '.'
            AND substr(evaluated_at_utc, 24, 1) = 'Z'
        )
);

CREATE INDEX reward_problem_completion_evaluations_by_problem
    ON reward_problem_completion_evaluations (problem_id, evaluated_at_utc, problem_completion_occurrence_id);

CREATE TABLE reward_problem_solved_positive_claims (
    problem_id INTEGER PRIMARY KEY
        REFERENCES problems(id) ON DELETE RESTRICT,
    reward_event_id TEXT NOT NULL UNIQUE,
    claimed_at_utc TEXT NOT NULL
        CHECK (
            length(claimed_at_utc) = 24
            AND substr(claimed_at_utc, 5, 1) = '-'
            AND substr(claimed_at_utc, 8, 1) = '-'
            AND substr(claimed_at_utc, 11, 1) = 'T'
            AND substr(claimed_at_utc, 14, 1) = ':'
            AND substr(claimed_at_utc, 17, 1) = ':'
            AND substr(claimed_at_utc, 20, 1) = '.'
            AND substr(claimed_at_utc, 24, 1) = 'Z'
        ),
    FOREIGN KEY (reward_event_id, problem_id)
        REFERENCES reward_events(id, problem_id) ON DELETE RESTRICT
);

CREATE INDEX reward_problem_solved_positive_claims_by_event
    ON reward_problem_solved_positive_claims (reward_event_id);

CREATE TRIGGER reward_problem_completion_evaluations_insert_guard
BEFORE INSERT ON reward_problem_completion_evaluations
WHEN NOT EXISTS (
    SELECT 1
    FROM problem_completion_occurrences occurrence
    WHERE occurrence.id = NEW.problem_completion_occurrence_id
      AND occurrence.problem_id = NEW.problem_id
      AND occurrence.semantic_kind = 'learning_completion'
)
BEGIN
    SELECT RAISE(ABORT, 'invalid ProblemSolved evaluation source');
END;

CREATE TRIGGER reward_problem_completion_evaluations_no_update
BEFORE UPDATE ON reward_problem_completion_evaluations
BEGIN
    SELECT RAISE(ABORT, 'ProblemSolved evaluations are immutable');
END;

CREATE TRIGGER reward_problem_completion_evaluations_no_delete
BEFORE DELETE ON reward_problem_completion_evaluations
BEGIN
    SELECT RAISE(ABORT, 'ProblemSolved evaluations are durable');
END;

CREATE TRIGGER reward_problem_solved_positive_claims_insert_guard
BEFORE INSERT ON reward_problem_solved_positive_claims
WHEN NOT EXISTS (
    SELECT 1
    FROM reward_events event
    JOIN reward_problem_completion_event_sources source
      ON source.reward_event_id = event.id
    JOIN problem_completion_occurrences occurrence
      ON occurrence.id = source.problem_completion_occurrence_id
    JOIN reward_problem_completion_evaluations evaluation
      ON evaluation.problem_completion_occurrence_id = occurrence.id
     AND evaluation.problem_id = NEW.problem_id
    JOIN reward_grants grant
      ON grant.reward_event_id = event.id
    WHERE event.id = NEW.reward_event_id
      AND event.problem_id = NEW.problem_id
      AND event.activation_relation = 'post_activation'
      AND occurrence.problem_id = NEW.problem_id
      AND occurrence.semantic_kind = 'learning_completion'
      AND evaluation.qualification = 'qualifying'
      AND evaluation.reason_code = 'explicitly_accepted_digested'
      AND evaluation.policy_key = 'problem_solved_v1'
      AND evaluation.policy_version = 1
      AND grant.xp_amount = 100
      AND grant.coin_amount = 100
      AND grant.decision_reason = 'qualified_positive'
      AND grant.policy_key = 'problem_solved_v1'
      AND grant.policy_version = 1
)
BEGIN
    SELECT RAISE(ABORT, 'invalid ProblemSolved positive claim');
END;

CREATE TRIGGER reward_problem_solved_positive_claims_no_update
BEFORE UPDATE ON reward_problem_solved_positive_claims
BEGIN
    SELECT RAISE(ABORT, 'ProblemSolved positive claims are immutable');
END;

CREATE TRIGGER reward_problem_solved_positive_claims_no_delete
BEFORE DELETE ON reward_problem_solved_positive_claims
BEGIN
    SELECT RAISE(ABORT, 'ProblemSolved positive claims are durable');
END;

CREATE TRIGGER reward_grants_problem_solved_insert_guard
BEFORE INSERT ON reward_grants
WHEN EXISTS (
    SELECT 1
    FROM reward_events event
    JOIN reward_problem_completion_event_sources source
      ON source.reward_event_id = event.id
    JOIN problem_completion_occurrences occurrence
      ON occurrence.id = source.problem_completion_occurrence_id
    WHERE event.id = NEW.reward_event_id
      AND occurrence.semantic_kind = 'learning_completion'
      AND (
          (event.activation_relation = 'pre_activation' AND NOT (
              NEW.xp_amount = 0 AND NEW.coin_amount = 0
              AND NEW.decision_reason = 'pre_activation'
              AND NEW.policy_key = 'problem_solved_v1'
              AND NEW.policy_version = 1
          ))
          OR
          (event.activation_relation = 'post_activation' AND NOT EXISTS (
              SELECT 1 FROM reward_problem_completion_evaluations evaluation
              WHERE evaluation.problem_completion_occurrence_id = occurrence.id
                AND evaluation.problem_id = occurrence.problem_id
          ))
          OR
          (event.activation_relation = 'post_activation' AND EXISTS (
              SELECT 1 FROM reward_problem_completion_evaluations evaluation
              WHERE evaluation.problem_completion_occurrence_id = occurrence.id
                AND evaluation.qualification = 'non_qualifying'
                AND NOT (
                    NEW.xp_amount = 0 AND NEW.coin_amount = 0
                    AND NEW.decision_reason = 'non_qualifying'
                    AND NEW.policy_key = 'problem_solved_v1'
                    AND NEW.policy_version = 1
                )
          ))
          OR
          (event.activation_relation = 'post_activation' AND EXISTS (
              SELECT 1 FROM reward_problem_completion_evaluations evaluation
              WHERE evaluation.problem_completion_occurrence_id = occurrence.id
                AND evaluation.qualification = 'qualifying'
          ) AND EXISTS (
              SELECT 1 FROM reward_problem_solved_positive_claims claim
              WHERE claim.problem_id = occurrence.problem_id
          ) AND NOT (
              NEW.xp_amount = 0 AND NEW.coin_amount = 0
              AND NEW.decision_reason = 'already_rewarded'
              AND NEW.policy_key = 'problem_solved_v1'
              AND NEW.policy_version = 1
          ))
          OR
          (event.activation_relation = 'post_activation' AND EXISTS (
              SELECT 1 FROM reward_problem_completion_evaluations evaluation
              WHERE evaluation.problem_completion_occurrence_id = occurrence.id
                AND evaluation.qualification = 'qualifying'
          ) AND NOT EXISTS (
              SELECT 1 FROM reward_problem_solved_positive_claims claim
              WHERE claim.problem_id = occurrence.problem_id
          ) AND NOT (
              NEW.xp_amount = 100 AND NEW.coin_amount = 100
              AND NEW.decision_reason = 'qualified_positive'
              AND NEW.policy_key = 'problem_solved_v1'
              AND NEW.policy_version = 1
          ))
      )
)
BEGIN
    SELECT RAISE(ABORT, 'invalid ProblemSolved reward grant');
END;

CREATE TRIGGER reward_grants_problem_solved_positive_claim
AFTER INSERT ON reward_grants
WHEN EXISTS (
    SELECT 1
    FROM reward_events event
    JOIN reward_problem_completion_event_sources source
      ON source.reward_event_id = event.id
    JOIN problem_completion_occurrences occurrence
      ON occurrence.id = source.problem_completion_occurrence_id
    JOIN reward_problem_completion_evaluations evaluation
      ON evaluation.problem_completion_occurrence_id = occurrence.id
    WHERE event.id = NEW.reward_event_id
      AND event.activation_relation = 'post_activation'
      AND occurrence.semantic_kind = 'learning_completion'
      AND evaluation.qualification = 'qualifying'
      AND evaluation.reason_code = 'explicitly_accepted_digested'
      AND evaluation.policy_key = 'problem_solved_v1'
      AND evaluation.policy_version = 1
      AND NEW.xp_amount = 100
      AND NEW.coin_amount = 100
      AND NEW.decision_reason = 'qualified_positive'
      AND NEW.policy_key = 'problem_solved_v1'
      AND NEW.policy_version = 1
)
BEGIN
    INSERT INTO reward_problem_solved_positive_claims
        (problem_id, reward_event_id, claimed_at_utc)
    SELECT event.problem_id, event.id, NEW.decided_at_utc
    FROM reward_events event
    WHERE event.id = NEW.reward_event_id;
END;

UPDATE app_metadata
SET schema_generation = 31
WHERE singleton = 1;
