# ACM-OS 项目记忆

更新日期：2026-10-26，时区 Asia/Shanghai。本文是接续开发的入口；源码改变后应更新对应条目。

## 1. 当前状态

- 2026-10-26 v0.1.3 发布准备：应用、Tauri 配置、Rust workspace 和 npm/Cargo 锁文件版本统一为 0.1.3，依赖声明未变，pnpm 锁文件无根版本字段。包含手动比赛表单清空和原题系统浏览器打开两项修复，不改变数据库结构 38。前端及 release/NSIS 构建通过；本轮完整 Rust 405 项通过（2 项真实网络忽略）、DOM 92 项、样式/可访问性/路由/日期 27 项、边界 5 项及静态检查通过。GitHub 附件上传与公开结果另记；本机保留上方记录的 0.1.2 修复构建，本轮未再次替换安装文件。

- 2026-10-26 已将手动比赛成功后清空表单及原题链接系统浏览器打开的 release 构建应用到本机 `D:/Software/app/ACMOS/ACM-OS/acm-os.exe`，SHA-256 与构建一致：`6048a9de0913d37ba3280aec181e92e6323cdcc0c570964e1a4d354d1c6f84c7`。版本仍为 0.1.2，是包含本次修复的本机构建，与 GitHub v0.1.2 附件不同；未发布新版本。旧程序正常退出，备份为 `D:/Software/app/ACMOS/ACM-OS/backups/2026-10-26-before-source-link-fix/acm-os.exe`，哈希 `3ba08eba8d673517a409cf2265630dd4ea02cebc53ad24798aa554611341827c`。已通过 UI Automation 从比赛进入 A. Explosion 并点击“打开原题”（URL 为 `https://codeforces.com/gym/702542/problem/A`），Chrome 被唤起且应用未显示浏览器启动失败；当前 Chrome 有 Codeforces 首页窗口（显示 Enter/Register），另有 Gym E 题窗口，未确认 A 题成功加载，不能将浏览器启动称为原站访问通过。未手动修改工作区数据或 Markdown。

- 2026-10-26 开发源码将两种题目详情路由的“打开原题”和比赛详情的“打开原比赛”接入已有系统浏览器 IPC，替代普通新窗口链接；打开失败显示提示与可复制网址并可重试。HTTP/HTTPS 原题来源兼容其他 OJ，拒绝非网页协议、凭据及地址中的空白/控制字符；无来源本地题继续隐藏链接。本机应用与实际检查结果见上方最新记录；网页是否可访问仍取决于原站及网络。

- 2026-10-26 开发源码修复手动比赛保存后仍保留上次输入的问题：成功后清空比赛 ID、标题、日期和所有题面，恢复为默认题号 A 的一张空白题目卡；失败保留全部输入供重试。现用比赛收藏页与旧比赛书架入口同步处理。本机应用结果见上方最新记录，未公开发布新版本。

