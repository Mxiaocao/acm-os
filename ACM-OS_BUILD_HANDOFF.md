# ACM-OS BUILD Handoff — M1 checkpoint

> 更新时间：2026-08-10（Asia/Shanghai）
>
> 用途：切换 Codex 窗口后，在同一 Windows 本地仓库继续 M1。
>
> 本文件是交接上下文，不是完成证明。接手者必须以四份权威文档、当前文件、实际工作区和命令结果为准。

## 1. Authority and frozen scope

权威顺序：

`ACM-OS_SPEC_v1.md > ACM-OS_DESIGN_v1.md > ACM-OS_PLAN_v1.md > implementation`

接手后完整阅读：

- `ACM-OS_SPEC_v1.md`
- `ACM-OS_DESIGN_v1.md`
- `ACM-OS_PLAN_v1.md`
- `ACM-OS_BUILD_HANDOFF.md`

当前 Milestone：`M1 — Real Contest Import → Lightweight Problems`。

冻结目标：

```text
真实 Codeforces Public Contest
→ 完整 manifest
→ 全部题成为 Lightweight Problem
→ 每题保存第一次成功的题面 snapshot
```

本窗口不得进入 M2+：Personal Markdown、Vault discovery/watchers、Problem learning lifecycle、Review scheduling/execution、Today planning。

## 2. Repository and Git checkpoint

真实路径：

`E:\项目开发\acm-os`

当前分支与已完成 M0：

```text
Branch: main
HEAD:   42815a8 build: complete B0.4 startup shells
Tag:    acm-os-m0-foundation → 42815a8
Remote: https://github.com/Mxiaocao/acm-os.git
```

M0 已完成并已推送：

- `main` 已推送到 `origin/main`；
- `acm-os-m0-foundation` 标签已推送到 `origin`。

当前 M1 尚未提交、尚未打标签、尚未推送。不要对 M1 执行 commit/tag/push，除非用户明确授权并完成阶段验收。

当前工作区预期：

```text
 M src-tauri/crates/acm-os-application/src/lib.rs
 M src-tauri/crates/acm-os-domain/src/lib.rs
 M src-tauri/crates/acm-os-infrastructure/src/persistence.rs
?? src-tauri/crates/acm-os-application/src/codeforces.rs
?? src-tauri/crates/acm-os-infrastructure/migrations/0003_create_contest_import.sql
```

交接时不要丢弃这些改动，不要 reset、clean 或 checkout 覆盖它们。

## 3. M1 implementation already present

### Domain identity

`src-tauri/crates/acm-os-domain/src/lib.rs`

- `CodeforcesContestIdentity`：`platform = codeforces`，正整数 contest id；
- `CodeforcesProblemIdentity`：`(codeforces, contest id, uppercase/digit index)`；
- 标题、URL、难度不是去重 identity；
- malformed contest id / problem index 被拒绝。

### Secure locator

`src-tauri/crates/acm-os-application/src/codeforces.rs`

只接受：

```text
https://codeforces.com/contest/<positive-id>
https://www.codeforces.com/contest/<positive-id>
```

可带一个尾部 `/`。HTTP、非 Codeforces host、problem URL、嵌套路径、零 id 均拒绝。后续 adapter 必须从 strong identity 自己构造远程 URL，不能把用户输入当任意 downloader URL。

### Canonical import contract

`src-tauri/crates/acm-os-application/src/lib.rs`

已加入：

- `ContestImportDraft`；
- ordered `ContestProblemSlotDraft`；
- `StatementSnapshotDraft`；
- `ContestImportPort`；
- `ContestImportStatus::{Incomplete, Complete}`；
- manifest validation：title/source URL、非空 manifest、连续 ordinal、slot contest identity、duplicate identity。

Application/domain 不得加入 filesystem、network、SQLite 或平台 authority。

### Persistence schema and idempotency

`src-tauri/crates/acm-os-infrastructure/migrations/0003_create_contest_import.sql`

schema v3 新增：

- `contests`：Codeforces contest strong identity、metadata、`incomplete/complete`；
- `problems`：Lightweight Problem strong identity；
- `contest_problems`：ordered Contest slots 与 snapshot state；
- `problem_statement_snapshots`：每个 Problem 单一 first snapshot，`ON CONFLICT DO NOTHING`。

`src-tauri/crates/acm-os-infrastructure/src/persistence.rs` 已实现：

