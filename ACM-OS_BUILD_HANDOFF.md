# ACM-OS BUILD Handoff — M8 Active

Updated: 2026-08-14 (Asia/Shanghai)

本文档用于下一个 Codex BUILD 窗口恢复上下文。它不替代 Git、冻结文档或实际命令输出。

## 0. M8 live checkpoint

当前阶段严格为：

```text
M8 — Recovery / Backup / Diagnostics / Failure Hardening
```

M0–M7 已完成，不得重复；不得进入 M9/M10。M8 当前约完成三分之二，仍按一个最小、独立、可验证 Slice 的节奏推进。

已闭环的 M8 基础包括：Critical Operation durable journal/startup gate、Safe Patch crash matrix、Problem/Knowledge Location Anomaly 显式修复、Knowledge 同名重建身份选择、手动一致性备份、备份 inventory/retention preview，以及自动 Daily Snapshot 基础。

Daily Snapshot 已接入以下 mutation：

1. `save_weekly_acm_budget`
2. `confirm_knowledge_understanding`
3. `register_knowledge_candidate`
4. `set_knowledge_candidate_disposition`
5. `save_contest_ai_analysis`
6. `set_contest_archived`
7. `correct_contest_problem_facts`
8. `update_problem_mastery_evidence`
9. `complete_contest_facts`
10. `delete_contest`
11. `commit_problem_lifecycle_decision`
12. `create_or_resume_review_attempt`
13. `reveal_review_help`
14. `commit_review_completion`
15. `void_review_attempt`
16. `prepare_personal_note_deletion`
17. `commit_personal_note_binding`
18. `confirm_personal_note_deleted`

Latest completed M8 Slice: `confirm_personal_note_deleted`.
It verifies the binding is missing, validates current DB/lifecycle/review state before backup, reuses or creates the current-day SQLite-consistent pre-mutation snapshot, then performs the downgrade and deletion inside the transaction. The backup retains `identity_type = personal` and the existing file binding; unrelated candidate Markdown remains untouched.

Latest implementation Slice: `knowledge_rebuild_with_existing_bindings_uses_a_daily_backup_boundary`. Knowledge index rebuild now creates or reuses the current-day SQLite-consistent backup before mutating existing bindings/index state; initial empty-index creation remains backup-free. Infrastructure verification is `163 passed / 0 failed / 2 ignored`.

The next completed repair boundary is `confirm_knowledge_markdown_deleted`: an anomaly-to-confirmed-deleted transition now creates or reuses the pre-mutation Daily Snapshot before changing binding state. The same Infrastructure gate remains `163 passed / 0 failed / 2 ignored`.

The latest repair Slice is Personal Note `rebind_personal_note`: candidate occupancy is rejected before backup, then the existing anomaly binding is backed up before rebind commit. Focused and Infrastructure tests pass; M8 remains active.
The latest repair Slice is now Knowledge `rebind_knowledge_node`: candidate occupancy is rejected before backup, then the anomaly binding is backed up before rebind commit. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest repair Slice is now `resolve_knowledge_identity_conflict`: candidate occupancy is rejected before backup, then the confirmed-deleted identity transition is backed up before commit. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest diagnostic-state Slice is `update_binding_state`: location-anomaly transitions now use a pre-mutation Daily Snapshot; external-source-unavailable diagnostics remain non-destructive and do not create a backup. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `reorder_today_snapshot`: invalid permutations are rejected before backup; a valid complete same-plan reorder creates or reuses the pre-mutation Daily Snapshot and is revalidated in the write transaction. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `complete_today_entry`: invalid or review-owned entries are rejected before backup, already-completed entries are idempotent and backup-free, and a real learning-entry completion is snapshotted then revalidated in the transaction. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `apply_today_replan`: stale or semantically tampered previews are rejected before backup; a valid replan creates or reuses the pre-mutation Daily Snapshot and is fully revalidated inside the write transaction. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `add_manual_today_entry`: stale or illegal acceptance is rejected in a no-transaction preflight; a valid acceptance is snapshotted, then the complete snapshot and candidate contract are revalidated inside the write transaction. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `create_or_load_today_snapshot`: an existing date is loaded idempotently without backup; first creation of a date creates or reuses the pre-mutation Daily Snapshot, while the unique-date transaction still resolves concurrent creators safely. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
The latest Today Slice is `reconcile_today_snapshot`: a read-only preflight detects only real reconciliation changes (Review state, binding availability, carry-ins, positions, or plan summaries); unchanged loads remain backup-free, while real derived writes create or reuse the pre-mutation Daily Snapshot before the original transaction. Infrastructure remains `163 passed / 0 failed / 2 ignored`.
Audit decision: initial/progressive Contest `persist_manifest` remains a bootstrap acquisition path, not a Daily Snapshot mutation. A trial backup boundary was reverted after it polluted five established business-mutation precondition tests; the restored gate is `163 passed / 0 failed / 2 ignored`. Re-import drift remains rejected and first snapshots remain immutable.

