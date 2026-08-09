# ACM-OS SPEC v1

> 状态：SPEC 冻结版，可进入 DESIGN / PLAN 前的设计阶段  
> 日期：2026-08-09  
> 产品：ACM-OS  
> 文档目标：把已经确认的产品行为写到开发者无需猜测；不规定技术栈、数据库、API、前后端架构或具体实现。

---

# 0. 文档地位与变更规则

本文件是 ACM-OS 进入 DESIGN 阶段时的产品规格事实来源。

本文件中的规则分为：

- **MUST / MVP**：第一版核心闭环的阻塞性要求。
- **CONFIRMED**：已确认产品规则，若属于 POST-MVP 也仍是正式蓝图的一部分。
- **POST-MVP**：保留在完整产品蓝图中，但不得阻塞 MVP。
- **DESIGN-DEFERRED**：产品行为已经确定到足够程度，但具体实现、算法、交互布局或技术方案留到 DESIGN。

进入 DESIGN 后，不应重新讨论已冻结的产品原则，除非发现：

1. 两条规格互相矛盾；
2. 某条规则无法验收；
3. 某条规则在实现上导致不可接受的数据损坏或不可恢复风险。

发现上述情况时必须显式提出冲突，不能偷偷改产品行为。

---

# 1. 产品上下文

## 1.1 产品定位

ACM-OS 是：

> **建立在真实 Obsidian / Markdown 知识资产之上的个人 ACM 学习操作系统。**

它不是：

- 在线 OJ；
- Markdown 编辑器；
- AI 解题聊天软件；
- 单纯题库；
- 单纯比赛记录工具；
- 单纯知识图谱；
- 单纯艾宾浩斯提醒器。

## 1.2 核心问题

传统个人 ACM 学习流程容易变成：

```text
比赛 / 刷题
→ 补懂
→ 写进 Obsidian
→ 很久不再打开
→ 最后忘掉
```

ACM-OS 要把它变成：

```text
比赛
→ 学习
→ Obsidian 沉淀
→ 补题
→ 冷启动验证
→ 真会 / 半会 / 回炉
→ 再学习
→ 长期复习
→ 最终真正掌握
```

## 1.3 第一优先级核心闭环

```text
导入一场真实 Contest
↓
产生全部轻量 Problem
↓
选择值得补的题
↓
创建我的笔记
↓
成为正式个人学习 Problem
↓
在 Obsidian 中学习、整理
↓
完成补题
↓
等待未来冷启动验证
↓
重新只看题面做题
↓
到原 OJ 提交
↓
记录提交与帮助使用情况
↓
判断：真会 / 半会 / 回炉
↓
安排下一次学习 / 复习
```

判断功能优先级时，首先问：

> **它是否直接帮助这条核心闭环？**

---

# 2. Goals 与 Non-goals

## 2.1 Goals

MVP 必须验证：

1. 一场真实 Contest 可以可靠进入系统；
2. 同一题在全系统保持“一题一份”；
3. 个人学习笔记始终是真实 Markdown；
4. 补题状态、Review、历史事实可以长期维护；
5. 冷启动 Review 真正隐藏旧知识并记录帮助；
6. Review 能把题判定为真会 / 半会 / 未通过；
7. 半会或未通过会重新进入回炉；
8. Today Plan 能把旧 Problem 在未来重新叫回来；
9. 外部 OJ / Vault / Markdown 异常不会偷偷损坏历史；
10. 用户连续真实使用数周时，系统仍然自然可用。

## 2.2 Non-goals / POST-MVP

以下正式保留在完整蓝图中，但 **不允许阻塞 MVP**：

- 奖励商城；
- 我的出题完整系统；
- 高级统计 / 诊断 Dashboard；
- 完整 AI Candidate 管理中心；
- Global Search；
- 自研 Knowledge Graph；
- 多设备同步；
- 主动通知；
- 复杂 Contest 标签管理；
- 复杂文件历史版本审计；
- 大量动画；
- 重游戏化系统；
- 完整积分经济平衡；
- 实时 AI Chat / Hint / Debug。

---

# 3. 核心产品原则

## 3.1 一题一份，多处引用，多种关系

同一道 Problem 可以同时出现在：

- Contest；
- Knowledge Node 的经典题；
- 某题的额外题；
- 补题队列；
- Review 队列；
- 我的题库。

但不能复制成多份不同 Problem。

## 3.2 Contest Result ≠ Learning Status

比赛结果描述比赛当时发生了什么。

学习状态描述今天这道题在个人长期学习生命周期中的位置。

两者不能互相覆盖。

## 3.3 Markdown 内容与系统事实必须分权

### Markdown 是最终来源的内容

- Problem 学习笔记正文；
- 前置知识；
- 题解正文；
- Solution Routes；
- Route 名称；
- 路线内部章节；
- 用户代码块、证明、思路、易错点；
- 用户自由增加的栏目；
- 额外题目列表；
- Knowledge Markdown 中用户建立的双链关系。

### ACM-OS 是最终来源的系统事实

- Problem 全局身份；
- OJ / 原始链接；
- 首次题面快照；
- 客观难度及来源；
- Contest 历史；
- 比赛结果；
- Learning Status；
- Review Attempt 历史；
- 真会 / 半会 / 未通过；
- 帮助使用记录；
- 失败原因；
- 历史最高状态；
- Review 调度；
- Today Plan 状态；
- 积分流水和奖励历史（若后续实现）。

系统不得让两个来源同时拥有同一事实的权威版本。

## 3.4 历史事实不能被偷偷重写

内容可以被用户删除或修改；已经发生的 Contest、Review、积分等历史事实不能因为后续状态改变而被静默覆盖。

## 3.5 无法确定身份时禁止猜

- Problem 去重无法确定：不自动合并；
- Markdown 写入目标不唯一：不自动选择；
- 文件看不到：不能自动判定被删除；
- Review 事实缺失：不能自动猜结果；
- AI 建议：不能自动变成正式知识关系。

---

# 4. 核心对象与术语

## 4.1 Problem

一道题的长期学习对象。

### 轻量 Problem

比赛导入后全部题目都存在，但只包含：

- 题名；
- 第一次导入题面快照；
- 原始链接；
- OJ；
- 客观难度；
- Contest 上下文；
- 基础元信息。

不会自动创建个人 Markdown。

### 正式个人 Problem

用户点击「创建我的笔记」后：