- schema v3 严格启动对象/列校验；
- manifest-first persistence；
- duplicate fast path；
- manifest drift rejection，不静默刷新首次 manifest；
- progressive import：成功项保留，缺失 snapshot 返回 `Incomplete`；
- retry missing；
- first snapshot no-overwrite；
- import status 重新计算。

注意：真实 Codeforces adapter、HTTP/API、statement HTML parser/sanitize、asset localization、IPC、Contest UI、My Problems UI、Problem statement view 尚未实现。

## 4. Verification evidence

最近一次通过：

```text
C:\Users\Mxiaocao\.cargo\bin\cargo.exe test --workspace --locked
```

结果：

```text
Tauri IPC:       6 passed
Application:     7 passed
Domain:          1 passed
Infrastructure: 26 passed
全部通过
```

边界检查：

```text
C:\Users\Mxiaocao\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe scripts\check-boundaries.mjs
```

结果：`boundary check passed`。

`git diff --check`：通过。Rust 测试期间只有已有的 Windows linker stdout warning，无编译失败。

当前未完成验证：

- Codeforces fixture adapter tests；
- HTML sanitize / assets tests；
- real Codeforces smoke；
- frontend/IPC/UI tests；
- M1 Tauri debug build and desktop smoke。

## 5. Exact next actions for the next Codex window

1. `Get-Location` 确认精确为 `E:\项目开发\acm-os`。
2. 完整阅读四份权威文档和本 handoff。
3. 执行：

   ```powershell
   git status
   git branch --show-current
   git log -3 --oneline --decorate
   git remote -v
   git tag --points-at HEAD
   ```

4. 读取全部当前 M1 diff，并单独读取两个未跟踪文件：`src-tauri/crates/acm-os-application/src/codeforces.rs`、`src-tauri/crates/acm-os-infrastructure/migrations/0003_create_contest_import.sql`。
5. 不提交当前 M1 改动。先实现 Codeforces adapter 的 fixture contract：官方 API metadata fixture、statement HTML fixture、identity validation、manifest completeness、URL construction security。
6. 再实现 statement sanitizer 与必要 asset localization；raw external HTML 永不直接 render，网络请求不得在长 SQLite transaction 内执行。
7. 用固定 fixture 覆盖：complete、partial、retry missing only、duplicate、manifest stability、first snapshot no-overwrite、sanitize/assets、URL security。
8. 运行 focused tests、`check-boundaries`、Rust workspace tests/check、`git diff --check`。
9. 之后再实现薄 IPC 与 M1 UI：Contest import、Contest shelf/detail、partial retry、My Problems index、Problem statement view；不要实现 M2+。
10. 只有 M1 DoD 全部有证据后，向用户请求允许提交；提交后再创建 `acm-os-m1-contest-import` 标签并推送。

## 6. M1 closure update (2026-08-10)

This section supersedes any earlier implementation-status notes in this handoff
that say the M1 Codeforces adapter, statement sanitizer/assets, IPC, contest UI,
or problem statement view are not implemented. M1 implementation and acceptance
verification are complete in the working tree, subject to the Git closure policy
below. The work remains intentionally uncommitted.

Implemented and verified:

- Codeforces fixed-request adapter, fixture contract, strict locator validation,
  sanitized statement snapshots, and localized assets.
- Manifest-first and idempotent SQLite import, partial retry, snapshot protection,
  Contest Import/Shelf/Detail IPC, My Problems, and local-only statement rendering.
- `cargo test --workspace --locked`, `cargo check --workspace --locked`, TypeScript
  type-check, Vite production build, Tauri Debug build, startup-shell tests,
  boundary check, and `git diff --check` all passed.
- The ignored real Codeforces metadata smoke and full import/idempotency smoke both
  passed when run manually.

Known environment limitation: `scripts/dom-shells.test.mjs` cannot start because
Windows returns EPERM while Node reads a pnpm dependency under `node_modules`; this
occurs before the application assertions. It is not a failing application test.

No commit, tag, or push has been performed. Do not enter M2 until the user grants
Git closure authority and M1 has been deliberately committed/tagged/pushed.

## 7. Git handoff rules

当前目标不是提交 M1，而是安全切换窗口并保留未提交进度。

禁止：

- `git reset --hard`、`git clean`、覆盖式 checkout；
- 把未完成 M1 改动推送到 M0 标签；
- 提前创建 M1 checkpoint tag；
- 将 fixture PASS 冒充 real Codeforces smoke PASS；
- 宣称 M1 完成。

阶段完成后的流程：

```text
实现 → focused verification → full verification → 用户确认
→ commit → 阶段 tag → push main + tag → 更新 handoff → 新 Codex 窗口
```
