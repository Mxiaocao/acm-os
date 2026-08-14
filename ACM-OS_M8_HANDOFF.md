# ACM-OS M8 Handoff

更新时间：2026-08-14

## 当前状态

当前阶段为 **M8 — Recovery / Backup / Diagnostics / Failure Hardening**。

M8 已完成并通过最终验收，当前工作树尚未提交。不得进入 M9/M10，也不得重复 M0–M7。

## 本轮完成范围

- Critical Operation durable journal、启动阻断与恢复
- Safe Patch recovery copy、崩溃矩阵与启动恢复
- Problem / Knowledge Location Anomaly 修复
- Knowledge identity conflict recovery
- Manual / Daily / Weekly backup、inventory、retention preview/apply
- System restore candidate、pre-restore snapshot、verified database swap
- Durable Restore Intent、启动消费、恢复诊断、回滚清理
- Post-restore Problem/Knowledge binding validation
- Derived knowledge rebuild preview、precondition、explicit apply
- System health snapshot
- Privacy-filtered diagnostic export preview 与实际原子 JSON 导出
- Recovery Shell diagnostic preview/export 操作
- Codeforces adapter 静态 health projection
- Backup inventory 长路径布局修复
- Desktop E2E 测试退出与长流程超时修复

## 重要恢复行为

Recovery runtime 在数据库不可用时仍保留 app-private data 根目录（如果根目录本身可用），因此可以生成脱敏诊断包；损坏数据库、无 SQLite pool 的 Recovery 导出已有专门测试。

诊断 JSON：

- 使用 `.partial` 写入后 atomic rename 发布
- 排除 Markdown 内容、题面内容、credentials、absolute workspace paths
- 包含 startup、restore、backup、critical operation、adapter health 摘要

## 验证证据

后端：

- `cargo test --workspace`：188 passed / 0 failed / 2 ignored
- ignored 仅为需要真实网络的 Codeforces smoke tests
- `cargo check --workspace`：通过
- `cargo fmt --all -- --check`：通过
- `git diff --check`：通过

前端与桌面：

- `npm.cmd run check:boundaries`：5/5 passed
- `npm.cmd run test:shells`：6/6 passed
- `npm.cmd run test:dom-shells`：35/35 passed
- `npm.cmd run build`：通过
- `npm.cmd run test:desktop-e2e`：通过

Desktop E2E 已覆盖：Knowledge discovery/status、accepted-intent explicit Safe Patch、重启恢复、weekly budget、date-local override、core loop recall。

正常 debug 可执行文件：

`E:\项目开发\acm-os\src-tauri\target\debug\acm-os.exe`

## 当前 Git 状态

- 分支：`main`
- 尚未 commit / tag / push
- 既有未提交修改和未知 migration 必须全部保留
- 未知 migration：
  - `0021_create_critical_operations.sql`
  - `0022_add_confirmed_deleted_knowledge_binding.sql`
  - `0023_add_knowledge_rebuild_decision.sql`

## 下一个窗口接管步骤

1. 先读取本文件、`ACM-OS_BUILD_HANDOFF.md` 与 `ACM-OS_RECOVERY_PROMPT.md`。
2. 不修改冻结 SPEC / DESIGN / PLAN，不进入 M9/M10。
3. 确认 `git status`，保留当前工作树。
4. 如用户要求提交，先复核 `git diff --check` 与关键测试证据，再提交当前全部 M8 修改。
5. 未经明确要求不得 push、tag 或改写历史。

## 提交建议

建议 commit message：

`Complete M8 recovery backup diagnostics hardening`

