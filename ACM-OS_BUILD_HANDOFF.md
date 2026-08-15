# ACM-OS BUILD Handoff — M10 Release Candidate

Updated: 2026-08-15 (Asia/Shanghai)

## 当前权威 checkpoint（2026-08-15 UI follow-up）

本节覆盖旧的 Git 快照记录，恢复时以真实仓库命令输出为准：

- Branch: `main`
- HEAD: `93e320b`
- Subject: `ui: localize interface and refine responsive layout`
- 本次提交已包含 7 个实现/测试文件；未包含 `acm-os.exe` 和 `Uninstall ACM-OS.lnk`。
- 未经用户再次授权，不得 amend、tag 或 push。

本窗口完成的实现范围：

- ACM-OS 操作界面主要英文文案已切换为中文。
- Codeforces 缓存中的俄文题目标题使用内置英文映射；未知俄文标题显示安全的 `Problem X`。
- A-F 题面保留本地英文快照；后续 Codeforces 题面抓取明确请求 `locale=en`。
- 比赛导入页的公开导入、手动导入和题目编辑区完成网格化 UI 调整。
- 普通页面统一使用居中的宽屏内容轨道，解决全屏后的左右边界不一致和右侧过大留白。
- 删除了界面中的中文速览功能；原始英文题面和 KaTeX 本地渲染保留。

本窗口验证：

- `npm.cmd run build`: PASS
- `npm.cmd run test:dom-shells`: 36/36 PASS
- `npm.cmd run test:visual-consistency`: 7/7 PASS
- `npm.cmd run test:accessibility`: 4/4 PASS
- Rust `real_adapter_url_construction_is_identity_bound_and_assets_stay_codeforces_only`: PASS
- `git diff --check`: PASS（仅 CRLF 转换提示）

当前未提交内容只应为用户要求保留的安装器生成文件：`acm-os.exe`、`Uninstall ACM-OS.lnk`。如实际状态不同，先报告差异，不要清理或覆盖。

本文档是后续 BUILD 窗口的正式交接文件。它不替代 Git、冻结文档、实际产物或命令输出；恢复时必须先核对真实仓库状态。

## 0. 权威顺序与保护边界

```text
SPEC > DESIGN > PLAN > IMPLEMENTATION
```

开始任何后续工作前，完整读取：

1. `ACM-OS_SPEC_v1.md`
2. `ACM-OS_DESIGN_v1.md`
3. `ACM-OS_PLAN_v1.md`
4. `ACM-OS_BUILD_HANDOFF.md`
5. `ACM-OS_RECOVERY_PROMPT.md`

冻结的 SPEC、DESIGN、PLAN 未被 M10 修改，未经用户明确授权不得修改。不得执行 reset、clean、覆盖式 checkout；不得丢弃未提交修改或未知文件；未经明确授权不得 commit、amend、tag 或 push。

## 1. 当前里程碑状态

- M0–M9：完成，不得重复。
- M9 人工验收：用户已判定合格。
- M10-0：完成，RC 范围、冻结合同和阻塞门禁已核对。
- M10-A：完成，自动化 RC 门禁已通过。
- M10-B：完成，Windows Release 产物、真实桌面 E2E 和隔离 Release 性能证据已建立。
- M10-C：完成，真实 Codeforces 联网 smoke 与本机 Obsidian 协议分派已验证。
- M10-D：完成，RC 证据与正式恢复交接已收口。

随后人工验收发现 Release 安装版 Review 中的 `Open original OJ` 裸 `target="_blank"` 在 WebView2
中无响应，已补为 HTTPS-only Tauri opener IPC，并增加错误反馈与 DOM 回归覆盖。修复后的 Release/MSI
已重新构建并升级安装；用户确认安装版现在可以打开原 OJ。

M10 的自动化工程准备已经闭环，但冻结 PLAN/DESIGN 要求的真实 15-step Blocking E2E 尚需用户人工执行。因此当前状态是：

