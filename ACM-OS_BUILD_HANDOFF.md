# ACM-OS BUILD Handoff — M4 complete

> 更新时间：2026-08-12（Asia/Shanghai）
>
> 用途：聊天上下文丢失或切换开发者后，从真实 Windows 仓库安全恢复 BUILD 工作。
>
> 本文件是状态参考和接管清单，不替代 Git、冻结文档或实际命令输出。

## 1. Authority and recovery rule

权威顺序严格保持：

`SPEC > DESIGN > PLAN > IMPLEMENTATION`

接手者必须完整读取：

- `ACM-OS_SPEC_v1.md`
- `ACM-OS_DESIGN_v1.md`
- `ACM-OS_PLAN_v1.md`
- `ACM-OS_BUILD_HANDOFF.md`

SPEC 是产品事实源。不得根据当前 UI 自行扩展 Today、翻译、Markdown 正文预览或未来 Milestone。若实现与上位文档存在真实冲突，标记 `SPEC-CONFLICT` 并停止叠加实现。

## 2. Repository checkpoint

真实路径：

`E:\项目开发\acm-os`

生成本文件时的提交基线：

```text
Branch: main
Pre-M4 HEAD: 22caa0e build: complete M3 learning lifecycle
origin/main: 6172668 docs: record M2 checkpoint
M2 tag: acm-os-m2-vault-binding -> 091b525
M3 tag: not created
M4 tag: not created
Remote: https://github.com/Mxiaocao/acm-os.git
```

本文件与 M4 实现将按用户授权一起创建本地提交。提交完成后必须以 `git log -1 --oneline --decorate` 记录实际 hash；本次授权不包含 tag 或 push。

`ACM-OS_RECOVERY_PROMPT.md` 含用户拥有的未提交修改。本次提交明确排除该文件；提交后 working tree 预计仍显示它为 modified。禁止覆盖、暂存、提交、reset、checkout 或删除该修改。

## 3. Current BUILD position

```text
Completed: M0 — Executable Foundation + Workspace Ready Gate
Completed: M1 — Real Contest Import -> Lightweight Problems
Completed: M2 — Personal Markdown -> External Obsidian Fresh Read
Completed: M3 — Upsolve Lifecycle -> First Review Schedule
Completed: M4 — Review Focus -> Evidence -> Judgement
Next:      M5 — Today Planner -> Stable Daily Execution
M4 state:  COMPLETE / AUTOMATED ACCEPTANCE PASS / USER SIGN-OFF
```

M4 Outcome：到期 Problem 能进入受控 Review；系统从真实完成事实与不可变帮助证据自动推导 Judgement，并推进长期复习或回炉，同时保留不可覆盖的历史事实。

## 4. M4 implementation record

### 4.1 Domain and scheduling

- 建立 Review eligibility、Attempt type、completion facts、help evidence、failure reason、Judgement 与 mastery evidence 合同；
- Judgement 只能由系统推导，用户不能直接选择 `Mastered / Partial / Fail`；
- 最终 AC、独立思路、独立实现、独立调试且无解题帮助时才可 `Mastered`；第一次 WA 后独立修正不阻止 Mastered；
- L1–L4 解题帮助最高为 `Partial`，L5 full solution 强制 `Fail`；无最终 AC 或未完成同样为 Fail；
- Partial/Fail 必须包含原因并进入 `Relearning`，暂停 active schedule；
- 长期 Review 间隔冻结为 `3 -> 10 -> 30 -> 75 -> 150 -> 240 -> 240` 个本地日历日；
- Early Review 的 Mastered 会留下完成历史，但不改变原 stage 或 due；
- Thorough Digestion 只在六项真实证据全部满足时成立，并区分 current 与 historical highest。

### 4.2 Persistence and migrations

