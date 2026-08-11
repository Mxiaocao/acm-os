# ACM-OS BUILD Handoff — M2.5 implemented / M2 checkpoint pending

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
M2.3 checkpoint: 6ae1cbf build: complete M2.3 binding resolution
M2.4 checkpoint: 6b80394 build: complete M2.4 revalidation and Obsidian open
Working tree: verified M2.5 implementation and handoff changes, pending commit
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
6ae1cbf build: complete M2.3 binding resolution
6b80394 build: complete M2.4 revalidation and Obsidian open
```

标签：

```text
acm-os-m0-foundation    → 42815a8
acm-os-m1-contest-import → 4e25359
```

M1 的 `main` 与标签均已在 GitHub 远程确认存在。M2.1/M2.2/M2.3/M2.4 提交目前只在本地 `main`，尚未 push。因此远程只能恢复到 M1，本地 Git 历史可恢复到 M2.4。

## 3. Current BUILD position

```text
Completed: M0 — Executable Foundation + Workspace Ready Gate
Completed: M1 — Real Contest Import → Lightweight Problems
Completed: M2.1 — Create Personal Note + File Binding
Completed: M2.2 — Fresh Read + Markdown Parser
Completed: M2.3 — Binding Resolution + Vault Availability
Completed: M2.4 — Revalidation Triggers + Open in Obsidian
Completed: M2.5 — Safe Patch + Recovery Copy Foundation (pending commit)
Next:      commit M2.5 → create acm-os-m2-vault-binding checkpoint → M3 planning
M2 state:  CODE COMPLETE / CHECKPOINT PENDING
```

不要重复实现 M1/M2.1/M2.2/M2.3/M2.4/M2.5。M2.5 未提交且 M2 checkpoint 未确认前，不得进入 M3+。

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

M2.3 已实现并由提交 `6ae1cbf` 封存：

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

## 5.4 M2.4 closure record

M2.4 已实现并由提交 `6b80394` 封存：

- 使用 `notify` native watcher 递归监听 Active Vault 中的 Markdown 变更；事件只发出 `personal-note-invalidated` 信号，不携带或注入正文事实；
- watcher 在应用启动和首次 workspace 配置完成后挂载，150ms 去抖；watcher 不可用时 window-focus Fresh Read 仍保证正确性；
- Problem Detail 将 initial/create/focus/watcher 统一到同一个 authoritative `personal_note_projection` 刷新函数；
- 窗口重新获得 focus 时必定重新调用 projection IPC，竞态结果按 sequence 丢弃，组件卸载后不更新状态；
- `Open in Obsidian` 只在 Ready Personal binding 上出现；后端再次 Fresh Read、解析 relocation、canonicalize 并验证目标仍在 Active Vault 内，再构造编码后的 `obsidian://open?path=...` URI；
- external open 失败只显示局部错误，并提供 Retry、Copy path、Check settings；Personal identity 与学习状态不变；
- Review Focus DOM 中不存在普通 `Open in Obsidian` 入口；
- architecture dependency allowlist 只授权 root Tauri adapter 使用 `notify`、`tauri-plugin-opener`、`url`，没有向 Domain/Application/Infrastructure 或 React 下放 filesystem authority。

验证证据：

- Rust workspace：67 passed，2 个真实网络 smoke ignored；
- native watcher temporary-Vault integration smoke：PASS；
- Obsidian canonical path / path escape tests：PASS；
- startup / boundary / DOM：21 passed；
- TypeScript type-check 与 Vite production build：PASS；
- `cargo check --workspace`：PASS；
- Tauri Debug executable（no bundle）：PASS；
- `git diff --check`：PASS。

M2.4 没有实现 Safe Patch、Recovery Copy 产品行为或任何 M3 lifecycle。下一切片 M2.5 应实现冻结的 Fresh Read → unique target → minimal patch → concurrent digest check → recovery copy → write → re-read/re-parse/semantic verify 事务链。

## 5.5 M2.5 closure record