最新闭环 Slice 是 `commit_personal_note_binding`：首次 lightweight → personal binding commit 先在备份前验证 Problem 状态，已有 Personal binding 直接幂等返回且不重复备份；合法创建在数据库 binding/identity 写入前生成或复用当天 pre-mutation SQLite-consistent snapshot。测试确认快照中 identity 仍为 lightweight、file binding 数为 0，第二次幂等创建复用同一快照；测试夹具会隔离 setup-only 创建快照，避免污染业务 mutation 断言。

最新自动化证据：Infrastructure `162 passed / 0 failed / 2 ignored`；Startup `6/6`；DOM `35/35`；Boundary `5/5`；Rust workspace/check/fmt、TypeScript、Vite build、Desktop E2E、`git diff --check` 全部通过。两项 ignored 仍是 release-only 联网 Codeforces smoke。

下一步不得直接铺开 restore/diagnostics。继续盘点剩余文件/数据库写入口及其 Critical Operation、recovery copy、Daily Snapshot 覆盖；本轮已确认 Knowledge derived rebuild 的边界，后续只从仍未覆盖的写入口中选择一个最小 Slice。

## 1. 权威顺序

```text
SPEC > DESIGN > PLAN > IMPLEMENTATION
```

开始修改前必须完整读取：

1. `ACM-OS_SPEC_v1.md`
2. `ACM-OS_DESIGN_v1.md`
3. `ACM-OS_PLAN_v1.md`
4. `ACM-OS_BUILD_HANDOFF.md`
5. `ACM-OS_RECOVERY_PROMPT.md`

BUILD 期间不得修改 SPEC、DESIGN、PLAN，除非用户明确授权。若实现与冻结文档真实冲突，报告 `SPEC-CONFLICT` 并停止扩展。

## 2. 仓库 checkpoint

恢复时应重新运行命令核对，以下值只是本次交接时的事实：

```text
Repository:  E:\项目开发\acm-os
Branch:      main
HEAD:        60fe56b11bd3d541768ffa96aeca9596e4bb05e6
Subject:     feat: complete M7 contest workflow
origin/main: 61726681f26e13a03f5f601643a9d18188faf9e8
Ahead:       5 commits
```

`main` 领先 `origin/main` 的 5 个提交不是异常分叉，而是尚未 push 的连续本地 checkpoint：

```text
22caa0e build: complete M3 learning lifecycle
e9dd877 build: complete M4 review lifecycle
5d3afe6 build: complete M5 daily planning
55a728c build: complete M6 knowledge system
60fe56b feat: complete M7 contest workflow
```

远端仍停在 M2 文档 checkpoint。没有执行 push，也没有为 M3–M7 创建 tag。不得在没有用户明确授权时 push 或补 tag。

## 3. 已完成里程碑

