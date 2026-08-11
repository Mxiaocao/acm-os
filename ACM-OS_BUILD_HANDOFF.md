# ACM-OS BUILD Handoff — M2.3 closed / M2.4 next

> 更新时间：2026-08-11（Asia/Shanghai）
>
> 用途：在丢失聊天上下文或切换 Codex 账号后，从真实 Windows 仓库安全恢复工作。
>
> 本文件是交接上下文，不替代当前仓库和实际命令证据。

## 1. Authority

权威顺序严格保持：

`SPEC > DESIGN > PLAN > IMPLEMENTATION`

接手者必须完整阅读：

- `ACM-OS_SPEC_v1.md`
- `ACM-OS_DESIGN_v1.md`
- `ACM-OS_PLAN_v1.md`
- `ACM-OS_BUILD_HANDOFF.md`

SPEC 是唯一产品事实来源。不得自行改变冻结的产品行为、Authority、状态机、Review、Today、Markdown、事务边界、测试策略或 Milestone 顺序。若发现真实冲突，标记 `SPEC-CONFLICT` 并停止。

## 2. Verified repository checkpoint

真实路径：

`E:\项目开发\acm-os`

2026-08-11 实际核对结果：

```text
Branch:      main
M2.1 code checkpoint: 27f785f build: complete M2.1 personal note binding
M2.1 handoff checkpoint: cec7d6f docs: record M2.1 handoff
M2.2 checkpoint: 8edc301 build: complete M2.2 fresh markdown read
Working tree: verified M2.3 implementation and handoff changes, pending commit
origin/main: 4e253590fd0eee8d5d7af61bb14529bff4cd6e6b
M1 tag:     acm-os-m1-contest-import
Remote tag: 4e253590fd0eee8d5d7af61bb14529bff4cd6e6b
Remote:      https://github.com/Mxiaocao/acm-os.git
```

里程碑历史：

```text
d00a915 build: complete B0.1 repository scaffold
ece09f8 build: complete B0.2 SQLite startup gate
f140316 build: complete B0.3 workspace configuration
42815a8 build: complete B0.4 startup shells
4e25359 build: complete M1 contest import
27f785f build: complete M2.1 personal note binding
cec7d6f docs: record M2.1 handoff
8edc301 build: complete M2.2 fresh markdown read
```

标签：

```text
acm-os-m0-foundation    → 42815a8
acm-os-m1-contest-import → 4e25359
```

M1 的 `main` 与标签均已在 GitHub 远程确认存在。M2.1/M2.2 提交目前只在本地 `main`，尚未 push。因此远程只能恢复到 M1，本地 Git 历史可恢复到 M2.2。

## 3. Current BUILD position

```text
Completed: M0 — Executable Foundation + Workspace Ready Gate
Completed: M1 — Real Contest Import → Lightweight Problems
Completed: M2.1 — Create Personal Note + File Binding
Completed: M2.2 — Fresh Read + Markdown Parser
Completed: M2.3 — Binding Resolution + Vault Availability
Next:      M2.4 — Revalidation Triggers + Open in Obsidian
M2 state:  IN PROGRESS
```

不要重复实现 M1/M2.1/M2.2/M2.3，也不要跳过 M2.4 或进入 M3+。

## 4. M1 closure record

M1 已实现并由提交 `4e25359` 封存：

- Codeforces locator 与 strong identity；
- fixture-backed adapter；
- manifest-first、progressive、idempotent import；
- statement sanitize 与必要 asset localization；
- first statement snapshot no-overwrite；
- Contest Import / Shelf / Detail typed IPC 与 UI；
- My Problems 与本地 statement view；
- schema migration `0003_create_contest_import.sql`。

提交前记录的验证证据：

- `cargo test --workspace --locked`：PASS；
- `cargo check --workspace --locked`：PASS；
- TypeScript type-check：PASS；
- Vite production build：PASS；
- Tauri Debug build：PASS；
- startup-shell tests：PASS；
- boundary checks：PASS；
- `git diff --check`：PASS；
- ignored real Codeforces metadata smoke：手动 PASS；
- full import/idempotency smoke：手动 PASS。

已知环境限制：`scripts/dom-shells.test.mjs` 曾在 Windows 读取 pnpm dependency 时因 EPERM 无法启动，发生在应用断言之前。接手者必须重新核对当前环境，不能把旧记录自动当成当前 PASS 或 FAIL。

## 5. M2.1 closure record

M2.1 已实现并由提交 `27f785f` 封存：

