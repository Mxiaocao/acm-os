# ACM-OS

ACM-OS is in incremental BUILD. The current repository corresponds to M0 through B0.2.

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

No workspace configuration, Vault, Contest, Review, Today, or other B0.3+ behavior belongs in B0.2.