- 创建真实 Markdown；
- Problem 成为正式个人 Problem；
- 此动作本身 **不等于加入补题**。

## 4.2 Contest

一场考试 / 模拟考试 / 综合训练的历史档案。

Contest 保存“当时发生了什么”，不是长期学习状态的副本。

## 4.3 Review Attempt

一次正式冷启动 / 长期复习尝试。

它保存这一次真实发生的：

- 帮助使用；
- 提交事实；
- 独立性确认；
- 失败原因；
- 最终结果。

## 4.4 Knowledge Node

新知识库中真实存在的一份知识点 Markdown 文件。

没有真实 Markdown 文件时，不算正式 Knowledge Node。

## 4.5 Today Plan

当天根据真实队列与时间预算生成的临时执行视图。

它不是第三个持久队列，不拥有 Problem 的真实生命周期状态。

---

# 5. Problem 身份与学习生命周期

## 5.1 Problem 身份状态

```text
轻量 Problem
    ↓ 创建我的笔记
正式个人 Problem
    ↓ 删除我的笔记
轻量 Problem
```

创建我的笔记只改变 Problem 身份，不改变学习状态。

## 5.2 Learning Status

正式个人 Problem 的用户可见主状态：

```text
未进入学习
    ↓ 加入补题
待补
    ↓ 开始学习
补题中
    ↓ 我已经补懂
已补懂，等待冷启动验证
    ↓ Review Attempt

真会
    → 长期复习

半会 / 未通过
    → 回炉中
    ↓ 重新学习
补题中
```

### 允许的回退

```text
补题中
→ 放回待补
→ 待补
```

不算失败。

```text
已补懂，等待冷启动
→ 撤回补懂 / 继续学习
→ 补题中
```

不创建失败 Review。

### 停止学习此题

待补 / 补题中 / 待冷启动 / 回炉中可执行：

```text
停止学习此题
→ 未进入学习
```

前提：当前没有进行中的 Review Attempt。

停止学习不会删除：

- Markdown；
- Contest 历史；
- Review 历史；
- 历史最高状态。

以后重新「加入补题」时从「待补」开始。

## 5.3 正式复习生命周期的起点

只有用户明确点击：

> 「本次补题完成 / 我已经补懂」

并进入：

> 「已补懂，等待第一次冷启动验证」

之后，才开始计算第一次冷启动。

创建笔记、加入补题、开始学习都不能提前启动 Review 调度。

---

# 6. Review Attempt 规格

## 6.1 统一 Review 模型

第一次冷启动和后续长期复习使用同一种 Review Attempt 行为。

区别只在 Attempt 结束后 Problem 的状态转换。

## 6.2 Review 开始

点击「开始 Review」后：

1. 创建一条 `进行中` Attempt；
2. 同一 Problem 同一时刻最多一个进行中 Attempt；
3. 若已有进行中 Attempt，再次点击必须继续原 Attempt；
4. 中途退出页面不算失败，可回来继续。

## 6.3 Review 初始可见内容

默认只显示：

- 完整题面；
- 原 OJ 提交入口。

默认隐藏：

- 前置知识；
- Contest 历史；
- AI 分析；
- 旧思路；
- Solution Routes；
- 旧代码；
- 额外题；
- 历史 Review。

Review Mode 不提供普通「在 Obsidian 中打开」入口。

## 6.4 受控帮助层级

允许跳级，不强制逐层点击。

系统必须记录实际打开过的帮助：

```text
0. 完全不看帮助
1. 显示前置知识名字
2. 分级 Hint
3. 打开前置知识内容
4. 查看旧思路 / 旧代码
5. 查看完整题解
```

Hints 第一阶段主要由用户学习阶段自行整理；MVP 不要求实时 AI 生成 Hint。

## 6.5 真会 / 半会 / 未通过

### 真会

从题面到最终 AC：

- 思路独立完成；
- 算法实现独立完成；
- Debug 独立完成。

第一发 WA 不影响真会，只要自己找到 Bug 并最终 AC。

允许：

- C++ / STL 文档；
- 语法查询；
- 通用编译器错误含义；
- 自己的通用 ACM 代码骨架。

不允许：

- 现成算法模板；
- 前置知识名字提示；
- Hint；
- AI 指具体 Bug；
- 打开旧题解 / 旧代码；
- 解题方向提示。

### 半会

最终 AC，但使用了解题性帮助，且没有查看完整题解。

例如：

- 前置知识提示；
- Hint；
- 具体 Bug 指示；
- 旧思路 / 旧代码。

半会一定进入「回炉中」。

### 未通过

包括：

- 查看完整题解后才完成；
- 或仍无法完成；
- 或主动点击「我没做出来 / 结束本次复习」。

未通过一定进入「回炉中」。

## 6.6 判定方式

用户不能直接随意点“真会 / 半会”。

结束时至少确认：

- 最终是否 AC；
- 第一发结果；
- 最终结果；
- 总提交次数；
- 思路是否独立；
- 实现是否独立；
- Debug 是否独立；
- 是否使用系统外未记录的解题性帮助。

系统结合：

- 受控帮助记录；
- 用户事实确认；
- 提交结果；

按规则自动得到：

```text
真会 / 半会 / 未通过
```

系统外帮助至少需要能区分：

- 解题提示级；
- 完整题解级。

## 6.7 失败原因

真会无需填写失败原因。

半会 / 未通过必须至少选择 1 个：

- 完全没思路；
- 有方向但关键性质卡住；
- 公式 / 推导卡住；
- 算法会但写不出来；
- 实现错误；
- 边界问题；
- 复杂度判断错误；
- 其他自由文本。

允许多选，因此聚合统计次数可以大于失败 Attempt 数。

## 6.8 Completed Review 历史

已结束 Attempt 是历史事实：

- 不允许直接改写核心结果；
- 不允许物理删除历史来掩盖发生过的事情；
- 误操作可以“作废”，但作废记录本身保留。

## 6.9 Review 状态转换

### 第一次冷启动

```text
待冷启动 + 真会
→ 长期复习
```

```text
待冷启动 + 半会 / 未通过
→ 回炉中
```

### 长期复习

```text
长期复习 + 真会
→ 保持长期复习
→ 下一次间隔原则上更远
```

```text
长期复习 + 半会 / 未通过
→ 回炉中
```

半会 / 未通过后不能直接等待普通下一次 Review，必须重新经历：

