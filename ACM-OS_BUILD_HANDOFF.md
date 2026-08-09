# ACM-OS BUILD Handoff

## 1. Authority

The project authority order remains:

`SPEC > DESIGN > PLAN > IMPLEMENTATION`

Frozen sources:

- `ACM-OS_SPEC_v1.md` — Product Source of Truth
- `ACM-OS_DESIGN_v1.md` — Design Source of Truth
- `ACM-OS_PLAN_v1.md` — Implementation Plan Source of Truth

No frozen product rule, architecture boundary, authority model, lifecycle, Review rule, Today rule, Markdown rule, transaction boundary, test strategy, or Milestone order may be silently changed during BUILD.

## 2. Current BUILD position

Current Milestone: `M0 — Executable Foundation + Workspace Ready Gate`

Current Slice: `B0.1 — Repository / Workspace Scaffold`

Current status: `PARTIAL / BLOCKED / NOT DONE`

Do not enter B0.2 until B0.1 has full build evidence.

This is not a `SPEC-CONFLICT`.

## 3. What the previous BUILD Chat actually did

The previous BUILD Chat had no existing ACM-OS repository available in its sandbox. `/mnt/data` only contained the uploaded frozen documents.

It created a new sandbox repository at:

`/mnt/data/acm-os`

and ran:

`git init -b main`

Observed Git state at the end:

- branch: `main`
- commits: `0`
- all project files untracked
- no commit, tag, push, reset, clean, or force operation performed

It created a B0.1 scaffold aligned with the frozen Modular Monolith architecture:

```text
React + TypeScript + Vite
        ↓
typed Tauri IPC
        ↓
Application
        ↓
Domain

Tauri Composition Root
        ↓
Infrastructure
```

Rust workspace shape created:

```text
src-tauri/
├─ src/                         # thin Tauri shell
└─ crates/
   ├─ acm-os-domain/
   ├─ acm-os-application/
   └─ acm-os-infrastructure/
```

A minimal `foundation_status` path was added only to verify B0.1 wiring:

`React → @tauri-apps/api invoke → Tauri command DTO → Application → Domain`

It does not contain real product business logic.

## 4. Actual change surface reported

The previous BUILD Chat reported:

- `26 files`
- `462 lines`

Main additions:

```text
.gitignore
README.md
package.json
tsconfig.json
vite.config.ts
index.html

src/
├─ main.tsx
├─ app/
│  ├─ App.tsx
│  └─ app.css
└─ ipc/
   └─ foundation.ts

src-tauri/
├─ Cargo.toml
├─ build.rs
├─ tauri.conf.json
├─ capabilities/default.json
├─ src/
│  ├─ main.rs
│  ├─ lib.rs
│  └─ ipc.rs
└─ crates/
   ├─ acm-os-domain/
   ├─ acm-os-application/
   └─ acm-os-infrastructure/

scripts/
└─ check-boundaries.mjs

docs/
└─ architecture-boundaries.md
```

It explicitly did not create:

- SQLite schema
- migrations
- Vault access
- Codeforces adapter
- Review engine
- Today planner
- workspace settings

Therefore there is no evidence that it entered B0.2+.

## 5. Verification evidence already obtained

### PASS — Architecture boundary gate

Executed:

`npm run check:boundaries`

Observed result:

`boundary check passed`

The check reportedly blocks:

- Domain → Tauri
- Domain → SQLx
- Domain → reqwest
- Domain → notify
- Application → Tauri
- Application → SQLx
- Application → reqwest
- Application → notify

It also checks expected dependency direction:

- Application → Domain
- Infrastructure → Application
- Tauri Shell → Application + Infrastructure

and checks that Frontend does not gain direct DB / FS authority through packages such as:

- `better-sqlite3`
- `sql.js`
- `@tauri-apps/plugin-fs`

A negative test was also performed in a temporary copy by intentionally adding `tauri = "2"` to Domain. The boundary check then failed with:

`boundary check failed: domain must not depend on tauri`

After removing the deliberate violation, the real scaffold again passed.

### PASS — Basic static checks

Executed:

`node --check scripts/check-boundaries.mjs`

Passed.