```text
M0  Executable Foundation + Workspace Ready Gate       COMPLETE
M1  Real Contest Import → Lightweight Problems        COMPLETE
M2  Personal Markdown → External Obsidian Fresh Read  COMPLETE
M3  Upsolve Lifecycle → First Review Schedule         COMPLETE
M4  Review Focus → Evidence → Judgement               COMPLETE
M5  Today Planner → Stable Daily Execution            COMPLETE
M6  Knowledge Integration → Obsidian Relationships    COMPLETE
M7  Complete Contest Workflow + Manual Contest        COMPLETE

NEXT: M8 Recovery / Backup / Diagnostics / Failure Hardening
```

不得重复实现 M0–M7。只有实际证据发现回归时，才允许做范围最小的回归修复并重新验证。

## 4. M7 交付内容

M7 commit `60fe56b` 实现：

- Contest Facts Snapshot；
- Unknown / Not Attempted 等比赛结果语义；
- 比赛结束时 upsolve decision；
- Contest Result 与实时 Learning Status 分离；
- 完成 Facts 后通过 Correction Event 原子纠错，并保留不可变历史；
- Post-Contest AI Analysis raw text、只读 preview、complete/partial/failed parse status；
- 保存时由 Rust 重新解析 raw text，前端不能伪造 projection；
- Manual Codeforces Contest / Problem / Statement fallback；
- Manual statement HTML escaping 与 first-snapshot no-overwrite；
- Contest archive / restore 独立状态；
- 删除前 consequence preview；
- Contest 删除事务清理 Contest、Facts、Analysis、Correction Events 和关系；
- 正式、Personal、有学习/Review/Today/Knowledge/纠错历史或其他引用的 Problem 保留；
- 仅清理完全无引用、无历史的纯 Lightweight Problem；
- schema generation 16–20；
- Contest 页面专用响应式 Facts 布局和 AI Analysis 表单布局。

M7 没有进入 M8、M9 或 M10。

## 5. M7 验证证据

提交前最终验证：

```text
Infrastructure Rust:          117 passed, 0 failed, 2 ignored
Workspace Rust:               passed
cargo check --workspace:      passed
cargo fmt --all -- --check:   passed
DOM/UI:                       29 passed
Boundary tests:               5 passed
Architecture checker:         passed
TypeScript:                    passed
Vite production build:        passed
Desktop E2E:                  passed
git diff --cached --check:    passed
Manual GUI acceptance:        accepted by user
```

两项 ignored test 是发布阶段才运行的真实 Codeforces 网络 smoke tests。

人工验收覆盖：

- Manual Contest / Problem / Statement；
- statement 中 `<` 作为普通文本显示；
- Facts Snapshot 与实时 Learning Status 分离；
- Correction Event 与 no-change rejection；
- AI Analysis COMPLETE/FAILED、preview 不保存、failed raw text 保留；
- archive / restore；
- delete preview；
- 纯 Lightweight Problem 删除；
- Contest 页面响应式布局。

人工验收期间发现 Contest Facts 行和 AI textarea 布局混乱，已修复并由用户复验通过。

安全删除另有精确自动化证据：

- Personal / 历史 Problem 保留；
- Knowledge link Problem 保留；
- Correction history Lightweight Problem 保留；
- 无引用、无历史 Lightweight Problem 清理。

已知非阻塞告警：Vite 主 chunk 约 534–535 kB，超过默认 500 kB warning threshold。该项属于后续质量/性能阶段，不应在 M8 首个 Slice 中顺手扩展。

## 6. 当前工作树注意事项

M7 产品代码已经提交到 `60fe56b`。

本次交接更新后预期仍有未提交文档/配置修改：

```text
M  .gitignore
M  ACM-OS_BUILD_HANDOFF.md
M  ACM-OS_RECOVERY_PROMPT.md
```

这些文件不是 `60fe56b` 的一部分。恢复时必须以实际 `git status` 为准，不得丢弃。

旧的 `.desktop-e2e-*` 临时目录和仓库内 `.pnpm-store/` 已删除，并已将以下模式加入 `.gitignore`：

```text
.desktop-e2e-*/
.pnpm-store/
```

不得使用 `git clean` 清理未知文件。

