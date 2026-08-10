# ACM-OS durable recovery prompt

将下面代码块中的全部内容复制到新的 Codex Local 工作窗口。

```text
你现在接管 ACM-OS 的 BUILD 工作。不要依赖任何旧聊天记忆，也不要默认交接文档仍然最新；所有结论必须以真实 Windows 本地仓库、Git 历史、冻结文档和实际命令结果为准。

真实项目目录应为：

E:\项目开发\acm-os

第一阶段只读取证，修改任何文件前必须运行并报告：

Get-Location
git status
git branch --show-current
git log -10 --oneline --decorate
git remote -v
git tag --points-at HEAD

预期的已知安全 checkpoint（仅用于比对，不可代替实际检查）：

- branch: main
- M1 commit: 4e253590fd0eee8d5d7af61bb14529bff4cd6e6b
- commit subject: build: complete M1 contest import
- M2.1 commit: 27f785f build: complete M2.1 personal note binding
- tag: acm-os-m1-contest-import
- origin: https://github.com/Mxiaocao/acm-os.git
- 2026-08-10 已确认远程 main 与远程 M1 tag 均指向该 commit
- M1、M2.1、M2.2 已完成；M2.2 变更当前尚待 commit
- M2.1 目前仅在本地 main，origin/main 仍为 M1

如果路径不正确、出现 acm-os\acm-os 嵌套、当前目录属于其他项目、存在未知用户文件、分支或 HEAD 不符、工作树有无法解释的改动，立即停止并报告。不要覆盖、merge、reset、clean 或 checkout 丢弃任何内容。

完整阅读以下文件，不能只搜索关键词或依赖摘要：

1. ACM-OS_SPEC_v1.md
2. ACM-OS_DESIGN_v1.md
3. ACM-OS_PLAN_v1.md
4. ACM-OS_BUILD_HANDOFF.md（如果存在）

权威顺序严格为：

SPEC > DESIGN > PLAN > IMPLEMENTATION

SPEC 是唯一产品事实来源。冻结的产品行为、Authority、状态机、Review、Today、Markdown、事务边界、测试策略和 Milestone 顺序都不能重新设计。发现真实冲突时标记 SPEC-CONFLICT 并停止。

若 ACM-OS_BUILD_HANDOFF.md 丢失或明显过时，不要卡住，也不要猜测。使用以下证据恢复上下文：

git show --stat 4e25359
git show --stat 27f785f
git show acm-os-m1-contest-import:ACM-OS_BUILD_HANDOFF.md
git diff
git status

当前下一切片是：

M2.3 — Binding Resolution + Vault Availability

冻结 Outcome：Lightweight Problem 创建真实 Personal Markdown；用户在外部 Obsidian 修改后，ACM-OS 必须 Fresh Read 最新内容。

M2 范围包括：

- create personal note
- initial Markdown skeleton
- File Binding Registry
- Windows file key
- digest / relocation
- Fresh Read / parser
- watcher（只做 cache invalidation / re-read trigger，不是事实源）
- window-focus revalidation
- Open in Obsidian
- Safe Patch engine
- Recovery Copy foundation

主要验收为 AC-PROBLEM-01、AC-MD-01~06；Candidate relation 的实际行为留到 M6。M2 blocking evidence 必须证明：即使已有 stale cache，外部修改 Markdown 且 watcher event 没发生，下一次读取仍通过 Fresh Read 得到最新内容。

必须保护以下边界：

- Markdown 正文事实由 Markdown 权威拥有，SQLite 不覆盖正文；
- 不向 Markdown 注入 ACM-OS 私有 ID；
- Problem Identity 与 File Binding 分离；
- React 不获得 filesystem 或 SQLite Authority；
- watcher 不是事实源；
- authoritative read 必须 Fresh Read；
- write 必须 fresh read、唯一目标验证、minimal/byte-preserving patch、digest 并发检查、pre-write recovery copy、写后重读/重解析/语义验证；
- 外部编辑优先，stale cache 不得覆盖新内容；
- Vault 暂时不可用只阻塞受影响 scope，不自动等于全局 Recovery；
- 文件或网络 I/O 不进入长 SQLite transaction。

本轮先不要直接写代码。先检查：

- 顶层目录与 Git 状态；
- package/Cargo manifests；
- package-lock.json 与 src-tauri/Cargo.lock；
- 当前 Node/npm/Rust/Cargo/Tauri/Windows 工具链；
- M1 与 M2.1 commit 的实际 change surface；
- 当前 Application/Infrastructure/Tauri/React Authority 边界；
- 与 M2 直接相关的现有 schema、ports、tests、UI 入口。

然后向我提交一份严格来源于 SPEC/DESIGN/PLAN 的 M2.3 最小切片计划与验收矩阵。计划必须列出 Outcome、Why、Dependencies、Change Surface、Implementation Steps、Focused Tests、Broader Verification、Done Evidence、Rollback，并明确哪些后续 M2 与 M3+ 能力被排除。M2.3 优先处理 path → Windows file key → digest 的绑定解析、歧义处理与 Vault unavailable affected-scope 状态；不要提前实现 watcher、Safe Patch 或 M3 lifecycle。得到我确认后再实施。

执行原则：

最小 Slice → 实现 → focused verification → broader verification → diff review → status

当前 Slice 有失败时，不叠加下一 Slice。不要伪造测试、编译、Windows exe 或启动 PASS。系统级安装前先询问。不得手写 lockfile。

未经我明确允许，不要 commit、tag、push 或创建远程仓库。M2 全部通过以前，不进入 M3，也不要创建 acm-os-m2-vault-binding。

如果本地仓库不存在或损坏，先报告。远程恢复来源为 https://github.com/Mxiaocao/acm-os.git；只允许在我确认的空目录中 clone，禁止覆盖现有目录或未知用户文件。

你的第一条正式状态汇报必须包含：实际路径、branch、HEAD、working tree、M1 tag/remote 证据、四份文档读取状态、M2 冻结范围、发现的风险或阻塞，以及建议的第一个最小 Slice。不要在这份汇报前修改代码。
```

## 仓库之外的备份建议

为了避免提示词与仓库一起丢失，至少再保存一份到下列任一位置：

- 个人云盘中的纯文本文件；
- 密码管理器的安全备注；
- 私人笔记系统；
- 新 Codex 任务的首条消息。

仓库内的本文件只有在被 commit 并 push 后才具备 Git 远程恢复能力。是否提交这两份文档必须由用户明确决定。
