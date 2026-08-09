# ACM-OS PLAN v1

> 状态：PLAN 冻结版，可进入 BUILD  
> 日期：2026-08-09  
> 产品：ACM-OS  
> 上游产品事实来源：`ACM-OS_SPEC_v1.md`  
> 上游设计事实来源：`ACM-OS_DESIGN_v1.md`  
> 文档目标：把已冻结的 SPEC + DESIGN 转换为可安全、增量、可验证执行的工程计划；后续 BUILD 不再临场决定产品行为。

---

# 0. 文档地位与变更规则

权威优先级固定为：

```text
SPEC > DESIGN > PLAN > IMPLEMENTATION
```

本 PLAN 只决定 DESIGN 明确留给 PLAN / IMPLEMENTATION 的工程问题，不重新解释产品行为。

若 BUILD 阶段出现以下情况：

1. PLAN 与 SPEC 直接冲突；
2. PLAN 与 DESIGN 冲突，而 DESIGN 与 SPEC 一致；
3. 实现无法满足已冻结的数据完整性、恢复能力或验收标准；
4. 某开发任务无法在 SPEC / DESIGN / PLAN 中找到需求或 enabling rationale；

必须停止对应实现并标记：

```text
SPEC-CONFLICT
```

不得为了实现方便偷偷修改产品规则。

POST-MVP 继续冻结，不进入 MVP 阻塞路径：Rewards Shop、完整“我的出题”、高级统计 Dashboard、Global Search、自研 Knowledge Graph、多设备同步、主动通知、实时 AI Chat / Hint / Debug、重游戏化、复杂 Contest Tag、完整 Markdown 版本审计等。

---

# 1. PLAN 总体结论

ACM-OS MVP 使用 Windows-first 的 Tauri 2 Desktop 架构：

```text
React + TypeScript + Vite
        ↓ typed business IPC
Tauri Desktop Shell
        ↓
Rust Application Core
        ├─ Domain Engines
        ├─ Application Use Cases
        ├─ SQLite / SQLx Persistence
        ├─ Vault / Markdown Engine
        ├─ Contest Adapters
        ├─ Backup / Recovery
        └─ Platform Integration
```

数据权威保持：

```text
Obsidian Markdown
= long-term knowledge-content truth

SQLite System Facts
= identity / history / lifecycle / schedule truth

Derived Cache
= disposable / rebuildable

React
= non-authoritative presentation + interaction
```

质量优先级继续继承 DESIGN：

```text
Data Integrity
> Recoverability
> Reliability
> User Control
> Accessibility
> Performance
> Visual Polish
```

MVP 正式阻塞平台：

```text
Windows 10/11 x64
```

macOS / Linux 只保留架构扩展可能，不作为 MVP Blocking Platform。

---

# 2. PLAN 01 — 技术栈冻结

## 2.1 Desktop Shell

```text
Tauri 2
```

理由：保留 Rust authoritative backend、系统 WebView、细粒度 capability/IPC 边界，并与本地文件、SQLite、Windows 文件身份集成保持自然边界。

## 2.2 UI

```text
React + TypeScript + Vite
```

Frontend 只负责：

- 页面与交互；
- URL state；
- form draft；
- ephemeral UI state；
- Core Query View 的展示缓存。

Frontend 不拥有：

- Learning Status；
- Review Judgement；
- Today Planning；
- SQLite schema；
- Vault write authority。

## 2.3 System Facts Storage

```text
SQLite + Rust SQLx
```

Frontend 禁止直接 SQL。

所有 System Fact mutation 必须经过 Rust Application Use Case。

## 2.4 HTTP / External Integration

外部 Contest / OJ 网络能力位于 Rust Infrastructure。Codeforces Adapter 使用共享 HTTP client / rate-limit boundary；React 不直接承担外部抓取逻辑。

## 2.5 BUILD-DEFERRED

以下不在 PLAN 锁 exact patch：

- Tauri / React / Vite / SQLx / reqwest patch version；
- npm vs pnpm；
- Router / Query / Form / Component library；
- exact Rust crate feature flags；
- exact lockfile strategy。

BUILD 创建仓库时必须重新查当前官方文档并锁定实际版本。

---

# 3. PLAN 02 — Repository 与模块边界

采用：

```text
Single Repository
+ Modular Monolith
+ compile-time Rust boundaries
```

建议一级结构：

```text
acm-os/
├─ src/                         # React frontend
│  ├─ app/
│  ├─ features/
│  ├─ shared/
│  └─ ipc/
│
├─ src-tauri/
│  ├─ src/                      # thin Tauri shell / composition root
│  └─ crates/
│     ├─ acm-os-domain/
│     ├─ acm-os-application/
│     └─ acm-os-infrastructure/
│
├─ tests/
│  ├─ fixtures/
│  └─ e2e/
└─ docs/
```

具体目录名属于 BUILD-DEFERRED；依赖方向不是。

## 3.1 Domain

Domain 只保存纯规则：

- Problem lifecycle；
- Review eligibility；
- Review judgement；
- Review scheduling；
- Today candidate / planner；
- history invariants。

Domain 禁止依赖：

```text
Tauri
React
SQLite / SQLx
filesystem
HTTP
Codeforces
Windows API
```

## 3.2 Application

Application 是唯一业务编排层，负责：

- load current authoritative facts；
- 调用 Domain Decision；
- 控制事务边界；
- 跨 SQLite / Vault / Adapter 协调；
- 把 Infrastructure error 转成 Application error；
- 产出 Query View / Command Result。

## 3.3 Infrastructure

实现 Application Ports：

```text
Persistence
Vault / Markdown
Contest Adapter
Platform / External Open
Backup / Recovery
Diagnostics
Clock / Local Calendar
File Identity
```

Infrastructure 不得自行组织产品工作流。

## 3.4 Tauri Shell

Tauri 只负责：

```text
Deserialize request
→ Call Application Use Case
→ Serialize typed result
```

