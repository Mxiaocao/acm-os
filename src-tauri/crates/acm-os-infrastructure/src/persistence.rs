use std::collections::HashMap;
use std::fs::{File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use acm_os_application::{
    AcceptedKnowledgeCandidateProjection, ActiveReviewCycle, CompletedReviewAttempt, ContestDetail,
    ContestImportDraft, ContestImportPersistenceError, ContestImportPort, ContestImportStatus,
    ContestReadError, ContestReadPort, ContestShelfItem, CreatedPersonalNoteFile,
    ExtraProblemLinkTarget, KnowledgeCandidateDisposition, KnowledgeCandidateError,
    KnowledgeCandidatePort, KnowledgeCandidateProjection, KnowledgeDetailPort,
    KnowledgeDetailProjection, KnowledgeIndexError, KnowledgeIndexPort, KnowledgeIndexProjection,
    KnowledgeLinkProjection, KnowledgeLinkResolution, KnowledgeLocationState,
    KnowledgeNodeProjection, KnowledgeRelationPort, KnowledgeUnderstandingPort,
    KnowledgeUnderstandingProjection, LightweightProblemDetail, LightweightProblemItem,
    LocalStatementAsset, PersistedContestImport, PersonalNoteBinding, PersonalNoteCreationContext,
    PersonalNoteDeletionError, PersonalNoteDeletionPort, PersonalNoteError, PersonalNotePatchError,
    PersonalNotePatchPort, PersonalNotePort, PersonalNoteReadError, PersonalNoteReadPort,
    PersonalNoteReadState, PreparedPersonalNoteDeletion, PrerequisiteLinkTarget,
    ProblemIdentityType, ProblemLifecycleError, ProblemLifecyclePort, ProblemLifecycleState,
    ProblemMarkdownProjection, ProblemMasteryProjection, RelatedKnowledgeProblemProjection,
    RevealedReviewHelp, ReviewAttempt, ReviewAttemptError, ReviewAttemptPort, ReviewAttemptStatus,
    ReviewCompletionContext, ReviewCompletionInput, ReviewFailureReason, ReviewFocusView,
    ReviewHelpDrawerView, ReviewHelpItem, ReviewHistoryItem, ReviewHistoryView, StartupGateStatus,
    StartupRecoveryReason, StatementReadState, StatementSnapshotDraft, SubmissionFact,
    TodayEntryOrigin, TodayEntryStatus, TodayGenerationCandidate, TodayGenerationContext,
    TodayReplanPreview, TodaySnapshot, TodaySnapshotEntry, TodaySnapshotError, TodaySnapshotPort,
    WeeklyAcmBudgetPort, WeeklyAcmBudgetSchedule, WorkspaceConfiguration,
    WorkspaceConfigurationPort, WorkspacePathResolutionError, WorkspacePersistenceError,
};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

use crate::file_binding::{
    markdown_files, resolve_personal_note, sha256_hex, windows_file_key, BindingResolution,
    ResolvedNoteFile,
};
use crate::knowledge_index::{
    discover_markdown, extract_wikilink_targets, replace_index, resolve_links,
    StoredKnowledgeBinding,
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
    recovery_root: Option<PathBuf>,
}

impl DatabaseRuntime {
    pub fn recovery(reason: StartupRecoveryReason) -> Self {
        Self {
            _pool: None,
            _startup_lock: None,
            status: StartupGateStatus::RecoveryRequired { reason },
            markdown_projection_cache: Mutex::new(HashMap::new()),
            recovery_root: None,
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

    async fn review_help_sources(
        &self,
        attempt_id: &str,
    ) -> Result<
        Vec<(
            acm_os_domain::ReviewHelpLevel,
            Option<crate::markdown::ReviewHelpContent>,
        )>,
        ReviewAttemptError,
    > {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT p.external_contest_key, p.external_problem_key \
             FROM review_attempts ra JOIN problems p ON p.id = ra.problem_id \
             WHERE ra.id = ?1 AND ra.attempt_status = 'in_progress'",
        )
        .bind(attempt_id)
        .fetch_optional(
            self._pool
                .as_ref()
                .ok_or(ReviewAttemptError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let (contest_id, index) = row.ok_or(ReviewAttemptError::AttemptNotFound)?;
        let contest = acm_os_domain::CodeforcesContestIdentity::new(
            u64::try_from(contest_id).map_err(|_| ReviewAttemptError::IntegrityViolation)?,
        )
        .map_err(|_| ReviewAttemptError::IntegrityViolation)?;
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, index)
            .map_err(|_| ReviewAttemptError::IntegrityViolation)?;
        let binding = match self.read_personal_note_projection(&problem).await {
            Ok(PersonalNoteReadState::Ready { binding, .. }) => binding,
            Ok(PersonalNoteReadState::LocationAnomaly { .. })
            | Ok(PersonalNoteReadState::VaultUnavailable { .. })
            | Err(PersonalNoteReadError::BindingUnavailable)
            | Err(PersonalNoteReadError::FileReadFailed) => {
                return Err(ReviewAttemptError::NoteUnavailable);
            }
            Err(PersonalNoteReadError::InvalidUtf8) => {
                return Err(ReviewAttemptError::InvalidMarkdown);
            }
            Err(PersonalNoteReadError::ProblemNotFound)
            | Err(PersonalNoteReadError::NotPersonal) => {
                return Err(ReviewAttemptError::IntegrityViolation);
            }
            Err(PersonalNoteReadError::PersistenceUnavailable) => {
                return Err(ReviewAttemptError::PersistenceUnavailable);
            }
        };
        let workspace = self
            .load_workspace_configuration()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?
            .ok_or(ReviewAttemptError::NoteUnavailable)?;
        let vault = workspace.active_vault_path().to_owned();
        let note_relative_path = binding.vault_relative_path;
        let knowledge_root = workspace.knowledge_root_path().to_owned();
        tokio::task::spawn_blocking(move || {
            build_review_help_sources(&vault, &note_relative_path, &knowledge_root)
        })
        .await
        .map_err(|_| ReviewAttemptError::NoteUnavailable)?
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
            Err(sqlx::Error::Database(database_error)) if database_error.is_unique_violation() => {
                return Ok(false);
            }
            Err(_) => return Err(PersonalNoteReadError::PersistenceUnavailable),
        };
        if result.rows_affected() == 1 {
            Ok(true)
        } else {
            let current: Option<(String, Option<String>, String)> = sqlx::query_as(
                "SELECT vault_relative_path, windows_file_key, content_digest \
                 FROM file_bindings WHERE problem_id = ?1",
            )
            .bind(problem_id)
            .fetch_optional(
                self._pool
                    .as_ref()
                    .ok_or(PersonalNoteReadError::PersistenceUnavailable)?,
            )
            .await
            .map_err(|_| PersonalNoteReadError::PersistenceUnavailable)?;
            if current.as_ref()
                == Some(&(
                    resolved.relative_path.clone(),
                    resolved.windows_file_key.clone(),
                    resolved.content_digest.clone(),
                ))
            {
                Ok(true)
            } else {
                Err(PersonalNoteReadError::BindingUnavailable)
            }
        }
    }
}

impl KnowledgeIndexPort for DatabaseRuntime {
    async fn rebuild_knowledge_index(
        &self,
    ) -> Result<KnowledgeIndexProjection, KnowledgeIndexError> {
        let workspace = self
            .load_workspace_configuration()
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?
            .ok_or(KnowledgeIndexError::WorkspaceUnavailable)?;
        let active_vault = workspace.active_vault_path().to_owned();
        let knowledge_root = workspace.knowledge_root_path().to_owned();
        let (discovered, relocation_candidates) =
            tokio::task::spawn_blocking(move || discover_markdown(&active_vault, &knowledge_root))
                .await
                .map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)??;

        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeIndexError::PersistenceUnavailable)?;
        let stored_rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT knowledge_node_id, vault_relative_path, windows_file_key, content_digest \
             FROM knowledge_file_bindings ORDER BY knowledge_node_id",
        )
        .fetch_all(pool)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let stored = stored_rows
            .into_iter()
            .map(
                |(node_id, relative_path, file_key, digest)| StoredKnowledgeBinding {
                    node_id,
                    relative_path,
                    file_key,
                    digest,
                },
            )
            .collect();
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let projection =
            replace_index(&mut transaction, stored, discovered, relocation_candidates).await?;
        transaction
            .commit()
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        Ok(projection)
    }

    async fn search_knowledge_index(
        &self,
        query: &str,
    ) -> Result<Vec<KnowledgeNodeProjection>, KnowledgeIndexError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeIndexError::PersistenceUnavailable)?;
        let pattern = format!("%{}%", query.to_lowercase());
        let rows: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT i.knowledge_node_id, i.display_name, i.vault_relative_path, \
                    i.content_digest, b.windows_file_key \
             FROM knowledge_discovery_index i JOIN knowledge_file_bindings b \
               ON b.knowledge_node_id = i.knowledge_node_id \
             WHERE lower(i.display_name) LIKE ?1 ESCAPE '\\' \
             ORDER BY lower(i.display_name), i.knowledge_node_id",
        )
        .bind(pattern)
        .fetch_all(pool)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    knowledge_node_id,
                    display_name,
                    vault_relative_path,
                    content_digest,
                    windows_file_key,
                )| {
                    KnowledgeNodeProjection {
                        knowledge_node_id,
                        display_name,
                        vault_relative_path,
                        content_digest,
                        windows_file_key,
                        location_state: KnowledgeLocationState::Ready,
                    }
                },
            )
            .collect())
    }
}

impl acm_os_application::KnowledgeRelationPort for DatabaseRuntime {
    async fn rebuild_knowledge_relations(
        &self,
    ) -> Result<Vec<KnowledgeLinkProjection>, KnowledgeIndexError> {
        let index = self.rebuild_knowledge_index().await?;
        let workspace = self
            .load_workspace_configuration()
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?
            .ok_or(KnowledgeIndexError::WorkspaceUnavailable)?;
        let active_vault = workspace.active_vault_path().to_owned();
        let vault_root = std::path::PathBuf::from(&active_vault);
        let formal_paths = index
            .nodes
            .iter()
            .map(|node| node.vault_relative_path.clone())
            .collect::<std::collections::HashSet<_>>();
        let non_knowledge_markdown_paths = markdown_files(&vault_root)
            .map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?
            .into_iter()
            .filter_map(|path| {
                path.strip_prefix(&vault_root)
                    .ok()
                    .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            })
            .filter(|path| !formal_paths.contains(path))
            .collect::<Vec<_>>();
        let mut links = Vec::new();
        for node in &index.nodes {
            let path = vault_root.join(&node.vault_relative_path);
            let markdown = std::fs::read_to_string(&path)
                .map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?;
            links.extend(
                extract_wikilink_targets(&markdown)
                    .into_iter()
                    .map(|target| {
                        (
                            "knowledge".to_owned(),
                            node.knowledge_node_id.clone(),
                            target,
                        )
                    }),
            );
        }
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeIndexError::PersistenceUnavailable)?;
        let problem_bindings: Vec<(i64, String, Option<String>, String)> = sqlx::query_as(
            "SELECT fb.problem_id, fb.vault_relative_path, fb.windows_file_key, fb.content_digest \
             FROM file_bindings fb JOIN problems p ON p.id = fb.problem_id \
             WHERE p.identity_type = 'personal' ORDER BY fb.problem_id",
        )
        .fetch_all(pool)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let problem_links = tokio::task::spawn_blocking(move || {
            let mut links = Vec::new();
            for (problem_id, relative_path, file_key, digest) in problem_bindings {
                let BindingResolution::Ready(note) = resolve_personal_note(
                    &active_vault,
                    &relative_path,
                    file_key.as_deref(),
                    &digest,
                ) else {
                    continue;
                };
                let Ok(markdown) = std::str::from_utf8(&note.bytes) else {
                    continue;
                };
                let Some(targets) = crate::markdown::prerequisite_targets(markdown) else {
                    continue;
                };
                links.extend(
                    targets
                        .into_iter()
                        .map(|target| ("problem".to_owned(), problem_id.to_string(), target)),
                );
            }
            links
        })
        .await
        .map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?;
        links.extend(problem_links);
        let mut tx = pool
            .begin()
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let projections = resolve_links(links, &index.nodes, &non_knowledge_markdown_paths);
        sqlx::query("DELETE FROM knowledge_link_index")
            .execute(&mut *tx)
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        for relation in &projections {
            sqlx::query("INSERT INTO knowledge_link_index (source_kind, source_id, target_ref, target_knowledge_node_id, resolution) VALUES (?1, ?2, ?3, ?4, ?5)")
                .bind(&relation.source_kind).bind(&relation.source_id).bind(&relation.target_ref).bind(&relation.target_knowledge_node_id).bind(match relation.resolution { KnowledgeLinkResolution::Resolved => "resolved", KnowledgeLinkResolution::Unresolved => "unresolved", KnowledgeLinkResolution::Ambiguous => "ambiguous", KnowledgeLinkResolution::NonKnowledgeTarget => "non_knowledge_target" })
                .execute(&mut *tx).await.map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        }
        tx.commit()
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        Ok(projections)
    }
}

impl KnowledgeUnderstandingPort for DatabaseRuntime {
    async fn confirm_knowledge_understanding(
        &self,
        knowledge_node_id: &str,
        selected: acm_os_domain::KnowledgeUnderstandingLevel,
        confirmed_on: acm_os_domain::LocalDate,
    ) -> Result<KnowledgeUnderstandingProjection, KnowledgeIndexError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeIndexError::PersistenceUnavailable)?;
        let exists: i64 = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM knowledge_discovery_index WHERE knowledge_node_id = ?1)",
        )
        .bind(knowledge_node_id)
        .fetch_one(pool)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        if exists == 0 {
            return Err(KnowledgeIndexError::IntegrityViolation);
        }
        let previous: Option<(String, String)> = sqlx::query_as("SELECT historical_highest_level, first_reached_highest_local_date FROM knowledge_understanding_states WHERE knowledge_node_id = ?1").bind(knowledge_node_id).fetch_optional(pool).await.map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let previous = previous
            .map(|(level, date)| {
                Ok((
                    parse_understanding(&level)?,
                    acm_os_domain::LocalDate::parse_iso(&date)
                        .map_err(|_| KnowledgeIndexError::IntegrityViolation)?,
                ))
            })
            .transpose()?;
        let decision =
            acm_os_domain::confirm_knowledge_understanding(previous, selected, confirmed_on);
        sqlx::query("INSERT INTO knowledge_understanding_states (knowledge_node_id, current_level, historical_highest_level, first_reached_highest_local_date) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(knowledge_node_id) DO UPDATE SET current_level = excluded.current_level, historical_highest_level = excluded.historical_highest_level, first_reached_highest_local_date = excluded.first_reached_highest_local_date, updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')")
            .bind(knowledge_node_id).bind(understanding_value(decision.current)).bind(understanding_value(decision.historical_highest)).bind(decision.first_reached_highest_on.to_iso_string()).execute(pool).await.map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        Ok(KnowledgeUnderstandingProjection {
            knowledge_node_id: knowledge_node_id.to_owned(),
            current: decision.current,
            historical_highest: decision.historical_highest,
            first_reached_highest_on: decision.first_reached_highest_on,
        })
    }
}

impl acm_os_application::KnowledgeReevaluationPort for DatabaseRuntime {
    async fn load_knowledge_reevaluation_suggestion(
        &self,
        knowledge_node_id: &str,
    ) -> Result<acm_os_application::KnowledgeReevaluationSuggestion, KnowledgeIndexError> {
        self.rebuild_knowledge_relations().await?;
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeIndexError::PersistenceUnavailable)?;
        let count: Option<i64> = sqlx::query_scalar("SELECT (SELECT COUNT(DISTINCT l.source_id) FROM knowledge_link_index l JOIN review_attempts ra ON l.source_kind = 'problem' AND CAST(l.source_id AS INTEGER) = ra.problem_id WHERE l.target_knowledge_node_id = kus.knowledge_node_id AND l.resolution = 'resolved' AND ra.attempt_status = 'completed' AND ra.judgement = 'mastered' AND ra.completed_at_utc > kus.updated_at_utc) FROM knowledge_understanding_states kus WHERE kus.knowledge_node_id = ?1").bind(knowledge_node_id).fetch_optional(pool).await.map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let count = count.ok_or(KnowledgeIndexError::KnowledgeNodeNotFound)?;
        Ok(acm_os_application::KnowledgeReevaluationSuggestion {
            knowledge_node_id: knowledge_node_id.to_owned(),
            should_suggest: count >= 3,
            qualifying_problem_count: count as u32,
        })
    }
}

impl KnowledgeDetailPort for DatabaseRuntime {
    async fn load_knowledge_detail(
        &self,
        knowledge_node_id: &str,
    ) -> Result<KnowledgeDetailProjection, KnowledgeIndexError> {
        self.rebuild_knowledge_relations().await?;
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeIndexError::PersistenceUnavailable)?;
        let node = load_knowledge_node(pool, knowledge_node_id)
            .await?
            .into_iter()
            .next()
            .ok_or(KnowledgeIndexError::KnowledgeNodeNotFound)?;
        let understanding_row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT current_level, historical_highest_level, first_reached_highest_local_date \
             FROM knowledge_understanding_states WHERE knowledge_node_id = ?1",
        )
        .bind(knowledge_node_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let understanding = understanding_row
            .map(|(current, highest, first_on)| {
                Ok(KnowledgeUnderstandingProjection {
                    knowledge_node_id: knowledge_node_id.to_owned(),
                    current: parse_understanding(&current)?,
                    historical_highest: parse_understanding(&highest)?,
                    first_reached_highest_on: acm_os_domain::LocalDate::parse_iso(&first_on)
                        .map_err(|_| KnowledgeIndexError::IntegrityViolation)?,
                })
            })
            .transpose()?;
        let incoming = load_incoming_knowledge_nodes(pool, knowledge_node_id).await?;
        let outgoing = load_outgoing_knowledge_nodes(pool, knowledge_node_id).await?;
        let problem_rows: Vec<(i64, i64, String, String)> = sqlx::query_as(
            "SELECT p.id, p.external_contest_key, p.external_problem_key, p.title \
             FROM knowledge_link_index l JOIN problems p ON CAST(l.source_id AS INTEGER) = p.id \
             WHERE l.source_kind = 'problem' AND l.resolution = 'resolved' \
               AND l.target_knowledge_node_id = ?1 \
             ORDER BY p.external_contest_key, p.external_problem_key, p.id",
        )
        .bind(knowledge_node_id)
        .fetch_all(pool)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let related_problems = problem_rows
            .into_iter()
            .map(|(problem_id, contest_id, index, title)| {
                let contest = acm_os_domain::CodeforcesContestIdentity::new(
                    u64::try_from(contest_id)
                        .map_err(|_| KnowledgeIndexError::IntegrityViolation)?,
                )
                .map_err(|_| KnowledgeIndexError::IntegrityViolation)?;
                let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, index)
                    .map_err(|_| KnowledgeIndexError::IntegrityViolation)?;
                Ok(RelatedKnowledgeProblemProjection {
                    problem_id: problem_id.to_string(),
                    problem,
                    title,
                })
            })
            .collect::<Result<Vec<_>, KnowledgeIndexError>>()?;
        Ok(KnowledgeDetailProjection {
            node,
            understanding,
            incoming,
            outgoing,
            related_problems,
        })
    }
}

impl KnowledgeCandidatePort for DatabaseRuntime {
    async fn list_knowledge_candidates(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<Vec<KnowledgeCandidateProjection>, KnowledgeCandidateError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeCandidateError::PersistenceUnavailable)?;
        let (problem_id, identity_type) = candidate_problem_row(pool, problem).await?;
        if identity_type != "personal" {
            return Err(KnowledgeCandidateError::NotPersonal);
        }
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT c.fingerprint, c.target_ref, c.disposition FROM knowledge_candidate_records c \
             WHERE c.problem_id = ?1 AND NOT EXISTS (SELECT 1 FROM knowledge_link_index l \
               WHERE l.source_kind = 'problem' AND l.source_id = CAST(c.problem_id AS TEXT) \
               AND l.target_ref = c.target_ref AND l.resolution = 'resolved') \
             ORDER BY c.fingerprint",
        )
        .bind(problem_id)
        .fetch_all(pool)
        .await
        .map_err(|_| KnowledgeCandidateError::PersistenceUnavailable)?;
        rows.into_iter()
            .map(|(fingerprint, target_ref, disposition)| {
                Ok(KnowledgeCandidateProjection {
                    problem: problem.clone(),
                    fingerprint,
                    target_ref,
                    disposition: parse_candidate_disposition(&disposition)?,
                })
            })
            .collect()
    }

    async fn register_knowledge_candidate(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        fingerprint: &str,
        target_ref: &str,
    ) -> Result<KnowledgeCandidateProjection, KnowledgeCandidateError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeCandidateError::PersistenceUnavailable)?;
        let (problem_id, identity_type) = candidate_problem_row(pool, problem).await?;
        if identity_type != "personal" {
            return Err(KnowledgeCandidateError::NotPersonal);
        }
        sqlx::query(
            "INSERT INTO knowledge_candidate_records \
                (problem_id, fingerprint, target_ref, disposition) \
             VALUES (?1, ?2, ?3, 'pending') \
             ON CONFLICT(problem_id, fingerprint) DO UPDATE SET \
                target_ref = excluded.target_ref, \
                updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        )
        .bind(problem_id)
        .bind(fingerprint)
        .bind(target_ref)
        .execute(pool)
        .await
        .map_err(|_| KnowledgeCandidateError::PersistenceUnavailable)?;
        load_candidate_projection(pool, problem, problem_id, fingerprint).await
    }

    async fn set_knowledge_candidate_disposition(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        fingerprint: &str,
        disposition: KnowledgeCandidateDisposition,
    ) -> Result<KnowledgeCandidateProjection, KnowledgeCandidateError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeCandidateError::PersistenceUnavailable)?;
        let (problem_id, identity_type) = candidate_problem_row(pool, problem).await?;
        if identity_type != "personal" {
            return Err(KnowledgeCandidateError::NotPersonal);
        }
        let result = sqlx::query(
            "UPDATE knowledge_candidate_records SET disposition = ?1, \
                updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE problem_id = ?2 AND fingerprint = ?3",
        )
        .bind(candidate_disposition_value(disposition))
        .bind(problem_id)
        .bind(fingerprint)
        .execute(pool)
        .await
        .map_err(|_| KnowledgeCandidateError::PersistenceUnavailable)?;
        if result.rows_affected() == 0 {
            return Err(KnowledgeCandidateError::CandidateNotFound);
        }
        load_candidate_projection(pool, problem, problem_id, fingerprint).await
    }

    async fn accept_existing_knowledge_candidate(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        fingerprint: &str,
        knowledge_node_id: &str,
    ) -> Result<AcceptedKnowledgeCandidateProjection, KnowledgeCandidateError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(KnowledgeCandidateError::PersistenceUnavailable)?;
        let (problem_id, identity_type) = candidate_problem_row(pool, problem).await?;
        if identity_type != "personal" {
            return Err(KnowledgeCandidateError::NotPersonal);
        }
        let candidate = load_candidate_projection(pool, problem, problem_id, fingerprint).await?;
        if candidate.disposition == KnowledgeCandidateDisposition::Ignored {
            return Err(KnowledgeCandidateError::IntegrityViolation);
        }
        let index = self
            .rebuild_knowledge_index()
            .await
            .map_err(map_knowledge_to_candidate_error)?;
        let matches = index
            .nodes
            .iter()
            .filter(|node| candidate_target_matches(&candidate.target_ref, node))
            .collect::<Vec<_>>();
        let [target] = matches.as_slice() else {
            return Err(KnowledgeCandidateError::IntegrityViolation);
        };
        if target.knowledge_node_id != knowledge_node_id {
            return Err(KnowledgeCandidateError::IntegrityViolation);
        }
        acm_os_application::add_prerequisite_link(self, problem, candidate.target_ref.clone())
            .await
            .map_err(map_patch_to_candidate_error)?;
        self.rebuild_knowledge_relations()
            .await
            .map_err(map_knowledge_to_candidate_error)?;
        let verified: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM knowledge_link_index WHERE source_kind = 'problem' \
             AND source_id = ?1 AND target_knowledge_node_id = ?2 AND resolution = 'resolved'",
        )
        .bind(problem_id.to_string())
        .bind(knowledge_node_id)
        .fetch_one(pool)
        .await
        .map_err(|_| KnowledgeCandidateError::PersistenceUnavailable)?;
        if verified != 1 {
            return Err(KnowledgeCandidateError::IntegrityViolation);
        }
        Ok(AcceptedKnowledgeCandidateProjection {
            knowledge_node_id: knowledge_node_id.to_owned(),
            target_ref: candidate.target_ref,
        })
    }
}

fn candidate_target_matches(target_ref: &str, node: &KnowledgeNodeProjection) -> bool {
    if target_ref.contains('/') {
        node.vault_relative_path
            .strip_suffix(".md")
            .is_some_and(|path| path.eq_ignore_ascii_case(target_ref))
    } else {
        node.display_name.eq_ignore_ascii_case(target_ref)
    }
}

fn map_knowledge_to_candidate_error(_: KnowledgeIndexError) -> KnowledgeCandidateError {
    KnowledgeCandidateError::IntegrityViolation
}

fn map_patch_to_candidate_error(error: PersonalNotePatchError) -> KnowledgeCandidateError {
    match error {
        PersonalNotePatchError::PersistenceUnavailable => {
            KnowledgeCandidateError::PersistenceUnavailable
        }
        PersonalNotePatchError::ProblemNotFound => KnowledgeCandidateError::ProblemNotFound,
        PersonalNotePatchError::NotPersonal => KnowledgeCandidateError::NotPersonal,
        _ => KnowledgeCandidateError::IntegrityViolation,
    }
}

async fn candidate_problem_row(
    pool: &SqlitePool,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<(i64, String), KnowledgeCandidateError> {
    sqlx::query_as(
        "SELECT id, identity_type FROM problems WHERE platform = 'codeforces' \
         AND external_contest_key = ?1 AND external_problem_key = ?2",
    )
    .bind(problem.contest().contest_id() as i64)
    .bind(problem.index())
    .fetch_optional(pool)
    .await
    .map_err(|_| KnowledgeCandidateError::PersistenceUnavailable)?
    .ok_or(KnowledgeCandidateError::ProblemNotFound)
}

async fn load_candidate_projection(
    pool: &SqlitePool,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
    problem_id: i64,
    fingerprint: &str,
) -> Result<KnowledgeCandidateProjection, KnowledgeCandidateError> {
    let row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT fingerprint, target_ref, disposition FROM knowledge_candidate_records \
         WHERE problem_id = ?1 AND fingerprint = ?2",
    )
    .bind(problem_id)
    .bind(fingerprint)
    .fetch_optional(pool)
    .await
    .map_err(|_| KnowledgeCandidateError::PersistenceUnavailable)?;
    let (fingerprint, target_ref, disposition) =
        row.ok_or(KnowledgeCandidateError::CandidateNotFound)?;
    Ok(KnowledgeCandidateProjection {
        problem: problem.clone(),
        fingerprint,
        target_ref,
        disposition: parse_candidate_disposition(&disposition)?,
    })
}

fn candidate_disposition_value(disposition: KnowledgeCandidateDisposition) -> &'static str {
    match disposition {
        KnowledgeCandidateDisposition::Pending => "pending",
        KnowledgeCandidateDisposition::AcceptedIntent => "accepted_intent",
        KnowledgeCandidateDisposition::Ignored => "ignored",
    }
}

fn parse_candidate_disposition(
    value: &str,
) -> Result<KnowledgeCandidateDisposition, KnowledgeCandidateError> {
    match value {
        "pending" => Ok(KnowledgeCandidateDisposition::Pending),
        "accepted_intent" => Ok(KnowledgeCandidateDisposition::AcceptedIntent),
        "ignored" => Ok(KnowledgeCandidateDisposition::Ignored),
        _ => Err(KnowledgeCandidateError::IntegrityViolation),
    }
}

type KnowledgeNodeRow = (String, String, String, String, Option<String>);

async fn load_knowledge_node(
    pool: &SqlitePool,
    knowledge_node_id: &str,
) -> Result<Vec<KnowledgeNodeProjection>, KnowledgeIndexError> {
    let rows: Vec<KnowledgeNodeRow> = sqlx::query_as(
        "SELECT i.knowledge_node_id, i.display_name, i.vault_relative_path, \
                i.content_digest, b.windows_file_key \
         FROM knowledge_discovery_index i JOIN knowledge_file_bindings b \
           ON b.knowledge_node_id = i.knowledge_node_id \
         WHERE i.knowledge_node_id = ?1",
    )
    .bind(knowledge_node_id)
    .fetch_all(pool)
    .await
    .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
    Ok(map_knowledge_node_rows(rows))
}

async fn load_incoming_knowledge_nodes(
    pool: &SqlitePool,
    knowledge_node_id: &str,
) -> Result<Vec<KnowledgeNodeProjection>, KnowledgeIndexError> {
    let rows: Vec<KnowledgeNodeRow> = sqlx::query_as(
        "SELECT i.knowledge_node_id, i.display_name, i.vault_relative_path, \
                i.content_digest, b.windows_file_key \
         FROM knowledge_discovery_index i JOIN knowledge_file_bindings b \
           ON b.knowledge_node_id = i.knowledge_node_id \
         WHERE EXISTS (SELECT 1 FROM knowledge_link_index l \
           WHERE l.source_kind = 'knowledge' AND l.resolution = 'resolved' \
             AND l.target_knowledge_node_id = ?1 AND l.source_id = i.knowledge_node_id) \
         ORDER BY lower(i.display_name), i.knowledge_node_id",
    )
    .bind(knowledge_node_id)
    .fetch_all(pool)
    .await
    .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
    Ok(map_knowledge_node_rows(rows))
}

async fn load_outgoing_knowledge_nodes(
    pool: &SqlitePool,
    knowledge_node_id: &str,
) -> Result<Vec<KnowledgeNodeProjection>, KnowledgeIndexError> {
    let rows: Vec<KnowledgeNodeRow> = sqlx::query_as(
        "SELECT i.knowledge_node_id, i.display_name, i.vault_relative_path, \
                i.content_digest, b.windows_file_key \
         FROM knowledge_discovery_index i JOIN knowledge_file_bindings b \
           ON b.knowledge_node_id = i.knowledge_node_id \
         WHERE EXISTS (SELECT 1 FROM knowledge_link_index l \
           WHERE l.source_kind = 'knowledge' AND l.resolution = 'resolved' \
             AND l.source_id = ?1 AND l.target_knowledge_node_id = i.knowledge_node_id) \
         ORDER BY lower(i.display_name), i.knowledge_node_id",
    )
    .bind(knowledge_node_id)
    .fetch_all(pool)
    .await
    .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
    Ok(map_knowledge_node_rows(rows))
}

fn map_knowledge_node_rows(rows: Vec<KnowledgeNodeRow>) -> Vec<KnowledgeNodeProjection> {
    rows.into_iter()
        .map(
            |(
                knowledge_node_id,
                display_name,
                vault_relative_path,
                content_digest,
                windows_file_key,
            )| KnowledgeNodeProjection {
                knowledge_node_id,
                display_name,
                vault_relative_path,
                content_digest,
                windows_file_key,
                location_state: KnowledgeLocationState::Ready,
            },
        )
        .collect()
}

fn understanding_value(value: acm_os_domain::KnowledgeUnderstandingLevel) -> &'static str {
    match value {
        acm_os_domain::KnowledgeUnderstandingLevel::NotLearned => "not_learned",
        acm_os_domain::KnowledgeUnderstandingLevel::Vague => "vague",
        acm_os_domain::KnowledgeUnderstandingLevel::Basic => "basic",
        acm_os_domain::KnowledgeUnderstandingLevel::Proficient => "proficient",
        acm_os_domain::KnowledgeUnderstandingLevel::Deep => "deep",
    }
}
fn parse_understanding(
    value: &str,
) -> Result<acm_os_domain::KnowledgeUnderstandingLevel, KnowledgeIndexError> {
    match value {
        "not_learned" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::NotLearned),
        "vague" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Vague),
        "basic" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Basic),
        "proficient" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Proficient),
        "deep" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Deep),
        _ => Err(KnowledgeIndexError::IntegrityViolation),
    }
}

fn resolved_binding(resolved: &ResolvedNoteFile) -> PersonalNoteBinding {
    PersonalNoteBinding {
        vault_relative_path: resolved.relative_path.clone(),
        content_digest: resolved.content_digest.clone(),
        windows_file_key: resolved.windows_file_key.clone(),
    }
}

