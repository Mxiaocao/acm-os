# IMPLEMENTATION REPORT

Date: 2026-09-22

Result: PASS — implementation and automated acceptance completed in the working copy.

## Scope and source

- Working copy: `E:\03-Projects\acm-os\acm-os-vault-data-space`.
- Baseline: `E:\03-Projects\acm-os\acm-os`, `main@0d16e4f`.
- The task directory was empty. Source was exported from that commit into the task directory after a source-directory clarification was presented and the default working-copy approach was stated.
- The original source checkout was not modified. No real user database migration or desktop installation was performed.
- Existing frontend dependencies were reused through a local `node_modules` junction. No dependencies were added.
- `contest-problem-library.patch` contains the focused source/test/report changes. `git apply --check` against the original checkout passed; the patch has not been applied there.

## ARCHITECTURE

### Audit before implementation

1. Contest: `contests` holds metadata; `contest_external_identities` resolves external identities; `contest_problems` holds ordered associations, import status, and contest facts. Existing constraints already allow a problem to belong to multiple contests.
2. Problem: `problems` holds canonical metadata. `problem_external_identities` holds aliases. Statement snapshots/assets, learning states, personal notes, and review history are separate records.
3. Library: contest taxonomy uses `contest_families`, `contest_series`, `contest_years`, and `contest_placements`. There was no personal-library membership table.
4. Import: `ContestLibraryPage` / `ContestShelf` → TypeScript IPC → Tauri online/manual import commands → application import plan → infrastructure `ContestImportPort::persist_manifest` and snapshot persistence.
5. Root cause: `ProblemIndex` called `list_lightweight_problems`, which queried all Codeforces problems. Saving a contest problem therefore made it visible in the personal library. There was no separate automatic membership insertion to remove. Import also creates learning-state records needed by existing contest-detail and lifecycle reads; these records are not membership.
6. Existing migrations: `0003` contest/problem associations, `0005` learning lifecycle, `0025` contest taxonomy, `0026` external identities, `0027` alias uniqueness, `0034` contest years. Latest existing schema was 34.

### Implemented relationship

```text
Contest ── ContestProblem ── Problem ── ProblemLibraryItem
                              │               │
                         canonical ID    unique problem_id
```

- Added an application `ProblemLibraryPort`, implemented in existing infrastructure and exposed through vault-scoped IPC.
- `problem_library_items` is the explicit membership authority, independent of `identity_type` and personal-note creation.
- Import still persists every contest/problem association and available statement. It never inserts new personal-library membership.
- Personal-library reads require membership and return one entry per canonical problem even if it has multiple Codeforces aliases.
- Online and manual imports open an inline selection preview. Existing members are marked and disabled; new problems start unchecked. Batch add and cancel are supported, with retry and pending-submission protection.
- Contest details can reopen selection after cancellation.
- Adds validate contest ownership, follow the existing daily-backup mechanism, and use a transaction plus `ON CONFLICT(problem_id) DO NOTHING`. Invalid selections cannot partially add a batch.
- Contest deletion excludes library members from orphan cleanup.
- Existing domain models, learning-state semantics, routing, note storage, and import adapter were retained.

## FILES_CHANGED

| File | Change |
| --- | --- |
| `src-tauri/crates/acm-os-application/src/lib.rs` | Membership preview models and application port |
| `src-tauri/crates/acm-os-infrastructure/migrations/0035_create_problem_library_items.sql` | Add membership and preserve legacy records |
| `src-tauri/crates/acm-os-infrastructure/src/persistence.rs` | Membership persistence, filtered reads, deletion protection, schema validation, acceptance tests and schema-aware regression fixtures |
| `src-tauri/crates/acm-os-infrastructure/src/vault_manager/tests.rs` | Explicit selection in existing cross-vault isolation test |
| `src-tauri/src/ipc.rs` | Vault-scoped preview/add commands and imported contest ID in response |
| `src-tauri/src/lib.rs` | Register commands |
| `src/ipc/contest.ts` | Typed frontend IPC |
| `src/app/ProblemLibrarySelection.tsx` | Selection, membership status, submit/cancel/retry UI |
| `src/app/shells.tsx` | Import/detail entry points and personal-library empty-state explanation |
| `scripts/dom-shells.test.mjs` | Online/manual preview assertions, explicit batch and cancel coverage |
| `docs/CONTEST_PROBLEM_LIBRARY_IMPLEMENTATION_REPORT.md` | Audit, verification and delivery record |