M2.5 已实现并完成验证，尚待用户明确允许后 commit：

- Application 只暴露 `AddExtraProblemLink` 语义命令；调用者不能传 path、offset、完整 Markdown 或 generic write；
- link target 在 Application 入口验证，拒绝空值、首尾空白、控制字符、嵌套 `[[...]]` 和 alias 注入；
- 写事务执行 binding resolve/Fresh Read、Active Vault canonical path 校验、UTF-8 校验、最新结构解析与唯一 `## 额外题目` 验证；
- patch 只在唯一目标 section 的 source offset 内追加真实 Markdown list item，目标区外 byte-for-byte 保持；
- 保留 UTF-8 BOM 与目标 section 的 LF/CRLF 风格；重复 link 由 Markdown list AST 判断，不把 code span 文本误认成正式条目；
- 写前在 App Private `markdown-recovery/problem-markdown` 创建 exact pre-write copy，不污染 Vault；bucket 由稳定 Problem identity hash 决定，文件 rename 后仍归入同一 bucket；
- recovery filename 保存完整 pre/post digest，为未来 Undo 的“当前 digest 必须等于写后 digest”守卫提供证据；每个 bucket 最多 10 份且最长 30 天；
- recovery 创建/裁剪失败时不写 Vault；写前再次读盘比较完整 digest，外部并发修改时返回 `markdown_concurrent_modification`，不 merge、不覆盖；
- 使用同目录临时文件、flush、`sync_all` 与原子 persist 替换；写后重读、重新解析并验证 bytes 与 semantic postcondition；
- 成功后 optimistic 更新 binding digest/file key 并失效受影响 projection cache；M2.5 不创建 Candidate/正式关系 System Fact，不提供产品 UI，后续 M6 在此事务成功后再提交对应事实。

验证证据：

- Rust workspace：80 passed，2 个真实网络 smoke ignored；
- temporary Vault：BOM/CRLF、byte-preserving patch、missing/ambiguous/duplicate section、invalid UTF-8、path escape、recovery failure、concurrent edit、atomic write failure、semantic verify failure：PASS；
- recovery exact bytes、稳定 bucket、完整 pre/post digest、10 copies / 30 days retention：PASS；
- startup / boundary / DOM：21 passed；
- TypeScript type-check 与 Vite production build：PASS；
- `cargo check --workspace --locked`：PASS；
- Tauri Debug executable（no bundle）：PASS；
- `git diff --check`：PASS。

M2.5 没有实现 Candidate 接受 UI、自动 Undo UI、任何 Learning Status/Review/Today 行为或 M3 lifecycle。完整 M2 代码已完成；提交 M2.5 后才可由用户明确决定创建 `acm-os-m2-vault-binding` tag。

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
4. 明确报告 M1/M2.1/M2.2/M2.3/M2.4 checkpoint 是否仍完整、M2.5 工作树是否只有已知改动；
5. 核对 Safe Patch / Recovery Copy 的真实 diff 与最后验证证据；
6. 未经用户明确允许，不 commit、tag、push；
7. M2.5 commit 后，再由用户明确决定是否创建 `acm-os-m2-vault-binding`；
8. checkpoint 未确认前不进入 M3；
9. 后续继续按“最小 Slice → focused verification → broader verification → diff review → status”执行；
10. 当前 Slice 失败时不叠加下一 Slice。

建议的首个动作不是写代码，而是：

`审阅 M2.5 staged/unstaged diff、Safe Patch transaction tests 与 recovery retention evidence；确认提交后再处理 M2 checkpoint tag。`

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
- M2.5 未提交或验证未通过就创建 `acm-os-m2-vault-binding`；
- M2 checkpoint 未确认就进入 M3+。

## 11. Durable recovery prompt

可直接复制到新 Codex 窗口的完整提示词保存在：

`ACM-OS_RECOVERY_PROMPT.md`

该提示词被设计为即使没有聊天记录、甚至没有本 handoff，也会先从 Git 和权威文档重建事实，而不是相信旧会话记忆。