禁止把核心业务规则塞进 command handler。

## 3.5 IPC Contract

冻结：

```text
IPC DTO ≠ Domain Model ≠ Persistence Model
```

IPC 只暴露 coarse-grained business commands / views，不暴露 generic SQL / generic file write。

---

# 4. PLAN 03 — Persistence / Schema / Migration

## 4.1 Persistence Model

采用：

```text
Current State
+ Required Immutable History
```

不采用 Full Event Sourcing。

## 4.2 Stable ID

核心对象使用 ACM-OS 内部稳定 UUIDv7：

- Problem；
- Contest；
- Review Attempt；
- Review Cycle；
- Knowledge Node；
- Today Plan；
- File Binding；
- Critical Operation；
- Backup。

External Identity 与内部 ID 分离。

## 4.3 Logical Schema

### Problem / Identity

```text
problems
problem_external_identities
problem_statement_snapshots
problem_mastery_evidence
```

Strong External Identity 必须有数据库唯一约束。

### Contest

```text
contests
contest_external_identities
contest_problem_records
contest_correction_events
post_contest_analysis
```

Contest 三维状态保持分离：Import Completeness / Facts Organization / Archive。

### Review

```text
review_attempts
review_help_usage_events
review_failure_reasons
review_void_events
review_cycles
```

Completed Attempt + Help Usage 作为不可静默覆盖历史。

### Today

```text
today_plans
today_plan_entries
```

同一 Local Calendar Date 最多一个 active Today Plan Snapshot。

### Knowledge

```text
knowledge_nodes
knowledge_candidate_records
```

Problem→Knowledge / Knowledge→Knowledge formal relation 不作为 SQLite Authority；只允许 derived index。

### Vault / Recovery

```text
file_bindings
critical_operations
markdown_recovery_metadata
```

### Backup / Settings

```text
backup_metadata
workspace_settings
planning_settings
```

### Derived / Rebuildable

```text
markdown_projection_cache
problem_knowledge_index
knowledge_link_index
```

Derived tables 可删可重建，不能成为事实源。

## 4.4 Required DB Invariants

数据库层必须尽量直接保护：

- 一个 Problem 最多一个 IN_PROGRESS Review；
- Strong External Identity 唯一归属；
- 一个 Problem 最多一个 First Statement Snapshot；
- 一个 Problem 最多一个 active Review Cycle；
- 一个日期最多一个 active Today Plan；
- 一个 Object 最多一个 Primary File Binding。

具体 UNIQUE / partial index / CHECK / FK 写法 BUILD 决定。

## 4.5 Date vs Instant

Review due / Today date / weekday：

```text
Local Calendar Date
```

Attempt started/completed / correction / reveal：

```text
Event Instant
```

二者不能混用。

## 4.6 DB Transaction Boundaries

必须单事务：

- Review Completed → Attempt + Failure Reasons + Problem + Cycle + Schedule；
- Mark Understood → Problem + new Cycle + +3 due；
- Review Failure → Attempt + Problem RELEARNING + suspended Cycle；
- Contest Correction → current fact + Correction Event；
- Today initial generation → Plan + all initial Entries；
- coherent Contest import item → resolve / relation / snapshot state。

## 4.7 Cross-Authority Crash Recovery

SQLite 与 Markdown 之间采用：

```text
Critical Operation Journal
→ Markdown write + semantic verify
→ final System Fact commit
```

启动时检查 PENDING operation：

- 文件仍为 pre-state：安全放弃；
- 文件精确满足 postcondition：完成 DB commit；
- 文件已被外部再次修改：NEEDS_RECOVERY，不猜。

## 4.8 Migration

```text
Forward-only embedded SQLx migrations
```

历史 migration 不编辑，只新增 migration。

启动：

```text
inspect schema
→ pending?
→ pre-migration backup
→ migrate
→ integrity verify
→ normal startup
```

失败进入 Recovery Mode。

旧 App 遇到更高 schema version：阻塞正常启动，不自动 down migration。

## 4.9 Backup / Restore

System Facts backup 必须使用 SQLite-consistent snapshot，不直接复制 live `.sqlite` 文件。

保留：

```text
7 Daily + 4 Weekly
```

另外：pre-migration / pre-restore / manual backup。

Restore：

```text
pre-restore backup
→ restore System Facts
→ migrate if needed
→ integrity check
→ validate bindings
→ Fresh Read current Markdown
→ rebuild derived projection
```

System Restore 永远不覆盖 Markdown。

---

# 5. PLAN 04 — Markdown / Vault Engine

## 5.1 Parser

采用：

```text
pulldown-cmark source offsets
+ ACM-OS narrow Obsidian extension parser
```

CommonMark 负责结构，ACM-OS extension 只解析必要 WikiLink / known sections。

不采用整份 AST re-serialize 作为正常写入方式。

## 5.2 Known Sections

Problem Markdown 识别：

```text
## 前置知识
## 题解
## 额外题目

optional:
## Hints
## 思路
## 代码
```

Solution Route：只认 `## 题解` 下直接 `###`；`####` 不是 Route。

## 5.3 WikiLink

支持必要 Obsidian 形式：target / anchor / alias。

Resolution 结果：

```text
RESOLVED
UNRESOLVED
AMBIGUOUS
NON_KNOWLEDGE_TARGET
```

不 fuzzy 自动选择。

## 5.4 File Binding / Relocation

稳定身份与文件绑定分离。

确定性顺序：

```text
original path
→ unique Windows file key
→ unique full-content digest
→ LOCATION_ANOMALY / manual recovery
```

filename / H1 / similarity 仅用于候选提示，不自动绑定。

Windows MVP 的 file key 使用可靠 platform file identity evidence（volume + file id 语义）；它仍不是 Problem Identity。

## 5.5 Fresh Read

正常打开 Problem / Knowledge：

