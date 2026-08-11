use std::collections::HashMap;
use std::fs::{File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use acm_os_application::{
    ContestImportDraft, ContestImportPersistenceError, ContestImportPort, ContestImportStatus,
    ContestDetail, ContestReadError, ContestReadPort, ContestShelfItem, LightweightProblemDetail,
    LightweightProblemItem, LocalStatementAsset, PersistedContestImport, StatementReadState,
    PersonalNoteBinding, PersonalNoteCreationContext, PersonalNoteError, PersonalNotePort,
    PersonalNoteReadError, PersonalNoteReadPort, PersonalNoteReadState, ProblemMarkdownProjection,
    ProblemIdentityType, CreatedPersonalNoteFile, StartupGateStatus, StartupRecoveryReason,
    StatementSnapshotDraft, WorkspaceConfiguration,
    WorkspaceConfigurationPort, WorkspacePathResolutionError, WorkspacePersistenceError,
};
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::SqlitePool;

use crate::file_binding::{
    resolve_personal_note, sha256_hex, windows_file_key, BindingResolution, ResolvedNoteFile,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
const DATABASE_FILENAME: &str = "system-facts.sqlite3";
const STARTUP_LOCK_FILENAME: &str = ".database-startup.lock";
const STARTUP_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const STARTUP_LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(50);

type SqliteColumnContract = (i64, String, String, i64, Option<String>, i64, i64);

pub struct DatabaseRuntime {
    _pool: Option<SqlitePool>,
    _startup_lock: Option<File>,
    status: StartupGateStatus,
    markdown_projection_cache: Mutex<HashMap<String, ProblemMarkdownProjection>>,
}

impl DatabaseRuntime {
    pub fn recovery(reason: StartupRecoveryReason) -> Self {
        Self {
            _pool: None,
            _startup_lock: None,
            status: StartupGateStatus::RecoveryRequired { reason },
            markdown_projection_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn status(&self) -> &StartupGateStatus {
        &self.status
    }

    fn pool(&self) -> Result<&SqlitePool, WorkspacePersistenceError> {
        self._pool
            .as_ref()
            .ok_or(WorkspacePersistenceError::Unavailable)
    }

    async fn update_binding_state(
        &self,
        problem_id: i64,
        state: &str,
        expected: &PersonalNoteBinding,
    ) -> Result<(), PersonalNoteReadError> {
        let result = sqlx::query(
            "UPDATE file_bindings \
             SET binding_state = ?1, updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE problem_id = ?2 AND vault_relative_path = ?3 \
               AND content_digest = ?4 AND windows_file_key IS ?5",
        )
        .bind(state)
        .bind(problem_id)
        .bind(&expected.vault_relative_path)
        .bind(&expected.content_digest)
        .bind(&expected.windows_file_key)
        .execute(
            self._pool
                .as_ref()
                .ok_or(PersonalNoteReadError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| PersonalNoteReadError::PersistenceUnavailable)?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(PersonalNoteReadError::BindingUnavailable)
        }
    }

    async fn commit_resolved_binding(
        &self,
        problem_id: i64,
        expected: &PersonalNoteBinding,
        resolved: &ResolvedNoteFile,
    ) -> Result<bool, PersonalNoteReadError> {
        let result = match sqlx::query(
            "UPDATE file_bindings \
             SET vault_relative_path = ?1, windows_file_key = ?2, content_digest = ?3, \
                 binding_state = 'linked', \
                 updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE problem_id = ?4 AND vault_relative_path = ?5 \
               AND content_digest = ?6 AND windows_file_key IS ?7",
        )
        .bind(&resolved.relative_path)
        .bind(&resolved.windows_file_key)
        .bind(&resolved.content_digest)
        .bind(problem_id)
        .bind(&expected.vault_relative_path)
        .bind(&expected.content_digest)
        .bind(&expected.windows_file_key)
        .execute(
            self._pool
                .as_ref()
                .ok_or(PersonalNoteReadError::PersistenceUnavailable)?,
        )
        .await
        {
            Ok(result) => result,
            Err(sqlx::Error::Database(database_error))
                if database_error.is_unique_violation() => {
                return Ok(false);
            }
            Err(_) => return Err(PersonalNoteReadError::PersistenceUnavailable),
        };
        if result.rows_affected() == 1 {
            Ok(true)
        } else {
            Err(PersonalNoteReadError::BindingUnavailable)
        }
    }
}

fn resolved_binding(resolved: &ResolvedNoteFile) -> PersonalNoteBinding {
    PersonalNoteBinding {
        vault_relative_path: resolved.relative_path.clone(),
        content_digest: resolved.content_digest.clone(),
        windows_file_key: resolved.windows_file_key.clone(),
    }
}

impl PersonalNoteReadPort for DatabaseRuntime {
    async fn read_personal_note_projection(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<PersonalNoteReadState, PersonalNoteReadError> {
        let row: Option<(
            i64,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT p.id, p.identity_type, ws.active_vault_path, fb.vault_relative_path, \
                    fb.content_digest, fb.windows_file_key \
             FROM problems p \
             LEFT JOIN file_bindings fb ON fb.problem_id = p.id \
             LEFT JOIN workspace_settings ws ON ws.singleton = 1 \
             WHERE p.platform = 'codeforces' \
               AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(
            self._pool
                .as_ref()
                .ok_or(PersonalNoteReadError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| PersonalNoteReadError::PersistenceUnavailable)?;
        let (problem_id, identity_type, active_vault, relative_path, digest, file_key) =
            row.ok_or(PersonalNoteReadError::ProblemNotFound)?;
        if identity_type != "personal" {
            return Err(PersonalNoteReadError::NotPersonal);
        }
        let active_vault = match active_vault {
            Some(active_vault) => active_vault,
            None => {
                return Err(PersonalNoteReadError::PersistenceUnavailable);
            }
        };
        let relative_path = relative_path.ok_or(PersonalNoteReadError::BindingUnavailable)?;
        let digest = digest.ok_or(PersonalNoteReadError::BindingUnavailable)?;
        let last_binding = PersonalNoteBinding {
            vault_relative_path: relative_path.clone(),
            content_digest: digest.clone(),
            windows_file_key: file_key.clone(),
        };
        let read_vault = active_vault.clone();
        let read_relative = relative_path.clone();
        let read_key = file_key.clone();
        let read_digest = digest.clone();
        let resolution = tokio::task::spawn_blocking(move || {
            resolve_personal_note(
                &read_vault,
                &read_relative,
                read_key.as_deref(),
                &read_digest,
            )
        })
        .await
        .map_err(|_| PersonalNoteReadError::FileReadFailed)?;

        let resolved = match resolution {
            BindingResolution::Ready(resolved) => resolved,
            BindingResolution::LocationAnomaly => {
                self.update_binding_state(problem_id, "location_anomaly", &last_binding)
                    .await?;
                return Ok(PersonalNoteReadState::LocationAnomaly {
                    binding: last_binding,
                });
            }
            BindingResolution::VaultUnavailable => {
                self.update_binding_state(
                    problem_id,
                    "external_source_unavailable",
                    &last_binding,
                )
                    .await?;
                return Ok(PersonalNoteReadState::VaultUnavailable {
                    binding: last_binding,
                });
            }
            BindingResolution::InvalidBinding => {
                return Err(PersonalNoteReadError::BindingUnavailable);
            }
        };
        if !self
            .commit_resolved_binding(problem_id, &last_binding, &resolved)
            .await?
        {
            self.update_binding_state(problem_id, "location_anomaly", &last_binding)
                .await?;
            return Ok(PersonalNoteReadState::LocationAnomaly {
                binding: last_binding,
            });
        }

        // Disk bytes are always read and digested before this cache is consulted.
        let content_digest = resolved.content_digest.clone();
        let cache_key = format!(
            "codeforces:{}:{}:{}",
            problem.contest().contest_id(),
            problem.index(),
            resolved.relative_path
        );
        {
            let cache = self
                .markdown_projection_cache
                .lock()
                .map_err(|_| PersonalNoteReadError::PersistenceUnavailable)?;
            if let Some(projection) = cache.get(&cache_key) {
                if projection.content_digest == content_digest {
                    return Ok(PersonalNoteReadState::Ready {
                        binding: resolved_binding(&resolved),
                        projection: projection.clone(),
                        relocated: resolved.relocated,
                    });
                }
            }
        }

        let markdown = std::str::from_utf8(&resolved.bytes)
            .map_err(|_| PersonalNoteReadError::InvalidUtf8)?;
        let projection = crate::markdown::parse_problem_markdown(markdown, content_digest);
        self.markdown_projection_cache
            .lock()
            .map_err(|_| PersonalNoteReadError::PersistenceUnavailable)?
            .insert(cache_key, projection.clone());
        Ok(PersonalNoteReadState::Ready {
            binding: resolved_binding(&resolved),
            projection,
            relocated: resolved.relocated,
        })
    }
}

impl WorkspaceConfigurationPort for DatabaseRuntime {
    async fn resolve_directory(
        &self,
        path: &str,
    ) -> Result<String, WorkspacePathResolutionError> {
        let path = path.to_owned();
        tokio::task::spawn_blocking(move || {
            let resolved = std::fs::canonicalize(path)
                .map_err(|_| WorkspacePathResolutionError::Unavailable)?;
            if !resolved.is_dir() {
                return Err(WorkspacePathResolutionError::NotDirectory);
            }
            resolved
                .to_str()
                .map(str::to_owned)
                .ok_or(WorkspacePathResolutionError::Unavailable)
        })
        .await
        .map_err(|_| WorkspacePathResolutionError::Unavailable)?
    }

    async fn load_workspace_configuration(
        &self,
    ) -> Result<Option<WorkspaceConfiguration>, WorkspacePersistenceError> {
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT active_vault_path, problem_root_path, knowledge_root_path \
             FROM workspace_settings WHERE singleton = 1",
        )
        .fetch_optional(self.pool()?)
        .await
        .map_err(|_| WorkspacePersistenceError::Unavailable)?;

        row.map(|(active_vault_path, problem_root_path, knowledge_root_path)| {
            WorkspaceConfiguration::from_resolved(
                active_vault_path,
                problem_root_path,
                knowledge_root_path,
            )
            .map_err(|_| WorkspacePersistenceError::Unavailable)
        })
        .transpose()
    }

    async fn insert_workspace_configuration(
        &self,
        configuration: &WorkspaceConfiguration,
    ) -> Result<(), WorkspacePersistenceError> {
        sqlx::query(
            "INSERT INTO workspace_settings (\
                singleton, active_vault_path, problem_root_path, knowledge_root_path\
             ) VALUES (1, ?1, ?2, ?3)",
        )
        .bind(configuration.active_vault_path())
        .bind(configuration.problem_root_path())
        .bind(configuration.knowledge_root_path())
        .execute(self.pool()?)
        .await
        .map(|_| ())
        .map_err(|error| match error {
            sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
                WorkspacePersistenceError::AlreadyConfigured
            }
            _ => WorkspacePersistenceError::Unavailable,
        })
    }
}

impl PersonalNotePort for DatabaseRuntime {
    async fn personal_note_creation_context(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<PersonalNoteCreationContext, PersonalNoteError> {
        let row: Option<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
                "SELECT p.identity_type, fb.vault_relative_path, \
                        fb.content_digest, fb.windows_file_key \
                 FROM problems p \
                 LEFT JOIN file_bindings fb ON fb.problem_id = p.id \
                 WHERE p.platform = 'codeforces' \
                   AND p.external_contest_key = ?1 \
                   AND p.external_problem_key = ?2",
            )
            .bind(problem.contest().contest_id() as i64)
            .bind(problem.index())
            .fetch_optional(self.personal_note_pool()?)
            .await
            .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
        let (identity_type, relative_path, digest, file_key) =
            row.ok_or(PersonalNoteError::ProblemNotFound)?;
        let existing_binding = match (relative_path, digest) {
            (Some(vault_relative_path), Some(content_digest)) => Some(PersonalNoteBinding {
                vault_relative_path,
                content_digest,
                windows_file_key: file_key,
            }),
            (None, None) => None,
            _ => return Err(PersonalNoteError::PersistenceUnavailable),
        };
        if (identity_type == "personal") != existing_binding.is_some() {
            return Err(PersonalNoteError::PersistenceUnavailable);
        }
        if identity_type != "lightweight" && identity_type != "personal" {
            return Err(PersonalNoteError::PersistenceUnavailable);
        }
        Ok(PersonalNoteCreationContext {
            problem: problem.clone(),
            existing_binding,
        })
    }

    async fn create_personal_note_file(
        &self,
        context: &PersonalNoteCreationContext,
        markdown: &[u8],
    ) -> Result<CreatedPersonalNoteFile, PersonalNoteError> {
        let (active_vault, problem_root): (String, String) = sqlx::query_as(
            "SELECT active_vault_path, problem_root_path \
             FROM workspace_settings WHERE singleton = 1",
        )
        .fetch_optional(self.personal_note_pool()?)
        .await
        .map_err(|_| PersonalNoteError::PersistenceUnavailable)?
        .ok_or(PersonalNoteError::WorkspaceUnavailable)?;
        let problem = context.problem.clone();
        let markdown = markdown.to_vec();

        tokio::task::spawn_blocking(move || {
            create_personal_note_file_on_disk(&active_vault, &problem_root, &problem, &markdown)
        })
        .await
        .map_err(|_| PersonalNoteError::FileWriteFailed)?
    }

    async fn commit_personal_note_binding(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        file: &CreatedPersonalNoteFile,
    ) -> Result<PersonalNoteBinding, PersonalNoteError> {
        let mut transaction = self
            .personal_note_pool()?
            .begin()
            .await
            .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, identity_type FROM problems \
             WHERE platform = 'codeforces' \
               AND external_contest_key = ?1 AND external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
        let (problem_id, identity_type) = row.ok_or(PersonalNoteError::ProblemNotFound)?;

        if identity_type == "personal" {
            let binding: Option<(String, String, Option<String>)> = sqlx::query_as(
                "SELECT vault_relative_path, content_digest, windows_file_key \
                 FROM file_bindings WHERE problem_id = ?1",
            )
            .bind(problem_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
            transaction
                .rollback()
                .await
                .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
            return binding
                .map(|(vault_relative_path, content_digest, windows_file_key)| {
                    PersonalNoteBinding {
                        vault_relative_path,
                        content_digest,
                        windows_file_key,
                    }
                })
                .ok_or(PersonalNoteError::PersistenceUnavailable);
        }
        if identity_type != "lightweight" {
            return Err(PersonalNoteError::PersistenceUnavailable);
        }

        sqlx::query(
            "INSERT INTO file_bindings (problem_id, vault_relative_path, windows_file_key, content_digest) \
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(problem_id)
        .bind(&file.vault_relative_path)
        .bind(&file.windows_file_key)
        .bind(&file.content_digest)
        .execute(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
        sqlx::query("UPDATE problems SET identity_type = 'personal' WHERE id = ?1")
            .bind(problem_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| PersonalNoteError::PersistenceUnavailable)?;
        Ok(file.clone().into())
    }

    async fn discard_created_personal_note(
        &self,
        file: &CreatedPersonalNoteFile,
    ) -> Result<(), PersonalNoteError> {
        let active_vault: String = sqlx::query_scalar(
            "SELECT active_vault_path FROM workspace_settings WHERE singleton = 1",
        )
        .fetch_optional(self.personal_note_pool()?)
        .await
        .map_err(|_| PersonalNoteError::PersistenceUnavailable)?
        .ok_or(PersonalNoteError::WorkspaceUnavailable)?;
        let file = file.clone();
        tokio::task::spawn_blocking(move || discard_created_note_on_disk(&active_vault, &file))
            .await
            .map_err(|_| PersonalNoteError::CompensationFailed)?
    }
}

fn create_personal_note_file_on_disk(
    active_vault: &str,
    problem_root: &str,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    markdown: &[u8],
) -> Result<CreatedPersonalNoteFile, PersonalNoteError> {
    let vault = std::fs::canonicalize(active_vault)
        .map_err(|_| PersonalNoteError::WorkspaceUnavailable)?;
    let root = std::fs::canonicalize(problem_root)
        .map_err(|_| PersonalNoteError::WorkspaceUnavailable)?;
    if !root.is_dir() || !root.starts_with(&vault) || root == vault {
        return Err(PersonalNoteError::WorkspaceUnavailable);
    }
    let filename = format!(
        "CF-{}-{}.md",
        problem.contest().contest_id(),
        problem.index()
    );
    let target = root.join(filename);
    let mut handle = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&target)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                PersonalNoteError::TargetAlreadyExists
            } else {
                PersonalNoteError::FileWriteFailed
            }
        })?;
    handle
        .write_all(markdown)
        .and_then(|_| handle.sync_all())
        .map_err(|_| PersonalNoteError::FileWriteFailed)?;
    drop(handle);

    let verified = std::fs::read(&target).map_err(|_| PersonalNoteError::FileVerificationFailed)?;
    if verified != markdown {
        return Err(PersonalNoteError::FileVerificationFailed);
    }
    let resolved = std::fs::canonicalize(&target)
        .map_err(|_| PersonalNoteError::FileVerificationFailed)?;
    if !resolved.starts_with(&vault) {
        return Err(PersonalNoteError::FileVerificationFailed);
    }
    let relative = resolved
        .strip_prefix(&vault)
        .map_err(|_| PersonalNoteError::FileVerificationFailed)?
        .to_string_lossy()
        .replace('\\', "/");
    if relative.is_empty() || relative.starts_with('/') {
        return Err(PersonalNoteError::FileVerificationFailed);
    }
    Ok(CreatedPersonalNoteFile {
        vault_relative_path: relative,
        content_digest: sha256_hex(&verified),
        windows_file_key: windows_file_key(&resolved),
    })
}

fn discard_created_note_on_disk(
    active_vault: &str,
    file: &CreatedPersonalNoteFile,
) -> Result<(), PersonalNoteError> {
    let vault = std::fs::canonicalize(active_vault)
        .map_err(|_| PersonalNoteError::CompensationFailed)?;
    let target = vault.join(Path::new(&file.vault_relative_path));
    let resolved = std::fs::canonicalize(&target)
        .map_err(|_| PersonalNoteError::CompensationFailed)?;
    if !resolved.starts_with(&vault) {
        return Err(PersonalNoteError::CompensationFailed);
    }
    let current = std::fs::read(&resolved).map_err(|_| PersonalNoteError::CompensationFailed)?;
    if sha256_hex(&current) != file.content_digest {
        return Err(PersonalNoteError::CompensationFailed);
    }
    std::fs::remove_file(resolved).map_err(|_| PersonalNoteError::CompensationFailed)
}

impl ContestImportPort for DatabaseRuntime {
    async fn persist_manifest(
        &self,
        draft: &ContestImportDraft,
    ) -> Result<PersistedContestImport, ContestImportPersistenceError> {
        let pool = self.contest_pool()?;
        let mut transaction = pool.begin().await.map_err(|_| ContestImportPersistenceError::Unavailable)?;
        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM contests WHERE platform = 'codeforces' AND external_contest_key = ?1",
        )
        .bind(draft.contest.contest_id() as i64)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        let contest_id = match existing {
            Some(id) => {
                let persisted_slots: Vec<(i64, String)> = sqlx::query_as(
                    "SELECT p.external_contest_key, p.external_problem_key FROM contest_problems cp \
                     JOIN problems p ON p.id = cp.problem_id WHERE cp.contest_id = ?1 ORDER BY cp.ordinal",
                )
                .bind(id)
                .fetch_all(&mut *transaction)
                .await
                .map_err(|_| ContestImportPersistenceError::Unavailable)?;
                let incoming_slots: Vec<(i64, String)> = draft
                    .slots
                    .iter()
                    .map(|slot| (slot.problem.contest().contest_id() as i64, slot.problem.index().to_owned()))
                    .collect();
                if persisted_slots != incoming_slots {
                    return Err(ContestImportPersistenceError::ManifestConflict);
                }
                id
            }
            None => {
                let result = sqlx::query(
                    "INSERT INTO contests (platform, external_contest_key, title, source_url, starts_at_utc, import_status) \
                     VALUES ('codeforces', ?1, ?2, ?3, ?4, 'incomplete')",
                )
                .bind(draft.contest.contest_id() as i64)
                .bind(&draft.title)
                .bind(&draft.source_url)
                .bind(&draft.starts_at_utc)
                .execute(&mut *transaction)
                .await
                .map_err(|_| ContestImportPersistenceError::Unavailable)?;
                let id = result.last_insert_rowid();
                for slot in &draft.slots {
                    sqlx::query(
                        "INSERT INTO problems (platform, external_contest_key, external_problem_key, title, rating, source_url) \
                         VALUES ('codeforces', ?1, ?2, ?3, ?4, ?5) \
                         ON CONFLICT(platform, external_contest_key, external_problem_key) DO NOTHING",
                    )
                    .bind(slot.problem.contest().contest_id() as i64)
                    .bind(slot.problem.index())
                    .bind(&slot.title)
                    .bind(slot.rating.map(i64::from))
                    .bind(&slot.source_url)
                    .execute(&mut *transaction)
                    .await
                    .map_err(|_| ContestImportPersistenceError::Unavailable)?;
                    let problem_id: i64 = sqlx::query_scalar(
                        "SELECT id FROM problems WHERE platform = 'codeforces' AND external_contest_key = ?1 AND external_problem_key = ?2",
                    )
                    .bind(slot.problem.contest().contest_id() as i64)
                    .bind(slot.problem.index())
                    .fetch_one(&mut *transaction)
                    .await
                    .map_err(|_| ContestImportPersistenceError::Unavailable)?;
                    sqlx::query(
                        "INSERT INTO contest_problems (contest_id, problem_id, ordinal, import_state) VALUES (?1, ?2, ?3, 'pending_snapshot')",
                    )
                    .bind(id)
                    .bind(problem_id)
                    .bind(i64::from(slot.ordinal))
                    .execute(&mut *transaction)
                    .await
                    .map_err(|_| ContestImportPersistenceError::Unavailable)?;
                }
                id
            }
        };
        transaction.commit().await.map_err(|_| ContestImportPersistenceError::Unavailable)?;
        self.import_state(contest_id).await
    }

    async fn persist_first_snapshot(
        &self,
        snapshot: &StatementSnapshotDraft,
    ) -> Result<PersistedContestImport, ContestImportPersistenceError> {
        let pool = self.contest_pool()?;
        let problem_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM problems WHERE platform = 'codeforces' AND external_contest_key = ?1 AND external_problem_key = ?2",
        )
        .bind(snapshot.problem.contest().contest_id() as i64)
        .bind(snapshot.problem.index())
        .fetch_optional(pool)
        .await
        .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        let problem_id = problem_id.ok_or(ContestImportPersistenceError::ManifestConflict)?;
        sqlx::query(
            "INSERT INTO problem_statement_snapshots (problem_id, source_html, sanitized_html) VALUES (?1, ?2, ?3) \
             ON CONFLICT(problem_id) DO NOTHING",
        )
        .bind(problem_id)
        .bind(&snapshot.source_html)
        .bind(&snapshot.sanitized_html)
        .execute(pool)
        .await
        .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        for asset in &snapshot.assets {
            sqlx::query(
                "INSERT INTO problem_statement_assets (problem_id, local_ref, media_type, bytes) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(problem_id, local_ref) DO NOTHING",
            )
            .bind(problem_id)
            .bind(&asset.local_ref)
            .bind(&asset.media_type)
            .bind(&asset.bytes)
            .execute(pool)
            .await
            .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        }
        sqlx::query(
            "UPDATE contest_problems SET import_state = 'ready' WHERE problem_id = ?1 AND EXISTS (SELECT 1 FROM problem_statement_snapshots WHERE problem_id = ?1)",
        )
        .bind(problem_id)
        .execute(pool)
        .await
        .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        let contest_id: i64 = sqlx::query_scalar(
            "SELECT contest_id FROM contest_problems WHERE problem_id = ?1 ORDER BY contest_id LIMIT 1",
        )
        .bind(problem_id)
        .fetch_one(pool)
        .await
        .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        self.import_state(contest_id).await
    }
}

impl ContestReadPort for DatabaseRuntime {
    async fn list_contests(&self) -> Result<Vec<ContestShelfItem>, ContestReadError> {
        let rows: Vec<(i64, String, String, i64, i64)> = sqlx::query_as(
            "SELECT c.external_contest_key, c.title, c.import_status, COUNT(cp.problem_id), \
                    SUM(CASE WHEN cp.import_state = 'pending_snapshot' THEN 1 ELSE 0 END) \
             FROM contests c JOIN contest_problems cp ON cp.contest_id = c.id \
             GROUP BY c.id ORDER BY c.created_at_utc DESC",
        )
        .fetch_all(self.contest_pool().map_err(|_| ContestReadError::Unavailable)?)
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        rows.into_iter().map(|(id, title, status, count, missing)| {
            Ok(ContestShelfItem {
                contest: acm_os_domain::CodeforcesContestIdentity::new(id as u64).map_err(|_| ContestReadError::Unavailable)?,
                title,
                import_status: match status.as_str() {
                    "incomplete" => ContestImportStatus::Incomplete,
                    "complete" => ContestImportStatus::Complete,
                    _ => return Err(ContestReadError::Unavailable),
                },
                problem_count: count as u32,
                missing_snapshot_count: missing as u32,
            })
        }).collect()
    }

    async fn contest_detail(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
    ) -> Result<ContestDetail, ContestReadError> {
        let pool = self.contest_pool().map_err(|_| ContestReadError::Unavailable)?;
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT title, source_url, import_status FROM contests WHERE platform = 'codeforces' AND external_contest_key = ?1",
        )
        .bind(contest.contest_id() as i64)
        .fetch_optional(pool)
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        let (title, source_url, import_status) = row.ok_or(ContestReadError::NotFound)?;
        let rows: Vec<(String, String, Option<i64>, i64, String)> = sqlx::query_as(
            "SELECT p.external_problem_key, p.title, p.rating, EXISTS(SELECT 1 FROM problem_statement_snapshots ss WHERE ss.problem_id = p.id), p.identity_type FROM contest_problems cp JOIN problems p ON p.id = cp.problem_id JOIN contests c ON c.id = cp.contest_id WHERE c.platform = 'codeforces' AND c.external_contest_key = ?1 ORDER BY cp.ordinal",
        )
        .bind(contest.contest_id() as i64)
        .fetch_all(pool)
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        let problems = rows.into_iter().map(|(index, title, rating, snapshot, identity_type)| {
            Ok(LightweightProblemItem {
                problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), index)
                    .map_err(|_| ContestReadError::Unavailable)?,
                title,
                rating: rating.map(|value| value as u32),
                has_statement_snapshot: snapshot != 0,
                identity_type: parse_problem_identity_type(&identity_type)?,
            })
        }).collect::<Result<Vec<_>, _>>()?;
        Ok(ContestDetail {
            contest: contest.clone(),
            title,
            source_url,
            import_status: match import_status.as_str() {
                "incomplete" => ContestImportStatus::Incomplete,
                "complete" => ContestImportStatus::Complete,
                _ => return Err(ContestReadError::Unavailable),
            },
            problems,
        })
    }

    async fn list_lightweight_problems(&self) -> Result<Vec<LightweightProblemItem>, ContestReadError> {
        let rows: Vec<(i64, String, String, Option<i64>, i64, String)> = sqlx::query_as(
            "SELECT p.external_contest_key, p.external_problem_key, p.title, p.rating, \
                    EXISTS(SELECT 1 FROM problem_statement_snapshots ss WHERE ss.problem_id = p.id), \
                    p.identity_type \
             FROM problems p ORDER BY p.external_contest_key DESC, p.external_problem_key ASC",
        )
        .fetch_all(self.contest_pool().map_err(|_| ContestReadError::Unavailable)?)
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        rows.into_iter().map(|(contest_id, index, title, rating, snapshot, identity_type)| {
            Ok(LightweightProblemItem {
                problem: acm_os_domain::CodeforcesProblemIdentity::new(
                    acm_os_domain::CodeforcesContestIdentity::new(contest_id as u64).map_err(|_| ContestReadError::Unavailable)?, index,
                ).map_err(|_| ContestReadError::Unavailable)?,
                title,
                rating: rating.map(|value| value as u32),
                has_statement_snapshot: snapshot != 0,
                identity_type: parse_problem_identity_type(&identity_type)?,
            })
        }).collect()
    }

    async fn lightweight_problem_detail(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<LightweightProblemDetail, ContestReadError> {
        let row: Option<(String, Option<i64>, String, Option<String>, String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT p.title, p.rating, p.source_url, ss.sanitized_html, p.identity_type, \
                    fb.vault_relative_path, fb.content_digest, fb.windows_file_key \
             FROM problems p \
             LEFT JOIN problem_statement_snapshots ss ON ss.problem_id = p.id \
             LEFT JOIN file_bindings fb ON fb.problem_id = p.id \
             WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 AND p.external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(self.contest_pool().map_err(|_| ContestReadError::Unavailable)?)
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        let (title, rating, source_url, sanitized_html, identity_type, relative_path, digest, file_key) = row.ok_or(ContestReadError::NotFound)?;
        let personal_note = match (relative_path, digest) {
            (Some(vault_relative_path), Some(content_digest)) => Some(PersonalNoteBinding {
                vault_relative_path,
                content_digest,
                windows_file_key: file_key,
            }),
            (None, None) => None,
            _ => return Err(ContestReadError::Unavailable),
        };
        let identity_type = parse_problem_identity_type(&identity_type)?;
        if (identity_type == ProblemIdentityType::Personal) != personal_note.is_some() {
            return Err(ContestReadError::Unavailable);
        }
        Ok(LightweightProblemDetail {
            problem: problem.clone(),
            title,
            rating: rating.map(|value| value as u32),
            source_url,
            statement: match sanitized_html {
                Some(sanitized_html) => StatementReadState::Ready { sanitized_html },
                None => StatementReadState::Pending,
            },
            identity_type,
            personal_note,
        })
    }

    async fn statement_assets(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<Vec<LocalStatementAsset>, ContestReadError> {
        let rows: Vec<(String, String, Vec<u8>)> = sqlx::query_as(
            "SELECT a.local_ref, a.media_type, a.bytes FROM problem_statement_assets a JOIN problems p ON p.id = a.problem_id WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 AND p.external_problem_key = ?2 ORDER BY a.local_ref",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_all(self.contest_pool().map_err(|_| ContestReadError::Unavailable)?)
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        Ok(rows.into_iter().map(|(local_ref, media_type, bytes)| LocalStatementAsset {
            local_ref,
            media_type,
            bytes,
        }).collect())
    }
}

impl DatabaseRuntime {
    fn personal_note_pool(&self) -> Result<&SqlitePool, PersonalNoteError> {
        self._pool
            .as_ref()
            .ok_or(PersonalNoteError::PersistenceUnavailable)
    }

    fn contest_pool(&self) -> Result<&SqlitePool, ContestImportPersistenceError> {
        self._pool.as_ref().ok_or(ContestImportPersistenceError::Unavailable)
    }

    async fn import_state(&self, contest_id: i64) -> Result<PersistedContestImport, ContestImportPersistenceError> {
        let pool = self.contest_pool()?;
        let missing: Vec<(i64, String)> = sqlx::query_as(
            "SELECT p.external_contest_key, p.external_problem_key FROM contest_problems cp \
             JOIN problems p ON p.id = cp.problem_id LEFT JOIN problem_statement_snapshots ss ON ss.problem_id = p.id \
             WHERE cp.contest_id = ?1 AND ss.problem_id IS NULL ORDER BY cp.ordinal",
        )
        .bind(contest_id)
        .fetch_all(pool)
        .await
        .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        let missing_snapshot_problems = missing.into_iter().map(|(contest_id, index)| {
            acm_os_domain::CodeforcesProblemIdentity::new(
                acm_os_domain::CodeforcesContestIdentity::new(contest_id as u64)
                    .map_err(|_| ContestImportPersistenceError::Unavailable)?,
                index,
            ).map_err(|_| ContestImportPersistenceError::Unavailable)
        }).collect::<Result<Vec<_>, _>>()?;
        let status = if missing_snapshot_problems.is_empty() { ContestImportStatus::Complete } else { ContestImportStatus::Incomplete };
        sqlx::query("UPDATE contests SET import_status = ?1 WHERE id = ?2")
            .bind(match status { ContestImportStatus::Incomplete => "incomplete", ContestImportStatus::Complete => "complete" })
            .bind(contest_id).execute(pool).await.map_err(|_| ContestImportPersistenceError::Unavailable)?;
        Ok(PersistedContestImport { status, missing_snapshot_problems })
    }
}

fn parse_problem_identity_type(value: &str) -> Result<ProblemIdentityType, ContestReadError> {
    match value {
        "lightweight" => Ok(ProblemIdentityType::Lightweight),
        "personal" => Ok(ProblemIdentityType::Personal),
        _ => Err(ContestReadError::Unavailable),
    }
}

pub async fn start_database(app_private_data: &Path) -> DatabaseRuntime {
    match try_start_database(app_private_data).await {
        Ok(runtime) => runtime,
        Err(reason) => DatabaseRuntime::recovery(reason),
    }
}

async fn try_start_database(
    app_private_data: &Path,
) -> Result<DatabaseRuntime, StartupRecoveryReason> {
    std::fs::create_dir_all(app_private_data)
        .map_err(|_| StartupRecoveryReason::AppDataUnavailable)?;
    let startup_lock = acquire_startup_lock(app_private_data, STARTUP_LOCK_TIMEOUT).await?;

    let database_path = app_private_data.join(DATABASE_FILENAME);
    let database_exists = database_path
        .try_exists()
        .map_err(|_| StartupRecoveryReason::DatabaseUnavailable)?;
    let supported_schema_version = supported_schema_version();

    let existing_schema_version = if database_exists {
        let inspection_pool = connect_read_only(&database_path).await?;
        verify_integrity(&inspection_pool).await?;
        let version = inspect_schema_version(&inspection_pool).await?;
        if version <= supported_schema_version {
            validate_schema_contract(&inspection_pool, version).await?;
        }
        inspection_pool.close().await;
        version
    } else {
        0
    };

    if existing_schema_version > supported_schema_version {
        return Err(StartupRecoveryReason::UnsupportedSchema {
            found: existing_schema_version,
            supported: supported_schema_version,
        });
    }

    let pool = connect_read_write(&database_path).await?;

    let migration_pending = existing_schema_version < supported_schema_version;
    if database_exists && migration_pending {
        create_pre_migration_backup(
            &pool,
            app_private_data,
            existing_schema_version,
            supported_schema_version,
        )
        .await?;
    }

    MIGRATOR
        .run(&pool)
        .await
        .map_err(|_| StartupRecoveryReason::MigrationFailed)?;
    if !database_exists || migration_pending {
        verify_integrity(&pool).await?;
    }

    let applied_schema_version = inspect_schema_version(&pool).await?;
    if applied_schema_version != supported_schema_version {
        return Err(StartupRecoveryReason::MigrationFailed);
    }
    validate_schema_contract(&pool, applied_schema_version).await?;

    Ok(DatabaseRuntime {
        _pool: Some(pool),
        _startup_lock: Some(startup_lock),
        status: StartupGateStatus::Ready {
            schema_version: applied_schema_version,
        },
        markdown_projection_cache: Mutex::new(HashMap::new()),
    })
}

async fn acquire_startup_lock(
    app_private_data: &Path,
    timeout: Duration,
) -> Result<File, StartupRecoveryReason> {
    let lock_path = app_private_data.join(STARTUP_LOCK_FILENAME);
    tokio::task::spawn_blocking(move || {
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|_| StartupRecoveryReason::DatabaseUnavailable)?;
        let deadline = Instant::now() + timeout;

        loop {
            match lock.try_lock() {
                Ok(()) => return Ok(lock),
                Err(TryLockError::WouldBlock) => {}
                Err(TryLockError::Error(_)) => {
                    return Err(StartupRecoveryReason::DatabaseUnavailable);
                }
            }

            if Instant::now() >= deadline {
                return Err(StartupRecoveryReason::DatabaseUnavailable);
            }
            thread::sleep(STARTUP_LOCK_RETRY_INTERVAL);
        }
    })
    .await
    .map_err(|_| StartupRecoveryReason::DatabaseUnavailable)?
}

fn supported_schema_version() -> i64 {
    MIGRATOR
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0)
}

async fn connect_read_only(path: &Path) -> Result<SqlitePool, StartupRecoveryReason> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| StartupRecoveryReason::DatabaseUnavailable)
}

