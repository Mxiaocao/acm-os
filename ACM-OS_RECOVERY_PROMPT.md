# ACM-OS Durable Recovery Prompt – M10 Release Candidate

## 最新窗口 prompt（优先于下方旧 RC 快照）

将下面内容复制到下一个 Codex Local 窗口：

```text
请接管 E:\项目开发\acm-os 的后续 BUILD 工作。

不要依赖旧聊天记忆。先执行并报告：

Get-Location
git status --short
git branch --show-current
git rev-parse HEAD
git log -10 --oneline --decorate
git remote -v
git tag --list
git tag --points-at HEAD
git diff --check

然后完整读取：
1. ACM-OS_SPEC_v1.md
2. ACM-OS_DESIGN_v1.md
3. ACM-OS_PLAN_v1.md
4. ACM-OS_BUILD_HANDOFF.md
5. ACM-OS_RECOVERY_PROMPT.md

权威顺序：SPEC > DESIGN > PLAN > IMPLEMENTATION。

当前已知实现 checkpoint（交接文档提交位于其后，真实 HEAD 以命令输出为准）：
- Branch: main
- Implementation commit: 93e320b ui: localize interface and refine responsive layout
- 该提交已完成 ACM-OS 操作界面中文化、俄文题目标题内置英文映射、locale=en 题面抓取、比赛导入页 UI 网格化和全屏统一内容轨道。
- 已验证：build PASS；DOM 36/36；visual-consistency 7/7；accessibility 4/4；定向 Rust URL 测试 PASS。

必须保留且不得提交的未知文件：
- acm-os.exe
- Uninstall ACM-OS.lnk

保护规则：
- 不得 git reset、git clean 或覆盖式 checkout。
- 不得删除或覆盖现有未提交修改和未知文件。
- 未经用户明确授权不得 commit、amend、tag 或 push。
- 不得修改冻结 SPEC、DESIGN、PLAN。
- 不得创建额外 M9/M10 handoff 文件；只更新 ACM-OS_BUILD_HANDOFF.md 和 ACM-OS_RECOVERY_PROMPT.md。
- 不得伪造 OJ 提交、Review 结果或人工 PASS。
- 除非用户明确要求，不进入新功能阶段。

恢复核对通过后，先报告真实状态和与本 prompt 的差异，再等待用户下一步指令。
```

将下面代码块中的全部内容复制到下一个 Codex Local 窗口。