```text
Resolve binding
→ read current disk bytes
→ digest
→ reuse cached parse only if digest matches
```

Cache 不能跳过真实磁盘读取。

## 5.6 Cache

建议 key：

```text
(binding_id, content_digest, parser_contract_version)
```

WikiLink parsing 与 Knowledge target resolution 分层，Knowledge index 变化可重算 resolution，不必重 parse 文件正文。

## 5.7 Watcher

采用 `notify` 类 native watcher：

```text
watcher event
→ invalidate affected cache
→ schedule Fresh Re-read
```

Watcher 不是事实源。

漏 event 只影响自动刷新速度，不能造成旧缓存覆盖真实 Markdown。

窗口重新获得 focus 时 revalidate 当前可见 bound file。

## 5.8 Safe Patch

Frontend / Application 只能发语义操作，例如：

```text
AddPrerequisiteLink
AddExtraProblem
UpdateExtraProblem
RemoveExtraProblem
CreateKnownSection
```

禁止 generic `write_markdown(path, new_text)`。

完整链：

```text
User explicit action
→ Resolve Binding
→ validate path inside Active Vault
→ Fresh Read
→ digest PRE
→ Parse latest
→ Unique target validation
→ Build semantic patch
→ Pre-write recovery copy
→ Re-read / digest concurrency check
→ byte-preserving splice
→ temp write + flush + safe replace
→ Re-read
→ Re-parse
→ Verify semantic postcondition
→ refresh binding evidence
→ Application commit corresponding System Fact
```

目标区域之外必须 byte-for-byte 保持。

保留原文件 LF/CRLF 与 UTF-8 BOM。

非可靠 UTF-8：禁止自动写。

## 5.9 Concurrent External Edit

若 Fresh Read 后到真正 write 前 digest 已变化：

```text
Cancel
→ MARKDOWN_CONCURRENT_MODIFICATION
→ Fresh Read
→ user retries
```

MVP 不做自动 merge。

## 5.10 Recovery Copy

每次 ACM-OS 写 Problem Markdown 前生成 recovery copy。

每文件：

```text
max 10 copies
max 30 days
```

自动 Undo 只有 current digest 仍等于 ACM-OS 刚写成功的 digest 时允许；用户后来又编辑过则禁止自动覆盖。

## 5.11 Security

所有写入基于 resolved filesystem target 验证仍位于 Active Vault；path escape / junction / symlink escape 必须阻断。

---

# 6. PLAN 05 — Contest Adapter / Import Pipeline

## 6.1 Architecture

```text
External Source
→ Adapter
→ Validate
→ Canonical Draft
→ Application Resolver
→ Persistence
```

Adapter 不直接写 SQLite、不创建业务对象、不决定 merge。

## 6.2 Codeforces MVP

自动支持：

```text
Codeforces Public Contest
```

结构化 Contest / Problem metadata 通过官方 API；完整 Statement 通过公开 Problem Page 捕获。

原因：Codeforces API Problem metadata 本身不提供完整 statement body / input / output / samples / notes。

## 6.3 Strong Identity

Codeforces：

```text
Contest = (platform, contestId)
Problem = (platform, contestId, index)
```

URL / title / similarity 均不是 identity。

## 6.4 Locator Security

用户 URL：

```text
parse + validate supported host/path
→ extract external identity
→ Adapter constructs its own request URL
```

禁止把 Adapter 变成任意 URL downloader / SSRF 通道。

## 6.5 Manifest-first

新自动 Contest：

```text
Fetch complete manifest
→ Validate full ordered problem list
→ Persist Contest(INCOMPLETE) + all Slots
```

Manifest 都拿不到时，不创建一个未知题数的正式空 Contest。

第一次成功 manifest 成为 Retry 基准；普通 Retry 不 silent refresh remote structure。

## 6.6 Progressive + Idempotent

每个 Slot：

```text
Strong Problem Resolution
→ reuse / create Lightweight Problem
→ first snapshot exists?
   ├─ yes: never refresh
   └─ no: fetch / validate / persist first snapshot
```

Partial failure 保留成功项；Retry Missing Only。

网络请求期间不持有长 SQLite transaction。

## 6.7 First Statement Snapshot

保存：

```text
Immutable Source Capture
+ Canonical Sanitized Projection
+ Local Asset Bundle
```

Source Capture 可支持未来从同一首次来源重新 parse，但不能重新抓远程新版本冒充“首次快照”。

必要 asset 本地化，避免长期依赖远程链接。

Raw external HTML 永不直接 render。

## 6.8 Manual Contest

Manual Draft Builder 产出相同 Canonical Import Contract。

有可确定 Strong Identity：复用已有 Problem。

只有弱证据：不猜，创建独立对象或等待用户确认。

Manual Statement 最终必须得到与自动 Adapter 同一 Statement Snapshot Contract。

## 6.9 Adapter Health / Retry

External Source：

```text
AVAILABLE / DEGRADED / UNAVAILABLE
```

Transient failure 可 bounded retry；identity mismatch / malformed semantic data 等 integrity failure 不无限重试。

Codeforces API 请求由 Adapter 统一 rate-limit；具体等待值、并发与 timeout 在 BUILD 根据当前官方 contract 锁定。

---

# 7. PLAN 06 — Review / Scheduling / Today Domain Engines

全部采用 deterministic pure engines：

```text
ProblemLifecycleEngine
ReviewEligibilityEngine
ReviewJudgementEngine
ReviewSchedulingEngine
TodayCandidateBuilder
TodayPlanner
TodayReplanner
```

Engine 不读 DB、不读 FS、不做 HTTP、不直接读 system clock。

时间通过明确 LocalDate / Instant input 注入。

## 7.1 Problem Lifecycle

状态：

```text
UNSTARTED
UPSOLVE_PENDING
LEARNING
WAITING_COLD_START
RELEARNING
LONG_TERM_REVIEW
```

关键 transition：