impl ProblemLifecyclePort for DatabaseRuntime {
    async fn load_problem_lifecycle(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<ProblemLifecycleState, ProblemLifecycleError> {
        let row: Option<(
            String,
            String,
            String,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT p.identity_type, pls.learning_status, pls.learning_status_since_utc, \
                    rc.cycle_number, rc.stage, rc.schedule_rule_version, rc.next_due_local_date \
             FROM problems p \
             LEFT JOIN problem_learning_states pls ON pls.problem_id = p.id \
             LEFT JOIN review_cycles rc ON rc.problem_id = p.id AND rc.cycle_status = 'active' \
             WHERE p.platform = 'codeforces' \
               AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(
            self._pool
                .as_ref()
                .ok_or(ProblemLifecycleError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
        let (identity_type, status, since, cycle_number, stage, rule_version, due) =
            row.ok_or(ProblemLifecycleError::ProblemNotFound)?;
        let identity_type = parse_problem_identity_type(&identity_type)
            .map_err(|_| ProblemLifecycleError::IntegrityViolation)?;
        let learning_status = parse_learning_status(&status)?;
        if since.is_empty() {
            return Err(ProblemLifecycleError::IntegrityViolation);
        }
        let active_review_cycle = match (cycle_number, stage, rule_version, due) {
            (None, None, None, None) => None,
            (Some(cycle_number), Some(stage), Some(rule_version), Some(due)) => {
                Some(ActiveReviewCycle {
                    cycle_number: u32::try_from(cycle_number)
                        .map_err(|_| ProblemLifecycleError::IntegrityViolation)?,
                    stage: u32::try_from(stage)
                        .map_err(|_| ProblemLifecycleError::IntegrityViolation)?,
                    schedule_rule_version: u32::try_from(rule_version)
                        .map_err(|_| ProblemLifecycleError::IntegrityViolation)?,
                    next_due_local_date: acm_os_domain::LocalDate::parse_iso(&due)
                        .map_err(|_| ProblemLifecycleError::IntegrityViolation)?,
                })
            }
            _ => return Err(ProblemLifecycleError::IntegrityViolation),
        };
        if matches!(
            learning_status,
            acm_os_domain::LearningStatus::WaitingColdStart
                | acm_os_domain::LearningStatus::LongTermReview
        ) != active_review_cycle.is_some()
        {
            return Err(ProblemLifecycleError::IntegrityViolation);
        }
        Ok(ProblemLifecycleState {
            identity_type,
            learning_status,
            learning_status_since_utc: since,
            active_review_cycle,
        })
    }

    async fn commit_problem_lifecycle_decision(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        decision: acm_os_domain::ProblemLifecycleDecision,
        first_due: Option<acm_os_domain::LocalDate>,
    ) -> Result<ProblemLifecycleState, ProblemLifecycleError> {
        use acm_os_domain::ReviewCycleDirective::{CancelActive, None, StartFirstColdStart};

        if (decision.review_cycle == StartFirstColdStart) != first_due.is_some() {
            return Err(ProblemLifecycleError::IntegrityViolation);
        }
        let pool = self
            ._pool
            .as_ref()
            .ok_or(ProblemLifecycleError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
        let row: Option<(i64, String, String)> = sqlx::query_as(
            "SELECT p.id, p.identity_type, pls.learning_status \
             FROM problems p \
             LEFT JOIN problem_learning_states pls ON pls.problem_id = p.id \
             WHERE p.platform = 'codeforces' \
               AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
        let (problem_id, identity_type, current_status) =
            row.ok_or(ProblemLifecycleError::ProblemNotFound)?;
        if identity_type != "personal" {
            return Err(ProblemLifecycleError::NotPersonal);
        }
        if parse_learning_status(&current_status)? != decision.previous_status {
            return Err(ProblemLifecycleError::InvalidTransition);
        }
        let in_progress_review: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_attempts \
             WHERE problem_id = ?1 AND attempt_status = 'in_progress'",
        )
        .bind(problem_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
        if in_progress_review != 0 {
            return Err(ProblemLifecycleError::InvalidTransition);
        }

        match decision.review_cycle {
            StartFirstColdStart => {
                let cycle_number: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(cycle_number), 0) + 1 FROM review_cycles WHERE problem_id = ?1",
                )
                .bind(problem_id)
                .fetch_one(&mut *transaction)
                .await
                .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
                sqlx::query(
                    "INSERT INTO review_cycles \
                        (id, problem_id, cycle_number, cycle_status, stage, schedule_rule_version, next_due_local_date) \
                     VALUES (?1, ?2, ?3, 'active', 0, ?4, ?5)",
                )
                .bind(uuid::Uuid::now_v7().to_string())
                .bind(problem_id)
                .bind(cycle_number)
                .bind(i64::from(acm_os_domain::ReviewSchedulingEngine::SCHEDULE_RULE_VERSION))
                .bind(first_due.expect("validated first due").to_iso_string())
                .execute(&mut *transaction)
                .await
                .map_err(|error| {
                    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
                        ProblemLifecycleError::IntegrityViolation
                    } else {
                        ProblemLifecycleError::PersistenceUnavailable
                    }
                })?;
            }
            CancelActive => {
                let result = sqlx::query(
                    "UPDATE review_cycles \
                     SET cycle_status = 'cancelled', next_due_local_date = NULL, \
                         ended_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
                     WHERE problem_id = ?1 AND cycle_status = 'active'",
                )
                .bind(problem_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
                if decision.previous_status == acm_os_domain::LearningStatus::WaitingColdStart
                    && result.rows_affected() != 1
                {
                    return Err(ProblemLifecycleError::IntegrityViolation);
                }
            }
            None => {}
        }

        let update = sqlx::query(
            "UPDATE problem_learning_states \
             SET learning_status = ?1, \
                 learning_status_since_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE problem_id = ?2 AND learning_status = ?3",
        )
        .bind(learning_status_value(decision.next_status))
        .bind(problem_id)
        .bind(learning_status_value(decision.previous_status))
        .execute(&mut *transaction)
        .await
        .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
        if update.rows_affected() != 1 {
            return Err(ProblemLifecycleError::InvalidTransition);
        }
        transaction
            .commit()
            .await
            .map_err(|_| ProblemLifecycleError::PersistenceUnavailable)?;
        self.load_problem_lifecycle(problem).await
    }
}

fn parse_learning_status(
    value: &str,
) -> Result<acm_os_domain::LearningStatus, ProblemLifecycleError> {
    match value {
        "unstarted" => Ok(acm_os_domain::LearningStatus::Unstarted),
        "upsolve_pending" => Ok(acm_os_domain::LearningStatus::UpsolvePending),
        "learning" => Ok(acm_os_domain::LearningStatus::Learning),
        "waiting_cold_start" => Ok(acm_os_domain::LearningStatus::WaitingColdStart),
        "relearning" => Ok(acm_os_domain::LearningStatus::Relearning),
        "long_term_review" => Ok(acm_os_domain::LearningStatus::LongTermReview),
        _ => Err(ProblemLifecycleError::IntegrityViolation),
    }
}

fn learning_status_value(value: acm_os_domain::LearningStatus) -> &'static str {
    match value {
        acm_os_domain::LearningStatus::Unstarted => "unstarted",
        acm_os_domain::LearningStatus::UpsolvePending => "upsolve_pending",
        acm_os_domain::LearningStatus::Learning => "learning",
        acm_os_domain::LearningStatus::WaitingColdStart => "waiting_cold_start",
        acm_os_domain::LearningStatus::Relearning => "relearning",
        acm_os_domain::LearningStatus::LongTermReview => "long_term_review",
    }
}

impl WeeklyAcmBudgetPort for DatabaseRuntime {
    async fn load_weekly_acm_budget(&self) -> Result<WeeklyAcmBudgetSchedule, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let rows: Vec<(i64, i64)> = sqlx::query_as(
            "SELECT weekday, budget_minutes FROM weekly_acm_budgets ORDER BY weekday",
        )
        .fetch_all(pool)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let mut values = [None; 7];
        for (weekday, minutes) in rows {
            let index =
                usize::try_from(weekday - 1).map_err(|_| TodaySnapshotError::IntegrityViolation)?;
            let slot = values
                .get_mut(index)
                .ok_or(TodaySnapshotError::IntegrityViolation)?;
            if slot.is_some() {
                return Err(TodaySnapshotError::IntegrityViolation);
            }
            *slot =
                Some(u32::try_from(minutes).map_err(|_| TodaySnapshotError::IntegrityViolation)?);
        }
        Ok(WeeklyAcmBudgetSchedule {
            monday: values[0],
            tuesday: values[1],
            wednesday: values[2],
            thursday: values[3],
            friday: values[4],
            saturday: values[5],
            sunday: values[6],
        })
    }

    async fn save_weekly_acm_budget(
        &self,
        schedule: &WeeklyAcmBudgetSchedule,
    ) -> Result<WeeklyAcmBudgetSchedule, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        sqlx::query("DELETE FROM weekly_acm_budgets")
            .execute(&mut *transaction)
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        for (weekday, minutes) in [
            schedule.monday,
            schedule.tuesday,
            schedule.wednesday,
            schedule.thursday,
            schedule.friday,
            schedule.saturday,
            schedule.sunday,
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(minutes) = minutes {
                sqlx::query(
                    "INSERT INTO weekly_acm_budgets (weekday, budget_minutes) VALUES (?1, ?2)",
                )
                .bind(
                    i64::try_from(weekday + 1)
                        .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                )
                .bind(i64::from(minutes))
                .execute(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
            }
        }
        transaction
            .commit()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        Ok(schedule.clone())
    }
}

impl TodaySnapshotPort for DatabaseRuntime {
    async fn load_today_snapshot(
        &self,
        local_date: acm_os_domain::LocalDate,
    ) -> Result<Option<TodaySnapshot>, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let plan: Option<(String, String, i64, i64, i64, i64)> = sqlx::query_as(
            "SELECT id, local_date, budget_minutes, planned_minutes, over_budget_minutes, \
                    review_only_streak \
             FROM today_plans WHERE local_date = ?1",
        )
        .bind(local_date.to_iso_string())
        .fetch_optional(pool)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let Some((plan_id, stored_date, budget, planned, over_budget, streak)) = plan else {
            return Ok(None);
        };
        let rows: Vec<(
            String,
            String,
            i64,
            String,
            String,
            Option<String>,
            String,
            String,
            i64,
            i64,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT e.id, CAST(e.problem_id AS TEXT), p.external_contest_key, \
                    p.external_problem_key, p.title, e.review_attempt_id, e.lane, e.reason, \
                    e.planning_cost_minutes, e.position, e.entry_origin, e.entry_status \
             FROM today_plan_entries e JOIN problems p ON p.id = e.problem_id \
             WHERE e.today_plan_id = ?1 ORDER BY e.position",
        )
        .bind(&plan_id)
        .fetch_all(pool)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let entries = rows
            .into_iter()
            .map(
                |(
                    entry_id,
                    problem_id,
                    contest_id,
                    problem_index,
                    problem_title,
                    review_attempt_id,
                    lane,
                    reason,
                    cost,
                    position,
                    origin,
                    status,
                )| {
                    Ok(TodaySnapshotEntry {
                        entry_id,
                        problem_id,
                        contest_id: u64::try_from(contest_id)
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        problem_index,
                        problem_title,
                        review_attempt_id,
                        lane: parse_today_lane(&lane)?,
                        reason: parse_today_reason(&reason)?,
                        planning_cost_minutes: u32::try_from(cost)
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        position: u32::try_from(position)
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        origin: parse_today_entry_origin(&origin)?,
                        status: parse_today_entry_status(&status)?,
                    })
                },
            )
            .collect::<Result<Vec<_>, TodaySnapshotError>>()?;
        let stored_date = acm_os_domain::LocalDate::parse_iso(&stored_date)
            .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
        if stored_date != local_date
            || entries
                .iter()
                .enumerate()
                .any(|(position, entry)| entry.position as usize != position)
        {
            return Err(TodaySnapshotError::IntegrityViolation);
        }
        Ok(Some(TodaySnapshot {
            plan_id,
            local_date: stored_date,
            budget_minutes: u32::try_from(budget)
                .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
            planned_minutes: u32::try_from(planned)
                .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
            over_budget_minutes: u32::try_from(over_budget)
                .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
            review_only_streak: u8::try_from(streak)
                .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
            entries,
        }))
    }

    async fn load_today_generation_context(
        &self,
        local_date: acm_os_domain::LocalDate,
    ) -> Result<TodayGenerationContext, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let rows: Vec<(String, i64, String, String, String, String, i64, Option<String>, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                "SELECT CAST(p.id AS TEXT), p.external_contest_key, p.external_problem_key, p.title, pls.learning_status, \
                        substr(pls.learning_status_since_utc, 1, 10), pls.pinned_priority, \
                        rc.next_due_local_date, ra.id, ra.scheduled_due_local_date, fb.binding_state \
                 FROM problems p \
                 JOIN problem_learning_states pls ON pls.problem_id = p.id \
                 JOIN file_bindings fb ON fb.problem_id = p.id \
                 LEFT JOIN review_cycles rc ON rc.problem_id = p.id AND rc.cycle_status = 'active' \
                 LEFT JOIN review_attempts ra ON ra.problem_id = p.id AND ra.attempt_status = 'in_progress' \
                 WHERE p.identity_type = 'personal' ORDER BY p.id",
            )
            .fetch_all(pool)
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let candidates = rows
            .into_iter()
            .map(
                |(
                    problem_id,
                    contest_id,
                    problem_index,
                    problem_title,
                    status,
                    since,
                    pinned,
                    due,
                    attempt_id,
                    attempt_due,
                    binding_state,
                )| {
                    Ok(TodayGenerationCandidate {
                        problem_id,
                        contest_id: u64::try_from(contest_id)
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        problem_index,
                        problem_title,
                        learning_status: parse_learning_status(&status)
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        learning_status_since: acm_os_domain::LocalDate::parse_iso(&since)
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        pinned: match pinned {
                            0 => false,
                            1 => true,
                            _ => return Err(TodaySnapshotError::IntegrityViolation),
                        },
                        active_review_due: due
                            .map(|value| acm_os_domain::LocalDate::parse_iso(&value))
                            .transpose()
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        in_progress_review_attempt_id: attempt_id,
                        in_progress_review_due: attempt_due
                            .map(|value| acm_os_domain::LocalDate::parse_iso(&value))
                            .transpose()
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                        available_for_today: binding_state == "linked",
                    })
                },
            )
            .collect::<Result<Vec<_>, TodaySnapshotError>>()?;
        let prior_streak: Option<i64> = sqlx::query_scalar(
            "SELECT review_only_streak FROM today_plans WHERE local_date < ?1 \
             ORDER BY local_date DESC LIMIT 1",
        )
        .bind(local_date.to_iso_string())
        .fetch_optional(pool)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        Ok(TodayGenerationContext {
            candidates,
            prior_review_only_streak: u8::try_from(prior_streak.unwrap_or(0))
                .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
        })
    }