## 7. 下一阶段：M8

冻结 PLAN 定义：

```text
M8 — Recovery / Backup / Diagnostics / Failure Hardening
```

Outcome：

> 异常情况下保护已知事实，不猜、不静默损坏。

冻结范围：

- Health；
- Location Anomaly repair；
- binding recovery；
- parse/concurrency UX；
- Critical Operation recovery；
- crash check；
- backup / retention / restore；
- derived rebuild；
- logs；
- diagnostic export preview；
- external-open failure；
- adapter health；
- 完整 Recovery Shell。

DoD：

```text
fault injection / backup restore / ambiguous relocation / crash matrix 全部有证据
```

计划 checkpoint：

```text
acm-os-m8-recovery
```

M8 的目标不是增加普通日常产品能力，而是确保异常、崩溃、迁移、文件移动、外部依赖失败和恢复操作中：

- 已知事实不丢失；
- 不确定状态不猜测；
- 不静默覆盖或修复；
- 用户能看见发生了什么、什么未受影响、下一步能做什么；
- 恢复操作具有可验证证据。

## 8. 下一个窗口必须先做什么

下一个窗口第一阶段只做恢复核对，不修改任何文件：

1. 运行 `ACM-OS_RECOVERY_PROMPT.md` 中的命令；
2. 完整读取五份权威/交接文档；
3. 核对 branch、HEAD、origin、ahead commits、tags、staged/unstaged/untracked；
4. 确认 M7 checkpoint 和验证边界；
5. 明确当前阶段只能是 M8。

恢复核对完成后，先做 M8 planning：

1. 从 SPEC / DESIGN / PLAN 提取 M8 的权威合同和 DoD；
2. 盘点现有 Recovery、backup、startup gate、Safe Patch recovery copy、binding relocation、health 和 diagnostics 能力；
3. 区分“已存在的基础能力”和“M8 真正缺口”；
4. 将 M8 切成独立可验证的 Slices；
5. 选择第一个最小 Slice，明确其包含和不包含的范围；
6. 得到清楚计划后才开始实现第一个 Slice。

不得：

- 重做 M0–M7；
- 一次性大规模实现整个 M8；
- 在首个 Slice 中混入后续 M8 能力；
- 进入 M9 accessibility/security/performance hardening；
- 进入 M10 release；
- 未经授权 commit、tag 或 push。

## 9. 每个 M8 Slice 的执行门禁

```text
implementation / fix
→ focused tests
→ infrastructure tests
→ workspace Rust tests
→ cargo check --workspace
→ cargo fmt --all -- --check
→ Startup / DOM / boundary（按 change surface）
→ TypeScript
→ Vite production build
→ Desktop E2E（按 change surface）
→ git diff --check
→ 完整 diff review
→ git status
```

当前 Slice 失败时不得叠加下一个 Slice。

每次报告必须区分：

- 已由实际命令证明的事实；
- 仍缺少的证据；
- 本次修改文件；
- 剩余 M8 Slices；
- 未提交、未暂存、未知文件；
- 是否触及 M9/M10（正常应为否）。

## 10. 最新闭环 M8 Slice — System Restore Candidate Preview

本 Slice 只建立真实 Restore 之前的只读候选预检，不执行恢复：

- 输入一个备份路径；
- canonical path 必须位于 App Private Data 的 `backups` 下；
- 只接受 `manual` / `pre-migration` / `pre-restore` / `daily` / `weekly` 目录中的已发布 `.sqlite3` 普通文件；
- 使用 read-only SQLite connection；
- 校验 SQLite integrity、foreign keys、migration ledger 与对应 schema contract；
- future schema 明确拒绝；
- older schema 返回 `migration_required = true`；
- preview 明确说明只恢复 System Facts，Markdown 不会被覆盖。

明确未包含：

- pre-restore backup；
- 替换当前数据库；
- 执行 migration；
- restore 后 binding validation / Fresh Read / derived rebuild / anomaly report；
- Recovery Shell 的恢复确认 UI。

2026-08-14 自动化证据：

