# ACM-OS Durable Recovery Prompt — M8 Active

将下面代码块中的全部内容复制到下一个 Codex Local 窗口。

```text
请接管 E:\项目开发\acm-os 的后续 BUILD 工作。

当前阶段必须明确为：

M8 — Recovery / Backup / Diagnostics / Failure Hardening

当前不是 M8 起点。M8 已完成约三分之二，不得重复已经闭环的 Slice。已完成：Critical Operation durable journal/startup gate、Safe Patch crash matrix、Problem/Knowledge Location Anomaly 显式修复、Knowledge 同名重建身份选择、手动一致性备份、backup inventory/retention preview、自动 Daily Snapshot 基础。

Daily Snapshot 已接入：`save_weekly_acm_budget`、`confirm_knowledge_understanding`、`register_knowledge_candidate`、`set_knowledge_candidate_disposition`、`save_contest_ai_analysis`、`set_contest_archived`、`correct_contest_problem_facts`、`update_problem_mastery_evidence`、`complete_contest_facts`、`delete_contest`、`commit_problem_lifecycle_decision`、`create_or_resume_review_attempt`、`reveal_review_help`、`commit_review_completion`、`void_review_attempt`、`prepare_personal_note_deletion`、`commit_personal_note_binding`。

最新闭环 Slice 是 `commit_personal_note_binding`：首次 lightweight → personal binding commit 先在备份前验证 Problem 状态，已有 Personal binding 直接幂等返回且不重复备份；合法创建在数据库 binding/identity 写入前生成或复用当天 pre-mutation 一致性快照。测试确认快照中 identity 仍为 lightweight、file binding 数为 0，第二次幂等创建复用同一快照；测试夹具会隔离 setup-only 创建快照，避免污染业务 mutation 断言。最新证据为 Infrastructure `162 passed / 0 failed / 2 ignored`、Startup `6/6`、DOM `35/35`、Boundary `5/5`，Rust workspace/check/fmt、TypeScript、Vite build、Desktop E2E 和 `git diff --check` 均通过。

恢复核对后，下一步先重新盘点剩余文件/数据库写入口及其 Critical Operation、recovery copy、Daily Snapshot 覆盖，确认真实缺口后再选择一个最小 Slice。不得直接铺开 restore、diagnostics 或完整 Recovery Shell。

M8 的核心目标是：异常情况下保护已知事实，不猜、不静默损坏。它不是新增普通业务功能的阶段。

不要依赖旧聊天记忆，也不要把本提示中的预期值直接当作事实。必须以真实 Windows 仓库、Git 历史、冻结文档、现有 diff、测试和实际命令输出为准。

第一阶段只做恢复核对，不修改任何文件。

先运行：

Get-Location
git status --short
git branch --show-current
git log -10 --oneline --decorate
git remote -v
git tag --list
git tag --points-at HEAD
git rev-parse HEAD
git rev-parse origin/main
git show --stat --oneline HEAD
git rev-list --left-right --count origin/main...HEAD
git log --oneline --reverse origin/main..HEAD
git diff --stat
git diff --check
git diff --cached --stat
git diff --cached --check

然后从头到尾完整读取：

1. ACM-OS_SPEC_v1.md
2. ACM-OS_DESIGN_v1.md
3. ACM-OS_PLAN_v1.md
4. ACM-OS_BUILD_HANDOFF.md
5. ACM-OS_RECOVERY_PROMPT.md

权威顺序严格为：

SPEC > DESIGN > PLAN > IMPLEMENTATION

保护规则：

- 不得执行 git reset --hard、git clean 或覆盖式 checkout。
- 不得丢弃任何现有未提交修改或未知文件。
- 未经用户明确允许，不得 commit、tag 或 push。
- 不得修改 SPEC、DESIGN 或 PLAN，除非用户明确授权。
- 不得重复实现 M0–M7。
- 不得进入 M9 或 M10。
- 不得把整个 M8 一次性铺开。
- 当前 Slice 失败时不得叠加下一个 Slice。
- 若实现与冻结文档存在真实冲突，标记 SPEC-CONFLICT 并停止扩展。

交接时已知 checkpoint 仅供核对：

- branch: main
- HEAD: 60fe56b11bd3d541768ffa96aeca9596e4bb05e6
- subject: feat: complete M7 contest workflow
- origin/main: 61726681f26e13a03f5f601643a9d18188faf9e8
- main 相对 origin/main ahead 5
- ahead 5 是连续的 M3、M4、M5、M6、M7 本地 checkpoint，不是异常分叉
- 这些提交尚未 push
- M3–M7 未创建 tag
- M7 产品实现已提交
- .gitignore、ACM-OS_BUILD_HANDOFF.md、ACM-OS_RECOVERY_PROMPT.md 可能存在尚未提交的交接更新，必须以实际 status/diff 为准，不得丢弃

预期的 5 个 ahead commits：

22caa0e build: complete M3 learning lifecycle
e9dd877 build: complete M4 review lifecycle
5d3afe6 build: complete M5 daily planning
55a728c build: complete M6 knowledge system
60fe56b feat: complete M7 contest workflow

M0–M7 已完成。M7 包括：

- Facts Snapshot、Unknown semantics、upsolve decision；
- Contest Result 与实时 Learning Status 分离；
- Correction Event；
- Post-Contest AI Analysis raw/preview/complete/partial/failed；
- Manual Contest/Problem/Statement；
- archive/restore/delete；
- consequence preview 与 safe lightweight cleanup；
- schema 16–20；
- Contest 页面响应式布局；
- Rust、DOM、boundary、TypeScript、Vite、Desktop E2E 和用户人工 GUI 验收。

M7 最终自动化证据包括：

- Infrastructure: 117 passed / 2 ignored
- DOM: 29 passed
- Boundary: 5 passed
- cargo check/fmt、TypeScript、Vite build、Desktop E2E passed
- 两项 ignored 是 release-only 真实 Codeforces 网络 smoke

不要重复实现 M7。若实际证据发现回归，只做范围最小的回归修复并重新验证。

冻结 PLAN 中下一阶段只能是：

M8 — Recovery / Backup / Diagnostics / Failure Hardening

冻结 Outcome：

异常情况下保护已知事实，不猜、不静默损坏。

冻结范围：

- Health
- Location Anomaly repair
- binding recovery
- parse/concurrency UX
- Critical Operation recovery
- crash check
- backup/retention/restore
- derived rebuild
- logs
- diagnostic export preview
- external-open failure
- adapter health
- 完整 Recovery Shell

DoD：

fault injection / backup restore / ambiguous relocation / crash matrix 全部有证据。

恢复核对完成后，不要立刻大规模编码。先完成以下工作：

1. 从 SPEC、DESIGN、PLAN 提取 M8 的权威合同、失败语义和 DoD。
2. 盘点现有代码已经具备的基础：
   - Startup Gate / Recovery routing
   - schema validation 与 migration backup
   - Safe Patch recovery copy
   - Fresh Read / concurrency rejection
   - deterministic relocation / ambiguity rejection
   - Vault unavailable 降级
   - external-open failure
   - derived Knowledge index rebuild
3. 明确这些基础与 M8 冻结范围之间的真实缺口。
4. 把 M8 划分为独立可验证 Slices。
5. 选择第一个最小 Slice，说明：
   - 权威合同；
   - 包含范围；
   - 明确不包含的后续范围；
   - 数据和失败语义；
   - focused test；
   - 完整门禁。
6. 先向用户报告恢复核对和 M8 planning，再开始第一个最小 Slice。

第一个 Slice 必须独立可验证。优先选择能建立 M8 恢复基础、同时不扩展到完整 Recovery Shell 或整个 backup/restore 系统的最小闭环；最终选择必须由冻结合同和现有代码盘点决定，不能凭提示词猜测。

执行节奏：

implementation/fix

Latest completed M8 Slice: `confirm_personal_note_deleted`. Daily Snapshot coverage now includes this mutation (18 total). The next implementation window must begin with a fresh inventory of remaining file/database write paths and their Critical Operation, recovery-copy, and Daily Snapshot coverage; select only one minimal independently verifiable Slice after that audit.
Additional Daily Snapshot mutation: `confirm_personal_note_deleted` (18 total).
Latest implementation Slice: `knowledge_rebuild_with_existing_bindings_uses_a_daily_backup_boundary`; existing Knowledge bindings are backed up before derived index rebuild mutation, while an initial empty index remains backup-free. Infrastructure evidence: `163 passed / 0 failed / 2 ignored`.

Latest completed M8 Slice: `durable_restore_intent_preparation`. The formal Application restore contract now validates a candidate, creates a verified pre-restore snapshot, publishes a verified staging copy under `backups/pre-restore`, and atomically writes `restore-intent.json`; pending intents cannot be overwritten. Startup consumes the intent before opening SQLite and routes failures to Recovery while retaining the intent. Focused evidence: 2/2; latest Infrastructure evidence: `175 passed / 0 failed / 2 ignored`.

The next implementation window must not repeat candidate preview, pre-restore snapshot, verified swap, or durable intent preparation. The next minimal Slice is the external controlled restart/restore orchestration boundary. Do not add migration, post-restore rebuild, Markdown/binding repair, or Recovery Shell UI in that Slice.

Latest completed M8 Slice: `controlled_restore_preparation_ipc`. Tauri now exposes `prepare_system_restore`, returning the durable staging path, pre-restore snapshot path, and candidate preview. It only persists the intent; it does not replace the live pool or silently restart the process. Rust workspace evidence remains `177 passed / 0 failed / 2 ignored`; frontend build was not runnable because Node is absent from PATH.

Next Slice: explicit restart handoff and post-startup status reporting only. Keep migration, post-restore rebuild, rollback cleanup, and Markdown/binding repair out of scope.

Latest completed M8 Slice: `explicit_restore_restart_handoff`. Tauri exposes `restart_for_pending_restore`; it refuses to restart without a durable intent and otherwise requests a controlled app restart. Startup remains the sole consumer of the intent before opening SQLite. Tauri library evidence: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: read-only startup restore outcome diagnostics only. Do not add migration, post-restore rebuild, rollback cleanup, or Markdown/binding repair.

Latest completed M8 Slice: `read_only_restore_outcome_diagnostics`. `restore_diagnostics` reports pending intent, rollback artifact path, and startup state without mutating files or SQLite. Focused restore evidence: `2 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: explicit user-confirmed rollback artifact cleanup only. Preserve failure recoverability and keep migration, post-restore rebuild, Markdown/binding repair, and full Recovery UI out of scope.

