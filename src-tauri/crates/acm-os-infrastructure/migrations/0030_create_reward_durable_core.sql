CREATE TABLE reward_activation_state (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    activation_status TEXT NOT NULL
        CHECK (activation_status IN ('inactive', 'active')),
    installed_at_utc TEXT NOT NULL
        DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (
            length(installed_at_utc) = 24
            AND substr(installed_at_utc, 5, 1) = '-'
            AND substr(installed_at_utc, 8, 1) = '-'
            AND substr(installed_at_utc, 11, 1) = 'T'
            AND substr(installed_at_utc, 14, 1) = ':'
            AND substr(installed_at_utc, 17, 1) = ':'
            AND substr(installed_at_utc, 20, 1) = '.'
            AND substr(installed_at_utc, 24, 1) = 'Z'
        ),
    activated_at_utc TEXT
        CHECK (
            activated_at_utc IS NULL
            OR (
                length(activated_at_utc) = 24
                AND substr(activated_at_utc, 5, 1) = '-'
                AND substr(activated_at_utc, 8, 1) = '-'
                AND substr(activated_at_utc, 11, 1) = 'T'
                AND substr(activated_at_utc, 14, 1) = ':'
                AND substr(activated_at_utc, 17, 1) = ':'
                AND substr(activated_at_utc, 20, 1) = '.'
                AND substr(activated_at_utc, 24, 1) = 'Z'
            )
        ),
    CHECK (
        (activation_status = 'inactive' AND activated_at_utc IS NULL)
        OR
        (activation_status = 'active' AND activated_at_utc IS NOT NULL)
    )
);

INSERT INTO reward_activation_state (singleton, activation_status)
VALUES (1, 'inactive');

CREATE TABLE reward_events (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    problem_id INTEGER REFERENCES problems(id) ON DELETE RESTRICT,
    source_occurred_at_utc TEXT NOT NULL CHECK (length(source_occurred_at_utc) = 24),
    activation_relation TEXT NOT NULL
        CHECK (activation_relation IN ('pre_activation', 'post_activation')),
    recorded_at_utc TEXT NOT NULL
        DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (length(recorded_at_utc) = 24),
    UNIQUE (id, problem_id)
);

CREATE TABLE reward_problem_completion_event_sources (
    reward_event_id TEXT PRIMARY KEY
        REFERENCES reward_events(id) ON DELETE RESTRICT,
    problem_completion_occurrence_id TEXT NOT NULL UNIQUE
        REFERENCES problem_completion_occurrences(id) ON DELETE RESTRICT
);

CREATE TABLE reward_review_event_sources (
    reward_event_id TEXT PRIMARY KEY,
    review_attempt_id TEXT NOT NULL UNIQUE,
    problem_id INTEGER NOT NULL,
    FOREIGN KEY (reward_event_id, problem_id)
        REFERENCES reward_events(id, problem_id) ON DELETE RESTRICT,
    FOREIGN KEY (review_attempt_id, problem_id)
        REFERENCES review_attempts(id, problem_id) ON DELETE RESTRICT
);

CREATE TABLE reward_grants (
    reward_event_id TEXT PRIMARY KEY
        REFERENCES reward_events(id) ON DELETE RESTRICT,
    xp_amount INTEGER NOT NULL
        CHECK (
            typeof(xp_amount) = 'integer'
            AND xp_amount >= 0
            AND xp_amount <= 9007199254740991
        ),
    coin_amount INTEGER NOT NULL
        CHECK (
            typeof(coin_amount) = 'integer'
            AND coin_amount >= 0
            AND coin_amount <= 9007199254740991
        ),
    decision_reason TEXT NOT NULL
        CHECK (length(trim(decision_reason)) BETWEEN 1 AND 100),
    policy_key TEXT NOT NULL
        CHECK (length(trim(policy_key)) BETWEEN 1 AND 100),
    policy_version INTEGER NOT NULL
        CHECK (typeof(policy_version) = 'integer' AND policy_version > 0),
    decided_at_utc TEXT NOT NULL
        DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (length(decided_at_utc) = 24)
);