    async fn reconcile_today_snapshot(
        &self,
        local_date: acm_os_domain::LocalDate,
    ) -> Result<TodaySnapshot, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let learning_entries: Vec<(i64, String)> = sqlx::query_as(
            "SELECT p.external_contest_key, p.external_problem_key \
             FROM today_plans tp \
             JOIN today_plan_entries e ON e.today_plan_id = tp.id \
             JOIN problems p ON p.id = e.problem_id \
             WHERE tp.local_date = ?1 AND e.entry_status != 'completed' \
               AND e.reason IN ('continue_learning', 'relearn', 'upsolve') \
             ORDER BY e.position",
        )
        .bind(local_date.to_iso_string())
        .fetch_all(pool)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        for (contest_id, index) in learning_entries {
            let contest = acm_os_domain::CodeforcesContestIdentity::new(
                u64::try_from(contest_id).map_err(|_| TodaySnapshotError::IntegrityViolation)?,
            )
            .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
            let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, index)
                .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
            match self.read_personal_note_projection(&problem).await {
                Ok(PersonalNoteReadState::Ready { .. })
                | Ok(PersonalNoteReadState::LocationAnomaly { .. })
                | Ok(PersonalNoteReadState::VaultUnavailable { .. })
                | Err(PersonalNoteReadError::BindingUnavailable)
                | Err(PersonalNoteReadError::FileReadFailed)
                | Err(PersonalNoteReadError::InvalidUtf8) => {}
                Err(PersonalNoteReadError::ProblemNotFound)
                | Err(PersonalNoteReadError::NotPersonal) => {
                    return Err(TodaySnapshotError::IntegrityViolation);
                }
                Err(PersonalNoteReadError::PersistenceUnavailable) => {
                    return Err(TodaySnapshotError::PersistenceUnavailable);
                }
            }
        }
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let plan_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM today_plans WHERE local_date = ?1")
                .bind(local_date.to_iso_string())
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let plan_id = plan_id.ok_or(TodaySnapshotError::IntegrityViolation)?;

        sqlx::query(
            "DELETE FROM today_plan_entries \
             WHERE today_plan_id = ?1 AND entry_origin = 'auto' \
               AND reason = 'continue_review' AND reconciliation_added = 1 \
               AND EXISTS (SELECT 1 FROM review_attempts ra \
                   WHERE ra.id = today_plan_entries.review_attempt_id \
                     AND ra.attempt_status = 'void')",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        sqlx::query(
            "UPDATE today_plan_entries SET lane = 'review', \
                    reason = CASE pls.learning_status \
                        WHEN 'waiting_cold_start' THEN 'due_first_cold_start' \
                        WHEN 'long_term_review' THEN 'due_long_term_review' END, \
                    entry_status = 'not_started', review_attempt_id = NULL \
             FROM today_plans tp, problem_learning_states pls, review_cycles rc, review_attempts ra \
             WHERE today_plan_entries.today_plan_id = ?1 \
               AND tp.id = today_plan_entries.today_plan_id \
               AND pls.problem_id = today_plan_entries.problem_id \
               AND rc.problem_id = today_plan_entries.problem_id AND rc.cycle_status = 'active' \
               AND ra.id = today_plan_entries.review_attempt_id AND ra.attempt_status = 'void' \
               AND today_plan_entries.reason = 'continue_review' \
               AND today_plan_entries.reconciliation_added = 0 \
               AND rc.next_due_local_date <= tp.local_date",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        sqlx::query(
            "DELETE FROM today_plan_entries \
             WHERE today_plan_id = ?1 AND reason = 'continue_review' \
               AND reconciliation_added = 0 \
               AND EXISTS (SELECT 1 FROM review_attempts ra \
                   WHERE ra.id = today_plan_entries.review_attempt_id \
                     AND ra.attempt_status = 'void')",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let remaining_entry_ids: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM today_plan_entries WHERE today_plan_id = ?1 ORDER BY position, id",
        )
        .bind(&plan_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        for (position, entry_id) in remaining_entry_ids.iter().enumerate() {
            sqlx::query("UPDATE today_plan_entries SET position = ?2 WHERE id = ?1")
                .bind(entry_id)
                .bind(
                    1_000_000_i64
                        + i64::try_from(position)
                            .map_err(|_| TodaySnapshotError::IntegrityViolation)?,
                )
                .execute(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        }
        for (position, entry_id) in remaining_entry_ids.iter().enumerate() {
            sqlx::query("UPDATE today_plan_entries SET position = ?2 WHERE id = ?1")
                .bind(entry_id)
                .bind(i64::try_from(position).map_err(|_| TodaySnapshotError::IntegrityViolation)?)
                .execute(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        }
        sqlx::query(
            "UPDATE today_plan_entries SET entry_status = 'not_started', review_attempt_id = NULL \
             WHERE today_plan_id = ?1 AND review_attempt_id IS NOT NULL \
               AND EXISTS (SELECT 1 FROM review_attempts ra \
                   WHERE ra.id = today_plan_entries.review_attempt_id \
                     AND ra.attempt_status = 'void')",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;

        sqlx::query(
            "UPDATE today_plan_entries SET entry_status = 'completed' \
             WHERE today_plan_id = ?1 AND review_attempt_id IS NOT NULL \
               AND entry_status != 'completed' \
               AND EXISTS (SELECT 1 FROM review_attempts ra \
                   WHERE ra.id = today_plan_entries.review_attempt_id \
                     AND ra.attempt_status = 'completed')",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;

        sqlx::query(
            "UPDATE today_plan_entries SET entry_status = 'in_progress', \
                    review_attempt_id = (SELECT ra.id FROM review_attempts ra \
                        WHERE ra.problem_id = today_plan_entries.problem_id \
                          AND ra.attempt_status = 'in_progress') \
             WHERE today_plan_id = ?1 AND entry_status = 'not_started' \
               AND EXISTS (SELECT 1 FROM review_attempts ra \
                   WHERE ra.problem_id = today_plan_entries.problem_id \
                     AND ra.attempt_status = 'in_progress')",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;

        sqlx::query(
            "UPDATE today_plan_entries SET entry_status = 'unavailable' \
             WHERE today_plan_id = ?1 AND entry_status != 'completed' \
               AND reason IN ('continue_learning', 'relearn', 'upsolve') \
               AND EXISTS (SELECT 1 FROM file_bindings fb \
                   WHERE fb.problem_id = today_plan_entries.problem_id \
                     AND fb.binding_state IN ('external_source_unavailable', 'location_anomaly'))",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        sqlx::query(
            "UPDATE today_plan_entries SET entry_status = \
                 CASE reason WHEN 'continue_learning' THEN 'in_progress' ELSE 'not_started' END \
             WHERE today_plan_id = ?1 AND entry_status = 'unavailable' \
               AND reason IN ('continue_learning', 'relearn', 'upsolve') \
               AND EXISTS (SELECT 1 FROM file_bindings fb \
                   WHERE fb.problem_id = today_plan_entries.problem_id \
                     AND fb.binding_state = 'linked')",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;

        let next_position: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM today_plan_entries \
             WHERE today_plan_id = ?1",
        )
        .bind(&plan_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let started: Vec<(i64, String)> = sqlx::query_as(
            "SELECT ra.problem_id, ra.id FROM review_attempts ra \
             JOIN problems p ON p.id = ra.problem_id \
             WHERE ra.attempt_status = 'in_progress' AND p.identity_type = 'personal' \
               AND NOT EXISTS (SELECT 1 FROM today_plan_entries e \
                   WHERE e.today_plan_id = ?1 AND e.problem_id = ra.problem_id) \
             ORDER BY ra.started_at_utc, ra.problem_id",
        )
        .bind(&plan_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        for (offset, (problem_id, attempt_id)) in started.into_iter().enumerate() {
            sqlx::query(
                "INSERT INTO today_plan_entries \
                    (id, today_plan_id, problem_id, review_attempt_id, lane, reason, \
                     planning_cost_minutes, position, entry_origin, entry_status, reconciliation_added) \
                 VALUES (?1, ?2, ?3, ?4, 'carry_in', 'continue_review', 30, ?5, 'auto', 'in_progress', 1)",
            )
            .bind(uuid::Uuid::now_v7().to_string())
            .bind(&plan_id)
            .bind(problem_id)
            .bind(attempt_id)
            .bind(next_position + i64::try_from(offset).map_err(|_| TodaySnapshotError::IntegrityViolation)?)
            .execute(&mut *transaction)
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        }
        sqlx::query(
            "UPDATE today_plans SET \
                 planned_minutes = COALESCE((SELECT SUM(planning_cost_minutes) \
                     FROM today_plan_entries e WHERE e.today_plan_id = today_plans.id), 0), \
                 over_budget_minutes = MAX(0, COALESCE((SELECT SUM(planning_cost_minutes) \
                     FROM today_plan_entries e WHERE e.today_plan_id = today_plans.id), 0) - budget_minutes) \
             WHERE id = ?1",
        )
        .bind(&plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;

        transaction
            .commit()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        self.load_today_snapshot(local_date)
            .await?
            .ok_or(TodaySnapshotError::IntegrityViolation)
    }

    async fn reorder_today_snapshot(
        &self,
        plan_id: &str,
        ordered_entry_ids: &[String],
    ) -> Result<TodaySnapshot, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let local_date: Option<String> =
            sqlx::query_scalar("SELECT local_date FROM today_plans WHERE id = ?1")
                .bind(plan_id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let local_date = local_date.ok_or(TodaySnapshotError::InvalidReorder)?;
        let stored_entry_ids: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM today_plan_entries WHERE today_plan_id = ?1 ORDER BY position",
        )
        .bind(plan_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        if stored_entry_ids.len() != ordered_entry_ids.len() {
            return Err(TodaySnapshotError::InvalidReorder);
        }
        let stored = stored_entry_ids
            .iter()
            .collect::<std::collections::HashSet<_>>();
        let requested = ordered_entry_ids
            .iter()
            .collect::<std::collections::HashSet<_>>();
        if stored.len() != stored_entry_ids.len()
            || requested.len() != ordered_entry_ids.len()
            || stored != requested
        {
            return Err(TodaySnapshotError::InvalidReorder);
        }

        for (position, entry_id) in ordered_entry_ids.iter().enumerate() {
            sqlx::query(
                "UPDATE today_plan_entries SET position = ?2 \
                 WHERE id = ?1 AND today_plan_id = ?3",
            )
            .bind(entry_id)
            .bind(
                1_000_000_i64
                    + i64::try_from(position).map_err(|_| TodaySnapshotError::InvalidReorder)?,
            )
            .bind(plan_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        }
        for (position, entry_id) in ordered_entry_ids.iter().enumerate() {
            let updated = sqlx::query(
                "UPDATE today_plan_entries SET position = ?2 \
                 WHERE id = ?1 AND today_plan_id = ?3",
            )
            .bind(entry_id)
            .bind(i64::try_from(position).map_err(|_| TodaySnapshotError::InvalidReorder)?)
            .bind(plan_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
            if updated.rows_affected() != 1 {
                return Err(TodaySnapshotError::InvalidReorder);
            }
        }
        transaction
            .commit()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let local_date = acm_os_domain::LocalDate::parse_iso(&local_date)
            .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
        self.load_today_snapshot(local_date)
            .await?
            .ok_or(TodaySnapshotError::IntegrityViolation)
    }

    async fn complete_today_entry(
        &self,
        plan_id: &str,
        entry_id: &str,
    ) -> Result<TodaySnapshot, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let local_date: Option<String> =
            sqlx::query_scalar("SELECT local_date FROM today_plans WHERE id = ?1")
                .bind(plan_id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let local_date = local_date.ok_or(TodaySnapshotError::InvalidTodayDone)?;
        let entry: Option<(String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT lane, reason, entry_status, review_attempt_id FROM today_plan_entries \
             WHERE id = ?1 AND today_plan_id = ?2",
        )
        .bind(entry_id)
        .bind(plan_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let (lane, reason, status, review_attempt_id) =
            entry.ok_or(TodaySnapshotError::InvalidTodayDone)?;
        let legal_learning_entry = matches!(
            (lane.as_str(), reason.as_str()),
            ("carry_in", "continue_learning") | ("study", "relearn" | "upsolve")
        );
        if !legal_learning_entry
            || review_attempt_id.is_some()
            || status == "unavailable"
            || !matches!(status.as_str(), "not_started" | "in_progress" | "completed")
        {
            return Err(TodaySnapshotError::InvalidTodayDone);
        }
        if status != "completed" {
            let updated = sqlx::query(
                "UPDATE today_plan_entries SET entry_status = 'completed' \
                 WHERE id = ?1 AND today_plan_id = ?2 AND entry_status = ?3",
            )
            .bind(entry_id)
            .bind(plan_id)
            .bind(&status)
            .execute(&mut *transaction)
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
            if updated.rows_affected() != 1 {
                return Err(TodaySnapshotError::InvalidTodayDone);
            }
        }
        transaction
            .commit()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let local_date = acm_os_domain::LocalDate::parse_iso(&local_date)
            .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
        self.load_today_snapshot(local_date)
            .await?
            .ok_or(TodaySnapshotError::IntegrityViolation)
    }

    async fn add_manual_today_entry(
        &self,
        expected_snapshot: &TodaySnapshot,
        suggestion: &acm_os_application::TodayExtraSuggestion,
    ) -> Result<TodaySnapshot, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let problem_id = suggestion
            .problem_id
            .parse::<i64>()
            .map_err(|_| TodaySnapshotError::InvalidExtraSuggestion)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let plan: Option<(String, i64, i64, i64, i64)> = sqlx::query_as(
            "SELECT local_date, budget_minutes, planned_minutes, over_budget_minutes, review_only_streak \
             FROM today_plans WHERE id = ?1",
        )
        .bind(&expected_snapshot.plan_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let plan = plan.ok_or(TodaySnapshotError::InvalidExtraSuggestion)?;
        if plan
            != (
                expected_snapshot.local_date.to_iso_string(),
                i64::from(expected_snapshot.budget_minutes),
                i64::from(expected_snapshot.planned_minutes),
                i64::from(expected_snapshot.over_budget_minutes),
                i64::from(expected_snapshot.review_only_streak),
            )
        {
            return Err(TodaySnapshotError::StaleExtraSuggestions);
        }
        let versions: Vec<(String, i64, String, String)> = sqlx::query_as(
            "SELECT id, position, entry_origin, entry_status FROM today_plan_entries \
             WHERE today_plan_id = ?1 ORDER BY position",
        )
        .bind(&expected_snapshot.plan_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        if versions.len() != expected_snapshot.entries.len()
            || versions.iter().zip(&expected_snapshot.entries).any(
                |((id, position, origin, status), expected)| {
                    id != &expected.entry_id
                        || *position != i64::from(expected.position)
                        || origin != today_entry_origin_value(expected.origin)
                        || status != today_entry_status_value(expected.status)
                },
            )
        {
            return Err(TodaySnapshotError::StaleExtraSuggestions);
        }
        if versions
            .iter()
            .any(|(_, _, _, status)| status != "completed")
            || suggestion.planning_cost_minutes
                > expected_snapshot
                    .budget_minutes
                    .saturating_sub(expected_snapshot.planned_minutes)
            || !matches!(
                (
                    suggestion.lane,
                    suggestion.reason,
                    suggestion.planning_cost_minutes
                ),
                (
                    acm_os_domain::TodayCandidateLane::CarryIn,
                    acm_os_domain::TodayCandidateReason::ContinueReview,
                    30
                ) | (
                    acm_os_domain::TodayCandidateLane::CarryIn,
                    acm_os_domain::TodayCandidateReason::ContinueLearning,
                    60
                ) | (
                    acm_os_domain::TodayCandidateLane::Review,
                    acm_os_domain::TodayCandidateReason::DueFirstColdStart
                        | acm_os_domain::TodayCandidateReason::DueLongTermReview,
                    30
                ) | (
                    acm_os_domain::TodayCandidateLane::Study,
                    acm_os_domain::TodayCandidateReason::Relearn
                        | acm_os_domain::TodayCandidateReason::Upsolve,
                    60
                )
            )
            || (suggestion.reason == acm_os_domain::TodayCandidateReason::ContinueReview)
                != suggestion.review_attempt_id.is_some()
        {
            return Err(TodaySnapshotError::InvalidExtraSuggestion);
        }
        let candidate_valid: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM problems p \
             JOIN problem_learning_states pls ON pls.problem_id = p.id \
             JOIN file_bindings fb ON fb.problem_id = p.id AND fb.binding_state = 'linked' \
             LEFT JOIN review_cycles rc ON rc.problem_id = p.id AND rc.cycle_status = 'active' \
             LEFT JOIN review_attempts ra ON ra.id = ?4 AND ra.problem_id = p.id \
             WHERE p.id = ?1 AND p.identity_type = 'personal' \
               AND NOT EXISTS (SELECT 1 FROM today_plan_entries e \
                   WHERE e.today_plan_id = ?2 AND e.problem_id = p.id) \
               AND ((?3 = 'continue_learning' AND pls.learning_status = 'learning') \
                 OR (?3 = 'relearn' AND pls.learning_status = 'relearning') \
                 OR (?3 = 'upsolve' AND pls.learning_status = 'upsolve_pending') \
                 OR (?3 = 'due_first_cold_start' AND pls.learning_status = 'waiting_cold_start' \
                     AND rc.next_due_local_date <= ?5 AND ?4 IS NULL) \
                 OR (?3 = 'due_long_term_review' AND pls.learning_status = 'long_term_review' \
                     AND rc.next_due_local_date <= ?5 AND ?4 IS NULL) \
                 OR (?3 = 'continue_review' AND ra.attempt_status = 'in_progress'))",
        )
        .bind(problem_id)
        .bind(&expected_snapshot.plan_id)
        .bind(today_reason_value(suggestion.reason))
        .bind(&suggestion.review_attempt_id)
        .bind(expected_snapshot.local_date.to_iso_string())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        if candidate_valid != 1 {
            return Err(TodaySnapshotError::InvalidExtraSuggestion);
        }
        let position = i64::try_from(expected_snapshot.entries.len())
            .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
        sqlx::query(
            "INSERT INTO today_plan_entries \
                (id, today_plan_id, problem_id, review_attempt_id, lane, reason, \
                 planning_cost_minutes, position, entry_origin, entry_status) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'manual', ?9)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(&expected_snapshot.plan_id)
        .bind(problem_id)
        .bind(&suggestion.review_attempt_id)
        .bind(today_lane_value(suggestion.lane))
        .bind(today_reason_value(suggestion.reason))
        .bind(i64::from(suggestion.planning_cost_minutes))
        .bind(position)
        .bind(
            if suggestion.lane == acm_os_domain::TodayCandidateLane::CarryIn {
                "in_progress"
            } else {
                "not_started"
            },
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::InvalidExtraSuggestion)?;
        sqlx::query(
            "UPDATE today_plans SET planned_minutes = planned_minutes + ?2, \
                 over_budget_minutes = MAX(0, planned_minutes + ?2 - budget_minutes) \
             WHERE id = ?1",
        )
        .bind(&expected_snapshot.plan_id)
        .bind(i64::from(suggestion.planning_cost_minutes))
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        self.load_today_snapshot(expected_snapshot.local_date)
            .await?
            .ok_or(TodaySnapshotError::IntegrityViolation)
    }

    async fn apply_today_replan(
        &self,
        preview: &TodayReplanPreview,
    ) -> Result<TodaySnapshot, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let current = self
            .reconcile_today_snapshot(preview.expected_snapshot.local_date)
            .await?;
        if current != preview.expected_snapshot {
            return Err(TodaySnapshotError::StaleReplanPreview);
        }
        let computed_minutes = preview.entries.iter().try_fold(0_u32, |total, entry| {
            total
                .checked_add(entry.planning_cost_minutes)
                .ok_or(TodaySnapshotError::IntegrityViolation)
        })?;
        if computed_minutes != preview.proposed_planned_minutes
            || preview.proposed_over_budget_minutes
                != computed_minutes.saturating_sub(preview.proposed_budget_minutes)
        {
            return Err(TodaySnapshotError::IntegrityViolation);
        }
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let current_version: Option<(i64, i64, i64, i64)> = sqlx::query_as(
            "SELECT budget_minutes, planned_minutes, over_budget_minutes, review_only_streak \
             FROM today_plans WHERE id = ?1 AND local_date = ?2",
        )
        .bind(&preview.expected_snapshot.plan_id)
        .bind(preview.expected_snapshot.local_date.to_iso_string())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        if current_version
            != Some((
                i64::from(preview.expected_snapshot.budget_minutes),
                i64::from(preview.expected_snapshot.planned_minutes),
                i64::from(preview.expected_snapshot.over_budget_minutes),
                i64::from(preview.expected_snapshot.review_only_streak),
            ))
        {
            return Err(TodaySnapshotError::StaleReplanPreview);
        }
        let entry_versions: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            i64,
            i64,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, CAST(problem_id AS TEXT), review_attempt_id, lane, reason, \
                    planning_cost_minutes, position, entry_origin, entry_status \
             FROM today_plan_entries \
             WHERE today_plan_id = ?1 ORDER BY position",
        )
        .bind(&preview.expected_snapshot.plan_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        if entry_versions.len() != preview.expected_snapshot.entries.len()
            || entry_versions
                .iter()
                .zip(&preview.expected_snapshot.entries)
                .any(
                    |(
                        (
                            id,
                            problem_id,
                            review_attempt_id,
                            lane,
                            reason,
                            cost,
                            position,
                            origin,
                            status,
                        ),
                        expected,
                    )| {
                        id != &expected.entry_id
                            || problem_id != &expected.problem_id
                            || review_attempt_id != &expected.review_attempt_id
                            || lane != today_lane_value(expected.lane)
                            || reason != today_reason_value(expected.reason)
                            || *cost != i64::from(expected.planning_cost_minutes)
                            || *position != i64::from(expected.position)
                            || origin != today_entry_origin_value(expected.origin)
                            || status != today_entry_status_value(expected.status)
                    },
                )
        {
            return Err(TodaySnapshotError::StaleReplanPreview);
        }

        let protected_ids = preview
            .expected_snapshot
            .entries
            .iter()
            .filter(|entry| {
                entry.origin == TodayEntryOrigin::Manual
                    || entry.status != TodayEntryStatus::NotStarted
            })
            .map(|entry| entry.entry_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let proposed_existing_ids = preview
            .entries
            .iter()
            .filter_map(|entry| entry.existing_entry_id.as_deref())
            .collect::<std::collections::HashSet<_>>();
        if proposed_existing_ids != protected_ids
            || preview.entries.iter().any(|entry| {
                entry.existing_entry_id.is_some()
                    && !(entry.origin == TodayEntryOrigin::Manual
                        || entry.status != TodayEntryStatus::NotStarted)
            })
        {
            return Err(TodaySnapshotError::IntegrityViolation);
        }

        sqlx::query(
            "DELETE FROM today_plan_entries WHERE today_plan_id = ?1 \
             AND entry_origin = 'auto' AND entry_status = 'not_started'",
        )
        .bind(&preview.expected_snapshot.plan_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        for (position, entry) in preview.entries.iter().enumerate() {
            let position =
                i64::try_from(position).map_err(|_| TodaySnapshotError::IntegrityViolation)?;
            if let Some(entry_id) = &entry.existing_entry_id {
                let updated = sqlx::query(
                    "UPDATE today_plan_entries SET position = ?2 \
                     WHERE id = ?1 AND today_plan_id = ?3",
                )
                .bind(entry_id)
                .bind(1_000_000_i64 + position)
                .bind(&preview.expected_snapshot.plan_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
                if updated.rows_affected() != 1 {
                    return Err(TodaySnapshotError::StaleReplanPreview);
                }
            }
        }
        for (position, entry) in preview.entries.iter().enumerate() {
            let position =
                i64::try_from(position).map_err(|_| TodaySnapshotError::IntegrityViolation)?;
            if let Some(entry_id) = &entry.existing_entry_id {
                sqlx::query("UPDATE today_plan_entries SET position = ?2 WHERE id = ?1")
                    .bind(entry_id)
                    .bind(position)
                    .execute(&mut *transaction)
                    .await
                    .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
            } else {
                let problem_id = entry
                    .problem_id
                    .parse::<i64>()
                    .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
                sqlx::query(
                    "INSERT INTO today_plan_entries \
                        (id, today_plan_id, problem_id, review_attempt_id, lane, reason, \
                         planning_cost_minutes, position, entry_origin, entry_status) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                )
                .bind(uuid::Uuid::now_v7().to_string())
                .bind(&preview.expected_snapshot.plan_id)
                .bind(problem_id)
                .bind(&entry.review_attempt_id)
                .bind(today_lane_value(entry.lane))
                .bind(today_reason_value(entry.reason))
                .bind(i64::from(entry.planning_cost_minutes))
                .bind(position)
                .bind(today_entry_origin_value(entry.origin))
                .bind(today_entry_status_value(entry.status))
                .execute(&mut *transaction)
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
            }
        }
        let updated_plan = sqlx::query(
            "UPDATE today_plans SET budget_minutes = ?2, planned_minutes = ?3, \
                 over_budget_minutes = ?4, review_only_streak = ?5 WHERE id = ?1",
        )
        .bind(&preview.expected_snapshot.plan_id)
        .bind(i64::from(preview.proposed_budget_minutes))
        .bind(i64::from(preview.proposed_planned_minutes))
        .bind(i64::from(preview.proposed_over_budget_minutes))
        .bind(i64::from(preview.proposed_review_only_streak))
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        if updated_plan.rows_affected() != 1 {
            return Err(TodaySnapshotError::StaleReplanPreview);
        }
        transaction
            .commit()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        self.load_today_snapshot(preview.expected_snapshot.local_date)
            .await?
            .ok_or(TodaySnapshotError::IntegrityViolation)
    }

    async fn create_or_load_today_snapshot(
        &self,
        local_date: acm_os_domain::LocalDate,
        draft: &acm_os_domain::TodayPlanDraft,
    ) -> Result<TodaySnapshot, TodaySnapshotError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(TodaySnapshotError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        let plan_id = uuid::Uuid::now_v7().to_string();
        let inserted = sqlx::query(
            "INSERT INTO today_plans \
                (id, local_date, budget_minutes, planned_minutes, over_budget_minutes, review_only_streak) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(local_date) DO NOTHING",
        )
        .bind(&plan_id)
        .bind(local_date.to_iso_string())
        .bind(i64::from(draft.budget_minutes))
        .bind(i64::from(draft.planned_minutes))
        .bind(i64::from(draft.over_budget_minutes))
        .bind(i64::from(draft.next_review_only_streak))
        .execute(&mut *transaction)
        .await
        .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        if inserted.rows_affected() == 0 {
            transaction
                .rollback()
                .await
                .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
            return self
                .load_today_snapshot(local_date)
                .await?
                .ok_or(TodaySnapshotError::IntegrityViolation);
        }

        for (position, entry) in draft.entries.iter().enumerate() {
            let problem_id = entry
                .problem_id
                .parse::<i64>()
                .map_err(|_| TodaySnapshotError::IntegrityViolation)?;
            sqlx::query(
                "INSERT INTO today_plan_entries \
                    (id, today_plan_id, problem_id, review_attempt_id, lane, reason, \
                     planning_cost_minutes, position, entry_origin, entry_status) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'auto', ?9)",
            )
            .bind(uuid::Uuid::now_v7().to_string())
            .bind(&plan_id)
            .bind(problem_id)
            .bind(&entry.review_attempt_id)
            .bind(today_lane_value(entry.lane))
            .bind(today_reason_value(entry.reason))
            .bind(i64::from(entry.planning_cost_minutes))
            .bind(i64::try_from(position).map_err(|_| TodaySnapshotError::IntegrityViolation)?)
            .bind(
                if entry.lane == acm_os_domain::TodayCandidateLane::CarryIn {
                    "in_progress"
                } else {
                    "not_started"
                },
            )
            .execute(&mut *transaction)
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| TodaySnapshotError::PersistenceUnavailable)?;
        self.load_today_snapshot(local_date)
            .await?
            .ok_or(TodaySnapshotError::IntegrityViolation)
    }
}

fn today_lane_value(value: acm_os_domain::TodayCandidateLane) -> &'static str {
    match value {
        acm_os_domain::TodayCandidateLane::CarryIn => "carry_in",
        acm_os_domain::TodayCandidateLane::Review => "review",
        acm_os_domain::TodayCandidateLane::Study => "study",
    }
}

fn parse_today_lane(value: &str) -> Result<acm_os_domain::TodayCandidateLane, TodaySnapshotError> {
    match value {
        "carry_in" => Ok(acm_os_domain::TodayCandidateLane::CarryIn),
        "review" => Ok(acm_os_domain::TodayCandidateLane::Review),
        "study" => Ok(acm_os_domain::TodayCandidateLane::Study),
        _ => Err(TodaySnapshotError::IntegrityViolation),
    }
}

fn today_reason_value(value: acm_os_domain::TodayCandidateReason) -> &'static str {
    match value {
        acm_os_domain::TodayCandidateReason::ContinueReview => "continue_review",
        acm_os_domain::TodayCandidateReason::ContinueLearning => "continue_learning",
        acm_os_domain::TodayCandidateReason::DueFirstColdStart => "due_first_cold_start",
        acm_os_domain::TodayCandidateReason::DueLongTermReview => "due_long_term_review",
        acm_os_domain::TodayCandidateReason::Relearn => "relearn",
        acm_os_domain::TodayCandidateReason::Upsolve => "upsolve",
    }
}

fn parse_today_reason(
    value: &str,
) -> Result<acm_os_domain::TodayCandidateReason, TodaySnapshotError> {
    match value {
        "continue_review" => Ok(acm_os_domain::TodayCandidateReason::ContinueReview),
        "continue_learning" => Ok(acm_os_domain::TodayCandidateReason::ContinueLearning),
        "due_first_cold_start" => Ok(acm_os_domain::TodayCandidateReason::DueFirstColdStart),
        "due_long_term_review" => Ok(acm_os_domain::TodayCandidateReason::DueLongTermReview),
        "relearn" => Ok(acm_os_domain::TodayCandidateReason::Relearn),
        "upsolve" => Ok(acm_os_domain::TodayCandidateReason::Upsolve),
        _ => Err(TodaySnapshotError::IntegrityViolation),
    }
}

fn parse_today_entry_origin(value: &str) -> Result<TodayEntryOrigin, TodaySnapshotError> {
    match value {
        "auto" => Ok(TodayEntryOrigin::Auto),
        "manual" => Ok(TodayEntryOrigin::Manual),
        _ => Err(TodaySnapshotError::IntegrityViolation),
    }
}

fn today_entry_origin_value(value: TodayEntryOrigin) -> &'static str {
    match value {
        TodayEntryOrigin::Auto => "auto",
        TodayEntryOrigin::Manual => "manual",
    }
}

fn today_entry_status_value(value: TodayEntryStatus) -> &'static str {
    match value {
        TodayEntryStatus::NotStarted => "not_started",
        TodayEntryStatus::InProgress => "in_progress",
        TodayEntryStatus::Completed => "completed",
        TodayEntryStatus::Unavailable => "unavailable",
    }
}

fn parse_today_entry_status(value: &str) -> Result<TodayEntryStatus, TodaySnapshotError> {
    match value {
        "not_started" => Ok(TodayEntryStatus::NotStarted),
        "in_progress" => Ok(TodayEntryStatus::InProgress),
        "completed" => Ok(TodayEntryStatus::Completed),
        "unavailable" => Ok(TodayEntryStatus::Unavailable),
        _ => Err(TodaySnapshotError::IntegrityViolation),
    }
}

fn build_review_help_sources(
    active_vault: &str,
    note_relative_path: &str,
    knowledge_root: &str,
) -> Result<
    Vec<(
        acm_os_domain::ReviewHelpLevel,
        Option<crate::markdown::ReviewHelpContent>,
    )>,
    ReviewAttemptError,
> {
    let vault =
        std::fs::canonicalize(active_vault).map_err(|_| ReviewAttemptError::NoteUnavailable)?;
    let note = std::fs::canonicalize(vault.join(note_relative_path))
        .map_err(|_| ReviewAttemptError::NoteUnavailable)?;
    if !note.starts_with(&vault) {
        return Err(ReviewAttemptError::NoteUnavailable);
    }
    let bytes = std::fs::read(&note).map_err(|_| ReviewAttemptError::NoteUnavailable)?;
    let markdown = std::str::from_utf8(&bytes).map_err(|_| ReviewAttemptError::InvalidMarkdown)?;
    let mut sources = Vec::new();
    for level_number in 1..=5 {
        let level = acm_os_domain::ReviewHelpLevel::from_number(level_number)
            .ok_or(ReviewAttemptError::IntegrityViolation)?;
        let content = if level == acm_os_domain::ReviewHelpLevel::PrerequisiteContent {
            resolve_prerequisite_content(markdown, knowledge_root)
        } else {
            crate::markdown::review_help_content(markdown, level)
        };
        sources.push((level, content));
    }
    Ok(sources)
}

fn resolve_prerequisite_content(
    problem_markdown: &str,
    knowledge_root: &str,
) -> Option<crate::markdown::ReviewHelpContent> {
    let targets = crate::markdown::prerequisite_targets(problem_markdown)?;
    let root = std::fs::canonicalize(knowledge_root).ok()?;
    let files = markdown_files_under(&root).ok()?;
    let mut sections = Vec::new();
    for target in targets {
        let normalized = target.replace('\\', "/");
        let matches = files
            .iter()
            .filter(|path| knowledge_target_matches(&root, path, &normalized))
            .collect::<Vec<_>>();
        let [path] = matches.as_slice() else {
            return None;
        };
        let resolved = std::fs::canonicalize(path).ok()?;
        if !resolved.starts_with(&root) {
            return None;
        }
        let content = std::fs::read_to_string(resolved).ok()?;
        sections.push(format!("# {target}\n\n{content}"));
    }
    Some(crate::markdown::ReviewHelpContent {
        title: "Prerequisite content".to_owned(),
        markdown: sections.join("\n\n---\n\n"),
    })
}

fn markdown_files_under(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                visit(&entry.path(), files)?;
            } else if file_type.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            {
                files.push(entry.path());
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(root, &mut files)?;
    Ok(files)
}

fn knowledge_target_matches(root: &Path, path: &Path, target: &str) -> bool {
    let relative = match path.strip_prefix(root) {
        Ok(relative) => relative,
        Err(_) => return false,
    };
    let without_extension = relative.with_extension("");
    let relative_target = without_extension.to_string_lossy().replace('\\', "/");
    if target.contains('/') {
        relative_target.eq_ignore_ascii_case(target)
    } else {
        without_extension
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(target))
    }
}

type ReviewAttemptRow = (String, i64, String, String, String, i64, i64, String);

#[derive(sqlx::FromRow)]
struct ReviewHistoryRow {
    id: String,
    contest_id: i64,
    problem_index: String,
    attempt_type: String,
    scheduled_due_local_date: String,
    started_early: i64,
    judgement_rule_version: i64,
    started_at_utc: String,
    attempt_status: String,
    judgement: Option<String>,
    completed_at_utc: Option<String>,
    completed_local_date: Option<String>,
    final_ac: Option<i64>,
    first_submission_result: Option<String>,
    final_result: Option<String>,
    total_submissions: Option<i64>,
    idea_independent: Option<i64>,
    implementation_independent: Option<i64>,
    debug_independence: Option<String>,
    external_help: Option<String>,
    evidence_codes_json: Option<String>,
    void_reason: Option<String>,
    voided_at_utc: Option<String>,
}

fn submission_fact_value(fact: &SubmissionFact) -> String {
    match fact.result {
        acm_os_domain::SubmissionResult::Accepted => "accepted".to_owned(),
        acm_os_domain::SubmissionResult::WrongAnswer => "wrong_answer".to_owned(),
        acm_os_domain::SubmissionResult::TimeLimitExceeded => "time_limit_exceeded".to_owned(),
        acm_os_domain::SubmissionResult::MemoryLimitExceeded => "memory_limit_exceeded".to_owned(),
        acm_os_domain::SubmissionResult::RuntimeError => "runtime_error".to_owned(),
        acm_os_domain::SubmissionResult::CompilationError => "compilation_error".to_owned(),
        acm_os_domain::SubmissionResult::Other => {
            format!("other:{}", fact.other_text.as_deref().unwrap_or_default())
        }
    }
}

fn parse_submission_fact(value: &str) -> Result<SubmissionFact, ReviewAttemptError> {
    let (result, other_text) = match value {
        "accepted" => (acm_os_domain::SubmissionResult::Accepted, None),
        "wrong_answer" => (acm_os_domain::SubmissionResult::WrongAnswer, None),
        "time_limit_exceeded" => (acm_os_domain::SubmissionResult::TimeLimitExceeded, None),
        "memory_limit_exceeded" => (acm_os_domain::SubmissionResult::MemoryLimitExceeded, None),
        "runtime_error" => (acm_os_domain::SubmissionResult::RuntimeError, None),
        "compilation_error" => (acm_os_domain::SubmissionResult::CompilationError, None),
        value if value.starts_with("other:") && value.len() > 6 => (
            acm_os_domain::SubmissionResult::Other,
            Some(value[6..].to_owned()),
        ),
        _ => return Err(ReviewAttemptError::IntegrityViolation),
    };
    Ok(SubmissionFact { result, other_text })
}

fn debug_independence_value(value: acm_os_domain::DebugIndependence) -> &'static str {
    match value {
        acm_os_domain::DebugIndependence::NotNeeded => "not_needed",
        acm_os_domain::DebugIndependence::Independent => "independent",
        acm_os_domain::DebugIndependence::UsedSolvingHelp => "used_solving_help",
    }
}

fn parse_debug_independence(
    value: &str,
) -> Result<acm_os_domain::DebugIndependence, ReviewAttemptError> {
    match value {
        "not_needed" => Ok(acm_os_domain::DebugIndependence::NotNeeded),
        "independent" => Ok(acm_os_domain::DebugIndependence::Independent),
        "used_solving_help" => Ok(acm_os_domain::DebugIndependence::UsedSolvingHelp),
        _ => Err(ReviewAttemptError::IntegrityViolation),
    }
}

fn external_help_value(value: acm_os_domain::ExternalHelpLevel) -> &'static str {
    match value {
        acm_os_domain::ExternalHelpLevel::None => "none",
        acm_os_domain::ExternalHelpLevel::SolvingHint => "solving_hint",
        acm_os_domain::ExternalHelpLevel::FullSolution => "full_solution",
    }
}

fn parse_external_help(
    value: &str,
) -> Result<acm_os_domain::ExternalHelpLevel, ReviewAttemptError> {
    match value {
        "none" => Ok(acm_os_domain::ExternalHelpLevel::None),
        "solving_hint" => Ok(acm_os_domain::ExternalHelpLevel::SolvingHint),
        "full_solution" => Ok(acm_os_domain::ExternalHelpLevel::FullSolution),
        _ => Err(ReviewAttemptError::IntegrityViolation),
    }
}

fn judgement_value(value: acm_os_domain::ReviewJudgement) -> &'static str {
    match value {
        acm_os_domain::ReviewJudgement::Mastered => "mastered",
        acm_os_domain::ReviewJudgement::Partial => "partial",
        acm_os_domain::ReviewJudgement::Fail => "fail",
    }
}

fn parse_judgement(value: &str) -> Result<acm_os_domain::ReviewJudgement, ReviewAttemptError> {
    match value {
        "mastered" => Ok(acm_os_domain::ReviewJudgement::Mastered),
        "partial" => Ok(acm_os_domain::ReviewJudgement::Partial),
        "fail" => Ok(acm_os_domain::ReviewJudgement::Fail),
        _ => Err(ReviewAttemptError::IntegrityViolation),
    }
}

fn failure_reason_value(reason: &ReviewFailureReason) -> (&'static str, Option<&str>) {
    match reason {
        ReviewFailureReason::NoIdea => ("no_idea", None),
        ReviewFailureReason::KeyPropertyBlocked => ("key_property_blocked", None),
        ReviewFailureReason::DerivationBlocked => ("derivation_blocked", None),
        ReviewFailureReason::CannotImplement => ("cannot_implement", None),
        ReviewFailureReason::ImplementationError => ("implementation_error", None),
        ReviewFailureReason::BoundaryError => ("boundary_error", None),
        ReviewFailureReason::ComplexityError => ("complexity_error", None),
        ReviewFailureReason::Other(text) => ("other", Some(text.as_str())),
    }
}

fn parse_failure_reason(
    code: &str,
    other: Option<String>,
) -> Result<ReviewFailureReason, ReviewAttemptError> {
    match (code, other) {
        ("no_idea", None) => Ok(ReviewFailureReason::NoIdea),
        ("key_property_blocked", None) => Ok(ReviewFailureReason::KeyPropertyBlocked),
        ("derivation_blocked", None) => Ok(ReviewFailureReason::DerivationBlocked),
        ("cannot_implement", None) => Ok(ReviewFailureReason::CannotImplement),
        ("implementation_error", None) => Ok(ReviewFailureReason::ImplementationError),
        ("boundary_error", None) => Ok(ReviewFailureReason::BoundaryError),
        ("complexity_error", None) => Ok(ReviewFailureReason::ComplexityError),
        ("other", Some(text)) if !text.trim().is_empty() => Ok(ReviewFailureReason::Other(text)),
        _ => Err(ReviewAttemptError::IntegrityViolation),
    }
}

async fn load_review_history_item_from_pool(
    pool: &SqlitePool,
    attempt_id: &str,
) -> Result<ReviewHistoryItem, ReviewAttemptError> {
    let row: Option<ReviewHistoryRow> = sqlx::query_as(
        "SELECT ra.id, p.external_contest_key AS contest_id, \
                p.external_problem_key AS problem_index, ra.attempt_type, \
                ra.scheduled_due_local_date, ra.started_early, ra.judgement_rule_version, \
                ra.started_at_utc, ra.attempt_status, ra.judgement, ra.completed_at_utc, \
                ra.completed_local_date, ra.final_ac, ra.first_submission_result, \
                ra.final_result, ra.total_submissions, ra.idea_independent, \
                ra.implementation_independent, ra.debug_independence, ra.external_help, \
                ra.evidence_codes_json, rve.reason AS void_reason, rve.voided_at_utc \
         FROM review_attempts ra \
         JOIN problems p ON p.id = ra.problem_id \
         LEFT JOIN review_void_events rve ON rve.review_attempt_id = ra.id \
         WHERE ra.id = ?1",
    )
    .bind(attempt_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
    let row = row.ok_or(ReviewAttemptError::AttemptNotFound)?;
    let attempt = review_attempt_from_row((
        row.id,
        row.contest_id,
        row.problem_index,
        row.attempt_type,
        row.scheduled_due_local_date,
        row.started_early,
        row.judgement_rule_version,
        row.started_at_utc,
    ))?;
    let reason_rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT reason_code, other_text FROM review_failure_reasons \
         WHERE review_attempt_id = ?1 ORDER BY reason_code",
    )
    .bind(attempt_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
    let failure_reasons = reason_rows
        .into_iter()
        .map(|(code, other)| parse_failure_reason(&code, other))
        .collect::<Result<Vec<_>, _>>()?;
    let help_numbers: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT help_level FROM review_help_usage_events \
         WHERE review_attempt_id = ?1 ORDER BY help_level",
    )
    .bind(attempt_id)
    .fetch_all(pool)
    .await
    .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
    let help_levels = help_numbers
        .into_iter()
        .map(|number| {
            u8::try_from(number)
                .ok()
                .and_then(acm_os_domain::ReviewHelpLevel::from_number)
                .ok_or(ReviewAttemptError::IntegrityViolation)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let status = match row.attempt_status.as_str() {
        "in_progress" => ReviewAttemptStatus::InProgress,
        "completed" => ReviewAttemptStatus::Completed,
        "void" => ReviewAttemptStatus::Void,
        _ => return Err(ReviewAttemptError::IntegrityViolation),
    };
    let judgement = row.judgement.as_deref().map(parse_judgement).transpose()?;
    let evidence_codes: Vec<String> = match row.evidence_codes_json {
        Some(value) => {
            serde_json::from_str(&value).map_err(|_| ReviewAttemptError::IntegrityViolation)?
        }
        None => Vec::new(),
    };
    let completion_input = match status {
        ReviewAttemptStatus::Completed => Some(ReviewCompletionInput {
            final_ac: match row.final_ac {
                Some(0) => false,
                Some(1) => true,
                _ => return Err(ReviewAttemptError::IntegrityViolation),
            },
            first_submission: parse_submission_fact(
                row.first_submission_result
                    .as_deref()
                    .ok_or(ReviewAttemptError::IntegrityViolation)?,
            )?,
            final_submission: parse_submission_fact(
                row.final_result
                    .as_deref()
                    .ok_or(ReviewAttemptError::IntegrityViolation)?,
            )?,
            total_submissions: u32::try_from(
                row.total_submissions
                    .ok_or(ReviewAttemptError::IntegrityViolation)?,
            )
            .map_err(|_| ReviewAttemptError::IntegrityViolation)?,
            idea_independent: match row.idea_independent {
                Some(0) => false,
                Some(1) => true,
                _ => return Err(ReviewAttemptError::IntegrityViolation),
            },
            implementation_independent: match row.implementation_independent {
                Some(0) => false,
                Some(1) => true,
                _ => return Err(ReviewAttemptError::IntegrityViolation),
            },
            debug_independence: parse_debug_independence(
                row.debug_independence
                    .as_deref()
                    .ok_or(ReviewAttemptError::IntegrityViolation)?,
            )?,
            external_help: parse_external_help(
                row.external_help
                    .as_deref()
                    .ok_or(ReviewAttemptError::IntegrityViolation)?,
            )?,
            failure_reasons: failure_reasons.clone(),
        }),
        ReviewAttemptStatus::InProgress | ReviewAttemptStatus::Void => None,
    };
    if (status == ReviewAttemptStatus::Completed)
        != (judgement.is_some()
            && row.completed_local_date.is_some()
            && row.completed_at_utc.is_some()
            && completion_input.is_some())
        || (status == ReviewAttemptStatus::Void)
            != (row.void_reason.is_some() && row.voided_at_utc.is_some())
    {
        return Err(ReviewAttemptError::IntegrityViolation);
    }
    Ok(ReviewHistoryItem {
        attempt,
        status,
        judgement,
        completion_input,
        evidence_codes,
        failure_reasons,
        help_levels,
        completed_at_utc: row.completed_at_utc,
        completed_local_date: row
            .completed_local_date
            .as_deref()
            .map(acm_os_domain::LocalDate::parse_iso)
            .transpose()
            .map_err(|_| ReviewAttemptError::IntegrityViolation)?,
        void_reason: row.void_reason,
        voided_at_utc: row.voided_at_utc,
    })
}

fn review_attempt_from_row(row: ReviewAttemptRow) -> Result<ReviewAttempt, ReviewAttemptError> {
    let (attempt_id, contest_id, index, attempt_type, due, started_early, rule_version, started_at) =
        row;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(
        u64::try_from(contest_id).map_err(|_| ReviewAttemptError::IntegrityViolation)?,
    )
    .map_err(|_| ReviewAttemptError::IntegrityViolation)?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, index)
        .map_err(|_| ReviewAttemptError::IntegrityViolation)?;
    let attempt_type = match attempt_type.as_str() {
        "first_cold_start" => acm_os_domain::ReviewAttemptType::FirstColdStart,
        "long_term_review" => acm_os_domain::ReviewAttemptType::LongTermReview,
        "early_check" => acm_os_domain::ReviewAttemptType::EarlyCheck,
        _ => return Err(ReviewAttemptError::IntegrityViolation),
    };
    let started_early = match started_early {
        0 => false,
        1 => true,
        _ => return Err(ReviewAttemptError::IntegrityViolation),
    };
    if (attempt_type == acm_os_domain::ReviewAttemptType::EarlyCheck) != started_early {
        return Err(ReviewAttemptError::IntegrityViolation);
    }
    Ok(ReviewAttempt {
        attempt_id,
        problem,
        attempt_type,
        scheduled_due_local_date: acm_os_domain::LocalDate::parse_iso(&due)
            .map_err(|_| ReviewAttemptError::IntegrityViolation)?,
        started_early,
        judgement_rule_version: u32::try_from(rule_version)
            .map_err(|_| ReviewAttemptError::IntegrityViolation)?,
        started_at_utc: started_at,
    })
}

fn review_attempt_type_value(value: acm_os_domain::ReviewAttemptType) -> &'static str {
    match value {
        acm_os_domain::ReviewAttemptType::FirstColdStart => "first_cold_start",
        acm_os_domain::ReviewAttemptType::LongTermReview => "long_term_review",
        acm_os_domain::ReviewAttemptType::EarlyCheck => "early_check",
    }
}

impl ReviewAttemptPort for DatabaseRuntime {
    async fn load_in_progress_review_attempt(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<Option<ReviewAttempt>, ReviewAttemptError> {
        let row: Option<ReviewAttemptRow> = sqlx::query_as(
            "SELECT ra.id, p.external_contest_key, p.external_problem_key, ra.attempt_type, \
                    ra.scheduled_due_local_date, ra.started_early, ra.judgement_rule_version, \
                    ra.started_at_utc \
             FROM review_attempts ra \
             JOIN problems p ON p.id = ra.problem_id \
             WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2 AND ra.attempt_status = 'in_progress'",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(
            self._pool
                .as_ref()
                .ok_or(ReviewAttemptError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        row.map(review_attempt_from_row).transpose()
    }

    async fn create_or_resume_review_attempt(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        eligibility: acm_os_domain::ReviewEligibilityDecision,
    ) -> Result<ReviewAttempt, ReviewAttemptError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(ReviewAttemptError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let existing: Option<ReviewAttemptRow> = sqlx::query_as(
            "SELECT ra.id, p.external_contest_key, p.external_problem_key, ra.attempt_type, \
                    ra.scheduled_due_local_date, ra.started_early, ra.judgement_rule_version, \
                    ra.started_at_utc \
             FROM review_attempts ra JOIN problems p ON p.id = ra.problem_id \
             WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2 AND ra.attempt_status = 'in_progress'",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        if let Some(row) = existing {
            transaction
                .commit()
                .await
                .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
            return review_attempt_from_row(row);
        }

        let current: Option<(i64, String, String, String, String, Option<i64>)> = sqlx::query_as(
            "SELECT p.id, p.identity_type, pls.learning_status, rc.id, rc.next_due_local_date, \
                    ss.problem_id \
             FROM problems p \
             JOIN problem_learning_states pls ON pls.problem_id = p.id \
             JOIN review_cycles rc ON rc.problem_id = p.id AND rc.cycle_status = 'active' \
             LEFT JOIN problem_statement_snapshots ss ON ss.problem_id = p.id \
             WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let (problem_id, identity_type, status, review_cycle_id, due, statement_problem_id) =
            current.ok_or(ReviewAttemptError::ProblemNotFound)?;
        if identity_type != "personal" {
            return Err(ReviewAttemptError::NotPersonal);
        }
        if statement_problem_id != Some(problem_id) {
            return Err(ReviewAttemptError::StatementMissing);
        }
        let status =
            parse_learning_status(&status).map_err(|_| ReviewAttemptError::IntegrityViolation)?;
        if due != eligibility.scheduled_due_local_date.to_iso_string() {
            return Err(ReviewAttemptError::IntegrityViolation);
        }
        let expected_scheduled_type = match status {
            acm_os_domain::LearningStatus::WaitingColdStart => {
                acm_os_domain::ReviewAttemptType::FirstColdStart
            }
            acm_os_domain::LearningStatus::LongTermReview => {
                acm_os_domain::ReviewAttemptType::LongTermReview
            }
            _ => return Err(ReviewAttemptError::NotEligible),
        };
        if (eligibility.started_early
            && eligibility.attempt_type != acm_os_domain::ReviewAttemptType::EarlyCheck)
            || (!eligibility.started_early && eligibility.attempt_type != expected_scheduled_type)
        {
            return Err(ReviewAttemptError::IntegrityViolation);
        }

        let attempt_id = uuid::Uuid::now_v7().to_string();
        let started_at: String = sqlx::query_scalar(
            "INSERT INTO review_attempts \
                (id, problem_id, review_cycle_id, attempt_type, scheduled_due_local_date, \
                 started_early, judgement_rule_version) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
             RETURNING started_at_utc",
        )
        .bind(&attempt_id)
        .bind(problem_id)
        .bind(review_cycle_id)
        .bind(review_attempt_type_value(eligibility.attempt_type))
        .bind(eligibility.scheduled_due_local_date.to_iso_string())
        .bind(i64::from(eligibility.started_early))
        .bind(i64::from(
            acm_os_domain::ReviewEligibilityEngine::JUDGEMENT_RULE_VERSION,
        ))
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
                ReviewAttemptError::IntegrityViolation
            } else {
                ReviewAttemptError::PersistenceUnavailable
            }
        })?;
        transaction
            .commit()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        Ok(ReviewAttempt {
            attempt_id,
            problem: problem.clone(),
            attempt_type: eligibility.attempt_type,
            scheduled_due_local_date: eligibility.scheduled_due_local_date,
            started_early: eligibility.started_early,
            judgement_rule_version: acm_os_domain::ReviewEligibilityEngine::JUDGEMENT_RULE_VERSION,
            started_at_utc: started_at,
        })
    }

    async fn load_review_focus(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewFocusView, ReviewAttemptError> {
        let row: Option<(
            String,
            i64,
            String,
            String,
            String,
            i64,
            i64,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT ra.id, p.external_contest_key, p.external_problem_key, ra.attempt_type, \
                    ra.scheduled_due_local_date, ra.started_early, ra.judgement_rule_version, \
                    ra.started_at_utc, p.title, p.source_url, ss.sanitized_html \
             FROM review_attempts ra \
             JOIN problems p ON p.id = ra.problem_id \
             JOIN problem_statement_snapshots ss ON ss.problem_id = p.id \
             WHERE ra.id = ?1 AND ra.attempt_status = 'in_progress'",
        )
        .bind(attempt_id)
        .fetch_optional(
            self._pool
                .as_ref()
                .ok_or(ReviewAttemptError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let (
            id,
            contest_id,
            index,
            attempt_type,
            due,
            started_early,
            rule_version,
            started_at,
            title,
            source_url,
            statement_sanitized_html,
        ) = row.ok_or(ReviewAttemptError::AttemptNotFound)?;
        let attempt = review_attempt_from_row((
            id,
            contest_id,
            index,
            attempt_type,
            due,
            started_early,
            rule_version,
            started_at,
        ))?;
        let statement_assets = self
            .statement_assets(&attempt.problem)
            .await
            .map_err(|error| match error {
                ContestReadError::NotFound => ReviewAttemptError::ProblemNotFound,
                ContestReadError::Unavailable => ReviewAttemptError::PersistenceUnavailable,
            })?;
        Ok(ReviewFocusView {
            attempt,
            title,
            source_url,
            statement_sanitized_html,
            statement_assets,
        })
    }

    async fn load_review_help_drawer(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewHelpDrawerView, ReviewAttemptError> {
        let sources = self.review_help_sources(attempt_id).await?;
        let revealed: Vec<(i64, String)> = sqlx::query_as(
            "SELECT help_level, MAX(revealed_at_utc) \
             FROM review_help_usage_events WHERE review_attempt_id = ?1 GROUP BY help_level",
        )
        .bind(attempt_id)
        .fetch_all(
            self._pool
                .as_ref()
                .ok_or(ReviewAttemptError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let items = sources
            .into_iter()
            .map(|(level, content)| ReviewHelpItem {
                level,
                available: content.is_some(),
                revealed_at_utc: revealed
                    .iter()
                    .find(|(number, _)| *number == i64::from(level.number()))
                    .map(|(_, revealed_at)| revealed_at.clone()),
            })
            .collect();
        Ok(ReviewHelpDrawerView {
            attempt_id: attempt_id.to_owned(),
            items,
        })
    }

    async fn reveal_review_help(
        &self,
        attempt_id: &str,
        level: acm_os_domain::ReviewHelpLevel,
        impact_acknowledged: bool,
    ) -> Result<RevealedReviewHelp, ReviewAttemptError> {
        // Authority order: fresh disk read and exact section resolution happen before any
        // durable evidence, while content is returned only after the event transaction commits.
        let sources = self.review_help_sources(attempt_id).await?;
        let content = sources
            .into_iter()
            .find(|(candidate, _)| *candidate == level)
            .and_then(|(_, content)| content)
            .ok_or(ReviewAttemptError::HelpContentUnavailable)?;
        let source_digest = sha256_hex(content.markdown.as_bytes());
        let pool = self
            ._pool
            .as_ref()
            .ok_or(ReviewAttemptError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_attempts WHERE id = ?1 AND attempt_status = 'in_progress'",
        )
        .bind(attempt_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        if active != 1 {
            return Err(ReviewAttemptError::AttemptNotFound);
        }
        let previously_revealed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_help_usage_events \
             WHERE review_attempt_id = ?1 AND help_level = ?2",
        )
        .bind(attempt_id)
        .bind(i64::from(level.number()))
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        if previously_revealed == 0 && !impact_acknowledged {
            return Err(ReviewAttemptError::HelpConfirmationRequired);
        }
        let event_id = uuid::Uuid::now_v7().to_string();
        let revealed_at_utc: String = sqlx::query_scalar(
            "INSERT INTO review_help_usage_events \
                (id, review_attempt_id, help_level, source_digest) \
             VALUES (?1, ?2, ?3, ?4) RETURNING revealed_at_utc",
        )
        .bind(&event_id)
        .bind(attempt_id)
        .bind(i64::from(level.number()))
        .bind(&source_digest)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        Ok(RevealedReviewHelp {
            event_id,
            attempt_id: attempt_id.to_owned(),
            level,
            title: content.title,
            content_markdown: content.markdown,
            source_digest,
            revealed_at_utc,
        })
    }

    async fn load_review_completion_context(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewCompletionContext, ReviewAttemptError> {
        let row: Option<(String, i64, String, String, String, i64, i64, String, String, i64)> =
            sqlx::query_as(
                "SELECT ra.id, p.external_contest_key, p.external_problem_key, ra.attempt_type, \
                        ra.scheduled_due_local_date, ra.started_early, ra.judgement_rule_version, \
                        ra.started_at_utc, pls.learning_status, rc.stage \
                 FROM review_attempts ra \
                 JOIN problems p ON p.id = ra.problem_id \
                 JOIN problem_learning_states pls ON pls.problem_id = p.id \
                 JOIN review_cycles rc ON rc.id = ra.review_cycle_id AND rc.cycle_status = 'active' \
                 WHERE ra.id = ?1 AND ra.attempt_status = 'in_progress'",
            )
            .bind(attempt_id)
            .fetch_optional(
                self._pool
                    .as_ref()
                    .ok_or(ReviewAttemptError::PersistenceUnavailable)?,
            )
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let (id, contest_id, index, attempt_type, due, early, rule, started, status, stage) =
            row.ok_or(ReviewAttemptError::AttemptNotFound)?;
        let highest_help: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(help_level) FROM review_help_usage_events WHERE review_attempt_id = ?1",
        )
        .bind(attempt_id)
        .fetch_one(
            self._pool
                .as_ref()
                .ok_or(ReviewAttemptError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        Ok(ReviewCompletionContext {
            attempt: review_attempt_from_row((
                id,
                contest_id,
                index,
                attempt_type,
                due,
                early,
                rule,
                started,
            ))?,
            learning_status: parse_learning_status(&status)
                .map_err(|_| ReviewAttemptError::IntegrityViolation)?,
            current_stage: u32::try_from(stage)
                .map_err(|_| ReviewAttemptError::IntegrityViolation)?,
            highest_help_level: highest_help
                .map(|number| {
                    u8::try_from(number)
                        .ok()
                        .and_then(acm_os_domain::ReviewHelpLevel::from_number)
                        .ok_or(ReviewAttemptError::IntegrityViolation)
                })
                .transpose()?,
        })
    }

    async fn commit_review_completion(
        &self,
        context: &ReviewCompletionContext,
        input: &ReviewCompletionInput,
        judgement: &acm_os_domain::ReviewJudgementDecision,
        scheduling: acm_os_domain::ReviewCompletionDecision,
        completed_on: acm_os_domain::LocalDate,
    ) -> Result<CompletedReviewAttempt, ReviewAttemptError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(ReviewAttemptError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let current: Option<(String, String, i64, String)> = sqlx::query_as(
            "SELECT ra.attempt_status, pls.learning_status, rc.stage, rc.cycle_status \
             FROM review_attempts ra \
             JOIN problem_learning_states pls ON pls.problem_id = ra.problem_id \
             JOIN review_cycles rc ON rc.id = ra.review_cycle_id \
             WHERE ra.id = ?1",
        )
        .bind(&context.attempt.attempt_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let (attempt_status, learning_status, stage, cycle_status) =
            current.ok_or(ReviewAttemptError::AttemptNotFound)?;
        if attempt_status != "in_progress" {
            return Err(ReviewAttemptError::AttemptAlreadyFinished);
        }
        if cycle_status != "active"
            || parse_learning_status(&learning_status)
                .map_err(|_| ReviewAttemptError::IntegrityViolation)?
                != context.learning_status
            || u32::try_from(stage).map_err(|_| ReviewAttemptError::IntegrityViolation)?
                != context.current_stage
        {
            return Err(ReviewAttemptError::IntegrityViolation);
        }
        let highest_help: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(help_level) FROM review_help_usage_events WHERE review_attempt_id = ?1",
        )
        .bind(&context.attempt.attempt_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let highest_help = highest_help
            .map(|number| {
                u8::try_from(number)
                    .ok()
                    .and_then(acm_os_domain::ReviewHelpLevel::from_number)
                    .ok_or(ReviewAttemptError::IntegrityViolation)
            })
            .transpose()?;
        if highest_help != context.highest_help_level {
            return Err(ReviewAttemptError::IntegrityViolation);
        }
        let verified_judgement =
            acm_os_domain::ReviewJudgementEngine::judge(&input.domain_facts(), highest_help)
                .map_err(|_| ReviewAttemptError::InvalidCompletionFacts)?;
        if &verified_judgement != judgement {
            return Err(ReviewAttemptError::IntegrityViolation);
        }
        let verified_scheduling = acm_os_domain::ReviewSchedulingEngine::complete_review(
            context.learning_status,
            context.attempt.attempt_type,
            judgement.judgement,
            context.current_stage,
            completed_on,
        )
        .map_err(|_| ReviewAttemptError::IntegrityViolation)?;
        if verified_scheduling != scheduling {
            return Err(ReviewAttemptError::IntegrityViolation);
        }
        if judgement.judgement == acm_os_domain::ReviewJudgement::Mastered {
            if !input.failure_reasons.is_empty() {
                return Err(ReviewAttemptError::InvalidCompletionFacts);
            }
        } else if input.failure_reasons.is_empty() {
            return Err(ReviewAttemptError::FailureReasonRequired);
        }
        for reason in &input.failure_reasons {
            let (code, other) = failure_reason_value(reason);
            sqlx::query(
                "INSERT INTO review_failure_reasons (review_attempt_id, reason_code, other_text) \
                 VALUES (?1, ?2, ?3)",
            )
            .bind(&context.attempt.attempt_id)
            .bind(code)
            .bind(other)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ReviewAttemptError::InvalidCompletionFacts)?;
        }
        let evidence_codes = judgement
            .evidence_codes
            .iter()
            .map(|code| (*code).to_owned())
            .collect::<Vec<_>>();
        let evidence_json = serde_json::to_string(&evidence_codes)
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let completed_at_utc: String = sqlx::query_scalar(
            "UPDATE review_attempts SET attempt_status = 'completed', judgement = ?2, \
                    completed_local_date = ?3, final_ac = ?4, first_submission_result = ?5, \
                    final_result = ?6, total_submissions = ?7, idea_independent = ?8, \
                    implementation_independent = ?9, debug_independence = ?10, \
                    external_help = ?11, evidence_codes_json = ?12, \
                    completed_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE id = ?1 AND attempt_status = 'in_progress' RETURNING completed_at_utc",
        )
        .bind(&context.attempt.attempt_id)
        .bind(judgement_value(judgement.judgement))
        .bind(completed_on.to_iso_string())
        .bind(i64::from(input.final_ac))
        .bind(submission_fact_value(&input.first_submission))
        .bind(submission_fact_value(&input.final_submission))
        .bind(i64::from(input.total_submissions))
        .bind(i64::from(input.idea_independent))
        .bind(i64::from(input.implementation_independent))
        .bind(debug_independence_value(input.debug_independence))
        .bind(external_help_value(input.external_help))
        .bind(evidence_json)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        match scheduling.cycle {
            acm_os_domain::ReviewCycleCompletion::Keep => {}
            acm_os_domain::ReviewCycleCompletion::Advance {
                next_stage,
                next_due,
            } => {
                let updated = sqlx::query(
                    "UPDATE review_cycles SET stage = ?2, next_due_local_date = ?3 \
                     WHERE id = (SELECT review_cycle_id FROM review_attempts WHERE id = ?1) \
                       AND cycle_status = 'active'",
                )
                .bind(&context.attempt.attempt_id)
                .bind(i64::from(next_stage))
                .bind(next_due.to_iso_string())
                .execute(&mut *transaction)
                .await
                .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
                if updated.rows_affected() != 1 {
                    return Err(ReviewAttemptError::IntegrityViolation);
                }
            }
            acm_os_domain::ReviewCycleCompletion::Suspend => {
                let updated = sqlx::query(
                    "UPDATE review_cycles SET cycle_status = 'suspended', next_due_local_date = NULL, \
                            ended_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
                     WHERE id = (SELECT review_cycle_id FROM review_attempts WHERE id = ?1) \
                       AND cycle_status = 'active'",
                )
                .bind(&context.attempt.attempt_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
                if updated.rows_affected() != 1 {
                    return Err(ReviewAttemptError::IntegrityViolation);
                }
            }
        }
        if scheduling.next_learning_status != context.learning_status {
            let updated = sqlx::query(
                "UPDATE problem_learning_states SET learning_status = ?2, \
                        learning_status_since_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
                 WHERE problem_id = (SELECT problem_id FROM review_attempts WHERE id = ?1)",
            )
            .bind(&context.attempt.attempt_id)
            .bind(learning_status_value(scheduling.next_learning_status))
            .execute(&mut *transaction)
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
            if updated.rows_affected() != 1 {
                return Err(ReviewAttemptError::IntegrityViolation);
            }
        }
        transaction
            .commit()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let lifecycle = self
            .load_problem_lifecycle(&context.attempt.problem)
            .await
            .map_err(|error| match error {
                ProblemLifecycleError::ProblemNotFound => ReviewAttemptError::ProblemNotFound,
                ProblemLifecycleError::PersistenceUnavailable => {
                    ReviewAttemptError::PersistenceUnavailable
                }
                _ => ReviewAttemptError::IntegrityViolation,
            })?;
        Ok(CompletedReviewAttempt {
            attempt: context.attempt.clone(),
            judgement: judgement.judgement,
            evidence_codes,
            failure_reasons: input.failure_reasons.clone(),
            completed_at_utc,
            completed_local_date: completed_on,
            lifecycle,
        })
    }

    async fn void_review_attempt(
        &self,
        attempt_id: &str,
        reason: &str,
    ) -> Result<ReviewHistoryItem, ReviewAttemptError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(ReviewAttemptError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let changed = sqlx::query(
            "UPDATE review_attempts SET attempt_status = 'void', \
                    completed_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE id = ?1 AND attempt_status = 'in_progress'",
        )
        .bind(attempt_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        if changed.rows_affected() != 1 {
            let exists: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM review_attempts WHERE id = ?1")
                    .bind(attempt_id)
                    .fetch_one(&mut *transaction)
                    .await
                    .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
            return Err(if exists == 0 {
                ReviewAttemptError::AttemptNotFound
            } else {
                ReviewAttemptError::AttemptAlreadyFinished
            });
        }
        sqlx::query(
            "INSERT INTO review_void_events (id, review_attempt_id, reason) VALUES (?1, ?2, ?3)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(attempt_id)
        .bind(reason)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        load_review_history_item_from_pool(pool, attempt_id).await
    }

    async fn load_review_attempt_history_item(
        &self,
        attempt_id: &str,
    ) -> Result<ReviewHistoryItem, ReviewAttemptError> {
        load_review_history_item_from_pool(
            self._pool
                .as_ref()
                .ok_or(ReviewAttemptError::PersistenceUnavailable)?,
            attempt_id,
        )
        .await
    }

    async fn load_review_history(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<ReviewHistoryView, ReviewAttemptError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(ReviewAttemptError::PersistenceUnavailable)?;
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM problems WHERE platform = 'codeforces' \
             AND external_contest_key = ?1 AND external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_one(pool)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        if exists != 1 {
            return Err(ReviewAttemptError::ProblemNotFound);
        }
        let ids: Vec<String> = sqlx::query_scalar(
            "SELECT ra.id FROM review_attempts ra JOIN problems p ON p.id = ra.problem_id \
             WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2 ORDER BY ra.started_at_utc DESC, ra.id DESC",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_all(pool)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        let mut attempts = Vec::with_capacity(ids.len());
        for id in ids {
            attempts.push(load_review_history_item_from_pool(pool, &id).await?);
        }
        let historical_best_review = attempts.iter().filter_map(|item| item.judgement).max();
        let mastery = load_problem_mastery_projection(pool, problem).await?;
        Ok(ReviewHistoryView {
            problem: problem.clone(),
            historical_best_review,
            mastery,
            attempts,
        })
    }

    async fn update_problem_mastery_evidence(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        evidence: acm_os_domain::ProblemMasteryEvidence,
        confirmed_on: acm_os_domain::LocalDate,
    ) -> Result<ProblemMasteryProjection, ReviewAttemptError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(ReviewAttemptError::PersistenceUnavailable)?;
        let thorough = evidence.is_thoroughly_digested();
        let updated = sqlx::query(
            "INSERT INTO problem_mastery_evidence (\
                 problem_id, recalls_problem, multiple_solutions_clear, knowledge_understood, \
                 implementation_fluent, can_adapt_or_create, transfer_solved_independently, \
                 historical_thoroughly_digested, first_thoroughly_digested_local_date, updated_at_utc\
             ) SELECT p.id, ?3, ?4, ?5, ?6, ?7, ?8, ?9, \
                      CASE WHEN ?9 = 1 THEN ?10 ELSE NULL END, \
                      strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
               FROM problems p WHERE p.platform = 'codeforces' \
                AND p.external_contest_key = ?1 AND p.external_problem_key = ?2 \
             ON CONFLICT(problem_id) DO UPDATE SET \
                 recalls_problem = excluded.recalls_problem, \
                 multiple_solutions_clear = excluded.multiple_solutions_clear, \
                 knowledge_understood = excluded.knowledge_understood, \
                 implementation_fluent = excluded.implementation_fluent, \
                 can_adapt_or_create = excluded.can_adapt_or_create, \
                 transfer_solved_independently = excluded.transfer_solved_independently, \
                 historical_thoroughly_digested = MAX(\
                     problem_mastery_evidence.historical_thoroughly_digested, \
                     excluded.historical_thoroughly_digested\
                 ), \
                 first_thoroughly_digested_local_date = COALESCE(\
                     problem_mastery_evidence.first_thoroughly_digested_local_date, \
                     excluded.first_thoroughly_digested_local_date\
                 ), \
                 updated_at_utc = excluded.updated_at_utc",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .bind(evidence.recalls_problem)
        .bind(evidence.multiple_solutions_clear)
        .bind(evidence.knowledge_understood)
        .bind(evidence.implementation_fluent)
        .bind(evidence.can_adapt_or_create)
        .bind(evidence.transfer_solved_independently)
        .bind(thorough)
        .bind(confirmed_on.to_iso_string())
        .execute(pool)
        .await
        .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(ReviewAttemptError::ProblemNotFound);
        }
        load_problem_mastery_projection(pool, problem).await
    }
}

async fn load_problem_mastery_projection(
    pool: &SqlitePool,
    problem: &acm_os_domain::CodeforcesProblemIdentity,
) -> Result<ProblemMasteryProjection, ReviewAttemptError> {
    let row: Option<(bool, bool, bool, bool, bool, bool, bool, Option<String>)> = sqlx::query_as(
        "SELECT pme.recalls_problem, pme.multiple_solutions_clear, pme.knowledge_understood, \
                pme.implementation_fluent, pme.can_adapt_or_create, \
                pme.transfer_solved_independently, pme.historical_thoroughly_digested, \
                pme.first_thoroughly_digested_local_date \
         FROM problem_mastery_evidence pme JOIN problems p ON p.id = pme.problem_id \
         WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 \
           AND p.external_problem_key = ?2",
    )
    .bind(problem.contest().contest_id() as i64)
    .bind(problem.index())
    .fetch_optional(pool)
    .await
    .map_err(|_| ReviewAttemptError::PersistenceUnavailable)?;
    let Some(row) = row else {
        return Ok(ProblemMasteryProjection {
            current: acm_os_domain::ProblemMasteryEvidence::default(),
            historical_thoroughly_digested: false,
            first_thoroughly_digested_local_date: None,
        });
    };
    let first_date = row
        .7
        .as_deref()
        .map(acm_os_domain::LocalDate::parse_iso)
        .transpose()
        .map_err(|_| ReviewAttemptError::IntegrityViolation)?;
    if row.6 != first_date.is_some() {
        return Err(ReviewAttemptError::IntegrityViolation);
    }
    Ok(ProblemMasteryProjection {
        current: acm_os_domain::ProblemMasteryEvidence {
            recalls_problem: row.0,
            multiple_solutions_clear: row.1,
            knowledge_understood: row.2,
            implementation_fluent: row.3,
            can_adapt_or_create: row.4,
            transfer_solved_independently: row.5,
        },
        historical_thoroughly_digested: row.6,
        first_thoroughly_digested_local_date: first_date,
    })
}

impl PersonalNoteDeletionPort for DatabaseRuntime {
    async fn prepare_personal_note_deletion(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<PreparedPersonalNoteDeletion, PersonalNoteDeletionError> {
        let binding = match self.read_personal_note_projection(problem).await {
            Ok(PersonalNoteReadState::Ready { binding, .. }) => binding,
            Ok(PersonalNoteReadState::LocationAnomaly { .. }) => {
                return Err(PersonalNoteDeletionError::LocationAnomaly)
            }
            Ok(PersonalNoteReadState::VaultUnavailable { .. }) => {
                return Err(PersonalNoteDeletionError::VaultUnavailable)
            }
            Err(PersonalNoteReadError::ProblemNotFound) => {
                return Err(PersonalNoteDeletionError::ProblemNotFound)
            }
            Err(PersonalNoteReadError::NotPersonal) => {
                return Err(PersonalNoteDeletionError::NotPersonal)
            }
            Err(PersonalNoteReadError::BindingUnavailable) => {
                return Err(PersonalNoteDeletionError::BindingUnavailable)
            }
            Err(PersonalNoteReadError::FileReadFailed) => {
                return Err(PersonalNoteDeletionError::VaultUnavailable)
            }
            Err(PersonalNoteReadError::InvalidUtf8) => {
                return Err(PersonalNoteDeletionError::IntegrityViolation)
            }
            Err(PersonalNoteReadError::PersistenceUnavailable) => {
                return Err(PersonalNoteDeletionError::PersistenceUnavailable)
            }
        };
        let in_progress_review: i64 = sqlx::query_scalar(
            "SELECT EXISTS(\
                SELECT 1 FROM review_attempts ra \
                JOIN problems p ON p.id = ra.problem_id \
                WHERE p.platform = 'codeforces' AND p.external_contest_key = ?1 \
                  AND p.external_problem_key = ?2 AND ra.attempt_status = 'in_progress'\
             )",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_one(
            self._pool
                .as_ref()
                .ok_or(PersonalNoteDeletionError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        if in_progress_review != 0 {
            return Err(PersonalNoteDeletionError::ReviewInProgress);
        }
        let workspace = self
            .load_workspace_configuration()
            .await
            .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?
            .ok_or(PersonalNoteDeletionError::VaultUnavailable)?;
        let vault = std::fs::canonicalize(workspace.active_vault_path())
            .map_err(|_| PersonalNoteDeletionError::VaultUnavailable)?;
        let target = std::fs::canonicalize(vault.join(&binding.vault_relative_path))
            .map_err(|_| PersonalNoteDeletionError::VaultUnavailable)?;
        if !target.starts_with(&vault) || !target.is_file() {
            return Err(PersonalNoteDeletionError::BindingUnavailable);
        }
        let bytes =
            std::fs::read(&target).map_err(|_| PersonalNoteDeletionError::VaultUnavailable)?;
        if sha256_hex(&bytes) != binding.content_digest {
            return Err(PersonalNoteDeletionError::ConcurrentModification);
        }

        let recovery_root = self
            .recovery_root
            .as_ref()
            .ok_or(PersonalNoteDeletionError::PersistenceUnavailable)?;
        let recovery_key = format!(
            "{}:{}:{}",
            problem.contest().platform(),
            problem.contest().contest_id(),
            problem.index()
        );
        let bucket = recovery_root
            .join("deleted-personal-notes")
            .join(sha256_hex(recovery_key.as_bytes()));
        std::fs::create_dir_all(&bucket)
            .map_err(|_| PersonalNoteDeletionError::RecoveryCopyFailed)?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| PersonalNoteDeletionError::RecoveryCopyFailed)?
            .as_nanos();
        let recovery_copy = bucket.join(format!("{timestamp}-{}.md", binding.content_digest));
        let mut copy = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&recovery_copy)
            .map_err(|_| PersonalNoteDeletionError::RecoveryCopyFailed)?;
        copy.write_all(&bytes)
            .and_then(|_| copy.sync_all())
            .map_err(|_| PersonalNoteDeletionError::RecoveryCopyFailed)?;

        let final_bytes = std::fs::read(&target)
            .map_err(|_| PersonalNoteDeletionError::ConcurrentModification)?;
        if sha256_hex(&final_bytes) != binding.content_digest {
            return Err(PersonalNoteDeletionError::ConcurrentModification);
        }
        std::fs::remove_file(&target).map_err(|_| PersonalNoteDeletionError::FileDeleteFailed)?;
        match target.try_exists() {
            Ok(false) => {}
            Ok(true) => return Err(PersonalNoteDeletionError::ConcurrentModification),
            Err(_) => {
                restore_exact_file(&target, &bytes)
                    .map_err(|_| PersonalNoteDeletionError::CompensationFailed)?;
                return Err(PersonalNoteDeletionError::FileDeleteFailed);
            }
        }
        Ok(PreparedPersonalNoteDeletion {
            vault_relative_path: binding.vault_relative_path,
            content_digest: binding.content_digest,
            recovery_copy_path: recovery_copy.to_string_lossy().into_owned(),
        })
    }

    async fn commit_personal_note_deletion(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        prepared: &PreparedPersonalNoteDeletion,
    ) -> Result<ProblemLifecycleState, PersonalNoteDeletionError> {
        let pool = self
            ._pool
            .as_ref()
            .ok_or(PersonalNoteDeletionError::PersistenceUnavailable)?;
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        let row: Option<(i64, String, String, String, String)> = sqlx::query_as(
            "SELECT p.id, p.identity_type, pls.learning_status, \
                    fb.vault_relative_path, fb.content_digest \
             FROM problems p \
             LEFT JOIN problem_learning_states pls ON pls.problem_id = p.id \
             LEFT JOIN file_bindings fb ON fb.problem_id = p.id \
             WHERE p.platform = 'codeforces' \
               AND p.external_contest_key = ?1 \
               AND p.external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        let (problem_id, identity_type, status, relative_path, digest) =
            row.ok_or(PersonalNoteDeletionError::ProblemNotFound)?;
        if identity_type != "personal" {
            return Err(PersonalNoteDeletionError::NotPersonal);
        }
        if relative_path != prepared.vault_relative_path || digest != prepared.content_digest {
            return Err(PersonalNoteDeletionError::ConcurrentModification);
        }
        let in_progress_review: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_attempts \
             WHERE problem_id = ?1 AND attempt_status = 'in_progress'",
        )
        .bind(problem_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        if in_progress_review != 0 {
            return Err(PersonalNoteDeletionError::ReviewInProgress);
        }
        let status = parse_learning_status(&status).map_err(map_lifecycle_to_deletion_error)?;
        let decision = acm_os_domain::ProblemLifecycleEngine::decide(
            status,
            acm_os_domain::ProblemLifecycleAction::DeletePersonalNote,
        )
        .map_err(|_| PersonalNoteDeletionError::IntegrityViolation)?;

        sqlx::query(
            "UPDATE review_cycles \
             SET cycle_status = 'cancelled', next_due_local_date = NULL, \
                 ended_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE problem_id = ?1 AND cycle_status = 'active'",
        )
        .bind(problem_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        let learning_status_since_utc: Option<String> = sqlx::query_scalar(
            "UPDATE problem_learning_states \
             SET learning_status = 'unstarted', \
                 learning_status_since_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE problem_id = ?1 AND learning_status = ?2 \
             RETURNING learning_status_since_utc",
        )
        .bind(problem_id)
        .bind(learning_status_value(decision.previous_status))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        let learning_status_since_utc =
            learning_status_since_utc.ok_or(PersonalNoteDeletionError::ConcurrentModification)?;
        let binding_delete = sqlx::query(
            "DELETE FROM file_bindings \
             WHERE problem_id = ?1 AND vault_relative_path = ?2 AND content_digest = ?3",
        )
        .bind(problem_id)
        .bind(&prepared.vault_relative_path)
        .bind(&prepared.content_digest)
        .execute(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        if binding_delete.rows_affected() != 1 {
            return Err(PersonalNoteDeletionError::ConcurrentModification);
        }
        let identity_update = sqlx::query(
            "UPDATE problems SET identity_type = 'lightweight' \
             WHERE id = ?1 AND identity_type = 'personal'",
        )
        .bind(problem_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        if identity_update.rows_affected() != 1 {
            return Err(PersonalNoteDeletionError::ConcurrentModification);
        }
        transaction
            .commit()
            .await
            .map_err(|_| PersonalNoteDeletionError::PersistenceUnavailable)?;
        Ok(ProblemLifecycleState {
            identity_type: ProblemIdentityType::Lightweight,
            learning_status: acm_os_domain::LearningStatus::Unstarted,
            learning_status_since_utc,
            active_review_cycle: None,
        })
    }

    async fn restore_deleted_personal_note(
        &self,
        prepared: &PreparedPersonalNoteDeletion,
    ) -> Result<(), PersonalNoteDeletionError> {
        let workspace = self
            .load_workspace_configuration()
            .await
            .map_err(|_| PersonalNoteDeletionError::CompensationFailed)?
            .ok_or(PersonalNoteDeletionError::CompensationFailed)?;
        let vault = std::fs::canonicalize(workspace.active_vault_path())
            .map_err(|_| PersonalNoteDeletionError::CompensationFailed)?;
        let target = vault.join(&prepared.vault_relative_path);
        if !target.starts_with(&vault)
            || target
                .try_exists()
                .map_err(|_| PersonalNoteDeletionError::CompensationFailed)?
        {
            return Err(PersonalNoteDeletionError::CompensationFailed);
        }
        let bytes = std::fs::read(&prepared.recovery_copy_path)
            .map_err(|_| PersonalNoteDeletionError::CompensationFailed)?;
        if sha256_hex(&bytes) != prepared.content_digest {
            return Err(PersonalNoteDeletionError::CompensationFailed);
        }
        restore_exact_file(&target, &bytes)
            .map_err(|_| PersonalNoteDeletionError::CompensationFailed)
    }
}

fn restore_exact_file(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut restored = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(target)?;
    restored.write_all(bytes)?;
    restored.sync_all()
}

fn map_lifecycle_to_deletion_error(error: ProblemLifecycleError) -> PersonalNoteDeletionError {
    match error {
        ProblemLifecycleError::ProblemNotFound => PersonalNoteDeletionError::ProblemNotFound,
        ProblemLifecycleError::NotPersonal => PersonalNoteDeletionError::NotPersonal,
        ProblemLifecycleError::InvalidTransition
        | ProblemLifecycleError::InvalidLocalDate
        | ProblemLifecycleError::IntegrityViolation => {
            PersonalNoteDeletionError::IntegrityViolation
        }
        ProblemLifecycleError::PersistenceUnavailable => {
            PersonalNoteDeletionError::PersistenceUnavailable
        }
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
                self.update_binding_state(problem_id, "external_source_unavailable", &last_binding)
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

        let markdown =
            std::str::from_utf8(&resolved.bytes).map_err(|_| PersonalNoteReadError::InvalidUtf8)?;
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

impl PersonalNotePatchPort for DatabaseRuntime {
    async fn add_prerequisite_link(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        target: &PrerequisiteLinkTarget,
    ) -> Result<PersonalNoteBinding, PersonalNotePatchError> {
        let state = self
            .read_personal_note_projection(problem)
            .await
            .map_err(map_personal_note_read_to_patch_error)?;
        let expected = match state {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            PersonalNoteReadState::LocationAnomaly { .. } => {
                return Err(PersonalNotePatchError::LocationAnomaly)
            }
            PersonalNoteReadState::VaultUnavailable { .. } => {
                return Err(PersonalNotePatchError::VaultUnavailable)
            }
        };
        let configuration = self
            .load_workspace_configuration()
            .await
            .map_err(|_| PersonalNotePatchError::PersistenceUnavailable)?
            .ok_or(PersonalNotePatchError::VaultUnavailable)?;
        let recovery_root = self
            .recovery_root
            .clone()
            .ok_or(PersonalNotePatchError::PersistenceUnavailable)?;
        let active_vault = configuration.active_vault_path().to_owned();
        let relative_path = expected.vault_relative_path.clone();
        let recovery_key = format!(
            "codeforces:{}:{}",
            problem.contest().contest_id(),
            problem.index()
        );
        let target = target.clone();
        let outcome = tokio::task::spawn_blocking(move || {
            crate::safe_patch::add_prerequisite_link(
                &active_vault,
                &relative_path,
                &recovery_root,
                &recovery_key,
                &target,
                |_| {},
            )
        })
        .await
        .map_err(|_| PersonalNotePatchError::WriteFailed)?
        .map_err(map_safe_patch_error)?;
        self.commit_patch_outcome(problem, &expected, outcome).await
    }

    async fn add_extra_problem_link(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        target: &ExtraProblemLinkTarget,
    ) -> Result<PersonalNoteBinding, PersonalNotePatchError> {
        let state = self
            .read_personal_note_projection(problem)
            .await
            .map_err(map_personal_note_read_to_patch_error)?;
        let expected = match state {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            PersonalNoteReadState::LocationAnomaly { .. } => {
                return Err(PersonalNotePatchError::LocationAnomaly);
            }
            PersonalNoteReadState::VaultUnavailable { .. } => {
                return Err(PersonalNotePatchError::VaultUnavailable);
            }
        };
        let configuration = self
            .load_workspace_configuration()
            .await
            .map_err(|_| PersonalNotePatchError::PersistenceUnavailable)?
            .ok_or(PersonalNotePatchError::VaultUnavailable)?;
        let recovery_root = self
            .recovery_root
            .clone()
            .ok_or(PersonalNotePatchError::PersistenceUnavailable)?;
        let active_vault = configuration.active_vault_path().to_owned();
        let relative_path = expected.vault_relative_path.clone();
        let recovery_key = format!(
            "codeforces:{}:{}",
            problem.contest().contest_id(),
            problem.index()
        );
        let target = target.clone();
        let outcome = tokio::task::spawn_blocking(move || {
            crate::safe_patch::add_extra_problem_link(
                &active_vault,
                &relative_path,
                &recovery_root,
                &recovery_key,
                &target,
                |_| {},
            )
        })
        .await
        .map_err(|_| PersonalNotePatchError::WriteFailed)?
        .map_err(map_safe_patch_error)?;

        self.commit_patch_outcome(problem, &expected, outcome).await
    }
}

impl DatabaseRuntime {
    async fn commit_patch_outcome(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
        expected: &PersonalNoteBinding,
        outcome: crate::safe_patch::SafePatchOutcome,
    ) -> Result<PersonalNoteBinding, PersonalNotePatchError> {
        let problem_id: i64 = sqlx::query_scalar(
            "SELECT id FROM problems WHERE platform = 'codeforces' \
             AND external_contest_key = ?1 AND external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_optional(
            self._pool
                .as_ref()
                .ok_or(PersonalNotePatchError::PersistenceUnavailable)?,
        )
        .await
        .map_err(|_| PersonalNotePatchError::PersistenceUnavailable)?
        .ok_or(PersonalNotePatchError::ProblemNotFound)?;
        let resolved = ResolvedNoteFile {
            relative_path: outcome.relative_path,
            bytes: Vec::new(),
            content_digest: outcome.content_digest,
            windows_file_key: outcome.windows_file_key,
            relocated: false,
        };
        if !self
            .commit_resolved_binding(problem_id, &expected, &resolved)
            .await
            .map_err(map_personal_note_read_to_patch_error)?
        {
            return Err(PersonalNotePatchError::BindingUnavailable);
        }
        let old_cache_key = format!(
            "codeforces:{}:{}:{}",
            problem.contest().contest_id(),
            problem.index(),
            expected.vault_relative_path
        );
        let new_cache_key = format!(
            "codeforces:{}:{}:{}",
            problem.contest().contest_id(),
            problem.index(),
            resolved.relative_path
        );
        let mut cache = self
            .markdown_projection_cache
            .lock()
            .map_err(|_| PersonalNotePatchError::PersistenceUnavailable)?;
        cache.remove(&old_cache_key);
        cache.remove(&new_cache_key);
        Ok(resolved_binding(&resolved))
    }
}

fn map_personal_note_read_to_patch_error(error: PersonalNoteReadError) -> PersonalNotePatchError {
    match error {
        PersonalNoteReadError::ProblemNotFound => PersonalNotePatchError::ProblemNotFound,
        PersonalNoteReadError::NotPersonal => PersonalNotePatchError::NotPersonal,
        PersonalNoteReadError::BindingUnavailable => PersonalNotePatchError::BindingUnavailable,
        PersonalNoteReadError::FileReadFailed => PersonalNotePatchError::WriteFailed,
        PersonalNoteReadError::InvalidUtf8 => PersonalNotePatchError::InvalidUtf8,
        PersonalNoteReadError::PersistenceUnavailable => {
            PersonalNotePatchError::PersistenceUnavailable
        }
    }
}

fn map_safe_patch_error(error: crate::safe_patch::SafePatchError) -> PersonalNotePatchError {
    use crate::safe_patch::SafePatchError;
    match error {
        SafePatchError::VaultUnavailable => PersonalNotePatchError::VaultUnavailable,
        SafePatchError::BindingUnavailable => PersonalNotePatchError::BindingUnavailable,
        SafePatchError::InvalidUtf8 => PersonalNotePatchError::InvalidUtf8,
        SafePatchError::TargetSectionMissing => PersonalNotePatchError::TargetSectionMissing,
        SafePatchError::TargetSectionAmbiguous => PersonalNotePatchError::TargetSectionAmbiguous,
        SafePatchError::LinkAlreadyPresent => PersonalNotePatchError::LinkAlreadyPresent,
        SafePatchError::ConcurrentModification => PersonalNotePatchError::ConcurrentModification,
        SafePatchError::RecoveryCopyFailed => PersonalNotePatchError::RecoveryCopyFailed,
        SafePatchError::WriteFailed => PersonalNotePatchError::WriteFailed,
        SafePatchError::VerificationFailed => PersonalNotePatchError::VerificationFailed,
    }
}

impl WorkspaceConfigurationPort for DatabaseRuntime {
    async fn resolve_directory(&self, path: &str) -> Result<String, WorkspacePathResolutionError> {
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

        row.map(
            |(active_vault_path, problem_root_path, knowledge_root_path)| {
                WorkspaceConfiguration::from_resolved(
                    active_vault_path,
                    problem_root_path,
                    knowledge_root_path,
                )
                .map_err(|_| WorkspacePersistenceError::Unavailable)
            },
        )
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
    let vault =
        std::fs::canonicalize(active_vault).map_err(|_| PersonalNoteError::WorkspaceUnavailable)?;
    let root =
        std::fs::canonicalize(problem_root).map_err(|_| PersonalNoteError::WorkspaceUnavailable)?;
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
    let resolved =
        std::fs::canonicalize(&target).map_err(|_| PersonalNoteError::FileVerificationFailed)?;
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
    let vault =
        std::fs::canonicalize(active_vault).map_err(|_| PersonalNoteError::CompensationFailed)?;
    let target = vault.join(Path::new(&file.vault_relative_path));
    let resolved =
        std::fs::canonicalize(&target).map_err(|_| PersonalNoteError::CompensationFailed)?;
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
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| ContestImportPersistenceError::Unavailable)?;
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
                    .map(|slot| {
                        (
                            slot.problem.contest().contest_id() as i64,
                            slot.problem.index().to_owned(),
                        )
                    })
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
                        "INSERT INTO problem_learning_states (problem_id) VALUES (?1) \
                         ON CONFLICT(problem_id) DO NOTHING",
                    )
                    .bind(problem_id)
                    .execute(&mut *transaction)
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
        transaction
            .commit()
            .await
            .map_err(|_| ContestImportPersistenceError::Unavailable)?;
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
        .fetch_all(
            self.contest_pool()
                .map_err(|_| ContestReadError::Unavailable)?,
        )
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        rows.into_iter()
            .map(|(id, title, status, count, missing)| {
                Ok(ContestShelfItem {
                    contest: acm_os_domain::CodeforcesContestIdentity::new(id as u64)
                        .map_err(|_| ContestReadError::Unavailable)?,
                    title,
                    import_status: match status.as_str() {
                        "incomplete" => ContestImportStatus::Incomplete,
                        "complete" => ContestImportStatus::Complete,
                        _ => return Err(ContestReadError::Unavailable),
                    },
                    problem_count: count as u32,
                    missing_snapshot_count: missing as u32,
                })
            })
            .collect()
    }

    async fn contest_detail(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
    ) -> Result<ContestDetail, ContestReadError> {
        let pool = self
            .contest_pool()
            .map_err(|_| ContestReadError::Unavailable)?;
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
        let problems = rows
            .into_iter()
            .map(|(index, title, rating, snapshot, identity_type)| {
                Ok(LightweightProblemItem {
                    problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), index)
                        .map_err(|_| ContestReadError::Unavailable)?,
                    title,
                    rating: rating.map(|value| value as u32),
                    has_statement_snapshot: snapshot != 0,
                    identity_type: parse_problem_identity_type(&identity_type)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
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

    async fn list_lightweight_problems(
        &self,
    ) -> Result<Vec<LightweightProblemItem>, ContestReadError> {
        let rows: Vec<(i64, String, String, Option<i64>, i64, String)> = sqlx::query_as(
            "SELECT p.external_contest_key, p.external_problem_key, p.title, p.rating, \
                    EXISTS(SELECT 1 FROM problem_statement_snapshots ss WHERE ss.problem_id = p.id), \
                    p.identity_type \
             FROM problems p ORDER BY p.external_contest_key DESC, p.external_problem_key ASC",
        )
        .fetch_all(self.contest_pool().map_err(|_| ContestReadError::Unavailable)?)
        .await
        .map_err(|_| ContestReadError::Unavailable)?;
        rows.into_iter()
            .map(
                |(contest_id, index, title, rating, snapshot, identity_type)| {
                    Ok(LightweightProblemItem {
                        problem: acm_os_domain::CodeforcesProblemIdentity::new(
                            acm_os_domain::CodeforcesContestIdentity::new(contest_id as u64)
                                .map_err(|_| ContestReadError::Unavailable)?,
                            index,
                        )
                        .map_err(|_| ContestReadError::Unavailable)?,
                        title,
                        rating: rating.map(|value| value as u32),
                        has_statement_snapshot: snapshot != 0,
                        identity_type: parse_problem_identity_type(&identity_type)?,
                    })
                },
            )
            .collect()
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
        let (
            title,
            rating,
            source_url,
            sanitized_html,
            identity_type,
            relative_path,
            digest,
            file_key,
        ) = row.ok_or(ContestReadError::NotFound)?;
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
        let lifecycle =
            self.load_problem_lifecycle(problem)
                .await
                .map_err(|error| match error {
                    ProblemLifecycleError::ProblemNotFound => ContestReadError::NotFound,
                    _ => ContestReadError::Unavailable,
                })?;
        if lifecycle.identity_type != identity_type {
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
            lifecycle,
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
        Ok(rows
            .into_iter()
            .map(|(local_ref, media_type, bytes)| LocalStatementAsset {
                local_ref,
                media_type,
                bytes,
            })
            .collect())
    }
}

impl DatabaseRuntime {
    fn personal_note_pool(&self) -> Result<&SqlitePool, PersonalNoteError> {
        self._pool
            .as_ref()
            .ok_or(PersonalNoteError::PersistenceUnavailable)
    }

    fn contest_pool(&self) -> Result<&SqlitePool, ContestImportPersistenceError> {
        self._pool
            .as_ref()
            .ok_or(ContestImportPersistenceError::Unavailable)
    }

    async fn import_state(
        &self,
        contest_id: i64,
    ) -> Result<PersistedContestImport, ContestImportPersistenceError> {
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
        let missing_snapshot_problems = missing
            .into_iter()
            .map(|(contest_id, index)| {
                acm_os_domain::CodeforcesProblemIdentity::new(
                    acm_os_domain::CodeforcesContestIdentity::new(contest_id as u64)
                        .map_err(|_| ContestImportPersistenceError::Unavailable)?,
                    index,
                )
                .map_err(|_| ContestImportPersistenceError::Unavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let status = if missing_snapshot_problems.is_empty() {
            ContestImportStatus::Complete
        } else {
            ContestImportStatus::Incomplete
        };
        sqlx::query("UPDATE contests SET import_status = ?1 WHERE id = ?2")
            .bind(match status {
                ContestImportStatus::Incomplete => "incomplete",
                ContestImportStatus::Complete => "complete",
            })
            .bind(contest_id)
            .execute(pool)
            .await
            .map_err(|_| ContestImportPersistenceError::Unavailable)?;
        Ok(PersistedContestImport {
            status,
            missing_snapshot_problems,
        })
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

    let (existing_schema_version, legacy_m5_schema) = if database_exists {
        let inspection_pool = connect_read_only(&database_path).await?;
        verify_integrity(&inspection_pool).await?;
        let version = inspect_schema_version(&inspection_pool).await?;
        let legacy_m5_schema = version == 10 && is_legacy_m5_schema(&inspection_pool).await?;
        if version <= supported_schema_version {
            if !legacy_m5_schema {
                validate_schema_contract(&inspection_pool, version).await?;
            }
        }
        inspection_pool.close().await;
        (version, legacy_m5_schema)
    } else {
        (0, false)
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

    if legacy_m5_schema {
        upgrade_legacy_m5_schema(&pool).await?;
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
        recovery_root: Some(app_private_data.join("markdown-recovery")),
    })
}

const LEGACY_M5_MIGRATION_10_CHECKSUM: &[u8] = &[
    0xa0, 0xde, 0xa4, 0xff, 0x7e, 0xf1, 0x2a, 0x40, 0xaa, 0x5a, 0x64, 0x33, 0xa5, 0x80, 0xd1, 0xaa,
    0x9f, 0x56, 0x14, 0x30, 0xb3, 0xc6, 0xc0, 0x27, 0x8e, 0x10, 0x28, 0x5a, 0x0e, 0x68, 0xe0, 0x5b,
    0x64, 0x22, 0x4e, 0x59, 0x8b, 0x38, 0x7a, 0x16, 0x20, 0x27, 0xe3, 0x96, 0x70, 0xd6, 0x2e, 0xdd,
];
const LEGACY_M5_LEARNING_STATES_SQL: &str = "\
    CREATE TABLE problem_learning_states (\
        problem_id INTEGER PRIMARY KEY REFERENCES problems(id) ON DELETE RESTRICT,\
        learning_status TEXT NOT NULL DEFAULT 'unstarted' CHECK (learning_status IN ('unstarted', 'upsolve_pending', 'learning', 'waiting_cold_start', 'relearning', 'long_term_review')),\
        learning_status_since_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
    )";
const LEGACY_M5_TODAY_PLANS_SQL: &str = "\
    CREATE TABLE today_plans (\
        id TEXT PRIMARY KEY CHECK (length(id) = 36),\
        local_date TEXT NOT NULL UNIQUE,\
        budget_minutes INTEGER NOT NULL CHECK (budget_minutes >= 0),\
        planned_minutes INTEGER NOT NULL CHECK (planned_minutes >= 0),\
        over_budget_minutes INTEGER NOT NULL CHECK (over_budget_minutes >= 0),\
        review_only_streak INTEGER NOT NULL CHECK (review_only_streak BETWEEN 0 AND 2),\
        created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
    )";
const LEGACY_M5_TODAY_ENTRIES_SQL: &str = "\
    CREATE TABLE today_plan_entries (\
        id TEXT PRIMARY KEY CHECK (length(id) = 36),\
        today_plan_id TEXT NOT NULL REFERENCES today_plans(id) ON DELETE RESTRICT,\
        problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,\
        review_attempt_id TEXT REFERENCES review_attempts(id) ON DELETE RESTRICT,\
        lane TEXT NOT NULL CHECK (lane IN ('carry_in', 'review', 'study')),\
        reason TEXT NOT NULL CHECK (reason IN ('continue_review', 'continue_learning', 'due_first_cold_start', 'due_long_term_review', 'relearn', 'upsolve')),\
        planning_cost_minutes INTEGER NOT NULL CHECK (planning_cost_minutes IN (30, 60)),\
        position INTEGER NOT NULL CHECK (position >= 0),\
        UNIQUE (today_plan_id, problem_id),\
        UNIQUE (today_plan_id, position),\
        CHECK ((reason = 'continue_review' AND lane = 'carry_in' AND review_attempt_id IS NOT NULL) OR (reason != 'continue_review' AND review_attempt_id IS NULL))\
    )";

async fn is_legacy_m5_schema(pool: &SqlitePool) -> Result<bool, StartupRecoveryReason> {
    let checksum: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT checksum FROM _sqlx_migrations WHERE version = 10 AND success = 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| StartupRecoveryReason::MigrationLedgerInvalid)?;
    if checksum.as_deref() != Some(LEGACY_M5_MIGRATION_10_CHECKSUM) {
        return Ok(false);
    }
    let objects: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT type, name, tbl_name FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if objects != expected_schema_objects(10) {
        return Ok(false);
    }
    validate_app_metadata_contract(pool, 10).await?;

    let learning_columns: Vec<SqliteColumnContract> =
        sqlx::query_as("PRAGMA table_xinfo('problem_learning_states')")
            .fetch_all(pool)
            .await
            .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    let today_columns: Vec<SqliteColumnContract> =
        sqlx::query_as("PRAGMA table_xinfo('today_plan_entries')")
            .fetch_all(pool)
            .await
            .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    let learning_names = learning_columns
        .iter()
        .map(|column| column.1.as_str())
        .collect::<Vec<_>>();
    let today_names = today_columns
        .iter()
        .map(|column| column.1.as_str())
        .collect::<Vec<_>>();
    if learning_names != ["problem_id", "learning_status", "learning_status_since_utc"]
        || today_names
            != [
                "id",
                "today_plan_id",
                "problem_id",
                "review_attempt_id",
                "lane",
                "reason",
                "planning_cost_minutes",
                "position",
            ]
    {
        return Ok(false);
    }
    for (table, expected) in [
        ("problem_learning_states", LEGACY_M5_LEARNING_STATES_SQL),
        ("today_plans", LEGACY_M5_TODAY_PLANS_SQL),
        ("today_plan_entries", LEGACY_M5_TODAY_ENTRIES_SQL),
    ] {
        let actual: Option<String> =
            sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1")
                .bind(table)
                .fetch_optional(pool)
                .await
                .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
        if actual
            .as_deref()
            .is_none_or(|sql| normalize_schema_sql(sql) != normalize_schema_sql(expected))
        {
            return Ok(false);
        }
    }

    validate_workspace_settings_contract(pool).await?;
    validate_contest_import_contract(pool, 10).await?;
    validate_personal_note_contract(pool).await?;
    validate_learning_lifecycle_contract(pool, 9).await?;
    validate_review_attempt_contract(pool, 10).await?;
    validate_review_help_usage_contract(pool).await?;
    validate_review_completion_contract(pool).await?;
    validate_problem_mastery_contract(pool).await?;

    let inconsistent_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM today_plans tp WHERE \
         tp.planned_minutes != COALESCE((SELECT SUM(planning_cost_minutes) \
             FROM today_plan_entries e WHERE e.today_plan_id = tp.id), 0) \
         OR tp.over_budget_minutes != MAX(tp.planned_minutes - tp.budget_minutes, 0) \
         OR EXISTS (SELECT 1 FROM today_plan_entries e WHERE e.today_plan_id = tp.id \
             AND e.position != (SELECT COUNT(*) FROM today_plan_entries earlier \
                 WHERE earlier.today_plan_id = e.today_plan_id AND earlier.position < e.position))",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    Ok(inconsistent_today == 0)
}

async fn upgrade_legacy_m5_schema(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
    let current_checksum = MIGRATOR
        .iter()
        .find(|migration| migration.version == 10)
        .ok_or(StartupRecoveryReason::MigrationFailed)?
        .checksum
        .clone();
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| StartupRecoveryReason::MigrationFailed)?;
    for statement in [
        "ALTER TABLE problem_learning_states ADD COLUMN pinned_priority INTEGER NOT NULL DEFAULT 0 CHECK (pinned_priority IN (0, 1))",
        "ALTER TABLE today_plan_entries RENAME TO today_plan_entries_legacy_m5",
        "CREATE TABLE today_plan_entries (\
            id TEXT PRIMARY KEY CHECK (length(id) = 36),\
            today_plan_id TEXT NOT NULL REFERENCES today_plans(id) ON DELETE RESTRICT,\
            problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE RESTRICT,\
            review_attempt_id TEXT REFERENCES review_attempts(id) ON DELETE RESTRICT,\
            lane TEXT NOT NULL CHECK (lane IN ('carry_in', 'review', 'study')),\
            reason TEXT NOT NULL CHECK (reason IN ('continue_review', 'continue_learning', 'due_first_cold_start', 'due_long_term_review', 'relearn', 'upsolve')),\
            planning_cost_minutes INTEGER NOT NULL CHECK (planning_cost_minutes IN (30, 60)),\
            position INTEGER NOT NULL CHECK (position >= 0),\
            entry_origin TEXT NOT NULL DEFAULT 'auto' CHECK (entry_origin IN ('auto', 'manual')),\
            entry_status TEXT NOT NULL DEFAULT 'not_started' CHECK (entry_status IN ('not_started', 'in_progress', 'completed', 'unavailable')),\
            reconciliation_added INTEGER NOT NULL DEFAULT 0 CHECK (reconciliation_added IN (0, 1)),\
            UNIQUE (today_plan_id, problem_id),\
            UNIQUE (today_plan_id, position),\
            CHECK (reason != 'continue_review' OR (lane = 'carry_in' AND review_attempt_id IS NOT NULL))\
        )",
        "INSERT INTO today_plan_entries (id, today_plan_id, problem_id, review_attempt_id, lane, reason, planning_cost_minutes, position, entry_origin, entry_status, reconciliation_added) \
         SELECT e.id, e.today_plan_id, e.problem_id, e.review_attempt_id, e.lane, e.reason, e.planning_cost_minutes, e.position, 'auto', \
                CASE WHEN ra.attempt_status = 'in_progress' THEN 'in_progress' WHEN ra.attempt_status = 'completed' THEN 'completed' ELSE 'not_started' END, 0 \
         FROM today_plan_entries_legacy_m5 e LEFT JOIN review_attempts ra ON ra.id = e.review_attempt_id",
        "DROP TABLE today_plan_entries_legacy_m5",
        "CREATE INDEX today_plan_entries_by_plan ON today_plan_entries(today_plan_id, position)",
    ] {
        sqlx::query(statement)
            .execute(&mut *transaction)
            .await
            .map_err(|_| StartupRecoveryReason::MigrationFailed)?;
    }
    sqlx::query("UPDATE _sqlx_migrations SET checksum = ?1 WHERE version = 10")
        .bind(current_checksum.as_ref())
        .execute(&mut *transaction)
        .await
        .map_err(|_| StartupRecoveryReason::MigrationFailed)?;
    transaction
        .commit()
        .await
        .map_err(|_| StartupRecoveryReason::MigrationFailed)
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

    if !matches!(
        schema_version,
        1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16
    ) {
        return Err(StartupRecoveryReason::UnsupportedSchema {
            found: schema_version,
            supported: supported_schema_version(),
        });
    }

    let expected_objects = expected_schema_objects(schema_version);
    if objects != expected_objects {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }

    validate_app_metadata_contract(pool, schema_version).await?;

    if schema_version >= 2 {
        validate_workspace_settings_contract(pool).await?;
    }
    if schema_version >= 3 {
        validate_contest_import_contract(pool, schema_version).await?;
    }
    if schema_version >= 4 {
        validate_personal_note_contract(pool).await?;
    }
    if schema_version >= 5 {
        validate_learning_lifecycle_contract(pool, schema_version).await?;
    }
    if schema_version >= 6 {
        validate_review_attempt_contract(pool, schema_version).await?;
    }
    if schema_version >= 7 {
        validate_review_help_usage_contract(pool).await?;
    }
    if schema_version >= 8 {
        validate_review_completion_contract(pool).await?;
    }
    if schema_version >= 9 {
        validate_problem_mastery_contract(pool).await?;
    }
    if schema_version >= 10 {
        validate_today_plan_contract(pool).await?;
    }
    if schema_version >= 11 {
        validate_weekly_acm_budget_contract(pool).await?;
    }
    if schema_version >= 12 {
        validate_knowledge_index_contract(pool).await?;
    }
    if schema_version >= 13 {
        validate_table_columns(
            pool,
            "knowledge_link_index",
            &[
                "source_kind",
                "source_id",
                "target_ref",
                "target_knowledge_node_id",
                "resolution",
            ],
        )
        .await?;
    }
    if schema_version >= 14 {
        validate_table_columns(
            pool,
            "knowledge_understanding_states",
            &[
                "knowledge_node_id",
                "current_level",
                "historical_highest_level",
                "first_reached_highest_local_date",
                "updated_at_utc",
            ],
        )
        .await?;
    }
    if schema_version >= 15 {
        validate_table_columns(
            pool,
            "knowledge_candidate_records",
            &[
                "problem_id",
                "fingerprint",
                "target_ref",
                "disposition",
                "created_at_utc",
                "updated_at_utc",
            ],
        )
        .await?;
    }

    Ok(())
}

fn expected_schema_objects(schema_version: i64) -> Vec<(String, String, String)> {
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
    if schema_version >= 5 {
        expected_objects.extend([
            (
                "index".to_owned(),
                "one_active_review_cycle_per_problem".to_owned(),
                "review_cycles".to_owned(),
            ),
            (
                "table".to_owned(),
                "problem_learning_states".to_owned(),
                "problem_learning_states".to_owned(),
            ),
            (
                "table".to_owned(),
                "review_cycles".to_owned(),
                "review_cycles".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    if schema_version >= 6 {
        expected_objects.extend([
            (
                "index".to_owned(),
                "one_in_progress_review_attempt_per_problem".to_owned(),
                "review_attempts".to_owned(),
            ),
            (
                "table".to_owned(),
                "review_attempts".to_owned(),
                "review_attempts".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    if schema_version >= 7 {
        expected_objects.push((
            "table".to_owned(),
            "review_help_usage_events".to_owned(),
            "review_help_usage_events".to_owned(),
        ));
        expected_objects.sort();
    }
    if schema_version >= 8 {
        expected_objects.extend([
            (
                "table".to_owned(),
                "review_failure_reasons".to_owned(),
                "review_failure_reasons".to_owned(),
            ),
            (
                "table".to_owned(),
                "review_void_events".to_owned(),
                "review_void_events".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    if schema_version >= 9 {
        expected_objects.push((
            "table".to_owned(),
            "problem_mastery_evidence".to_owned(),
            "problem_mastery_evidence".to_owned(),
        ));
        expected_objects.sort();
    }
    if schema_version >= 10 {
        expected_objects.extend([
            (
                "index".to_owned(),
                "today_plan_entries_by_plan".to_owned(),
                "today_plan_entries".to_owned(),
            ),
            (
                "table".to_owned(),
                "today_plan_entries".to_owned(),
                "today_plan_entries".to_owned(),
            ),
            (
                "table".to_owned(),
                "today_plans".to_owned(),
                "today_plans".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    if schema_version >= 11 {
        expected_objects.push((
            "table".to_owned(),
            "weekly_acm_budgets".to_owned(),
            "weekly_acm_budgets".to_owned(),
        ));
        expected_objects.sort();
    }
    if schema_version >= 12 {
        expected_objects.extend([
            (
                "index".to_owned(),
                "knowledge_discovery_index_by_name".to_owned(),
                "knowledge_discovery_index".to_owned(),
            ),
            (
                "table".to_owned(),
                "knowledge_discovery_index".to_owned(),
                "knowledge_discovery_index".to_owned(),
            ),
            (
                "table".to_owned(),
                "knowledge_file_bindings".to_owned(),
                "knowledge_file_bindings".to_owned(),
            ),
            (
                "table".to_owned(),
                "knowledge_nodes".to_owned(),
                "knowledge_nodes".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    if schema_version >= 13 {
        expected_objects.extend([
            (
                "index".to_owned(),
                "knowledge_link_index_by_target".to_owned(),
                "knowledge_link_index".to_owned(),
            ),
            (
                "table".to_owned(),
                "knowledge_link_index".to_owned(),
                "knowledge_link_index".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    if schema_version >= 14 {
        expected_objects.push((
            "table".to_owned(),
            "knowledge_understanding_states".to_owned(),
            "knowledge_understanding_states".to_owned(),
        ));
        expected_objects.sort();
    }
    if schema_version >= 15 {
        expected_objects.extend([
            (
                "index".to_owned(),
                "knowledge_candidate_records_by_problem".to_owned(),
                "knowledge_candidate_records".to_owned(),
            ),
            (
                "table".to_owned(),
                "knowledge_candidate_records".to_owned(),
                "knowledge_candidate_records".to_owned(),
            ),
        ]);
        expected_objects.sort();
    }
    expected_objects
}

async fn validate_contest_import_contract(
    pool: &SqlitePool,
    schema_version: i64,
) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "contests",
        &[
            "id",
            "platform",
            "external_contest_key",
            "title",
            "source_url",
            "starts_at_utc",
            "import_status",
            "created_at_utc",
        ],
    )
    .await?;
    let problem_columns = if schema_version >= 4 {
        vec![
            "id",
            "platform",
            "external_contest_key",
            "external_problem_key",
            "title",
            "rating",
            "source_url",
            "created_at_utc",
            "identity_type",
        ]
    } else {
        vec![
            "id",
            "platform",
            "external_contest_key",
            "external_problem_key",
            "title",
            "rating",
            "source_url",
            "created_at_utc",
        ]
    };
    validate_table_columns(pool, "problems", &problem_columns).await?;
    validate_table_columns(
        pool,
        "contest_problems",
        &["contest_id", "problem_id", "ordinal", "import_state"],
    )
    .await?;
    validate_table_columns(
        pool,
        "problem_statement_snapshots",
        &[
            "problem_id",
            "source_html",
            "sanitized_html",
            "captured_at_utc",
        ],
    )
    .await?;
    validate_table_columns(
        pool,
        "problem_statement_assets",
        &["problem_id", "local_ref", "media_type", "bytes"],
    )
    .await
}

async fn validate_personal_note_contract(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
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

async fn validate_learning_lifecycle_contract(
    pool: &SqlitePool,
    schema_version: i64,
) -> Result<(), StartupRecoveryReason> {
    let mut columns = vec!["problem_id", "learning_status", "learning_status_since_utc"];
    if schema_version >= 10 {
        columns.push("pinned_priority");
    }
    validate_table_columns(pool, "problem_learning_states", &columns).await?;
    validate_table_columns(
        pool,
        "review_cycles",
        &[
            "id",
            "problem_id",
            "cycle_number",
            "cycle_status",
            "stage",
            "schedule_rule_version",
            "next_due_local_date",
            "created_at_utc",
            "ended_at_utc",
        ],
    )
    .await?;
    let missing_states: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM problems p \
         LEFT JOIN problem_learning_states pls ON pls.problem_id = p.id \
         WHERE pls.problem_id IS NULL",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if missing_states != 0 {
        return Err(StartupRecoveryReason::IntegrityCheckFailed);
    }
    Ok(())
}

async fn validate_review_attempt_contract(
    pool: &SqlitePool,
    schema_version: i64,
) -> Result<(), StartupRecoveryReason> {
    let mut columns = vec![
        "id",
        "problem_id",
        "review_cycle_id",
        "attempt_type",
        "attempt_status",
        "scheduled_due_local_date",
        "started_early",
        "judgement_rule_version",
        "started_at_utc",
        "completed_at_utc",
    ];
    if schema_version >= 8 {
        columns.extend([
            "judgement",
            "completed_local_date",
            "final_ac",
            "first_submission_result",
            "final_result",
            "total_submissions",
            "idea_independent",
            "implementation_independent",
            "debug_independence",
            "external_help",
            "evidence_codes_json",
        ]);
    }
    validate_table_columns(pool, "review_attempts", &columns).await
}

async fn validate_review_help_usage_contract(
    pool: &SqlitePool,
) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "review_help_usage_events",
        &[
            "id",
            "review_attempt_id",
            "help_level",
            "source_digest",
            "revealed_at_utc",
        ],
    )
    .await
}

async fn validate_review_completion_contract(
    pool: &SqlitePool,
) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "review_failure_reasons",
        &["review_attempt_id", "reason_code", "other_text"],
    )
    .await?;
    validate_table_columns(
        pool,
        "review_void_events",
        &["id", "review_attempt_id", "reason", "voided_at_utc"],
    )
    .await?;
    let inconsistent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM review_attempts ra WHERE \
         (ra.attempt_status = 'completed' AND (ra.judgement IS NULL OR ra.completed_local_date IS NULL \
            OR ra.final_ac IS NULL OR ra.first_submission_result IS NULL OR ra.final_result IS NULL \
            OR ra.total_submissions IS NULL OR ra.idea_independent IS NULL \
            OR ra.implementation_independent IS NULL OR ra.debug_independence IS NULL \
            OR ra.external_help IS NULL OR ra.evidence_codes_json IS NULL)) \
         OR (ra.attempt_status != 'completed' AND (ra.judgement IS NOT NULL \
            OR ra.completed_local_date IS NOT NULL OR ra.final_ac IS NOT NULL \
            OR ra.first_submission_result IS NOT NULL OR ra.final_result IS NOT NULL \
            OR ra.total_submissions IS NOT NULL OR ra.idea_independent IS NOT NULL \
            OR ra.implementation_independent IS NOT NULL OR ra.debug_independence IS NOT NULL \
            OR ra.external_help IS NOT NULL OR ra.evidence_codes_json IS NOT NULL)) \
         OR (ra.attempt_status = 'void') != EXISTS(SELECT 1 FROM review_void_events v WHERE v.review_attempt_id = ra.id) \
         OR (ra.attempt_status = 'completed' AND json_valid(ra.evidence_codes_json) != 1) \
         OR (ra.attempt_status = 'completed' AND ra.final_ac != (ra.final_result = 'accepted')) \
         OR (ra.attempt_status = 'completed' AND ra.total_submissions = 1 \
             AND ra.first_submission_result != ra.final_result) \
         OR (ra.attempt_status = 'completed' AND ra.judgement = 'mastered' \
             AND EXISTS(SELECT 1 FROM review_failure_reasons r WHERE r.review_attempt_id = ra.id)) \
         OR (ra.attempt_status = 'completed' AND ra.judgement != 'mastered' \
             AND NOT EXISTS(SELECT 1 FROM review_failure_reasons r WHERE r.review_attempt_id = ra.id)) \
         OR (ra.attempt_status != 'completed' \
             AND EXISTS(SELECT 1 FROM review_failure_reasons r WHERE r.review_attempt_id = ra.id)) \
         OR EXISTS(SELECT 1 FROM review_cycles rc WHERE rc.stage > 6)",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if inconsistent == 0 {
        Ok(())
    } else {
        Err(StartupRecoveryReason::IntegrityCheckFailed)
    }
}

async fn validate_problem_mastery_contract(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "problem_mastery_evidence",
        &[
            "problem_id",
            "recalls_problem",
            "multiple_solutions_clear",
            "knowledge_understood",
            "implementation_fluent",
            "can_adapt_or_create",
            "transfer_solved_independently",
            "historical_thoroughly_digested",
            "first_thoroughly_digested_local_date",
            "updated_at_utc",
        ],
    )
    .await?;
    let inconsistent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM problem_mastery_evidence WHERE \
         historical_thoroughly_digested != (first_thoroughly_digested_local_date IS NOT NULL)",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if inconsistent == 0 {
        Ok(())
    } else {
        Err(StartupRecoveryReason::IntegrityCheckFailed)
    }
}

async fn validate_today_plan_contract(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "today_plans",
        &[
            "id",
            "local_date",
            "budget_minutes",
            "planned_minutes",
            "over_budget_minutes",
            "review_only_streak",
            "created_at_utc",
        ],
    )
    .await?;
    validate_table_columns(
        pool,
        "today_plan_entries",
        &[
            "id",
            "today_plan_id",
            "problem_id",
            "review_attempt_id",
            "lane",
            "reason",
            "planning_cost_minutes",
            "position",
            "entry_origin",
            "entry_status",
            "reconciliation_added",
        ],
    )
    .await?;
    let inconsistent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM today_plans tp WHERE \
         tp.planned_minutes != COALESCE((SELECT SUM(planning_cost_minutes) \
             FROM today_plan_entries e WHERE e.today_plan_id = tp.id), 0) \
         OR tp.over_budget_minutes != MAX(tp.planned_minutes - tp.budget_minutes, 0) \
         OR EXISTS (SELECT 1 FROM today_plan_entries e WHERE e.today_plan_id = tp.id \
             AND e.position != (SELECT COUNT(*) FROM today_plan_entries earlier \
                 WHERE earlier.today_plan_id = e.today_plan_id AND earlier.position < e.position)) \
         OR EXISTS (SELECT 1 FROM today_plan_entries e \
             JOIN review_attempts ra ON ra.id = e.review_attempt_id \
             WHERE e.today_plan_id = tp.id AND \
               ((e.entry_status = 'in_progress' AND ra.attempt_status != 'in_progress') \
                OR (e.entry_status = 'completed' AND ra.attempt_status != 'completed')))",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if inconsistent == 0 {
        Ok(())
    } else {
        Err(StartupRecoveryReason::IntegrityCheckFailed)
    }
}

async fn validate_weekly_acm_budget_contract(
    pool: &SqlitePool,
) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(
        pool,
        "weekly_acm_budgets",
        &["weekday", "budget_minutes", "updated_at_utc"],
    )
    .await?;
    let inconsistent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM weekly_acm_budgets WHERE weekday NOT BETWEEN 1 AND 7 OR budget_minutes < 0",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if inconsistent == 0 {
        Ok(())
    } else {
        Err(StartupRecoveryReason::IntegrityCheckFailed)
    }
}

async fn validate_knowledge_index_contract(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
    validate_table_columns(pool, "knowledge_nodes", &["id", "created_at_utc"]).await?;
    validate_table_columns(
        pool,
        "knowledge_file_bindings",
        &[
            "knowledge_node_id",
            "vault_relative_path",
            "windows_file_key",
            "content_digest",
            "location_state",
            "updated_at_utc",
        ],
    )
    .await?;
    validate_table_columns(
        pool,
        "knowledge_discovery_index",
        &[
            "knowledge_node_id",
            "display_name",
            "vault_relative_path",
            "content_digest",
            "indexed_at_utc",
        ],
    )
    .await?;
    let inconsistent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM knowledge_discovery_index i \
         JOIN knowledge_file_bindings b ON b.knowledge_node_id = i.knowledge_node_id \
         WHERE b.location_state != 'ready' OR b.vault_relative_path != i.vault_relative_path \
            OR b.content_digest != i.content_digest",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if inconsistent == 0 {
        Ok(())
    } else {
        Err(StartupRecoveryReason::IntegrityCheckFailed)
    }
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
        "problem_learning_states" => "PRAGMA table_xinfo('problem_learning_states')",
        "review_cycles" => "PRAGMA table_xinfo('review_cycles')",
        "review_attempts" => "PRAGMA table_xinfo('review_attempts')",
        "review_help_usage_events" => "PRAGMA table_xinfo('review_help_usage_events')",
        "review_failure_reasons" => "PRAGMA table_xinfo('review_failure_reasons')",
        "review_void_events" => "PRAGMA table_xinfo('review_void_events')",
        "problem_mastery_evidence" => "PRAGMA table_xinfo('problem_mastery_evidence')",
        "today_plans" => "PRAGMA table_xinfo('today_plans')",
        "today_plan_entries" => "PRAGMA table_xinfo('today_plan_entries')",
        "weekly_acm_budgets" => "PRAGMA table_xinfo('weekly_acm_budgets')",
        "knowledge_nodes" => "PRAGMA table_xinfo('knowledge_nodes')",
        "knowledge_file_bindings" => "PRAGMA table_xinfo('knowledge_file_bindings')",
        "knowledge_discovery_index" => "PRAGMA table_xinfo('knowledge_discovery_index')",
        "knowledge_link_index" => "PRAGMA table_xinfo('knowledge_link_index')",
        "knowledge_understanding_states" => "PRAGMA table_xinfo('knowledge_understanding_states')",
        "knowledge_candidate_records" => "PRAGMA table_xinfo('knowledge_candidate_records')",
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
        (
            1,
            "description".to_owned(),
            "TEXT".to_owned(),
            1,
            None,
            0,
            0,
        ),
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

async fn validate_app_metadata_columns(pool: &SqlitePool) -> Result<(), StartupRecoveryReason> {
    let actual: Vec<SqliteColumnContract> = sqlx::query_as("PRAGMA table_xinfo('app_metadata')")
        .fetch_all(pool)
        .await
        .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    let expected = vec![
        (
            0,
            "singleton".to_owned(),
            "INTEGER".to_owned(),
            0,
            None,
            1,
            0,
        ),
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

async fn validate_app_metadata_contract(
    pool: &SqlitePool,
    schema_version: i64,
) -> Result<(), StartupRecoveryReason> {
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
    let metadata: Vec<(i64, i64, String)> =
        sqlx::query_as("SELECT singleton, schema_generation, created_at_utc FROM app_metadata")
            .fetch_all(pool)
            .await
            .map_err(|_| StartupRecoveryReason::IntegrityCheckFailed)?;
    if metadata.len() == 1
        && metadata[0].0 == 1
        && metadata[0].1 == schema_version
        && !metadata[0].2.is_empty()
    {
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
        (
            0,
            "singleton".to_owned(),
            "INTEGER".to_owned(),
            0,
            None,
            1,
            0,
        ),
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
        accept_existing_knowledge_candidate, accept_today_extra_suggestion, add_extra_problem_link,
        apply_today_replan, complete_review, complete_today_entry, configure_workspace,
        confirm_knowledge_understanding, create_personal_note, delete_personal_note,
        import_codeforces_contest, list_knowledge_candidates, load_knowledge_detail,
        load_or_generate_today_snapshot, preview_today_extra_suggestions, preview_today_replan,
        query_workspace_configuration, rebuild_knowledge_index, rebuild_knowledge_relations,
        register_knowledge_candidate, reorder_today_snapshot, reveal_review_help, review_focus,
        review_help_drawer, review_history, search_knowledge_index,
        set_knowledge_candidate_disposition, start_or_resume_review, transition_problem_lifecycle,
        update_problem_mastery_evidence, void_review, weekly_acm_budget_for_date,
        ContestImportDraft, ContestImportPort, ContestImportSource, ContestImportSourceError,
        ContestImportStatus, ContestProblemSlotDraft, ContestReadPort, PersonalNoteError,
        PersonalNotePatchError, PersonalNoteReadPort, PersonalNoteReadState, ProblemIdentityType,
        ProblemLifecyclePort, ReviewCompletionInput, ReviewFailureReason, StartupGateStatus,
        StartupRecoveryReason, StatementAssetDraft, StatementSnapshotDraft, SubmissionFact,
        TodaySnapshotPort, WeeklyAcmBudgetPort, WeeklyAcmBudgetSchedule,
        WorkspaceConfigurationDraft, WorkspaceConfigurationError, WorkspaceConfigurationStatus,
        WorkspacePathField, INITIAL_PROBLEM_MARKDOWN,
    };
    use sqlx::Executor;
    use tempfile::TempDir;

    use super::*;

    struct CoreLoopContestSource {
        manifest: ContestImportDraft,
        snapshots: Vec<StatementSnapshotDraft>,
    }

    #[tokio::test]
    async fn weekly_budget_repeats_while_today_override_stays_date_local() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let schedule = WeeklyAcmBudgetSchedule {
            monday: None,
            tuesday: None,
            wednesday: Some(95),
            thursday: None,
            friday: None,
            saturday: Some(101),
            sunday: Some(0),
        };
        assert_eq!(
            runtime
                .save_weekly_acm_budget(&schedule)
                .await
                .expect("save weekly defaults"),
            schedule
        );
        let first_wednesday =
            acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("first Wednesday");
        let next_wednesday =
            acm_os_domain::LocalDate::parse_iso("2026-08-19").expect("next Wednesday");
        assert_eq!(
            weekly_acm_budget_for_date(&schedule, first_wednesday),
            Some(95)
        );
        assert_eq!(
            weekly_acm_budget_for_date(&schedule, next_wednesday),
            Some(95)
        );

        let first = load_or_generate_today_snapshot(&runtime, first_wednesday, 95)
            .await
            .expect("first Wednesday plan accepts arbitrary minutes");
        assert_eq!(first.budget_minutes, 95);
        let preview = preview_today_replan(&runtime, first_wednesday, 47)
            .await
            .expect("one-day arbitrary-minute override preview");
        let overridden = apply_today_replan(&runtime, &preview)
            .await
            .expect("apply one-day override");
        assert_eq!(overridden.budget_minutes, 47);

        let unchanged = runtime
            .load_weekly_acm_budget()
            .await
            .expect("reload weekly defaults");
        assert_eq!(unchanged, schedule);
        let next = load_or_generate_today_snapshot(
            &runtime,
            next_wednesday,
            weekly_acm_budget_for_date(&unchanged, next_wednesday).expect("Wednesday default"),
        )
        .await
        .expect("next Wednesday plan");
        assert_eq!(next.budget_minutes, 95);
        assert_eq!(
            runtime
                .load_today_snapshot(first_wednesday)
                .await
                .expect("first Wednesday read")
                .expect("first Wednesday snapshot")
                .budget_minutes,
            47
        );
    }

    impl ContestImportSource for CoreLoopContestSource {
        async fn fetch_manifest(
            &self,
            _contest: &acm_os_domain::CodeforcesContestIdentity,
        ) -> Result<ContestImportDraft, ContestImportSourceError> {
            Ok(self.manifest.clone())
        }

        async fn fetch_snapshot(
            &self,
            problem: &acm_os_domain::CodeforcesProblemIdentity,
        ) -> Result<StatementSnapshotDraft, ContestImportSourceError> {
            self.snapshots
                .iter()
                .find(|snapshot| &snapshot.problem == problem)
                .cloned()
                .ok_or(ContestImportSourceError::InvalidRemoteData)
        }
    }

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
        let mut snapshot = snapshot(
            index,
            "<img src=\"acm-os-asset://fixture\">",
            "<img src=\"acm-os-asset://fixture\">",
        );
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

    #[tokio::test]
    async fn knowledge_discovery_uses_only_real_recursive_markdown_and_fresh_reindex() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, _problems, knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        fs::create_dir_all(knowledge.join("Graphs")).expect("nested knowledge");
        fs::write(knowledge.join("Graphs/DFS.md"), "# ignored H1\n").expect("DFS markdown");
        fs::write(knowledge.join("notes.txt"), "not markdown").expect("non markdown");

        let first = rebuild_knowledge_index(&runtime)
            .await
            .expect("first discovery");
        assert_eq!(first.nodes.len(), 1);
        assert_eq!(first.nodes[0].display_name, "DFS");
        assert_eq!(
            first.nodes[0].vault_relative_path,
            "Knowledge/Graphs/DFS.md"
        );
        assert!(first.location_anomalies.is_empty());

        fs::write(knowledge.join("Segment Tree.MD"), "# external addition\n")
            .expect("external markdown");
        let before_reindex = search_knowledge_index(&runtime, "segment")
            .await
            .expect("derived search before reindex");
        assert!(before_reindex.is_empty());
        let refreshed = rebuild_knowledge_index(&runtime)
            .await
            .expect("fresh reindex");
        assert_eq!(refreshed.nodes.len(), 2);
        assert_eq!(
            search_knowledge_index(&runtime, "SEGMENT")
                .await
                .expect("case insensitive search")[0]
                .display_name,
            "Segment Tree"
        );
        assert_eq!(
            fs::read_dir(&knowledge).expect("knowledge root").count(),
            3,
            "indexing must not create empty Markdown"
        );
    }

    #[tokio::test]
    async fn knowledge_move_preserves_identity_but_ambiguous_digest_does_not_guess() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, _problems, knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        let original = knowledge.join("Old.md");
        fs::write(&original, "unique content").expect("original markdown");
        let first = rebuild_knowledge_index(&runtime)
            .await
            .expect("first discovery");
        let stable_id = first.nodes[0].knowledge_node_id.clone();

        let moved = knowledge.join("Nested/New name.md");
        fs::create_dir_all(moved.parent().expect("parent")).expect("nested root");
        fs::rename(&original, &moved).expect("deterministic move");
        let after_move = rebuild_knowledge_index(&runtime)
            .await
            .expect("move reindex");
        assert_eq!(after_move.nodes[0].knowledge_node_id, stable_id);
        assert_eq!(after_move.nodes[0].display_name, "New name");

        fs::remove_file(&moved).expect("remove moved file");
        fs::write(knowledge.join("same-a.md"), "unique content").expect("same a");
        fs::write(knowledge.join("same-b.md"), "unique content").expect("same b");
        let ambiguous = rebuild_knowledge_index(&runtime)
            .await
            .expect("ambiguous reindex");
        assert_eq!(ambiguous.nodes.len(), 2);
        assert!(ambiguous
            .nodes
            .iter()
            .all(|node| node.knowledge_node_id != stable_id));
        assert_eq!(ambiguous.location_anomalies.len(), 1);
        assert_eq!(ambiguous.location_anomalies[0].knowledge_node_id, stable_id);
    }

    #[tokio::test]
    async fn knowledge_derived_index_rebuild_ignores_orphan_database_nodes() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, _problems, knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        fs::write(knowledge.join("Real.md"), "real markdown").expect("real markdown");
        let first = rebuild_knowledge_index(&runtime)
            .await
            .expect("initial index");
        let stable_id = first.nodes[0].knowledge_node_id.clone();
        let pool = runtime._pool.as_ref().expect("ready pool");
        sqlx::query("DELETE FROM knowledge_discovery_index")
            .execute(pool)
            .await
            .expect("delete derived index");
        sqlx::query("INSERT INTO knowledge_nodes (id) VALUES (?1)")
            .bind(uuid::Uuid::now_v7().to_string())
            .execute(pool)
            .await
            .expect("orphan identity record");

        assert!(search_knowledge_index(&runtime, "")
            .await
            .expect("empty derived index")
            .is_empty());
        let rebuilt = rebuild_knowledge_index(&runtime)
            .await
            .expect("rebuild from Markdown");
        assert_eq!(rebuilt.nodes.len(), 1);
        assert_eq!(rebuilt.nodes[0].knowledge_node_id, stable_id);
        assert!(rebuilt.location_anomalies.is_empty());
    }

    #[tokio::test]
    async fn knowledge_relocation_requires_a_bijective_deterministic_match() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (vault, _problems, knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        fs::write(knowledge.join("One.md"), "same bytes").expect("one");
        fs::write(knowledge.join("Two.md"), "same bytes").expect("two");
        let first = rebuild_knowledge_index(&runtime)
            .await
            .expect("initial same-digest nodes");
        assert_eq!(first.nodes.len(), 2);

        fs::remove_file(knowledge.join("One.md")).expect("remove one");
        fs::remove_file(knowledge.join("Two.md")).expect("remove two");
        fs::write(vault.join("moved-outside-root.md"), "same bytes")
            .expect("one ambiguous relocation candidate");
        let ambiguous = rebuild_knowledge_index(&runtime)
            .await
            .expect("ambiguous relocation");
        assert!(ambiguous.nodes.is_empty());
        assert_eq!(ambiguous.location_anomalies.len(), 2);
    }

    #[tokio::test]
    async fn knowledge_relations_resolve_unique_links_and_preserve_ambiguity() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, _problems, knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        fs::write(knowledge.join("Graphs.md"), "# Graphs\n").expect("graphs");
        fs::write(
            knowledge.join("Other.md"),
            "# Other\n[[Graphs]] [[Missing]] [[Problems/Not Knowledge]]\n",
        )
        .expect("other");
        fs::write(
            directory.path().join("vault/Problems/Not Knowledge.md"),
            "# Not Knowledge\n",
        )
        .expect("non-knowledge markdown");
        fs::create_dir_all(knowledge.join("nested")).expect("nested");
        fs::write(knowledge.join("nested/Graphs.md"), "# Duplicate\n").expect("duplicate target");

        let relations = rebuild_knowledge_relations(&runtime)
            .await
            .expect("rebuild relations");
        assert_eq!(relations.len(), 3);
        assert!(relations.iter().any(|relation| {
            relation.target_ref == "Missing"
                && relation.resolution == acm_os_application::KnowledgeLinkResolution::Unresolved
        }));
        assert!(relations.iter().any(|relation| {
            relation.target_ref == "Graphs"
                && relation.resolution == acm_os_application::KnowledgeLinkResolution::Ambiguous
                && relation.target_knowledge_node_id.is_none()
        }));
        assert!(relations.iter().any(|relation| {
            relation.target_ref == "Problems/Not Knowledge"
                && relation.resolution
                    == acm_os_application::KnowledgeLinkResolution::NonKnowledgeTarget
                && relation.target_knowledge_node_id.is_none()
        }));
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_link_index")
            .fetch_one(runtime._pool.as_ref().expect("pool"))
            .await
            .expect("relation index count");
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn knowledge_relations_read_nodes_from_a_nested_knowledge_root() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let vault = directory.path().join("vault");
        let problems = vault.join("Problems");
        let knowledge = vault.join("Notes/Knowledge");
        fs::create_dir_all(&problems).expect("problem root");
        fs::create_dir_all(&knowledge).expect("nested knowledge root");
        configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: vault.to_string_lossy().into_owned(),
                problem_root_path: problems.to_string_lossy().into_owned(),
                knowledge_root_path: knowledge.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect("configure nested root");
        fs::write(knowledge.join("Target.md"), "# Target\n").expect("target");
        fs::write(knowledge.join("Source.md"), "# Source\n[[Target]]\n").expect("source");

        let relations = rebuild_knowledge_relations(&runtime)
            .await
            .expect("relations from nested root");
        assert!(relations.iter().any(|relation| {
            relation.source_kind == "knowledge"
                && relation.target_ref == "Target"
                && relation.resolution == acm_os_application::KnowledgeLinkResolution::Resolved
        }));
    }

    #[tokio::test]
    async fn problem_knowledge_relations_only_come_from_a_unique_prerequisite_section() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let binding = match runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("read personal note")
        {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            other => panic!("expected ready personal note, got {other:?}"),
        };
        let knowledge = problems.parent().expect("vault").join("Knowledge");
        fs::write(knowledge.join("Graphs.md"), "# Graphs\n").expect("knowledge markdown");
        fs::write(
            vault.join(&binding.vault_relative_path),
            "# Problem\n\n[[IgnoredOutsideSection]]\n\n## 前置知识\n- [[Graphs#DFS|Traversal]]\n",
        )
        .expect("problem markdown");

        let relations = rebuild_knowledge_relations(&runtime)
            .await
            .expect("rebuild relations");
        let problem_relations = relations
            .iter()
            .filter(|relation| relation.source_kind == "problem")
            .collect::<Vec<_>>();
        assert_eq!(problem_relations.len(), 1);
        assert_eq!(problem_relations[0].target_ref, "Graphs");
        assert_eq!(
            problem_relations[0].resolution,
            acm_os_application::KnowledgeLinkResolution::Resolved
        );
        assert!(problem_relations[0].target_knowledge_node_id.is_some());

        fs::write(
            vault.join(&binding.vault_relative_path),
            "# Problem\n\n## 前置知识\n- [[Graphs]]\n\n## 前置知识\n- [[Graphs]]\n",
        )
        .expect("ambiguous problem markdown");
        let rebuilt = rebuild_knowledge_relations(&runtime)
            .await
            .expect("rebuild after duplicate section");
        assert!(rebuilt
            .iter()
            .all(|relation| relation.source_kind != "problem"));
        let persisted_problem_relations: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM knowledge_link_index WHERE source_kind = 'problem'",
        )
        .fetch_one(runtime._pool.as_ref().expect("pool"))
        .await
        .expect("problem relation count");
        assert_eq!(persisted_problem_relations, 0);
    }

    #[tokio::test]
    async fn knowledge_understanding_is_user_confirmed_and_preserves_historical_highest() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, _problems, knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        fs::write(knowledge.join("Flow.md"), "# Flow\n").expect("knowledge markdown");
        let node = rebuild_knowledge_index(&runtime)
            .await
            .expect("index")
            .nodes
            .remove(0);
        let first_date = acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("date");
        let deep = confirm_knowledge_understanding(
            &runtime,
            &node.knowledge_node_id,
            acm_os_domain::KnowledgeUnderstandingLevel::Deep,
            first_date,
        )
        .await
        .expect("confirm deep");
        assert_eq!(
            deep.current,
            acm_os_domain::KnowledgeUnderstandingLevel::Deep
        );
        let later = acm_os_domain::LocalDate::parse_iso("2026-08-20").expect("later");
        let lowered = confirm_knowledge_understanding(
            &runtime,
            &node.knowledge_node_id,
            acm_os_domain::KnowledgeUnderstandingLevel::Vague,
            later,
        )
        .await
        .expect("confirm lower");
        assert_eq!(
            lowered.current,
            acm_os_domain::KnowledgeUnderstandingLevel::Vague
        );
        assert_eq!(
            lowered.historical_highest,
            acm_os_domain::KnowledgeUnderstandingLevel::Deep
        );
        assert_eq!(lowered.first_reached_highest_on, first_date);
        assert_eq!(
            confirm_knowledge_understanding(
                &runtime,
                "00000000-0000-0000-0000-000000000000",
                acm_os_domain::KnowledgeUnderstandingLevel::Basic,
                later
            )
            .await,
            Err(acm_os_application::KnowledgeIndexError::IntegrityViolation)
        );
    }

    #[tokio::test]
    async fn knowledge_detail_projects_fresh_neighbors_understanding_and_related_problems() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let binding = match runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("read personal note")
        {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            other => panic!("expected ready personal note, got {other:?}"),
        };
        let knowledge = problems.parent().expect("vault").join("Knowledge");
        fs::write(knowledge.join("Target.md"), "# Target\n[[Outgoing]]\n")
            .expect("target markdown");
        fs::write(knowledge.join("Outgoing.md"), "# Outgoing\n").expect("outgoing markdown");
        fs::write(knowledge.join("Incoming.md"), "# Incoming\n[[Target]]\n")
            .expect("incoming markdown");
        fs::write(
            vault.join(binding.vault_relative_path),
            "# Problem\n\n## 前置知识\n- [[Target]]\n",
        )
        .expect("problem prerequisite");
        let index = rebuild_knowledge_index(&runtime)
            .await
            .expect("knowledge index");
        let target = index
            .nodes
            .iter()
            .find(|node| node.display_name == "Target")
            .expect("target node");

        let initial = load_knowledge_detail(&runtime, &target.knowledge_node_id)
            .await
            .expect("initial detail");
        assert!(initial.understanding.is_none());
        assert_eq!(
            initial
                .incoming
                .iter()
                .map(|node| node.display_name.as_str())
                .collect::<Vec<_>>(),
            ["Incoming"]
        );
        assert_eq!(
            initial
                .outgoing
                .iter()
                .map(|node| node.display_name.as_str())
                .collect::<Vec<_>>(),
            ["Outgoing"]
        );
        assert_eq!(initial.related_problems.len(), 1);
        assert_eq!(initial.related_problems[0].problem, problem);
        assert_eq!(initial.related_problems[0].title, "Problem A");

        let confirmed_on = acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("date");
        confirm_knowledge_understanding(
            &runtime,
            &target.knowledge_node_id,
            acm_os_domain::KnowledgeUnderstandingLevel::Basic,
            confirmed_on,
        )
        .await
        .expect("confirm understanding");
        let confirmed = load_knowledge_detail(&runtime, &target.knowledge_node_id)
            .await
            .expect("confirmed detail");
        assert_eq!(
            confirmed.understanding.expect("understanding").current,
            acm_os_domain::KnowledgeUnderstandingLevel::Basic
        );
        assert_eq!(
            load_knowledge_detail(&runtime, "00000000-0000-0000-0000-000000000000").await,
            Err(acm_os_application::KnowledgeIndexError::KnowledgeNodeNotFound)
        );
    }

    #[tokio::test]
    async fn knowledge_candidates_preserve_user_disposition_without_creating_authority() {
        let (_directory, runtime, vault, _problems, problem) = personal_note_fixture().await;
        let binding = match runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("read personal note")
        {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            other => panic!("expected ready personal note, got {other:?}"),
        };
        let note_path = vault.join(&binding.vault_relative_path);
        let original_markdown = fs::read(&note_path).expect("original markdown");
        let pool = runtime._pool.as_ref().expect("ready pool");
        let nodes_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_nodes")
            .fetch_one(pool)
            .await
            .expect("knowledge node count");
        let relations_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_link_index")
            .fetch_one(pool)
            .await
            .expect("knowledge relation count");
        let fingerprint = "A1".repeat(32);

        let pending =
            register_knowledge_candidate(&runtime, &problem, &fingerprint, "  Segment Tree  ")
                .await
                .expect("register candidate");
        assert_eq!(pending.fingerprint, fingerprint.to_ascii_lowercase());
        assert_eq!(pending.target_ref, "Segment Tree");
        assert_eq!(pending.disposition, KnowledgeCandidateDisposition::Pending);

        let accepted_intent = set_knowledge_candidate_disposition(
            &runtime,
            &problem,
            &fingerprint,
            KnowledgeCandidateDisposition::AcceptedIntent,
        )
        .await
        .expect("accept candidate intent");
        assert_eq!(
            accepted_intent.disposition,
            KnowledgeCandidateDisposition::AcceptedIntent
        );
        let ignored = set_knowledge_candidate_disposition(
            &runtime,
            &problem,
            &fingerprint,
            KnowledgeCandidateDisposition::Ignored,
        )
        .await
        .expect("ignore candidate");
        assert_eq!(ignored.disposition, KnowledgeCandidateDisposition::Ignored);
        let repeated =
            register_knowledge_candidate(&runtime, &problem, &fingerprint, "Segment Trees")
                .await
                .expect("repeat candidate");
        assert_eq!(repeated.disposition, KnowledgeCandidateDisposition::Ignored);
        assert_eq!(repeated.target_ref, "Segment Trees");
        let listed = list_knowledge_candidates(&runtime, &problem)
            .await
            .expect("list candidates");
        assert_eq!(listed, vec![repeated]);

        assert_eq!(
            fs::read(&note_path).expect("unchanged markdown"),
            original_markdown
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM knowledge_nodes")
                .fetch_one(pool)
                .await
                .expect("knowledge node count after"),
            nodes_before
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM knowledge_link_index")
                .fetch_one(pool)
                .await
                .expect("knowledge relation count after"),
            relations_before
        );

        let lightweight = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("lightweight problem");
        assert_eq!(
            register_knowledge_candidate(&runtime, &lightweight, &"b".repeat(64), "Graphs").await,
            Err(KnowledgeCandidateError::NotPersonal)
        );
        assert_eq!(
            register_knowledge_candidate(&runtime, &problem, "bad", "Graphs").await,
            Err(KnowledgeCandidateError::InvalidFingerprint)
        );
    }

    #[tokio::test]
    async fn knowledge_reevaluation_suggestion_requires_three_distinct_new_mastered_problems() {
        let (directory, runtime, _vault, problems, first_problem) = personal_note_fixture().await;
        let knowledge = problems.parent().expect("vault").join("Knowledge");
        fs::write(knowledge.join("Reevaluation.md"), "# Reevaluation\n")
            .expect("knowledge markdown");
        let target = rebuild_knowledge_index(&runtime)
            .await
            .expect("knowledge index")
            .nodes
            .into_iter()
            .find(|node| node.display_name == "Reevaluation")
            .expect("target node");
        let confirmed_on = acm_os_domain::LocalDate::parse_iso("2026-08-01").expect("date");
        confirm_knowledge_understanding(
            &runtime,
            &target.knowledge_node_id,
            acm_os_domain::KnowledgeUnderstandingLevel::Basic,
            confirmed_on,
        )
        .await
        .expect("initial confirmation");
        let pool = runtime._pool.as_ref().expect("ready pool");
        sqlx::query("UPDATE knowledge_understanding_states SET updated_at_utc = '2026-08-01T00:00:00.000Z' WHERE knowledge_node_id = ?1")
            .bind(&target.knowledge_node_id).execute(pool).await.expect("fix confirmation time");

        let mut problems_to_link = vec![first_problem];
        for index in ["B", "C"] {
            let problem = acm_os_domain::CodeforcesProblemIdentity::new(
                acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
                index,
            )
            .expect("problem");
            if index == "C" {
                sqlx::query("INSERT INTO problems (platform, external_contest_key, external_problem_key, title, rating, source_url) VALUES ('codeforces', 1979, 'C', 'Problem C', 1000, 'https://codeforces.com/contest/1979/problem/C')")
                    .execute(pool).await.expect("third problem");
                let problem_id: i64 = sqlx::query_scalar("SELECT id FROM problems WHERE external_contest_key = 1979 AND external_problem_key = 'C'")
                    .fetch_one(pool).await.expect("third problem id");
                sqlx::query("INSERT INTO problem_learning_states (problem_id) VALUES (?1)")
                    .bind(problem_id)
                    .execute(pool)
                    .await
                    .expect("third lifecycle");
            }
            create_personal_note(&runtime, &problem)
                .await
                .expect("personal note");
            problems_to_link.push(problem);
        }
        for problem in &problems_to_link {
            acm_os_application::add_prerequisite_link(&runtime, problem, "Reevaluation".to_owned())
                .await
                .expect("formal prerequisite link");
        }
        rebuild_knowledge_relations(&runtime)
            .await
            .expect("relations");

        for (position, problem) in problems_to_link.iter().enumerate() {
            let problem_id: i64 = sqlx::query_scalar("SELECT id FROM problems WHERE external_contest_key = 1979 AND external_problem_key = ?1")
                .bind(problem.index()).fetch_one(pool).await.expect("problem id");
            let cycle_id = uuid::Uuid::now_v7().to_string();
            sqlx::query("INSERT INTO review_cycles (id, problem_id, cycle_number, cycle_status, stage, schedule_rule_version, next_due_local_date) VALUES (?1, ?2, 1, 'active', 0, 1, '2026-08-02')")
                .bind(&cycle_id).bind(problem_id).execute(pool).await.expect("cycle");
            sqlx::query("INSERT INTO review_attempts (id, problem_id, review_cycle_id, attempt_type, attempt_status, scheduled_due_local_date, started_early, judgement_rule_version, started_at_utc, completed_at_utc, judgement, completed_local_date, final_ac, first_submission_result, final_result, total_submissions, idea_independent, implementation_independent, debug_independence, external_help, evidence_codes_json) VALUES (?1, ?2, ?3, 'first_cold_start', 'completed', '2026-08-02', 0, 1, '2026-08-02T00:00:00.000Z', ?4, 'mastered', '2026-08-02', 1, 'accepted', 'accepted', 1, 1, 1, 'not_needed', 'none', '[]')")
                .bind(uuid::Uuid::now_v7().to_string()).bind(problem_id).bind(&cycle_id)
                .bind(format!("2026-08-02T00:00:0{position}.000Z")).execute(pool).await.expect("mastered review");
            let suggestion = acm_os_application::load_knowledge_reevaluation_suggestion(
                &runtime,
                &target.knowledge_node_id,
            )
            .await
            .expect("suggestion");
            assert_eq!(suggestion.qualifying_problem_count, (position + 1) as u32);
            assert_eq!(suggestion.should_suggest, position == 2);
        }

        let first_id: i64 = sqlx::query_scalar("SELECT id FROM problems WHERE external_contest_key = 1979 AND external_problem_key = 'A'")
            .fetch_one(pool).await.expect("first problem id");
        let first_cycle: String =
            sqlx::query_scalar("SELECT id FROM review_cycles WHERE problem_id = ?1")
                .bind(first_id)
                .fetch_one(pool)
                .await
                .expect("first cycle");
        sqlx::query("INSERT INTO review_attempts (id, problem_id, review_cycle_id, attempt_type, attempt_status, scheduled_due_local_date, started_early, judgement_rule_version, started_at_utc, completed_at_utc, judgement, completed_local_date, final_ac, first_submission_result, final_result, total_submissions, idea_independent, implementation_independent, debug_independence, external_help, evidence_codes_json) VALUES (?1, ?2, ?3, 'first_cold_start', 'completed', '2026-08-03', 0, 1, '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z', 'mastered', '2026-08-03', 1, 'accepted', 'accepted', 1, 1, 1, 'not_needed', 'none', '[]')")
            .bind(uuid::Uuid::now_v7().to_string()).bind(first_id).bind(first_cycle).execute(pool).await.expect("duplicate mastered review");
        assert_eq!(
            acm_os_application::load_knowledge_reevaluation_suggestion(
                &runtime,
                &target.knowledge_node_id
            )
            .await
            .expect("deduplicated")
            .qualifying_problem_count,
            3
        );

        confirm_knowledge_understanding(
            &runtime,
            &target.knowledge_node_id,
            acm_os_domain::KnowledgeUnderstandingLevel::Basic,
            acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("date"),
        )
        .await
        .expect("reconfirm");
        let reset = acm_os_application::load_knowledge_reevaluation_suggestion(
            &runtime,
            &target.knowledge_node_id,
        )
        .await
        .expect("reset");
        assert_eq!(reset.qualifying_problem_count, 0);
        assert!(!reset.should_suggest);
        drop(runtime);
        let restarted = start_database(directory.path()).await;
        let after_restart = acm_os_application::load_knowledge_reevaluation_suggestion(
            &restarted,
            &target.knowledge_node_id,
        )
        .await
        .expect("suggestion after restart");
        assert_eq!(after_restart, reset);
    }

    #[tokio::test]
    async fn accepting_existing_knowledge_candidate_patches_markdown_then_verifies_relation() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let knowledge = problems.parent().expect("vault").join("Knowledge");
        fs::write(knowledge.join("Segment Tree.md"), "# Segment Tree\n")
            .expect("knowledge markdown");
        let target = rebuild_knowledge_index(&runtime)
            .await
            .expect("knowledge index")
            .nodes
            .into_iter()
            .find(|node| node.display_name == "Segment Tree")
            .expect("target node");
        let fingerprint = "cd".repeat(32);
        register_knowledge_candidate(&runtime, &problem, &fingerprint, "Segment Tree")
            .await
            .expect("register candidate");

        let accepted = accept_existing_knowledge_candidate(
            &runtime,
            &problem,
            &fingerprint,
            &target.knowledge_node_id,
        )
        .await
        .expect("accept existing node");
        assert_eq!(accepted.knowledge_node_id, target.knowledge_node_id);

        let binding = match runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("fresh problem note")
        {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            other => panic!("expected ready note, got {other:?}"),
        };
        let markdown =
            fs::read_to_string(vault.join(binding.vault_relative_path)).expect("patched markdown");
        assert!(markdown.contains("## 前置知识\n- [[Segment Tree]]"));
        assert!(list_knowledge_candidates(&runtime, &problem)
            .await
            .expect("remaining candidates")
            .is_empty());
        let detail = load_knowledge_detail(&runtime, &target.knowledge_node_id)
            .await
            .expect("knowledge detail");
        assert!(detail
            .related_problems
            .iter()
            .any(|item| item.problem == problem));
    }

    #[tokio::test]
    async fn accepted_intent_waits_for_real_markdown_and_a_second_explicit_acceptance() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let fingerprint = "ac".repeat(32);
        register_knowledge_candidate(&runtime, &problem, &fingerprint, "Fenwick Tree")
            .await
            .expect("register missing candidate");
        set_knowledge_candidate_disposition(
            &runtime,
            &problem,
            &fingerprint,
            KnowledgeCandidateDisposition::AcceptedIntent,
        )
        .await
        .expect("save accepted intent");
        let binding = match runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("read note")
        {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            other => panic!("expected ready note, got {other:?}"),
        };
        let note_path = vault.join(&binding.vault_relative_path);
        let before = fs::read(&note_path).expect("before markdown");

        fs::write(
            problems
                .parent()
                .expect("vault")
                .join("Knowledge/Fenwick Tree.md"),
            "# Fenwick Tree\n",
        )
        .expect("external knowledge markdown");
        let target = rebuild_knowledge_index(&runtime)
            .await
            .expect("fresh knowledge index")
            .nodes
            .into_iter()
            .find(|node| node.display_name == "Fenwick Tree")
            .expect("new real node");
        let listed = list_knowledge_candidates(&runtime, &problem)
            .await
            .expect("candidate remains listed");
        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0].disposition,
            KnowledgeCandidateDisposition::AcceptedIntent
        );
        assert_eq!(fs::read(&note_path).expect("still unchanged"), before);

        accept_existing_knowledge_candidate(
            &runtime,
            &problem,
            &fingerprint,
            &target.knowledge_node_id,
        )
        .await
        .expect("second explicit Safe Patch acceptance");
        assert!(fs::read_to_string(note_path)
            .expect("patched markdown")
            .contains("[[Fenwick Tree]]"));
    }

    #[tokio::test]
    async fn accepting_ambiguous_knowledge_candidate_never_writes_markdown() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let knowledge = problems.parent().expect("vault").join("Knowledge");
        fs::create_dir_all(knowledge.join("A")).expect("knowledge A");
        fs::create_dir_all(knowledge.join("B")).expect("knowledge B");
        fs::write(knowledge.join("A/Graphs.md"), "# A\n").expect("A graph");
        fs::write(knowledge.join("B/Graphs.md"), "# B\n").expect("B graph");
        let index = rebuild_knowledge_index(&runtime)
            .await
            .expect("ambiguous index");
        let target_id = index
            .nodes
            .first()
            .expect("a node")
            .knowledge_node_id
            .clone();
        let fingerprint = "ef".repeat(32);
        register_knowledge_candidate(&runtime, &problem, &fingerprint, "Graphs")
            .await
            .expect("register candidate");
        let binding = match runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("read note")
        {
            PersonalNoteReadState::Ready { binding, .. } => binding,
            other => panic!("expected ready note, got {other:?}"),
        };
        let note_path = vault.join(binding.vault_relative_path);
        let before = fs::read(&note_path).expect("before markdown");

        assert_eq!(
            accept_existing_knowledge_candidate(&runtime, &problem, &fingerprint, &target_id).await,
            Err(KnowledgeCandidateError::IntegrityViolation)
        );
        assert_eq!(fs::read(note_path).expect("unchanged markdown"), before);
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

    async fn review_ready_fixture() -> (
        TempDir,
        DatabaseRuntime,
        PathBuf,
        PathBuf,
        acm_os_domain::CodeforcesProblemIdentity,
        ReviewAttempt,
    ) {
        let (directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("lifecycle transition");
        }
        let attempt = start_or_resume_review(
            &runtime,
            &problem,
            acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due"),
        )
        .await
        .expect("start review");
        (directory, runtime, vault, problems, problem, attempt)
    }

    fn mastered_input() -> ReviewCompletionInput {
        ReviewCompletionInput {
            final_ac: true,
            first_submission: SubmissionFact {
                result: acm_os_domain::SubmissionResult::WrongAnswer,
                other_text: None,
            },
            final_submission: SubmissionFact {
                result: acm_os_domain::SubmissionResult::Accepted,
                other_text: None,
            },
            total_submissions: 2,
            idea_independent: true,
            implementation_independent: true,
            debug_independence: acm_os_domain::DebugIndependence::Independent,
            external_help: acm_os_domain::ExternalHelpLevel::None,
            failure_reasons: Vec::new(),
        }
    }

    fn files_under(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory).expect("read directory") {
                let entry = entry.expect("directory entry");
                if entry.file_type().expect("file type").is_dir() {
                    pending.push(entry.path());
                } else {
                    files.push(entry.path());
                }
            }
        }
        files
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
        pool.execute("INSERT INTO app_metadata (singleton, schema_generation) VALUES (1, 1)")
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

    async fn rewrite_as_legacy_m5_schema(pool: &SqlitePool) {
        let mut transaction = pool.begin().await.expect("legacy schema transaction");
        for statement in [
            "DROP INDEX knowledge_candidate_records_by_problem",
            "DROP TABLE knowledge_candidate_records",
            "DROP TABLE knowledge_understanding_states",
            "DROP INDEX knowledge_link_index_by_target",
            "DROP TABLE knowledge_link_index",
            "DROP INDEX knowledge_discovery_index_by_name",
            "DROP TABLE knowledge_discovery_index",
            "DROP TABLE knowledge_file_bindings",
            "DROP TABLE knowledge_nodes",
            "DROP TABLE weekly_acm_budgets",
            "ALTER TABLE problem_learning_states RENAME TO problem_learning_states_current",
            LEGACY_M5_LEARNING_STATES_SQL,
            "INSERT INTO problem_learning_states (problem_id, learning_status, learning_status_since_utc) SELECT problem_id, learning_status, learning_status_since_utc FROM problem_learning_states_current",
            "DROP TABLE problem_learning_states_current",
            "DROP INDEX today_plan_entries_by_plan",
            "ALTER TABLE today_plan_entries RENAME TO today_plan_entries_current",
            LEGACY_M5_TODAY_ENTRIES_SQL,
            "INSERT INTO today_plan_entries (id, today_plan_id, problem_id, review_attempt_id, lane, reason, planning_cost_minutes, position) SELECT id, today_plan_id, problem_id, review_attempt_id, lane, reason, planning_cost_minutes, position FROM today_plan_entries_current",
            "DROP TABLE today_plan_entries_current",
            "CREATE INDEX today_plan_entries_by_plan ON today_plan_entries(today_plan_id, position)",
            "DELETE FROM _sqlx_migrations WHERE version IN (11, 12, 13, 14, 15)",
            "UPDATE app_metadata SET schema_generation = 10 WHERE singleton = 1",
        ] {
            sqlx::query(statement)
                .execute(&mut *transaction)
                .await
                .expect("rewrite legacy schema");
        }
        sqlx::query("UPDATE _sqlx_migrations SET checksum = ?1 WHERE version = 10")
            .bind(LEGACY_M5_MIGRATION_10_CHECKSUM)
            .execute(&mut *transaction)
            .await
            .expect("restore legacy checksum");
        transaction.commit().await.expect("commit legacy schema");
    }

    #[tokio::test]
    async fn new_database_migrates_and_passes_integrity() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;

        assert_eq!(
            runtime.status(),
            &StartupGateStatus::Ready { schema_version: 15 }
        );
        let pool = runtime._pool.as_ref().expect("ready database pool");
        let ledger_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(pool)
            .await
            .expect("migration ledger");
        assert_eq!(ledger_count, 15);
        verify_integrity(pool).await.expect("database integrity");
    }

    #[tokio::test]
    async fn known_legacy_m5_schema_upgrades_without_losing_today_or_learning_facts() {
        let (directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            day,
        )
        .await
        .expect("study candidate");
        let original = load_or_generate_today_snapshot(&runtime, day, 95)
            .await
            .expect("legacy Today plan");
        assert_eq!(original.entries.len(), 1);
        let pool = runtime._pool.as_ref().expect("ready pool").clone();
        drop(runtime);
        rewrite_as_legacy_m5_schema(&pool).await;
        pool.close().await;

        let upgraded = start_database(directory.path()).await;
        assert_eq!(
            upgraded.status(),
            &StartupGateStatus::Ready { schema_version: 15 }
        );
        let restored = upgraded
            .load_today_snapshot(day)
            .await
            .expect("load restored Today")
            .expect("restored Today plan");
        assert_eq!(restored.plan_id, original.plan_id);
        assert_eq!(restored.budget_minutes, 95);
        assert_eq!(restored.entries.len(), 1);
        assert_eq!(
            restored.entries[0].problem_id,
            original.entries[0].problem_id
        );
        assert_eq!(restored.entries[0].origin, TodayEntryOrigin::Auto);
        assert_eq!(restored.entries[0].status, TodayEntryStatus::NotStarted);
        let pinned: i64 = sqlx::query_scalar(
            "SELECT pinned_priority FROM problem_learning_states WHERE problem_id = ?1",
        )
        .bind(
            restored.entries[0]
                .problem_id
                .parse::<i64>()
                .expect("problem id"),
        )
        .fetch_one(upgraded._pool.as_ref().expect("upgraded pool"))
        .await
        .expect("pinned default");
        assert_eq!(pinned, 0);
        let backups = files_under(&directory.path().join("backups").join("pre-migration"));
        assert_eq!(backups.len(), 1);
        assert!(backups[0]
            .file_name()
            .expect("backup filename")
            .to_string_lossy()
            .starts_with("schema-10-to-15-"));
    }

    #[tokio::test]
    async fn unknown_schema_near_legacy_m5_fingerprint_still_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let pool = runtime._pool.as_ref().expect("ready pool").clone();
        drop(runtime);
        rewrite_as_legacy_m5_schema(&pool).await;
        sqlx::query("ALTER TABLE today_plans ADD COLUMN unknown_state TEXT")
            .execute(&pool)
            .await
            .expect("unknown schema change");
        pool.close().await;

        let blocked = start_database(directory.path()).await;
        assert_eq!(
            blocked.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
    }

    #[tokio::test]
    async fn legacy_m5_fingerprint_with_wrong_metadata_never_runs_compatibility_writes() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let pool = runtime._pool.as_ref().expect("ready pool").clone();
        drop(runtime);
        rewrite_as_legacy_m5_schema(&pool).await;
        sqlx::query("UPDATE app_metadata SET schema_generation = 9 WHERE singleton = 1")
            .execute(&pool)
            .await
            .expect("tamper metadata generation");
        pool.close().await;

        let blocked = start_database(directory.path()).await;
        assert_eq!(
            blocked.status(),
            &StartupGateStatus::RecoveryRequired {
                reason: StartupRecoveryReason::IntegrityCheckFailed,
            }
        );
        let inspection = connect_read_only(&directory.path().join(DATABASE_FILENAME))
            .await
            .expect("inspect blocked database");
        let columns: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM pragma_table_xinfo('problem_learning_states') ORDER BY cid",
        )
        .fetch_all(&inspection)
        .await
        .expect("learning columns remain legacy");
        assert_eq!(
            columns,
            ["problem_id", "learning_status", "learning_status_since_utc"]
        );
        inspection.close().await;
    }

    #[tokio::test]
    async fn today_snapshot_is_stable_for_date_restart_and_uses_fresh_next_day_facts() {
        let (directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let first_day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("first day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            first_day,
        )
        .await
        .expect("pending study candidate");

        let generated = load_or_generate_today_snapshot(&runtime, first_day, 60)
            .await
            .expect("generate first snapshot");
        assert_eq!(generated.budget_minutes, 60);
        assert_eq!(generated.planned_minutes, 60);
        assert_eq!(generated.entries.len(), 1);
        assert_eq!(
            generated.entries[0].lane,
            acm_os_domain::TodayCandidateLane::Study
        );

        let same_process = load_or_generate_today_snapshot(&runtime, first_day, 999)
            .await
            .expect("reuse first snapshot");
        assert_eq!(same_process, generated);
        drop(runtime);

        let restarted = start_database(directory.path()).await;
        let after_restart = load_or_generate_today_snapshot(&restarted, first_day, 15)
            .await
            .expect("reuse after restart");
        assert_eq!(after_restart, generated);

        sqlx::query(
            "UPDATE problem_learning_states SET learning_status = 'unstarted', \
             learning_status_since_utc = '2026-08-13T00:00:00.000Z'",
        )
        .execute(restarted._pool.as_ref().expect("ready pool"))
        .await
        .expect("change next-day source fact");
        let next_day = acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("next day");
        let fresh = load_or_generate_today_snapshot(&restarted, next_day, 60)
            .await
            .expect("generate next-day snapshot");
        assert_ne!(fresh.plan_id, generated.plan_id);
        assert!(fresh.entries.is_empty());
        assert_eq!(fresh.planned_minutes, 0);
        assert_eq!(fresh.review_only_streak, 0);
    }

    #[tokio::test]
    async fn today_plan_and_entries_roll_back_as_one_snapshot() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        let draft = acm_os_domain::TodayPlanDraft {
            entries: vec![acm_os_domain::TodayCandidate {
                problem_id: "999999".to_owned(),
                review_attempt_id: None,
                lane: acm_os_domain::TodayCandidateLane::Study,
                reason: acm_os_domain::TodayCandidateReason::Upsolve,
                planning_cost_minutes: 60,
                pinned: false,
                learning_status_since: day,
                scheduled_due_local_date: None,
            }],
            budget_minutes: 60,
            planned_minutes: 60,
            over_budget_minutes: 0,
            unplanned_review_count: 0,
            unplanned_study_count: 0,
            next_review_only_streak: 0,
        };
        assert!(runtime
            .create_or_load_today_snapshot(day, &draft)
            .await
            .is_err());
        let counts: (i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM today_plans), \
                    (SELECT COUNT(*) FROM today_plan_entries)",
        )
        .fetch_one(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("snapshot counts");
        assert_eq!(counts, (0, 0));
    }

    #[tokio::test]
    async fn today_reconciliation_projects_review_start_and_completion_without_regeneration() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("marked date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("lifecycle transition");
        }
        let due = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due date");
        let initial = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("initial due plan");
        assert_eq!(initial.entries.len(), 1);
        assert_eq!(initial.entries[0].status, TodayEntryStatus::NotStarted);
        assert!(initial.entries[0].review_attempt_id.is_none());

        let attempt = start_or_resume_review(&runtime, &problem, due)
            .await
            .expect("start outside Today");
        let started = load_or_generate_today_snapshot(&runtime, due, 999)
            .await
            .expect("reconcile started review");
        assert_eq!(started.plan_id, initial.plan_id);
        assert_eq!(started.budget_minutes, initial.budget_minutes);
        assert_eq!(started.entries.len(), 1);
        assert_eq!(started.entries[0].status, TodayEntryStatus::InProgress);
        assert_eq!(
            started.entries[0].review_attempt_id.as_deref(),
            Some(attempt.attempt_id.as_str())
        );

        complete_review(&runtime, &attempt.attempt_id, mastered_input(), due)
            .await
            .expect("complete review");
        let completed = load_or_generate_today_snapshot(&runtime, due, 999)
            .await
            .expect("reconcile completed review");
        assert_eq!(completed.plan_id, initial.plan_id);
        assert_eq!(completed.entries.len(), 1);
        assert_eq!(completed.entries[0].status, TodayEntryStatus::Completed);
    }

    #[tokio::test]
    async fn today_reconciliation_appends_only_real_external_carry_in() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let today = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("today");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, today)
                .await
                .expect("lifecycle transition");
        }
        let initial = load_or_generate_today_snapshot(&runtime, today, 0)
            .await
            .expect("empty plan before future review");
        assert!(initial.entries.is_empty());

        let attempt = start_or_resume_review(&runtime, &problem, today)
            .await
            .expect("start early outside Today");
        let reconciled = load_or_generate_today_snapshot(&runtime, today, 0)
            .await
            .expect("append real carry-in");
        assert_eq!(reconciled.plan_id, initial.plan_id);
        assert_eq!(reconciled.entries.len(), 1);
        assert_eq!(reconciled.entries[0].status, TodayEntryStatus::InProgress);
        assert_eq!(
            reconciled.entries[0].reason,
            acm_os_domain::TodayCandidateReason::ContinueReview
        );
        assert_eq!(
            reconciled.entries[0].review_attempt_id,
            Some(attempt.attempt_id)
        );
        assert_eq!(reconciled.planned_minutes, 30);
        assert_eq!(reconciled.over_budget_minutes, 30);

        let stable = load_or_generate_today_snapshot(&runtime, today, 300)
            .await
            .expect("idempotent reconciliation");
        assert_eq!(stable, reconciled);
    }

    #[tokio::test]
    async fn today_reconciliation_does_not_treat_void_as_completed_work() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("marked date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("lifecycle transition");
        }
        let due = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due");
        let initial = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("due entry");
        let attempt = start_or_resume_review(&runtime, &problem, due)
            .await
            .expect("start review");
        load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("project in progress");
        void_review(&runtime, &attempt.attempt_id, "mistaken start")
            .await
            .expect("void attempt");
        let reconciled = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("reconcile void");
        assert_eq!(reconciled.plan_id, initial.plan_id);
        assert_eq!(reconciled.entries.len(), 1);
        assert_eq!(reconciled.entries[0].status, TodayEntryStatus::NotStarted);
        assert!(reconciled.entries[0].review_attempt_id.is_none());
    }

    #[tokio::test]
    async fn today_void_restores_an_initial_due_review_carry_in_instead_of_deleting_it() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("marked date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("lifecycle transition");
        }
        let due = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due");
        let attempt = start_or_resume_review(&runtime, &problem, due)
            .await
            .expect("start before Today generation");
        let initial = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("initial carry-in");
        assert_eq!(
            initial.entries[0].reason,
            acm_os_domain::TodayCandidateReason::ContinueReview
        );
        void_review(&runtime, &attempt.attempt_id, "mistaken start")
            .await
            .expect("void attempt");
        let reconciled = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("restore due review");
        assert_eq!(reconciled.entries.len(), 1);
        assert_eq!(
            reconciled.entries[0].lane,
            acm_os_domain::TodayCandidateLane::Review
        );
        assert_eq!(
            reconciled.entries[0].reason,
            acm_os_domain::TodayCandidateReason::DueFirstColdStart
        );
        assert_eq!(reconciled.entries[0].status, TodayEntryStatus::NotStarted);
        assert!(reconciled.entries[0].review_attempt_id.is_none());
    }

    #[tokio::test]
    async fn today_reorder_persists_complete_same_plan_permutation() {
        let (directory, runtime, _vault, _problems, problem_a) = personal_note_fixture().await;
        let problem_b = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("problem B");
        create_personal_note(&runtime, &problem_b)
            .await
            .expect("personal B");
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        for problem in [&problem_a, &problem_b] {
            transition_problem_lifecycle(
                &runtime,
                problem,
                acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
                day,
            )
            .await
            .expect("pending study candidate");
        }
        let initial = load_or_generate_today_snapshot(&runtime, day, 120)
            .await
            .expect("two-entry plan");
        assert_eq!(initial.entries.len(), 2);
        let reversed = initial
            .entries
            .iter()
            .rev()
            .map(|entry| entry.entry_id.clone())
            .collect::<Vec<_>>();
        let reordered = reorder_today_snapshot(&runtime, &initial.plan_id, &reversed)
            .await
            .expect("reorder snapshot");
        assert_eq!(
            reordered
                .entries
                .iter()
                .map(|entry| entry.entry_id.clone())
                .collect::<Vec<_>>(),
            reversed
        );
        assert_eq!(reordered.budget_minutes, initial.budget_minutes);
        assert_eq!(reordered.planned_minutes, initial.planned_minutes);
        for reordered_entry in &reordered.entries {
            let original = initial
                .entries
                .iter()
                .find(|entry| entry.entry_id == reordered_entry.entry_id)
                .expect("same entry");
            assert_eq!(reordered_entry.problem_id, original.problem_id);
            assert_eq!(reordered_entry.status, original.status);
            assert_eq!(reordered_entry.origin, original.origin);
        }

        let reopened = load_or_generate_today_snapshot(&runtime, day, 1)
            .await
            .expect("reopen without algorithmic shuffle");
        assert_eq!(reopened, reordered);
        drop(runtime);
        let restarted = start_database(directory.path()).await;
        let after_restart = load_or_generate_today_snapshot(&restarted, day, 1)
            .await
            .expect("reopen after restart");
        assert_eq!(after_restart, reordered);
    }

    #[tokio::test]
    async fn today_reorder_rejects_partial_duplicate_unknown_and_cross_plan_ids_atomically() {
        let (_directory, runtime, _vault, _problems, problem_a) = personal_note_fixture().await;
        let problem_b = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("problem B");
        create_personal_note(&runtime, &problem_b)
            .await
            .expect("personal B");
        let first_day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("first day");
        for problem in [&problem_a, &problem_b] {
            transition_problem_lifecycle(
                &runtime,
                problem,
                acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
                first_day,
            )
            .await
            .expect("pending study candidate");
        }
        let first = load_or_generate_today_snapshot(&runtime, first_day, 120)
            .await
            .expect("first plan");
        let second_day = acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("second day");
        let second = load_or_generate_today_snapshot(&runtime, second_day, 120)
            .await
            .expect("second plan");
        let original_ids = first
            .entries
            .iter()
            .map(|entry| entry.entry_id.clone())
            .collect::<Vec<_>>();
        let invalid_orders = vec![
            vec![original_ids[0].clone()],
            vec![original_ids[0].clone(), original_ids[0].clone()],
            vec![original_ids[0].clone(), uuid::Uuid::now_v7().to_string()],
            vec![original_ids[0].clone(), second.entries[1].entry_id.clone()],
        ];
        for invalid in invalid_orders {
            assert_eq!(
                reorder_today_snapshot(&runtime, &first.plan_id, &invalid).await,
                Err(TodaySnapshotError::InvalidReorder)
            );
            let unchanged = runtime
                .load_today_snapshot(first_day)
                .await
                .expect("load first plan")
                .expect("first plan exists");
            assert_eq!(
                unchanged
                    .entries
                    .iter()
                    .map(|entry| entry.entry_id.clone())
                    .collect::<Vec<_>>(),
                original_ids
            );
        }
    }

    #[tokio::test]
    async fn today_replan_preview_is_read_only_and_apply_replaces_only_auto_not_started() {
        let (_directory, runtime, _vault, _problems, problem_a) = personal_note_fixture().await;
        let problem_b = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("problem B");
        create_personal_note(&runtime, &problem_b)
            .await
            .expect("personal B");
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        for problem in [&problem_a, &problem_b] {
            transition_problem_lifecycle(
                &runtime,
                problem,
                acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
                day,
            )
            .await
            .expect("pending candidate");
        }
        let initial = load_or_generate_today_snapshot(&runtime, day, 120)
            .await
            .expect("initial plan");
        assert_eq!(initial.entries.len(), 2);

        sqlx::query("UPDATE today_plan_entries SET entry_origin = 'manual' WHERE id = ?1")
            .bind(&initial.entries[1].entry_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("mark protected manual entry");
        let protected = runtime
            .load_today_snapshot(day)
            .await
            .expect("load plan")
            .expect("plan");
        let before_rows: Vec<(String, i64, String, String)> = sqlx::query_as(
            "SELECT id, position, entry_origin, entry_status FROM today_plan_entries ORDER BY position",
        )
        .fetch_all(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("before rows");

        let preview = preview_today_replan(&runtime, day, 60)
            .await
            .expect("read-only preview");
        assert_eq!(preview.expected_snapshot, protected);
        assert_eq!(preview.proposed_budget_minutes, 60);
        assert_eq!(preview.entries.len(), 1);
        assert_eq!(
            preview.entries[0].existing_entry_id.as_deref(),
            Some(protected.entries[1].entry_id.as_str())
        );
        assert_eq!(preview.entries[0].origin, TodayEntryOrigin::Manual);
        let after_preview_rows: Vec<(String, i64, String, String)> = sqlx::query_as(
            "SELECT id, position, entry_origin, entry_status FROM today_plan_entries ORDER BY position",
        )
        .fetch_all(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("after preview rows");
        assert_eq!(after_preview_rows, before_rows);

        let applied = apply_today_replan(&runtime, &preview)
            .await
            .expect("explicit apply");
        assert_eq!(applied.plan_id, initial.plan_id);
        assert_eq!(applied.budget_minutes, 60);
        assert_eq!(applied.entries.len(), 1);
        assert_eq!(applied.entries[0].entry_id, protected.entries[1].entry_id);
        assert_eq!(applied.entries[0].origin, TodayEntryOrigin::Manual);
    }

    #[tokio::test]
    async fn today_replan_apply_rejects_stale_preview_without_writing() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            day,
        )
        .await
        .expect("pending candidate");
        let initial = load_or_generate_today_snapshot(&runtime, day, 60)
            .await
            .expect("initial plan");
        let preview = preview_today_replan(&runtime, day, 0)
            .await
            .expect("preview");
        sqlx::query("UPDATE today_plan_entries SET entry_status = 'completed' WHERE id = ?1")
            .bind(&initial.entries[0].entry_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("Today completion changed the snapshot");
        let changed = runtime
            .load_today_snapshot(day)
            .await
            .expect("load changed")
            .expect("plan");
        assert_eq!(
            apply_today_replan(&runtime, &preview).await,
            Err(TodaySnapshotError::StaleReplanPreview)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(day)
                .await
                .expect("reload")
                .expect("plan"),
            changed
        );
    }

    #[tokio::test]
    async fn today_replan_apply_rejects_tampered_entries_without_writing() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            day,
        )
        .await
        .expect("pending candidate");
        let initial = load_or_generate_today_snapshot(&runtime, day, 60)
            .await
            .expect("initial plan");
        let mut preview = preview_today_replan(&runtime, day, 60)
            .await
            .expect("preview");
        preview.entries[0].reason = acm_os_domain::TodayCandidateReason::Relearn;

        assert_eq!(
            apply_today_replan(&runtime, &preview).await,
            Err(TodaySnapshotError::StaleReplanPreview)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(day)
                .await
                .expect("reload")
                .expect("plan"),
            initial
        );
    }

    #[tokio::test]
    async fn today_done_persists_learning_entry_completion_without_changing_lifecycle() {
        let (directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let first_day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("first day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            first_day,
        )
        .await
        .expect("upsolve pending");
        let upsolve_plan = load_or_generate_today_snapshot(&runtime, first_day, 60)
            .await
            .expect("upsolve plan");
        assert_eq!(
            upsolve_plan.entries[0].reason,
            acm_os_domain::TodayCandidateReason::Upsolve
        );
        let lifecycle_before = runtime
            .load_problem_lifecycle(&problem)
            .await
            .expect("lifecycle");
        let upsolve_completed = complete_today_entry(
            &runtime,
            &upsolve_plan.plan_id,
            &upsolve_plan.entries[0].entry_id,
        )
        .await
        .expect("complete upsolve entry");
        assert_eq!(
            upsolve_completed.entries[0].status,
            TodayEntryStatus::Completed
        );
        assert_eq!(
            complete_today_entry(
                &runtime,
                &upsolve_plan.plan_id,
                &upsolve_plan.entries[0].entry_id
            )
            .await
            .expect("idempotent Today Done"),
            upsolve_completed
        );
        assert_eq!(
            runtime
                .load_problem_lifecycle(&problem)
                .await
                .expect("lifecycle"),
            lifecycle_before
        );

        let second_day = acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("second day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            second_day,
        )
        .await
        .expect("start learning");
        let learning_plan = load_or_generate_today_snapshot(&runtime, second_day, 60)
            .await
            .expect("learning plan");
        assert_eq!(
            learning_plan.entries[0].reason,
            acm_os_domain::TodayCandidateReason::ContinueLearning
        );
        let learning_before = runtime
            .load_problem_lifecycle(&problem)
            .await
            .expect("learning lifecycle");
        complete_today_entry(
            &runtime,
            &learning_plan.plan_id,
            &learning_plan.entries[0].entry_id,
        )
        .await
        .expect("complete learning entry");
        assert_eq!(
            runtime
                .load_problem_lifecycle(&problem)
                .await
                .expect("learning lifecycle"),
            learning_before
        );

        let numeric_problem_id = learning_plan.entries[0]
            .problem_id
            .parse::<i64>()
            .expect("numeric problem id");
        sqlx::query("UPDATE problem_learning_states SET learning_status = 'relearning' WHERE problem_id = ?1")
            .bind(numeric_problem_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("relearning authoritative fixture");
        let third_day = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("third day");
        let relearn_plan = load_or_generate_today_snapshot(&runtime, third_day, 60)
            .await
            .expect("relearn plan");
        assert_eq!(
            relearn_plan.entries[0].reason,
            acm_os_domain::TodayCandidateReason::Relearn
        );
        let relearn_before = runtime
            .load_problem_lifecycle(&problem)
            .await
            .expect("relearn lifecycle");
        let completed = complete_today_entry(
            &runtime,
            &relearn_plan.plan_id,
            &relearn_plan.entries[0].entry_id,
        )
        .await
        .expect("complete relearn entry");
        assert_eq!(
            runtime
                .load_problem_lifecycle(&problem)
                .await
                .expect("relearn lifecycle"),
            relearn_before
        );
        let reopened = load_or_generate_today_snapshot(&runtime, third_day, 1)
            .await
            .expect("reopen completed plan");
        assert_eq!(reopened, completed);

        drop(runtime);
        let restarted = start_database(directory.path()).await;
        let after_restart = load_or_generate_today_snapshot(&restarted, third_day, 1)
            .await
            .expect("completed plan after restart");
        assert_eq!(after_restart, completed);
    }

    #[tokio::test]
    async fn today_done_rejects_review_unavailable_unknown_and_cross_plan_without_writing() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("marked date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("review lifecycle");
        }
        let due = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due");
        let review_plan = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("review plan");
        let review_entry = &review_plan.entries[0];
        assert!(matches!(
            review_entry.reason,
            acm_os_domain::TodayCandidateReason::DueFirstColdStart
                | acm_os_domain::TodayCandidateReason::DueLongTermReview
                | acm_os_domain::TodayCandidateReason::ContinueReview
        ));
        let attempts_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM review_attempts")
            .fetch_one(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("attempt count");
        assert_eq!(
            complete_today_entry(&runtime, &review_plan.plan_id, &review_entry.entry_id).await,
            Err(TodaySnapshotError::InvalidTodayDone)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(due)
                .await
                .expect("review plan load")
                .expect("review plan"),
            review_plan
        );
        let attempts_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM review_attempts")
            .fetch_one(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("attempt count after rejection");
        assert_eq!(attempts_after, attempts_before);

        let study_problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("study problem");
        create_personal_note(&runtime, &study_problem)
            .await
            .expect("personal study problem");
        let study_day = acm_os_domain::LocalDate::parse_iso("2026-08-15").expect("study day");
        transition_problem_lifecycle(
            &runtime,
            &study_problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            study_day,
        )
        .await
        .expect("study fixture");
        let study_plan = load_or_generate_today_snapshot(&runtime, study_day, 60)
            .await
            .expect("study plan");
        let study_entry = &study_plan.entries[0];
        sqlx::query("UPDATE today_plan_entries SET entry_status = 'unavailable' WHERE id = ?1")
            .bind(&study_entry.entry_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("unavailable projection fixture");
        let unavailable = runtime
            .load_today_snapshot(study_day)
            .await
            .expect("load unavailable")
            .expect("study plan");
        let unknown_entry = uuid::Uuid::now_v7().to_string();
        let invalid_requests = [
            (study_plan.plan_id.as_str(), study_entry.entry_id.as_str()),
            (study_plan.plan_id.as_str(), unknown_entry.as_str()),
            (review_plan.plan_id.as_str(), study_entry.entry_id.as_str()),
        ];
        for (plan_id, entry_id) in invalid_requests {
            assert_eq!(
                complete_today_entry(&runtime, plan_id, entry_id).await,
                Err(TodaySnapshotError::InvalidTodayDone)
            );
            assert_eq!(
                runtime
                    .load_today_snapshot(study_day)
                    .await
                    .expect("study plan load")
                    .expect("study plan"),
                unavailable
            );
            assert_eq!(
                runtime
                    .load_today_snapshot(due)
                    .await
                    .expect("review plan load")
                    .expect("review plan"),
                review_plan
            );
        }
    }

    #[tokio::test]
    async fn today_reconciliation_projects_learning_entries_unavailable_and_restores_them() {
        let (directory, runtime, vault, _problems, problem) = personal_note_fixture().await;
        let first_day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("first day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            first_day,
        )
        .await
        .expect("upsolve pending");
        let upsolve = load_or_generate_today_snapshot(&runtime, first_day, 60)
            .await
            .expect("upsolve plan");
        let lifecycle_before = runtime
            .load_problem_lifecycle(&problem)
            .await
            .expect("lifecycle");

        let offline = directory.path().join("vault-offline");
        fs::rename(&vault, &offline).expect("make vault unavailable");
        let unavailable = load_or_generate_today_snapshot(&runtime, first_day, 1)
            .await
            .expect("Today detects unavailable vault");
        assert_eq!(unavailable.entries[0].status, TodayEntryStatus::Unavailable);
        assert_eq!(
            runtime
                .load_problem_lifecycle(&problem)
                .await
                .expect("lifecycle"),
            lifecycle_before
        );

        fs::rename(&offline, &vault).expect("restore vault");
        let restored = load_or_generate_today_snapshot(&runtime, first_day, 1)
            .await
            .expect("Today detects restored vault");
        assert_eq!(restored.entries[0].status, TodayEntryStatus::NotStarted);
        assert_eq!(restored.entries[0].entry_id, upsolve.entries[0].entry_id);

        let second_day = acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("second day");
        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            second_day,
        )
        .await
        .expect("start learning");
        let learning = load_or_generate_today_snapshot(&runtime, second_day, 60)
            .await
            .expect("learning plan");
        assert_eq!(learning.entries[0].status, TodayEntryStatus::InProgress);
        fs::rename(&vault, &offline).expect("make vault unavailable again");
        let learning_unavailable = load_or_generate_today_snapshot(&runtime, second_day, 1)
            .await
            .expect("Today detects unavailable learning note");
        assert_eq!(
            learning_unavailable.entries[0].status,
            TodayEntryStatus::Unavailable
        );
        fs::rename(&offline, &vault).expect("restore vault again");
        let learning_restored = load_or_generate_today_snapshot(&runtime, second_day, 1)
            .await
            .expect("Today detects restored learning note");
        assert_eq!(
            learning_restored.entries[0].status,
            TodayEntryStatus::InProgress
        );

        complete_today_entry(&runtime, &learning.plan_id, &learning.entries[0].entry_id)
            .await
            .expect("complete learning entry");
        fs::rename(&vault, &offline).expect("make vault unavailable after completion");
        runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("project unavailable after completion");
        let completed = load_or_generate_today_snapshot(&runtime, second_day, 1)
            .await
            .expect("completed stays completed");
        assert_eq!(completed.entries[0].status, TodayEntryStatus::Completed);

        fs::rename(&offline, &vault).expect("restore vault for relearn");
        let numeric_problem_id = learning.entries[0]
            .problem_id
            .parse::<i64>()
            .expect("numeric problem id");
        sqlx::query("UPDATE problem_learning_states SET learning_status = 'relearning' WHERE problem_id = ?1")
            .bind(numeric_problem_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("relearning authoritative fixture");
        let third_day = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("third day");
        let relearn = load_or_generate_today_snapshot(&runtime, third_day, 60)
            .await
            .expect("relearn plan");
        assert_eq!(
            relearn.entries[0].reason,
            acm_os_domain::TodayCandidateReason::Relearn
        );
        let relearn_lifecycle = runtime
            .load_problem_lifecycle(&problem)
            .await
            .expect("relearn lifecycle");
        fs::rename(&vault, &offline).expect("make relearn note unavailable");
        let relearn_unavailable = load_or_generate_today_snapshot(&runtime, third_day, 1)
            .await
            .expect("Today detects unavailable relearn note");
        assert_eq!(
            relearn_unavailable.entries[0].status,
            TodayEntryStatus::Unavailable
        );
        fs::rename(&offline, &vault).expect("restore relearn note");
        let relearn_restored = load_or_generate_today_snapshot(&runtime, third_day, 1)
            .await
            .expect("Today restores relearn entry");
        assert_eq!(
            relearn_restored.entries[0].status,
            TodayEntryStatus::NotStarted
        );
        assert_eq!(
            runtime
                .load_problem_lifecycle(&problem)
                .await
                .expect("relearn lifecycle"),
            relearn_lifecycle
        );
    }

    #[tokio::test]
    async fn today_vault_availability_does_not_override_review_authority() {
        let (directory, runtime, vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("marked date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("review lifecycle");
        }
        let due = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due");
        let review_plan = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("review plan");
        assert_eq!(review_plan.entries[0].status, TodayEntryStatus::NotStarted);
        let offline = directory.path().join("vault-offline-review");
        fs::rename(&vault, &offline).expect("make vault unavailable");
        runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("project unavailable binding");
        let reconciled = load_or_generate_today_snapshot(&runtime, due, 30)
            .await
            .expect("review reconciliation");
        assert_eq!(reconciled, review_plan);
        assert_ne!(reconciled.entries[0].status, TodayEntryStatus::Unavailable);
    }

    #[tokio::test]
    async fn contest_import_is_progressive_idempotent_and_preserves_first_snapshot() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let draft = contest_draft();

        let initial = runtime
            .persist_manifest(&draft)
            .await
            .expect("persist manifest");
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
        let duplicate = runtime
            .persist_manifest(&draft)
            .await
            .expect("duplicate manifest");
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
        )
        .expect("asset problem");
        let assets = runtime
            .statement_assets(&asset_problem)
            .await
            .expect("read localized assets");
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
            assert!(first
                .windows_file_key
                .as_deref()
                .is_some_and(|key| key.starts_with("same-file-1:")));
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
    async fn learning_lifecycle_persists_first_due_and_survives_restart() {
        let (directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let today = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("local date");

        let initial = runtime
            .load_problem_lifecycle(&problem)
            .await
            .expect("initial lifecycle");
        assert_eq!(
            initial.learning_status,
            acm_os_domain::LearningStatus::Unstarted
        );
        assert!(initial.active_review_cycle.is_none());

        let pending = transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            today,
        )
        .await
        .expect("join upsolve");
        assert_eq!(
            pending.learning_status,
            acm_os_domain::LearningStatus::UpsolvePending
        );

        transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            today,
        )
        .await
        .expect("start learning");
        let waiting = transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
            today,
        )
        .await
        .expect("mark understood");
        assert_eq!(
            waiting.learning_status,
            acm_os_domain::LearningStatus::WaitingColdStart
        );
        let cycle = waiting.active_review_cycle.expect("active first cycle");
        assert_eq!(cycle.cycle_number, 1);
        assert_eq!(cycle.stage, 0);
        assert_eq!(cycle.schedule_rule_version, 1);
        assert_eq!(cycle.next_due_local_date.to_iso_string(), "2026-08-14");

        drop(runtime);
        let restarted = start_database(directory.path()).await;
        let restored = restarted
            .load_problem_lifecycle(&problem)
            .await
            .expect("restored lifecycle");
        assert_eq!(
            restored.learning_status,
            acm_os_domain::LearningStatus::WaitingColdStart
        );
        assert_eq!(
            restored
                .active_review_cycle
                .expect("restored active cycle")
                .next_due_local_date
                .to_iso_string(),
            "2026-08-14"
        );
    }

    #[tokio::test]
    async fn review_attempt_starts_once_resumes_and_exposes_only_focus_data() {
        let (_directory, runtime, _vault, problems, problem) = personal_note_fixture().await;
        let note_path = problems.join("CF-1979-A.md");
        fs::write(
            &note_path,
            "# Secret note\n\n## 题解\n\nDO NOT SEND THIS TO REVIEW\n",
        )
        .expect("external note edit");
        runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("refresh note binding");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("lifecycle transition");
        }

        let due = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due date");
        assert_eq!(
            start_or_resume_review(&runtime, &problem, due).await,
            Err(ReviewAttemptError::StatementMissing)
        );
        runtime
            .persist_first_snapshot(&snapshot(
                "A",
                "<div class=\"problem-statement\">A statement</div>",
                "<div class=\"problem-statement\">A statement</div>",
            ))
            .await
            .expect("statement snapshot");
        let first = start_or_resume_review(&runtime, &problem, due)
            .await
            .expect("start review");
        assert_eq!(
            first.attempt_type,
            acm_os_domain::ReviewAttemptType::FirstColdStart
        );
        assert!(!first.started_early);
        let resumed = start_or_resume_review(&runtime, &problem, due)
            .await
            .expect("resume review");
        assert_eq!(resumed.attempt_id, first.attempt_id);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_attempts WHERE attempt_status = 'in_progress'",
        )
        .fetch_one(runtime._pool.as_ref().expect("pool"))
        .await
        .expect("attempt count");
        assert_eq!(count, 1);

        let focus = review_focus(&runtime, &first.attempt_id)
            .await
            .expect("focus view");
        assert_eq!(focus.attempt.attempt_id, first.attempt_id);
        assert_eq!(focus.title, "Problem A");
        assert!(focus.statement_sanitized_html.contains("A"));
        assert!(!focus
            .statement_sanitized_html
            .contains("DO NOT SEND THIS TO REVIEW"));
        assert_eq!(
            transition_problem_lifecycle(
                &runtime,
                &problem,
                acm_os_domain::ProblemLifecycleAction::WithdrawUnderstood,
                due,
            )
            .await,
            Err(ProblemLifecycleError::InvalidTransition)
        );
        assert_eq!(
            delete_personal_note(&runtime, &problem).await,
            Err(PersonalNoteDeletionError::ReviewInProgress)
        );
        assert!(note_path.exists());
    }

    #[tokio::test]
    async fn help_reveal_commits_evidence_before_returning_fresh_content() {
        let (_directory, runtime, vault, problems, problem) = personal_note_fixture().await;
        let note_path = problems.join("CF-1979-A.md");
        fs::write(
            &note_path,
            "# P\n\n## 前置知识\n- [[Graphs#DFS|Traversal]]\n\n## Hints\n### Hint 1\nold hint\n\n## 思路\nold idea\n\n## 代码\n```cpp\nsolve();\n```\n\n## 题解\nfull answer\n",
        )
        .expect("write help fixture");
        fs::write(
            vault.join("Knowledge/Graphs.md"),
            "# Graphs\n\nDFS knowledge\n",
        )
        .expect("write knowledge fixture");
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("lifecycle transition");
        }
        let attempt = start_or_resume_review(
            &runtime,
            &problem,
            acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due"),
        )
        .await
        .expect("start review");

        let drawer = review_help_drawer(&runtime, &attempt.attempt_id)
            .await
            .expect("open drawer");
        assert!(drawer.items.iter().all(|item| item.available));
        let pool = runtime._pool.as_ref().expect("pool");
        let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM review_help_usage_events")
            .fetch_one(pool)
            .await
            .expect("event count");
        assert_eq!(before, 0, "opening the drawer is not usage");
        assert_eq!(
            reveal_review_help(
                &runtime,
                &attempt.attempt_id,
                acm_os_domain::ReviewHelpLevel::Hints,
                false,
            )
            .await,
            Err(ReviewAttemptError::HelpConfirmationRequired)
        );
        let after_refusal: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM review_help_usage_events")
                .fetch_one(pool)
                .await
                .expect("event count");
        assert_eq!(after_refusal, 0);

        fs::write(
            &note_path,
            "# P\n\n## 前置知识\n- [[Graphs#DFS|Traversal]]\n\n## Hints\n### Hint 1\nfresh external hint\n\n## 思路\nold idea\n\n## 代码\n```cpp\nsolve();\n```\n\n## 题解\nfull answer\n",
        )
        .expect("external edit before reveal");
        let hint = reveal_review_help(
            &runtime,
            &attempt.attempt_id,
            acm_os_domain::ReviewHelpLevel::Hints,
            true,
        )
        .await
        .expect("confirmed hint reveal");
        assert!(hint.content_markdown.contains("fresh external hint"));
        assert!(!hint.content_markdown.contains("full answer"));
        assert_eq!(hint.source_digest.len(), 64);
        let persisted: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_help_usage_events WHERE review_attempt_id = ?1",
        )
        .bind(&attempt.attempt_id)
        .fetch_one(pool)
        .await
        .expect("persisted evidence");
        assert_eq!(persisted, 1);

        let knowledge = reveal_review_help(
            &runtime,
            &attempt.attempt_id,
            acm_os_domain::ReviewHelpLevel::PrerequisiteContent,
            true,
        )
        .await
        .expect("knowledge reveal");
        assert!(knowledge.content_markdown.contains("DFS knowledge"));
        let reopened = reveal_review_help(
            &runtime,
            &attempt.attempt_id,
            acm_os_domain::ReviewHelpLevel::Hints,
            false,
        )
        .await
        .expect("already acknowledged level can reopen");
        assert!(reopened.content_markdown.contains("fresh external hint"));
        let final_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_help_usage_events WHERE review_attempt_id = ?1",
        )
        .bind(&attempt.attempt_id)
        .fetch_one(pool)
        .await
        .expect("append-only evidence");
        assert_eq!(final_count, 3);
    }

    #[tokio::test]
    async fn completed_reviews_advance_then_relearn_without_overwriting_history() {
        let (directory, runtime, _vault, _problems, problem, first_attempt) =
            review_ready_fixture().await;
        let first = complete_review(
            &runtime,
            &first_attempt.attempt_id,
            mastered_input(),
            acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("completed date"),
        )
        .await
        .expect("mastered first cold start");
        assert_eq!(first.judgement, acm_os_domain::ReviewJudgement::Mastered);
        assert_eq!(
            first.lifecycle.learning_status,
            acm_os_domain::LearningStatus::LongTermReview
        );
        let next_cycle = first
            .lifecycle
            .active_review_cycle
            .expect("continued cycle");
        assert_eq!(next_cycle.stage, 1);
        assert_eq!(next_cycle.next_due_local_date.to_iso_string(), "2026-08-24");

        let second_attempt = start_or_resume_review(
            &runtime,
            &problem,
            acm_os_domain::LocalDate::parse_iso("2026-08-24").expect("next due"),
        )
        .await
        .expect("long-term review");
        let mut partial_input = mastered_input();
        partial_input.external_help = acm_os_domain::ExternalHelpLevel::SolvingHint;
        partial_input.failure_reasons = vec![ReviewFailureReason::KeyPropertyBlocked];
        let second = complete_review(
            &runtime,
            &second_attempt.attempt_id,
            partial_input,
            acm_os_domain::LocalDate::parse_iso("2026-08-24").expect("completed date"),
        )
        .await
        .expect("partial long-term review");
        assert_eq!(second.judgement, acm_os_domain::ReviewJudgement::Partial);
        assert_eq!(
            second.lifecycle.learning_status,
            acm_os_domain::LearningStatus::Relearning
        );
        assert!(second.lifecycle.active_review_cycle.is_none());

        let history = review_history(&runtime, &problem)
            .await
            .expect("review history");
        assert_eq!(history.attempts.len(), 2);
        assert_eq!(
            history.historical_best_review,
            Some(acm_os_domain::ReviewJudgement::Mastered)
        );
        assert!(history.attempts.iter().any(|item| {
            item.attempt.attempt_id == first_attempt.attempt_id
                && item.judgement == Some(acm_os_domain::ReviewJudgement::Mastered)
        }));
        assert!(history.attempts.iter().any(|item| {
            item.attempt.attempt_id == second_attempt.attempt_id
                && item.judgement == Some(acm_os_domain::ReviewJudgement::Partial)
        }));
        drop(runtime);
        let restarted = start_database(directory.path()).await;
        let restored = review_history(&restarted, &problem)
            .await
            .expect("history after restart");
        assert_eq!(restored.attempts.len(), 2);
        assert_eq!(
            restored.historical_best_review,
            Some(acm_os_domain::ReviewJudgement::Mastered)
        );
    }

    #[tokio::test]
    async fn invalid_or_incomplete_facts_leave_attempt_in_progress() {
        let (_directory, runtime, _vault, _problems, _problem, attempt) =
            review_ready_fixture().await;
        let mut missing_reason = mastered_input();
        missing_reason.idea_independent = false;
        assert_eq!(
            complete_review(
                &runtime,
                &attempt.attempt_id,
                missing_reason,
                acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("date"),
            )
            .await,
            Err(ReviewAttemptError::FailureReasonRequired)
        );
        let mut contradiction = mastered_input();
        contradiction.final_ac = false;
        assert_eq!(
            complete_review(
                &runtime,
                &attempt.attempt_id,
                contradiction,
                acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("date"),
            )
            .await,
            Err(ReviewAttemptError::InvalidCompletionFacts)
        );
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM review_attempts WHERE id = ?1 AND attempt_status = 'in_progress'",
        )
        .bind(&attempt.attempt_id)
        .fetch_one(runtime._pool.as_ref().expect("pool"))
        .await
        .expect("active attempt");
        assert_eq!(active, 1);
        let mut no_ac = mastered_input();
        no_ac.final_ac = false;
        no_ac.first_submission.result = acm_os_domain::SubmissionResult::WrongAnswer;
        no_ac.final_submission.result = acm_os_domain::SubmissionResult::WrongAnswer;
        no_ac.total_submissions = 1;
        no_ac.failure_reasons = vec![ReviewFailureReason::ImplementationError];
        let failed = complete_review(
            &runtime,
            &attempt.attempt_id,
            no_ac,
            acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("date"),
        )
        .await
        .expect("no final AC completes as fail");
        assert_eq!(failed.judgement, acm_os_domain::ReviewJudgement::Fail);
        assert_eq!(
            failed.lifecycle.learning_status,
            acm_os_domain::LearningStatus::Relearning
        );
    }

    #[tokio::test]
    async fn full_solution_help_forces_fail_and_void_preserves_schedule_and_help_history() {
        let (_directory, runtime, _vault, problems, _problem, attempt) =
            review_ready_fixture().await;
        fs::write(
            problems.join("CF-1979-A.md"),
            "# P\n\n## 前置知识\n\n## 题解\ncomplete answer\n\n## 额外题目\n",
        )
        .expect("solution note");
        reveal_review_help(
            &runtime,
            &attempt.attempt_id,
            acm_os_domain::ReviewHelpLevel::FullSolution,
            true,
        )
        .await
        .expect("reveal solution");
        let mut failed_input = mastered_input();
        failed_input.failure_reasons = vec![ReviewFailureReason::NoIdea];
        let failed = complete_review(
            &runtime,
            &attempt.attempt_id,
            failed_input,
            acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("date"),
        )
        .await
        .expect("solution forces failure");
        assert_eq!(failed.judgement, acm_os_domain::ReviewJudgement::Fail);
        assert_eq!(
            failed.lifecycle.learning_status,
            acm_os_domain::LearningStatus::Relearning
        );

        let (_directory2, runtime2, _vault2, problems2, problem2, mistaken) =
            review_ready_fixture().await;
        fs::write(
            problems2.join("CF-1979-A.md"),
            "# P\n\n## Hints\n### H1\na hint\n\n## 题解\n\n",
        )
        .expect("hint note");
        reveal_review_help(
            &runtime2,
            &mistaken.attempt_id,
            acm_os_domain::ReviewHelpLevel::Hints,
            true,
        )
        .await
        .expect("mistaken help reveal");
        let before = runtime2
            .load_problem_lifecycle(&problem2)
            .await
            .expect("before void");
        let voided = void_review(&runtime2, &mistaken.attempt_id, "Opened the wrong problem")
            .await
            .expect("void mistaken attempt");
        assert_eq!(voided.status, ReviewAttemptStatus::Void);
        assert_eq!(voided.help_levels, [acm_os_domain::ReviewHelpLevel::Hints]);
        let after = runtime2
            .load_problem_lifecycle(&problem2)
            .await
            .expect("after void");
        assert_eq!(before, after);
        let replacement = start_or_resume_review(
            &runtime2,
            &problem2,
            acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("same due"),
        )
        .await
        .expect("replacement attempt");
        assert_ne!(replacement.attempt_id, mistaken.attempt_id);
        let history = review_history(&runtime2, &problem2)
            .await
            .expect("void history");
        assert_eq!(history.attempts.len(), 2);
        assert!(history
            .attempts
            .iter()
            .any(|item| item.status == ReviewAttemptStatus::Void));
    }

    #[tokio::test]
    async fn early_mastered_completion_keeps_original_stage_and_due() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot("A", "<p>A</p>", "<p>A</p>"))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("transition");
        }
        let early = start_or_resume_review(
            &runtime,
            &problem,
            acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("early"),
        )
        .await
        .expect("early attempt");
        let completed = complete_review(
            &runtime,
            &early.attempt_id,
            mastered_input(),
            acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("completed"),
        )
        .await
        .expect("early mastered");
        let cycle = completed
            .lifecycle
            .active_review_cycle
            .expect("unchanged cycle");
        assert_eq!(
            completed.lifecycle.learning_status,
            acm_os_domain::LearningStatus::WaitingColdStart
        );
        assert_eq!(cycle.stage, 0);
        assert_eq!(cycle.next_due_local_date.to_iso_string(), "2026-08-14");
    }

    #[tokio::test]
    async fn completed_review_and_historical_best_survive_personal_note_deletion() {
        let (_directory, runtime, _vault, problems, problem, attempt) =
            review_ready_fixture().await;
        complete_review(
            &runtime,
            &attempt.attempt_id,
            mastered_input(),
            acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("date"),
        )
        .await
        .expect("completed review");
        delete_personal_note(&runtime, &problem)
            .await
            .expect("delete after completion");
        assert!(!problems.join("CF-1979-A.md").exists());
        let history = review_history(&runtime, &problem)
            .await
            .expect("history after downgrade");
        assert_eq!(history.attempts.len(), 1);
        assert_eq!(
            history.historical_best_review,
            Some(acm_os_domain::ReviewJudgement::Mastered)
        );
        assert_eq!(history.attempts[0].status, ReviewAttemptStatus::Completed);
    }

    #[tokio::test]
    async fn early_review_attempt_preserves_the_original_cycle_due() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        runtime
            .persist_first_snapshot(&snapshot(
                "A",
                "<div class=\"problem-statement\">A statement</div>",
                "<div class=\"problem-statement\">A statement</div>",
            ))
            .await
            .expect("statement snapshot");
        let marked_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, marked_on)
                .await
                .expect("lifecycle transition");
        }

        let early = start_or_resume_review(
            &runtime,
            &problem,
            acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("early date"),
        )
        .await
        .expect("early review");
        assert_eq!(
            early.attempt_type,
            acm_os_domain::ReviewAttemptType::EarlyCheck
        );
        assert!(early.started_early);
        assert_eq!(early.scheduled_due_local_date.to_iso_string(), "2026-08-14");
        let lifecycle = runtime
            .load_problem_lifecycle(&problem)
            .await
            .expect("lifecycle preserved");
        assert_eq!(
            lifecycle.learning_status,
            acm_os_domain::LearningStatus::WaitingColdStart
        );
        assert_eq!(
            lifecycle
                .active_review_cycle
                .expect("active cycle")
                .next_due_local_date
                .to_iso_string(),
            "2026-08-14"
        );
    }

    #[tokio::test]
    async fn withdraw_and_stop_cancel_schedule_without_touching_personal_identity() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let today = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("local date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, today)
                .await
                .expect("lifecycle transition");
        }
        let learning = transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::WithdrawUnderstood,
            today,
        )
        .await
        .expect("withdraw understood");
        assert_eq!(
            learning.learning_status,
            acm_os_domain::LearningStatus::Learning
        );
        assert!(learning.active_review_cycle.is_none());

        let stopped = transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::StopLearning,
            today,
        )
        .await
        .expect("stop learning");
        assert_eq!(
            stopped.learning_status,
            acm_os_domain::LearningStatus::Unstarted
        );
        assert_eq!(stopped.identity_type, ProblemIdentityType::Personal);
        let counts: (i64, i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM contest_problems), \
                    (SELECT COUNT(*) FROM file_bindings), \
                    (SELECT COUNT(*) FROM review_cycles WHERE cycle_status = 'cancelled')",
        )
        .fetch_one(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("preserved facts");
        assert_eq!(counts, (2, 1, 1));
    }

    #[tokio::test]
    async fn lightweight_problem_cannot_enter_learning_lifecycle() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        runtime
            .persist_manifest(&contest_draft())
            .await
            .expect("persist manifest");
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");
        let result = transition_problem_lifecycle(
            &runtime,
            &problem,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("local date"),
        )
        .await;
        assert_eq!(result, Err(ProblemLifecycleError::NotPersonal));
    }

    #[tokio::test]
    async fn delete_personal_note_downgrades_problem_and_preserves_history_relations() {
        let (_directory, runtime, _vault, problems, problem) = personal_note_fixture().await;
        let note_path = problems.join("CF-1979-A.md");
        let user_markdown =
            b"# User-owned title\n\n## \xe9\xa2\x98\xe8\xa7\xa3\n\nMy durable explanation.\n";
        fs::write(&note_path, user_markdown).expect("external user edit");
        runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("refresh binding evidence");
        let today = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("local date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, today)
                .await
                .expect("lifecycle transition");
        }

        let deleted = delete_personal_note(&runtime, &problem)
            .await
            .expect("delete personal note");
        assert_eq!(deleted.identity_type, ProblemIdentityType::Lightweight);
        assert_eq!(
            deleted.learning_status,
            acm_os_domain::LearningStatus::Unstarted
        );
        assert!(deleted.active_review_cycle.is_none());
        assert!(!note_path.exists());

        let counts: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM contest_problems), \
                    (SELECT COUNT(*) FROM file_bindings), \
                    (SELECT COUNT(*) FROM review_cycles), \
                    (SELECT COUNT(*) FROM review_cycles WHERE cycle_status = 'cancelled')",
        )
        .fetch_one(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("preserved history counts");
        assert_eq!(counts, (2, 0, 1, 1));
        let recovery_files = files_under(
            &runtime
                .recovery_root
                .as_ref()
                .expect("recovery root")
                .join("deleted-personal-notes"),
        );
        assert_eq!(recovery_files.len(), 1);
        assert_eq!(
            fs::read(&recovery_files[0]).expect("recovery copy"),
            user_markdown
        );

        let recreated = create_personal_note(&runtime, &problem)
            .await
            .expect("recreate personal note after explicit deletion");
        assert_eq!(recreated.vault_relative_path, "Problems/CF-1979-A.md");
        assert_eq!(
            fs::read_to_string(note_path).expect("recreated note"),
            INITIAL_PROBLEM_MARKDOWN
        );
    }

    #[tokio::test]
    async fn delete_personal_note_refuses_vault_unavailable_without_downgrade() {
        let (_directory, runtime, vault, _problems, problem) = personal_note_fixture().await;
        fs::rename(&vault, vault.with_extension("offline")).expect("make vault unavailable");

        let result = delete_personal_note(&runtime, &problem).await;
        assert_eq!(result, Err(PersonalNoteDeletionError::VaultUnavailable));
        let detail = runtime
            .lightweight_problem_detail(&problem)
            .await
            .expect("system facts remain available");
        assert_eq!(detail.identity_type, ProblemIdentityType::Personal);
        assert_eq!(
            detail.lifecycle.learning_status,
            acm_os_domain::LearningStatus::Unstarted
        );
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
            projection: cached, ..
        } = cached
        else {
            panic!("initial projection must be ready");
        };
        let PersonalNoteReadState::Ready {
            projection: fresh, ..
        } = fresh
        else {
            panic!("fresh projection must be ready");
        };
        assert_ne!(fresh.content_digest, cached.content_digest);
        assert_eq!(fresh.solution_routes.len(), 1);
        assert_eq!(fresh.solution_routes[0].name, "External edit ×");
    }

    #[tokio::test]
    async fn safe_patch_updates_only_the_extra_problem_section_and_refreshes_binding_evidence() {
        let (directory, runtime, _vault, problems, problem) = personal_note_fixture().await;
        let note = problems.join("CF-1979-A.md");
        let before = "\u{feff}# Custom title\r\n\r\n## 前置知识\r\nkeep\r\n\r\n## 题解\r\n\r\n### Mine\r\nbody\r\n\r\n## 额外题目\r\n\r\n## User section\r\ndo not touch\r\n";
        fs::write(&note, before.as_bytes()).expect("custom note fixture");
        runtime
            .read_personal_note_projection(&problem)
            .await
            .expect("refresh external edit");

        let binding = add_extra_problem_link(&runtime, &problem, "CF-2000-A")
            .await
            .expect("safe semantic patch");
        let after = fs::read(&note).expect("patched note");
        let expected = before.replace(
            "## 额外题目\r\n\r\n## User section",
            "## 额外题目\r\n- [[CF-2000-A]]\r\n\r\n## User section",
        );
        assert_eq!(after, expected.as_bytes());
        assert_eq!(binding.content_digest, sha256_hex(&after));
        let persisted_digest: String = sqlx::query_scalar(
            "SELECT content_digest FROM file_bindings fb JOIN problems p ON p.id = fb.problem_id \
             WHERE p.external_contest_key = 1979 AND p.external_problem_key = 'A'",
        )
        .fetch_one(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("binding digest");
        assert_eq!(persisted_digest, binding.content_digest);

        let recovery_files = files_under(&directory.path().join("markdown-recovery"));
        assert_eq!(recovery_files.len(), 1);
        let recovery_name = recovery_files[0]
            .file_name()
            .expect("recovery filename")
            .to_string_lossy();
        assert!(recovery_name.contains(&sha256_hex(before.as_bytes())));
        assert!(recovery_name.contains(&binding.content_digest));
        assert_eq!(
            fs::read(&recovery_files[0]).expect("pre-write recovery"),
            before.as_bytes()
        );
    }

    #[tokio::test]
    async fn safe_patch_rejects_ambiguous_or_invalid_markdown_without_writing() {
        let (directory, runtime, _vault, problems, problem) = personal_note_fixture().await;
        let note = problems.join("CF-1979-A.md");
        let ambiguous = "## 额外题目\nfirst\n\n## 额外题目\nsecond\n";
        fs::write(&note, ambiguous).expect("ambiguous note");

        assert_eq!(
            add_extra_problem_link(&runtime, &problem, "CF-2000-A").await,
            Err(PersonalNotePatchError::TargetSectionAmbiguous)
        );
        assert_eq!(
            fs::read_to_string(&note).expect("unchanged note"),
            ambiguous
        );
        assert!(!directory.path().join("markdown-recovery").exists());

        fs::write(&note, [0xff, 0xfe, 0xfd]).expect("invalid utf-8 note");
        assert_eq!(
            add_extra_problem_link(&runtime, &problem, "CF-2000-B").await,
            Err(PersonalNotePatchError::InvalidUtf8)
        );
        assert_eq!(
            fs::read(&note).expect("invalid note preserved"),
            [0xff, 0xfe, 0xfd]
        );
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
        } = state
        else {
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
            binding, relocated, ..
        } = state
        else {
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
        assert!(matches!(
            state,
            PersonalNoteReadState::LocationAnomaly { .. }
        ));
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
        assert!(matches!(
            state,
            PersonalNoteReadState::VaultUnavailable { .. }
        ));
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
        let restored_state: String = sqlx::query_scalar("SELECT binding_state FROM file_bindings")
            .fetch_one(runtime._pool.as_ref().expect("ready database pool"))
            .await
            .expect("restored binding state");
        assert_eq!(restored_state, "linked");
    }

    #[tokio::test]
    async fn invalid_binding_path_never_reads_outside_the_vault() {
        let (directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        fs::write(directory.path().join("outside.md"), "outside secret").expect("outside fixture");
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
        runtime
            .persist_manifest(&contest_draft())
            .await
            .expect("persist manifest");
        runtime
            .persist_first_snapshot(&snapshot(
                "A",
                "<script>unsafe()</script><p>source only</p>",
                "<p>safe local statement</p>",
            ))
            .await
            .expect("persist snapshot");

        let contest = acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest");
        let contest_detail = runtime
            .contest_detail(&contest)
            .await
            .expect("contest detail");
        assert_eq!(contest_detail.problems.len(), 2);
        assert_eq!(contest_detail.problems[0].problem.index(), "A");
        let ready = runtime
            .lightweight_problem_detail(
                &acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "A")
                    .expect("problem A"),
            )
            .await
            .expect("ready problem detail");
        assert_eq!(ready.title, "Problem A");
        assert_eq!(
            ready.statement,
            StatementReadState::Ready {
                sanitized_html: "<p>safe local statement</p>".to_owned(),
            }
        );
        assert!(runtime
            .statement_assets(
                &acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "A")
                    .expect("problem A assets"),
            )
            .await
            .expect("statement assets")
            .is_empty());

        let pending = runtime
            .lightweight_problem_detail(
                &acm_os_domain::CodeforcesProblemIdentity::new(contest, "B").expect("problem B"),
            )
            .await
            .expect("pending problem detail");
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
        assert_eq!(
            runtime
                .list_contests()
                .await
                .expect("shelf after retry")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn reimport_rejects_manifest_drift_without_changing_the_first_manifest() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        runtime
            .persist_manifest(&contest_draft())
            .await
            .expect("first manifest");

        let contest = acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest");
        let drifted = ContestImportDraft::validated(
            contest.clone(),
            "Changed remote title".to_owned(),
            "https://codeforces.com/contest/1979".to_owned(),
            None,
            vec![ContestProblemSlotDraft {
                ordinal: 1,
                problem: acm_os_domain::CodeforcesProblemIdentity::new(contest, "A")
                    .expect("problem"),
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
            .fetch_one(pool)
            .await
            .expect("persisted slots");
        assert_eq!(persisted_count, 2);
    }

    #[tokio::test]
    async fn future_schema_is_blocked_without_running_migrations() {
        let directory = TempDir::new().expect("temporary app data");
        let database_path = directory.path().join(DATABASE_FILENAME);
        let pool = connect_read_write(&database_path)
            .await
            .expect("future database");
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

        let inspection = connect_read_only(&database_path)
            .await
            .expect("inspect future database");
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
        let pool = connect_read_write(&database_path)
            .await
            .expect("malformed database");
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
        fs::write(
            directory.path().join(DATABASE_FILENAME),
            b"not a sqlite database",
        )
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
        let backup_pool = connect_read_only(&backup_path)
            .await
            .expect("backup database");
        verify_integrity(&backup_pool)
            .await
            .expect("backup integrity");
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
                        CHECK (schema_generation < 16), \
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
    async fn inconsistent_today_budget_summary_requires_recovery() {
        let directory = TempDir::new().expect("temporary app data");
        {
            let runtime = start_database(directory.path()).await;
            let pool = runtime._pool.as_ref().expect("ready database pool");
            sqlx::query(
                "INSERT INTO today_plans \
                    (id, local_date, budget_minutes, planned_minutes, over_budget_minutes, review_only_streak) \
                 VALUES (?1, '2026-08-12', 0, 0, 1, 0)",
            )
            .bind(uuid::Uuid::now_v7().to_string())
            .execute(pool)
            .await
            .expect("insert inconsistent Today summary");
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
        let pool = connect_read_write(&database_path)
            .await
            .expect("unknown database");
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
        let pool = connect_read_write(&database_path)
            .await
            .expect("version zero database");
        create_empty_migration_ledger(&pool).await;
        pool.close().await;

        let runtime = start_database(directory.path()).await;
        assert_eq!(
            runtime.status(),
            &StartupGateStatus::Ready { schema_version: 15 }
        );

        let backup_directory = directory.path().join("backups").join("pre-migration");
        let backups: Vec<PathBuf> = fs::read_dir(backup_directory)
            .expect("pre-migration backup directory")
            .map(|entry| entry.expect("backup entry").path())
            .collect();
        assert_eq!(backups.len(), 1);
        let backup_pool = connect_read_only(&backups[0])
            .await
            .expect("version zero backup");
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
            &StartupGateStatus::Ready { schema_version: 15 }
        );
        let runtime_pool = runtime._pool.as_ref().expect("migrated database pool");
        let workspace_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspace_settings")
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
        let pool = connect_read_write(&database_path)
            .await
            .expect("version zero database");
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

        let inspection = connect_read_only(&database_path)
            .await
            .expect("inspect version zero");
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
    async fn thorough_digestion_history_survives_current_regression_and_note_deletion() {
        let (_directory, runtime, _vault, _problems, problem) = personal_note_fixture().await;
        let reached = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("date");
        let all_six = acm_os_domain::ProblemMasteryEvidence {
            recalls_problem: true,
            multiple_solutions_clear: true,
            knowledge_understood: true,
            implementation_fluent: true,
            can_adapt_or_create: true,
            transfer_solved_independently: true,
        };
        let achieved = update_problem_mastery_evidence(&runtime, &problem, all_six, reached)
            .await
            .expect("confirm six evidence criteria");
        assert!(achieved.current.is_thoroughly_digested());
        assert!(achieved.historical_thoroughly_digested);
        assert_eq!(achieved.first_thoroughly_digested_local_date, Some(reached));

        let regressed = update_problem_mastery_evidence(
            &runtime,
            &problem,
            acm_os_domain::ProblemMasteryEvidence {
                transfer_solved_independently: false,
                ..all_six
            },
            acm_os_domain::LocalDate::parse_iso("2026-08-20").expect("later date"),
        )
        .await
        .expect("confirm current regression");
        assert_eq!(regressed.current.achieved_count(), 5);
        assert!(regressed.historical_thoroughly_digested);
        assert_eq!(
            regressed.first_thoroughly_digested_local_date,
            Some(reached)
        );

        delete_personal_note(&runtime, &problem)
            .await
            .expect("delete personal note");
        let history = review_history(&runtime, &problem)
            .await
            .expect("history remains readable after lightweight downgrade");
        assert_eq!(history.mastery.current.achieved_count(), 5);
        assert!(history.mastery.historical_thoroughly_digested);
        assert_eq!(
            history.mastery.first_thoroughly_digested_local_date,
            Some(reached)
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
            &StartupGateStatus::Ready { schema_version: 15 }
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

    #[tokio::test]
    async fn today_extra_suggestion_is_read_only_and_manual_acceptance_persists() {
        let (directory, runtime, _vault, _problems, problem_a) = personal_note_fixture().await;
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        transition_problem_lifecycle(
            &runtime,
            &problem_a,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            day,
        )
        .await
        .expect("first candidate");
        let initial = load_or_generate_today_snapshot(&runtime, day, 120)
            .await
            .expect("initial plan");
        assert_eq!(initial.entries.len(), 1);
        let completed =
            complete_today_entry(&runtime, &initial.plan_id, &initial.entries[0].entry_id)
                .await
                .expect("complete initial work");

        let problem_b = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("problem B");
        create_personal_note(&runtime, &problem_b)
            .await
            .expect("personal B");
        transition_problem_lifecycle(
            &runtime,
            &problem_b,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            day,
        )
        .await
        .expect("second candidate");
        let lifecycle_before = runtime
            .load_problem_lifecycle(&problem_b)
            .await
            .expect("lifecycle before");
        let review_facts_before: (i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM review_cycles), \
                    (SELECT COUNT(*) FROM review_attempts)",
        )
        .fetch_one(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("review facts before");
        let rows_before: Vec<(String, String, i64, String)> = sqlx::query_as(
            "SELECT id, CAST(problem_id AS TEXT), position, entry_origin \
             FROM today_plan_entries ORDER BY position",
        )
        .fetch_all(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("rows before preview");

        let preview = preview_today_extra_suggestions(&runtime, day)
            .await
            .expect("extra suggestions");
        assert_eq!(preview.expected_snapshot, completed);
        assert_eq!(preview.remaining_budget_minutes, 60);
        assert_eq!(preview.suggestions.len(), 1);
        assert_eq!(
            preview.suggestions[0].reason,
            acm_os_domain::TodayCandidateReason::Upsolve
        );
        let rows_after_preview: Vec<(String, String, i64, String)> = sqlx::query_as(
            "SELECT id, CAST(problem_id AS TEXT), position, entry_origin \
             FROM today_plan_entries ORDER BY position",
        )
        .fetch_all(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("rows after preview");
        assert_eq!(rows_after_preview, rows_before);

        let accepted =
            accept_today_extra_suggestion(&runtime, &preview, &preview.suggestions[0].problem_id)
                .await
                .expect("accept explicit suggestion");
        assert_eq!(accepted.entries.len(), 2);
        assert_eq!(accepted.entries[0], completed.entries[0]);
        assert_eq!(accepted.entries[1].origin, TodayEntryOrigin::Manual);
        assert_eq!(accepted.entries[1].status, TodayEntryStatus::NotStarted);
        assert_eq!(accepted.planned_minutes, 120);
        assert_eq!(accepted.over_budget_minutes, 0);
        assert_eq!(
            runtime
                .load_problem_lifecycle(&problem_b)
                .await
                .expect("lifecycle after"),
            lifecycle_before
        );
        let review_facts_after: (i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM review_cycles), \
                    (SELECT COUNT(*) FROM review_attempts)",
        )
        .fetch_one(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("review facts after");
        assert_eq!(review_facts_after, review_facts_before);
        assert_eq!(
            load_or_generate_today_snapshot(&runtime, day, 1)
                .await
                .expect("same-day reopen"),
            accepted
        );
        drop(runtime);
        let restarted = start_database(directory.path()).await;
        assert_eq!(
            load_or_generate_today_snapshot(&restarted, day, 1)
                .await
                .expect("restart reopen"),
            accepted
        );
    }

    #[tokio::test]
    async fn today_extra_suggestion_rejects_illegal_or_stale_acceptance_without_writing() {
        let (_directory, runtime, _vault, _problems, problem_a) = personal_note_fixture().await;
        let day = acm_os_domain::LocalDate::parse_iso("2026-08-12").expect("day");
        transition_problem_lifecycle(
            &runtime,
            &problem_a,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            day,
        )
        .await
        .expect("first candidate");
        let initial = load_or_generate_today_snapshot(&runtime, day, 120)
            .await
            .expect("initial plan");
        assert!(preview_today_extra_suggestions(&runtime, day)
            .await
            .expect("unfinished preview")
            .suggestions
            .is_empty());
        complete_today_entry(&runtime, &initial.plan_id, &initial.entries[0].entry_id)
            .await
            .expect("complete initial work");
        let problem_b = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "B",
        )
        .expect("problem B");
        create_personal_note(&runtime, &problem_b)
            .await
            .expect("personal B");
        transition_problem_lifecycle(
            &runtime,
            &problem_b,
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            day,
        )
        .await
        .expect("second candidate");
        sqlx::query("UPDATE today_plans SET budget_minutes = 90 WHERE id = ?1")
            .bind(&initial.plan_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("leave only thirty minutes");
        assert!(preview_today_extra_suggestions(&runtime, day)
            .await
            .expect("insufficient-budget preview")
            .suggestions
            .is_empty());
        sqlx::query("UPDATE today_plans SET budget_minutes = 120 WHERE id = ?1")
            .bind(&initial.plan_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("restore suggestion budget");
        let preview = preview_today_extra_suggestions(&runtime, day)
            .await
            .expect("valid preview");
        let unchanged = runtime
            .load_today_snapshot(day)
            .await
            .expect("load")
            .expect("plan");

        sqlx::query("UPDATE today_plans SET budget_minutes = 90 WHERE id = ?1")
            .bind(&initial.plan_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("make preview unaffordable");
        let budget_changed = runtime
            .load_today_snapshot(day)
            .await
            .expect("load")
            .expect("plan");
        assert_eq!(
            accept_today_extra_suggestion(&runtime, &preview, &preview.suggestions[0].problem_id)
                .await,
            Err(TodaySnapshotError::StaleExtraSuggestions)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(day)
                .await
                .expect("load")
                .expect("plan"),
            budget_changed
        );
        sqlx::query("UPDATE today_plans SET budget_minutes = 120 WHERE id = ?1")
            .bind(&initial.plan_id)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("restore valid preview version");
        assert_eq!(
            accept_today_extra_suggestion(&runtime, &preview, "unknown").await,
            Err(TodaySnapshotError::InvalidExtraSuggestion)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(day)
                .await
                .expect("load")
                .expect("plan"),
            unchanged
        );

        let numeric_b = preview.suggestions[0]
            .problem_id
            .parse::<i64>()
            .expect("numeric B");
        sqlx::query(
            "UPDATE file_bindings SET binding_state = 'location_anomaly' WHERE problem_id = ?1",
        )
        .bind(numeric_b)
        .execute(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("invalidate candidate");
        assert_eq!(
            accept_today_extra_suggestion(&runtime, &preview, &preview.suggestions[0].problem_id)
                .await,
            Err(TodaySnapshotError::InvalidExtraSuggestion)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(day)
                .await
                .expect("load")
                .expect("plan"),
            unchanged
        );
        sqlx::query("UPDATE file_bindings SET binding_state = 'linked' WHERE problem_id = ?1")
            .bind(numeric_b)
            .execute(runtime._pool.as_ref().expect("ready pool"))
            .await
            .expect("restore candidate");

        let accepted =
            accept_today_extra_suggestion(&runtime, &preview, &preview.suggestions[0].problem_id)
                .await
                .expect("accept once");
        assert_eq!(
            accept_today_extra_suggestion(&runtime, &preview, &preview.suggestions[0].problem_id)
                .await,
            Err(TodaySnapshotError::StaleExtraSuggestions)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(day)
                .await
                .expect("load")
                .expect("plan"),
            accepted
        );

        let next_day = acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("next day");
        let next_plan = load_or_generate_today_snapshot(&runtime, next_day, 0)
            .await
            .expect("cross-plan fixture");
        let mut cross_plan = preview.clone();
        cross_plan.expected_snapshot.plan_id = next_plan.plan_id;
        assert_eq!(
            accept_today_extra_suggestion(
                &runtime,
                &cross_plan,
                &preview.suggestions[0].problem_id
            )
            .await,
            Err(TodaySnapshotError::StaleExtraSuggestions)
        );
        assert_eq!(
            runtime
                .load_today_snapshot(day)
                .await
                .expect("load")
                .expect("plan"),
            accepted
        );
    }

    #[tokio::test]
    async fn m5_core_loop_imports_markdown_learns_reviews_and_recalls_in_today() {
        let directory = TempDir::new().expect("temporary app data");
        let runtime = start_database(directory.path()).await;
        let (_vault, _problems, _knowledge) =
            configure_temporary_workspace(&runtime, &directory).await;
        let manifest = contest_draft();
        let problem = manifest.slots[0].problem.clone();
        let source = CoreLoopContestSource {
            manifest,
            snapshots: vec![
                snapshot(
                    "A",
                    "<div class=\"problem-statement\">A</div>",
                    "<div class=\"problem-statement\">A</div>",
                ),
                snapshot(
                    "B",
                    "<div class=\"problem-statement\">B</div>",
                    "<div class=\"problem-statement\">B</div>",
                ),
            ],
        };
        let imported = import_codeforces_contest(
            &runtime,
            &source,
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
        )
        .await
        .expect("fixture contest import through application boundary");
        assert_eq!(imported.persisted.status, ContestImportStatus::Complete);
        assert!(imported.failed_snapshot_problems.is_empty());

        let binding = create_personal_note(&runtime, &problem)
            .await
            .expect("real personal Markdown creation");
        assert!(directory
            .path()
            .join("vault")
            .join(binding.vault_relative_path)
            .is_file());
        let learned_on = acm_os_domain::LocalDate::parse_iso("2026-08-11").expect("learned date");
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            transition_problem_lifecycle(&runtime, &problem, action, learned_on)
                .await
                .expect("learning transition");
        }
        let due = acm_os_domain::ReviewSchedulingEngine::first_cold_start_due(learned_on)
            .expect("first due");
        let attempt = start_or_resume_review(&runtime, &problem, due)
            .await
            .expect("real controlled Review Attempt");
        let completed = complete_review(&runtime, &attempt.attempt_id, mastered_input(), due)
            .await
            .expect("fact-derived Review completion");
        assert_eq!(
            completed.judgement,
            acm_os_domain::ReviewJudgement::Mastered
        );
        let next_due = completed
            .lifecycle
            .active_review_cycle
            .expect("long-term schedule")
            .next_due_local_date;
        assert!(next_due > due);

        let today = load_or_generate_today_snapshot(&runtime, next_due, 30)
            .await
            .expect("later Today recall");
        let numeric_problem_id: i64 = sqlx::query_scalar(
            "SELECT id FROM problems WHERE platform = 'codeforces' \
             AND external_contest_key = ?1 AND external_problem_key = ?2",
        )
        .bind(problem.contest().contest_id() as i64)
        .bind(problem.index())
        .fetch_one(runtime._pool.as_ref().expect("ready pool"))
        .await
        .expect("problem id");
        assert_eq!(today.entries.len(), 1);
        assert_eq!(today.entries[0].problem_id, numeric_problem_id.to_string());
        assert_eq!(
            today.entries[0].lane,
            acm_os_domain::TodayCandidateLane::Review
        );
        assert_eq!(
            today.entries[0].reason,
            acm_os_domain::TodayCandidateReason::DueLongTermReview
        );
        assert_eq!(today.entries[0].status, TodayEntryStatus::NotStarted);
    }
}
