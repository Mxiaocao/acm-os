# ACM-OS BUILD Handoff

> 更新时间：2026-08-09 23:17（Asia/Shanghai）
>
> 用途：供切换账号后的 Codex 在同一 Windows 本地仓库继续完成 B0.4。
>
> 注意：本文件是历史交接记录，不是验证通过证明；接手者必须以当前文件和实际命令结果为准。

## 1. Authority

权威顺序始终为：

`SPEC > DESIGN > PLAN > IMPLEMENTATION`

接手后必须完整阅读：

- `ACM-OS_SPEC_v1.md` — 唯一产品事实来源；
- `ACM-OS_DESIGN_v1.md` — 已冻结设计与架构边界；
- `ACM-OS_PLAN_v1.md` — 已冻结实施顺序与验收策略；
- `ACM-OS_BUILD_HANDOFF.md` — 仅作为交接上下文。

不得重新设计或静默修改已经冻结的产品行为、Authority、状态机、Review、Today、Markdown、事务边界、测试策略或 Milestone 顺序。若发现真实冲突，标记 `SPEC-CONFLICT` 并停止。

## 2. Repository snapshot

真实项目目录：

`E:\项目开发\acm-os`

Git 状态：

```text
Branch: main
HEAD:   f140316 build: complete B0.3 workspace configuration
```

已有完成提交：

```text
f140316 build: complete B0.3 workspace configuration
ece09f8 build: complete B0.2 SQLite startup gate
d00a915 build: complete B0.1 repository scaffold
```

当前 B0.4 工作未提交。没有执行 commit、tag、push、reset、clean，也没有创建远程仓库。

## 3. Current BUILD position

```text
Milestone: M0 — Executable Foundation + Workspace Ready Gate
Slice:     B0.4 — Startup shells
Status:    IMPLEMENTED / REVIEW BLOCKED / NOT DONE
```

B0.4 冻结目标：

```text
Recovery / Setup / Normal / Review layout boundary
```

当前不要进入 B0.5，也不要实现 M1+ 的 Contest、Problem、Knowledge、Review 执行、Today 规划、Vault/Markdown 等业务能力。

## 4. Current working tree

已修改的跟踪文件：

```text
README.md
package.json
src-tauri/crates/acm-os-application/src/lib.rs
src-tauri/src/ipc.rs
src-tauri/src/lib.rs
src/app/App.tsx
src/app/app.css
ACM-OS_BUILD_HANDOFF.md
```

未跟踪文件：

```text
scripts/startup-shells.test.mjs
src/app/routing.ts
src/app/shells.tsx
src/ipc/app-shell.ts
```

重要：普通 `git diff` 不显示未跟踪文件。审阅时必须同时读取上面四个未跟踪文件，不能因为 `git diff` 没显示它们就声称没有变更。

`package-lock.json` 与 `src-tauri/Cargo.lock` 当前均未修改。

## 5. B0.4 implementation present in the working tree

### Application startup decision

`acm-os-application` 新增：

- `StartupDestination::{Recovery, Setup, Normal}`；
- `select_startup_destination`；
- Recovery 优先于 workspace 路由；
- Ready + Unconfigured → Setup；
- Ready + Configured → Normal；
- workspace 查询结果缺失 → fail-closed Recovery(`DatabaseUnavailable`)。

### Thin Tauri IPC

新增异步命令：

`app_shell_status`

调用链为：

```text
React
→ typed app-shell IPC
→ thin Tauri command
→ Application startup/workspace decisions
→ Infrastructure persistence port
```

React 没有获得 SQLite 或 filesystem Authority。

### React shells and routing

已实现：

- Loading shell；
- Recovery shell，隐藏普通导航；
- Setup shell，沿用 B0.3 workspace 配置命令；
- Normal shell；
- `/` 在 Normal 状态规范化为 `/today`；
- 一级导航：Today、Contests、我的题库、Knowledge；
- 工具导航：Settings；
- `/review/:attemptId` 使用独立全屏 Focus shell，隐藏普通导航；
- 未知路由显示 Not Found；
- Today 与其他产品页面在 M0 保持空壳，不包含 B0.5/M1+ 业务逻辑。

### Accessibility mechanical fix from the final review

最终短复审自动把以下正文提升到 `1rem`：

- `.safe-note`；
- `.system-caption`；
- `.field-error`。

该 CSS 小修之后已重新运行 frontend build，但尚未重新生成包含该小修的 Tauri exe。

## 6. Verification evidence obtained for the current source tree

### PASS — Shell route tests

执行：

```text
node --test scripts/startup-shells.test.mjs
```