```text
UNSTARTED --JoinUpsolve--> UPSOLVE_PENDING
UPSOLVE_PENDING --StartLearning--> LEARNING
LEARNING --ReturnToPending--> UPSOLVE_PENDING
LEARNING --MarkUnderstood--> WAITING_COLD_START + new Review Cycle(+3)
WAITING_COLD_START --Withdraw--> LEARNING + cancel current cycle
RELEARNING --StartRelearning--> LEARNING
eligible states --StopLearning--> UNSTARTED
```

非法 transition 返回显式 error，不 silent no-op。

`learning_status_since` 独立于 generic `updated_at`。

## 7.2 Delete Personal Note — Coverage Repair

为补齐 AC-HISTORY-03，本 PLAN 明确把正式个人笔记删除行为放入 M3：

前提：

- Personal Problem；
- 无 IN_PROGRESS Review；
- 用户明确执行 destructive action + Consequence Preview。

结果：

```text
delete bound Personal Markdown
→ verify deletion semantics
→ identity type becomes Lightweight
→ exit current learning lifecycle to UNSTARTED
→ cancel active schedule if any
→ preserve Contest history
→ preserve Completed Review history
→ preserve historical highest evidence
```

M4 加入 IN_PROGRESS Review 后，必须以 Application + DB invariant 阻止相冲突删除。

这只是对已冻结 SPEC 行为补明确 Milestone 归属，不改变产品行为。

## 7.3 Review Start

若已有 IN_PROGRESS Attempt：resume same attempt。

否则 WAITING_COLD_START / LONG_TERM_REVIEW 可开始；未到期主动开始为 EARLY_CHECK。

Attempt start 冻结：

- attempt type；
- scheduled due；
- started early；
- judgement rule version。

Schedule 数据缺失时返回 integrity error，不自动脑补历史。

## 7.4 Hidden Knowledge Isolation

Review initial IPC payload 只含：

- complete first statement snapshot；
- original OJ action；
- attempt metadata。

未 Reveal 的 prerequisites / hints / old idea / old code / solution / histories 不发送到 frontend，因此也不进入 DOM / Accessibility Tree。

## 7.5 Evidence Before Reveal

真正 Reveal：

```text
Fresh Read current Markdown
→ resolve requested content
→ durable HelpUsageEvent commit
→ only then release content to UI
```

Level 1–4 最高半会；Level 5 只能未通过。

Help Event 不提供撤销；整个 Attempt 可 Void，但历史保留。

## 7.6 Completion Facts

严格结构化：

```text
SubmissionFacts
IndependenceFacts
ExternalHelpFacts
FailureReasons
```

矛盾事实拒绝完成。

半会 / 未通过必须至少 1 个 failure reason。

## 7.7 Review Judgement

优先级：

```text
No final AC
→ FAIL

Full-solution-level help
→ FAIL

Final AC + solving help / non-independent key step
→ PARTIAL

Final AC + idea independent + implementation independent
+ debug independent/not-needed + no solving help
→ MASTERED
```

第一发 WA 本身不降级。

Engine 同时返回 evidence codes，由 UI 解释，不在 React 重新推导理由。

## 7.8 Review Completion Transaction

```text
load Attempt + immutable Help Events
→ validate facts
→ judge
→ validate failure reasons
→ scheduling decision
→ lifecycle transition
→ one SQLite transaction
```

Today reconciliation 不进入这个 authoritative transaction。

## 7.9 Scheduling

固定：

```text
3 → 10 → 30 → 75 → 150 → 240 → 240 days
```

Mark Understood：新 Cycle stage 0，due = today + 3。

正式到期真会：stage +1，next due 从实际 completed LocalDate 计算。

Early Check 真会：记录 Attempt，但 stage / due 不变。

任意 PARTIAL / FAIL：Problem → RELEARNING；Cycle suspended；无 due；重新学习后再次 Mark Understood 创建新 Cycle，从 +3 开始。

Overdue：只保留一个 due candidate，不制造欠债。

无需后台 scheduler/service。

Schedule Rule Version 只影响未来新 Scheduling Decision，不静默重算已存在 due。

## 7.10 Today

分：

```text
TodayCandidateBuilder
TodayPlanner
```

Candidate：

- Carry-in：IN_PROGRESS Review、LEARNING；
- Review Lane：due WAITING_COLD_START、due LONG_TERM_REVIEW；
- Study Lane：RELEARNING、UPSOLVE_PENDING。

Review 排序：overdue 更久 → pinned → first cold start → stable tie-break。

Study 排序：RELEARNING → UPSOLVE_PENDING → pinned → waiting longer → stable tie-break。

Planning Cost：

```text
Review = 30m
Upsolve/Relearn = 60m
```

只是预算块，不记录真实耗时。

Anti-starvation：

- Carry-in 优先；
- 若两类 backlog 存在且预算可各容纳一个，至少各一个；
- 若预算只能容纳一类，在存在 Study backlog 的条件下最多连续 2 个 generated Review-only days，第 3 个受约束日给 Study slot；
- 无 Study backlog 的 Review-only day 不累计 starvation debt。

Existing Today Plan：load + reconcile，不重新 generate。

预算变化：preview → user apply，只自动调整 AUTO + NOT_STARTED entries。

Drag / keyboard reorder：只改当日 order；重开不洗牌。

Today completion 不等于 Learning completion。

---

# 8. PLAN 07 — UI Implementation Map

## 8.1 Shells

```text
Setup Shell
Normal App Shell
Review Focus Shell
Recovery Shell
```

Startup Gate：

```text
System Facts unsafe?
→ Recovery

else Workspace Ready?
→ no: Setup
→ yes: Normal / Today
```

Vault temporarily unavailable 不等于 Recovery Shell；普通 App 可降级运行 System Facts 部分。

## 8.2 Routing

对象 route 使用 Stable Internal ID。

逻辑 route：

