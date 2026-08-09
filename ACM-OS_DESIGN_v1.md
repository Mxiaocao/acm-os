# ACM-OS DESIGN v1

> 状态：DESIGN 冻结版，可进入 PLAN
> 日期：2026-08-09
> 产品：ACM-OS
> 上游唯一产品事实来源：`ACM-OS_SPEC_v1.md`
> 文档目标：把冻结 SPEC 转换为开发前可执行的系统与交互设计，使后续 PLAN 可以拆 Milestone / Task，而不需要开发者临场猜测产品行为。

---

# 0. 文档地位与变更规则

`ACM-OS_SPEC_v1.md` 仍是产品需求的唯一事实来源。本 DESIGN 只决定 SPEC 中标记为 DESIGN-DEFERRED 的问题，以及开发前必须明确的交互、数据边界、调度、错误恢复和质量指标。

优先级：

```text
SPEC > DESIGN > PLAN > IMPLEMENTATION
```

若后续实现发现冲突：

1. 不能为了实现方便修改 SPEC 行为；
2. 若 DESIGN 与 SPEC 冲突，以 SPEC 为准；
3. 若实现发现 DESIGN 无法满足数据完整性、可恢复性或验收标准，应回到 DESIGN 显式修订；
4. 不能在 PLAN / BUILD 阶段偷偷重新解释已冻结规则。

本 DESIGN 完成后，以下内容仍不得提前回流到 MVP：奖励商城、完整“我的出题”、高级统计 Dashboard、Global Search、自研 Graph、多设备同步、实时 AI Chat / Hint / Debug、大量动画和重游戏化。

---

# 1. DESIGN 总体结论

ACM-OS MVP 的设计中心不是页面数量，而是下面一条长期学习链：

```text
Contest
→ Lightweight Problem
→ Create Personal Markdown
→ Upsolve / Learn
→ Mark Understood
→ Cold-start Review
→ Real OJ Submission
→ Evidence-based Judgement
→ Long-term Review or Relearn
→ Today Plan recalls the Problem later
```

所有设计优先保护四件事：

1. Markdown 真实知识资产不能被缓存或数据库覆盖；
2. Contest / Review / 历史最高状态不能被后续状态静默重写；
3. Review 必须真的隔离旧知识，区分“独立会”和“靠帮助会”；
4. Today Plan 必须减少选择成本，而不是制造任务债务。

质量优先级冻结为：

```text
Data Integrity
> Recoverability
> Reliability
> User Control
> Accessibility
> Performance
> Visual Polish
```

---

# 2. DESIGN 01 — 总体信息架构与导航

## 2.1 一级导航

桌面优先，四个一级学习入口：

```text
Today
Contests
我的题库
Knowledge
```

独立工具入口：

```text
Settings
```

Problem 与 Review 不作为一级导航：

- Problem 是核心对象页面；
- Review Mode 是受控任务执行环境。

## 2.2 Review Mode

Review 使用全屏 Focus Mode：

- 进入后隐藏普通一级导航；
- 普通 Problem 页不能承担正式 Review；
- “保存并退出”只离开 UI，Attempt 仍进行中；
- “结束本次 Review”才进入事实确认与自动判定。

## 2.3 桌面优先

MVP 按桌面优先设计；窄屏保证可适配，但手机端不是第一版同等主流程。

---

# 3. DESIGN 02 — 核心对象与数据权威

## 3.1 四层权威模型

所有数据归入四类：

```text
A. System Facts
B. Markdown Facts
C. Derived Projection
D. Ephemeral UI State
```

### System Facts — ACM-OS 权威

包括：

- Problem 稳定身份；
- OJ / 原始链接；
- 第一次题面快照；
- 客观难度及来源；
- Learning Status；
- Contest Facts；
- Review Attempt 历史；
- Review Result；
- 帮助使用记录；
- 历史最高状态；
- Review Schedule；
- Today Plan Snapshot；
- Knowledge Understanding Status。

### Markdown Facts — Markdown 权威

包括：

- Problem 学习正文；
- 前置知识；
- Hints / 思路 / 代码等用户栏目；
- Solution Routes；
- 额外题；
- Knowledge → Knowledge WikiLinks；
- Problem → Knowledge 正式 WikiLinks。

### Derived Projection

解析 / 索引 / 缓存，例如：

- Route 列表；
- Related Problems；
- incoming / outgoing links；
- Last Known Markdown Projection。

缓存可以失效、重建，但永远不能反写覆盖 Markdown。

## 3.2 稳定内部 ID

以下核心对象拥有与路径、标题、URL 解耦的内部稳定 ID：

- Problem；
- Contest；
- Review Attempt；
- Knowledge Node。

## 3.3 Contest 三维状态

Contest 不使用一个巨大 status，而拆分：

```text
Import Completeness: Incomplete / Complete
Facts Organization: 待整理 / Completed
Archive: Active / Archived
```

## 3.4 Review 历史

Completed Review Attempt 保存：

- 原始事实；
- 最终判定；
- 判定规则版本。

Completed 后核心结果不可直接修改；误开只能 Void，且 Void 记录保留。

## 3.5 Today Plan

Today Plan 是“按日期持久化的当天执行视图”，保证当天稳定，但永远不是长期生命周期权威。

---

# 4. Problem 产品数据契约

Problem 是一道题在 ACM-OS 中唯一、长期存在的学习对象。

至少拥有：