结果：`4 passed / 0 failed`。

覆盖纯路由解析：Normal 路由、Review 路由、非法/嵌套/畸形 Review 路由、Setup/Recovery URL 不伪装普通页面。

### PASS — Architecture boundary gate

执行等价命令：

```text
node --test scripts/check-boundaries.test.mjs
node scripts/check-boundaries.mjs
```

结果：`5 passed / 0 failed`，并输出 `boundary check passed`。

### PASS — Frontend compilation

由于 `node` 不在 PATH，使用 Codex App 自带 Node 的绝对路径执行 TypeScript 与 Vite：

```text
C:\Users\Mxiaocao\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe
```

结果：

```text
TypeScript: PASS
Vite:       PASS
Modules:    23 transformed
JS bundle:  201.80 kB (63.36 kB gzip)
CSS bundle: 5.23 kB (1.83 kB gzip)
```

### PASS — Rust workspace tests

由于 `cargo` 不在 PATH，使用：

```text
C:\Users\Mxiaocao\.cargo\bin\cargo.exe test --workspace --locked
```

结果共 `35 passed / 0 failed`：

```text
Tauri IPC:       6
Application:     5
Infrastructure: 24
```

### PASS — Rust workspace check

执行：

```text
C:\Users\Mxiaocao\.cargo\bin\cargo.exe check --workspace --locked
```

结果：PASS。

### PASS — Diff whitespace check

执行：

`git diff --check`

结果：PASS，无 whitespace error。

### Earlier PASS — Tauri build and minimal launch

在最终 CSS 字号小修之前，B0.4 源码曾真实完成：

```text
Tauri debug build: PASS
Executable: E:\项目开发\acm-os\src-tauri\target\debug\acm-os.exe
Size: 16,760,320 bytes
Launch: PASS
Observed window title: ACM-OS
Observed process: responding, non-zero window handle
```

该进程在验证后已正常关闭。

这证明当时的 React → typed IPC → Tauri → Application → Domain/Infrastructure 能够真实连接并启动，但不能代替最终修复后的重新构建与启动验证。

## 7. Final short review verdict

最终短复审结论：

`NOT READY TO COMMIT`

安全、性能和 typed IPC 契约专项审阅没有发现阻断问题。独立红队发现并确认了冻结路由契约问题。当前必须处理以下事项。

### BLOCKER 1 — Review route 未落实 UUIDv7 Stable Internal ID

文件：`src/app/routing.ts`

当前 `/review/:attemptId` 接受任意非空且不含 `/` 的字符串，测试还把 `attempt-123` 固化为合法 ID。

冻结依据：

- `ACM-OS_PLAN_v1.md` 4.2：核心对象（含 Review Attempt）使用内部稳定 UUIDv7；
- `ACM-OS_PLAN_v1.md` 8.2：对象 route 使用 Stable Internal ID；
- `/review/:attemptId` 绑定 Attempt ID，而不是标题或路径。

接手修复要求：

- 在路由边界验证规范 UUIDv7；
- 拒绝非 UUID、非 v7、非法 variant、百分号编码别名和带 `/` 的值；
- 把测试中的 `attempt-123` 替换为合法 UUIDv7 fixture；
- 增加拒绝错误 UUID version/variant/encoded alias 的测试；
- 不实现真实 Review 查询或 M4 行为。

### BLOCKER 2 — SPA route transition 缺少焦点迁移与页面播报

文件：`src/app/App.tsx`、`src/app/shells.tsx`

当前 sidebar link、Return to Today、Go to Today 和浏览器 back/forward 只更新 pathname。键盘/屏幕阅读器焦点可能留在旧控件，或控件卸载后落回 body，新页面标题不会被可靠播报。

接手修复要求：

- 为 routed main 或页面主标题建立稳定 focus target；
- pathname/shell 切换后把焦点移到新内容；
- 更新 `document.title` 或提供可靠的 polite route announcement；
- 为持久 sidebar 提供 skip-to-content；
- 验证退出 Review Focus 后焦点落到 Today 内容，而不是已卸载按钮。

### REVIEW GAP 3 — Shell tests 只覆盖 parser

文件：`scripts/startup-shells.test.mjs`

当前测试不能证明 React 实际隐藏/显示正确 shell，也没有覆盖：

- app shell IPC rejection → Recovery；
- Recovery / Setup 不出现普通导航；
- Normal shell 导航；
- Setup 成功后进入 Normal/Today；
- Review Focus 隔离；
- Return to Today；
- push/replace/popstate；
- loading/error rendering。