```text
/today
/contests
/contests/:contestId
/problems
/problems/:problemId
/knowledge
/knowledge/:knowledgeId
/settings
/review/:attemptId
/setup
/recovery
```

Review 绑定 Attempt ID，不绑定 Problem title/path。

## 8.3 Query / Command Contract

采用轻量 CQRS 风格：

```text
Query = read purpose-built View
Command = request business change
```

React 不直接拿 ORM row / Domain Entity 后自己重建业务规则。

Mutation 默认不做 authoritative optimistic update：Core commit 成功后刷新相关 View。

## 8.4 Frontend State

只分：

- URL state；
- Core Query state（cache is non-authoritative）；
- Form Draft state；
- Ephemeral UI state。

不建立巨大 frontend System Facts mirror。

## 8.5 Error Scope

五级：

```text
Blocking Startup
Global Dependency Health
Page Error
Region Partial Error
Action Error
```

局部失败只阻塞受影响区域。

统一 Application Error Contract 至少提供 machine-readable code / scope / safe message / retryability / diagnostic id / recovery actions。

## 8.6 Core Event Bridge

只用于 invalidation：

```text
ProblemChanged
MarkdownProjectionChanged
ContestImportChanged
TodayPlanChanged
KnowledgeIndexChanged
SystemHealthChanged
```

Frontend 收到 event 后重新 Query Core，不自己做业务推导。

## 8.7 Accessibility

作为组件合同，而非最后补：

- native semantics first；
- keyboard-equivalent Today reorder；
- visible logical focus；
- dialog/drawer focus management；
- field error association；
- async live announcement；
- status 不只靠颜色；
- 200% zoom core flow 可用；
- reduced motion；
- Review hidden content not fetched / not rendered。

## 8.8 Visual Direction

实现应继承 DESIGN 的“数字书房 + 轻游戏化”与成熟玫瑰粉 / 灰粉方向；M0 建立最小 design tokens，M9 再做一致性与视觉 polish。不得借视觉阶段引入 POST-MVP 功能。

---

# 9. PLAN 08 — Testing Strategy

测试证据链：

```text
T0 Static / Build Gates
T1 Domain Unit + Contract
T2 Application + Persistence Integration
T3 Temporary Vault Integration
T4 Contest Adapter Contract
T5 UI / Accessibility Behavior
T6 Automated Desktop E2E
T7 Real Blocking E2E
```

原则：

- 能在纯函数证明的，不依赖 E2E 才证明；
- 只有真实集成能证明的，不用 Mock 冒充；
- 不追求任意 line coverage 数字替代 requirements coverage；
- 数据完整性 bug 必须先有 regression test；
- flaky 不允许靠“重跑几次”解决。

## 9.1 T1 Domain

覆盖：

- lifecycle legal / illegal transitions；
- judgement golden cases；
- completion facts invariants；
- schedule chain / early / overdue / reset；
- Today candidate legality；
- Today ordering / anti-starvation / determinism；
- property / generative invariants。

## 9.2 T2 Persistence

使用真实临时 SQLite：

- Review atomicity；
- Mark Understood atomicity；
- Contest Correction atomicity；
- Today Snapshot atomicity；
- DB unique invariants；
- cascade safety；
- safe cleanup；
- migration；
- backup / restore。

## 9.3 T3 Temporary Vault

自动测试每次创建临时真实 Vault，禁止碰用户真实 Vault。

覆盖：

- stale cache + Fresh Read；
- parser fixtures；
- byte-preserving patch；
- partial parse warnings；
- concurrent edit；
- relocation path/file-key/digest/ambiguous；
- Vault unavailable；
- path escape；
- write/verify failures；
- recovery copies；
- Critical Operation crash matrix；
- watcher integration smoke。

## 9.4 T4 Adapter

CI 使用固定 API JSON / statement HTML / asset fixtures，不依赖实时 Codeforces。

覆盖：

- complete import；
- duplicate；
- Problem reuse；
- partial import；
- retry missing only；
- first snapshot no-overwrite；
- manifest stability；
- URL security；
- statement sanitize / identity validation；
- rate-limit fake-clock behavior。

Release 单独跑真实 Codeforces smoke。

## 9.5 T5 UI / Accessibility

验证用户行为，不重复测试 Domain 算法：

- startup routing；
- Review hidden data absent from DOM/accessibility tree；
- exit / resume same Attempt；
- missing facts form preserves draft + focus error；
- Today drag / keyboard reorder persistence；
- partial error scope；
- Vault banner；
- consequence preview；
- automated a11y + manual keyboard/zoom/tree/contrast/reduced-motion gate。

## 9.6 T6 Automated Desktop E2E

真实 Tauri Test Build + Test SQLite + Temporary Vault + Fake Contest HTTP + Test Clock + Controlled External Opener。

必须穿过真实 Application / Persistence / Vault，只替换不可控 Internet / Real OJ / Wall Clock。

M5 完成后必须先跑通自动核心链。

## 9.7 T7 Real Blocking E2E

Release Candidate 必须真实经过：

- real Codeforces Contest；
- real Obsidian edit；
- real multi-day cold-start due；
- real original OJ submission；
- later Today recall。

不得用 `final_ac=true` debug shortcut 替代。

---

# 10. PLAN 09 — Milestones / Vertical Slices

## M0 — Executable Foundation + Workspace Ready Gate

Outcome：

```text
Launch
→ DB / migration / recovery check
→ Setup or Recovery or Normal App
→ empty Today
```

范围：Tauri/React/Rust boundaries、SQLite migration skeleton、typed IPC、App Data、Setup/Normal/Review/Recovery shells、Active Vault + two Roots、root validation、basic System Health、test skeleton、minimal design tokens。

DoD：release build 启动；workspace persist；invalid roots rejected；unsupported/broken schema blocks normal startup；Frontend 无 direct SQL/FS authority。

Checkpoint：`acm-os-m0-foundation`。

---

## M1 — Real Contest Import → Lightweight Problems