CREATE TABLE reward_ledger_entries (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('xp', 'coin')),
    delta INTEGER NOT NULL
        CHECK (
            typeof(delta) = 'integer'
            AND delta != 0
            AND delta BETWEEN -9007199254740991 AND 9007199254740991
        ),
    recorded_at_utc TEXT NOT NULL
        DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        CHECK (length(recorded_at_utc) = 24),
    UNIQUE (id, resource_kind)
);

CREATE TABLE reward_grant_ledger_origins (
    ledger_entry_id TEXT PRIMARY KEY,
    reward_event_id TEXT NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('xp', 'coin')),
    UNIQUE (reward_event_id, resource_kind),
    FOREIGN KEY (ledger_entry_id, resource_kind)
        REFERENCES reward_ledger_entries(id, resource_kind) ON DELETE RESTRICT,
    FOREIGN KEY (reward_event_id)
        REFERENCES reward_grants(reward_event_id) ON DELETE RESTRICT
);

CREATE INDEX reward_events_by_problem
    ON reward_events (problem_id, source_occurred_at_utc, id);
CREATE INDEX reward_events_by_recorded_at
    ON reward_events (recorded_at_utc, id);
CREATE INDEX reward_ledger_entries_by_resource
    ON reward_ledger_entries (resource_kind, id);

CREATE TRIGGER reward_activation_state_no_delete
BEFORE DELETE ON reward_activation_state
BEGIN
    SELECT RAISE(ABORT, 'reward activation authority is durable');
END;

CREATE TRIGGER reward_activation_state_transition_guard
BEFORE UPDATE ON reward_activation_state
WHEN NOT (
    OLD.singleton = 1
    AND NEW.singleton = OLD.singleton
    AND NEW.installed_at_utc = OLD.installed_at_utc
    AND OLD.activation_status = 'inactive'
    AND OLD.activated_at_utc IS NULL
    AND NEW.activation_status = 'active'
    AND NEW.activated_at_utc IS NOT NULL
)
BEGIN
    SELECT RAISE(ABORT, 'invalid reward activation transition');
END;

CREATE TRIGGER reward_events_no_update
BEFORE UPDATE ON reward_events
BEGIN
    SELECT RAISE(ABORT, 'reward events are immutable');
END;

CREATE TRIGGER reward_events_no_delete
BEFORE DELETE ON reward_events
BEGIN
    SELECT RAISE(ABORT, 'reward events are durable');
END;

CREATE TRIGGER reward_problem_completion_sources_insert_guard
BEFORE INSERT ON reward_problem_completion_event_sources
WHEN NOT EXISTS (
    SELECT 1
    FROM reward_events event
    JOIN problem_completion_occurrences occurrence
      ON occurrence.id = NEW.problem_completion_occurrence_id
    WHERE event.id = NEW.reward_event_id
      AND event.problem_id IS NOT NULL
      AND event.problem_id = occurrence.problem_id
      AND event.source_occurred_at_utc = occurrence.recorded_at_utc
)
BEGIN
    SELECT RAISE(ABORT, 'invalid reward problem completion source');
END;

CREATE TRIGGER reward_problem_completion_sources_exclusive
BEFORE INSERT ON reward_problem_completion_event_sources
WHEN EXISTS (
    SELECT 1 FROM reward_review_event_sources
    WHERE reward_event_id = NEW.reward_event_id
)
BEGIN
    SELECT RAISE(ABORT, 'reward event already has another source family');
END;

CREATE TRIGGER reward_problem_completion_sources_no_update
BEFORE UPDATE ON reward_problem_completion_event_sources
BEGIN
    SELECT RAISE(ABORT, 'reward event sources are immutable');
END;

CREATE TRIGGER reward_problem_completion_sources_no_delete
BEFORE DELETE ON reward_problem_completion_event_sources
BEGIN
    SELECT RAISE(ABORT, 'reward event sources are durable');
END;