- schema migration `0004_create_personal_notes.sql`；
- Lightweight / Personal identity 与 File Binding Registry；
- 冻结 initial Markdown skeleton；
- `create_new` 防覆盖、写后重读验证、SHA-256 digest；
- 安全封装的 Windows file key；
- binding commit 失败时的 digest-guarded 补偿；
- typed IPC 与 Problem Detail 创建笔记 UI。

验证证据：

- Rust workspace：51 passed，2 个真实网络 smoke ignored；
- startup / boundary / DOM：18 passed；
- TypeScript type-check、Vite production build、Cargo check：PASS；
- Tauri Debug build：PASS；
- `git diff --check`：PASS。

M2.1 没有实现 Fresh Read parser、relocation、watcher、Safe Patch 产品行为或 M3 lifecycle。

## 5.2 M2.2 closure record

M2.2 已实现并由提交 `8edc301` 封存：

- `pulldown-cmark` CommonMark source-offset parser；
- 已知 H2：`前置知识`、`题解`、`额外题目`、`Hints`、`思路`、`代码`；
- 仅识别 `## 题解` 下直接 `###` 为 Solution Route，`####` 不识别，Route 名称原样保留；
- 重复 Known Section 产生局部 warning，不猜测唯一目标；
- 每次 authoritative read 先读取当前磁盘 bytes 并计算 SHA-256；
- projection cache 仅在 digest 相同时复用；
- typed IPC 与 Problem Detail 最新 projection UI；
- stale cache + 外部直接编辑 + 无 watcher event 的 blocking evidence。

验证证据：

- Rust workspace：56 passed，2 个真实网络 smoke ignored；
- startup / boundary / DOM：18 passed；
- TypeScript type-check 与 Vite production build：PASS；
- Tauri Debug executable（no bundle）：PASS；
- `git diff --check`：PASS。

M2.2 没有实现 relocation、watcher、window-focus revalidation、Open in Obsidian、Safe Patch、Recovery Copy 或 M3 lifecycle。M2.3 应先处理 path → Windows file key → digest 的绑定解析与 Vault unavailable 的 affected-scope 状态。

## 5.3 M2.3 closure record

M2.3 已实现并完成验证，尚待用户明确允许后 commit：

- 确定性 binding resolution：原路径 → 唯一 Windows file key → 唯一完整内容 digest；
- relocation 成功后以短 SQL optimistic guard 更新 path、file key、digest 与 `linked` 状态；
- digest/file-key 多候选或候选已绑定其他 Problem 时进入 `location_anomaly`，禁止抢占；
- Vault 不可用进入 `external_source_unavailable`，保留 Personal Problem 与全部 System Facts；
- Vault 恢复且绑定可解析时自动恢复 `linked`；
- path escape 在扫描前拒绝，扫描只接受 Active Vault 内 canonical Markdown 文件；
- typed IPC/UI 显式区分 Ready、Location Anomaly、Vault Unavailable；
- UI 展示 relocation 后的新路径，不把受影响 scope 误报成全局 Recovery。

验证证据：

- Rust workspace：63 passed，2 个真实网络 smoke ignored；
- startup / boundary / DOM：19 passed；
- TypeScript type-check 与 Vite production build：PASS；
- Tauri Debug executable（no bundle）：PASS；
- `git diff --check`：PASS。

M2.3 没有实现 watcher、window-focus revalidation、Open in Obsidian、Safe Patch、Recovery Copy 或 M3 lifecycle。M2.4 应只增加 revalidation triggers 与 external open，不得让 watcher 成为事实源。

## 6. Frozen M2 outcome and scope

PLAN 中的 M2 Outcome：

```text
Lightweight Problem 创建真实 Personal Markdown
→ 外部 Obsidian 修改
→ ACM-OS Fresh Read 最新内容
```

M2 范围：

- create personal note；
- initial Markdown skeleton；
- File Binding Registry；
- Windows file key；
- digest 与 relocation；
- Fresh Read / parser；
- watcher（只做 cache invalidation / re-read trigger，不是事实源）；
- window-focus revalidation；
- Open in Obsidian；
- Safe Patch engine；
- Recovery Copy foundation。

主要验收：

- `AC-PROBLEM-01`；
- `AC-MD-01` 至 `AC-MD-06`；
- Candidate relation 的实际产品行为仍留到 M6。

M2 blocking evidence：