Outcome：导入真实 Codeforces Public Contest，全部题成为 lightweight Problem 并拥有 first statement snapshot。

范围：Contest Shelf/Detail、Import UI、Codeforces locator/adapter、manifest、identity resolution、progressive import、snapshot/assets/sanitize、partial retry、duplicate fast path、My Problems 基础索引、Problem statement view。

主 AC：AC-CONTEST-01 / 02 / 03 / 05。

DoD：完整导入、partial retry、duplicate import、first snapshot no-overwrite、real Codeforces smoke 均有证据。

Checkpoint：`acm-os-m1-contest-import`。

---

## M2 — Personal Markdown → External Obsidian Fresh Read

Outcome：Lightweight Problem 创建真实 Personal Markdown；外部 Obsidian 修改后 ACM-OS 读取最新内容。

范围：create personal note、initial skeleton、File Binding、Windows file key、digest/relocation、Fresh Read/parser、Watcher、window-focus revalidation、Open in Obsidian、Safe Patch engine、Recovery Copy foundation。

主 AC：AC-PROBLEM-01、AC-MD-01~06（其中实际 Candidate relation 行为 M6 再复用）。

Blocking evidence：stale cache + external edit + no watcher event 后仍读到最新内容。

Checkpoint：`acm-os-m2-vault-binding`。

---

## M3 — Upsolve Lifecycle → First Review Schedule

Outcome：

```text
UNSTARTED
→ PENDING
→ LEARNING
→ Mark Understood
→ WAITING_COLD_START +3d
```

范围：ProblemLifecycleEngine、learning_status_since、Review Cycle、first due、withdraw/stop/relearn、Problem Header actions、Delete Personal Note AC-HISTORY-03 行为。

主 AC：AC-PROBLEM-02~06、AC-HISTORY-03 的主体行为。

DoD：重启后状态 / due 保持；delete personal note 正确降级为 Lightweight 且保留 Contest/Review/highest。

Checkpoint：`acm-os-m3-learning-lifecycle`。

---

## M4 — Review Focus → Evidence → Judgement

Outcome：due Problem 完成真实受控 Review，并进入 Long-term 或 Relearn。

范围：Eligibility、Focus Shell、create/resume one Attempt、hidden knowledge isolation、Help Drawer、Evidence-before-Reveal、facts form、judgement、Evidence Card、Review history、Void、first/long-term/early scheduling、IN_PROGRESS destructive-action protection、historical highest projection/invariants。

主 AC：AC-REVIEW-01~11、AC-HISTORY-01、AC-HISTORY-02，并补齐 AC-HISTORY-03 的 IN_PROGRESS conflict。

DoD：至少自动跑通 WAITING→MASTERED→LONG_TERM 和 WAITING/LONG_TERM→PARTIAL→RELEARNING 两条链；历史 Attempt 同时保留。

Checkpoint：`acm-os-m4-review`。

---

## M5 — Today Planner → Stable Daily Execution

Outcome：真实 System Facts + budget 生成稳定、确定、可解释的 Today Plan，并在未来 recall Problem。

范围：Candidate Builder、Carry-in、lanes、sorting、anti-starvation、planning cost、budget、daily snapshot、drag/keyboard reorder、replan preview、reconciliation、today-done、extra suggestions。

主 AC：AC-TODAY-01~05。

重大 Gate：M5 完成后必须跑通 Automated Contest→Markdown→Learning→Review→Today Core-loop E2E；不通过则暂停 M6+。

Checkpoint：`acm-os-m5-today`。

---

## M6 — Knowledge Integration → Obsidian Relationships

Outcome：Knowledge Node / relations 始终来源于真实 Markdown，System Facts 只保存理解状态与历史。

范围：Knowledge discovery/index/detail、WikiLink resolution、derived relations、understanding current/highest、related problems、Obsidian/Graph open、Candidate minimum capability、Safe Patch accept existing Knowledge Node、reevaluation suggestion。

主 AC：AC-KNOWLEDGE-01~03。

Checkpoint：`acm-os-m6-knowledge`。

---

## M7 — Complete Contest Workflow + Manual Contest

Outcome：补齐 Contest 历史产品面与 Manual fallback。

范围：Facts Snapshot、Unknown semantics、upsolve decision、post-contest organization、Contest Result vs live Learning Status、Correction Event、Post-Contest AI Analysis raw/preview/partial/failed、Manual Contest/Problem/Statement、archive/delete、safe lightweight cleanup。

主 AC：AC-CONTEST-04，并补齐 Contest page contracts。

Checkpoint：`acm-os-m7-contest-workflow`。

---

## M8 — Recovery / Backup / Diagnostics / Failure Hardening

Outcome：异常情况下保护已知事实，不猜、不静默损坏。

范围：Health、Location Anomaly repair、binding recovery、parse/concurrency UX、Critical Operation recovery、crash check、backup/retention/restore、derived rebuild、logs、diagnostic export preview、external-open failure、adapter health、完整 Recovery Shell。

DoD：fault injection / backup restore / ambiguous relocation / crash matrix 全部有证据。

Checkpoint：`acm-os-m8-recovery`。

---

## M9 — Accessibility / Security / Performance / UX Hardening

Outcome：不加新产品能力，只让完整 MVP 达到冻结质量门槛。

范围：keyboard flow、focus、200% zoom、reduced motion、contrast、screen-reader async status、Review tree isolation、sanitizer、unsafe schemes、path escape/junction、Tauri capabilities/CSP、diagnostic privacy、Reference Dataset benchmarks、loading/error/empty/responsive/visual consistency。

Checkpoint：`acm-os-m9-quality`。

---

## M10 — Release Candidate + Blocking E2E

顺序：

```text
Full automated gates
→ Real Codeforces smoke
→ Real 15-step multi-day Blocking E2E
```

全部 PASS 才能判定：

```text
Technical MVP Accepted
```

