# ACM-OS 项目记忆

更新日期：2026-10-14，时区 Asia/Shanghai。本文是接续开发的入口；源码改变后应更新对应条目。

## 1. 当前状态

- 项目：面向算法竞赛和日常刷题的 Windows 桌面学习工作台。
- 核心流程：比赛 → 赛后补题 → 个人题解 → 冷启动复习 → 长期复习；通过 Obsidian 管理 Markdown 题解与知识笔记。
- 仓库：[Mxiaocao/acm-os](https://github.com/Mxiaocao/acm-os)，默认分支 `main`。
- 已发布：[v0.1.0](https://github.com/Mxiaocao/acm-os/releases/tag/v0.1.0)，Windows x64 安装包、免安装 ZIP、SHA-256 校验文件。
- 发布二进制对应 `27725d16506d9ea6d51262eae3fb214997e6879d`；发布后的完整项目介绍提交为 `712a9c6`。后续文档提交不改变已发布二进制和标签。
- 安装包尚未进行 Windows 代码签名；当前没有产品自动更新插件。
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

## 3. 功能与源码定位

| 模块 | 当前能力 | 主要入口 |
|---|---|---|
| 启动与工作区 | 恢复、配置、正常模式；多个 Vault 的注册、切换与隔离 | [App.tsx](src/app/App.tsx)、[VaultManagerPanel.tsx](src/app/VaultManagerPanel.tsx)、[vault_ipc.rs](src-tauri/src/vault_ipc.rs)、[vault_manager.rs](src-tauri/crates/acm-os-infrastructure/src/vault_manager.rs) |
| 比赛 | Codeforces 自动/手动导入、赛后事实、分类/系列/年份、归档、删除、响应式书架 | [shells.tsx](src/app/shells.tsx) 中 `ContestLibraryPage` / `ContestCabinet` / `ContestDetail`、[contest.ts](src/ipc/contest.ts) |
| 个人题库 | 比赛选题入库、独立单题导入、搜索、预览、日期、基础编辑与软删除 | [ProblemLibrary.tsx](src/app/ProblemLibrary.tsx)、[SingleProblemImport.tsx](src/app/SingleProblemImport.tsx)、[ProblemActions.tsx](src/app/ProblemActions.tsx) |
| 单题后端 | 输入验证、强身份去重、原子入库、首次题面快照 | [应用用例](src-tauri/crates/acm-os-application/src/single_problem.rs)、[持久化](src-tauri/crates/acm-os-infrastructure/src/persistence/single_problem.rs)、[IPC 包装](src/ipc/single-problem.ts) |
| 学习与复习 | 补题、学习、首次冷启动、提前检查、长期复习、帮助记录、失败原因与历史 | [Domain](src-tauri/crates/acm-os-domain/src/lib.rs)、[Application](src-tauri/crates/acm-os-application/src/lib.rs)、`shells.tsx` 中 `ReviewFocusShell` |
| 今日 | 时间预算、任务重排、完成、建议、预览重新规划；展示真实摘要 | `shells.tsx` 的今日页面、[TodayOverview.tsx](src/app/TodayOverview.tsx) |
| 学习月历 | 历史学习与复习、当前下一次到期安排、选中日期详情 | [LearningCalendar.tsx](src/app/LearningCalendar.tsx)、[日期计算](src/app/calendar-dates.ts)、[月历持久化](src-tauri/crates/acm-os-infrastructure/src/persistence/learning_calendar.rs) |
| 知识与 Obsidian | Markdown 发现、搜索、理解等级、关联解析、打开笔记与外部图谱、绑定异常修复 | `shells.tsx` 中 `KnowledgePage`、[knowledge_index.rs](src-tauri/crates/acm-os-infrastructure/src/knowledge_index.rs)、[knowledge.ts](src/ipc/knowledge.ts) |
| 奖励 | 启用模式、XP/金币/等级、合格复习奖励、自定义兑换与退款记录 | `shells.tsx` 中 `RewardPage`、[reward.ts](src/ipc/reward.ts)、Domain 与持久化中的奖励策略 |
| 外观 | 樱花粉样式、月历布局、书架与图标素材 | [app.css](src/app/app.css)、[sakura-theme.css](src/app/sakura-theme.css)、[learning-calendar.css](src/app/learning-calendar.css)、[图标配置](src-tauri/tauri.conf.json) |

路由定义见 [routing.ts](src/app/routing.ts)：`/today`、`/calendar`、`/contests`、`/problems`、`/knowledge`、`/reward`、`/settings`。题目详情同时支持外部题号路由与内部 ID 路由；专注复习使用 `/review/:attemptId`。

## 4. 数据和业务规则

- React 通过类型化 IPC 调用后端，不持有通用 SQL 或文件系统写权限。Domain 处理纯规则，Application 编排用例，Infrastructure 负责数据库、文件和网络实现，Tauri 负责组合与 DTO 边界。
- SQLite 保存身份、学习状态、计划、复习历史与奖励等系统事实；Vault 内 Markdown 保存个人题解与知识内容。数据库备份不覆盖 Markdown 文件。
- 当前数据库结构版本为 36。变更结构要走现有迁移与结构校验，不能直接改用户数据库绕过契约。
- `problems.id` 是稳定内部身份。修改显示标题或来源 URL 不改变外部身份，也不自动重新抓取题面。
- 比赛题目与个人题库成员分开；比赛导入不会默认把所有题目加入个人题库。
- 独立单题导入使用已有表，不创建空比赛、placement 或比赛题目关系。无 URL 的手动题目使用内部 `acmos://local-problem/...` 地址，界面不应提供该地址的“原题网站”链接。
- 去重依赖平台与外部身份，不按相似标题合并。重复导入保持首次成功题面及原始元数据，并恢复对应题库可见性。
- 导入日期来自 `problems.created_at_utc`，按本地日期显示；不是“本次打开日期”，也不是再次加入题库的时间。前端没有日期数据时显示未记录。
- 题库搜索使用题面提取的文本；展示只插入安全文本与 KaTeX 生成的公式，不能直接把来源 HTML 放进列表。
- 题目软删除保留历史及笔记；删除题库题目与删除个人题解是不同操作。当前没有通用回收站 UI。
- 月历查询只读，不生成新的今日计划。历史来源为计划完成、复习周期创建和已完成复习；未来展示每题当前下一次复习，逾期汇总到今天。不能将其描述为完整未来复习模拟或原始 OJ 做题日志。
- 学习状态包括待学、待补题、学习中、等待冷启动、重学、长期复习。复习依据记录的事实判定掌握程度，不代表软件自动编译提交并验证 OJ AC。
- Vault 切换使用后端上下文和 epoch 隔离；业务调用沿用现有 IPC 包装，不能绕过旧上下文拒绝机制。

## 5. 已知范围和未确认事项

- 自动题面导入围绕 Codeforces；其他来源走手动录入，不能宣称已实现通用多 OJ 抓取。
- 题目没有持久化算法标签字段；列表不得伪造标签或缺失的评分。
- 知识图谱按钮打开 Obsidian 原生图谱，需要启用 Obsidian CLI；本应用没有内置自由图谱画布。
- 赛后分析是粘贴文本及固定结构解析，没有接入自动生成式 AI 服务。
- 奖励模式与复习奖励已接入；“学会题目奖励”虽有后端策略，但不能据此承诺标记学会自动发奖。完整逐条账本展示也未完成。
- 外部题号路由限制为 `[A-Z][0-9]?`，领域允许范围更宽；长题号是需要另行核验的边界。
- 最近发布验证没有重新进行真实 Codeforces 网络用例、真实 Obsidian 联动或全部桌面 E2E。这些外部条件不能写成已完整验收。
- 后续重构、标签、统计、签名、自动更新等只是可能的改进方向，尚无新的任务授权。

## 6. 验证记录与常用命令

最近一次发布验证：2026-10-13。

| 检查 | 当次结果 |
|---|---|
| Rust workspace tests，锁定 Cargo 依赖 | 通过；含单题原子写入、去重、重启、月历查询等测试；两个忽略用例未执行 |
| DOM 界面测试 | 86 项通过 |
| 样式、可访问性、路由与日期测试 | 27 项通过 |
| 依赖边界测试与静态检查 | 5 项测试通过，边界检查通过 |
| npm 依赖审计 | 当时锁文件审计为零漏洞；不表示以后永久为零 |
| release 编译、NSIS 打包、附件校验 | 通过；ZIP 内程序与 release 二进制哈希一致，上传附件与本地 SHA-256 一致 |

以上是历史结果，2026-10-14 的记忆文档整理没有重新运行完整产品测试。

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

## 8. 文档导航和更新方式

- [README](README.md)：面向用户的完整项目介绍与使用方式。
- [开发记录](docs/DEVELOPMENT_LOG.md)：已确认的变化、关键取舍与发布经过。
- [发布说明](docs/RELEASE_NOTES_v0.1.0.md)：当前公开 Release 的产品说明。
- [架构边界](docs/architecture-boundaries.md)：分层原则；执行检查以当前脚本为准。
- [旧实现审计](docs/ACM_OS_CURRENT_IMPLEMENTATION.md)：2026-09-22 历史快照，保留供追溯；其中单题导入、搜索、月历和未发布等结论已经过时。
- [Vault 契约](docs/R6_VAULT_MANAGER_CONTRACT.md)、[个人题库实现报告](docs/CONTEST_PROBLEM_LIBRARY_IMPLEMENTATION_REPORT.md)：对应模块的历史契约和实现背景，需要结合当前源码阅读。

维护时更新本文日期、受影响功能、源码入口、限制及本次实际验证范围；重要变化追加开发记录。将事实、用户取舍、历史结果和待核验事项分别写清楚，不把建议改成完成状态。