- `problem_id`；
- external identities；
- OJ / source；
- canonical/original URL；
- OJ 正式题名；
- first successful statement snapshot；
- objective difficulty + source；
- identity type：Lightweight / Personal；
- Learning Status；
- File Binding 状态；
- Contest relations；
- Review histories；
- current Review schedule；
- personal pinned priority；
- historical highest evidence。

## 4.1 Learning Status

冻结为：

```text
未进入学习
待补
补题中
已补懂，等待冷启动验证
回炉中
长期复习
```

`真会 / 半会 / 未通过` 是 Review Attempt Result，不是 Problem Learning Status。

## 4.2 笔记关联状态

至少区分：

```text
UNLINKED
LINKED
LOCATION_ANOMALY
EXTERNAL_SOURCE_UNAVAILABLE
CONFIRMED_DELETED
```

文件暂时找不到不能直接降级 Problem。

## 4.3 彻底消化

不维护“掌握度百分比”。保留 6 条证据标准；只有 6/6 才产生“历史最高：彻底消化”。后来回炉时历史最高仍保留。

---

# 5. DESIGN 03 — Markdown / Obsidian 文件身份与同步

## 5.1 不向 Markdown 注入 ACM-OS 私有 ID

MVP 不向用户 Markdown 注入：

- hidden comment problem id；
- YAML `acm_os_id`；
- Knowledge 私有身份元数据。

使用内部 File Binding Registry + 确定性文件证据 + 必要时人工重新关联。

## 5.2 Problem Identity 与 File Binding 分离

```text
Problem Identity ≠ filename ≠ path ≠ H1
```

用户移动、改名、改 H1，不应产生新 Problem。

## 5.3 Relocation 证据层级

自动重新定位只接受确定性证据：

1. 原路径仍存在；
2. 可靠文件身份 / file key 唯一匹配；
3. 内容 digest 完全一致且唯一。

名称、H1、文本相似只能作为候选提示，不能自动关联。

## 5.4 删除语义

路径失效后的流程：

```text
路径消失
→ 确认 Vault 正常
→ 尝试确定性 relocation
→ 无法确定
→ LOCATION_ANOMALY
→ 用户重新关联 / 明确确认删除
```

只有人工明确确认删除才执行 Problem / Knowledge 删除语义。

## 5.5 Fresh Read

正常打开 Problem / Knowledge 时，Markdown 读取流程：

```text
File Binding
→ Read current disk file
→ Parse
→ Build UI projection
```

旧缓存不能成为正常读取事实源。

## 5.6 Watcher

File Watcher 只用于 invalidate cache / 触发重新读取；Watcher 事件本身不是事实源。

## 5.7 Safe Patch Transaction

所有 ACM-OS Markdown 修改统一遵循：

```text
User explicit action
→ Confirm Binding = Linked
→ Fresh read
→ Parse latest structure
→ Unique target validation
→ Minimal surgical patch
→ Optimistic concurrency check
→ Pre-write recovery copy
→ Write
→ Re-read
→ Re-parse
→ Verify target semantic result
→ Commit corresponding system fact
```

目标不唯一时拒绝写入。

## 5.8 不自动 Merge 并发编辑

检测到 Obsidian 在当前 Patch 期间修改文件：

```text
Cancel write
→ Re-read
→ User retries
```

MVP 不做三方 Merge UI。

## 5.9 缺失目标 Section

需要写 `## 前置知识` / `## 额外题目`，但栏目不存在时：

- 不静默创建；
- 明确询问“创建该栏目并继续”。

## 5.10 Problem Markdown 可识别栏目

初始骨架仍遵守 SPEC：

```markdown
# Problem

## 前置知识

## 题解

### 标准推导

## 额外题目
```

另外 Review 可识别但不强制生成的可选栏目：

```markdown
## Hints
### Hint 1

## 思路

## 代码
```

Solution Route 只认 `## 题解` 下直接 `###`。

---

# 6. DESIGN 04 — Contest 导入、去重与 Snapshot

## 6.1 MVP 自动导入范围

自动 Contest Import 第一版正式保证：

```text
Codeforces Public Contest
```

统一兜底：

```text
Manual Contest
```

其它 OJ 后续通过 Adapter 扩展，不阻塞 MVP。

## 6.2 Strong Identity

Problem 自动去重只有平台 Strong External Identity 可以触发。

Weak Evidence（题名、题面、URL 相似、难度等）只能提示潜在冲突，不自动 Merge。

同一个 Problem 可以在用户明确确认 / Adapter 确定性证明后拥有多个 External Identity。

## 6.3 Contest Identity

自动支持平台由 Adapter 产出：

```text
platform + external_contest_key
```

重复 URL 形式只要解析为同一 identity，就进入原 Contest。

手动 Contest 不按“标题 + 日期”自动去重。

## 6.4 Adapter 边界

外部来源 → Adapter → Validate → Canonical Import Draft → ACM-OS Core。

外部输入全部视为不可信数据，必须验证必要字段与 identity。

## 6.5 Progressive + Idempotent Import

导入不是 All-or-Nothing：

- 已成功 Problem / Snapshot 保留；
- 失败项单独标记；
- Contest = Incomplete；
- Retry 只补缺；
- 不复制成功对象。

## 6.6 Import Complete 定义

只有满足：

```text
Contest metadata complete
+ complete Problem list known
+ each Contest Problem resolved to Problem
+ each Problem has first successful full statement snapshot
```

才标记 Import Complete。

## 6.7 First Statement Snapshot

第一次成功题面快照之后自动 re-import 不覆盖。

完整题面至少能够表达：

- 正文；
- input / output；
- samples；
- notes；
- formulas；
- 必要图片 / assets。

## 6.8 Contest Facts Snapshot

