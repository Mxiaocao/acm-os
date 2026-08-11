# ACM-OS BUILD Handoff — M3 complete

> 更新时间：2026-08-12（Asia/Shanghai）
>
> 用途：聊天上下文丢失或切换开发者后，从真实 Windows 仓库安全恢复 BUILD 工作。
>
> 本文件是恢复参考，不替代 Git、冻结文档和实际命令证据。

## 1. Authority and recovery rule

权威顺序保持不变：

`SPEC > DESIGN > PLAN > IMPLEMENTATION`

接手者必须完整阅读：

- `ACM-OS_SPEC_v1.md`
- `ACM-OS_DESIGN_v1.md`
- `ACM-OS_PLAN_v1.md`
- `ACM-OS_BUILD_HANDOFF.md`

SPEC 是唯一产品事实来源。不得自行改变冻结的产品行为、Authority、状态机、Review、Today、Markdown、事务边界、测试策略或 Milestone 顺序。发现真实冲突时标记 `SPEC-CONFLICT` 并停止。

## 2. Repository checkpoint

真实路径：

`E:\项目开发\acm-os`

生成本交接文档前的实际状态：

```text
Branch: main
Base HEAD: 6172668 docs: record M2 checkpoint
origin/main: 6172668
M2 tag: acm-os-m2-vault-binding -> 091b525
Remote: https://github.com/Mxiaocao/acm-os.git
```

历史 checkpoint：

```text
d00a915 build: complete B0.1 repository scaffold
ece09f8 build: complete B0.2 SQLite startup gate
f140316 build: complete B0.3 workspace configuration
42815a8 build: complete B0.4 startup shells
4e25359 build: complete M1 contest import
27f785f build: complete M2.1 personal note binding
8edc301 build: complete M2.2 fresh markdown read
6ae1cbf build: complete M2.3 binding resolution
6b80394 build: complete M2.4 revalidation and Obsidian open
091b525 build: complete M2.5 safe patch foundation
6172668 docs: record M2 checkpoint
```

本文件与 M3 实现将在用户明确授权的本地提交中一起封存。提交后应以 `git log -1 --oneline --decorate` 核对真实 commit；当前没有创建 M3 tag，也没有 push。

重要：`ACM-OS_RECOVERY_PROMPT.md` 在 M3 提交前已有用户拥有的本地修改。本次提交不得暂存或覆盖该文件，因此提交后的 working tree 预计仍会显示它为 modified。

## 3. Current BUILD position

```text
Completed: M0 — Executable Foundation + Workspace Ready Gate
Completed: M1 — Real Contest Import -> Lightweight Problems
Completed: M2 — Personal Vault Binding
Completed: M3 — Upsolve Lifecycle -> First Review Schedule
Next:      M4 planning only
M3 state:  COMPLETE / AUTOMATED TESTS PASS / DESKTOP ACCEPTANCE PASS
```

M3 冻结 Outcome：

```text
UNSTARTED
-> PENDING
-> LEARNING
-> Mark Understood
-> WAITING_COLD_START +3 local calendar days
```

冻结范围：

- `ProblemLifecycleEngine`；
- `learning_status_since`；
- Review Cycle 与 first due；
- withdraw / stop / relearn；
- Problem Header actions；
- Delete Personal Note 的 `AC-HISTORY-03` 行为。

主体 AC：`AC-PROBLEM-02` 至 `AC-PROBLEM-06`、`AC-HISTORY-03`。

M3 不包含 M4 的正式 Review Focus、Evidence 或 Judgement 流程。

## 4. M3 implementation record

### 4.1 Domain

- 新增 `LearningStatus`、`ProblemLifecycleAction`、`ReviewCycleDirective` 与 `ProblemLifecycleDecision`；
- `ProblemLifecycleEngine` 统一决定允许动作、下一状态和 Review Cycle 指令；
- 非法状态转换显式失败，不由 UI 猜测；
- `MarkUnderstood` 进入 `WaitingColdStart` 并启动首次冷启动周期；
- `WithdrawUnderstood` 返回 `Learning` 并取消 active cycle；
- `StopLearning` 返回 `Unstarted`；
- Delete Personal Note 从任意学习状态退出并取消 active scheduling；
- 本地日期值负责计算三个本地日历日后的 first due。

### 4.2 Application