- focused restore candidate tests：`5 passed / 0 failed`；
- Infrastructure：`168 passed / 0 failed / 2 ignored`；
- Rust workspace：通过；
- `cargo check --workspace`：通过；
- `cargo fmt --all -- --check`：通过；
- Startup：`6/6`；
- DOM：`35/35`；
- Boundary：`5/5` + boundary script passed；
- TypeScript：通过；
- Vite production build：通过。

两项 ignored 仍是 release-only Codeforces 网络 smoke。Desktop E2E 未运行：本 Slice 未加入 UI 操作流，真实 Tauri command registration 已由 Rust workspace 编译覆盖。

下一 Slice 仍必须保持最小边界。建议从真实 System Restore 的第一段开始：在任何数据库替换前创建并验证 `pre-restore` 一致性快照；不得同时铺开数据库替换、迁移、post-restore rebuild 与完整 Recovery Shell。

## 11. 最新闭环 M8 Slice — Pre-restore Snapshot Boundary

真实 restore 的第一段基础边界已完成，但仍未执行数据库替换：

- 先重新验证 restore candidate；
- candidate 无效时不创建 pre-restore 文件；
- 从当前 live System Facts 创建 SQLite-consistent snapshot；
- 快照先写 `.partial`，通过 integrity / foreign-key 校验后才发布到 `backups/pre-restore`；
- 快照失败返回独立 `pre_restore_backup_failed`，当前数据库保持可读且不进入后续 restore；
- 返回值同时保留 candidate preview 与 current schema，供后续 restore 编排使用。

明确未包含：当前数据库替换、旧 schema migration、restore 后 integrity/binding/Fresh Read/derived rebuild、Markdown 写入、IPC/UI。

2026-08-14 自动化证据：

- focused pre-restore tests：`3 passed / 0 failed`；
- Infrastructure：`171 passed / 0 failed / 2 ignored`；
- Rust workspace tests：通过；
- `cargo check --workspace`：通过；
- `cargo fmt --all -- --check`：通过。

本 Slice 没有新增 IPC、TypeScript 或 UI change surface，因此未重复运行 DOM/Vite/Desktop E2E。下一 Slice 应只处理候选数据库的受控替换与失败回滚边界；migration 和 post-restore rebuild 仍需继续拆分。

## 12. 最新闭环 M8 Slice — Verified Database Swap Primitive

已建立受控数据库文件交换原语，供后续 restore 编排使用：

- 调用前必须已有 verified staging database 和 pre-restore snapshot；
- 检查 current/staging/snapshot 都是普通文件；
- 检查 current SQLite WAL/SHM sidecar，不在数据库仍 busy 时交换；
- current database 先移动为 rollback 文件，再发布 staging；
- rollback 文件保留到后续 restore commit，不能在本 Slice 擅自删除；
- 缺少 pre-restore snapshot、busy 数据库、staging/current 无效或已有 rollback 时 fail closed。

该原语尚未接入 Tauri State/IPC 在线运行时，因此没有声称完成完整 restore。明确未包含：运行时连接重建、migration、restore 后 validation、binding/Fresh Read/derived rebuild、rollback commit/cleanup、Markdown 与 UI。

2026-08-14 自动化证据：

- focused swap tests：`2 passed / 0 failed`；
- Infrastructure：`173 passed / 0 failed / 2 ignored`；
- Rust workspace tests：通过；
- `cargo check --workspace`：通过；
- `cargo fmt --all -- --check`：通过。

下一 Slice 只能把该原语接入关闭 SQLite 连接、受控重启和失败回滚的运行时编排，仍不得同时加入 migration 或 post-restore rebuild。

## 13. 最新闭环 M8 Slice — Durable Restore Intent Preparation

本 Slice 完成了真实 restore 编排的持久化准备边界：