The following JSON files were also parsed successfully:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`

Observed result:

`json parse passed`

### PASS — Diff review

In a separate temporary Git review copy, the previous BUILD Chat ran:

`git diff --cached --check`

No whitespace/error output was reported.

## 6. Unresolved blockers

### BLOCKER A — npm dependency resolution

Executed:

`npm install --package-lock-only --ignore-scripts`

Observed failure:

```text
npm ERR! 404
'@tauri-apps/api@^2' is not in this registry
```

The request was going through the sandbox internal registry:

`packages.applied-caas-gateway1.internal.api.openai.org`

Therefore:

- `package-lock.json` was not generated
- `node_modules` was not generated

No lockfile was fabricated.

### BLOCKER B — Frontend build

Executed:

`npm run build`

It failed because dependencies were unavailable, including React, `@tauri-apps/api/core`, and Vite.

This is not accepted as a PASS.

### BLOCKER C — Rust toolchain unavailable

Executed:

`cargo --version`

`rustc --version`

Observed:

- `cargo: command not found`
- `rustc: command not found`

Therefore the following were not run and must not be claimed as verified:

- `cargo check --workspace`
- Tauri build/check

The previous sandbox was also Linux, while the frozen MVP blocking platform is Windows 10/11 x64.

## 7. B0.1 current verdict

```text
Structure implementation:             PASS
Architecture boundary verification:   PASS
Diff review:                           PASS
Dependency resolution:                BLOCKED
Frontend compile:                      BLOCKED
Rust workspace compile:                BLOCKED
Tauri executable verification:         BLOCKED

Overall: PARTIAL / NOT DONE
```

B0.1 must not be marked Done merely because the scaffold looks structurally correct.

## 8. Evidence still required before B0.1 can close

In an environment with a usable npm registry and Rust/Tauri toolchain, verify at minimum:

```text
npm install
npm run check:boundaries
npm run build

cd src-tauri
cargo check --workspace
```

Then perform the appropriate actual Tauri build/check for the target environment and prove that the following wiring compiles together:

```text
React
→ typed IPC
→ thin Tauri shell
→ Application
→ Domain
```

Inspect the generated:

- `package-lock.json`
- `Cargo.lock`

and confirm actual resolved versions rather than guessed versions.

## 9. Git checkpoint state

No commit should be assumed to exist.

Previous recommendation after B0.1 is fully verified:

`chore: scaffold ACM-OS workspace boundaries`

Do not create the Milestone tag `acm-os-m0-foundation` until the entire M0 Definition of Done is complete.

## 10. Critical handoff warning

The scaffold above was created in the previous Chat's sandbox at `/mnt/data/acm-os`.

That sandbox path is not proof that the same files exist in the user's real local ACM-OS repository or in the new Work environment.

Before doing anything, the new Work must inspect the actual accessible repository/filesystem and distinguish:

1. real user repository state;
2. uploaded scaffold ZIP/files, if provided;
3. previous Chat's reported sandbox state.

Do not recreate, overwrite, or merge files blindly.

If the scaffold ZIP is supplied, inspect it before deciding whether to copy/import it into the actual repository.

If an actual repository already exists, its `git status`, current branch, files, manifests, and diff are the source of truth for current implementation state.

## 11. Exact next action for the new BUILD Work

Do not start B0.2.

First:

1. Read `ACM-OS_SPEC_v1.md`, `ACM-OS_DESIGN_v1.md`, and `ACM-OS_PLAN_v1.md` completely.
2. Read this handoff.
3. Inspect the actual repository / uploaded scaffold available in the new environment.
4. Run and report `git status` and `git branch --show-current` if a Git repository exists.
5. Compare the actual files against the B0.1 handoff claims.
6. If the B0.1 scaffold is present, continue only the missing B0.1 verification work.
7. If the scaffold is not present but the ZIP is provided, inspect the ZIP and establish the safest way to place it into the intended repository without overwriting unrelated user work.
8. Resolve or explicitly report the Node/Rust/Tauri environment blockers.
9. Run the missing B0.1 compile/build checks.
10. Review the actual diff/status.
11. Only after all B0.1 Done evidence passes may B0.2 begin.

The correct immediate goal is:

`Finish and verify B0.1 — not redesign it, not restart it, and not advance to B0.2 prematurely.`