- `0006_create_review_attempts.sql`：Review Attempt、固定开始元数据与每题至多一个 IN_PROGRESS invariant；
- `0007_create_review_help_usage_events.sql`：不可变 Help Reveal 使用事件；
- `0008_complete_review_attempts.sql`：completion facts、Judgement、failure reasons、Evidence Card 与 sealed completion；
- `0009_create_problem_mastery_evidence.sql`：六项 mastery evidence、首次 6/6 日期与历史最高事实；
- Start-or-resume 不重复创建 Attempt，退出和重启后恢复同一 Attempt；
- Focus query 只投影题面、原始 OJ 和 Attempt 元数据，不泄漏旧知识；
- Reveal 在返回帮助正文前先持久化 evidence；只打开 Drawer 不计使用；
- complete transaction 原子封存 Attempt、写入 evidence、更新 lifecycle/schedule；无效或不完整表单保持 IN_PROGRESS；
- Completed/Void Attempt 均保留；Void 原因保留且不改变 schedule，既有 help evidence 不删除；
- 后续退步、回炉或删除 Personal Note 不覆盖既有 Mastered、首次 6/6 或历史最高事实；
- IN_PROGRESS Review 存在时阻止删除 Personal Note；完成后删除仍遵守 AC-HISTORY-03。

### 4.3 Application, IPC and UI

- 新增 start/resume、Focus、Help Drawer、Reveal、complete、Void、Attempt/Problem history 与 mastery evidence typed contracts；
- Problem Detail 根据后端投影显示 Start Review、Start Early Review 或 Continue Review；
- Review 使用隔离 Focus shell，隐藏普通导航、Personal Markdown、旧思路/代码、Contest history、AI 和 Obsidian 入口；
- facts form 保留未完成输入，并展示后端语义错误；完成后显示不可编辑 Evidence Card；
- Problem Detail 展示 Review history、historical best、current/historical Thorough Digestion；
- Help Drawer、Reveal confirmation 与 Void dialog 具备初始焦点、Tab/Shift+Tab 焦点约束、Escape 关闭和触发点焦点返回；
- React 不获得 SQLite、filesystem 或 Markdown Authority。

### 4.4 Markdown help projection

- Review Help 只从冻结的 Known Sections / Solution Routes 结构中解析明确可揭示内容；
- 空、重复、歧义或跨顶级 section 的内容不被猜测为可用帮助；
- Drawer 元数据与真正 reveal 内容分离，避免查询阶段泄漏正文；
- Markdown 仍拥有正文 Authority，SQLite 只保存使用证据与系统事实。

## 5. M4 acceptance coverage

已覆盖并签收：

- `AC-REVIEW-01`：合法到期/提前 Review 创建 Attempt；
- `AC-REVIEW-02`：退出后恢复同一 IN_PROGRESS Attempt，不判失败；
- `AC-REVIEW-03`：数据库与应用层禁止并行 Attempt；
- `AC-REVIEW-04`：Evidence-before-Reveal；打开 Drawer 不计使用；
- `AC-REVIEW-05`：无帮助且事实满足时自动 Mastered；
- `AC-REVIEW-06`：AC 但使用帮助或独立性不足时 Partial + reason + Relearning；
- `AC-REVIEW-07`：L5/full solution 强制 Fail；
- `AC-REVIEW-08`：无最终 AC 强制 Fail + reason；
- `AC-REVIEW-09`：缺失/矛盾事实不能结束 Attempt；
- `AC-REVIEW-10`：首次通过进入长期调度；
- `AC-REVIEW-11`：长期退步进入 Relearning，历史不可覆盖；
- `AC-HISTORY-01`：Completed/Void Attempt append-only 保留；
- `AC-HISTORY-02`：历史最高 Review 与首次 Thorough Digestion 保留；
- `AC-HISTORY-03`：删除 Personal Note 时保留历史，并阻止与 IN_PROGRESS Review 冲突。

DoD 两条主链均通过：

1. Due/early Review -> isolated Focus -> facts/evidence -> Mastered -> frozen long-term schedule；
2. Help/no-AC/incomplete independence -> Partial/Fail -> reason -> Relearning，且 Completed history 不被后续状态覆盖。

## 6. Verification evidence

2026-08-12 收口验收实际通过：

```text
Rust root IPC:        18 passed
Rust application:     11 passed
Rust domain:          14 passed
Rust infrastructure:  73 passed, 2 ignored
Rust total executed: 116 passed, 0 failed
DOM shells:           17 passed, 0 failed
Startup/routing:       6 passed, 0 failed
Boundary tests:        5 passed, 0 failed
Architecture checker: passed
TypeScript:            passed
Vite production build: passed
cargo check --workspace: passed
git diff --check:      passed
```

两个 ignored 测试是明确标记为 release-only、需要真实网络的 Codeforces smoke，不是本地失败。

用户于 2026-08-12 根据功能状态和验收清单确认 M4 收口。验收期间发现的 modal/drawer 焦点管理缺口已修复并加入 DOM 回归测试。