async fn connect_read_write(path: &Path) -> Result<SqlitePool, StartupRecoveryReason> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| StartupRecoveryReason::DatabaseUnavailable)
}

async fn inspect_schema_version(pool: &SqlitePool) -> Result<i64, StartupRecoveryReason> {
    let ledger_exists: i64 = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations')",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::MigrationLedgerInvalid)?;

    if ledger_exists == 0 {
        let unexpected_tables: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )
        .fetch_one(pool)
        .await
        .map_err(|_| StartupRecoveryReason::MigrationLedgerInvalid)?;

        return if unexpected_tables == 0 {
            Ok(0)
        } else {
            Err(StartupRecoveryReason::MigrationLedgerInvalid)
        };
    }

    validate_migration_ledger_contract(pool).await?;

    sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(|_| StartupRecoveryReason::MigrationLedgerInvalid)
}

async fn validate_schema_contract(
    pool: &SqlitePool,
    schema_version: i64,
) -> Result<(), StartupRecoveryReason> {
    let objects: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT type, name, tbl_name FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;

    if schema_version == 0 {
        let valid = objects.is_empty()
            || objects
                == vec![(
                    "table".to_owned(),
                    "_sqlx_migrations".to_owned(),
                    "_sqlx_migrations".to_owned(),
                )];
        return if valid {
            Ok(())
        } else {
            Err(StartupRecoveryReason::MigrationLedgerInvalid)
        };
    }

    if !matches!(schema_version, 1 | 2 | 3 | 4) {
        return Err(StartupRecoveryReason::UnsupportedSchema {
            found: schema_version,
            supported: supported_schema_version(),
        });
    }

    let mut expected_objects = vec![
        (
            "table".to_owned(),
            "_sqlx_migrations".to_owned(),
            "_sqlx_migrations".to_owned(),
        ),
        (
            "table".to_owned(),
            "app_metadata".to_owned(),
            "app_metadata".to_owned(),
        ),
    ];
    if schema_version >= 2 {
        expected_objects.push((
            "table".to_owned(),
            "workspace_settings".to_owned(),
            "workspace_settings".to_owned(),
        ));
    }
    if schema_version >= 3 {
        expected_objects.extend([
            (
                "table".to_owned(),
                "contest_problems".to_owned(),
                "contest_problems".to_owned(),
            ),
            (
                "table".to_owned(),
                "contests".to_owned(),
                "contests".to_owned(),
            ),
            (
                "table".to_owned(),
                "problem_statement_snapshots".to_owned(),
                "problem_statement_snapshots".to_owned(),
            ),
            (
                "table".to_owned(),
                "problem_statement_assets".to_owned(),
                "problem_statement_assets".to_owned(),
            ),
            (
                "table".to_owned(),
                "problems".to_owned(),
                "problems".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    if schema_version >= 4 {
        expected_objects.push((
            "table".to_owned(),
            "file_bindings".to_owned(),
            "file_bindings".to_owned(),
        ));
        expected_objects.sort();
    }
    if objects != expected_objects {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    validate_app_metadata_columns(pool).await?;

    let table_sql: String = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'app_metadata'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?
    .ok_or(StartupRecoveryReason::IntegrityCheckFailed)?;
    const EXPECTED_APP_METADATA_SQL: &str = "\
        CREATE TABLE app_metadata (\
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),\
            schema_generation INTEGER NOT NULL CHECK (schema_generation > 0),\
            created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
        )";
    if normalize_schema_sql(&table_sql) != normalize_schema_sql(EXPECTED_APP_METADATA_SQL) {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    let metadata: Vec<(i64, i64, String)> = sqlx::query_as(
        "SELECT singleton, schema_generation, created_at_utc FROM app_metadata",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if metadata.len() != 1
        || metadata[0].0 != 1
        || metadata[0].1 != schema_version
        || metadata[0].2.is_empty()
    {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    if schema_version >= 2 {
        validate_workspace_settings_contract(pool).await?;
    }
    if schema_version >= 3 {
        validate_contest_import_contract(pool, schema_version).await?;
    }
    if schema_version >= 4 {
        validate_personal_note_contract(pool).await?;
    }

    Ok(())
}

async fn validate_contest_import_contract(
    pool: &SqlitePool,
    schema_version: i64,
) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "contests",
        &["id", "platform", "external_contest_key", "title", "source_url", "starts_at_utc", "import_status", "created_at_utc"],
    )
    .await?;
    let problem_columns = if schema_version >= 4 {
        vec!["id", "platform", "external_contest_key", "external_problem_key", "title", "rating", "source_url", "created_at_utc", "identity_type"]
    } else {
        vec!["id", "platform", "external_contest_key", "external_problem_key", "title", "rating", "source_url", "created_at_utc"]
    };
    validate_table_columns(pool, "problems", &problem_columns).await?;
    validate_table_columns(pool, "contest_problems", &["contest_id", "problem_id", "ordinal", "import_state"]).await?;
    validate_table_columns(pool, "problem_statement_snapshots", &["problem_id", "source_html", "sanitized_html", "captured_at_utc"]).await?;
    validate_table_columns(pool, "problem_statement_assets", &["problem_id", "local_ref", "media_type", "bytes"]).await
}

async fn validate_personal_note_contract(
    pool: &SqlitePool,
) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "file_bindings",
        &[
            "id",
            "problem_id",
            "vault_relative_path",
            "windows_file_key",
            "content_digest",
            "binding_state",
            "created_at_utc",
            "updated_at_utc",
        ],
    )
    .await
}

async fn validate_table_columns(
    pool: &SqlitePool,
    table: &str,
    expected_names: &[&str],
) -> Result<(), StartupRecoveryReason> {
    let sql = match table {
        "contests" => "PRAGMA table_xinfo('contests')",
        "problems" => "PRAGMA table_xinfo('problems')",
        "contest_problems" => "PRAGMA table_xinfo('contest_problems')",
        "problem_statement_snapshots" => "PRAGMA table_xinfo('problem_statement_snapshots')",
        "problem_statement_assets" => "PRAGMA table_xinfo('problem_statement_assets')",
        "file_bindings" => "PRAGMA table_xinfo('file_bindings')",
        _ => return Err(StartupRecoveryReason::IntegrityCheckFailed),
    };
    let actual: Vec<SqliteColumnContract> = sqlx::query_as(sql)
        .fetch_all(pool)
        .await
        .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    let actual_names: Vec<&str> = actual.iter().map(|column| column.1.as_str()).collect();
    if actual_names == expected_names {
        Ok(())
    } else {
        Err(StartupRecoveryReason::IntegrityCheckFailed)
    }
}

async fn validate_migration_ledger_contract(
    pool: &SqlitePool,
) -> Result<(), StartupRecoveryReason> {
    let actual: Vec<SqliteColumnContract> =
        sqlx::query_as("PRAGMA table_xinfo('_sqlx_migrations')")
            .fetch_all(pool)
            .await
            .map_err(|_| StartupRecoveryReason::MigrationLedgerInvalid)?;
    let expected = vec![
        (0, "version".to_owned(), "BIGINT".to_owned(), 0, None, 1, 0),
        (1, "description".to_owned(), "TEXT".to_owned(), 1, None, 0, 0),
        (
            2,
            "installed_on".to_owned(),
            "TIMESTAMP".to_owned(),
            1,
            Some("CURRENT_TIMESTAMP".to_owned()),
            0,
            0,
        ),
        (3, "success".to_owned(), "BOOLEAN".to_owned(), 1, None, 0, 0),
        (4, "checksum".to_owned(), "BLOB".to_owned(), 1, None, 0, 0),
        (
            5,
            "execution_time".to_owned(),
            "BIGINT".to_owned(),
            1,
            None,
            0,
            0,
        ),
    ];

    if actual != expected {
        return Err(StartupRecoveryReason::MigrationLedgerInvalid);
    }

    let table_sql: String = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| StartupRecoveryReason::MigrationLedgerInvalid)?
    .ok_or(StartupRecoveryReason::MigrationLedgerInvalid)?;
    const EXPECTED_MIGRATION_LEDGER_SQL: &str = "\
        CREATE TABLE _sqlx_migrations (\
            version BIGINT PRIMARY KEY,\
            description TEXT NOT NULL,\
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,\
            success BOOLEAN NOT NULL,\
            checksum BLOB NOT NULL,\
            execution_time BIGINT NOT NULL\
        )";
    if normalize_schema_sql(&table_sql) != normalize_schema_sql(EXPECTED_MIGRATION_LEDGER_SQL) {
        return Err(StartupRecoveryReason::MigrationLedgerInvalid);
    }

    Ok(())
}

fn normalize_schema_sql(sql: &str) -> String {
    sql.chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

async fn validate_app_metadata_columns(
    pool: &SqlitePool,
) -> Result<(), StartupRecoveryReason> {
    let actual: Vec<SqliteColumnContract> = sqlx::query_as("PRAGMA table_xinfo('app_metadata')")
        .fetch_all(pool)
        .await
        .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    let expected = vec![
        (0, "singleton".to_owned(), "INTEGER".to_owned(), 0, None, 1, 0),
        (
            1,
            "schema_generation".to_owned(),
            "INTEGER".to_owned(),
            1,
            None,
            0,
            0,
        ),
        (
            2,
            "created_at_utc".to_owned(),
            "TEXT".to_owned(),
            1,
            Some("strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_owned()),
            0,
            0,
        ),
    ];

    if actual == expected {
        Ok(())
    } else {
        Err(StartupRecoveryReason::IntegrityCheckFailed)
    }
}

async fn validate_workspace_settings_contract(
    pool: &SqlitePool,
) -> Result<(), StartupRecoveryReason> {
    let actual: Vec<SqliteColumnContract> =
        sqlx::query_as("PRAGMA table_xinfo('workspace_settings')")
            .fetch_all(pool)
            .await
            .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    let expected = vec![
        (0, "singleton".to_owned(), "INTEGER".to_owned(), 0, None, 1, 0),
        (
            1,
            "active_vault_path".to_owned(),
            "TEXT".to_owned(),
            1,
            None,
            0,
            0,
        ),
        (
            2,
            "problem_root_path".to_owned(),
            "TEXT".to_owned(),
            1,
            None,
            0,
            0,
        ),
        (
            3,
            "knowledge_root_path".to_owned(),
            "TEXT".to_owned(),
            1,
            None,
            0,
            0,
        ),
        (
            4,
            "updated_at_utc".to_owned(),
            "TEXT".to_owned(),
            1,
            Some("strftime('%Y-%m-%dT%H:%M:%fZ', 'now')".to_owned()),
            0,
            0,
        ),
    ];
    if actual != expected {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    let table_sql: String = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'workspace_settings'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?
    .ok_or(StartupRecoveryReason::IntegrityCheckFailed)?;
    const EXPECTED_WORKSPACE_SETTINGS_SQL: &str = "\
        CREATE TABLE workspace_settings (\
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),\
            active_vault_path TEXT NOT NULL CHECK (length(active_vault_path) > 0),\
            problem_root_path TEXT NOT NULL CHECK (length(problem_root_path) > 0),\
            knowledge_root_path TEXT NOT NULL CHECK (length(knowledge_root_path) > 0),\
            updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
        )";
    if normalize_schema_sql(&table_sql) != normalize_schema_sql(EXPECTED_WORKSPACE_SETTINGS_SQL) {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    let rows: Vec<(i64, String, String, String, String)> = sqlx::query_as(
        "SELECT singleton, active_vault_path, problem_root_path, knowledge_root_path, \
                updated_at_utc FROM workspace_settings",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if rows.len() > 1
        || rows.first().is_some_and(|row| {
            row.0 != 1
                || row.1.is_empty()
                || row.2.is_empty()
                || row.3.is_empty()
                || row.4.is_empty()
        })
    {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }
    if let Some(row) = rows.first() {
        WorkspaceConfiguration::from_resolved(row.1.clone(), row.2.clone(), row.3.clone())
            .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    }

    Ok(())
}

async fn verify_integrity(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
    let results: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_all(pool)
        .await
        .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if results.len() != 1 || results[0] != "ok" {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    let foreign_key_violation = sqlx::query("PRAGMA foreign_key_check")
        .fetch_optional(pool)
        .await
        .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if foreign_key_violation.is_some() {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    Ok(())
}

async fn create_pre_migration_backup(
    pool: &SqlitePool,
    app_private_data: &Path,
    current_version: i64,
    target_version: i64,
) -> Result<PathBuf, StartupRecoveryReason> {
    let backup_directory = app_private_data.join("backups").join("pre-migration");
    std::fs::create_dir_all(&backup_directory)
        .map_err(|_| StartupRecoveryReason::PreMigrationBackupFailed)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StartupRecoveryReason::PreMigrationBackupFailed)?
        .as_nanos();
    let backup_path = backup_directory.join(format!(
        "schema-{current_version}-to-{target_version}-{timestamp}.sqlite3"
    ));
    let mut partial_path = backup_path.as_os_str().to_os_string();
    partial_path.push(".partial");
    let partial_path = PathBuf::from(partial_path);
    let partial_filename = partial_path.to_string_lossy().into_owned();

    sqlx::query("VACUUM INTO ?1")
        .bind(partial_filename)
        .execute(pool)
        .await
        .map_err(|_| {
            let _ = std::fs::remove_file(&partial_path);
            StartupRecoveryReason::PreMigrationBackupFailed
        })?;

    verify_and_publish_backup(&partial_path, &backup_path).await?;

    Ok(backup_path)
}

async fn verify_and_publish_backup(
    partial_path: &Path,
    backup_path: &Path,
) -> Result<(), StartupRecoveryReason> {
    let verification_pool = match connect_read_only(partial_path).await {
        Ok(pool) => pool,
        Err(_) => {
            let _ = std::fs::remove_file(partial_path);
            return Err(StartupRecoveryReason::PreMigrationBackupFailed);
        }
    };
    let verification_result = verify_integrity(&verification_pool).await;
    verification_pool.close().await;
    if verification_result.is_err() {
        let _ = std::fs::remove_file(partial_path);
        return Err(StartupRecoveryReason::PreMigrationBackupFailed);
    }

    std::fs::rename(partial_path, backup_path).map_err(|_| {
        let _ = std::fs::remove_file(partial_path);
        StartupRecoveryReason::PreMigrationBackupFailed
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use acm_os_application::{
        configure_workspace, create_personal_note, import_codeforces_contest,
        query_workspace_configuration, ContestImportDraft, ContestImportPort, ContestReadPort,
        ContestImportStatus, ContestProblemSlotDraft, StartupGateStatus, StatementAssetDraft, StatementSnapshotDraft,
        StartupRecoveryReason, WorkspaceConfigurationDraft, WorkspaceConfigurationError,
        WorkspaceConfigurationStatus, WorkspacePathField, PersonalNoteError, PersonalNoteReadPort,
        PersonalNoteReadState,
        ProblemIdentityType, INITIAL_PROBLEM_MARKDOWN,
    };
    use sqlx::Executor;
    use tempfile::TempDir;

    use super::*;

    fn contest_draft() -> ContestImportDraft {
        let contest = acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest");
        ContestImportDraft::validated(
            contest.clone(),
            "Codeforces Round".to_owned(),
            "https://codeforces.com/contest/1979".to_owned(),
            None,
            ["A", "B"]
                .into_iter()
                .enumerate()
                .map(|(position, index)| ContestProblemSlotDraft {
                    ordinal: position as u32 + 1,
                    problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), index)
                        .expect("problem identity"),
                    title: format!("Problem {index}"),
                    rating: Some(800 + position as u32 * 100),
                    source_url: format!("https://codeforces.com/contest/1979/problem/{index}"),
                })
                .collect(),
        )
        .expect("valid manifest")
    }

    fn snapshot(index: &str, source: &str, sanitized: &str) -> StatementSnapshotDraft {
        StatementSnapshotDraft {
            problem: acm_os_domain::CodeforcesProblemIdentity::new(
                acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
                index,
            )
            .expect("problem identity"),
            source_html: source.to_owned(),
            sanitized_html: sanitized.to_owned(),
            assets: Vec::new(),
        }
    }

    fn snapshot_with_asset(index: &str) -> StatementSnapshotDraft {
        let mut snapshot = snapshot(index, "<img src=\"acm-os-asset://fixture\">", "<img src=\"acm-os-asset://fixture\">");
        snapshot.assets.push(StatementAssetDraft {
            local_ref: "acm-os-asset://fixture".to_owned(),
            media_type: "image/png".to_owned(),
            bytes: vec![1, 2, 3],
        });
        snapshot
    }

    async fn configure_temporary_workspace(
        runtime: &DatabaseRuntime,
        directory: &TempDir,
    ) -> (PathBuf, PathBuf, PathBuf) {
        let vault = directory.path().join("vault");
        let problems = vault.join("Problems");
        let knowledge = vault.join("Knowledge");
        fs::create_dir_all(&problems).expect("problem root");
        fs::create_dir_all(&knowledge).expect("knowledge root");
        configure_workspace(
            runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: vault.to_string_lossy().into_owned(),
                problem_root_path: problems.to_string_lossy().into_owned(),
                knowledge_root_path: knowledge.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect("configure temporary workspace");
        (vault, problems, knowledge)
    }

    async fn personal_note_fixture() -> (
        TempDir,
        DatabaseRuntime,
        PathBuf,
        PathBuf,
        acm_os_domain::CodeforcesProblemIdentity,
    ) {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (vault, problems, _knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        runtime
            .persist_manifest(&contest_draft())
            .await
            .expect("persist manifest");
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");
        create_personal_note(&runtime, &problem)
            .await
            .expect("create personal note");
        (directory, runtime, vault, problems, problem)
    }

    async fn create_empty_migration_ledger(pool: &SqlitePool) {
        pool.execute(
            "CREATE TABLE _sqlx_migrations (\
                version BIGINT PRIMARY KEY, \
                description TEXT NOT NULL, \
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, \
                success BOOLEAN NOT NULL, \
                checksum BLOB NOT NULL, \
                execution_time BIGINT NOT NULL\
            )",
        )
        .await
        .expect("create migration ledger");
    }

    async fn create_version_one_database(pool: &SqlitePool) {
        create_empty_migration_ledger(pool).await;
        pool.execute(
            "CREATE TABLE app_metadata (\
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1), \
                schema_generation INTEGER NOT NULL CHECK (schema_generation > 0), \
                created_at_utc TEXT NOT NULL \
                    DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
            )",
        )
        .await
        .expect("create version one metadata");
        pool.execute(
            "INSERT INTO app_metadata (singleton, schema_generation) VALUES (1, 1)",
        )
        .await
        .expect("insert version one metadata");

        let migration = MIGRATOR
            .iter()
            .find(|migration| migration.version == 1)
            .expect("version one migration");
        sqlx::query(
            "INSERT INTO _sqlx_migrations \
                (version, description, success, checksum, execution_time) \
             VALUES (?1, ?2, 1, ?3, 0)",
        )
        .bind(migration.version)
        .bind(migration.description.as_ref())
        .bind(migration.checksum.as_ref())
        .execute(pool)
        .await
        .expect("record version one migration");
    }

    #[tokio::test]
    async fn new_database_migrates_and_passes_integrity() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;

        assert_eq!(
            runtime.status(),
            &StartupGateStatus::Ready { schema_version: 4 }
        );
        let pool = runtime._pool.as_ref().expect("ready database pool");
        let ledger_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(pool)
            .await
            .expect("migration ledger");
        assert_eq!(ledger_count, 4);
        verify_integrity(pool).await.expect("database integrity");
    }

    #[tokio::test]
    async fn contest_import_is_progressive_idempotent_and_preserves_first_snapshot() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let draft = contest_draft();

        let initial = runtime.persist_manifest(&draft).await.expect("persist manifest");
        assert_eq!(initial.status, ContestImportStatus::Incomplete);
        assert_eq!(initial.missing_snapshot_problems.len(), 2);

        let after_a = runtime
            .persist_first_snapshot(&snapshot_with_asset("A"))
            .await
            .expect("persist first snapshot");
        assert_eq!(after_a.status, ContestImportStatus::Incomplete);
        assert_eq!(after_a.missing_snapshot_problems.len(), 1);
        assert_eq!(after_a.missing_snapshot_problems[0].index(), "B");

        // A duplicate manifest is the existing manifest fast path: no objects
        // are copied and it retains the known missing slot.
        let duplicate = runtime.persist_manifest(&draft).await.expect("duplicate manifest");
        assert_eq!(duplicate, after_a);

        let complete = runtime
            .persist_first_snapshot(&snapshot("B", "<p>first B</p>", "<p>first B</p>"))
            .await
            .expect("retry missing snapshot");
        assert_eq!(complete.status, ContestImportStatus::Complete);
        assert!(complete.missing_snapshot_problems.is_empty());

        // Re-importing a first snapshot must be a no-overwrite operation.
        let after_reimport = runtime
            .persist_first_snapshot(&snapshot("A", "<p>later A</p>", "<p>later A</p>"))
            .await
            .expect("idempotent snapshot");
        assert_eq!(after_reimport.status, ContestImportStatus::Complete);

        let pool = runtime._pool.as_ref().expect("ready database pool");
        let counts: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM contests), (SELECT COUNT(*) FROM problems), \
                    (SELECT COUNT(*) FROM contest_problems), (SELECT COUNT(*) FROM problem_statement_snapshots)",
        )
        .fetch_one(pool)
        .await
        .expect("import counts");
        assert_eq!(counts, (1, 2, 2, 2));
        let asset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM problem_statement_assets")
            .fetch_one(pool)
            .await
            .expect("stored localized asset");
        assert_eq!(asset_count, 1);
        let asset_problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("asset contest"),
            "A",
        ).expect("asset problem");
        let assets = runtime.statement_assets(&asset_problem).await.expect("read localized assets");
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].local_ref, "acm-os-asset://fixture");
        let stored: String = sqlx::query_scalar(
            "SELECT source_html FROM problem_statement_snapshots ss JOIN problems p ON p.id = ss.problem_id \
             WHERE p.external_contest_key = 1979 AND p.external_problem_key = 'A'",
        )
        .fetch_one(pool)
        .await
        .expect("stored first snapshot");
        assert_eq!(stored, "<img src=\"acm-os-asset://fixture\">");
    }

    #[tokio::test]
    async fn create_personal_note_writes_the_frozen_skeleton_and_commits_one_binding() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, problems, _knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        runtime
            .persist_manifest(&contest_draft())
            .await
            .expect("persist manifest");
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");

        let first = create_personal_note(&runtime, &problem)
            .await
            .expect("create personal note");
        assert_eq!(first.vault_relative_path, "Problems/CF-1979-A.md");
        assert_eq!(first.content_digest.len(), 64);
        if cfg!(windows) {
            assert!(first.windows_file_key.as_deref().is_some_and(|key| key.starts_with("same-file-1:")));
        }
        assert_eq!(
            fs::read_to_string(problems.join("CF-1979-A.md")).expect("read created note"),
            INITIAL_PROBLEM_MARKDOWN
        );

        let second = create_personal_note(&runtime, &problem)
            .await
            .expect("idempotent create");
        assert_eq!(second, first);
        let detail = runtime
            .lightweight_problem_detail(&problem)
            .await
            .expect("personal detail");
        assert_eq!(detail.identity_type, ProblemIdentityType::Personal);
        assert_eq!(detail.personal_note, Some(first));
        let pool = runtime._pool.as_ref().expect("ready database pool");
        let counts: (i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM file_bindings), \
                    (SELECT COUNT(*) FROM problems WHERE identity_type = 'personal')",
        )
        .fetch_one(pool)
        .await
        .expect("personal note counts");
        assert_eq!(counts, (1, 1));
    }

    #[tokio::test]
    async fn fresh_read_ignores_a_stale_projection_cache_without_a_watcher_event() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, problems, _knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        runtime
            .persist_manifest(&contest_draft())
            .await
            .expect("persist manifest");
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");
        create_personal_note(&runtime, &problem)
            .await
            .expect("create personal note");

        let cached = runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("initial projection");
        fs::write(
            problems.join("CF-1979-A.md"),
            "# Problem\n\n## 题解\n\n### External edit ×\n\n#### Not a route\n",
        )
        .expect("external edit without watcher event");

        let fresh = runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("fresh projection");
        let PersonalNoteReadState::Ready {
            projection: cached,
            ..
        } = cached else {
            panic!("initial projection must be ready");
        };
        let PersonalNoteReadState::Ready {
            projection: fresh,
            ..
        } = fresh else {
            panic!("fresh projection must be ready");
        };
        assert_ne!(fresh.content_digest, cached.content_digest);
        assert_eq!(fresh.solution_routes.len(), 1);
        assert_eq!(fresh.solution_routes[0].name, "External edit ×");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn relocation_uses_the_unique_windows_file_key_before_digest() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let moved_root = vault.join("Moved");
        fs::create_dir_all(&moved_root).expect("moved root");
        let moved = moved_root.join("renamed.md");
        fs::rename(problems.join("CF-1979-A.md"), &moved).expect("external rename");
        fs::write(&moved, "# Changed\n\n## 题解\n\n### File key route\n")
            .expect("external edit after rename");

        let state = runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("file-key relocation");
        let PersonalNoteReadState::Ready {
            binding,
            projection,
            relocated,
        } = state else {
            panic!("file-key relocation must resolve");
        };
        assert!(relocated);
        assert_eq!(binding.vault_relative_path, "Moved/renamed.md");
        assert_eq!(projection.solution_routes[0].name, "File key route");
    }

    #[tokio::test]
    async fn relocation_uses_a_unique_full_content_digest_when_file_key_is_unavailable() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let original = problems.join("CF-1979-A.md");
        let bytes = fs::read(&original).expect("original note");
        fs::remove_file(&original).expect("remove original path");
        let moved_root = vault.join("Archive");
        fs::create_dir_all(&moved_root).expect("archive root");
        fs::write(moved_root.join("note.md"), bytes).expect("replacement at new path");
        sqlx::query("UPDATE file_bindings SET windows_file_key = NULL")
            .execute(runtime._pool.as_ref().expect("ready database pool"))
            .await
            .expect("remove file-key evidence");

        let state = runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("digest relocation");
        let PersonalNoteReadState::Ready {
            binding,
            relocated,
            ..
        } = state else {
            panic!("unique digest must resolve");
        };
        assert!(relocated);
        assert_eq!(binding.vault_relative_path, "Archive/note.md");
    }

    #[tokio::test]
    async fn ambiguous_digest_preserves_personal_identity_and_marks_location_anomaly() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let original = problems.join("CF-1979-A.md");
        let bytes = fs::read(&original).expect("original note");
        fs::remove_file(&original).expect("remove original path");
        fs::write(vault.join("copy-one.md"), &bytes).expect("first digest match");
        fs::write(vault.join("copy-two.md"), &bytes).expect("second digest match");
        sqlx::query("UPDATE file_bindings SET windows_file_key = NULL")
            .execute(runtime._pool.as_ref().expect("ready database pool"))
            .await
            .expect("remove file-key evidence");

        let state = runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("location anomaly state");
        assert!(matches!(state, PersonalNoteReadState::LocationAnomaly { .. }));
        let detail = runtime
            .lightweight_problem_detail(&problem)
            .await
            .expect("preserved problem");
        assert_eq!(detail.identity_type, ProblemIdentityType::Personal);
        let binding_state: String = sqlx::query_scalar("SELECT binding_state FROM file_bindings")
            .fetch_one(runtime._pool.as_ref().expect("ready database pool"))
            .await
            .expect("binding state");
        assert_eq!(binding_state, "location_anomaly");
    }

    #[tokio::test]
    async fn unavailable_vault_preserves_personal_identity_and_system_facts() {
        let (directory, runtime, vault, _problems, problem) = personal_note_fixture().await;
        let offline = directory.path().join("vault-offline");
        fs::rename(&vault, &offline).expect("make vault unavailable");

        let state = runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("vault unavailable state");
        assert!(matches!(state, PersonalNoteReadState::VaultUnavailable { .. }));
        let detail = runtime
            .lightweight_problem_detail(&problem)
            .await
            .expect("preserved problem facts");
        assert_eq!(detail.identity_type, ProblemIdentityType::Personal);
        assert!(detail.personal_note.is_some());
        let binding_state: String = sqlx::query_scalar("SELECT binding_state FROM file_bindings")
            .fetch_one(runtime._pool.as_ref().expect("ready database pool"))
            .await
            .expect("binding state");
        assert_eq!(binding_state, "external_source_unavailable");

        fs::rename(&offline, &vault).expect("restore vault");
        assert!(matches!(
            runtime
                .read_personal_note_projection(&problem)
                .await
                .expect("restored vault read"),
            PersonalNoteReadState::Ready { .. }
        ));
        let restored_state: String =
            sqlx::query_scalar("SELECT binding_state FROM file_bindings")
                .fetch_one(runtime._pool.as_ref().expect("ready database pool"))
                .await
                .expect("restored binding state");
        assert_eq!(restored_state, "linked");
    }

    #[tokio::test]
    async fn invalid_binding_path_never_reads_outside_the_vault() {
        let (directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        fs::write(directory.path().join("outside.md"), "outside secret")
            .expect("outside fixture");
        sqlx::query("UPDATE file_bindings SET vault_relative_path = '../outside.md'")
            .execute(runtime._pool.as_ref().expect("ready database pool"))
            .await
            .expect("invalid binding fixture");

        assert_eq!(
            runtime.read_personal_note_projection(&problem).await,
            Err(PersonalNoteReadError::BindingUnavailable)
        );
    }

    #[tokio::test]
    async fn deterministic_relocation_never_steals_another_problem_binding() {
        let (_directory, runtime, _vault, problems, problem_a) = personal_note_fixture().await;
        let problem_b = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("problem B");
        create_personal_note(&runtime, &problem_b)
            .await
            .expect("create B note");
        fs::remove_file(problems.join("CF-1979-A.md")).expect("remove A path");
        sqlx::query(
            "UPDATE file_bindings SET windows_file_key = NULL \
             WHERE problem_id = (SELECT id FROM problems WHERE external_problem_key = 'A')",
        )
        .execute(runtime._pool.as_ref().expect("ready database pool"))
        .await
        .expect("remove A file-key evidence");

        assert!(matches!(
            runtime
                .read_personal_note_projection(&problem_a)
                .await
                .expect("occupied relocation state"),
            PersonalNoteReadState::LocationAnomaly { .. }
        ));
        let paths: Vec<String> =
            sqlx::query_scalar("SELECT vault_relative_path FROM file_bindings ORDER BY problem_id")
                .fetch_all(runtime._pool.as_ref().expect("ready database pool"))
                .await
                .expect("preserved bindings");
        assert_eq!(paths, ["Problems/CF-1979-A.md", "Problems/CF-1979-B.md"]);
    }

    #[tokio::test]
    async fn create_personal_note_never_overwrites_an_existing_target() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, problems, _knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        runtime
            .persist_manifest(&contest_draft())
            .await
            .expect("persist manifest");
        let target = problems.join("CF-1979-A.md");
        fs::write(&target, "external user note").expect("external note fixture");
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");

        assert_eq!(
            create_personal_note(&runtime, &problem).await,
            Err(PersonalNoteError::TargetAlreadyExists)
        );
        assert_eq!(
            fs::read_to_string(&target).expect("preserved external note"),
            "external user note"
        );
        let detail = runtime
            .lightweight_problem_detail(&problem)
            .await
            .expect("lightweight detail");
        assert_eq!(detail.identity_type, ProblemIdentityType::Lightweight);
        assert!(detail.personal_note.is_none());
    }

    #[tokio::test]
    async fn problem_detail_exposes_only_sanitized_snapshot_or_pending_state() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        runtime.persist_manifest(&contest_draft()).await.expect("persist manifest");
        runtime.persist_first_snapshot(&snapshot(
            "A",
            "<script>unsafe()</script><p>source only</p>",
            "<p>safe local statement</p>",
        )).await.expect("persist snapshot");

        let contest = acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest");
        let contest_detail = runtime.contest_detail(&contest).await.expect("contest detail");
        assert_eq!(contest_detail.problems.len(), 2);
        assert_eq!(contest_detail.problems[0].problem.index(), "A");
        let ready = runtime.lightweight_problem_detail(
            &acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "A").expect("problem A"),
        ).await.expect("ready problem detail");
        assert_eq!(ready.title, "Problem A");
        assert_eq!(ready.statement, StatementReadState::Ready {
            sanitized_html: "<p>safe local statement</p>".to_owned(),
        });
        assert!(runtime.statement_assets(
            &acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "A").expect("problem A assets"),
        ).await.expect("statement assets").is_empty());

        let pending = runtime.lightweight_problem_detail(
            &acm_os_domain::CodeforcesProblemIdentity::new(contest, "B").expect("problem B"),
        ).await.expect("pending problem detail");
        assert_eq!(pending.statement, StatementReadState::Pending);
    }

    #[tokio::test]
    #[ignore = "release-only real Codeforces import smoke; requires live network"]
    async fn real_codeforces_import_smoke_is_idempotent_in_a_temporary_database() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let adapter = crate::codeforces::CodeforcesHttpAdapter::new().expect("HTTP adapter");
        let contest = acm_os_domain::CodeforcesContestIdentity::new(1).expect("contest identity");

        let first = import_codeforces_contest(&runtime, &adapter, contest.clone())
            .await
            .expect("first real import");
        let shelf = runtime.list_contests().await.expect("contest shelf");
        assert_eq!(shelf.len(), 1);
        assert_eq!(shelf[0].contest, contest);
        assert!(shelf[0].problem_count > 0);

        let second = import_codeforces_contest(&runtime, &adapter, contest)
            .await
            .expect("idempotent real retry");
        // A retry may either leave an already-complete import unchanged or
        // fill additional pending snapshots. It must never create a second
        // contest or increase the missing set.
        assert!(
            second.persisted.missing_snapshot_problems.len()
                <= first.persisted.missing_snapshot_problems.len()
        );
        assert_eq!(runtime.list_contests().await.expect("shelf after retry").len(), 1);
    }

    #[tokio::test]
    async fn reimport_rejects_manifest_drift_without_changing_the_first_manifest() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        runtime.persist_manifest(&contest_draft()).await.expect("first manifest");

        let contest = acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest");
        let drifted = ContestImportDraft::validated(
            contest.clone(),
            "Changed remote title".to_owned(),
            "https://codeforces.com/contest/1979".to_owned(),
            None,
            vec![ContestProblemSlotDraft {
                ordinal: 1,
                problem: acm_os_domain::CodeforcesProblemIdentity::new(contest, "A").expect("problem"),
                title: "Problem A".to_owned(),
                rating: Some(800),
                source_url: "https://codeforces.com/contest/1979/problem/A".to_owned(),
            }],
        )
        .expect("valid but drifted remote manifest");
        assert_eq!(
            runtime.persist_manifest(&drifted).await,
            Err(ContestImportPersistenceError::ManifestConflict)
        );

        let pool = runtime._pool.as_ref().expect("ready database pool");
        let persisted_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contest_problems")
            .fetch_one(pool).await.expect("persisted slots");
        assert_eq!(persisted_count, 2);
    }

    #[tokio::test]
    async fn future_schema_is_blocked_without_running_migrations() {
        let directory = TempDir::new().expect("temporary app data");
        let database_path = directory.path().join(DATABASE_FILENAME);
        let pool = connect_read_write(&database_path).await.expect("future database");
        let supported = supported_schema_version();
        let found = supported + 1;
        create_empty_migration_ledger(&pool).await;
        sqlx::query(
            "INSERT INTO _sqlx_migrations \
                (version, description, success, checksum, execution_time) \
             VALUES (?1, 'future migration', 1, X'00', 0)",
        )
        .bind(found)
        .execute(&pool)
        .await
        .expect("future version");
        pool.close().await;

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::UnsupportedSchema { found, supported },
            }
        );

        let inspection = connect_read_only(&database_path).await.expect("inspect future database");
        let app_metadata_exists: i64 = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'app_metadata')",
        )
        .fetch_one(&inspection)
        .await
        .expect("inspect tables");
        assert_eq!(app_metadata_exists, 0);
    }

    #[tokio::test]
    async fn malformed_migration_ledger_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        let database_path = directory.path().join(DATABASE_FILENAME);
        let pool = connect_read_write(&database_path).await.expect("malformed database");
        pool.execute("CREATE TABLE _sqlx_migrations (unexpected INTEGER)")
            .await
            .expect("malformed ledger");
        pool.close().await;

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::MigrationLedgerInvalid,
            }
        );
    }

    #[tokio::test]
    async fn edited_migration_history_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        {
            let runtime = start_database(directory.path()).await;
            let pool = runtime._pool.as_ref().expect("ready database pool");
            sqlx::query("UPDATE _sqlx_migrations SET checksum = X'00' WHERE version = 1")
                .execute(pool)
                .await
                .expect("tamper migration checksum");
        }

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::MigrationFailed,
            }
        );
    }

    #[tokio::test]
    async fn unreadable_database_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        fs::write(directory.path().join(DATABASE_FILENAME), b"not a sqlite database")
            .expect("corrupt database fixture");

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn consistent_backup_contains_the_source_schema() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let pool = runtime._pool.as_ref().expect("ready database pool");

        let backup_path = create_pre_migration_backup(pool, directory.path(), 1, 2)
            .await
            .expect("consistent backup");
        let backup_pool = connect_read_only(&backup_path).await.expect("backup database");
        verify_integrity(&backup_pool).await.expect("backup integrity");
        let metadata_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_metadata")
            .fetch_one(&backup_pool)
            .await
            .expect("backup metadata");
        assert_eq!(metadata_count, 1);
        let mut partial_path = backup_path.as_os_str().to_os_string();
        partial_path.push(".partial");
        assert!(!PathBuf::from(partial_path).exists());
    }

    #[tokio::test]
    async fn invalid_partial_backup_is_not_published() {
        let directory = TempDir::new().expect("temporary backup directory");
        let partial_path = directory.path().join("candidate.sqlite3.partial");
        let backup_path = directory.path().join("candidate.sqlite3");
        fs::write(&partial_path, b"not a sqlite backup").expect("invalid partial backup");

        let result = verify_and_publish_backup(&partial_path, &backup_path).await;

        assert_eq!(
            result.expect_err("invalid backup must not be published"),
            StartupRecoveryReason::PreMigrationBackupFailed
        );
        assert!(!partial_path.exists());
        assert!(!backup_path.exists());
    }

    #[tokio::test]
    async fn missing_required_table_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        {
            let runtime = start_database(directory.path()).await;
            let pool = runtime._pool.as_ref().expect("ready database pool");
            pool.execute("DROP TABLE app_metadata")
                .await
                .expect("damage logical schema");
            pool.close().await;
        }

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn missing_required_default_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        {
            let runtime = start_database(directory.path()).await;
            let pool = runtime._pool.as_ref().expect("ready database pool");
            pool.execute("ALTER TABLE app_metadata RENAME TO app_metadata_old")
                .await
                .expect("rename metadata table");
            pool.execute(
                "CREATE TABLE app_metadata (\
                    singleton INTEGER PRIMARY KEY CHECK (singleton = 1), \
                    schema_generation INTEGER NOT NULL CHECK (schema_generation > 0), \
                    created_at_utc TEXT NOT NULL\
                )",
            )
            .await
            .expect("recreate metadata without default");
            pool.execute(
                "INSERT INTO app_metadata SELECT singleton, schema_generation, created_at_utc \
                 FROM app_metadata_old",
            )
            .await
            .expect("preserve metadata row");
            pool.execute("DROP TABLE app_metadata_old")
                .await
                .expect("remove old metadata table");
            pool.close().await;
        }

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn additional_check_constraint_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        {
            let runtime = start_database(directory.path()).await;
            let pool = runtime._pool.as_ref().expect("ready database pool");
            pool.execute("ALTER TABLE app_metadata RENAME TO app_metadata_old")
                .await
                .expect("rename metadata table");
            pool.execute(
                "CREATE TABLE app_metadata (\
                    singleton INTEGER PRIMARY KEY CHECK (singleton = 1), \
                    schema_generation INTEGER NOT NULL CHECK (schema_generation > 0) \
                        CHECK (schema_generation < 5), \
                    created_at_utc TEXT NOT NULL \
                        DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
                )",
            )
            .await
            .expect("recreate metadata with hidden constraint");
            pool.execute(
                "INSERT INTO app_metadata SELECT singleton, schema_generation, created_at_utc \
                 FROM app_metadata_old",
            )
            .await
            .expect("preserve metadata row");
            pool.execute("DROP TABLE app_metadata_old")
                .await
                .expect("remove old metadata table");
            pool.close().await;
        }

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn unknown_trigger_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        {
            let runtime = start_database(directory.path()).await;
            let pool = runtime._pool.as_ref().expect("ready database pool");
            pool.execute(
                "CREATE TRIGGER corrupt_metadata AFTER UPDATE ON app_metadata \
                 BEGIN DELETE FROM app_metadata; END",
            )
            .await
            .expect("create unknown trigger");
            pool.close().await;
        }

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn unexpected_column_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        {
            let runtime = start_database(directory.path()).await;
            let pool = runtime._pool.as_ref().expect("ready database pool");
            pool.execute("ALTER TABLE app_metadata ADD COLUMN unexpected TEXT")
                .await
                .expect("add unexpected column");
            pool.close().await;
        }

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn empty_ledger_with_unknown_table_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        let database_path = directory.path().join(DATABASE_FILENAME);
        let pool = connect_read_write(&database_path).await.expect("unknown database");
        create_empty_migration_ledger(&pool).await;
        pool.execute("CREATE TABLE foreign_user_data (value TEXT)")
            .await
            .expect("create unknown table");
        pool.close().await;

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::MigrationLedgerInvalid,
            }
        );
    }

    #[tokio::test]
    async fn pre_existing_version_zero_database_is_backed_up_before_migration() {
        let directory = TempDir::new().expect("temporary app data");
        let database_path = directory.path().join(DATABASE_FILENAME);
        let pool = connect_read_write(&database_path).await.expect("version zero database");
        create_empty_migration_ledger(&pool).await;
        pool.close().await;

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::Ready { schema_version: 4 }
        );

        let backup_directory = directory.path().join("backups").join("pre-migration");
        let backups: Vec<PathBuf> = fs::read_dir(backup_directory)
            .expect("pre-migration backup directory")
            .map(|entry| entry.expect("backup entry").path())
            .collect();
        assert_eq!(backups.len(), 1);
        let backup_pool = connect_read_only(&backups[0]).await.expect("version zero backup");
        let application_tables: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_metadata'",
        )
        .fetch_one(&backup_pool)
        .await
        .expect("inspect version zero backup");
        assert_eq!(application_tables, 0);
    }

    #[tokio::test]
    async fn version_one_database_is_backed_up_then_migrated_to_current_version() {
        let directory = TempDir::new().expect("temporary app data");
        let database_path = directory.path().join(DATABASE_FILENAME);
        let pool = connect_read_write(&database_path)
            .await
            .expect("version one database");
        create_version_one_database(&pool).await;
        pool.close().await;

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::Ready { schema_version: 4 }
        );
        let runtime_pool = runtime._pool.as_ref().expect("migrated database pool");
        let workspace_rows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM workspace_settings")
                .fetch_one(runtime_pool)
                .await
                .expect("empty workspace settings");
        assert_eq!(workspace_rows, 0);

        let backup_directory = directory.path().join("backups").join("pre-migration");
        let backups: Vec<PathBuf> = fs::read_dir(backup_directory)
            .expect("pre-migration backup directory")
            .map(|entry| entry.expect("backup entry").path())
            .collect();
        assert_eq!(backups.len(), 1);
        let backup_pool = connect_read_only(&backups[0])
            .await
            .expect("version one backup");
        assert_eq!(
            inspect_schema_version(&backup_pool)
                .await
                .expect("backup schema version"),
            1
        );
        let workspace_table_exists: i64 = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master \
             WHERE type = 'table' AND name = 'workspace_settings')",
        )
        .fetch_one(&backup_pool)
        .await
        .expect("inspect version one backup");
        assert_eq!(workspace_table_exists, 0);
    }

    #[tokio::test]
    async fn failed_pre_migration_backup_does_not_run_migration() {
        let directory = TempDir::new().expect("temporary app data");
        let database_path = directory.path().join(DATABASE_FILENAME);
        let pool = connect_read_write(&database_path).await.expect("version zero database");
        create_empty_migration_ledger(&pool).await;
        pool.close().await;
        fs::write(directory.path().join("backups"), b"block backup directory")
            .expect("create backup blocker");

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::PreMigrationBackupFailed,
            }
        );

        let inspection = connect_read_only(&database_path).await.expect("inspect version zero");
        let ledger_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&inspection)
            .await
            .expect("inspect unchanged ledger");
        let metadata_exists: i64 = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'app_metadata')",
        )
        .fetch_one(&inspection)
        .await
        .expect("inspect absent metadata table");
        assert_eq!(ledger_rows, 0);
        assert_eq!(metadata_exists, 0);
    }

    #[tokio::test]
    async fn workspace_configuration_persists_across_restart() {
        let app_data = TempDir::new().expect("temporary app data");
        let vault = TempDir::new().expect("temporary vault");
        let problem_root = vault.path().join("Problems");
        let knowledge_root = vault.path().join("Knowledge");
        fs::create_dir(&problem_root).expect("problem root");
        fs::create_dir(&knowledge_root).expect("knowledge root");

        let runtime = start_database(app_data.path()).await;
        assert_eq!(
            query_workspace_configuration(&runtime)
                .await
                .expect("initial workspace status"),
            WorkspaceConfigurationStatus::Unconfigured
        );
        let saved = configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: vault.path().to_string_lossy().into_owned(),
                problem_root_path: problem_root.to_string_lossy().into_owned(),
                knowledge_root_path: knowledge_root.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect("configure workspace");
        let expected_vault = fs::canonicalize(vault.path())
            .expect("canonical vault")
            .to_string_lossy()
            .into_owned();
        assert_eq!(saved.active_vault_path(), expected_vault);
        drop(runtime);

        let restarted = start_database(app_data.path()).await;
        assert_eq!(
            query_workspace_configuration(&restarted)
                .await
                .expect("persisted workspace status"),
            WorkspaceConfigurationStatus::Configured(saved)
        );
    }

    #[tokio::test]
    async fn workspace_roots_must_be_inside_vault_and_non_overlapping() {
        let app_data = TempDir::new().expect("temporary app data");
        let vault = TempDir::new().expect("temporary vault");
        let outside = TempDir::new().expect("outside directory");
        let problem_root = vault.path().join("Problems");
        let nested_knowledge_root = problem_root.join("Knowledge");
        fs::create_dir(&problem_root).expect("problem root");
        fs::create_dir(&nested_knowledge_root).expect("nested knowledge root");
        let runtime = start_database(app_data.path()).await;

        let outside_error = configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: vault.path().to_string_lossy().into_owned(),
                problem_root_path: problem_root.to_string_lossy().into_owned(),
                knowledge_root_path: outside.path().to_string_lossy().into_owned(),
            },
        )
        .await
        .expect_err("outside root must be rejected");
        assert_eq!(
            outside_error,
            WorkspaceConfigurationError::RootOutsideVault {
                field: WorkspacePathField::KnowledgeRoot,
            }
        );

        let overlap_error = configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: vault.path().to_string_lossy().into_owned(),
                problem_root_path: problem_root.to_string_lossy().into_owned(),
                knowledge_root_path: nested_knowledge_root.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect_err("nested roots must be rejected");
        assert_eq!(overlap_error, WorkspaceConfigurationError::RootsOverlap);
        assert_eq!(
            query_workspace_configuration(&runtime)
                .await
                .expect("workspace remains unconfigured"),
            WorkspaceConfigurationStatus::Unconfigured
        );
    }

    #[tokio::test]
    async fn workspace_paths_must_exist_and_be_directories() {
        let app_data = TempDir::new().expect("temporary app data");
        let vault = TempDir::new().expect("temporary vault");
        let problem_file = vault.path().join("problem-file");
        let knowledge_root = vault.path().join("Knowledge");
        fs::write(&problem_file, b"not a directory").expect("problem file");
        fs::create_dir(&knowledge_root).expect("knowledge root");
        let runtime = start_database(app_data.path()).await;

        let error = configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: vault.path().to_string_lossy().into_owned(),
                problem_root_path: problem_file.to_string_lossy().into_owned(),
                knowledge_root_path: knowledge_root.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect_err("file root must be rejected");
        assert_eq!(
            error,
            WorkspaceConfigurationError::PathNotDirectory {
                field: WorkspacePathField::ProblemRoot,
            }
        );

        let required_error = configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: "   ".to_owned(),
                problem_root_path: problem_file.to_string_lossy().into_owned(),
                knowledge_root_path: knowledge_root.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect_err("blank vault path must be rejected");
        assert_eq!(
            required_error,
            WorkspaceConfigurationError::PathRequired {
                field: WorkspacePathField::ActiveVault,
            }
        );

        let unavailable_error = configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: vault.path().join("missing").to_string_lossy().into_owned(),
                problem_root_path: problem_file.to_string_lossy().into_owned(),
                knowledge_root_path: knowledge_root.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect_err("missing vault path must be rejected");
        assert_eq!(
            unavailable_error,
            WorkspaceConfigurationError::PathUnavailable {
                field: WorkspacePathField::ActiveVault,
            }
        );
    }

    #[tokio::test]
    async fn initial_workspace_configuration_cannot_be_silently_replaced() {
        let app_data = TempDir::new().expect("temporary app data");
        let vault = TempDir::new().expect("temporary vault");
        let problem_root = vault.path().join("Problems");
        let knowledge_root = vault.path().join("Knowledge");
        fs::create_dir(&problem_root).expect("problem root");
        fs::create_dir(&knowledge_root).expect("knowledge root");
        let runtime = start_database(app_data.path()).await;
        let draft = WorkspaceConfigurationDraft {
            active_vault_path: vault.path().to_string_lossy().into_owned(),
            problem_root_path: problem_root.to_string_lossy().into_owned(),
            knowledge_root_path: knowledge_root.to_string_lossy().into_owned(),
        };

        configure_workspace(&runtime, draft.clone())
            .await
            .expect("initial configuration");
        assert_eq!(
            configure_workspace(&runtime, draft)
                .await
                .expect_err("replacement requires a future preview/confirm flow"),
            WorkspaceConfigurationError::AlreadyConfigured
        );
    }

    #[tokio::test]
    async fn concurrent_initial_configuration_persists_exactly_one_winner() {
        let app_data = TempDir::new().expect("temporary app data");
        let first_vault = TempDir::new().expect("first temporary vault");
        let second_vault = TempDir::new().expect("second temporary vault");
        let make_draft = |vault: &TempDir| {
            let problem_root = vault.path().join("Problems");
            let knowledge_root = vault.path().join("Knowledge");
            fs::create_dir(&problem_root).expect("problem root");
            fs::create_dir(&knowledge_root).expect("knowledge root");
            WorkspaceConfigurationDraft {
                active_vault_path: vault.path().to_string_lossy().into_owned(),
                problem_root_path: problem_root.to_string_lossy().into_owned(),
                knowledge_root_path: knowledge_root.to_string_lossy().into_owned(),
            }
        };
        let first_draft = make_draft(&first_vault);
        let second_draft = make_draft(&second_vault);
        let runtime = start_database(app_data.path()).await;

        let (first, second) = tokio::join!(
            configure_workspace(&runtime, first_draft),
            configure_workspace(&runtime, second_draft),
        );
        let winner = match (&first, &second) {
            (Ok(winner), Err(WorkspaceConfigurationError::AlreadyConfigured))
            | (Err(WorkspaceConfigurationError::AlreadyConfigured), Ok(winner)) => winner,
            outcomes => panic!("expected one winner and one duplicate rejection: {outcomes:?}"),
        };
        assert_eq!(
            query_workspace_configuration(&runtime)
                .await
                .expect("persisted concurrent winner"),
            WorkspaceConfigurationStatus::Configured(winner.clone())
        );
    }

    #[tokio::test]
    async fn corrupted_workspace_relationship_requires_recovery() {
        let app_data = TempDir::new().expect("temporary app data");
        let vault = TempDir::new().expect("temporary vault");
        let problem_root = vault.path().join("Problems");
        let knowledge_root = vault.path().join("Knowledge");
        fs::create_dir(&problem_root).expect("problem root");
        fs::create_dir(&knowledge_root).expect("knowledge root");
        {
            let runtime = start_database(app_data.path()).await;
            configure_workspace(
                &runtime,
                WorkspaceConfigurationDraft {
                    active_vault_path: vault.path().to_string_lossy().into_owned(),
                    problem_root_path: problem_root.to_string_lossy().into_owned(),
                    knowledge_root_path: knowledge_root.to_string_lossy().into_owned(),
                },
            )
            .await
            .expect("configure workspace");
            let pool = runtime._pool.as_ref().expect("ready database pool");
            pool.execute(
                "UPDATE workspace_settings SET knowledge_root_path = active_vault_path \
                 WHERE singleton = 1",
            )
            .await
            .expect("corrupt persisted relationship");
        }

        let restarted = start_database(app_data.path()).await;
        assert_eq!(
            restarted.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn ready_runtime_holds_startup_lock_until_drop() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::Ready { schema_version: 4 }
        );

        let blocked = acquire_startup_lock(directory.path(), Duration::from_millis(75)).await;
        assert_eq!(
            blocked.expect_err("live runtime must retain the startup lock"),
            StartupRecoveryReason::DatabaseUnavailable
        );

        drop(runtime);
        acquire_startup_lock(directory.path(), Duration::from_secs(1))
            .await
            .expect("dropping runtime must release the startup lock");
    }

    #[tokio::test]
    async fn startup_lock_wait_is_bounded() {
        let directory = TempDir::new().expect("temporary app data");
        let held = acquire_startup_lock(directory.path(), Duration::from_secs(1))
            .await
            .expect("hold startup lock");

        let result = acquire_startup_lock(directory.path(), Duration::from_millis(75)).await;
        assert_eq!(
            result.expect_err("second lock should time out"),
            StartupRecoveryReason::DatabaseUnavailable
        );
        drop(held);
    }
}