```text
回炉中
→ 重新学习
→ 补题中
→ 再次确认补懂
→ 待冷启动
→ 新 Review
```

具体 Review 间隔算法：**DESIGN-DEFERRED**。

---

# 7. 掌握与“彻底消化”

最终保留 6 条严格标准：

1. 看到题能想起它要解决什么；
2. 多种解法清晰；
3. 相关知识点真正理解；
4. 能较快速清晰实现代码；
5. 理解出题背景、会改编并出新题；
6. 能独立解决其他相关 / 迁移题。

只有 **6/6** 才叫「彻底消化」。

不使用“掌握度 87%”作为主要表达，应展示真实证据。

历史最高状态必须保留，例如：

```text
历史最高：彻底消化
首次达到：YYYY-MM-DD
当前：回炉中
```

以后忘了不抹掉曾经达到过的高度。

---

# 8. Contest 与 Snapshot

## 8.1 Contest 与 Problem 严格分离

Contest 是比赛历史档案。

Problem 是长期学习对象。

### Contest Facts Snapshot 保存

整体可保存：

- 整体难度；
- 赛后事实；
- 比赛结束时补题决策。

每道题可保存：

- 当时是否看过；
- 有没有想法；
- 是否识别知识点；
- 是否写代码；
- WA / TLE 等情况；
- 最终结果；
- 队友帮助；
- 当时卡点；
- 比赛结束时最后一份代码；
- 当时记忆印象。

这里保存的是比赛结束后的快照，不是分钟级时间线。

## 8.2 最小 Snapshot 必填

点击「完成赛后整理」前，最低要求：

- Contest 标题；
- 日期；
- 平台；
- 完整 Problem 列表；
- 每题比赛结束时最终结果。

其他赛后复盘字段全部可选。

未记录就是“未知 / 未记录”，系统不能自动脑补。

## 8.3 比赛结束时补题决策

Contest 不保存一份独立动态 Learning Status。

只保存：

```text
计划补 / 暂不补 / 未决定
```

作为比赛结束时的补题决策。

若 Contest 页面需要展示今天的学习状态，应实时引用当前 Problem，而不是复制保存。

## 8.4 公开 Contest 导入

```text
粘贴 Contest 链接
→ 导入标题 / 日期 / 平台 / 链接
→ 导入全部 Problem
→ 保存每题第一次成功导入的题面快照
```

全部题目立即创建 / 关联为轻量 Problem。

若同题已存在，复用原 Problem。

第一次题面快照之后不自动覆盖，不做题面版本管理。

## 8.5 手动 Contest

私有 / 不公开 Contest 可以手动创建。

手动 Contest 与公开 Contest 使用完全相同的：

- Problem 去重；
- Snapshot；
- 补题；
- Review；
- 长期学习规则。

区别只在数据来源。

## 8.6 Contest 状态

导入后先进入：

```text
待整理
```

用户完成必要赛后事实后点击：

```text
完成赛后整理
```

形成正式 Facts Snapshot。

以后允许用户主动纠错，但后续 Problem 学习状态不能自动改写 Snapshot。

## 8.7 Post-Contest Analysis

与 Facts Snapshot 分离。

### Facts Snapshot

描述“比赛当时真实发生了什么”。

### Post-Contest Analysis

描述“后来怎么看这场比赛”，可以在比赛结束后补充 / 替换，例如：

- 整体 AI 分析；
- 难度判断；
- 推荐补题顺序；
- 每题知识点建议；
- 每题 AI 诊断。

Post-Contest Analysis 不得改写：

- Contest Facts；
- Problem Learning Status；
- 正式 Knowledge Relation。

## 8.8 外部 AI Contest 模板

第一阶段不内置实时 AI 聊天。

外部 AI 流程：

```text
Contest
→ 用户提供真实比赛事实
→ 外部 ChatGPT / Claude / Gemini 分析
→ 按固定模板输出
→ 整段复制进入 ACM-OS
→ ACM-OS 解析
```

建议的 MVP 模板契约：

```markdown
# Contest AI Analysis

## Overall

### Overall Difficulty
...

### Overall Diagnosis
...

### Recommended Upsolve Order
1. C
2. F
3. H

### Overall Suggestions
...

## Problem A

### Knowledge Suggestions
- ...

### Analysis
...

### Upsolve Recommendation
...

### Priority Reason
...
```

AI 模板只承载“分析与建议”，不能声明具有系统权威性的 Contest Result / Learning Status。

## 8.9 导入失败

若部分 Problem 获取失败：

- 已成功数据保留；
- Contest 标记「导入不完整」；
- 用户可重试或人工补充；
- 重试不得复制已成功对象；
- 全部 Problem 与首次题面快照齐全前，不宣称导入完成。

## 8.10 重复导入

同一 Contest 再次导入：

- 不创建副本；
- 若原 Contest 不完整，则只补缺；
- 若已完整，则进入已有 Contest；
- 不覆盖题面；
- 不改写 Facts Snapshot；
- 不自动替换 Post-Contest Analysis。

## 8.11 Contest 纠错

Snapshot 后允许主动纠错：

- 当前事实更新；
- 标记“已人工纠正”；
- MVP 不要求完整字段版本历史。

Contest Problem 列表也允许纠错：

- 增加关系：建立 Contest ↔ Problem；
- 移除关系：只解除 Contest 关系，不删除全局 Problem。

## 8.12 Contest 归档与删除

正常真实 Contest 优先使用「归档」。

错误创建 / 重复 Contest 允许明确删除。

删除 Contest 只影响：

- Contest；
- Facts Snapshot；
- Post-Contest Analysis；
- Contest ↔ Problem 关系。

不得影响：

- 正式 Problem；
- Problem Markdown；
- Review 历史；
- Learning Status；
- 历史最高状态；
- 其他 Contest 关系。

只有完全无引用、无历史的纯轻量 Problem 才可作为错误导入垃圾一起清理。

---

# 9. Problem Markdown 规格

## 9.1 初始轻骨架

创建我的笔记时生成：

```markdown
# Problem

## 前置知识

## 题解

### 标准推导

## 额外题目
```

不默认生成：

- 思路；
- 证明；
- 复杂度；
- AC Code；
- 易错点；
- 模板。

用户完全自由写。

## 9.2 Solution Route

只有 `## 题解` 下一级的直接 `###` 才识别为 Solution Route。