- Application `ManualBackupPort` 新增 `prepare_restore_intent` 正式 contract；
- 先只读预检 candidate，再创建并校验 `pre-restore` 当前数据库快照；
- candidate 复制到 `backups/pre-restore` 下的 `.partial` staging，完成 integrity 校验后原子发布；
- 通过 `restore-intent.json` 原子 rename 持久化 staging 与 pre-restore snapshot 路径；
- 已有 pending intent 拒绝覆盖，写入失败返回显式错误并清理 staging；
- 下一次 startup 在打开 SQLite pool 前消费 intent，失败时保留 intent 并进入 Recovery。

验证证据：`prepare_restore_intent` focused 2/2；Infrastructure `175 passed / 0 failed / 2 ignored`；workspace check 与 rustfmt 通过。

明确未包含：IPC/UI、自动重启、migration、post-restore rebuild、Markdown/binding 修复、rollback artifact 清理策略。

下一 Slice 只能处理受控 restart/restore 编排的外部入口，继续保持 migration 与 post-restore rebuild 分离。

## 14. 最新闭环 M8 Slice — Controlled Restore Preparation IPC

已将 durable restore preparation 接入 Tauri IPC：`prepare_system_restore` 接收已选择的 backup candidate，返回 staging、pre-restore snapshot 与 candidate preview 的 camelCase DTO。该入口只创建持久化 intent，不在当前不可变 `DatabaseRuntime` 中替换 pool，也不隐式执行重启；下一次启动仍由 startup gate 消费 intent。

验证证据：workspace `177 passed / 0 failed / 2 ignored`；`cargo check --workspace` 与 rustfmt 通过。前端构建未能在当前环境执行，因为 node runtime 不在 PATH。

明确未包含：UI 操作面板、进程重启 API、migration、post-restore rebuild、rollback 清理与 Markdown/binding 修复。

下一 Slice 只能实现显式、可确认的 restart handoff/恢复后状态回报，不得扩展到 migration 或 derived rebuild。

## 15. 最新闭环 M8 Slice — Explicit Restore Restart Handoff

已完成显式重启 handoff：Tauri `restart_for_pending_restore` 仅在 `restore-intent.json` 存在且为普通文件时调用 `AppHandle::request_restart()`；无 pending intent 时返回 `restore_intent_missing`。该命令不自行替换数据库，恢复仍由下一次 startup gate 在 SQLite pool 打开前消费。

验证证据：Tauri library tests `23 passed / 0 failed`；`cargo check --workspace`、rustfmt、`git diff --check` 通过。

明确未包含：重启后的 UI 状态回报、migration、post-restore rebuild、rollback 清理、Markdown/binding 修复。

下一 Slice 只能补充 startup 后 restore outcome 的只读诊断投影，继续禁止扩展到 migration 或 derived rebuild。

## 16. 最新闭环 M8 Slice — Read-only Restore Outcome Diagnostics

已新增 `restore_diagnostics` 只读 IPC：返回 `pendingIntent`、`rollbackArtifactPath` 与 startup state。该诊断只观察 durable intent、当前数据库对应的 rollback artifact 和既有 startup gate 状态，不删除文件、不改变 pool、不执行恢复动作。

验证证据：restore intent focused `2 passed / 0 failed`；Tauri library `23 passed / 0 failed`；`cargo check --workspace`、rustfmt、`git diff --check` 通过。

明确未包含：rollback 提交/清理、migration、post-restore rebuild、Markdown/binding 修复、完整 Recovery UI。

下一 Slice 只能处理 rollback artifact 的显式用户确认清理，并必须保持失败可恢复。

## 17. 最新闭环 M8 Slice — Explicit Rollback Artifact Cleanup

已新增 `confirm_restore_rollback_cleanup` 显式清理入口：

- 必须传入与当前 app data 精确匹配的 rollback artifact 路径；
- pending restore intent 存在时拒绝清理；
- rollback 文件必须是普通文件且通过 read-only integrity 校验；
- 仅在全部条件满足后删除 rollback artifact；
- 所有失败均返回明确错误，不自动猜测或强制删除。

验证证据：rollback cleanup focused `1 passed / 0 failed`；`cargo check --workspace`、rustfmt 通过。