- 新增 lifecycle read/transition port、typed state 与错误合同；
- transition 只接受强 Problem identity、Domain action 和调用者本地日期；
- Lightweight Problem 不能进入 Personal learning lifecycle；
- Delete Personal Note 使用独立语义命令，不伪装成普通生命周期按钮；
- Problem detail 将 identity type、learning state、active cycle 和 Domain 提供的 available actions 一并投影给 UI。

### 4.3 Infrastructure and schema

- migration `0005_create_learning_lifecycle.sql` 将 schema generation 升到 5；
- `problem_learning_states` 持久化当前状态与 `learning_status_since_utc`；
- `review_cycles` 持久化周期编号、状态、stage、规则版本、first due 与结束时间；
- partial unique index 保证每题至多一个 active review cycle；
- lifecycle 状态和 active cycle 在同一短 SQLite transaction 中原子更新；
- 重启后学习状态和 first due 保持；
- withdraw/stop 取消 active cycle，但不改变 Personal identity；
- Delete Personal Note 先 Fresh Read/解析绑定和 Vault，再执行文件删除与短数据库提交；数据库提交失败时恢复文件；
- 删除后降级为 Lightweight、学习状态回到 `Unstarted`、active schedule 取消，同时保留 Contest、Review 和 historical highest 相关历史事实；
- Vault 不可用时拒绝删除，不错误降级身份。

### 4.4 Tauri IPC and React

- 新增 typed lifecycle query/transition/delete IPC；
- Problem Header 只展示 Domain 当前允许的动作；
- UI 展示当前学习状态、first cold-start due、撤回补懂、停止学习与重新学习路径；
- 删除 Personal Note 有明确确认步骤与局部错误状态；
- React 不获得 filesystem 或 SQLite authority；所有事实仍由后端 Fresh Read 与 typed IPC 提供。

## 5. Acceptance-period fixes included in the M3 checkpoint

这些修复在真实 Windows 桌面验收中暴露，属于让已冻结的 M1/M2/M3 行为可用所需的缺陷修复，不改变里程碑 Authority。

### 5.1 Codeforces contest import

- canonical contest URL 现在同时接受有无尾部 `/` 的形式；
- standings metadata 使用有界流式读取，在巨大 participant rows 前停止，避免总超时；
- statement asset allowlist 支持 Codeforces 官方 `espresso.codeforces.com`，仍拒绝 lookalike host；
- contest URL 输入使用实时 input 事件并清除 stale error；
- Codeforces 2256 桌面导入结果：6 道题全部导入。

### 5.2 Local statement LaTeX

- 前端使用 `katex@0.18.1` 在本地渲染 Codeforces `$$$...$$$` 公式；
- 跳过 `pre`、`code` 和已渲染 KaTeX 节点；
- `trust: false`，不授予公式任意可信 HTML 能力；
- KaTeX CSS 和字体进入 production bundle；
- 公式渲染不改变本地 statement snapshot 的 Authority。

### 5.3 Personal Markdown and Obsidian

- 修复 React StrictMode effect cleanup 导致 ready projection 被丢弃、编辑入口不显示的问题；
- Personal Problem 显示“在 Obsidian 中打开并编辑题解”；
- URI 出口会把 Windows canonical verbatim path `\\?\E:\...` 转为普通 `E:\...`，并正确处理 `\\?\UNC\...`；
- canonicalize、Active Vault 边界与文件存在校验仍在 URI 构造前执行；
- Obsidian 必须已将 Active Vault 登记为 Vault；这属于 Obsidian 外部配置，不由 ACM-OS 绕过；
- “My note” 是 Markdown 结构投影，只显示 Known Sections、Solution Routes 与解析 warning，不显示完整题解正文。

## 6. Verification and desktop acceptance

提交前最后验证应重新执行并以本次命令输出为准：

```powershell
npm run build
npm run test:shells
npm run test:dom-shells
npm run check:boundaries
Set-Location src-tauri
cargo test --workspace
cargo check --workspace
Set-Location ..
git diff --check
```

本轮已取得的自动化证据：

- Rust workspace：95 passed，0 failed，2 个 release-only live-network smoke ignored；
- lifecycle Domain/Application/Infrastructure、重启持久化、撤回/停止、删除与历史保留测试：PASS；
- Obsidian URI targeted tests：3 passed；
- DOM shells：14 passed；
- startup shells：6 passed；
- boundary tests：5 passed，architecture checker：PASS；
- TypeScript type-check 与 Vite production build：PASS；
- `cargo check --workspace`：PASS；
- `git diff --check`：PASS。