```text
请接管 E:\项目开发\acm-os 的后续 BUILD 工作。

不要依赖旧聊天记忆，也不要把本提示中的快照当作事实。先以真实 Windows 仓库、Git 历史、冻结文档、现有 diff、产物和实际命令输出恢复上下文。

第一步只做恢复核对，不修改文件。执行并报告：

Get-Location
git status --short
git branch --show-current
git rev-parse HEAD
git log -10 --oneline --decorate
git remote -v
git tag --list
git tag --points-at HEAD
git rev-parse origin/main
git rev-list --left-right --count origin/main...HEAD
git diff --stat
git diff
git diff --check
git diff --cached --stat
git diff --cached --check

然后从头到尾完整读取：

1. ACM-OS_SPEC_v1.md
2. ACM-OS_DESIGN_v1.md
3. ACM-OS_PLAN_v1.md
4. ACM-OS_BUILD_HANDOFF.md
5. ACM-OS_RECOVERY_PROMPT.md

权威顺序严格为：

SPEC > DESIGN > PLAN > IMPLEMENTATION

保护规则：

- 不得执行 git reset --hard、git clean 或覆盖式 checkout。
- 不得丢弃任何未提交修改或未知文件。
- 未经用户明确允许，不得 commit、amend、tag 或 push。
- 不得修改冻结的 SPEC、DESIGN、PLAN，除非用户明确授权。
- 正式维护的交接文件只有 ACM-OS_BUILD_HANDOFF.md 和 ACM-OS_RECOVERY_PROMPT.md；不得创建额外 M9/M10 handoff。
- 不得重复 M0-M9。
- 不得在人工 Blocking E2E 全部通过前声称 Technical MVP Accepted。
- 不得自行进入新功能阶段。
- 若实现与冻结文档真实冲突，报告 SPEC-CONFLICT 并停止扩展。

交接时观测到的 Git checkpoint 仅供差异核对：

- repository: E:\项目开发\acm-os
- branch: main
- HEAD: 97f57e589e4b019278d91fe321ca8fa543a6faf8
- subject: build: complete M9 hardening
- origin/main: d739ea5 Add M8 recovery handoff
- M10 修改尚未 commit、tag 或 push
- 预期未提交文件：ACM-OS_BUILD_HANDOFF.md、ACM-OS_RECOVERY_PROMPT.md、package.json、scripts/desktop-e2e.mjs、src-tauri/tauri.conf.json，以及未知文件 scripts/release-desktop-benchmark.mjs
- 本机 MSI 安装器另外生成了未知文件 `acm-os.exe` 与 `Uninstall ACM-OS.lnk`；必须保留，不得清理或纳入提交。

实际状态与以上任何一项不同都不代表可以覆盖或清理；先报告差异并保留现场。

当前里程碑状态：

- M0-M9 完成，不得重复；M9 人工验收已由用户判定合格。
- M10-0 readiness audit：完成。
- M10-A automated RC gates：完成。
- M10-B Windows Release packaging、Desktop E2E、隔离 Release 性能：完成。
- M10-C 真实 Codeforces smoke、本机 Obsidian 协议分派：完成。
- M10-D RC 证据与正式交接收口：完成。

后续人工验收发现并修复了 Review `Open original OJ` 在 WebView2 中无响应的问题：现在通过
HTTPS-only Tauri opener IPC 打开 Codeforces，并在失败时显示错误；修复版 Release/MSI 已重新构建、安装并由用户确认可打开原 OJ。

当前准确结论：M10 自动化工程准备已闭环，Release Candidate 已准备好进行最终人工 Blocking E2E；Technical MVP Accepted 尚未成立。

关键产物快照：

- src-tauri\target\release\acm-os.exe
  - 22,816,256 bytes
  - SHA-256 217038E100144B6C37D829BEA8E3F1F8083CD314107B1B4E49C856417D0385DA
- src-tauri\target\release\bundle\msi\ACM-OS_0.1.0_x64_en-US.msi
  - 8,499,200 bytes
  - SHA-256 0B38AE6BC96E2224B91531CEC752DE43C7306444EB9033C01E3605FF37BB8F5B

产物如被重建，必须重新计算哈希，不能继续引用旧值。

最新自动化证据：

- Release Desktop E2E PASS。
- 7 次隔离 Release 基准：Today 可交互启动 P95 1705.27 ms <= 2500 ms；进程树稳态 RAM P95 373.09 MiB <= 500 MiB。
- test:release-network 两项真实 Codeforces smoke PASS。
- 完整 Rust workspace 含 ignored 联网项：261 passed / 0 failed / 0 ignored；Infrastructure 190 passed。
- boundaries 5/5、Vite/TypeScript build、cargo check、rustfmt、git diff --check 均 PASS。
- OJ opener 修复后 DOM `35/35`、cargo check、rustfmt、Vite build 和 Release/MSI bundling 均 PASS。
- Obsidian 1.12.7 的 obsidian://open Windows 协议分派 PASS，但尚不能替代从 ACM-OS 打开正确 Markdown 的人工验证。

注意：公共 Codeforces 曾在并行测试时瞬时返回 Unavailable；正式 test:release-network 使用 --test-threads=1 并通过。若恢复后联网测试失败，先区分外部服务波动与产品回归。

已知非阻塞 warning：Vite chunk 约 545.51 kB；Tauri identifier dev.acmos.app 以 .app 结尾的跨平台提示。不要在 RC 收口中顺手改 identifier。

下一步不是继续编码新功能。只在用户指令下执行最终人工验收或修复人工验收发现的 RC 阻塞缺陷。

最终人工门禁必须包括：

1. MSI 安装、安装版启动、卸载。
2. Release UI 导入真实 Codeforces Contest。
3. 从 ACM-OS 打开真实 Personal Note/Knowledge，并确认 Obsidian 定位到正确 Markdown。
4. 冻结 DESIGN 第 17 节的完整真实 15-step multi-day Blocking E2E：真实 Contest、Personal Markdown、Obsidian 外部编辑、冷启动 due Review、隐藏旧知识、打开原 OJ 并真实提交、确认 facts、自动判定、生命周期转换、Attempt 历史保留、后续 Today 再召回。

不得用 mock、final_ac=true、仅协议分派或仅联网单测替代真实 15-step 链。

用户随后明确跳过真实 OJ 提交、Review facts/生命周期闭环和卸载验收，并要求停止测试。恢复时必须如实保留该差异：人工验收是 qualified with explicitly skipped blocking steps，不能宣称 Technical MVP Accepted；除非用户重新授权，不要伪造提交结果或补写 PASS。

恢复核对完成后，只报告实际状态、与交接快照的差异、当前缺少的人工证据，以及是否可以开始人工 Blocking E2E；等待用户明确指令。不要自动 commit、tag 或 push。
```

只有这两份正式交接文件被有意提交并推送后，远端 Git 才能承担恢复作用；当前不得自行执行这些操作。