`####` 只是路线内部结构。

Route 名称完全由用户自由定义。

例如名称中的 `×` 只是文字，系统不能自动推断 `Route.status = failed`。

Markdown 当前结构动态决定 Route 的新增、改名和删除。

## 9.3 Markdown 修改优先

用户在 Obsidian 修改后，ACM-OS 再次读取时必须展示最新 Markdown。

系统不得通过“自动同步修复”把旧缓存覆盖回去。

解析失败时应报告异常，而不是猜用户本意。

## 9.4 ACM-OS 何时可写 Markdown

只有用户执行明确包含“修改知识内容”的动作时才允许写，例如：

- 创建我的笔记；
- 接受额外题；
- 修改额外题关系；
- 移除额外题；
- 接受已有 Knowledge Node 的正式关系。

以下动作不得偷偷修改正文：

- Review；
- 回炉；
- Learning Status；
- Today Plan；
- Contest AC；
- 积分变化。

## 9.5 局部写入

ACM-OS 不拥有整个文件。

例如接受额外题时，只允许修改 `## 额外题目` 目标区域。

用户自定义栏目、文字和结构不得因为“不符合模板”被删除、重排或重写。

## 9.6 结构歧义

- 结构明确：正常解析 / 写入；
- 结构异常但不影响其他区域：可确定部分继续读取并提示警告；
- 需要写入但目标位置不唯一：拒绝自动写入。

系统不得随机选择一个目标。

## 9.7 文件名 / 路径 / H1

Problem 身份不能由 Markdown 文件路径、文件名或 H1 决定。

用户移动、改名时，只要能无歧义判断为原笔记，应保持原 Problem 关联。

无法判断“移动还是删除”时：

- 标记笔记位置异常 / 待确认；
- 暂时不降级 Problem。

Markdown H1 可自由修改或删除，不改变 OJ 正式题名、Problem 身份或 Contest 关系。

具体文件身份追踪方式：**DESIGN-DEFERRED**。

## 9.8 删除个人 Markdown

前提：没有进行中的 Review Attempt。

```text
正式个人 Problem
→ 删除我的笔记
→ 轻量 Problem
```

同时：

- 当前学习生命周期退出；
- Learning Status 回到「未进入学习」；
- 从补题 / Review 调度中移除；
- Contest 历史保留；
- Review 历史保留；
- 历史最高状态保留。

以后可重新创建我的笔记并再次成为正式个人 Problem。

---

# 10. 额外题 / 迁移题

## 10.1 目的

额外题用于验证“一题 AC 不代表真正会迁移”。

可能类型：

- 同套路巩固；
- 轻微变式；
- 结构迁移；
- 组合知识；
- 难度升级；
- 易混 / 反例；
- 出题灵感。

## 10.2 Candidate 与正式关系

外部 AI 推荐先进入 Candidate。

用户决定：

```text
接受 / 不要
```

接受后写入原 Problem Markdown 的 `## 额外题目`。

接受不等于：

- 正式导入完整 Problem；
- 自动加入补题队列。

若额外题尚不存在，先做轻量引用；以后正式收录时升级为真正 Problem 并保留关系。

## 10.3 Markdown 契约

`## 额外题目` 下的直接一级列表项代表一条额外题关系。

例如：

```markdown
## 额外题目

- [CF 1900X](https://...) 
- [ABC 999 F](https://...) — 结构迁移
```

普通正文不自动变成额外题关系。

无法解析某个一级列表项时：

- 保留原文；
- 标记无法解析；
- 不自动创建错误 Problem。

## 10.4 双向同步

Obsidian：

- 新增 → ACM-OS 出现；
- 修改 → ACM-OS 更新；
- 删除 → ACM-OS 删除关系。

ACM-OS：

- 接受 → 写 Markdown；
- 修改 → 修改 Markdown；
- 移除 → 从 Markdown 删除。

Markdown 是最终来源。

---

# 11. Knowledge Node 与 Obsidian Graph

## 11.1 节点存在条件

Knowledge Node 只有真实 Markdown 文件存在时才正式存在。

AI Suggestion 或数据库记录不能单独制造正式节点。

不存在节点时：

- Candidate 可以存在；
- unresolved link 可以存在；
- 不自动创建空知识节点。

## 11.2 Problem → Knowledge 正式关系

只有 `## 前置知识` 中明确的 Obsidian 双链才建立正式关系。

例如：

```markdown
## 前置知识

- [[线段树]]
- [[扫描线]]
```

普通文字“这题和线段树有关”不算正式关系。

若链接目标不存在：

- 保留双链；
- 标记 unresolved；
- 不自动创建节点。

## 11.3 Knowledge → Knowledge

Knowledge Markdown 中用户主动建立的 Obsidian 双链直接构成 Graph 关系。

ACM-OS 不维护另一套更高优先级的自定义知识图谱关系。

## 11.4 AI Knowledge Candidate

支持：

```text
接受 / 拒绝 / 暂不处理
```

### 接受

若 Knowledge Node 已存在：

- ACM-OS 写入 Problem Markdown 双链；
- 写入成功后才成为正式关系；
- Candidate 消失。

若节点不存在：

- 不自动创建空节点；
- 保留为 Candidate。

### 拒绝

- 从正常待处理列表消失；
- 保存轻量 ignored 记录，避免同一建议反复出现；
- 不进入 Graph；
- 用户以后可恢复。

完整 Candidate 管理中心：**POST-MVP**。

## 11.5 Knowledge Understanding Status

五级：

```text
未学
学过但模糊
基本理解
熟练使用
深入理解
```

保存：

- 当前理解状态；
- 历史最高理解状态；
- 首次达到历史最高状态日期。

MVP 不要求保存完整人工状态变更时间线。

Problem 表现可作为建议依据，但系统不能自动升降级，最终由用户确认。

## 11.6 Knowledge 不制造长期理论债务

Knowledge Status 本身不自动创建理论复习任务。

Problem 仍然是正式长期复习核心。

## 11.7 Knowledge 文件移动 / 删除

移动 / 改名：若能无歧义识别，应保留节点身份、理解状态、Problem 关系和历史。

无法确定时标记位置异常，不立即删除。

用户明确删除 Knowledge Markdown 后：

- 正式 Knowledge Node 不再存在；
- Graph 当前不再显示节点；
- 残留双链成为 unresolved；
- 不自动重建空节点。