```text
Release Candidate ready for final human Blocking E2E
Technical MVP Accepted: NOT YET
```

不得在人工 Blocking E2E 全部 PASS 前宣称 `Technical MVP Accepted`，也不得自行创建建议 tag `acm-os-mvp-rc1`。

## 2. 恢复时必须核对的 Git 快照

以下只记录本窗口观测值，不得当作恢复后的事实：

- Repository: `E:\项目开发\acm-os`
- Branch: `main`
- HEAD: `97f57e589e4b019278d91fe321ca8fa543a6faf8`
- Subject: `build: complete M9 hardening`
- `origin/main`: `d739ea5 Add M8 recovery handoff`
- M10 修改尚未提交、未打 tag、未 push。

本次交接完成后的预期未提交文件为：

```text
 M ACM-OS_BUILD_HANDOFF.md
 M ACM-OS_RECOVERY_PROMPT.md
 M package.json
 M scripts/desktop-e2e.mjs
 M src-tauri/tauri.conf.json
?? scripts/release-desktop-benchmark.mjs
?? acm-os.exe
?? Uninstall ACM-OS.lnk
```

`acm-os.exe` 与 `Uninstall ACM-OS.lnk` 是本机 MSI 安装器在仓库根目录生成的未知文件；它们已保留但不纳入本次提交，不得用 `git clean` 删除。

恢复时必须重新运行 `git status --short`、branch、HEAD、log、remote、diff 和 tag 核对；若实际状态不同，以实际状态为准并先报告差异。

## 3. M10 实现变更

### Release 桌面基准

新增 `scripts/release-desktop-benchmark.mjs` 与 `npm run benchmark:release-desktop`。它启动真实 Release Tauri 可执行文件，使用隔离的应用数据、测试时钟和 WebView2 用户目录，采集 Today 可交互启动时间与进程树稳态 RAM，并执行 7 次样本的 P95 预算判断。

该证据是隔离环境中的真实 Release 进程测量；它不同于 M9 的确定性 Node Reference Dataset benchmark，也不等同于任意用户机器上的首次安装冷启动。

### Desktop E2E 稳定性

`scripts/desktop-e2e.mjs` 为每轮使用隔离的 WebView2 用户目录，并在 Windows 上可靠清理完整进程树，避免 WebView2 状态和残留子进程污染后续测试。

### Release packaging

`src-tauri/tauri.conf.json` 显式指定 `icons/icon.ico`，Windows MSI bundling 已成功完成。

### 真实联网门禁

新增 `npm run test:release-network`。它串行运行两项 ignored Codeforces 测试，避免公共服务并发请求造成不必要波动：

1. 大 standings 有界元数据与真实 2256C 题面/资源。
2. Contest 1 真实导入与幂等重试。

公共 Codeforces 是外部依赖；曾有一次并行总门禁遇到瞬时 `Unavailable`，随后正式单线程门禁与完整 workspace 均通过。恢复时若失败，必须区分产品回归与外部服务波动，不得静默忽略。

## 4. Release 产物证据

Windows Release EXE：

```text
src-tauri\target\release\acm-os.exe
size: 22,816,256 bytes
SHA-256: 217038E100144B6C37D829BEA8E3F1F8083CD314107B1B4E49C856417D0385DA
```

Windows MSI：

```text
src-tauri\target\release\bundle\msi\ACM-OS_0.1.0_x64_en-US.msi
size: 8,499,200 bytes
SHA-256: 0B38AE6BC96E2224B91531CEC752DE43C7306444EB9033C01E3605FF37BB8F5B
```

这些哈希只对应本窗口生成的产物。任何重建都必须重新计算并报告新哈希。

## 5. M10 自动化证据

- Release Desktop E2E：PASS。
- Release 桌面隔离基准，7 次：
  - Today 可交互启动 P95 `1705.27 ms`，预算 `<= 2500 ms`。
  - 进程树稳态 RAM P95 `373.09 MiB`，预算 `<= 500 MiB`。