Contest ↔ Problem 使用 Contest Problem Record 保存比赛结束时事实。

最小完成条件：

- title；
- date；
- platform；
- complete Problem list；
- each Problem final contest result。

其它字段允许 Unknown / 未记录。

`unknown` 不等于 `false`。

## 6.9 Contest 与 Learning Status 严格分离

Contest Problem Record 可以保存：

- 比赛结果；
- 当时是否看过 / 有思路 / 写代码；
- 卡点；
- 最后代码；
- 补题决策：计划补 / 暂不补 / 未决定。

不能拥有当前 Learning Status 副本。

## 6.10 Correction Event

Snapshot 后人工纠错：

- 当前 Facts 更新；
- 保留轻量 Correction Event：field / old / new / corrected_at；
- 不做完整 Snapshot Versioning。

## 6.11 Re-import 原则

```text
Fill Missing, no Silent Refresh
```

不得静默覆盖：

- first statement snapshot；
- corrected Facts；
- Post-Contest Analysis；
- Learning Status。

---

# 7. DESIGN 05 — Review Mode 与 Review Attempt

## 7.1 Review Mode 定义

Review Mode 是围绕一个 Review Attempt 的冷启动验证环境，不是普通 Problem 页面。

允许正式 Review 的 Problem：

- 已补懂，等待冷启动；
- 长期复习。

## 7.2 Attempt 开始

点击 Review：

```text
若无 IN_PROGRESS
→ create Attempt
若已有
→ resume same Attempt
```

同一 Problem 同时最多一个 IN_PROGRESS Attempt。

Attempt 开始时固定记录：

- problem；
- attempt type：FIRST_COLD_START / LONG_TERM_REVIEW / EARLY_CHECK；
- started_at；
- scheduled_due_at；
- started_early；
- status。

## 7.3 初始可见内容

只显示：

- 完整题面；
- 原 OJ 入口。

隐藏：

- Contest 历史；
- Knowledge；
- Hints；
- 旧思路；
- 旧代码；
- Solution Routes；
- 额外题；
- Review 历史；
- Obsidian 入口。

## 7.4 Help Drawer

帮助允许跳级：

```text
1 前置知识名字
2 Hints
3 前置知识内容
4 旧思路 / 旧代码
5 完整题解
```

只打开 Drawer 不算使用；真正 Reveal 成功后记录 Help Usage Event，且本 Attempt 中不可撤销。

第一次 Reveal Level 1-4 前提醒：本次最高变为“半会”。

Level 5 再次单独确认：查看完整题解后，本次只能“未通过”。

## 7.5 Help 内容来源

- 前置知识名字：`## 前置知识` WikiLinks；
- Hint：可选 `## Hints` 的直接 `###`；
- Knowledge 内容：真实 Knowledge Markdown；
- 旧思路：明确 `## 思路`；
- 旧代码：明确 `## 代码`；
- 完整题解：`## 题解` 完整 section。

系统不从任意正文猜内容类型。

## 7.6 OJ Submission

MVP：

```text
Open real original OJ
→ user really submits
→ user returns and confirms facts
```

不依赖自动登录 / 抓 Submission。

## 7.7 结束事实确认

### Submission Facts

- final AC yes/no；
- first submission result；
- final result；
- total submissions。

### Independence

- idea independent yes/no；
- implementation independent yes/no；
- Debug：无需 Debug / 独立 Debug / 使用解题性帮助 Debug。

### External Help

- 无；
- 解题提示级；
- 完整题解级。

## 7.8 自动判定

规则：

```text
No final AC
→ 未通过

Full-solution-level help used
→ 未通过

Final AC + any problem-solving help / non-independent key step
→ 半会

Final AC + idea independent + implementation independent
+ Debug independent or not needed
+ no solving help
→ 真会
```

第一发 WA 本身不影响真会资格。

## 7.9 Failure Reasons

半会 / 未通过必须至少一个：

- 完全没思路；
- 有方向但关键性质卡住；
- 公式 / 推导卡住；
- 算法会但写不出来；
- 实现错误；
- 边界问题；
- 复杂度判断错误；
- 其他。

信息缺失时 Attempt 继续 IN_PROGRESS，不能 Completed。

## 7.10 Completed Result

Completed 后展示 Evidence Card，解释为什么是真会 / 半会 / 未通过。

半会 / 未通过一定进入回炉。

## 7.11 Void

只用于误开；Void 本身保留历史。实际尝试但没做出来应结束为“未通过”，不能用 Void 掩盖失败。

---

# 8. DESIGN 06 — Review Scheduling

## 8.1 模型

MVP 使用固定阶段式 Schedule，不使用 SM-2 / 隐藏记忆分数。

## 8.2 默认间隔

最终冻结：

```text
3 → 10 → 30 → 75 → 150 → 240 → 240 天
```

含义：

```text
Mark Understood
→ +3d first cold start
真会
→ +10d
真会
→ +30d
真会
→ +75d
真会
→ +150d
真会
→ +240d
后续持续真会
→ 每 240d
```

## 8.3 Calendar Date

Review due、Today Plan date、Weekly Budget weekday 使用当前 Workspace 所在机器的 Local Calendar Date。

`next_due` 是日期值，不是 UTC 时间点；操作系统时区变化不让既有 due date 漂移。

历史事件如 started_at / completed_at 仍保存时间戳。

## 8.4 真会

正式到期 Review 真会只前进一个 stage；submission count、difficulty、第一发 AC 等不改变间隔。

## 8.5 半会 / 未通过

立即：

```text
End current Review Cycle
→ Problem 回炉中
→ Schedule suspended
```