历史理解证据可保留为系统历史，但不存在的 Markdown 不能继续作为正式节点显示。

同名重建如何恢复旧身份：**DESIGN-DEFERRED / Edge Case**。

## 11.8 ACM-OS 与 Graph 的职责边界

MVP Knowledge 页面提供：

- Knowledge Node 索引；
- 搜索；
- 当前理解状态；
- 历史最高状态；
- 相关 Problem；
- AI Candidate（最低能力可延后）；
- 在 Obsidian 中打开；
- 打开 / 跳转到 Obsidian Graph。

不做：

- 自研力导向 Graph；
- 自研复杂图谱筛选；
- 自研 mastery heatmap；
- 自研节点拖拽布局。

---

# 12. Today Plan / 队列 / 调度

## 12.1 Today Plan 定位

Today Plan 是综合首页的视觉核心，但不是第三个持久队列。

它从真实状态生成：

```text
Problem Learning Status
+ Review 调度
+ 用户可用 ACM 时间
→ Today Plan
```

Today Plan 排序变化不能改写 Problem 的真实生命周期。

## 12.2 自动候选

### 补题侧

- 待补；
- 补题中；
- 回炉中。

### Review 侧

- 已到期第一次冷启动；
- 已到期长期 Review；
- 进行中的 Review Attempt。

不得自动加入：

- 未进入学习 Problem；
- 尚未到期 Review。

用户仍可以从 Problem 页面主动提前做题。

## 12.3 候选优先原则

原则性优先：

1. 进行中的 Review / 补题；
2. 已到期 Review；
3. 回炉题；
4. 普通待补。

但当预算足够且补题 backlog 存在时，不应长期连续多天只安排 Review。

具体权重与比例：**DESIGN-DEFERRED**。

## 12.4 当天稳定性

当天第一次生成 Today Plan 后，形成相对稳定任务列表。

- 用户拖动顺序保持；
- 再次打开首页不能无理由洗牌；
- 完成任务后，可以建议补充新任务；
- 未经用户允许不能自动塞入额外任务。

## 12.5 拖动排序 vs 固定个人优先级

### 拖动排序

只影响今天。

### 固定个人优先级

跨天持续作为推荐输入，直到用户取消。

固定优先级不能突破生命周期，例如不能强制让未到期 Review 出现。

## 12.6 未完成不产生欠债

当天任务没完成：

- 不算失败；
- 不扣分；
- 不形成“昨日欠债”；
- 第二天根据真实状态、Review 到期情况和新预算重新生成计划。

进行中的真实工作仍然应优先显示“继续”。

## 12.7 时间预算

用户可以维护每周 ACM 大概时间表，并当天临时覆盖。

当天覆盖只影响当天，不修改以后同星期默认值。

时间预算只用于决定安排多少个完整任务：

- 不作为单题限时；
- 不要求记录真实学习时长；
- 不把多个完整任务压缩成浅尝辄止任务。

## 12.8 backlog 过大

即使存在大量到期 Review 和待补题：

- Today Plan 只安排符合当天预算的少量完整任务；
- 其他继续留在真实队列；
- 可展示“还有 N 个待安排候选”，但不全部变成今日必须完成。

## 12.9 当天预算变化

预算变少时：

- 保留已完成；
- 保留进行中；
- 系统重新给剩余任务推荐优先级；
- 不未经用户允许删除其手动稳定列表；
- 用户可选择“采用重新规划”。

## 12.10 完成后补位

计划完成且仍有预算时：

系统可展示：

```text
还想继续？
推荐：Review X / Upsolve Y
```

只有用户主动加入后才成为今日任务。

---

# 13. MVP 使用面

MVP 必须提供 6 个核心使用面。具体导航 / 路由：**DESIGN-DEFERRED**。

## 13.1 首页 / Today

MUST：

- 今日任务；
- 进行中 Review / 补题；
- 当前时间预算；
- 开始 / 继续任务；
- Today Plan 拖动排序；
- 固定 / 取消 Problem 优先级；
- 修改当天时间预算；
- 建议重新规划；
- 完成后建议额外任务。

不要求：

- 高级 Dashboard；
- 连续签到；
- 倒计时；
- 学习时长统计；
- “昨日欠债”；
- 重游戏化首页。

## 13.2 Contest

MUST：

- Contest 列表 / 基础书架；
- 公开 Contest 导入；
- 手动创建；
- 全 Problem 创建 / 关联；
- 导入不完整与补齐；
- Facts Snapshot；
- 每题最终比赛结果；
- 比赛结束时补题决策；
- 人工纠错；
- Post-Contest AI Analysis 粘贴 / 查看；
- 归档。

不要求：

- 所有 OJ 自动适配；
- 分钟级比赛时间线；
- 完整版本审计；
- 内置实时 AI。

## 13.3 Problem

MUST：

- 完整题面快照；
- OJ / 原链接；
- 客观难度及来源；
- 当前 Problem 身份；
- 当前 Learning Status；
- 创建我的笔记；
- 在 Obsidian 打开；
- 读取前置知识；
- 读取 Solution Routes；
- 读取额外题；
- 加入补题；
- 开始学习；
- 放回待补；
- 确认补懂；
- 撤回补懂；
- 停止学习；
- Review 历史摘要。

不要求：

- 在线 OJ；
- Markdown 编辑器；
- 自研代码编辑器；
- 内置 AI；
- 自定义 Graph；
- 学习时长计时。

## 13.4 Review Mode

MUST：

- 默认题面 + 原 OJ；
- 进行中 Attempt；
- 隐藏旧内容；
- 受控帮助；
- 帮助使用记录；
- 第一发 / 最终结果 / 提交次数；
- 独立性事实确认；
- 系统外帮助确认；
- 自动判定；
- 半会 / 未通过失败原因；
- 无 AC 主动结束；
- Completed Attempt 历史。

Review Mode 是独立产品行为面，不能缩水成普通 Problem 页面自己看着做。

## 13.5 我的题库

MUST：

- 全部 Problem 总索引；
- 区分轻量 / 正式个人 Problem；
- 基础名称搜索；
- 基础状态过滤；
- 进入 Problem；
- 找回从 Contest 导入的旧题。

Global Search：**POST-MVP**。

## 13.6 Knowledge

MUST：