用户于 2026-08-12 确认桌面人工验收全部通过，包括：

- contest 2256 完整导入；
- Personal Note 创建；
- lifecycle 主路径与 first due `+3d`；
- withdraw、重新补懂、stop；
- Delete Personal Note 确认、降级、schedule 取消与历史保留；
- 从 ACM-OS 打开 Obsidian 并编辑真实 Markdown；
- 外部编辑后 Known Sections / Solution Routes 结构投影刷新；
- 含 LaTeX 的本地题面显示。

## 7. Preserved authority boundaries

后续必须继续保护：

- Markdown 正文由 Markdown 权威拥有，SQLite 不覆盖正文；
- 不向 Markdown 注入 ACM-OS 私有 ID；
- Problem Identity 与 File Binding 分离；
- watcher 只是 invalidation/re-read trigger，不是事实源；
- authoritative read 必须 Fresh Read；
- write 必须 fresh read、唯一目标验证、minimal/byte-preserving patch、digest 并发检查、pre-write recovery copy、写后重读/重解析/语义验证；
- stale cache 不得覆盖外部编辑；
- Vault unavailable 只阻塞受影响 scope，不自动等于全局 Recovery；
- React 不获得 filesystem 或 SQLite Authority；
- 文件或网络 I/O 不进入长 SQLite transaction。

## 8. Explicitly deferred work

以下内容没有在 M3 实现，也不能被后续接手者误认为已完成：

- ACM-OS 内完整 Markdown 题解正文预览；
- 类似 Codeforces Better 的中文翻译与原文/中文切换；
- 任何翻译 provider、API key、翻译 cache 或正文改写；
- M4 Review Focus / Evidence / Judgement；
- M5+、M6 Candidate 产品 UI 或其他未来里程碑行为。

完整题解预览与中文翻译是用户提出的后续需求。翻译会扩大当前 SPEC，实施前必须先得到用户对 SPEC/PLAN 变更的明确授权；公式、代码块、原始 snapshot 与 Markdown Authority 必须继续受保护。

## 9. Exact next-session workflow

新会话第一阶段只读取证：

```powershell
Get-Location
git status
git branch --show-current
git log -10 --oneline --decorate
git remote -v
git tag --list
git rev-list -n 1 acm-os-m2-vault-binding
git ls-remote origin refs/heads/main refs/tags/acm-os-m2-vault-binding
```

然后：

1. 完整阅读四份权威文档；
2. 核对本地 M3 commit 的真实 hash、subject 和 change surface；
3. 识别 `ACM-OS_RECOVERY_PROMPT.md` 的用户拥有修改，不覆盖、不 reset、不误提交；
4. 重新核对 manifests、lockfiles、schema generation 5 与测试入口；
5. 确认 M3 验证证据仍可复现；
6. 在用户授权前不创建 `acm-os-m3-learning-lifecycle` tag，不 push；
7. 先完成 M4 planning，再按最小 Slice 实现；当前 Slice 失败时不叠加下一 Slice。

## 10. M4 starting boundary

下一步只允许进入 M4 planning。接手者必须从 SPEC/DESIGN/PLAN 重新提取 M4 冻结 Outcome、AC、DoD 和最小 Slice，不得根据 M3 UI 猜测 Review 产品行为。

特别禁止：

- 把 `WaitingColdStart` 直接扩展成未经规格确认的 Review Focus；
- 提前加入 Evidence/Judgement 写入；
- 把 Markdown projection 当成正文 Authority；
- 因翻译或正文预览需求改变 M4 的冻结顺序。

## 11. Git and safety status

- 用户已明确授权生成本交接文档并创建本地提交；
- 本次授权不包含 tag 或 push；
- 不得暂存 `ACM-OS_RECOVERY_PROMPT.md`；
- 禁止 `git reset --hard`、`git clean`、覆盖式 checkout 或丢弃未知用户内容；
- M3 tag 预定名为 `acm-os-m3-learning-lifecycle`，但只有在用户再次明确允许后才能创建；
- 远程仍以实际 `git ls-remote` 结果为准，不能把本地提交描述成已推送。

## 12. Durable recovery prompt

仓库中的恢复提示词位于：

`ACM-OS_RECOVERY_PROMPT.md`

该文件当前含用户拥有的未提交修改。本次 M3 commit 不包含它；未来是否提交、更新或推送必须由用户单独决定。