Latest completed M8 Slice: `explicit_rollback_artifact_cleanup`. Cleanup requires exact artifact path matching, no pending intent, regular-file validation, and read-only integrity verification before deletion. Focused evidence: `1 passed / 0 failed`; workspace check and formatting pass.

Next Slice: read-only post-restore schema/integrity outcome projection only. Do not add automatic retention cleanup, migration, post-restore rebuild, Markdown/binding repair, or Recovery UI.

Latest completed M8 Slice: `post_restore_schema_integrity_projection`. Restore diagnostics now reports startup state, current schema version, pending intent, rollback artifact path, and read-only rollback integrity outcome. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: preview/plan for post-restore derived-state rebuild only. Do not execute rebuild, modify Markdown, or add migration/Recovery UI.

Latest completed M8 Slice: `post_restore_rebuild_preview`. The read-only preview reports problem bindings, knowledge bindings, existing derived relations, and explicitly states that apply will revalidate bindings/rebuild derived Knowledge without overwriting Markdown. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: the first post-restore binding-validation segment only. Preserve anomalies as explicit results and do not execute derived rebuild in the same Slice.

Latest completed M8 Slice: `post_restore_problem_binding_validation`. Problem bindings are re-resolved read-only; ready totals and explicit `location_anomaly`, `vault_unavailable`, and `invalid_binding` results are returned without updating binding state or Markdown. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: post-restore Knowledge binding validation only. Do not execute derived rebuild or modify Markdown.