- Knowledge Node 索引；
- 搜索；
- 当前理解状态；
- 历史最高状态；
- 相关 Problem；
- 在 Obsidian 打开；
- 打开 Obsidian Graph；
- unresolved link / 文件异常基础展示。

完整 AI Candidate 管理中心：**POST-MVP**。

## 13.7 Settings

MVP 必须让用户能够修改必要配置，例如：

- Obsidian 知识库位置；
- 每周 ACM 时间预算；
- 当天覆盖预算；
- 必要导入设置。

是否使用独立 Settings 页面：**DESIGN-DEFERRED**。

---

# 14. Error / Edge Cases

## 14.1 Vault / 文件暂时不可访问

“看不到”不能自动等于“被删除”。

当知识库、目录、文件因外部原因暂时不可访问：

- 标记 External Source Unavailable；
- 保留 Problem / Knowledge 身份；
- 保留历史；
- 暂停依赖 Markdown 的读取 / 写入；
- 提示用户恢复访问。

只有在 Vault 正常可访问且确认目标文件不存在时，才执行删除语义。

## 14.2 Markdown 局部解析失败

采用局部降级：

- 文件仍存在；
- Problem 仍存在；
- 能确定区域继续展示；
- 异常区域提示 Parse Warning；
- 涉及异常区域的自动写入禁用；
- 不让整个页面失效。

## 14.3 Markdown 写入失败

对于 Markdown 权威内容：

```text
用户执行写入动作
→ 实际写入成功
→ 重新读取 / 确认
→ 才视为正式成功
```

失败时：

- UI 显示失败；
- 正式关系 / 正式状态不提前改变；
- 用户可以重试。

## 14.4 OJ / 外部 Contest 源失效

外部源后来失效不能删除或修改已保存：

- Contest Snapshot；
- 首次题面快照；
- Problem；
- Review 历史。

只影响新的外部操作。

## 14.5 Problem 身份冲突

若系统怀疑两份 Problem 是同一题，但无法确定：

- 标记身份冲突；
- 两边暂时独立存在；
- 禁止自动合并；
- 禁止自动删除。

若双方都已有真实历史，MVP 可以只报告冲突。

复杂历史合并：**DESIGN-DEFERRED / POST-MVP**。

## 14.6 孤立轻量 Problem

只有同时满足：

- 无 Contest 引用；
- 无 Markdown；
- 无学习历史；
- 无 Review；
- 无额外题关系；
- 无其他真实用户历史；

才可以安全清理。

存在任何真实历史就保留，并可从我的题库找到。

## 14.7 进行中 Review 与破坏性动作冲突

存在进行中 Review Attempt 时，禁止直接：

- 删除个人 Markdown；
- 停止学习此题；
- 创建第二个 Review。

必须先：

- 继续 Review；
- 结束为真实结果；
- 或作废误开的 Attempt。

## 14.8 Today Plan 暂不可执行

若任务因 Vault / 外部依赖不可用而暂时无法执行：

- 标记「暂不可执行」；
- 不算失败；
- 不改变 Problem 状态；
- 不自动创建失败 Review；
- 可以建议替代任务。

这是 Today Plan / UI 状态，不是新的 Learning Status。

## 14.9 Review 必要事实缺失

若完成 Attempt 所需事实缺失：

- 不生成真会 / 半会 / 未通过；
- Attempt 保持进行中；
- 明确指出缺少字段。

若半会 / 未通过没有至少一个失败原因，同样不能完成 Attempt。

## 14.10 外部 Obsidian / OJ 打开失败

打开外部程序 / 链接失败：

- 显示明确错误；
- 允许重试或复制路径 / URL；
- 不改变 Learning Status；
- 不自动把 Review 判失败。

---

# 15. 客观难度

只记录相对客观的 CF-like difficulty。

不做“这题对我来说多难”。

- Codeforces 有官方 rating：使用官方 rating；
- 其他平台：可以由外部 AI 估算 CF-equivalent rating；
- 允许 `≈1870` 这类估算；
- 必须明确区分「官方」与「AI 估算」。

---

# 16. 我的出题（POST-MVP 正式蓝图）

存在独立入口「我的出题」。

状态：

```text
草稿
已完成
已发布
```

草稿可以只有：

- 题面想法；
- 条件改动；
- 大致做法。

只有具备：

- 完整题面；
- 可验证正确标程；
- 完整题解；

才升级为正式 Problem。

达到“已完成”即可作为「彻底消化」第 5 条的证据，不要求发布到 OJ。

自己的正式 Problem 不默认进入长期复习，是否进入由用户决定。

MVP 不阻塞于此模块。

---

# 17. 积分 / 奖励 / 统计（POST-MVP）

## 17.1 积分

使用可消费积分，不使用 EXP。

系统根据真实学习事件自动奖励，尽量不允许用户随意自加。

失败不扣积分。

奖励事件未来可包括：

- 完成冷启动；
- 独立 AC；
- 第一发 AC bonus；
- 完成补题；
- 完成迁移题；
- 第一次 6/6；
- 长期稳定保持。

具体数值：**DESIGN-DEFERRED / POST-MVP**。

## 17.2 奖励商城

用户自己创建奖励并定价。

兑换：

```text
点击兑换
→ 立即扣积分
→ 进入待兑现
```

可同时存在多个待兑现。

使用完成后归档。

取消时积分退回。

历史兑换保留当时实际价格。

## 17.3 统计

统计要同时服务：

- 成就；
- 诊断。

不同阶段的 AC 不混成一个数字，应区分：

- Contest；
- 补题；
- 冷启动；
- 迁移题。

不记录：

- 学习时长；
- 单题思考时间。

高级统计 Dashboard：**POST-MVP**。

---

# 18. 视觉与体验方向

整体不是传统 ACM 后台管理系统。

方向：

> **数字书房 + 轻游戏化**

颜色：

> 成熟玫瑰粉 / 灰粉 / dusty pink

避免：

- Neon 粉紫；
- 幼稚卡通化；
- 满屏电竞 RGB。

原则：

> 阅读区域像粉色数字书房，行动区域像轻游戏界面。

Contest 首页保留“书架式视觉”的产品方向。

具体组件、布局、导航、动效：**DESIGN-DEFERRED**。

---

# 19. Quality Attributes（产品级）

## 19.1 Data Integrity — MUST

- Markdown 外部修改优先，不能被旧缓存覆盖；
- 已发生历史不能被后续状态静默重写；
- 身份不确定时不能自动合并；
- 写入失败不能显示成功；
- 暂时不可访问不能当作删除。