必须重新学习、再次“我已经补懂”，创建新 Cycle，从 +3d 开始。

过去成功历史不删除。

## 8.6 Overdue

逾期：

- 不失败；
- 不扣分；
- 不产生多份欠债 Review；
- 继续保持一个 due candidate；
- 下次 due 从实际 Completed 日期计算。

## 8.7 IN_PROGRESS

进行中 Attempt 存在时不产生新 Review。

Void 不推进、不重置 schedule。

## 8.8 撤回补懂

待首次冷启动时撤回补懂：取消当前 schedule，回补题中，不产生失败 Review。

## 8.9 提前检查 Early Check

用户可未到期主动进入同样的 Review Mode：

- 真会：记录完整 Attempt，但不推进 Stage、不改变原 due date；
- 半会 / 未通过：立即接受真实负面证据，回炉并取消原 schedule。

## 8.10 Schedule Rule Version

Schedule 规则带版本。规则升级默认只影响未来新的 Scheduling Decision，不静默重算已有 due date。

---

# 9. DESIGN 07 — Today Plan

## 9.1 定位

Today Plan 是当天执行视图，不是第三个长期队列。

## 9.2 合法候选

三层：

### Carry-in

- IN_PROGRESS Review；
- 补题中。

### Review Lane

- 到期第一次冷启动；
- 到期长期 Review。

未到期 Review 不自动加入。

### Study Lane

- 回炉中；
- 待补。

## 9.3 候选排序

不使用黑盒总评分。

Review Lane 默认：

```text
更久 overdue
→ pinned priority
→ 同日时 first cold start 优先 long-term
→ stable tie-break
```

Study Lane：

```text
回炉中
→ 待补
→ pinned
→ 等待更久
→ stable tie-break
```

CF rating 不参与 MVP Today 优先级。

## 9.4 固定个人优先级

MVP 只有：

```text
Normal / Pinned
```

Pinned 是跨天推荐输入，不突破生命周期。

## 9.5 Review / Study Anti-Starvation

Review 默认优先，但 Study 不能长期被饿死。

规则：

1. Carry-in 先处理；
2. 若 Review + Study backlog 都存在，且预算能完整容纳两类至少各一个，则至少各安排一个；
3. 若预算只能容纳一种，并且存在 Study backlog，最多连续 2 个生成过的 Today Plan 为 Review-only；下一天必须给 Study 一个 slot；
4. 这不是 Review 欠债，未安排 Review 只继续 overdue。

## 9.6 Planning Cost

默认：

```text
Review = 30 min
Upsolve / Relearn = 60 min
```

只是计划预算块，不是计时器，不记录真实耗时。

用户可改默认值和单日计划预算。

## 9.7 装箱原则

只安排完整任务：剩余预算放不下完整 Planning Cost 时停止，不把 60m Upsolve 压缩成 20m。

## 9.8 当天稳定

第一次生成 Today Plan 后持久化当天 Snapshot：

- 重开不洗牌；
- 用户拖动后顺序优先；
- 第二天重新生成；
- 用户从其它页面主动启动的真实 IN_PROGRESS 工作必须即时反映。

## 9.9 预算变化

预算变小 / 变大均不能静默改 Plan。

系统提供“建议重新规划”，用户确认后才调整系统自动生成且未开始的 Entry。

已完成、进行中、手动加入默认保留。

## 9.10 Today 完成语义

Review：Attempt Completed 才自动完成 Entry。

Upsolve：允许“今日先到这里”，只完成 Today Entry，Problem 仍为补题中，第二天优先继续。

## 9.11 暂不可执行

Vault / 外部依赖异常导致任务暂不可执行：

- 不失败；
- 不改 Problem；
- 不创建失败 Review；
- 可以建议替代任务，但必须用户主动加入。

## 9.12 完成后补位

若还有未分配 Planning Budget，可展示额外建议；只有用户主动加入才成为 Today Entry。

---

# 10. DESIGN 08 — Knowledge / Obsidian Graph

## 10.1 Knowledge Root

初始化选择一个 Knowledge Root，递归发现其中 Markdown 作为新 Knowledge Nodes。

Knowledge Root 是 Discovery Scope，不是身份来源。已绑定节点移动出 Root 后，只要确定性识别，仍保持原 identity。

## 10.2 Knowledge Node 名称

默认 UI 名称为当前 Markdown 文件名去掉 `.md`；名称 / H1 不承担稳定身份。

## 10.3 WikiLink Resolution

只做确定性解析：

```text
唯一目标 → RESOLVED
不存在 → UNRESOLVED
多个可能 → AMBIGUOUS
文件存在但不是 Knowledge Node → NON_KNOWLEDGE_TARGET
```

不 fuzzy 自动选择。

Obsidian alias / heading link 仍连接到目标 Knowledge Node；anchor 可用于打开定位，但不产生新节点。

## 10.4 Problem → Knowledge

正式关系只来自 Problem Markdown `## 前置知识` 中真实 WikiLink。

其它正文、AI Analysis、Candidate 不自动成为正式关系。

## 10.5 Knowledge → Knowledge

Knowledge Markdown 中用户主动写的 WikiLinks 直接形成 Graph 关系。

ACM-OS 只维护 Derived Index，不提供更高优先级 Graph。

## 10.6 AI Candidate

MVP 不做独立 Candidate Center，主要在 Problem Detail 的前置知识区域处理。

状态：接受 / 不要 / 暂不处理。