已知非阻塞提示：Vite 主 JS chunk 约 503 kB，超过默认 500 kB warning 阈值；当前不影响 M4 正确性。Windows working copy 存在 LF/CRLF 提示，但 `git diff --check` 通过。

## 7. Preserved authority boundaries

后续必须继续保护：

- Markdown 正文由 Markdown Authority 拥有，SQLite 不覆盖正文；
- 不向 Markdown 注入 ACM-OS 私有 ID；Problem Identity 与 File Binding 分离；
- watcher 只是 invalidation/re-read trigger，authoritative read 必须 Fresh Read；
- write 必须唯一目标验证、minimal patch、digest 并发检查、pre-write recovery copy、写后重读/重解析/语义验证；
- stale cache 不得覆盖外部编辑；Vault unavailable 只阻塞受影响 scope；
- Contest Result、Learning Status、Review Judgement 和 Today Entry 不互相冒充 Authority；
- React 不直接读写 filesystem、SQLite 或网络事实；
- 文件或网络 I/O 不进入长 SQLite transaction；
- Completed Review、Help Reveal、历史最高 evidence 与首次 6/6 日期不可被当前状态覆盖。

## 8. Explicitly deferred work

以下不是 M4 缺陷，也没有在 M4 实现：

- 独立的补题队列聚合页面；
- M5 Today Plan、budget、候选生成、当天稳定 snapshot、排序和未来 recall；
- ACM-OS 内完整 Markdown 题解正文预览；
- 类似 Codeforces Better 的中文翻译、provider、API key 或翻译 cache；
- M6+ Knowledge、完整 Contest Workflow、Recovery hardening 和 Release E2E。

中文翻译会扩大当前 SPEC。未经用户明确批准，不得直接实现或修改 SPEC。

## 9. Exact next-session workflow

新会话第一阶段先执行真实恢复核对：

```powershell
Get-Location
git status
git branch --show-current
git log -10 --oneline --decorate
git remote -v
git tag --list
git tag --points-at HEAD
git rev-parse HEAD
git rev-parse origin/main
git ls-remote origin refs/heads/main refs/tags/acm-os-m4-review
```

然后：

1. 完整读取四份权威文档；
2. 以 Git 核对 M4 commit 的真实 hash、subject 和 change surface；
3. 识别并保护 `ACM-OS_RECOVERY_PROMPT.md` 的用户修改；
4. 核对 migrations 0006–0009、schema generation 与测试入口；
5. 必要时重跑第 6 节验收命令；
6. 未经明确授权不创建 `acm-os-m4-review` tag、不 push；
7. 从 SPEC/DESIGN/PLAN 提取 M5 Outcome、AC、DoD，先 planning，再按最小 Slice 实现。

## 10. M5 starting boundary

下一阶段是 `M5 — Today Planner -> Stable Daily Execution`。M5 才负责从真实补题/回炉/Review 候选与 budget 生成稳定、确定、可解释的当天执行视图，并在未来 recall Problem。

开始 M5 前必须明确区分：

- Problem Learning Status 是长期生命周期 Authority；
- Review schedule 是复习到期事实；
- Today Plan 是按日期持久化的临时执行视图，不是第三个长期队列；
- Today 排序、跳过或“今日先到这里”不能改写真实 Learning Status 或制造失败 Review；
- 当前 Problems 页面是题目索引，不等于冻结的补题队列/Today Planner 已实现。

M5 完成后必须通过冻结的 Automated Contest -> Markdown -> Learning -> Review -> Today core-loop gate，未通过不得继续扩展 M6+。

## 11. Git and safety status

- 用户已授权更新本交接文档并创建 M4 本地提交；
- 本次授权不包含 tag 或 push；
- `ACM-OS_RECOVERY_PROMPT.md` 必须保持 unstaged、uncommitted；
- 禁止 `git reset --hard`、`git clean`、覆盖式 checkout 或丢弃未知用户内容；
- M4 预定 checkpoint tag 为 `acm-os-m4-review`，仅在用户再次明确授权后创建；
- 远程状态必须以实际 `git ls-remote` 为准，不得把本地提交描述为已推送。

## 12. Durable recovery prompt

恢复提示词位于 `ACM-OS_RECOVERY_PROMPT.md`。该文件当前含用户拥有的未提交修改，本次 M4 commit 不包含它；未来是否更新、提交或推送必须由用户单独决定。