建议 RC tag：`acm-os-mvp-rc1`。

2~4 周真实持续使用属于独立：

```text
Product Habit Validation
```

若用户仍绕开系统，不继续堆功能，应回到核心流程 rethink。

---

# 11. Milestone Dependency Graph

```text
M0 Foundation
      ↓
M1 Contest Import
      ↓
M2 Personal Markdown
      ↓
M3 Learning Lifecycle
      ↓
M4 Review
      ↓
M5 Today
      ↓
M6 Knowledge
      ↓
M7 Contest Complete
      ↓
M8 Recovery
      ↓
M9 Quality
      ↓
M10 RC / Real E2E
```

关键 Core Loop：

```text
M1 → M2 → M3 → M4 → M5
```

M5 Gate 不通过，不继续扩展 M6+。

---

# 12. SPEC Acceptance Criteria → PLAN Coverage Review

结论：

```text
Contest AC             5 / 5 mapped
Problem Lifecycle AC   6 / 6 mapped
Review AC             11 / 11 mapped
Markdown AC            6 / 6 mapped
Knowledge AC           3 / 3 mapped
Today AC               5 / 5 mapped
History Integrity AC   3 / 3 mapped

Total                  39 / 39 mapped
```

## 12.1 Contest

| AC | Primary Milestone | Primary Proof |
|---|---|---|
| AC-CONTEST-01 完整导入 | M1 | T4 Adapter + T2 Persistence + T6/T7 |
| AC-CONTEST-02 Problem 去重 | M1 | Strong-ID DB invariant + adapter integration |
| AC-CONTEST-03 部分导入失败 | M1 | progressive import fixture + retry missing only |
| AC-CONTEST-04 Snapshot 不被学习覆盖 | M7 | Contest fact/history integration + M4 lifecycle |
| AC-CONTEST-05 重复导入 | M1 | duplicate fast path + snapshot no-overwrite |

## 12.2 Problem Lifecycle

| AC | Primary Milestone | Primary Proof |
|---|---|---|
| AC-PROBLEM-01 创建我的笔记 | M2 | Temporary Vault + persistence + UI |
| AC-PROBLEM-02 加入补题 | M3 | Domain contract + persistence |
| AC-PROBLEM-03 开始学习 | M3 | Domain contract + UI action projection |
| AC-PROBLEM-04 确认补懂 | M3 | atomic Problem + Review Cycle + due |
| AC-PROBLEM-05 撤回补懂 | M3 | lifecycle + schedule cancellation contract |
| AC-PROBLEM-06 停止学习 | M3 | lifecycle + history-preservation integration |

## 12.3 Review

| AC | Primary Milestone | Primary Proof |
|---|---|---|
| AC-REVIEW-01 开始 Review | M4 | eligibility + UI hidden-content test |
| AC-REVIEW-02 中途退出 | M4 | resume same attempt UI/E2E |
| AC-REVIEW-03 禁止并行 | M4 | DB unique invariant + application test |
| AC-REVIEW-04 帮助记录 | M4 | Evidence-before-Reveal integration |
| AC-REVIEW-05 真会 | M4 | Judgement golden contract |
| AC-REVIEW-06 半会 | M4 | Judgement + failure reason + relearn transaction |
| AC-REVIEW-07 完整题解 | M4 | Help L5 → FAIL golden contract |
| AC-REVIEW-08 没做出来 | M4 | no final AC → FAIL + reason |
| AC-REVIEW-09 必要事实缺失 | M4 | validation + form preservation |
| AC-REVIEW-10 首次通过 | M4 | WAITING→LONG_TERM scheduling integration |
| AC-REVIEW-11 长期退步 | M4 | LONG_TERM→RELEARNING + immutable history |

## 12.4 Markdown

| AC | Primary Milestone | Primary Proof |
|---|---|---|
| AC-MD-01 外部修改优先 | M2 | stale cache + no watcher event Fresh Read |
| AC-MD-02 Solution Route | M2 | parser fixtures |
| AC-MD-03 局部写入 | M2/M6 | byte-preserving surgical patch |
| AC-MD-04 写入失败 | M2/M6 | write failure + no formal relation commit |
| AC-MD-05 解析冲突 | M2 | duplicate target refuses auto-write |
| AC-MD-06 Vault 暂时不可用 | M2/M8 | degraded state; facts preserved |

## 12.5 Knowledge

| AC | Primary Milestone | Primary Proof |
|---|---|---|
| AC-KNOWLEDGE-01 节点存在条件 | M6 | real Markdown discovery / no empty node |
| AC-KNOWLEDGE-02 正式关系 | M6 | Markdown patch verify first → formal relation |
| AC-KNOWLEDGE-03 不自动升级 | M6 | suggestion only; user-owned understanding state |

## 12.6 Today

| AC | Primary Milestone | Primary Proof |
|---|---|---|
| AC-TODAY-01 合法候选 | M5 | CandidateBuilder contract |
| AC-TODAY-02 当天稳定 | M5 | snapshot + reorder persistence |
| AC-TODAY-03 未完成不算失败 | M5 | next-day generation / no debt |
| AC-TODAY-04 时间预算 | M5 | complete-task packing contract |
| AC-TODAY-05 暂不可执行 | M5/M8 | availability projection; lifecycle unchanged |

## 12.7 History Integrity

| AC | Primary Milestone | Primary Proof |
|---|---|---|
| AC-HISTORY-01 Completed Attempt 不覆盖 | M4 | append/sealed review history |
| AC-HISTORY-02 历史最高保留 | M4 | current vs historical-highest projection/invariant |
| AC-HISTORY-03 删除个人笔记 | M3 + M4 | Personal→Lightweight; lifecycle exit; histories preserved; IN_PROGRESS blocks |

PLAN 10 coverage repair 后，不存在未分配的阻塞 AC。

---

# 13. 15-step Blocking E2E → Milestone Mapping