- 已有 Knowledge Node：接受后 Safe Patch 写 Problem Markdown，重读确认后正式成立；
- 节点不存在：允许记录“接受意图”，但仍保持 Candidate，不创建空节点、不提前写关系；以后真实 Node 出现后仍需用户显式点击写入；
- 不要：保存轻量 ignored fingerprint，避免同一建议反复出现。

## 10.7 Understanding Status

五级：

```text
未学
学过但模糊
基本理解
熟练使用
深入理解
```

保存当前、历史最高、首次达到历史最高日期。

只能用户修改，Problem 表现不能自动升降。

## 10.8 重新评估建议

自上次人工确认状态后，如果至少 3 道不同正式相关 Problem 获得新的“真会” Review Evidence，可以产生一次非阻塞“考虑重新评估”提示。

它：

- 不自动选等级；
- 不进入 Today；
- 不创建理论 Review Queue。

## 10.9 Graph 职责

Knowledge Detail 可显示 incoming / outgoing 邻接列表，但不自研 Graph Editor / 力导向图。

需要看图：打开 Obsidian / Obsidian Graph。

---

# 11. DESIGN 09 — 初始化、Settings 与外部依赖

## 11.1 Workspace

MVP 一个 Workspace 只管理一个 Active Vault。

Vault 内必须配置两个不重叠 Root：

```text
Problem Notes Root
Knowledge Root
```

Problem Root 是新 Problem Markdown 默认 Creation Target，不扫描里面所有 Markdown 猜 Problem。

Knowledge Root 是新 Knowledge Node Discovery Scope。

两个 Root 不得相等或互为父子目录。

## 11.2 初始化 Ready Gate

首次启动：

```text
1 Connect Vault
2 Select Problem Notes Root / Knowledge Root
3 Optional weekly ACM budget
4 Environment Check
5 Enter Today
```

最低必填：

```text
Active Vault
Problem Notes Root
Knowledge Root
```

Weekly Budget 可稍后设置；没有预算时第一次进入 Today 询问当天 Budget。

Planning Cost 使用默认 30m / 60m，不在 onboarding 强迫配置。

## 11.3 初始化扫描

Knowledge Root 初始索引只读，不修改文件。

Problem Root 不批量接管历史 Problem Markdown。

旧 Problem Markdown 的迁移属于未来明确 Migration 功能。

## 11.4 Settings IA

```text
Workspace
Planning
Import
System Status
```

### Workspace

- Active Vault；
- Problem Notes Root；
- Knowledge Root；
- Obsidian integration；
- reconnect / repair。

### Planning

- weekly ACM budget；
- default Review Planning Cost；
- default Upsolve Planning Cost。

当天 budget 仍在 Today 修改。

### Import

- Codeforces Public supported；
- Manual Contest always available。

### System Status

- Vault health；
- Knowledge index；
- Binding anomalies；
- Parse warnings；
- Database integrity；
- Backup；
- Adapter state。

## 11.5 Root 变化

Problem Root 改变只影响未来新笔记。

Knowledge Root 改变只影响未来自动发现。

已有 Binding 不自动搬家、不删除。

## 11.6 Active Vault 改变

必须：

```text
Validate
→ Preview relocation impact
→ User Confirm
→ Commit
```

无法重新定位的文件进入 Location Anomaly，不能判删除。

## 11.7 外部依赖健康

统一状态：

```text
AVAILABLE / DEGRADED / UNAVAILABLE
```

### Vault

全局 Persistent Health Banner；系统事实仍可访问，依赖 Markdown 的实时读写降级。

### Obsidian App

与 Vault file access 分离；打不开只影响 external open，提供 retry / copy path / check settings。

### OJ

打不开提供 retry / copy URL，不改变 Learning Status / Review Result。

### Contest Source

不可用只影响新的 import / retry，不影响已保存 Contest / Snapshot / Review。

---

# 12. DESIGN 10 — Error / Conflict UX

## 12.1 总原则

```text
Detect
→ protect known facts
→ block only affected scope
→ preserve diagnostics
→ explain impact
→ explicit recovery action
→ re-validate
```

每个错误尽量回答：

1. 发生了什么？
2. 什么没有受到影响？
3. 下一步能做什么？

## 12.2 Location Anomaly

对象保持身份；UI 提供：

- 查找可能位置；
- 手动选择；
- 确认已删除。

确定性 relocation 可以自动恢复 Binding；只有疑似候选时必须用户确认。

手动关联若文件已绑定另一 Primary Object，禁止抢占。

## 12.3 Problem Identity Conflict

两边都有真实个人历史时，MVP 不 Merge。

允许：

- 确认不是同一道题并记住；
- 暂不处理。

仅完全无 Markdown、无学习历史、无 Review、无其它用户历史的纯轻量错误对象，才允许进入严格 Safe Cleanup；仍需用户确认。

## 12.4 Parse Warning

局部异常局部降级。

例如两个 `## 额外题目`：

- 其它区域正常；
- 额外题区域 Warning；
- 自动写入禁用；
- 提供在 Obsidian 打开 / 重查 / 复制诊断；
- 不自动“修复”用户 Markdown。

## 12.5 Concurrency Error

外部修改出现：取消 Patch，提示重新读取后重试；不做 Merge。

## 12.6 Knowledge 同名重建

历史节点已确认删除后又出现同名 Markdown：永不自动恢复身份。

用户选择：

```text
恢复旧 Knowledge Node
或
作为新的 Knowledge Node
```

恢复身份只恢复系统 identity / historical understanding；正式当前 Graph 关系仍由当前 Markdown 决定。

## 12.7 Post-Contest AI Analysis

数据模型：

