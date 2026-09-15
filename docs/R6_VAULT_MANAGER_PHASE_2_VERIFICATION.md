# ACM-OS R6 Vault Manager Phase 2 — 验收记录

验证日期：2026-09-15（Asia/Shanghai）。开发与提交位于原仓库 `E:\项目开发\acm-os`。

```text
R6_VAULT_MANAGER_PHASE_2 = PASS
TYPED_IPC = PASS
REACT_UI = PASS
CREATE_FROM_UI = PASS
OPEN_FROM_UI = PASS
DELETE_FROM_UI = PASS
CROSS_VAULT_WRITE_BLOCK = PASS
RESTART_PERSISTENCE = PASS
CARGO_TEST = PASS
CARGO_CHECK = PASS
FRONTEND_BUILD = PASS
```

## 实现范围

- Settings / Workspace 提供当前 Vault 名称、路径、状态、列表与 Create / Open / Delete。无当前 Vault、路径不可用或注册表异常时提供 Manager 入口，不回退到 Legacy。
- 五个管理 IPC 使用明确 DTO 与结构化错误：status、create、open、preview-delete、delete。
- 每个 Vault 使用真实迁移后的 `.acm-os/system-facts.sqlite3`，独立连接池和 Markdown 缓存。Phase 1 的空文件占位实现已替换，不把它当作 SQLite 验证证据。
- 121 个数据库业务命令必须携带 `{vaultId, epoch}` 并持有 Manager 读租约。切换持有写锁，等待在途命令完成；旧 ID/epoch 请求在数据库操作之前拒绝。重启生成新 epoch。
- Open 先只读检查数据库的 workspace 归属，再初始化运行时。注册表持久化成功后切换运行时并重绑 watcher。前端撤销旧上下文、重载 WebView，销毁旧页面、定时器和闭包。
- Delete 需要后端预览令牌和准确的 `DELETE <name>` 文本；禁止删除当前 Vault。注册表写入失败会回滚目录重命名。
- 删除是可恢复删除：移除注册，保留笔记，把管理数据移动到 `.acm-os-deleted-<id>`。UI 在确认前显示恢复目录。管理及恢复目录不会成为 Markdown 索引/重定位/重绑目标。
- Legacy 数据库不迁移、不复制、不合并；保护其注册的原 Vault 根目录。真实 Legacy SQLite 文件逐字节一致性测试通过。

## 分阶段门禁

| 阶段 | cargo test --workspace | cargo check --workspace | tsc + vite build | git diff 审查 |
| --- | --- | --- | --- | --- |
| 2-A | PASS | PASS | PASS | PASS |
| 2-B | PASS | PASS | PASS | PASS |
| 2-C 最终版 | PASS | PASS | PASS | PASS |

对应记录：`verification/r6-phase2/phase-a-results.json`、`phase-b-results.json`、`phase-c-results.json`；同目录保存各命令完整日志。最终 C 门禁的实际进程退出码见 `final-phase-c-exit.json`（0）。

最终 Rust：**380 passed / 2 ignored / 0 failed**，其中基础设施 **258 passed / 2 ignored**，包含 **10 个 Vault Manager 测试**。两个 ignored 为原有外部网络测试，本阶段未将其算作通过。

## 真实隔离证据

| 验收场景 | 证据 |
| --- | --- |
| A 创建 Contest A；B 不存在 | 实际 schema + production import/read port；Rust 测试与真实 Tauri IPC 均通过 |
| B 创建 Problem B；A 不存在 | Rust 创建个人笔记并读回；桌面切换后题目列表无 B |
| A 增加 XP / Coin；B 不受影响 | Rust 通过学习完成事件和正式 Reward policy 发放正向 XP/Coin；B 两者为 0；返回 A 余额一致 |
| A→B→A→B | Rust 和真实 WebView 均通过，旧 A epoch 不会重新有效 |
| 重启仍为 B | Rust shutdown/reopen 与真实桌面进程退出/再启动均通过 |
| 阻断旧请求与缺失上下文 | 真实 Tauri 旧 context 写请求返回 stale_vault_context；缺失 context 在 IPC 参数解析阶段拒绝 |
| 在途写入 | 持有 A 租约时切换 future 为 Pending；A 命令完成后才能进入 B，B 无该记录 |
| 故障保护 | 注册表损坏、数据库归属错误、提交失败、路径/marker 异常、错误/重复确认令牌均有拒绝/回滚测试 |
| Legacy | 用原引擎创建真实 Legacy DB 和 sentinel family；所有 Vault 操作和重启前后数据库字节相同 |

主要组合测试：`contest_problem_reward_isolation_a_b_a_b_and_restart_preserves_legacy_bytes`。其余测试名称及结果均在 `phase-c-cargo-test.log` 中。

## UI 与桌面闭环证据

- 新增 UI 自动测试：**5/5 PASS**，覆盖创建、预览/输入/取消删除、打开后撤销上下文、过期响应、后端错误展示。此组使用 IPC mock，不单独作为完整闭环证据。
- 原有前端回归：**127/127 PASS**，覆盖页面、Reward、路由、可访问性和视觉结构检查。
- Vault IPC 边界覆盖：**2/2 PASS**，覆盖全部 121 个数据库命令。
- 真实 Tauri + WebView：实际点击创建 A/B/C、打开 A/B/A/B、二次确认删除 C，重启确认 B；独立 SQLite 文件头校验通过。`vault-desktop-final-exit.json` 为 0，`desktop-results.json` 为 PASS。
- 原有真实桌面业务回归：知识发现、显式 Safe Patch、每周预算、日期覆盖、复习闭环和重启通过；`core-desktop-regression-exit.json` 为 0。

旧桌面测试脚本已适配新的 Vault 初始化和 context，并修正此前依赖英文标签/旧 code 元素的定位；没有放宽业务结果断言。架构依赖检查器补登记基线中早已存在的 Application chrono 依赖，没有新增依赖。

## Diff 审查

`final-diff-audit.json` 记录暂存区 whitespace 检查和变更边界，`source-manifest.json` 记录被验证源文件 SHA-256。

- Domain / Application 业务实现：未修改。
- SQL migrations / Cargo manifests / lockfiles：未修改。
- 不包含生产数据库、真实 Vault、密钥或用户数据变更。
- Rust 使用隔离 TempDir；桌面测试使用仓库内 `.desktop-e2e-*` 临时目录。已有生产应用进程未操作。
- 非阻断构建提示：原有未使用测试导入、Windows linker 信息及前端大 chunk 提示；不影响测试/构建退出码。

交付时将源码及证据一并提交，并以提交后 `git status --porcelain` 无输出确认工作区干净。

## 复现

```powershell
node scripts/verify-vault-phase.mjs phase-c
node --import tsx --test scripts/vault-manager-ui.test.mjs
node --test scripts/vault-boundary.test.mjs
node --test --test-concurrency=1 scripts/dom-shells.test.mjs scripts/reward-page.test.mjs scripts/startup-shells.test.mjs scripts/check-boundaries.test.mjs scripts/accessibility-css.test.mjs scripts/visual-consistency.test.mjs
node scripts/check-boundaries.mjs
node scripts/desktop-e2e.mjs
node scripts/vault-desktop-e2e.mjs
npm run build
```

桌面构建与普通构建共享 dist，须串行运行。最后的普通 build 恢复不含测试驱动的前端产物。

## 明确边界

本阶段不实现永久清除恢复目录、Legacy 浏览/导入、云同步、账号、AI 或社区。失败创建留下的管理目录不会被自动删除或悄悄接管；应先检查失败目录，再选择其他现有根目录。未声称完成断电级文件系统故障注入测试。
