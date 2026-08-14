# ACM-OS M8 Recovery Handoff

Updated: 2026-08-14

## Purpose

This file is the recovery handoff for the next Codex window. Read it before doing any further BUILD work.

## Authoritative phase

The project is at **M8 — Recovery / Backup / Diagnostics / Failure Hardening**.

M8 is complete and has been committed. Do not repeat M0–M7 and do not enter M9 or M10 unless the user explicitly changes scope.

## Repository checkpoint

- Repository: `E:\项目开发\acm-os`
- Branch: `main`
- HEAD: `8dd129e` — `Complete M8 recovery backup diagnostics hardening`
- Working tree: clean at the time this handoff was written
- No tag or push has been performed

## First recovery checks

Run these read-only checks first:

```powershell
Set-Location 'E:\项目开发\acm-os'
git status --short
git branch --show-current
git rev-parse HEAD
git log -10 --oneline --decorate
git diff --check
```

Then read, in full, and treat as frozen authority:

1. `ACM-OS_SPEC_v1.md`
2. `ACM-OS_DESIGN_v1.md`
3. `ACM-OS_PLAN_v1.md`
4. `ACM-OS_BUILD_HANDOFF.md`
5. `ACM-OS_RECOVERY_PROMPT.md`
6. this file

Authority order is `SPEC > DESIGN > PLAN > implementation`.

## M8 completion evidence

The completed M8 chain includes durable critical-operation journaling and startup recovery, Safe Patch recovery copies and crash handling, Problem/Knowledge anomaly repair, identity-conflict recovery, daily/weekly/manual backup, inventory and retention preview/apply, restore candidate validation, pre-restore snapshot, verified database swap, durable restore intent and startup consumption, restore diagnostics and rollback cleanup, post-restore binding validation, derived Knowledge rebuild preview/preconditions/explicit apply, system health aggregation, privacy-filtered diagnostic export, Recovery Shell export operations, adapter health projection, backup path-layout hardening, and Desktop E2E timeout/exit hardening.

Recorded verification:

- `cargo test --workspace`: 188 passed, 0 failed, 2 ignored
- `cargo check --workspace`: passed
- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed
- `npm.cmd run check:boundaries`: 5/5
- `npm.cmd run test:shells`: 6/6
- `npm.cmd run test:dom-shells`: 35/35
- `npm.cmd run build`: passed
- `npm.cmd run test:desktop-e2e`: passed

The two ignored Rust tests are release-only Codeforces network smoke tests.

## Files intentionally included in the M8 commit

The following migrations are real project files and must not be deleted or treated as disposable unknowns:

- `src-tauri/crates/acm-os-infrastructure/migrations/0021_create_critical_operations.sql`
- `src-tauri/crates/acm-os-infrastructure/migrations/0022_add_confirmed_deleted_knowledge_binding.sql`
- `src-tauri/crates/acm-os-infrastructure/migrations/0023_add_knowledge_rebuild_decision.sql`

## Safe continuation rules

- Preserve any new user changes or unknown files discovered after this checkpoint.
- Do not use `git reset --hard`, `git clean`, or overwrite-style checkout.
- Do not amend or rewrite commit `8dd129e` without explicit user authorization.
- Do not tag or push without explicit user authorization.
- If a future request is outside M8, stop and ask for scope confirmation.
- If a frozen document conflicts with implementation, report `SPEC-CONFLICT` and stop expansion.

## If the user asks for more work

Start with a fresh evidence-based audit of the requested change and its tests. Do not infer missing work from historical prompt text; the current repository and frozen documents are authoritative. Keep any new slice minimal and independently verifiable.