Latest completed M8 Slice: `post_restore_knowledge_binding_validation`. Knowledge bindings are re-discovered and checked read-only; ready, confirmed-deleted, and explicit `location_anomaly` results are returned without state updates, derived rebuild, or Markdown writes. Focused evidence: `1 passed / 0 failed`; workspace check and formatting pass.

Next Slice: explicit precondition checking before any derived Knowledge rebuild apply. Do not execute rebuild automatically.

Latest completed M8 Slice: `derived_rebuild_preconditions`. The read-only precondition check aggregates pending intent, startup recovery, Problem binding anomalies, and Knowledge binding anomalies, returning `eligible` only when all blockers are absent. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: explicit user-confirmed derived Knowledge rebuild apply only. Keep automatic execution disabled.

Latest completed M8 Slice: `explicit_derived_rebuild_apply`. The apply IPC rechecks every precondition and refuses pending intent, startup recovery, or binding anomalies before invoking the existing Knowledge rebuild. It returns node/relation/anomaly totals and never writes Markdown. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: audit final M8 DoD and remaining diagnostics/UI gaps. Do not enter M9.

M8 final DoD audit result: the core recovery chain is closed through explicit derived rebuild apply, but M8 is not complete. Remaining frozen-scope gaps are Weekly Snapshot generation, real retention apply, aggregated System Health/adapter health, logs and diagnostic export preview, unified Recovery UX for parse/concurrency/external-open failures, complete Recovery Shell, and final concentrated fault-injection/backup-restore/ambiguous-relocation/crash-matrix evidence.