## 19.2 Reliability — MUST

局部外部依赖失败应尽量局部降级，而不是让整个系统不可用。

例如：

- 某 Markdown 区域解析失败，不影响其他可确定区域；
- OJ 暂时不可用，不影响已保存历史；
- Vault 临时不可用，不影响系统事实。

## 19.3 User Control — MUST

- 系统推荐 Today Plan，但不强制抢回排序控制权；
- AI 只有建议权；
- 知识状态最终由用户确认；
- Snapshot 纠错必须是用户主动操作。

## 19.4 Accessibility / Performance / Security

详细指标、性能预算、文件访问与本地安全策略、可访问性目标：**DESIGN-DEFERRED**。

DESIGN 必须给出可验收指标，不能使用“快”“流畅”“安全”这类无法验证的描述。

---

# 20. MVP Acceptance Criteria

以下为阻塞性验收标准。

## 20.1 Contest

### AC-CONTEST-01 完整导入

**Given** 用户提供一个支持导入的公开 Contest  
**When** Contest 及全部题目成功获取  
**Then**：

- 创建一份 Contest；
- 全部题目出现在 Contest；
- 每道题有对应轻量 Problem；
- 每道题保存首次题面快照；
- 不自动创建个人 Markdown；
- Contest 标记导入完成。

### AC-CONTEST-02 Problem 去重

**Given** 某题已作为 Problem 存在  
**When** 新 Contest 再次包含该题  
**Then**：复用原 Problem，不创建副本，只新增 Contest 引用。

### AC-CONTEST-03 部分导入失败

**Given** Contest 有 A/B/C/D  
**When** C 获取失败，其他成功  
**Then**：成功数据保留；Contest 为「导入不完整」；允许重试 / 人工补充；重试不复制已有对象。

### AC-CONTEST-04 Snapshot 不被学习覆盖

**Given** Contest Snapshot 中 B = WA  
**When** 后来补题 AC 且 Review 真会  
**Then**：Contest 中仍显示比赛结果 WA。

### AC-CONTEST-05 重复导入

**Given** Contest 已完整存在  
**When** 再次导入同一 Contest  
**Then**：不创建副本；不覆盖题面；不改 Snapshot；不自动替换 Analysis。

## 20.2 Problem Lifecycle

### AC-PROBLEM-01 创建我的笔记

轻量 Problem 点击「创建我的笔记」后：

- 创建真实 Markdown；
- 成为正式个人 Problem；
- Learning Status 仍为「未进入学习」；
- 不自动加入补题。

### AC-PROBLEM-02 加入补题

```text
未进入学习
→ 加入补题
→ 待补
```

Contest Result 不变。

### AC-PROBLEM-03 开始学习

```text
待补
→ 开始学习
→ 补题中
```

### AC-PROBLEM-04 确认补懂

```text
补题中
→ 我已经补懂
→ 已补懂，等待冷启动
```

从此开始第一次冷启动调度。

### AC-PROBLEM-05 撤回补懂

Review 尚未开始时：

```text
待冷启动
→ 撤回补懂
→ 补题中
```

不创建失败 Review。

### AC-PROBLEM-06 停止学习

待补 / 补题中 / 待冷启动 / 回炉中，且无进行中 Review：

```text
停止学习
→ 未进入学习
```

历史与 Markdown 保留。

## 20.3 Review

### AC-REVIEW-01 开始 Review

到期 Problem 点击开始后：

- 创建进行中 Attempt；
- 默认仅显示题面 + 原 OJ；
- 隐藏旧知识内容。

### AC-REVIEW-02 中途退出

离开页面后 Attempt 仍进行中，不判失败，再次进入继续同一个 Attempt。

### AC-REVIEW-03 禁止并行

已有进行中 Attempt 时，再点击 Review 必须继续原 Attempt。

### AC-REVIEW-04 帮助记录

打开前置知识 / Hint / 旧思路 / 完整题解等时，系统必须记录实际使用；允许跳级。

### AC-REVIEW-05 真会

最终 AC，且思路 / 实现 / Debug 独立、无解题性帮助时，结果为真会。第一发 WA 后自行 Debug 仍可真会。

### AC-REVIEW-06 半会

最终 AC，但使用解题性帮助且未查看完整题解时，最高为半会；必须填失败原因；Problem 进入回炉中。

### AC-REVIEW-07 完整题解

打开完整题解后，本次失去真会 / 半会资格，结束时属于未通过并回炉。

### AC-REVIEW-08 没做出来

没有最终 AC，可主动结束为未通过；必须填失败原因；Problem 进入回炉中。

### AC-REVIEW-09 必要事实缺失

最终结果 / 独立性 / 必要失败原因缺失时，不允许完成 Attempt；保持进行中并提示补全。

### AC-REVIEW-10 首次通过

```text
待冷启动 + 真会
→ 长期复习
```

### AC-REVIEW-11 长期退步

```text
长期复习 + 半会 / 未通过
→ 回炉中
```

历史 Attempt 不丢失。

## 20.4 Markdown

### AC-MD-01 外部修改优先

Obsidian 改 Route 名称后，ACM-OS 下次读取显示新名称，不写回旧缓存。

### AC-MD-02 Solution Route

`## 题解` 下直接 `###` 是 Route；`####` 不是。

### AC-MD-03 局部写入

接受额外题时只能修改 `## 额外题目` 目标区域，不能改用户其他自定义正文。

### AC-MD-04 写入失败

文件无法写入时操作失败；Candidate 不升级为正式关系；不能产生系统 / Markdown 双重状态。

### AC-MD-05 解析冲突

存在两个无法区分的 `## 额外题目` 时，自动写入必须拒绝并提示冲突。

### AC-MD-06 Vault 暂时不可用

不能直接判定文件删除；Problem 身份、Review、Learning Status 保持。

## 20.5 Knowledge

### AC-KNOWLEDGE-01 节点存在条件

AI 推荐但无真实 Markdown 时，不创建正式节点 / 空文件，只能是 Candidate / unresolved。

### AC-KNOWLEDGE-02 正式关系

已有 Knowledge Markdown 时，用户接受 Candidate → 写 Problem Markdown 双链；写入成功后才正式成立。

### AC-KNOWLEDGE-03 不自动升级

连续多题真会可以触发“建议升级”，但不能自动改变 Understanding Status。