- `npm run test:release-network`：2 项真实 Codeforces smoke PASS。
- 完整 Rust workspace（含 ignored 联网项）：`261 passed / 0 failed / 0 ignored`。
- Infrastructure：`190 passed`。
- `npm run check:boundaries`：`5 passed`，boundary check passed。
- `npm run build`：PASS。
- `cargo check --workspace`：PASS。
- `cargo fmt --all -- --check`：PASS。
- `git diff --check`：PASS。

OJ opener 修复后的追加证据：

- `npm run test:dom-shells`：`35 passed / 0 failed`。
- `cargo check --workspace`：PASS。
- `cargo fmt --all -- --check`：PASS。
- `npm run build`：PASS。
- Release/MSI bundling：PASS，WiX/NSIS 均生成。

本机 Obsidian 证据：

```text
D:\software\Obsidian\Obsidian.exe
version: 1.12.7
obsidian://open protocol dispatch: PASS
handler: "D:\software\Obsidian\Obsidian.exe" "%1"
```

协议分派 PASS 只证明 Windows 能把 `obsidian://open` 交给 Obsidian；最终仍需人工确认 ACM-OS 打开的 Personal Note/Knowledge 是正确 Markdown 文件。

## 6. 已知非阻塞 warning

- Vite 生产构建报告单个 chunk 约 `545.51 kB`；当前构建通过，未超过冻结功能或性能门禁，但后续可作为优化项。
- Tauri 对 identifier `dev.acmos.app` 以 `.app` 结尾给出跨平台提示；Windows RC packaging 不受阻塞。若未来调整 identifier，必须按迁移和兼容性变更处理，不能在 RC 收口中顺手修改。

## 7. 最终人工 Blocking E2E

冻结 DESIGN 第 17 节与 PLAN 第 9.7/10 节要求 Release Candidate 至少真实完成一次 15-step Blocking E2E：

1. 导入真实受支持的公开 Contest。
2. 全部题目成为 lightweight Problems。
3. 选择一题并创建 Personal Markdown。
4. 加入 upsolve 并开始学习。
5. 在 Obsidian 外部编辑 Markdown。
6. ACM-OS 读取最新内容。
7. 标记题目已理解。
8. 跨日期并冷启动到 due Review。
9. 进入 Review，旧知识保持隐藏。
10. 打开原 OJ 并进行真实提交。
11. 确认 Review facts。
12. 系统自动判断真会/半会/未通过。
13. 转入长期复习或 relearn。
14. 已完成 Attempt 历史保持不变。
15. 后续 Today Plan 再次召回该题。

不得用 mock、`final_ac=true`、仅协议分派或仅联网单测替代这条真实链。另需人工执行 MSI 安装、从安装版启动和卸载检查。

本次用户明确跳过了真实 OJ 提交及后续 Review facts/生命周期闭环，也未继续执行卸载验收。已完成的人工证据包括安装、启动、真实 Contest 导入、Personal Markdown、Obsidian 外部编辑、Fresh Read、补题、开始学习、补懂、重启持久化、Review 隔离和修复后的 OJ 打开；因此只能记录为：

```text
M10 engineering gates: COMPLETE
Manual RC acceptance: QUALIFIED WITH EXPLICITLY SKIPPED BLOCKING STEPS
Technical MVP Accepted: NOT CLAIMED
```

## 8. 下一步边界

下一步只能是用户主导的最终人工 Blocking E2E 与 MSI 验收，或用户明确指定的问题修复。若人工验收发现缺陷：先记录复现步骤和证据，只修阻塞 RC 的问题，完整回归后更新本文件与 `ACM-OS_RECOVERY_PROMPT.md`。

在人工门禁通过前，不进入新功能里程碑，不 tag，不 push，不宣称 Technical MVP Accepted。