Next Slice: read-only System Health aggregation only. Do not add export, retention mutation, or Recovery UI in that Slice; do not enter M9.

Latest completed M8 Slice: `read_only_system_health_aggregation`. `system_health_snapshot` aggregates startup/schema state, pending or needs-recovery Critical Operations, published backup count, pending restore intent, and rollback integrity without mutations. Focused evidence: `1 passed / 0 failed`; workspace check and formatting pass.

Next Slice: diagnostic export preview only. Do not add retention mutation or Recovery UI.

Latest completed M8 Slice: `diagnostic_export_preview`. It declares output directory, diagnostic sections, and privacy exclusions with `createsFiles: false`; no directory or artifact is created, and Markdown/statement content, credentials, and absolute workspace paths are excluded. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: Weekly Snapshot generation boundary only. Keep retention apply and Recovery UI separate.

Latest completed M8 Slice: `weekly_snapshot_boundary`. Explicit `create_weekly_backup` publishes a SQLite-consistent, integrity-verified `weekly-` snapshot under `backups/weekly`; no retention pruning or UI was added. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: weekly/daily retention apply preview and policy checks only.

Latest completed M8 Slice: `backup_retention_preview`. It returns protected and prune-candidate paths under the frozen `7 Daily + 4 Weekly` policy without deletion or backup-directory creation. Focused evidence: `1 passed / 0 failed`; Tauri library: `23 passed / 0 failed`; workspace check and formatting pass.

