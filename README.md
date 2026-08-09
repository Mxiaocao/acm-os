# ACM-OS

ACM-OS is in incremental BUILD. The current repository scaffold corresponds to M0 / B0.1 only.

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

`check:boundaries` uses only Node built-ins plus `cargo metadata --locked`. It validates the exact B0.1 Rust and frontend dependency allowlists and rejects direct Domain/Application or frontend authority escapes. The commands above are the locked Windows B0.1 verification path; release installer bundling is intentionally outside this scaffold check.

Frontend and desktop builds require the Node and Rust/Tauri prerequisites described by the current official Tauri documentation.

No SQLite, Vault, Contest, Review, Today, or other later-milestone behavior belongs in B0.1.
