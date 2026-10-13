# ACM-OS

ACM-OS 是用于算法训练、题目收藏和复习安排的 Windows 桌面应用。当前发布版本为 `v0.1.0`。

## 下载和使用

从 [GitHub Releases](https://github.com/Mxiaocao/acm-os/releases) 下载 Windows 安装包，或解压免安装版并运行 `acm-os.exe`。首次启动按界面提示设置工作区和笔记目录。

- 比赛书架支持分类、系列和年份组织，随窗口宽度调整每层容量。
- 题库支持比赛题目收藏、Codeforces 单题链接导入和手动录入。
- 题目列表提供题面预览、LaTeX 公式、首次导入日期和关键词搜索。
- 学习月历展示每日学习记录及待复习题目。
- 樱花粉界面搭配“未来知识代码之书”应用图标。

## 从源码构建

安装 Node.js 和 Rust，以及 Windows 上的 Tauri 构建依赖后执行：

```bash
npm ci
npm run check:boundaries
npm run build
cargo test --workspace --locked --manifest-path src-tauri/Cargo.toml
npm run test:dom-shells
npm run tauri -- build --bundles nsis
```

安装包输出到 `src-tauri/target/release/bundle/nsis/`。以下保留早期阶段的检查说明，当前功能以发布说明和源码为准。

## B0.1 checks

```bash
npm ci
npm run check:boundaries
npm run build
cd src-tauri
cargo check --workspace --locked
cd ..
npm run tauri build -- --debug --no-bundle
```

`check:boundaries` uses only Node built-ins plus `cargo metadata --locked`. It validates the current Rust and frontend dependency allowlists and rejects direct Domain/Application or frontend authority escapes. The commands above remain the locked Windows scaffold verification path; release installer bundling is intentionally outside this check.

## B0.2 SQLite startup gate

```bash
cd src-tauri
cargo test --workspace --locked
cd ..
```

B0.2 stores the SQLite database in Tauri App Local Data, runs forward-only embedded SQLx migrations, verifies database and foreign-key integrity, creates a SQLite-consistent pre-migration backup when required, and blocks unsupported or damaged schemas behind a typed recovery status.

Frontend and desktop builds require the Node and Rust/Tauri prerequisites described by the current official Tauri documentation.

## B0.3 workspace configuration

B0.3 adds the initial Active Vault, Problem Notes Root, and Knowledge Root configuration. All three paths must already exist as directories. The two roots must be strict descendants of the resolved Vault and may not be equal or contain one another. Validation is owned by Application, filesystem resolution and SQLite persistence are owned by Infrastructure, and React uses typed IPC only.

The initial configuration is create-only. Replacing an Active Vault remains blocked until the later Validate → Preview → Confirm → Commit flow exists. B0.3 does not scan Markdown, adopt existing notes, create problem files, or add B0.4 workspace shells.

## B0.4 startup shells

The Application startup gate selects Recovery, Setup, or Normal before React applies URL state. Recovery and Setup never expose normal navigation. A configured workspace enters the Normal shell at Today; `/review/:attemptId` uses a separate full-screen Focus shell with ordinary navigation hidden.

```bash
npm run test:shells
```

B0.4 provides layout and routing boundaries only. Contest, Problem, Knowledge, Review execution, and Today planning behavior remain outside M0.