Next Slice: explicit user-confirmed retention apply with exact preview paths only; no broad deletion.
The next completed repair boundary is `confirm_knowledge_markdown_deleted`: anomaly confirmation now snapshots the pre-mutation binding state before marking it confirmed deleted. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest repair Slice is Personal Note `rebind_personal_note`: candidate occupancy is checked before backup, then the anomaly binding is snapshotted before rebind commit. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest repair Slice is now Knowledge `rebind_knowledge_node`: candidate occupancy is checked before backup, then the anomaly binding is snapshotted before rebind commit. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest repair Slice is now `resolve_knowledge_identity_conflict`: candidate occupancy is checked before backup, then the confirmed-deleted identity transition is snapshotted before commit. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest diagnostic-state Slice is `update_binding_state`: location-anomaly transitions now use a pre-mutation Daily Snapshot, while external-source-unavailable diagnostics remain non-destructive and backup-free. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `reorder_today_snapshot`: invalid permutations are rejected before backup; a valid complete same-plan reorder is snapshotted and revalidated inside the write transaction. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `complete_today_entry`: invalid or review-owned entries are rejected before backup, already-completed entries are idempotent and backup-free, and a real learning-entry completion is snapshotted then transactionally revalidated. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `apply_today_replan`: stale or semantically tampered previews are rejected before backup; a valid replan is snapshotted and fully revalidated inside the write transaction. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `add_manual_today_entry`: stale or illegal acceptance is rejected in a no-transaction preflight; a valid acceptance is snapshotted, then fully revalidated inside the write transaction. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `create_or_load_today_snapshot`: an existing date is loaded idempotently without backup; first creation of a date is snapshotted, while the unique-date transaction safely resolves concurrent creators. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `reconcile_today_snapshot`: a read-only preflight detects real Review/binding/carry-in/position/summary changes; unchanged loads remain backup-free, while real derived writes are snapshotted before the original transaction. Infrastructure evidence remains `163 passed / 0 failed / 2 ignored`.
Audit decision: initial/progressive Contest `persist_manifest` is a bootstrap acquisition path, not a Daily Snapshot mutation. A trial backup boundary was reverted after five established business-mutation precondition tests exposed the contract mismatch; the restored Infrastructure gate is `163 passed / 0 failed / 2 ignored`.
Latest completed restore Slice: `preview_system_restore_candidate`. It canonicalizes candidates into App Private Data `backups`, accepts only published `.sqlite3` files from known backup categories, opens read-only, verifies integrity/schema, rejects future schema, reports migration for older schema, and explicitly states that System Facts restore will not overwrite Markdown. It does not create a pre-restore backup or execute restore. Evidence: focused `5/5`, Infrastructure `168 passed / 0 failed / 2 ignored`, Rust workspace/check/fmt, Startup `6/6`, DOM `35/35`, Boundary `5/5`, TypeScript and Vite build passed. The next Slice must remain separate and should begin with the pre-restore backup boundary before any database replacement.
Latest completed restore Slice: `create_pre_restore_snapshot`. It revalidates the candidate, creates and verifies a SQLite-consistent snapshot of the current live System Facts under `backups/pre-restore`, publishes only after integrity verification, and fails with `pre_restore_backup_failed` without changing the current database. It does not replace the database, migrate, rebuild projections, write Markdown, or expose IPC/UI. Evidence: focused `3/3`, Infrastructure `171 passed / 0 failed / 2 ignored`, Rust workspace/check/fmt passed. The next Slice must keep controlled database replacement and rollback separate from migration and post-restore rebuild.
Latest completed restore Slice: `swap_verified_database_with_staging`. It requires verified current/staging/pre-restore files, refuses current SQLite WAL/SHM busy state, moves the current database to a retained rollback path, then publishes staging. It fails closed before any rename when the pre-restore snapshot is missing or inputs are invalid. The primitive is not yet wired to Tauri runtime restart/IPC and does not perform migration, post-restore validation, derived rebuild, Markdown work, or rollback commit/cleanup. Evidence: focused `2/2`, Infrastructure `173 passed / 0 failed / 2 ignored`, Rust workspace/check/fmt passed. The next Slice must wire runtime connection shutdown/restart and failure rollback only.

M9 is now active by explicit user direction. Latest completed M9 Slice: `native_control_focus_indicator_coverage`.
The shared `:focus-visible` rule now covers native buttons, links, text inputs, textareas, and selects.
Added `scripts/accessibility-css.test.mjs` and the `test:accessibility` package script.
Evidence: accessibility `1 passed / 0 failed`; DOM shells `35 passed / 0 failed`; startup shells
`6 passed / 0 failed`; boundaries `5 passed / boundary check passed`; Vite/TypeScript build passed;
`git diff --check` passed. This slice is limited to accessibility focus visibility and does not enter M10.

Latest completed M9-A Slice: `reduced_motion_preference`. `src/app/app.css` now honors
`prefers-reduced-motion: reduce` by disabling non-essential animation/transition duration and using
immediate scrolling. Evidence: accessibility `2 passed / 0 failed`; DOM shells `35 passed / 0 failed`;
Vite/TypeScript build passed; `git diff --check` passed. No business behavior changed and M10 was not touched.