| Step | Required behavior | First implementation milestone |
|---|---|---|
| 1 | 导入真实 Contest | M1 |
| 2 | 全部题为 Lightweight | M1 |
| 3 | 创建 Personal Markdown | M2 |
| 4 | 加入并开始补题 | M3 |
| 5 | Obsidian 外部修改 | M2 |
| 6 | Fresh Read 最新内容 | M2 |
| 7 | Mark Understood | M3 |
| 8 | 到期 Cold-start Review | M3/M4 |
| 9 | 旧知识隐藏 | M4 |
| 10 | 原 OJ 真实提交 | M4 UI + M10 real evidence |
| 11 | 填真实 Review Facts | M4 |
| 12 | 自动判定 | M4 |
| 13 | Long-term / Relearn | M4 |
| 14 | Completed Attempt preserved | M4 |
| 15 | Later Today recalls Problem | M5 |

M5 后自动核心 E2E 必须覆盖同样逻辑链；M10 再用真实 Codeforces + Obsidian + OJ + 多日时间进行最终阻塞验收。

---

# 14. Release Quality Gates

Technical MVP Accepted 前至少：

```text
G1  Static / Build               PASS
G2  Domain Contract              PASS
G3  Persistence Integration      PASS
G4  Temporary Vault Integration  PASS
G5  Adapter Fixtures             PASS
G6  UI / Recovery                PASS
G7  Accessibility               PASS
G8  Security / Integrity         PASS
G9  Performance                 PASS
G10 Backup / Restore             PASS
G11 Real Codeforces Smoke        PASS
G12 Automated Desktop E2E        PASS
G13 Real 15-step Blocking E2E    PASS
```

Reference Dataset 与 P95 budgets 完全继承 DESIGN，不在 PLAN 修改。

---

# 15. BUILD-DEFERRED — 允许开发阶段决定的细节

以下仍可在 BUILD 依据当前官方文档 / spike / test 选择，不需要重新开产品决策：

## Dependency / tooling

- exact dependency patch versions；
- npm vs pnpm；
- React Router / Query / Form / DnD / component library；
- exact Rust crate versions/features；
- formatter/linter/test helper choices；
- CI provider。

## Persistence

- UUID BLOB vs TEXT；
- Instant INTEGER vs TEXT；
- exact SQLite table/index/constraint names；
- WAL / synchronous / pool tuning；
- exact migration filenames；
- Online Backup API vs VACUUM INTO；
- derived cache storage details。

## Vault

- digest SHA-256 vs BLAKE3；
- exact Windows Rust binding crate；
- temp filename / atomic writer implementation；
- watcher debounce/poll fallback；
- exact parser structs；
- cache eviction；
- exact source-range patch internals。

## Adapter

- exact URL parser implementation；
- HTML parser / sanitizer / math renderer；
- selector strategy；
- HTTP timeout / retry count / bounded concurrency；
- asset size limits；
- exact canonical HTML internal fields。

## UI

- component names；
- CSS solution；
- breakpoints；
- icon library；
- toast/dialog implementation；
- animation duration；
- exact View DTO names。

## Tests

- property testing crate；
- frontend test library；
- desktop E2E driver；
- accessibility scanner；
- fixture server；
- benchmark harness；
- coverage reporting tool。

这些选择不得破坏冻结的 Authority / Invariant / AC / E2E 合同。

---

# 16. BUILD Entry Conditions

进入 BUILD 前，本 PLAN 的判定：

```text
Architecture stack                         PASS
Module ownership / dependency direction    PASS
System Facts persistence contract          PASS
Migration / backup / restore contract      PASS
Markdown Fresh Read / Safe Patch contract  PASS
File identity / relocation contract        PASS
Contest Adapter / import contract          PASS
Review / scheduling / Today engines        PASS
UI state / shell / error boundaries         PASS
Testing evidence strategy                  PASS
Milestones / dependencies                  PASS
SPEC AC mapping 39 / 39                    PASS
15-step E2E mapping 15 / 15                PASS
Blocking unresolved product decision       NONE
SPEC-CONFLICT                              NONE FOUND
```

因此：

```text
ACM-OS PLAN v1 = READY FOR BUILD
```

---

# 17. BUILD 的第一个任务

不要从“写首页”或“先把所有数据库表建完”开始。

第一阶段固定进入：

```text
M0 — Executable Foundation + Workspace Ready Gate
```

M0 内部建议顺序：

```text
B0.1 Repository / workspace scaffold
     → 建立 React / Tauri / Rust crate 依赖边界

B0.2 App Private Data + SQLite startup gate
     → migration ledger / integrity / unsupported-schema behavior

B0.3 Workspace configuration
     → Active Vault + Problem Root + Knowledge Root
     → non-overlap validation + persistence

B0.4 Startup shells
     → Recovery / Setup / Normal / Review layout boundary

B0.5 M0 verification
     → build + migration + workspace persistence + routing + boundary checks
     → milestone review
     → stable tag only after DoD
```

每个 Build Task 必须继续使用：

```text
Outcome
Why
Dependencies
Change Surface
Implementation Steps
Tests
Done Evidence
Rollback
```

一个任务没有明确 Done Evidence，不进入实施。

---

# 18. 最终冻结结论

ACM-OS 第一版工程路线不是“先造一个大而全的题库管理器”，而是按纵向切片逐步证明：

```text
Real Contest
→ one Problem identity
→ real Obsidian Markdown
→ learning lifecycle
→ controlled cold-start Review
→ evidence-based judgement
→ durable history
→ staged review schedule
→ stable Today recall
```

任何优化、UI 便利、Cache、自动化和未来 AI 能力，都不能破坏这条链中的三条底线：

```text
真实 Markdown 不被旧状态覆盖
历史事实不被当前状态覆盖
身份 / 恢复不确定时绝不猜
```

从本 PLAN 冻结后，后续工作正式进入：

```text
BUILD
```
