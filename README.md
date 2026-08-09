# ACM-OS

ACM-OS is in incremental BUILD. The current repository corresponds to M0 through B0.3.

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