M9-A is now complete as a consolidated stage. It includes native focus indicators, reduced-motion
handling, keyboard-equivalent Today reorder, Today replan dialog focus/Tab/Escape behavior and
focus return, async Today mutation announcements, Review help dialog description semantics and
hidden-content isolation, plus narrow-viewport fallbacks for 200% zoom/core keyboard use.
Final evidence: accessibility `3 passed / 0 failed`; DOM shells `35 passed / 0 failed`; startup shells
`6 passed / 0 failed`; boundaries `5 passed / boundary check passed`; Vite/TypeScript build passed;
`git diff --check` passed. M9-B/M9-C/M9-D and M10 remain untouched.

M9-B is now complete as a consolidated stage. Tauri production and E2E configs use restrictive CSP
policies instead of `csp: null`; statement rendering rejects non-HTTPS links after trimming and keeps
unsafe schemes inert. Evidence: accessibility `4 passed / 0 failed`; DOM shells `35 passed / 0 failed`;
startup shells `6 passed / 0 failed`; boundaries `5 passed / boundary check passed`; Vite/TypeScript
build passed; `cargo check --workspace` passed; rustfmt passed; `git diff --check` passed. M9-C/M9-D
and M10 remain untouched.

M9-C is now complete as a consolidated stage. Added the deterministic Reference Dataset benchmark
harness `scripts/performance-benchmark.ts` and `benchmark:performance`. It covers the frozen
2,000/1,000/300/10,000/1,000/20,000 dataset sizes and enforces DESIGN P95 budgets for startup
projection, Today open, Knowledge search, Markdown parsing, relation projection, and local
navigation. Latest run passed all six budgets: 2.84ms, 0.23ms, 0.92ms, 0.38ms, 1.58ms, and 0.58ms
respectively. M9-D and M10 remain untouched.
→ focused tests
→ infrastructure tests
→ workspace Rust tests
→ cargo check --workspace
→ cargo fmt --all -- --check
→ Startup/DOM/boundary（按 change surface）
→ TypeScript
→ Vite production build
→ Desktop E2E（按 change surface）
→ git diff --check
→ 完整 diff review
→ git status

每次报告必须区分：

- 已由实际命令证明的事实；
- 仍缺少的证据；
- 本次修改的文件；
- 剩余 M8 Slices；
- 是否有未提交、未暂存或未知文件；
- 是否触及 M9/M10。

最终不得声称完成或通过，除非有对应实际命令输出、测试证据和必要的人工验收。
```

建议在仓库外另存一份本提示。只有文档被有意 commit 并 push 后，远端 Git 才能承担恢复作用。

M9-D is complete. Visual consistency hardening added shared danger/button-row/error/status
styles, normalized loading/empty/panel/modal surfaces, and narrow-viewport action stacking.
Added `scripts/visual-consistency.test.mjs` with `test:visual-consistency`.

M9 Final Gate evidence: visual consistency `6/6`; accessibility `4/4`; DOM `35/35`; startup
`6/6`; boundaries `5/5`; TypeScript/Vite build passed; performance benchmark passed all six
P95 budgets; Rust workspace `188 passed / 0 failed / 2 ignored`; cargo check and rustfmt passed;
`git diff --check` passed. M9-A through M9-D are complete. M10 has not been entered.

Remaining limitation: the performance result is a deterministic Node reference-data benchmark,
not a release Tauri cold-start/RAM measurement. Human visual acceptance on target desktop and
narrow/zoomed layouts is the only remaining optional confirmation.

M9 manual acceptance was completed in the real Tauri desktop window and marked qualified by the
user. Accepted surfaces include Today, Today replan, Knowledge, Contest shelf/detail/Facts/AI form,
delete consequence preview, and Problem Detail. Acceptance found and fixed three presentation
defects: oversized route-heading focus rings, two-column CSS leaking into item lists, and weak
Codeforces statement hierarchy. The statement hierarchy fix has automated coverage but its second
manual recheck was explicitly skipped by the user.

All current M9 changes were committed on `main` with subject `build: complete M9 hardening` after
explicit user authorization and a successful cached diff check. On recovery, do not trust a stale
hash copied from prose: rerun branch, HEAD, status, log, remote, and `git diff --check`, then report
the actual repository state. Do not push or tag without separate explicit authorization. Do not
enter M10 automatically.