接手者应添加最小 DOM 级测试。若需要新增测试依赖，必须使用真实依赖解析并让 `package-lock.json` 正常更新，不得手写 lockfile。

### REVIEW GAP 4 — Foundation failure 永久显示 checking

文件：`src/app/App.tsx`、`src/app/shells.tsx`

`getFoundationStatus()` 失败时错误被丢弃，`foundation` 继续保持 `null`，Setup/Normal 永久显示 `checking`。

应区分：

```text
checking / ready / unavailable
```

这是诊断状态修复，不应改变 Startup Gate、Recovery Authority 或业务逻辑。

## 8. Reviewed findings intentionally not treated as blockers

以下审阅建议已分析，但当前不应据此扩大 B0.4：

- 旧 `startup_status` / `workspace_status` IPC 暂不删除。它们来自已经完成的 B0.2/B0.3 合同，删除不是完成 B0.4 的必要条件；
- 五个普通路由的小规模 metadata 重复暂不重构。当前规模有限，重构收益不足以扩大 Slice；
- workspace query failure 映射为 Recovery `database_unavailable` 是当前 fail-closed 策略，Application 的错误面目前只有 `PersistenceUnavailable`。不要把它误判为可以正常继续启动；
- 首次 app-shell invoke 失败后要求重启应用符合当前 B0.4 “无自动修复”的保守边界。若要增加 retry/recheck，必须先确认不会越过冻结设计与当前 Slice。

## 9. Local toolchain caveats

当前 Codex App shell 中：

- `node` 不在 PATH；
- `npm` 不在 PATH；
- `cargo` 不在 PATH；
- Node 与 Cargo 的绝对路径可用；
- `rustfmt` 先前检查为未安装，不要在未经用户允许的情况下做系统级安装。

Tauri 当前配置的 `beforeBuildCommand` 是：

`npm run build`

因此最终短复审中直接调用 Tauri CLI 时，因 `npm` 不在 PATH 而失败。不要把这个环境失败伪装成代码构建失败，也不要用假 npm、假 cargo 或禁用检查来制造 PASS。

接手者应先重新检查自己的账号/Local 环境工具链。若 npm 可用，优先运行原始项目命令；若仍不可用，应使用官方、安全、可审计的最小方案，系统级安装前先征得用户同意。

外部 Codex CLI 的额外模型审阅曾因“可能把未提交源码交给外部目的地”被权限审查拒绝；没有绕过该限制。内部本地专项审阅与独立红队已经完成。

## 10. Exact next actions

接手账号应按以下顺序执行：

1. 确认 `Get-Location` 精确为 `E:\项目开发\acm-os`；
2. 完整阅读四份权威文档；
3. 运行 `git status`、`git branch --show-current`、`git log -3 --oneline`；
4. 读取全部 B0.4 diff，并单独读取四个未跟踪文件；
5. 修复 UUIDv7 Review route 边界及 focused tests；
6. 修复 SPA 路由焦点、标题/播报和 skip link；
7. 把 Foundation 状态建模为 checking / ready / unavailable；
8. 增加最小 DOM shell 状态矩阵测试；
9. 运行 focused verification；
10. 运行 boundary、frontend build、Rust tests/check；
11. 使用真实 npm/cargo 环境重新执行 Tauri debug no-bundle build；
12. 确认可执行文件是修复后的新产物，并完成最小窗口启动验证；
13. 运行 `git diff --check`、完整 diff review、`git status`；
14. 确认两份 lockfile 的真实状态；
15. 只有最终短复审无阻断项时，才可把 B0.4 判定为 Done 并请求用户允许提交。

建议验证命令（接手者需根据实际 PATH 调整，但不得伪造工具）：

```powershell
npm run test:shells
npm run check:boundaries
npm run build

Set-Location src-tauri
cargo test --workspace --locked
cargo check --workspace --locked
Set-Location ..

npm run tauri build -- --debug --no-bundle
git diff --check
git diff
git status
git branch --show-current
```

## 11. Completion and Git constraints

B0.4 当前尚未满足 Done，原因是 UUIDv7 route 契约和 SPA accessibility 仍有阻断项，而且最终修复后的 Tauri exe 尚未重新构建和启动验证。

当前禁止：

- 进入 B0.5；
- commit；
- tag；
- push；
- 创建远程仓库；
- 宣称 B0.4 已完成。

当且仅当所有阻断项修复、focused/broader verification、Windows Tauri build、最新 exe 启动验证和最终 diff review 都通过后，建议的提交信息为：

`build: complete B0.4 startup shells`

M0 稳定 tag 只能在 B0.5 完成并满足整个 M0 Definition of Done 后创建。