明确未包含：自动 retention 清理、migration、post-restore rebuild、Markdown/binding 修复、Recovery UI。

下一 Slice 只能处理 restore 后 schema/integrity outcome 的只读投影。

## 18. 最新闭环 M8 Slice — Post-restore Schema / Integrity Projection

`restore_diagnostics` 现已同时投影：startup state、当前 schema version、pending intent、rollback artifact path 与 rollback read-only integrity 结果。rollback 不存在时 integrity 为 `null`；存在但无法打开或校验失败时为 `false`。

该 Slice 仅执行只读检查，不运行 migration、不清理 artifact、不修改 System Facts。

验证证据：rollback diagnostics focused `1 passed / 0 failed`；Tauri library `23 passed / 0 failed`；`cargo check --workspace` 与 rustfmt 通过。

下一 Slice 只能处理 post-restore derived-state rebuild 的 preview/plan，不得直接执行 rebuild 或修改 Markdown。

## 19. 最新闭环 M8 Slice — Post-restore Rebuild Preview

已新增 `preview_post_restore_rebuild` 只读 contract 与 IPC，报告：problem binding 数量、knowledge binding 数量、现有 derived relation 数量，以及后续 apply 将重新验证 binding、重建 derived Knowledge、绝不覆盖 Markdown 的范围声明。

该 preview 不写数据库、不扫描或修改 Markdown、不执行 rebuild。

验证证据：focused `1 passed / 0 failed`；Tauri library `23 passed / 0 failed`；`cargo check --workspace` 与 rustfmt 通过。

下一 Slice 只能执行 post-restore binding validation 的第一段，并必须把 anomaly 作为显式结果保留；不得同时执行 derived rebuild。

## 20. 最新闭环 M8 Slice — Post-restore Problem Binding Validation

新增 `validate_post_restore_problem_bindings` 只读 contract 与 IPC：逐条重新解析 Problem file binding，统计 ready 数量，并将 `location_anomaly`、`vault_unavailable`、`invalid_binding` 作为显式 anomaly 返回。

该 Slice 不更新 binding state、不写 Markdown、不执行 Knowledge binding 验证或 derived rebuild。

验证证据：focused `1 passed / 0 failed`；Tauri library `23 passed / 0 failed`；`cargo check --workspace` 与 rustfmt 通过。

下一 Slice 只能处理 post-restore Knowledge binding validation。

## 21. 最新闭环 M8 Slice — Post-restore Knowledge Binding Validation

新增 `validate_post_restore_knowledge_bindings` 只读 contract 与 IPC：重新发现 Knowledge Markdown，按冻结路径/identity digest 规则验证现有 Knowledge bindings，并返回 ready 数量、`confirmed_deleted` 数量及显式 `location_anomaly` 列表。

该 Slice 不更新 binding state、不执行 derived rebuild、不写 Markdown。

验证证据：focused `1 passed / 0 failed`；`cargo check --workspace` 与 rustfmt 通过。

下一 Slice 只能实现 derived Knowledge rebuild 的显式 apply 前置条件检查，仍不得自动执行 rebuild。

## 22. 最新闭环 M8 Slice — Derived Rebuild Preconditions

新增 `check_post_restore_rebuild_preconditions` 只读 contract 与 IPC，聚合检查：pending restore intent、startup recovery 状态、Problem binding anomalies、Knowledge binding anomalies。只有所有 blocker 为空时才返回 `eligible: true`。

该 Slice 只做前置条件判断，不执行 derived rebuild、不写数据库、不修改 Markdown。

验证证据：focused `1 passed / 0 failed`；Tauri library `23 passed / 0 failed`；`cargo check --workspace` 与 rustfmt 通过。

下一 Slice 仍需保持显式用户确认，再实现 derived Knowledge rebuild apply；不得自动执行。

## 23. 最新闭环 M8 Slice — Explicit Derived Rebuild Apply