- 2026-10-22 已公开发布 [v0.1.2](https://github.com/Mxiaocao/acm-os/releases/tag/v0.1.2) 并独立查询确认为 latest。应用和 Rust workspace 版本为 0.1.2；标签对应构建来源 `f7704201f3983fd215dfbb67d96def38952f42cf`，安装包、免安装 ZIP 和 SHA256SUMS 三个附件均已按服务器大小、uploaded 状态和 SHA-256 与本地核对。旧 v0.1.0/v0.1.1 标签和附件保持原样。Windows x64 安装包 SHA-256：`402ec767aa6fd9b67c4aca015caacabe1666d2241df84cd35d3fc1c8486fe883`；免安装 ZIP：`16625ee073a85fcde8b86bf9e2e02d724776acecc84bdbf84d328a1caa7d8672`。本机已安装并启动同一构建，当前工作区结构 38。后续仅补发布记忆与日志，不改变标签或二进制来源。

- 2026-10-22 v0.1.2 已完成发布准备并应用到本机。应用、npm/Cargo 锁文件和 Rust workspace 包统一为 0.1.2；Vite 更新为兼容补丁 8.2.4，npm 与 pnpm 锁文件同步。最终 release 程序及 NSIS 安装包构建通过，程序 SHA-256：`3ba08eba8d673517a409cf2265630dd4ea02cebc53ad24798aa554611341827c`。本机 `D:/Software/app/ACMOS/ACM-OS/acm-os.exe` 与构建一致，版本资源 0.1.2，旧程序备份在 `D:/Software/app/ACMOS/ACM-OS/backups/2026-10-22-before-v0.1.2/acm-os.exe`，旧哈希为 `ed0cc6118dca6382751c3196eda4b9ef73c4dd844a3b302128e10caef3848e00`。替换前旧程序已退出，无需强制结束。启动响应正常，UI Automation 读取到“今日 · ACM-OS”。当前工作区正式迁移 37→38，新增备份 `schema-37-to-38-1792664686611217900.sqlite3` 存在且只读验证结构 37 和完整性正常；升级后完整性正常，题目 15、比赛 1、题库成员 8、笔记绑定 2、复习尝试 0、参赛记录 0 均保持一致，日期覆盖表 0，未补造历史。GitHub v0.1.2 已公开，结果见本节最新发布记录。

- 2026-10-22 按用户验收后要求完成正式 release 构建：`npm run tauri -- build --no-bundle` 成功，原始程序为 `src-tauri/target/release/acm-os.exe`，交付副本为 `output/completion-release/ACM-OS.exe`，两者 SHA-256 一致：`b9cbf5fe5aa300cbfb6bf841cf24d75e3a2be9e51d8e7876a9a7037602f722e3`。资源产品名 ACM-OS、文件版本 0.1.1、大小 37,593,088 字节；同目录含 README、SHA256SUMS 和验证 JSON。本轮只重新执行前端与正式 release 构建、资源信息和文件一致性检查、Git diff 检查，没有重跑此前 404/89 项测试，也未称为完整桌面 E2E。未启动正式构建、未替换安装目录程序、未修改真实工作区或发布 GitHub；安装 EXE 只读核对仍为 `ed0cc6118dca6382751c3196eda4b9ef73c4dd844a3b302128e10caef3848e00`。构建日志在 `output/completion-release-build.log`。

- 2026-10-22 用户已人工查看并认可独立桌面预览，随后要求正式构建。预览位于 `output/completion-preview/ACM-OS-做题日期预览.exe`，使用独立标识 `dev.acmos.completion-preview`、独立 WebView2 目录和示例 Vault（`output/completion-preview/Vault`），没有替换安装目录程序。实际 UI Automation 读取到示例题 10 月 20 日做题 1 道、10 月 22 日 0 道，以及题库导入日期为 10 月 22 日；这是预览启动与展示检查，不是完整桌面 E2E。正式 release 构建结果见本节最新条目。

- 2026-10-22 开发源码已支持入库同时填写实际做题日期（单题手动录入、Codeforces 单题、比赛批量选题，默认今天），题目与日期原子保存，重复入库保留原日期。月历继续支持日期补录与修改：顶部和日期详情提供题库选题表单，已有首次做题记录可直接改日期。默认仍按首次主动入库日记录，手动日期覆盖默认值，跨月移动而不额外新增一条；支持旧迁移成员明确补录。新增结构版本 38，已在 2026-10-22 以 0.1.2 正式应用到本机；公开发布状态见最新条目。

- 2026-10-15 参赛记录功能已正式应用到本机：比赛详情支持现场/实时参赛、赛后 VP 的补录、修改和删除；月历按实际本地开始日期显示比赛场次与详情，允许同一比赛多次 VP。`D:/Software/app/ACMOS/ACM-OS/acm-os.exe` 与新 release 构建 SHA-256 一致：`ed0cc6118dca6382751c3196eda4b9ef73c4dd844a3b302128e10caef3848e00`，程序资源版本仍为 0.1.1；这是包含未发布改动的本机构建，不是 GitHub v0.1.1 原附件。旧程序正常退出后替换并启动，当前工作区正式迁移到 37，迁移前数据库备份已生成；旧 EXE 备份在 `D:/Software/app/ACMOS/ACM-OS/backups/2026-10-15-before-participation-update/acm-os.exe`。未更改 GitHub Release。
- 2026-10-15 已公开发布 GitHub v0.1.1 并确认为 latest，包含月历做题记录、笔记阅读与 Prompt 改进。应用、Rust workspace 包和对应锁文件版本均为 0.1.1；pnpm 锁文件无根项目版本字段且本轮依赖声明未变。Windows x64 安装包、免安装 ZIP、SHA-256 校验文件均已上传并核验服务端大小和摘要，v0.1.0 原附件保持不变。
- 2026-10-15 月历优化已正式应用到 `D:/Software/app/ACMOS/ACM-OS/acm-os.exe`。正常关闭旧程序失败后，用户明确要求关闭，随后结束旧进程、替换并重新启动；安装文件与本次 release 构建 SHA-256 一致：`aa76e40c38c43e965e67633de2bf196aaf456471be6b3c98bf2e9f5b0823bcfc`。启动进程响应正常，WebView 显示“今日 · ACM-OS”。替换前备份位于 `D:/Software/app/ACMOS/ACM-OS/backups/2026-10-15-before-calendar-update/acm-os.exe`。本轮未修改工作区数据库或个人 Markdown，未更新 GitHub Release。
- 项目：面向算法竞赛和日常刷题的 Windows 桌面学习工作台。
- 核心流程：比赛 → 赛后补题 → 个人题解 → 冷启动复习 → 长期复习；通过 Obsidian 管理 Markdown 题解与知识笔记。
- 仓库：[Mxiaocao/acm-os](https://github.com/Mxiaocao/acm-os)，默认分支 `main`。
- 最新发布：[v0.1.2](https://github.com/Mxiaocao/acm-os/releases/tag/v0.1.2)，Windows x64 安装包、免安装 ZIP、SHA-256 校验文件；构建源码与标签对应 `f7704201f3983fd215dfbb67d96def38952f42cf`。本机安装版本 0.1.2、当前工作区结构 38，与发布构建一致。发布后的文档提交不改变二进制或标签，旧 v0.1.1 附件保留原样。
- 历史 [v0.1.0](https://github.com/Mxiaocao/acm-os/releases/tag/v0.1.0) 二进制对应 `27725d16506d9ea6d51262eae3fb214997e6879d`；发布后的完整项目介绍提交为 `712a9c6`，原标签与附件未改动。
- 安装包尚未进行 Windows 代码签名；当前没有产品自动更新插件。
- 2026-10-14 已将笔记改动的本地 release 构建应用到 `D:/Software/app/ACMOS/ACM-OS/acm-os.exe` 并启动；本机程序 SHA-256 为 `b8c07f5ea1b33cb34c5aa462413aeeb1aa0fdc436465fd3de5a16a4201bbcd07`。版本号仍为 0.1.0，本次没有更新 GitHub Release 附件或标签。原程序备份在 `D:/Software/app/ACMOS/ACM-OS/backups/2026-10-14-before-note-update/acm-os.exe`。
- 本地开发路径曾为 `E:/03-Projects/acm-os/acm-os`。本文的源码链接采用仓库相对路径，便于其他环境使用。

## 2. 用户已经确认的产品取舍

1. 保留樱花粉界面。应用外部图标使用“未来知识代码之书”；侧栏樱花标识是独立的界面元素。
2. 比赛书架每层容量由窗口和实际可用宽度决定。先填同一柜的第一、第二、第三层，再增加下一柜；不能第一层满了就新建柜，也不能固定每层五本。
3. 学习月历是左侧独立栏目；今日页保留每日任务执行功能。
4. 月历采用有间隔的日期单元、工作量提示和下方详情；统计与任务必须来自实际数据。
5. 单题导入不要求先导入比赛。支持 Codeforces 单题链接，也支持无来源链接的手动录入。
6. 题库强调完整题号、标题、题面预览、真实难度、首次导入日期，并保留关键词搜索。
7. 题库外层摘要也要渲染 LaTeX；截断不能切断完整公式。
8. 题面录入框限制在面板宽度内，只能上下调整尺寸。
9. 导入入口、搜索栏和空题库提示分区排列并拉开间距。即使题库为空也保留搜索栏。
10. 年份全局共用，各分类均可选用；新建年份立即显示，输入已有年份直接复用。创建年份本身不会创建比赛或关联比赛。
11. 项目介绍必须覆盖整个学习流程，尤其是补题和 Obsidian 联动，不能只列最近的 UI 修改。
12. 个人笔记的方法属于“题解”章节，不另设“解题路线”标题。方法名显示为加粗的强调色文字，不使用方框、底色或“打开”箭头；点击“标准推导”等文字，弹窗阅读对应正文。AI 辅助编写以用户提供材料为主，章节结构需兼容 Obsidian 与现有解析。
13. 笔记阅读以 Obsidian Markdown 为源，按实际章节顺序展示；顶部大致题意完整显示，前置知识、Hints、题解、额外题目等默认折叠。题解展开后列方法文字入口。AI 入口用于保存和复制 Prompt，不自动整理材料或调用模型；优先读取笔记中的 `## Prompt` / `## 提示词`，缺少时提供默认模板。
14. Prompt 属于创建笔记之前的准备操作，入口放在上方“个人 Markdown”区域，尚未创建笔记时也必须可用；“我的笔记”不再展示 Prompt。已有笔记中的自定义提示词可继续在上方入口复制。
15. 提示词入口标题仅为“Prompt”。默认模板按用户最新说明使用单美元符号 `$...$` 写公式；短代码与主逻辑集中在单行；中文与英文或数字之间不加空格（代码、公式等语法所需空格保留）。题解必须有逐步跟踪样例变量、从小数据扩大规模的路线，以及标准推导；前置知识以链接列表为主，额外题目介绍后附可靠原题链接。模板内包含适用于题解的去 AI 味写作规则。
16. 用户通常做完题才加入题库；首次主动入库默认视为入库当天做过的题；2026-10-22 用户要求允许忘记当天入库后的补录，入库时可顺便填写实际做题日期（默认今天），月历也可调整首次做题记录；比赛选题同批共用日期，不同日期可分批加入。月历展示每天做题数、题目列表和“查看题解”链接；同一天同一道题的入库、学习与复习只计一道，保留活动与复习判定说明。
17. 月历增加当日比赛，现场/实时参赛与赛后 VP 分开计场次，按实际本地开始日期归属，跨午夜只计一场。同一原比赛可有多次参赛记录；导入、收藏、补题不代表参赛。参赛时间、赛内 AC 数及总题数手动记录，原比赛日期与旧赛后快照不用于推测 VP 成绩或日期。比赛场次、赛内 AC 与现有做题数分别展示，避免重复计数；旧比赛不自动补记参赛历史。

## 3. 功能与源码定位

| 模块 | 当前能力 | 主要入口 |
|---|---|---|
| 启动与工作区 | 恢复、配置、正常模式；多个 Vault 的注册、切换与隔离 | [App.tsx](src/app/App.tsx)、[VaultManagerPanel.tsx](src/app/VaultManagerPanel.tsx)、[vault_ipc.rs](src-tauri/src/vault_ipc.rs)、[vault_manager.rs](src-tauri/crates/acm-os-infrastructure/src/vault_manager.rs) |
| 比赛 | Codeforces 自动/手动导入、赛后事实、分类/系列/年份、归档、删除、响应式书架 | [shells.tsx](src/app/shells.tsx) 中 `ContestLibraryPage` / `ContestCabinet` / `ContestDetail`、[contest.ts](src/ipc/contest.ts) |
| 参赛记录 | 现场与 VP 独立记录、重复 VP、补录与修改时间/成绩、删除确认、月历场次与跳转 | [ContestParticipationPanel.tsx](src/app/ContestParticipationPanel.tsx)、[类型化 IPC](src/ipc/contest-participation.ts)、[应用端口](src-tauri/crates/acm-os-application/src/contest_participation.rs)、[持久化](src-tauri/crates/acm-os-infrastructure/src/persistence/contest_participation.rs)、[Tauri](src-tauri/src/participation_ipc.rs) |
| 个人题库 | 比赛选题入库、独立单题导入、入库时填写实际做题日期、搜索、预览、基础编辑与软删除 | [ProblemLibrary.tsx](src/app/ProblemLibrary.tsx)、[SingleProblemImport.tsx](src/app/SingleProblemImport.tsx)、[ProblemActions.tsx](src/app/ProblemActions.tsx) |
| 单题后端 | 输入验证、强身份去重、原子入库、首次题面快照 | [应用用例](src-tauri/crates/acm-os-application/src/single_problem.rs)、[持久化](src-tauri/crates/acm-os-infrastructure/src/persistence/single_problem.rs)、[IPC 包装](src/ipc/single-problem.ts) |
| 学习与复习 | 补题、学习、首次冷启动、提前检查、长期复习、帮助记录、失败原因与历史 | [Domain](src-tauri/crates/acm-os-domain/src/lib.rs)、[Application](src-tauri/crates/acm-os-application/src/lib.rs)、`shells.tsx` 中 `ReviewFocusShell` |
| 今日 | 时间预算、任务重排、完成、建议、预览重新规划；展示真实摘要 | `shells.tsx` 的今日页面、[TodayOverview.tsx](src/app/TodayOverview.tsx) |
| 学习月历 | 首次主动入库默认计入当天，支持实际做题日期补录/修改、按日按题去重、题解链接、历史学习与复习、当前下一次到期安排 | [LearningCalendar.tsx](src/app/LearningCalendar.tsx)、[日期补录表单](src/app/CalendarCompletionEditor.tsx)、[日期计算](src/app/calendar-dates.ts)、[月历持久化](src-tauri/crates/acm-os-infrastructure/src/persistence/learning_calendar.rs) |
| 知识与 Obsidian | Markdown 发现、搜索、理解等级、关联解析、打开笔记与外部图谱、绑定异常修复 | `shells.tsx` 中 `KnowledgePage`、[knowledge_index.rs](src-tauri/crates/acm-os-infrastructure/src/knowledge_index.rs)、[knowledge.ts](src/ipc/knowledge.ts) |
| 个人笔记阅读与 Prompt | 原文顺序、题意完整显示、其他章节默认折叠；“题解”下的方法文字打开独立正文弹窗，支持公式、代码、表格；刷新外部编辑；上方个人 Markdown 区域可在创建笔记前复制 Prompt | [PersonalNotePanel.tsx](src/app/PersonalNotePanel.tsx)、[Prompt 入口](src/app/NotePromptPanel.tsx)、[提示词模板](src/app/note-prompt.ts)、[笔记样式](src/app/personal-note.css)、[Markdown 解析](src-tauri/crates/acm-os-infrastructure/src/markdown.rs) |
| 奖励 | 启用模式、XP/金币/等级、合格复习奖励、自定义兑换与退款记录 | `shells.tsx` 中 `RewardPage`、[reward.ts](src/ipc/reward.ts)、Domain 与持久化中的奖励策略 |
| 外观 | 樱花粉样式、月历布局、书架与图标素材 | [app.css](src/app/app.css)、[sakura-theme.css](src/app/sakura-theme.css)、[learning-calendar.css](src/app/learning-calendar.css)、[图标配置](src-tauri/tauri.conf.json) |

路由定义见 [routing.ts](src/app/routing.ts)：`/today`、`/calendar`、`/contests`、`/problems`、`/knowledge`、`/reward`、`/settings`。题目详情同时支持外部题号路由与内部 ID 路由；专注复习使用 `/review/:attemptId`。

## 4. 数据和业务规则

- React 通过类型化 IPC 调用后端，不持有通用 SQL 或文件系统写权限。Domain 处理纯规则，Application 编排用例，Infrastructure 负责数据库、文件和网络实现，Tauri 负责组合与 DTO 边界。
- SQLite 保存身份、学习状态、计划、复习历史与奖励等系统事实；Vault 内 Markdown 保存个人题解与知识内容。数据库备份不覆盖 Markdown 文件。
- 当前源码与本机当前工作区数据库结构版本均为 38；此前公开的 v0.1.1 为 36。其他工作区在后续挂载时由启动流程迁移。`contest_participations`（37）和 `library_completion_dates`（38）均走正式迁移与结构校验；开发验证使用临时数据库，本机应用由程序启动执行迁移，不直接修改用户数据库绕过契约。
- `problems.id` 是稳定内部身份。修改显示标题或来源 URL 不改变外部身份，也不自动重新抓取题面。
- 比赛题目与个人题库成员分开；比赛导入不会默认把所有题目加入个人题库。
- 独立单题导入使用已有表，不创建空比赛、placement 或比赛题目关系。无 URL 的手动题目使用内部 `acmos://local-problem/...` 地址，界面不应提供该地址的“原题网站”链接。
- 去重依赖平台与外部身份，不按相似标题合并。重复导入保持首次成功题面及原始元数据，并恢复对应题库可见性。
- 导入日期来自 `problems.created_at_utc`，按本地日期显示；不是“本次打开日期”，也不是再次加入题库的时间。前端没有日期数据时显示未记录。
- 题库搜索使用题面提取的文本；展示只插入安全文本与 KaTeX 生成的公式，不能直接把来源 HTML 放进列表。
- 题目软删除保留历史及笔记；删除题库题目与删除个人题解是不同操作。当前没有通用回收站 UI。
- 月历查询只读，不生成新的今日计划。历史来源为首次做题（优先采用 `library_completion_dates.local_date` 用户填写的日期（入库或月历修改），否则采用 `problem_library_items.origin = 'explicit'` 的 `added_at_utc` 本地日期）、计划完成、复习周期创建和已完成复习；同一天同一道题只计一道，月汇总为各日做题数之和。重复导入或恢复软删除保留首次入库及手动做题日期；修改实际日期仅移动首次做题记录，不改变入库/导入时间、学习或复习历史、计划、奖励和 Markdown。日期范围为 1900-01-01 至今天，明确保存为本地日期；旧迁移成员也可手动补录，但不自动猜测其历史。比赛导入和创建笔记本身不计做题。`legacy` 迁移成员缺少可靠主动入库时间，不凭迁移数据补记。题库列表的首次导入日期仍采用 `problems.created_at_utc`，与月历首次入库日期用途不同。软删除保留历史记录，但被删除题目的详情可能不可用。未来展示每题当前下一次复习，逾期汇总到今天。不能将其描述为完整未来复习模拟或原始 OJ 做题日志。
- 学习状态包括待学、待补题、学习中、等待冷启动、重学、长期复习。复习依据记录的事实判定掌握程度，不代表软件自动编译提交并验证 OJ AC。
- Vault 切换使用后端上下文和 epoch 隔离；业务调用沿用现有 IPC 包装，不能绕过旧上下文拒绝机制。
- 参赛记录属于当前 Vault，使用 UUID 稳定标识，保存重试按相同 ID 更新，同一比赛/本地日期/开始时间不可重复登记；修改成绩不新增参赛。日期和时间保存为明确的本地开始事实，不受原比赛举办时间影响。AC 数可留空，0 是明确成绩，总题数可留空；有总题数时 AC 不得超过它。记录不改赛后快照、个人题库、学习状态、计划或奖励。删除比赛时保留参赛标题与事实，外键设空，月历显示“比赛已删除”且不提供失效跳转；删除单次参赛记录则移除该次场次。

## 5. 已知范围和未确认事项

- 自动题面导入围绕 Codeforces；其他来源走手动录入，不能宣称已实现通用多 OJ 抓取。
- 题目没有持久化算法标签字段；列表不得伪造标签或缺失的评分。
- 知识图谱按钮打开 Obsidian 原生图谱，需要启用 Obsidian CLI；本应用没有内置自由图谱画布。
- 赛后分析是粘贴文本及固定结构解析，没有接入自动生成式 AI 服务。
- 个人题解的 AI 入口只展示和复制 Prompt，没有内置模型调用、文件上传、材料自动拼接或 AI 结果自动写入。自定义 Prompt 存在 Obsidian Markdown 的二级章节中，应用只读；没有时提供默认模板。题面、截图/PDF 等在外部 AI 对话中补充；结果由用户检查并存入 Obsidian，保留原属性与个人记录后刷新。章节与方法正文按当前 Markdown 的 Rust UTF-8 字节边界提取，通过现有工作区 IPC 返回；前端过滤 HTML 后渲染，不直接访问文件。
- 奖励模式与复习奖励已接入；“学会题目奖励”虽有后端策略，但不能据此承诺标记学会自动发奖。完整逐条账本展示也未完成。
- 外部题号路由限制为 `[A-Z][0-9]?`，领域允许范围更宽；长题号是需要另行核验的边界。
- 最近发布验证没有重新进行真实 Codeforces 网络用例、真实 Obsidian 联动或全部桌面 E2E。这些外部条件不能写成已完整验收。
- 后续重构、标签、统计、签名、自动更新等只是可能的改进方向，尚无新的任务授权。

## 6. 验证记录与常用命令

2026-10-26 本机更新本轮实际验证：重新执行 `npm run tauri -- build --no-bundle --ci`，前端及 release 构建通过；旧程序备份哈希一致、正常退出，安装文件与新构建哈希一致且启动响应正常。UI Automation 在安装版进入截图对应题目并调用原题链接，Chrome 已启动且应用未报打开失败；未确认 A 题原站内容加载成功。未重新执行上轮 DOM/Rust 套件、完整桌面 E2E、真实工作区写入验证或 GitHub 发布。日志在 `output/external-source-release-build.log`。

2026-10-26 原题链接修复本轮实际验证：最终前端构建通过；按 `Original source|Manual Contest` 筛选的 DOM 4 项通过（Node 堆上限 4096MB），覆盖两种题目详情、原比赛的 IPC 打开、阻止 WebView 默认跳转、失败提示/网址、防重复打开与重试，并重验手动比赛表单清空；Rust `-p acm-os --lib -j 1 original_url_validation` 1 项通过；边界 5 项及静态检查、`git diff --check` 通过。首次并行验证因内存/分页文件不足失败；串行定位发现新增比赛用例模拟了错误的分类查询命令，断言比较 DOM 节点导致内存耗尽，修正模拟与文本断言后最终重跑通过。未重跑全套 DOM/Rust，未在安装版验证系统浏览器实际启动或真实原站访问，未构建 release、替换本机或发布。

2026-10-26 手动比赛表单清空修复本轮实际验证：前端 `npm run build` 通过；按 `Manual Contest` 筛选的 DOM 回归 1 项通过，覆盖多题失败保留、原参数重试、成功后清空全部输入与恢复单卡、继续提供入库选题；边界测试 5 项及静态检查通过，`git diff --check` 通过。本轮未重跑完整 DOM 或 Rust 测试，未执行桌面 E2E、release 构建、安装替换或公开发布。

2026-10-22 v0.1.2 发布本轮实际验证：完整锁定 Cargo 依赖的 Rust workspace 404 项通过、2 项真实网络用例忽略；Vite 8.2.4 下最终完整 DOM 89 项、样式/可访问性/路由/日期 27 项、边界 5 项和静态检查通过。npm 缓存安装通过，后续按兼容补丁升级 Vite 并同步 npm/pnpm 锁文件；npm CLI 审计端点超时，改用相同官方 bulk 公告接口查询最终锁文件各包版本，返回空对象，未把失败的 npm CLI 请求当作成功。补丁后重新执行最终前端与 release/NSIS 构建、DOM 和前端静态检查。安装文件与构建哈希一致，版本 0.1.2；本机启动与正式迁移、备份和数据数量检查通过。未进行真实 Codeforces 网络、Obsidian 人工联动或完整桌面 E2E。日志在 `output/release-v0.1.2/`。

2026-10-22 按用户验收后要求完成正式 release 构建：`npm run tauri -- build --no-bundle` 成功，原始程序为 `src-tauri/target/release/acm-os.exe`，交付副本为 `output/completion-release/ACM-OS.exe`，两者 SHA-256 一致：`b9cbf5fe5aa300cbfb6bf841cf24d75e3a2be9e51d8e7876a9a7037602f722e3`。资源产品名 ACM-OS、文件版本 0.1.1、大小 37,593,088 字节；同目录含 README、SHA256SUMS 和验证 JSON。本轮只重新执行前端与正式 release 构建、资源信息和文件一致性检查、Git diff 检查，没有重跑此前 404/89 项测试，也未称为完整桌面 E2E。未启动正式构建、未替换安装目录程序、未修改真实工作区或发布 GitHub；安装 EXE 只读核对仍为 `ed0cc6118dca6382751c3196eda4b9ef73c4dd844a3b302128e10caef3848e00`。构建日志在 `output/completion-release-build.log`。

2026-10-22 本轮按用户进一步说明调整入库入口的实际验证：完整锁定依赖 Rust workspace 404 项通过、2 项真实网络用例忽略；完整 DOM 89 项通过，前端构建通过；样式/可访问性/路由/日期 27 项、边界 5 项和静态检查通过。覆盖单题和比赛选题入库携带所选日期、默认日期可用、未来日期拒绝、远端失败保留日期、防重复提交、重复入库保留原日期，以及单题题面失败和批量日期失败的事务回滚。测试使用临时数据库，未修改真实工作区。日志为 `output/import-completion-*.log`。未构建 release、替换本机程序、发布 GitHub 或执行真实 OJ/Obsidian 人工联动与完整桌面 E2E。

2026-10-22 本轮实际验证：完整锁定依赖 Rust workspace 403 项通过、2 项真实网络用例忽略；完整 DOM 89 项通过；前端构建通过；样式、可访问性、路由与日期 27 项及架构边界 5 项和静态检查通过。新增用例覆盖跨月移动、原入库日不重复计数、同日合并、格式/未来日期拒绝、保存失败重试与防重入、重复导入、重启、旧成员手动补录、软删除历史、独立工作区和 37→38 迁移/结构损坏拒绝；学习与复习事实仍保留原日期。回归发现日期解析器接受非补零格式，补上严格校验后重跑完整 Rust 通过；原月历 DOM 断言同步采用新的“首次做题记录”标签后完整重跑通过。验证日志位于忽略目录 `output/calendar-completion-*.log`。未进行 release 构建、安装替换、真实 OJ/Obsidian 人工联动或完整桌面 E2E，未发布 GitHub。

2026-10-15 参赛记录正式应用到本机这一轮：重新执行 `npm run tauri -- build --no-bundle`，前端与 release 编译通过。备份旧 EXE 并核验 SHA-256 为 `aa76e40c38c43e965e67633de2bf196aaf456471be6b3c98bf2e9f5b0823bcfc`；旧程序正常退出，新安装文件与构建哈希一致，安装目录进程响应正常，UI Automation 读取到“今日 · ACM-OS”WebView 标题。启动后只读检查当前工作区：数据库从 36 升至 37，已生成 `backups/pre-migration/schema-36-to-37-1792068446791694700.sqlite3`；题目 15、比赛 1、题库成员 8、笔记绑定 2、复习尝试 0，与升级前数量一致；参赛记录 0，`PRAGMA quick_check` 为 `ok`。本轮没有再次运行上段开发测试，也未将启动与数量检查称为完整桌面 E2E；未修改 GitHub 附件，未人工创建真实参赛记录。

2026-10-15 现场/VP 参赛记录开发的本次实际验证：完整 Rust workspace tests 401 项通过、两个真实网络用例忽略；最终外部别名去重修正后按 `contest_participation` 定向重跑 3 项通过。前端最终构建、完整 DOM 88 项、样式/可访问性/路由/日期 27 项、边界 5 项与静态检查通过。新增测试覆盖方式/时间/成绩验证、迁移 36→37 且不猜旧历史、重复 VP、同时间防重复与同 ID 重试、修改/删除、跨月、重启、工作区数据独立、外部别名不增加场次、删除比赛保留参赛历史、月历与做题数独立及比赛导航。完整界面回归发现新增参赛面板与选题面板 key 冲突，修复后 88 项全通过。Playwright 在独立示例页核验现场/VP 展示、比赛跳转和录入表单；示例不连接真实工作区。未重新进行 release 编译、安装替换、GitHub 发布、真实 OJ/Obsidian 联动或完整桌面 E2E。日志在忽略目录 `output/participation-*.log`。

2026-10-15 v0.1.1 发布的本次实际验证：`npm ci` 通过，随后以 0.1.1 包版本执行完整 Rust workspace tests 通过（398 项通过、两个真实网络用例忽略）；完整 DOM 86 项、样式/可访问性/路由/日期 27 项、边界 5 项及静态检查通过；npm 全量依赖审计零漏洞。前端构建、Tauri release 编译和 NSIS 打包通过；程序资源版本为 0.1.1，ZIP 中程序与构建 SHA-256 一致。GitHub 已公开为 latest，三个附件的服务器大小与 SHA-256 均匹配本地；旧 v0.1.0 附件也已复核未变。验证日志在忽略目录 `output/release-v0.1.1/`，不是引用历史通过结果。真实 Codeforces 网络、Obsidian 人工联动及完整桌面 E2E 未执行。
2026-10-15 月历做题体验这一轮实际验证：前端 `npm run build` 通过；完整 DOM 86 项通过（扩展月历用例覆盖纯入库显示、同日同题合并、跨日分别计数、题解链接与路由）；Infrastructure 按 `learning_calendar` 筛选的 2 项 Rust 测试通过，覆盖手动题/比赛选题入库、重复入库、本地凌晨日期转换、月份边界、重启、软删除历史、迁移成员排除及原有学习/复习只读查询；边界测试 5 项及静态检查通过；可访问性、视觉样式和日期测试共 20 项通过。首次 workspace 筛选测试因新增测试缺少 trait 导入而编译失败，补齐后按 Infrastructure 范围重跑通过；未重跑完整 Rust workspace、真实桌面 E2E、Obsidian 人工联动或 release 构建，未替换本机安装程序。

以下为 v0.1.0 的历史发布验证：2026-10-13；最新 v0.1.1 验证见上文。

| 检查 | 当次结果 |
|---|---|
| Rust workspace tests，锁定 Cargo 依赖 | 通过；含单题原子写入、去重、重启、月历查询等测试；两个忽略用例未执行 |
| DOM 界面测试 | 86 项通过 |
| 样式、可访问性、路由与日期测试 | 27 项通过 |
| 依赖边界测试与静态检查 | 5 项测试通过，边界检查通过 |
| npm 依赖审计 | 当时锁文件审计为零漏洞；不表示以后永久为零 |
| release 编译、NSIS 打包、附件校验 | 通过；ZIP 内程序与 release 二进制哈希一致，上传附件与本地 SHA-256 一致 |

以上是历史结果，2026-10-14 的记忆文档整理没有重新运行完整产品测试。

2026-10-14 笔记体验开发的本次验证：前端 `npm run build` 通过；完整 DOM 套件 86 项通过，最终公式处理修改后补跑路线弹窗与笔记 Fresh Read 两项均通过；Rust workspace 按 `markdown` 名称筛选的 21 项测试通过（含解析、公式/代码边界、IPC 序列化、笔记持久化相关用例）；边界测试 5 项及静态检查通过；可访问性与视觉样式测试 19 项通过。此次未运行完整 Rust 套件、真实外部 AI、真实 Obsidian 人工联动或桌面 E2E，也未发布新安装包。

同日随后根据人工反馈，将方法入口改为“题解”下的加粗强调色文字。本轮重新运行前端构建通过，并用 Playwright 在独立示例页核验无边框/透明背景、文字字重与颜色，以及弹窗点击和 Escape 关闭；未重新执行上一轮完整测试套件。

同日笔记结构调整这一轮：前端构建、完整 DOM 86 项、按 `markdown` 筛选的 Rust 22 项、边界 5 项及静态检查均通过。新测试覆盖原文章节顺序、完整题意、YAML 属性隐藏、方法正文隔离、默认折叠和自定义 Prompt 原文复制；Playwright 确认示例页初始只展示题意，其余章节收起。本轮未运行完整 Rust 套件、真实 Obsidian 人工联动或发布构建。

同日将 Prompt 移到创建区域这一轮：前端构建、三个相关 DOM 用例通过，核验创建笔记前可用、复制不创建笔记、已有自定义 Prompt 在上方区域复制；Playwright 确认示例页入口位置。未重新执行完整 DOM/Rust 套件。

同日 Prompt 文案调整这一轮：前端构建通过，Playwright 核验标题及默认模板内容已更新。仅改标题与模板文本，未重复执行 DOM/Rust 套件；AI 实际输出质量仍需使用所选模型检查。

2026-10-14 正式应用到本机这一轮：重新运行完整 Rust workspace tests（两项忽略用例未执行）、完整 DOM 86 项、边界测试 5 项及静态检查、可访问性与视觉样式 19 项均通过；`npm run tauri -- build --no-bundle` release 编译成功。原程序备份与旧程序哈希一致，新程序与构建哈希一致；已从安装目录启动并确认进程响应、WebView 显示今日页标题。本轮未运行真实 Codeforces 网络用例、真实 Obsidian 人工联动或完整桌面 E2E；没有公开发布新安装包。

```powershell
npm ci
npm run check:boundaries
cargo test --workspace --locked --manifest-path src-tauri/Cargo.toml
npm run test:dom-shells
node --test scripts/accessibility-css.test.mjs scripts/visual-consistency.test.mjs scripts/startup-shells.test.mjs scripts/calendar-dates.test.mjs
npm run build
npm run tauri -- build --debug --no-bundle
npm run tauri -- build --bundles nsis
```

项目同时保留 npm 和 pnpm 锁文件。发布前保持二者与 `package.json` 一致，实际验证所用依赖必须与构建一致。

## 7. Windows 开发与发布经验

- DOM 测试的 Vite runner 会监听文件；不要与 native 构建同时运行，已遇到监听正在链接的 DLL 导致 `EBUSY`。先结束 native 测试/构建，再跑 DOM 验证。
- 可执行文件正在运行时可能被锁定。不要强制关闭用户正在填写内容的旧版本；人工验收产物使用清楚的新名称，并核实实际路径。
- 之前的验收程序多在 `src-tauri/target/calendar-acceptance/`，这些是 debug 产物，不是 GitHub 发布的 release 安装包。
- GitHub 下载打包工具时，Tauri 的网络请求未自动采用 Git 的代理设置。必要时仅在当前构建进程中使用机器现有网络代理，不将凭据写进仓库。
- NSIS 解压曾遇到跨盘重命名 `os error 17`。当次解决方法是让临时目录与项目本地工具缓存同盘，并使用临时配置 `{"bundle":{"useLocalToolsDir":true}}` 重试 `tauri bundle`。
- 先完成编译，再打包时可用 `npm run tauri -- bundle --bundles nsis --ci --config <临时配置路径>`；无需为下载失败反复重编所有源码。
- GitHub Release 附件与源码分发分开；临时截图、测试日志、打包工具和验收 EXE 位于忽略目录，不提交到源码。
- 发布时先核对版本、验证和来源提交，再上传附件与校验文件，全部核验后发布。仅更新介绍文档不重建或替换原附件，不移动现有发布标签。

### v0.1.0 附件记录

| 文件 | SHA-256 |
|---|---|
| `ACM-OS_0.1.0_x64-setup.exe` | `8c2d292c49863f8f92d8c9913162487e8ff72bcdd079e009cff67437bb4e6594` |
| `ACM-OS_0.1.0_windows-x64_portable.zip` | `4be0e625558181dc6d69d2193d5ca8afe765b0c79e0f67071eaf1678f69fc8ee` |

### v0.1.1 附件记录

构建来源 `5e881dde9bccef6c639a0916e585eb46586fff1a`；release 程序 SHA-256 为 `23ee3d624c730c29e5bd916c4c7cf47007aa98e786a362a39f9f9dea0529e9fa`。

| 文件 | SHA-256 |
|---|---|
| `ACM-OS_0.1.1_x64-setup.exe` | `a14020c26f7c92d16e1e60c73fd9bba68578d1a0b5ab632eb57ab940c533a7c5` |
| `ACM-OS_0.1.1_windows-x64_portable.zip` | `e05340e2325d1e2452dd9ae3f12e18c1d22f3c1be4a0e30813836808697d4c9d` |
| `SHA256SUMS.txt` | `483ee9f3632f5020f4d5409f2234905858c395826394aeb0d33460fd44164468` |

## 8. 文档导航和更新方式

- [README](README.md)：面向用户的完整项目介绍与使用方式。
- [开发记录](docs/DEVELOPMENT_LOG.md)：已确认的变化、关键取舍与发布经过。
- [v0.1.1 发布说明](docs/RELEASE_NOTES_v0.1.1.md)：当前公开 Release；[v0.1.0 发布说明](docs/RELEASE_NOTES_v0.1.0.md) 保留历史产品说明。
- [架构边界](docs/architecture-boundaries.md)：分层原则；执行检查以当前脚本为准。
- [旧实现审计](docs/ACM_OS_CURRENT_IMPLEMENTATION.md)：2026-09-22 历史快照，保留供追溯；其中单题导入、搜索、月历和未发布等结论已经过时。
- [Vault 契约](docs/R6_VAULT_MANAGER_CONTRACT.md)、[个人题库实现报告](docs/CONTEST_PROBLEM_LIBRARY_IMPLEMENTATION_REPORT.md)：对应模块的历史契约和实现背景，需要结合当前源码阅读。

维护时更新本文日期、受影响功能、源码入口、限制及本次实际验证范围；重要变化追加开发记录。将事实、用户取舍、历史结果和待核验事项分别写清楚，不把建议改成完成状态。
