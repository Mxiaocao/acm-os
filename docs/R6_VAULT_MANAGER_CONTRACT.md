# R6 Vault Manager — Phase 2 contract (supersedes Phase 1 draft)

## Evidence and compatibility

Inspection on 2026-09-15 found Phase 1 only created a zero-byte placeholder and changed a JSON pointer. No business runtime switch existed. Phase 2 must initialize the real production schema and route every database-backed desktop command through the manager. R0–R4 business rules and migrations remain unchanged.

Legacy `system-facts.sqlite3` in app-private data is never opened for writing, migrated, copied, or merged. No-active startup presents the manager. A corrupt registry fails closed; it must never be treated as an empty registry.

## Storage and lifecycle

Registry: app-private `vault-manager.json`, atomically replaced, process lock held while manager lives. Each registered canonical root owns `.acm-os/system-facts.sqlite3` (the existing persistence engine's real database filename), plus an ownership marker. No arbitrary database-path request is accepted. Phase 1 placeholder roots are not silently adopted or migrated.

Creation accepts an existing root, rejects duplicate/nested roots, app-private/Legacy roots, and existing `.acm-os` contents. It initializes the real schema and `Problems`/`Knowledge` roots. Creation registers but does NOT switch current Vault; user explicitly opens it.

Open validates canonical root, owned data directory, marker and database before starting a fresh runtime. On failure the old binding stays current. Successful open atomically persists the selected ID, swaps the runtime/cache, and issues a new session epoch. Restart resolves the saved ID; missing/corrupt target is `unavailable` and never falls back to Legacy or another Vault.

Delete is two steps: preview produces a one-use token bound to ID/session epoch; confirm requires token AND exact `DELETE <name>`. Current Vault cannot be deleted. Validation is repeated at commit. Only the owned data directory is renamed to `.acm-os-deleted-<id>` and registration removed; this recoverable deletion retains notes and backups. The UI states that retained directory explicitly. No recursive deletion or Markdown removal occurs. If registry commit fails the rename is rolled back.

## Typed IPC

All DTO fields use camelCase, states/error codes use snake_case.

- `vault_manager_status() -> ManagerStatus { vaults: VaultSummary[], current: CurrentVaultState, context: VaultContext|null }`
- `create_vault({name, rootPath}) -> VaultSummary`
- `open_vault({vaultId}) -> ManagerStatus`
- `preview_delete_vault({vaultId}) -> DeletePreview {vaultId, name, rootPath, retainedDataPath, confirmationText, token}`
- `delete_vault({vaultId, token, confirmation}) -> ManagerStatus`

`VaultSummary {id, name, rootPath, databasePath, createdAtUtc, isCurrent}`.
`CurrentVaultState = {state:none} | {state:ready, vault} | {state:unavailable, vault, reason}`.
`VaultContext {vaultId, epoch}` is required on EVERY database command. Missing context is rejected at IPC deserialization; a stale/wrong context is rejected before any database work. Manager errors are structured `{code}` enum values.

## Isolation and concurrency

Manager owns the only active DatabaseRuntime. A command holds a read lease for its entire operation, including asynchronous work. Switching/deletion hold an exclusive lock and wait for earlier commands to finish. No Arc/runtime escape. Epoch prevents stale writes after A→B→A and after restart. Frontend drops late responses when its context changes, unmounts previous workspace state and reloads the shell on switch. No old-Vault payload may be automatically retried in the new context.

Workspace setup is constrained to the selected root. Watcher is stopped/rebound on open. The manager is available in Settings and in no-active/unavailable startup.

## Verification gates

For Phase 2-A, 2-B, and 2-C: cargo test, cargo check, frontend build, git diff audit with saved logs. Real migrated SQLite integration tests cover Contest A absent in B, Problem B absent in A, XP/Coin independent, A→B→A→B, restart B, wrong/stale/missing context, switch waiting on in-flight operations, duplicate/nested paths, corrupt registry, current deletion, token reuse/mismatch, and Legacy byte preservation. DOM tests cover forms and delayed responses; desktop WebView tests cross actual React→IPC→Rust→SQLite and restart.

## Phase 2 review notes

- Database preflight checks persisted workspace ownership read-only before any migration/recovery work.
- `.acm-os` and `.acm-os-deleted-*` are reserved private paths: excluded from Markdown discovery, relocation candidates and explicit rebinding.
- The current implementation intentionally preserves failed-create artifacts instead of deleting or silently adopting them. A failed create never publishes a Vault registration or changes the active binding; choose another existing empty root after inspecting the failed root.
- Legacy browsing/adoption, permanent recovery-directory purge, automatic migration of Phase 1 zero-byte placeholders, and OS power-loss fault injection are outside this phase. Missing/corrupt records fail closed.
- Existing dependencies were not changed. The architecture checker now recognizes the pre-existing Application `chrono` dependency (present in the baseline manifest).