```text
Raw Text
Parse Status: COMPLETE / PARTIAL / FAILED
Parsed Projection
```

粘贴先 Parse Preview，再显式保存 / 替换。

PARTIAL / FAILED 可保留 Raw Text；只有成功解析的部分进入结构化 UI。

AI Analysis 永远没有修改 Contest Facts / Learning Status / formal Knowledge Relation 的权限。

## 12.8 Destructive Action

危险动作不用模糊“确定吗”，而使用 Consequence Preview，列出：

- 会改变什么；
- 什么会保留。

存在 IN_PROGRESS Review 等冲突时直接阻塞。

## 12.9 Void Review

明确说明只用于误开；实际尝试失败应结束为“未通过”。

---

# 13. DESIGN 11 — 性能、安全、可访问性、日志、备份恢复

## 13.1 Reference Dataset

MVP 性能基准数据规模：

```text
Problems: 2,000
Personal Problem Markdown: 1,000
Contests: 300
Review Attempts: 10,000
Knowledge Nodes: 1,000
WikiLinks / Relations: 20,000
Today Candidates: 500
Single normal Markdown: <= 1 MB
```

参考机器：4-core CPU / 8GB RAM / SSD / Release Build。

## 13.2 P95 性能预算

- normal startup → Today interactive：≤ 2.5s；
- existing Today Plan open：≤ 300ms；
- first Today generation：≤ 300ms；
- Problem system facts load：≤ 300ms；
- local page navigation：≤ 300ms；
- library / knowledge name search：≤ 150ms；
- normal Problem Markdown read + parse：≤ 500ms；
- single Knowledge reparse：≤ 300ms；
- input visual response：≤ 100ms；
- initial 1,000 Knowledge index：目标 ≤ 10s，但 UI 不得阻塞；
- normal file-change projection update：P95 ≤ 1s；
- normal stable RAM：目标 ≤ 500MB。

网络延迟不算入本地预算，但用户点击后 ≤100ms 进入 loading，≤500ms 给明确进度 / 状态反馈。

任何性能优化不得省略 Markdown write 前 Fresh Read。

## 13.3 Local-first Security

MVP：

```text
Local-first
Single-user
No automatic telemetry
No automatic crash upload
No cloud account
```

## 13.4 File Scope

Read Scope：Active Vault + ACM-OS App Private Data。

Write Scope 更窄：

- Problem Notes Root 中 ACM-OS 创建的新 Problem Markdown；
- 已明确绑定 Problem Markdown 的受控最小 Patch；
- ACM-OS App Private Data / Backup。

MVP 不主动写 Knowledge Markdown。

任何 write 前必须确认 resolved path 仍位于 Active Vault；路径逃逸时阻止。

## 13.5 Untrusted Content

Markdown / Statement HTML 均视为不可信输入：

- sanitize；
- 禁止 arbitrary JS；
- 禁止 inline event handlers；
- 禁止 plugin scripts / shell execution；
- 外部链接通过安全 external open。

MVP 不保存 OJ password / cookie / token。

## 13.6 App Data

System Facts、Binding Registry、Logs、Backups 存 App Private Data，不混入 Obsidian Vault。

## 13.7 Logs

日志只用于诊断，不是系统事实源。

结构化字段建议：

- timestamp；
- severity；
- event_code；
- component；
- operation_id；
- object_type / id；
- duration_ms；
- result；
- error_code。

禁止默认日志保存：

- 完整 Markdown；
- 用户代码块；
- 完整题面；
- Raw AI Analysis；
- 用户自由文本失败原因；
- secrets；
- 剪贴板。

保留：14 天或 20MB，先到限制即滚动清理。

## 13.8 Diagnostic Export

用户主动触发并 Preview。

默认可以包含：

- App / schema version；
- OS/runtime basics；
- 近 7 天脱敏日志；
- error codes；
- index / binding health；
- sanitized config structure。

默认不含完整 Vault / Markdown / code / free text。

## 13.9 System Facts Backup

自动：

- 每天第一次产生 system fact 修改时，如果当天无 backup，则创建 Daily Snapshot；
- schema migration / manual restore 前必须额外 backup。

保留：

```text
7 Daily + 4 Weekly
```

## 13.10 Markdown Recovery Copy

每次 ACM-OS 写 Problem Markdown 前保存 Pre-write Copy。

每个文件最多：

```text
10 copies
且最长 30 days
```

存 App Private Recovery Area，不污染 Vault。

Undo 只有当当前文件 digest 仍等于写入成功后的 digest 时才能自动恢复；若用户之后又编辑过，只允许查看 recovery copy / 手动处理，不能覆盖新修改。

## 13.11 System Restore

Restore 前先保存 current recovery snapshot。

恢复 System Facts 不自动恢复 Markdown。

Restore 后：

```text
validate bindings
→ fresh read current Markdown
→ rebuild derived projections
→ report anomalies
```

## 13.12 原子语义

以下系统动作必须具有单一 Commit Boundary：

- Review Completed → Attempt + Problem state + Schedule；
- Today initial plan → complete Today Snapshot；
- Snapshot correction → current facts + Correction Event；
- Contest import item → coherent resolve/relation/snapshot state。

跨 Markdown 操作遵循“Markdown 先成功并验证，再 commit system fact”。

## 13.13 Crash Recovery

异常退出后启动进行轻量 Recovery Check：

- System Facts integrity；
- pending critical write；
- binding anomalies。

System Facts 无法可靠读取时进入 Recovery Mode，而不是正常进入半坏状态。

## 13.14 Accessibility

核心 MVP Flow 以 WCAG 2.2 AA 为设计/测试目标。