## 20.6 Today Plan

### AC-TODAY-01 合法候选

自动候选只能来自：

- 待补；
- 补题中；
- 回炉中；
- 已到期第一次 Review；
- 已到期长期 Review；
- 进行中 Review。

未进入学习 / 未到期 Review 不自动加入。

### AC-TODAY-02 当天稳定

用户手动调整顺序后，再次打开首页不能无理由洗牌。

### AC-TODAY-03 未完成不算失败

当天未完成任务：不失败、不扣分、不产生昨日欠债，第二天重新调度。

### AC-TODAY-04 时间预算

时间少时减少完整任务数量，不把多个任务强制压缩成短时任务；不作为单题倒计时。

### AC-TODAY-05 暂不可执行

Vault / 外部依赖异常导致任务无法执行时：标记暂不可执行，不改变 Problem 生命周期，可推荐替代任务。

## 20.7 历史完整性

### AC-HISTORY-01 Completed Attempt 不覆盖

若 Review #1 = 真会、Review #2 = 半会，两条历史必须同时存在；#2 不改写 #1。

### AC-HISTORY-02 历史最高保留

曾经彻底消化、后来回炉时，允许同时显示：

```text
历史最高：彻底消化
当前：回炉中
```

### AC-HISTORY-03 删除个人笔记

正式个人 Problem 删除 Markdown 且无进行中 Review 后：

- 降级轻量 Problem；
- 退出当前学习生命周期；
- Contest / Review / 历史最高保留。

---

# 21. MVP End-to-End Blocking Scenario

MVP 必须至少真实跑通一次：

```text
1. 导入一场真实 Contest
2. 全部题目成为轻量 Problem
3. 选择一题创建个人笔记
4. 加入补题并开始学习
5. 在 Obsidian 修改 Markdown
6. ACM-OS 正确读取最新内容
7. 确认补懂
8. 到期后开始冷启动 Review
9. 默认旧知识被隐藏
10. 去原 OJ 完成真实提交
11. 填写 Review 事实
12. 系统正确判定真会 / 半会 / 未通过
13. 真会进入长期复习，或半会 / 未通过进入回炉
14. Completed Attempt 被完整保留
15. 后续 Today Plan 能再次把该 Problem 叫回来
```

只有这条链真实跑通，才能判定：

> **ACM-OS MVP 核心闭环通过验收。**

---

# 22. MVP 成功与 rethink 标准

## 22.1 成功标准

不能只是“功能写完”。

成功是：

> ACM-OS 可以承载连续几周真实 ACM 学习；用户会主动打开它完成补题 / 复习；旧 Problem 真正被系统重新叫回来重新做、重新暴露问题、再次掌握。

## 22.2 rethink 标准

核心闭环可用后，连续真实使用约 2～4 周，用户仍不愿主动打开 ACM-OS，补题与复习仍然绕开它：

> **暂停堆功能，重新检查核心流程。**

优先检查：

- Today Plan 是否太重；
- 记录步骤是否太多；
- 冷启动是否麻烦；
- Obsidian 来回跳是否痛苦；
- 首页是否缺乏吸引力；
- 核心流程是否真的帮助学习。

不要优先堆：

- 统计图；
- 游戏化；
- AI；
- 图谱；
- 页面数量。

---

# 23. DESIGN-DEFERRED 清单

进入 DESIGN 时必须解决，但不能在 SPEC 阶段提前锁死具体技术方案：

1. Problem Markdown / Knowledge Markdown 的稳定身份追踪机制；
2. OJ 支持范围与导入适配策略；
3. Review 第一次间隔、长期调度算法与回炉后间隔；
4. Today Plan 的候选评分、Review/Upsolve 平衡权重；
5. Today Plan 对“任务预计预算”的估算方式；
6. MVP 的导航 / 路由 / 页面布局；
7. 是否有独立 Settings 页面；
8. Contest 书架具体交互；
9. Markdown 局部安全写入与冲突恢复体验；
10. 文件位置异常后的重新关联体验；
11. Problem / Knowledge 同名重建或身份冲突的人工恢复方式；
12. 外部 Obsidian / OJ 打开失败的恢复入口；
13. Review Mode 具体求助 UI 与事实确认 UI；
14. Post-Contest AI Analysis 解析失败体验；
15. 可访问性目标；
16. 性能预算；
17. 本地数据与文件访问安全策略；
18. 日志 / 诊断 / 错误可观察性；
19. 数据备份与实现级 rollback 策略；
20. MVP 首次初始化流程。

以下仍然明确不属于 DESIGN 第一阶段的 MVP 必做：

- 奖励商城；
- 我的出题完整系统；
- 高级统计；
- Global Search；
- 自研 Graph；
- 多设备；
- 实时 AI Chat。

---

# 24. PLAN 阶段进入条件

只有当 DESIGN 至少回答以下问题后才进入 PLAN：

- 6 个 MVP 使用面的信息架构与交互路径；
- 本地 Markdown 文件如何可靠识别与安全写入；
- Contest 导入的支持范围与失败行为；
- Problem / Contest / Review / Knowledge / Today Plan 的产品数据契约；
- Review 调度算法；
- Today Plan 调度算法；
- MVP 配置与初始化；
- 核心错误恢复 UI；
- 关键性能 / 安全 /可访问性指标；
- E2E 验收路径如何落地测试。

完成 DESIGN 后再将本 SPEC 拆成可执行的 PLAN / Milestones，不允许直接从当前 SPEC 跳到大规模编码。

---

# 25. SPEC v1 最终结论

ACM-OS 第一版的核心不是：

> “把比赛、题库、图谱、AI、游戏化、统计都做出来。”

而是：

> **让一道真实 ACM Problem 从 Contest 历史进入个人 Markdown 学习资产，经过补题、冷启动、真实 OJ 提交、帮助记录与客观判定，在遗忘时被重新叫回来并回炉，直到形成长期可靠掌握。**

所有设计与开发决策应优先保护以下四件事：

1. **真实 Markdown 资产不被系统绑架或覆盖；**
2. **比赛历史、学习历史和 Review 历史不被后续状态重写；**
3. **冷启动验证必须真的能区分“真会”和“靠提示会”；**
4. **Today Plan 必须让用户愿意每天打开，而不是制造任务债务。**

如果这四件事没有跑通，其余高级功能都不应成为优先级。
