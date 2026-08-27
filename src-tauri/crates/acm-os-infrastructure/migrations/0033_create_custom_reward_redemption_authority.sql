CREATE TABLE custom_reward_redemptions (
    redemption_id TEXT PRIMARY KEY CHECK (length(redemption_id) = 36),
    custom_reward_id TEXT NOT NULL
        REFERENCES custom_rewards(custom_reward_id) ON DELETE RESTRICT,
    coin_cost_paid INTEGER NOT NULL CHECK (
        typeof(coin_cost_paid) = 'integer'
        AND coin_cost_paid >= 1
        AND coin_cost_paid <= 9007199254740991
    ),
    redeemed_at_utc TEXT NOT NULL CHECK (length(redeemed_at_utc) = 24)
);

CREATE INDEX custom_reward_redemptions_by_reward
    ON custom_reward_redemptions (custom_reward_id, redeemed_at_utc, redemption_id);

CREATE TABLE custom_reward_refunds (
    refund_id TEXT PRIMARY KEY CHECK (length(refund_id) = 36),
    redemption_id TEXT NOT NULL UNIQUE
        REFERENCES custom_reward_redemptions(redemption_id) ON DELETE RESTRICT,
    refunded_at_utc TEXT NOT NULL CHECK (length(refunded_at_utc) = 24)
);

CREATE TABLE custom_reward_redemption_ledger_origins (
    ledger_entry_id TEXT PRIMARY KEY,
    redemption_id TEXT NOT NULL UNIQUE
        REFERENCES custom_reward_redemptions(redemption_id) ON DELETE RESTRICT,
    resource_kind TEXT NOT NULL CHECK (resource_kind = 'coin'),
    FOREIGN KEY (ledger_entry_id, resource_kind)
        REFERENCES reward_ledger_entries(id, resource_kind) ON DELETE RESTRICT
);

CREATE TABLE custom_reward_refund_ledger_origins (
    ledger_entry_id TEXT PRIMARY KEY,
    refund_id TEXT NOT NULL UNIQUE
        REFERENCES custom_reward_refunds(refund_id) ON DELETE RESTRICT,
    resource_kind TEXT NOT NULL CHECK (resource_kind = 'coin'),
    FOREIGN KEY (ledger_entry_id, resource_kind)
        REFERENCES reward_ledger_entries(id, resource_kind) ON DELETE RESTRICT
);

CREATE TRIGGER custom_reward_redemptions_no_update
BEFORE UPDATE ON custom_reward_redemptions
BEGIN
    SELECT RAISE(ABORT, 'custom reward redemptions are immutable');
END;

CREATE TRIGGER custom_reward_redemptions_no_delete
BEFORE DELETE ON custom_reward_redemptions
BEGIN
    SELECT RAISE(ABORT, 'custom reward redemptions are durable');
END;

CREATE TRIGGER custom_reward_refunds_no_update
BEFORE UPDATE ON custom_reward_refunds
BEGIN
    SELECT RAISE(ABORT, 'custom reward refunds are immutable');
END;

CREATE TRIGGER custom_reward_refunds_no_delete
BEFORE DELETE ON custom_reward_refunds
BEGIN
    SELECT RAISE(ABORT, 'custom reward refunds are durable');
END;

CREATE TRIGGER custom_reward_redemption_origins_insert_guard
BEFORE INSERT ON custom_reward_redemption_ledger_origins
WHEN NOT EXISTS (
    SELECT 1
    FROM reward_ledger_entries entry
    JOIN custom_reward_redemptions redemption
      ON redemption.redemption_id = NEW.redemption_id
    WHERE entry.id = NEW.ledger_entry_id
      AND entry.resource_kind = 'coin'
      AND entry.delta = -redemption.coin_cost_paid
)
OR EXISTS (
    SELECT 1 FROM reward_grant_ledger_origins origin
    WHERE origin.ledger_entry_id = NEW.ledger_entry_id
)
OR EXISTS (
    SELECT 1 FROM custom_reward_refund_ledger_origins origin
    WHERE origin.ledger_entry_id = NEW.ledger_entry_id
)
BEGIN
    SELECT RAISE(ABORT, 'invalid custom reward redemption ledger origin');
END;

CREATE TRIGGER custom_reward_redemption_origins_no_update
BEFORE UPDATE ON custom_reward_redemption_ledger_origins
BEGIN
    SELECT RAISE(ABORT, 'reward ledger origins are immutable');
END;

CREATE TRIGGER custom_reward_redemption_origins_no_delete
BEFORE DELETE ON custom_reward_redemption_ledger_origins
BEGIN
    SELECT RAISE(ABORT, 'reward ledger origins are durable');
END;

CREATE TRIGGER custom_reward_refund_origins_insert_guard
BEFORE INSERT ON custom_reward_refund_ledger_origins
WHEN NOT EXISTS (
    SELECT 1
    FROM reward_ledger_entries entry
    JOIN custom_reward_refunds refund ON refund.refund_id = NEW.refund_id
    JOIN custom_reward_redemptions redemption
      ON redemption.redemption_id = refund.redemption_id
    WHERE entry.id = NEW.ledger_entry_id
      AND entry.resource_kind = 'coin'
      AND entry.delta = redemption.coin_cost_paid
)
OR EXISTS (
    SELECT 1 FROM reward_grant_ledger_origins origin
    WHERE origin.ledger_entry_id = NEW.ledger_entry_id
)
OR EXISTS (
    SELECT 1 FROM custom_reward_redemption_ledger_origins origin
    WHERE origin.ledger_entry_id = NEW.ledger_entry_id
)
BEGIN
    SELECT RAISE(ABORT, 'invalid custom reward refund ledger origin');
END;

CREATE TRIGGER custom_reward_refund_origins_no_update
BEFORE UPDATE ON custom_reward_refund_ledger_origins
BEGIN
    SELECT RAISE(ABORT, 'reward ledger origins are immutable');
END;

CREATE TRIGGER custom_reward_refund_origins_no_delete
BEFORE DELETE ON custom_reward_refund_ledger_origins
BEGIN
    SELECT RAISE(ABORT, 'reward ledger origins are durable');
END;

CREATE TRIGGER reward_grant_ledger_origins_exactly_one_insert_guard
BEFORE INSERT ON reward_grant_ledger_origins
WHEN EXISTS (
    SELECT 1 FROM custom_reward_redemption_ledger_origins origin
    WHERE origin.ledger_entry_id = NEW.ledger_entry_id
)
OR EXISTS (
    SELECT 1 FROM custom_reward_refund_ledger_origins origin
    WHERE origin.ledger_entry_id = NEW.ledger_entry_id
)
BEGIN
    SELECT RAISE(ABORT, 'reward ledger entry already has another origin family');
END;

UPDATE app_metadata
SET schema_generation = 33
WHERE singleton = 1;