新增 `apply_post_restore_rebuild` 显式 IPC。每次 apply 前重新运行全部前置条件；存在 pending intent、startup recovery 或任一 binding anomaly 时拒绝执行。通过后调用既有 Knowledge index rebuild，并返回 node、relation 与 anomaly 数量。

该 apply 不写 Markdown；既有 rebuild mutation 继续使用 Daily Snapshot 边界。

验证证据：focused `1 passed / 0 failed`；Tauri library `23 passed / 0 failed`；`cargo check --workspace` 与 rustfmt 通过。

下一 Slice 应盘点 M8 最终 DoD 与剩余 diagnostics/UI 缺口，不得直接进入 M9。

## 24. M8 Final DoD Audit — Remaining Gaps

对照冻结 PLAN M8 范围与 DoD，已闭环：Critical Operation/crash matrix、Location Anomaly repair、binding recovery、Safe Patch failure handling、consistent manual/daily/pre-migration/pre-restore backup、restore/startup swap、post-restore validation/rebuild、rollback diagnostics/cleanup。

仍未闭环：

- Weekly Snapshot 的真实生成边界；
- 7 Daily + 4 Weekly retention 的显式 apply（当前只有 preview）；
- 聚合 System Health / adapter health；
- logs 与 diagnostic export preview；
- parse/concurrency 与 external-open failure 的统一 Recovery UX；
- 完整 Recovery Shell；
- M8 fault-injection、backup/restore、ambiguous relocation、crash matrix 的最终集中证据。

因此 M8 尚未完成，禁止进入 M9。下一 Slice 选择只读 System Health 聚合，不同时加入 export、retention 或 UI。
-
## 25. Latest M8 Slice - Read-only System Health Aggregation

Added `system_health_snapshot` read-only IPC aggregating startup/schema state, pending or needs-recovery Critical Operations, published backup count, pending restore intent, and rollback integrity.

This slice performs no repair, cleanup, export, retention mutation, or UI transition.

Evidence: focused `1 passed / 0 failed`; `cargo check --workspace`; rustfmt; `git diff --check`.

Next slice: diagnostic export preview only. Retention mutation and Recovery UI remain out of scope.

## 26. Latest M8 Slice - Diagnostic Export Preview

Added `preview_diagnostic_export` read-only contract and IPC. It declares the output directory, diagnostic sections, and privacy exclusions while guaranteeing `createsFiles: false`.

The preview excludes Markdown/statement content, credentials, and absolute workspace paths. It creates no directory or export artifact.

Evidence: focused `1 passed / 0 failed`; Tauri library `23 passed / 0 failed`; workspace check and formatting pass.

Next slice: Weekly Snapshot generation boundary only. Retention apply and Recovery UI remain separate.

## 27. Latest M8 Slice - Weekly Snapshot Boundary

Added explicit `create_weekly_backup` contract and IPC. It creates a SQLite-consistent snapshot under `backups/weekly` with a published `weekly-` filename and integrity verification before publication.

Retention pruning, schedule automation, and Recovery UI remain out of scope.

Evidence: focused `1 passed / 0 failed`; Tauri library `23 passed / 0 failed`; workspace check and formatting pass.

Next slice: weekly/daily retention apply preview and policy checks only.

## 28. Latest M8 Slice - Backup Retention Preview

Added `preview_backup_retention` read-only contract and IPC. It returns protected and prune-candidate paths under the frozen `7 Daily + 4 Weekly` policy without deleting files or even creating a backup directory.

Evidence: focused `1 passed / 0 failed`; Tauri library `23 passed / 0 failed`; workspace check and formatting pass.

Next slice: explicit, user-confirmed retention apply with exact preview paths and no broad deletion.

## 29. Latest M8 Slice - Explicit Retention Apply

Added `apply_backup_retention`. It recomputes the current retention preview, requires the submitted path set to exactly match current prune candidates, restricts deletion to regular `.sqlite3` files under daily/weekly backup roots, and rejects mismatches without mutation.

Evidence: focused retention test passed; `cargo check --workspace`, rustfmt, and `git diff --check` pass.