## DATABASE_CHANGE

Read-only production inspection before implementation:

- Current registered database: `E:\05-Knowledge-Base\26_Aut\.acm-os\system-facts.sqlite3`.
- `app_metadata.schema_generation = 34`; migrations 1–34 all successful.
- Counts: 1 contest, 7 contest associations, 7 problems, 0 personal-note file bindings.
- All 7 problems had `identity_type = lightweight`.
- `PRAGMA quick_check` returned `ok`.

Migration 0035 adds:

```sql
problem_library_items (
    problem_id INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE RESTRICT,
    added_at_utc TEXT NOT NULL DEFAULT (...),
    origin TEXT NOT NULL CHECK (origin IN ('legacy', 'explicit'))
)
```

Existing problems are backfilled once with `origin = legacy` and their original creation timestamps. New user selections use `origin = explicit`. No old rows, learning history, notes, or contest associations are removed or rewritten by this migration. The existing startup migration mechanism creates its pre-migration backup and validates the new schema contract.

Compatibility exception: historical imports do not record selection intent. Preserving existing visibility means the current 7 problems remain historical library members after upgrade. They are not silently classified as explicit selections or deleted. The 7/0 acceptance case applies to a new contest in a fresh schema-35 test database.

## TEST_RESULT

| Acceptance | Observed result | Status |
| --- | --- | --- |
| Case 1: full import of A/B/C/D/E1/E2/F | Contest detail has 7; personal library has 0; preview shows 7 unselected non-members | PASS |
| Case 2: explicitly select A and C | Contest remains 7; library returns exactly A and C | PASS |
| Case 3: repeat import and duplicate selections | 1 contest, 7 associations, 2 membership rows; A remains one record | PASS |
| Case 4: personal library read | Only selected members returned on fresh schema; unselected B/D/E1/E2/F absent | PASS |
| Historical upgrade and restart | Schema 34 → 35 retains legacy problems and membership across restart | PASS |
| Concurrent adds through two contests sharing a problem | One membership and one library list entry | PASS |
| Invalid selection / empty selection | No partial membership write | PASS |
| Delete contest after selecting A/C | Unselected orphan problems cleaned; A/C remain in personal library | PASS |
| Preview selection/cancel | Existing entries disabled; only checked IDs submitted; cancel makes no write | PASS |
| Online/manual import UI | Opens selection preview, with no implicit selections | PASS |
| Data-space isolation | Separate memberships stay isolated across A/B switches and restart | PASS |

Commands and results:

- `cargo test --manifest-path src-tauri/Cargo.toml -p acm-os-infrastructure --offline --lib`: **261 passed, 2 ignored**. The ignored tests are existing real-Codeforces network smoke tests.
- `cargo test --manifest-path src-tauri/Cargo.toml -p acm-os-application -p acm-os-domain --offline`: **32 application + 5 application integration + 47 domain tests passed**.
- `node --test scripts/dom-shells.test.mjs`: **76 passed**.
- `node --test scripts/startup-shells.test.mjs`: **7 passed**.
- `node --import tsx --test scripts/vault-manager-ui.test.mjs`: **6 passed**.
- Boundary unit tests: **5 passed**; `node scripts/check-boundaries.mjs`: **passed**.
- `npm run build`: **passed** (TypeScript and Vite).
- `cargo check --manifest-path src-tauri/Cargo.toml -p acm-os --offline`: **passed**, including new desktop IPC.

Logs are in the working-copy root: `infrastructure-tests.log`, `domain-application-tests.log`, `dom-tests.log`, `startup-tests.log`, `vault-ui-tests.log`, `boundaries-tests.log`, `frontend-build.log`, `desktop-check.log`.

Initial broad regression failures were schema-34 expectations and old import-implies-membership fixtures; these were updated and the full suites passed. Sandbox-only Vite cache/tsx user-info failures were resolved by running the same build/tests with approved access. No approval review rejected an action.

## PASS/FAIL

**PASS for implementation and automated acceptance in this working copy.**

Limits: no real Codeforces network smoke or installed-desktop visual E2E run was performed. The production database remains schema 34 and untouched. Historical membership is deliberately preserved as described above. Vite still emits the existing large-bundle warning; this task does not change bundle architecture.