要求：

- 完整键盘路径；
- 可见且逻辑 Focus；
- Today 排序存在键盘替代；
- Dialog / Drawer focus management；
- status 不能只靠颜色；
- normal text contrast ≥ 4.5:1；
- large text ≥ 3:1；
- important UI boundary/status ≥ 3:1；
- 200% zoom 核心操作仍可用；
- `prefers-reduced-motion`；
- async errors 可被辅助技术感知；
- Review 未 Reveal 的旧知识不得存在于 Accessibility Tree。

---

# 14. DESIGN 12 — 页面合同补齐

## 14.1 Contest Shelf

Contest 首页使用“数字书架”而不是后台表格。

```text
Contests

[导入 Contest] [手动创建]

需要处理
[Book] [Book]

我的比赛
[Book] [Book] [Book]

[查看已归档]
```

“需要处理”是 Projection：Import Incomplete 或 Facts 待整理。

Book 只显示：

- title；
- date；
- platform；
- Problem count；
- status marker。

整卡进入 Contest Detail。

## 14.2 Contest Detail

三个主要区域：

```text
Problems
Facts Snapshot
AI Analysis
```

Problem 行明确并列：

- Problem identity；
- contest final result；
- contest upsolve decision；
- current live Learning Status。

Partial Import 在 Problems 区域提供 Retry / Manual Supplement。

## 14.3 Problem Detail

Header：

- Problem identity / OJ / rating；
- current Learning Status；
- 当前主要生命周期动作。

正文：

```text
题面
我的笔记
历史
```

### 题面

系统事实：statement snapshot / OJ / link / difficulty。

### 我的笔记

Fresh Markdown projection：prerequisites / routes / extras / hints；提供在 Obsidian 中打开。

### 历史

Contest histories / Review histories / historical highest / digestion evidence。

危险动作（如删除我的笔记）放次级菜单，走 Consequence Preview。

## 14.4 Problem Header 的动作映射

```text
Lightweight → 创建我的笔记
未进入学习 → 加入补题
待补 → 开始学习 / 停止学习
补题中 → 我已经补懂 / 放回待补 / 停止学习
待冷启动 → 到期开始 Review / 未到期提前检查 / 撤回补懂 / 停止学习
回炉中 → 重新学习 / 停止学习
长期复习 → 到期开始 Review / 未到期提前检查
```

Today 场景可额外显示“今日先到这里”。

## 14.5 我的题库

桌面优先使用高密度 List / Table，不使用 Contest 书架视觉。

必须提供：

- name search；
- identity filter：全部 / Lightweight / Personal；
- Learning Status filter；
- stable ordering；
- 进入统一 Problem Detail。

默认排序：最近进入 ACM-OS 的 Problem 在前。

不扩张成 Global Search。

---

# 15. 关键系统不变量 Consolidated

## Identity / History

1. 同一道确定身份的 Problem 全系统一份，多处引用；
2. 不确定身份时不自动 Merge；
3. Contest Result 与 Learning Status 永远分离；
4. Completed Review 不能被未来 Review 覆盖；
5. 当前状态下降不删除历史最高；
6. Location Anomaly 不等于删除。

## Markdown

7. current disk Markdown 永远高于 Derived Cache；
8. path / filename / H1 不定义 Problem / Knowledge identity；
9. 写入目标不唯一就拒绝写；
10. write 必须 Fresh Read + minimal patch + concurrency check + re-read verify；
11. 并发外部修改时取消 Patch，不自动 Merge；
12. ACM-OS 状态变化不能后台修改正文；
13. formal Markdown relation 只有真实文件写入并验证后成立。

## Review

14. 一个 Problem 同时最多一个 IN_PROGRESS Attempt；
15. 离开 Review Mode 不代表失败；
16. help reveal 是不可撤销历史事实；
17. Level 1-4 解题帮助使最高结果为半会；
18. full solution 使结果只能未通过；
19. 用户不能直接选择 Review result；
20. 第一发 WA 不影响独立真会；
21. 半会 / 未通过必须回炉。

## Scheduling / Today

22. 只有“我已经补懂”开启新的 Review Cycle；
23. 只有正式到期真会推进 schedule；
24. 提前真会不推进 stage，提前失败立即回炉；
25. overdue 不制造失败 / 欠债；
26. Today Plan 不拥有 Learning Status；
27. Carry-in 真实工作高于新推荐；
28. 未到期 Review 不自动进入 Today；
29. Today drag order 当天优先于算法；
30. Planning Cost 不是计时器；
31. Today 未完成不形成“昨日欠债”。

## Knowledge

32. 没有真实 Markdown 就没有正式 Knowledge Node；
33. WikiLink 无法唯一解析时不猜；
34. AI Candidate 不是正式关系；
35. Understanding Status 最终只能用户确认；
36. Knowledge Status 不产生理论 Review Queue；
37. ACM-OS 不维护高于 Obsidian 的第二套 Graph。

## Safety / Recovery

38. 外部依赖失败只能局部降级；
39. 进行中 Review 阻止相冲突的破坏性动作；
40. System Restore 不能覆盖当前 Markdown；
41. Markdown 写入必须有 Pre-write Recovery Copy；
42. 安全恢复必须重新验证真实状态；
43. Data Integrity 永远高于性能优化。

---

# 16. SPEC Acceptance Criteria Coverage Review

DESIGN 12 Review 结论：

