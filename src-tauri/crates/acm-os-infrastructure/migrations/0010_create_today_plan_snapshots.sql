ALTER TABLE problem_learning_states
ADD COLUMN pinned_priority INTEGER NOT NULL DEFAULT 0 CHECK (pinned_priority IN (0, 1));

CREATE TABLE today_plans (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    local_date TEXT NOT NULL UNIQUE,
    budget_minutes INTEGER NOT NULL CHECK (budget_minutes >= 0),
    planned_minutes INTEGER NOT NULL CHECK (planned_minutes >= 0),
    over_budget_minutes INTEGER NOT NULL CHECK (over_budget_minutes >= 0),
    review_only_streak INTEGER NOT NULL CHECK (review_only_streak BETWEEN 0 AND 2),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE today_plan_entries (
    id TEXT PRIMARY KEY CHECK (length(id) = 36),
    today_plan_id TEXT NOT NULL REFERENCES today_plans(id) ON DELETE RESTRICT,
    problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,
    review_attempt_id TEXT REFERENCES review_attempts(id) ON DELETE RESTRICT,
    lane TEXT NOT NULL CHECK (lane IN ('carry_in', 'review', 'study')),
    reason TEXT NOT NULL CHECK (reason IN (
        'continue_review',
        'continue_learning',
        'due_first_cold_start',
        'due_long_term_review',
        'relearn',
        'upsolve'
    )),
    planning_cost_minutes INTEGER NOT NULL CHECK (planning_cost_minutes IN (30, 60)),
    position INTEGER NOT NULL CHECK (position >= 0),
    entry_origin TEXT NOT NULL DEFAULT 'auto' CHECK (entry_origin IN ('auto', 'manual')),
    entry_status TEXT NOT NULL DEFAULT 'not_started'
        CHECK (entry_status IN ('not_started', 'in_progress', 'completed', 'unavailable')),
    reconciliation_added INTEGER NOT NULL DEFAULT 0 CHECK (reconciliation_added IN (0, 1)),
    UNIQUE (today_plan_id, problem_id),
    UNIQUE (today_plan_id, position),
    CHECK (reason != 'continue_review' OR (lane = 'carry_in' AND review_attempt_id IS NOT NULL))
);

CREATE INDEX today_plan_entries_by_plan
ON today_plan_entries(today_plan_id, position);

UPDATE app_metadata
SET schema_generation = 10
WHERE singleton = 1;