```text
已有 stale cache
→ 外部编辑 Markdown
→ watcher event 丢失或没有发生
→ 再次读取时仍必须通过 Fresh Read 得到最新内容
```

M2 checkpoint 仅在完整 DoD 后创建：

`acm-os-m2-vault-binding`

## 7. Frozen Markdown and Authority constraints

接手者必须特别保护：

- Markdown 内容由 Markdown 权威拥有，SQLite 不能覆盖正文事实；
- 不向 Markdown 注入 ACM-OS 私有 ID；
- Problem Identity 与 File Binding 分离；
- watcher 事件不是事实源；
- 每次 authoritative read 必须 Fresh Read；
- write 必须执行 fresh read、唯一目标验证、minimal/byte-preserving patch、并发 digest 检查、pre-write recovery copy、写后重读/重解析/语义验证；
- 外部编辑优先，不能用 stale cache 回写覆盖；
- Vault 暂时不可用是 degraded/affected-scope 状态，不等于全局 Startup Recovery；
- React 不得获得 filesystem 或 SQLite Authority；
- 网络或文件 I/O 不得放进长 SQLite transaction；
- M2 不实现 Problem learning lifecycle、Review、Today 或 Knowledge 正式关系。

## 8. Recovery procedure if chat or handoff is lost

如果聊天窗口消失，但仓库还在：

1. 打开 `E:\项目开发\acm-os`；
2. 运行 `Get-Location`、`git status`、`git branch --show-current`、`git log --oneline --decorate -10`；
3. 运行 `git tag --points-at HEAD`、`git remote -v`；
4. 完整阅读三份冻结文档；
5. 用 `git show --stat 4e25359`、`git show --stat 27f785f` 和 `git show acm-os-m1-contest-import:ACM-OS_BUILD_HANDOFF.md` 查看 M1/M2.1 历史证据；
6. 若工作树非空，先识别用户改动，禁止 reset/clean/覆盖；
7. 只有 HEAD/分支/工作树与用户目标厘清后才继续。

如果本地仓库也丢失：

- 远程仓库为 `https://github.com/Mxiaocao/acm-os.git`；
- 已确认远程 `main` 与 `acm-os-m1-contest-import` 都指向完整 M1 提交 `4e253590fd0eee8d5d7af61bb14529bff4cd6e6b`；
- 只在用户指定的空目录中重新 clone；不要覆盖未知目录或现有用户文件；
- clone 后再次读取权威文档并核对 tag，不凭本 handoff 直接实施。

## 9. Exact next-session workflow

新 Codex 首轮必须先只读取证：

```powershell
Get-Location
git status
git branch --show-current
git log -10 --oneline --decorate
git remote -v
git tag --points-at HEAD
```

然后：

1. 完整阅读四份权威文档；
2. 检查顶层目录、manifests、lockfiles、toolchain 与当前测试入口；
3. 检查 `4e25359` 和 `27f785f` 的实际 change surface；
4. 明确报告 M1/M2.1/M2.2/M2.3 checkpoint 是否仍完整、工作树是否有未知改动；
5. 从 SPEC/DESIGN/PLAN 提取 M2.4 最小纵向 Slice 和 Done Evidence；
6. 在修改前提交 M2.4 实施计划供用户确认；
7. 按“最小 Slice → focused verification → broader verification → diff review → status”执行；
8. 当前 Slice 失败时不叠加下一 Slice；
9. 未经用户明确允许，不 commit、tag、push；
10. M2 完成前不进入 M3。

建议的 M2.4 首个动作不是写代码，而是：

`审阅 M2.3 resolver 与 Tauri lifecycle 边界，并形成 watcher invalidation、window-focus authoritative re-read、Open in Obsidian failure isolation 的最小实施切片与验收矩阵。`

## 10. Git and safety rules

禁止：

- `git reset --hard`；
- `git clean`；
- 覆盖式 checkout；
- 在未知目录 clone/复制覆盖；
- 手写或伪造 lockfile；
- 用假工具、跳过边界检查或伪造 build/test PASS；
- 未经允许安装大型或系统级工具；
- 未经允许 commit、tag、push；
- M2 未完成就创建 `acm-os-m2-vault-binding`；
- 进入 M3+。

## 11. Durable recovery prompt

可直接复制到新 Codex 窗口的完整提示词保存在：

`ACM-OS_RECOVERY_PROMPT.md`

该提示词被设计为即使没有聊天记录、甚至没有本 handoff，也会先从 Git 和权威文档重建事实，而不是相信旧会话记忆。