```text
Contest AC: 5 / 5 covered
Problem Lifecycle AC: 6 / 6 covered
Review AC: 11 / 11 covered
Markdown AC: 6 / 6 covered
Knowledge AC: 3 / 3 covered
Today Plan AC: 5 / 5 covered
History Integrity AC: 3 / 3 covered

Total: 39 / 39 covered
```

没有发现必须修改 SPEC 的矛盾。

---

# 17. MVP 15-step Blocking E2E Test

Release Candidate 必须至少真实完成一次：

```text
1 Import a real supported public Contest
2 All Problems become lightweight Problems
3 Select one Problem and create Personal Markdown
4 Join upsolve and start learning
5 Edit Markdown externally in Obsidian
6 ACM-OS reads the newest content
7 Mark the Problem understood
8 Wait until / reach due cold-start Review
9 Enter Review Mode with old knowledge hidden
10 Open original OJ and make a real submission
11 Confirm Review facts
12 System automatically judges 真会 / 半会 / 未通过
13 Transition to long-term review or relearn
14 Preserve completed Attempt history
15 A later Today Plan recalls the Problem again
```

这条真实链是 MVP Blocking Gate。不能用纯 mock / `final_ac=true` 代替最终真实产品验收。

---

# 18. 测试设计边界

## 18.1 Deterministic Contract Tests

覆盖：

- Problem lifecycle；
- Review judgement；
- Review scheduling；
- Today candidate selection；
- history invariants。

例如必须固定测试：

```text
WA + independent debug + final AC = 真会
Hint + final AC = 半会
Full solution + final AC = 未通过
半会 → 回炉
```

## 18.2 Temporary Vault Integration Tests

使用临时测试 Vault 验证：

- Fresh Read；
- Route parser；
- Surgical Patch；
- concurrency conflict；
- file move / rename；
- Location Anomaly；
- Vault unavailable；
- Pre-write Recovery。

自动测试不得直接操作用户真实 Vault。

## 18.3 Contest Adapter Contract Tests

使用固定 Fixtures 验证：

- complete import；
- partial failure；
- retry；
- duplicate contest；
- Problem resolve；
- first snapshot no-overwrite。

CI 不依赖 Codeforces 实时在线。

另保留真实 Codeforces smoke test 用于发布前外部集成验证。

## 18.4 UI / Recovery Flow Tests

覆盖：

- Today drag persistence；
- Review exit / resume；
- Help reveal；
- missing facts cannot complete；
- consequence preview；
- parse warning partial degradation；
- Vault banner；
- keyboard-only paths。

## 18.5 Quality Gates

发布前额外验证：

- performance benchmark；
- fault / integrity tests；
- backup restore；
- keyboard-only core flow；
- 200% zoom；
- reduced motion；
- contrast；
- Review hidden content not exposed in accessibility tree。

---

# 19. PLAN 阶段仍可决定的实现自由度

以下仍属于 PLAN / IMPLEMENTATION，不应在 DESIGN 中锁死：

- Tauri / Electron / Web + local service 等技术栈；
- 前端框架；
- SQLite / 其它本地 system-fact storage；
- File Binding Registry 具体 schema；
- Markdown parser 库；
- File Watcher 实现；
- Windows / macOS file identity API；
- Obsidian URI / external open 具体实现；
- Codeforces statement capture 具体技术；
- exact module / repository layout；
- component naming；
- internal API / IPC shapes；
- cache implementation；
- transaction implementation；
- concrete backup file format；
- exact sanitized renderer library；
- CI pipeline；
- Milestone / Sprint / Issue decomposition。

这些实现选择必须满足本 DESIGN 的行为合同和 SPEC Acceptance Criteria。

---

# 20. POST-MVP 保持冻结，不进入 PLAN 阻塞路径

以下继续不阻塞 MVP：

- Rewards Shop；
- complete problem-authoring system；
- advanced statistics / diagnosis dashboard；
- Global Search；
- self-built Knowledge Graph；
- full AI Candidate center；
- multi-device sync；
- proactive notification；
- real-time AI Chat / Hint / Debug；
- heavy gamification；
- complex Contest tag system；
- full Markdown version audit。

---

# 21. PLAN Entry Conditions

目前全部满足：

```text
6 MVP surfaces IA / navigation / page contracts        PASS
Core object / data contracts                            PASS
Markdown identity / safe write / relocation             PASS
Contest import scope / dedupe / snapshot / recovery     PASS
Review Mode / Attempt judgement                          PASS
Review Scheduling                                        PASS
Today Plan scheduling / budget                           PASS
Knowledge / Obsidian integration                         PASS
Initialization / Settings / external dependency recovery PASS
Error / Conflict UX                                      PASS
Performance / security / accessibility / diagnostics     PASS
Backup / rollback                                        PASS
SPEC AC coverage 39/39                                    PASS
15-step E2E test boundary                                 PASS
```

---

# 22. DESIGN v1 最终冻结结论

ACM-OS DESIGN v1 正式完成。

MVP 的工程核心应始终围绕以下边界展开：

```text
Obsidian Markdown
= long-term knowledge-content truth

ACM-OS System Facts
= identity / history / lifecycle / schedule truth

Review Mode
= controlled cold-start evidence environment

Today Plan
= limited, stable daily execution view

Contest
= immutable-ish historical context, never current learning truth

Problem
= the central long-lived learning object
```

从这一刻起，可以进入 PLAN，但 PLAN 的工作是：

> 把 SPEC + DESIGN 拆成可以安全、增量、可测试地实现的 Milestones。

PLAN 不再负责重新决定核心产品行为。

如果某个开发任务无法在 SPEC / DESIGN 中找到依据，应先标记为设计缺口，而不是开发者自行猜测。