CREATE TRIGGER reward_review_sources_insert_guard
BEFORE INSERT ON reward_review_event_sources
WHEN NOT EXISTS (
    SELECT 1
    FROM reward_events event
    JOIN review_attempts attempt
      ON attempt.id = NEW.review_attempt_id
     AND attempt.problem_id = NEW.problem_id
    WHERE event.id = NEW.reward_event_id
      AND event.problem_id = NEW.problem_id
      AND event.source_occurred_at_utc = attempt.completed_at_utc
      AND attempt.attempt_status = 'completed'
      AND (
          (
              attempt.attempt_type = 'early_check'
              AND NOT EXISTS (
                  SELECT 1 FROM scheduled_review_ordinal_facts fact
                  WHERE fact.review_attempt_id = attempt.id
              )
          )
          OR
          (
              attempt.attempt_type IN ('first_cold_start', 'long_term_review')
              AND EXISTS (
                  SELECT 1 FROM scheduled_review_ordinal_facts fact
                  WHERE fact.review_attempt_id = attempt.id
                    AND fact.problem_id = attempt.problem_id
              )
          )
      )
)
BEGIN
    SELECT RAISE(ABORT, 'invalid reward review source');
END;

CREATE TRIGGER reward_review_sources_exclusive
BEFORE INSERT ON reward_review_event_sources
WHEN EXISTS (
    SELECT 1 FROM reward_problem_completion_event_sources
    WHERE reward_event_id = NEW.reward_event_id
)
BEGIN
    SELECT RAISE(ABORT, 'reward event already has another source family');
END;

CREATE TRIGGER reward_review_sources_no_update
BEFORE UPDATE ON reward_review_event_sources
BEGIN
    SELECT RAISE(ABORT, 'reward event sources are immutable');
END;

CREATE TRIGGER reward_review_sources_no_delete
BEFORE DELETE ON reward_review_event_sources
BEGIN
    SELECT RAISE(ABORT, 'reward event sources are durable');
END;

CREATE TRIGGER reward_grants_insert_guard
BEFORE INSERT ON reward_grants
WHEN NOT EXISTS (
    SELECT 1 FROM reward_events event
    WHERE event.id = NEW.reward_event_id
      AND (
          event.activation_relation = 'post_activation'
          OR (NEW.xp_amount = 0 AND NEW.coin_amount = 0)
      )
)
BEGIN
    SELECT RAISE(ABORT, 'invalid reward grant');
END;

CREATE TRIGGER reward_grants_no_update
BEFORE UPDATE ON reward_grants
BEGIN
    SELECT RAISE(ABORT, 'reward grants are immutable');
END;

CREATE TRIGGER reward_grants_no_delete
BEFORE DELETE ON reward_grants
BEGIN
    SELECT RAISE(ABORT, 'reward grants are durable');
END;

CREATE TRIGGER reward_ledger_entries_no_update
BEFORE UPDATE ON reward_ledger_entries
BEGIN
    SELECT RAISE(ABORT, 'reward ledger entries are immutable');
END;

CREATE TRIGGER reward_ledger_entries_no_delete
BEFORE DELETE ON reward_ledger_entries
BEGIN
    SELECT RAISE(ABORT, 'reward ledger entries are durable');
END;

CREATE TRIGGER reward_grant_ledger_origins_insert_guard
BEFORE INSERT ON reward_grant_ledger_origins
WHEN NOT EXISTS (
    SELECT 1
    FROM reward_ledger_entries entry
    JOIN reward_grants grant ON grant.reward_event_id = NEW.reward_event_id
    WHERE entry.id = NEW.ledger_entry_id
      AND entry.resource_kind = NEW.resource_kind
      AND entry.delta > 0
      AND entry.delta = CASE NEW.resource_kind
          WHEN 'xp' THEN grant.xp_amount
          WHEN 'coin' THEN grant.coin_amount
      END
)
BEGIN
    SELECT RAISE(ABORT, 'reward ledger effect disagrees with grant');
END;

CREATE TRIGGER reward_grant_ledger_origins_no_update
BEFORE UPDATE ON reward_grant_ledger_origins
BEGIN
    SELECT RAISE(ABORT, 'reward ledger origins are immutable');
END;

CREATE TRIGGER reward_grant_ledger_origins_no_delete
BEFORE DELETE ON reward_grant_ledger_origins
BEGIN
    SELECT RAISE(ABORT, 'reward ledger origins are durable');
END;

UPDATE app_metadata
SET schema_generation = 30
WHERE singleton = 1;
