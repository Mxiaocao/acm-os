//! The desktop's sole owner of the active data space and command leases.
use crate::{start_database, DatabaseRuntime};
use acm_os_application::{
    StartupGateStatus, WorkspaceConfigurationDraft, WorkspaceConfigurationPort,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
};
use tokio::sync::{RwLock, RwLockReadGuard};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VaultSummary {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub database_path: String,
    pub created_at_utc: String,
    pub is_current: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VaultContext {
    pub vault_id: String,
    pub epoch: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CurrentVaultState {
    None,
    Ready {
        vault: VaultSummary,
    },
    Unavailable {
        vault: VaultSummary,
        reason: VaultErrorCode,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerStatus {
    pub vaults: Vec<VaultSummary>,
    pub current: CurrentVaultState,
    pub context: Option<VaultContext>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateVaultRequest {
    pub name: String,
    pub root_path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VaultRequest {
    pub vault_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteVaultRequest {
    pub vault_id: String,
    pub token: String,
    pub confirmation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePreview {
    pub vault_id: String,
    pub name: String,
    pub root_path: String,
    pub retained_data_path: String,
    pub confirmation_text: String,
    pub token: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VaultErrorCode {
    InvalidName,
    RootMissing,
    RootNested,
    DuplicateRoot,
    LegacyProtected,
    ExistingData,
    VaultNotFound,
    VaultUnavailable,
    CurrentVaultDeleteForbidden,
    ConfirmationMismatch,
    InvalidConfirmationToken,
    StaleVaultContext,
    NoActiveVault,
    RegistryUnavailable,
    RegistryCorrupt,
    ManagerInUse,
    PersistenceUnavailable,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultError {
    pub code: VaultErrorCode,
}
impl From<VaultErrorCode> for VaultError {
    fn from(code: VaultErrorCode) -> Self {
        Self { code }
    }
}
impl VaultError {
    pub fn code(&self) -> &'static str {
        use VaultErrorCode::*;
        match self.code {
            InvalidName => "invalid_name",
            RootMissing => "root_missing",
            RootNested => "root_nested",
            DuplicateRoot => "duplicate_root",
            LegacyProtected => "legacy_protected",
            ExistingData => "existing_data",
            VaultNotFound => "vault_not_found",
            VaultUnavailable => "vault_unavailable",
            CurrentVaultDeleteForbidden => "current_vault_delete_forbidden",
            ConfirmationMismatch => "confirmation_mismatch",
            InvalidConfirmationToken => "invalid_confirmation_token",
            StaleVaultContext => "stale_vault_context",
            NoActiveVault => "no_active_vault",
            RegistryUnavailable => "registry_unavailable",
            RegistryCorrupt => "registry_corrupt",
            ManagerInUse => "manager_in_use",
            PersistenceUnavailable => "persistence_unavailable",
        }
    }
}
type Result<T> = std::result::Result<T, VaultError>;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Registry {
    vaults: Vec<VaultSummary>,
    current_id: Option<String>,
}
struct Session {
    registry: Registry,
    runtime: Option<DatabaseRuntime>,
    epoch: String,
    deletion: Option<(DeletePreview, String)>,
}
pub struct VaultManager {
    app_data: PathBuf,
    legacy_root: Option<PathBuf>,
    _lock: File,
    session: RwLock<Session>,
}
pub struct VaultLease<'a> {
    session: RwLockReadGuard<'a, Session>,
}
impl VaultLease<'_> {
    pub fn inner(&self) -> &DatabaseRuntime {
        self.session
            .runtime
            .as_ref()
            .expect("lease requires runtime")
    }
    pub fn root_path(&self) -> &str {
        &self
            .session
            .registry
            .vaults
            .iter()
            .find(|v| Some(&v.id) == self.session.registry.current_id.as_ref())
            .expect("validated current")
            .root_path
    }
}
impl Deref for VaultLease<'_> {
    type Target = DatabaseRuntime;
    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl VaultManager {
    pub async fn open(app_data: &Path) -> Result<Self> {
        fs::create_dir_all(app_data).map_err(|_| VaultErrorCode::RegistryUnavailable)?;
        let app_data =
            fs::canonicalize(app_data).map_err(|_| VaultErrorCode::RegistryUnavailable)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(app_data.join(".vault-manager.lock"))
            .map_err(|_| VaultErrorCode::RegistryUnavailable)?;
        lock.try_lock().map_err(|_| VaultErrorCode::ManagerInUse)?;
        let registry: Registry = match fs::read(app_data.join("vault-manager.json")) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|_| VaultErrorCode::RegistryCorrupt)?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Registry::default(),
            Err(_) => return Err(VaultErrorCode::RegistryUnavailable.into()),
        };
        let mut ids = std::collections::HashSet::new();
        for v in &registry.vaults {
            if Uuid::parse_str(&v.id).is_err()
                || !ids.insert(&v.id)
                || !Path::new(&v.root_path).is_absolute()
                || Path::new(&v.database_path)
                    != Path::new(&v.root_path).join(".acm-os/system-facts.sqlite3")
            {
                return Err(VaultErrorCode::RegistryCorrupt.into());
            }
        }
        if registry
            .current_id
            .as_ref()
            .is_some_and(|id| !ids.contains(id))
        {
            return Err(VaultErrorCode::RegistryCorrupt.into());
        }
        let legacy = app_data.join("system-facts.sqlite3");
        let legacy_root = if legacy.exists() {
            use sqlx::Connection;
            let options = sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&legacy)
                .read_only(true);
            let mut conn = sqlx::SqliteConnection::connect_with(&options)
                .await
                .map_err(|_| VaultErrorCode::LegacyProtected)?;
            let has_settings: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM sqlite_master WHERE name='workspace_settings'",
            )
            .fetch_one(&mut conn)
            .await
            .map_err(|_| VaultErrorCode::LegacyProtected)?;
            let root: Option<String> = if has_settings > 0 {
                sqlx::query_scalar(
                    "SELECT active_vault_path FROM workspace_settings WHERE singleton=1",
                )
                .fetch_optional(&mut conn)
                .await
                .map_err(|_| VaultErrorCode::LegacyProtected)?
            } else {
                None
            };
            conn.close()
                .await
                .map_err(|_| VaultErrorCode::LegacyProtected)?;
            root.map(PathBuf::from)
        } else {
            None
        };
        let manager = Self {
            app_data,
            legacy_root,
            _lock: lock,
            session: RwLock::new(Session {
                registry,
                runtime: None,
                epoch: Uuid::now_v7().to_string(),
                deletion: None,
            }),
        };
        {
            let mut session = manager.session.write().await;
            if let Some(v) = session
                .registry
                .vaults
                .iter()
                .find(|v| Some(&v.id) == session.registry.current_id.as_ref())
                .cloned()
            {
                if manager.validate(&v).is_ok() {
                    session.runtime = Self::start(&v).await.ok();
                }
            }
        }
        Ok(manager)
    }
    fn save(&self, registry: &Registry) -> Result<()> {
        let mut tmp = tempfile::NamedTempFile::new_in(&self.app_data)
            .map_err(|_| VaultErrorCode::RegistryUnavailable)?;
        tmp.write_all(
            &serde_json::to_vec_pretty(registry)
                .map_err(|_| VaultErrorCode::RegistryUnavailable)?,
        )
        .map_err(|_| VaultErrorCode::RegistryUnavailable)?;
        tmp.as_file()
            .sync_all()
            .map_err(|_| VaultErrorCode::RegistryUnavailable)?;
        tmp.persist(self.app_data.join("vault-manager.json"))
            .map_err(|_| VaultErrorCode::RegistryUnavailable)?;
        Ok(())
    }
    fn protected(&self, root: &Path) -> bool {
        overlaps(root, &self.app_data)
            || self.legacy_root.as_ref().is_some_and(|legacy| {
                overlaps(
                    root,
                    &fs::canonicalize(legacy).unwrap_or_else(|_| legacy.clone()),
                )
            })
    }
    fn validate(&self, v: &VaultSummary) -> Result<()> {
        let root = Path::new(&v.root_path);
        let owned = root.join(".acm-os");
        let db = owned.join("system-facts.sqlite3");
        if self.protected(root)
            || fs::canonicalize(root).ok().as_deref() != Some(root)
            || !root.is_dir()
            || fs::canonicalize(&owned).ok().as_deref() != Some(owned.as_path())
            || fs::canonicalize(&db).ok().as_deref() != Some(db.as_path())
            || !db.is_file()
            || Path::new(&v.database_path) != db
            || fs::read_to_string(owned.join("vault-id")).ok().as_deref() != Some(v.id.as_str())
        {
            return Err(VaultErrorCode::VaultUnavailable.into());
        }
        // Reject aliasing Legacy even if a hard link was placed in the owned directory.
        let legacy = self.app_data.join("system-facts.sqlite3");
        if legacy.exists() && same_file::is_same_file(&db, &legacy).unwrap_or(true) {
            return Err(VaultErrorCode::LegacyProtected.into());
        }
        Ok(())
    }
    async fn start(v: &VaultSummary) -> Result<DatabaseRuntime> {
        // Inspect identity read-only before migrations/recovery may write to the target.
        // A copied or hard-linked database from another Vault must never be started.
        use sqlx::Connection;
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&v.database_path)
            .read_only(true);
        let mut inspection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .map_err(|_| VaultErrorCode::VaultUnavailable)?;
        let roots:Option<(String,String,String)>=sqlx::query_as("SELECT active_vault_path, problem_root_path, knowledge_root_path FROM workspace_settings WHERE singleton=1")
      .fetch_optional(&mut inspection).await.map_err(|_|VaultErrorCode::VaultUnavailable)?;
        inspection
            .close()
            .await
            .map_err(|_| VaultErrorCode::VaultUnavailable)?;
        let (root, problems, knowledge) = roots.ok_or(VaultErrorCode::VaultUnavailable)?;
        if root != v.root_path
            || !Path::new(&problems).starts_with(&root)
            || !Path::new(&knowledge).starts_with(&root)
        {
            return Err(VaultErrorCode::VaultUnavailable.into());
        }

        let runtime = start_database(
            Path::new(&v.database_path)
                .parent()
                .ok_or(VaultErrorCode::VaultUnavailable)?,
        )
        .await;
        if !matches!(runtime.status(), StartupGateStatus::Ready { .. }) {
            runtime.close().await;
            return Err(VaultErrorCode::PersistenceUnavailable.into());
        }
        let configuration = runtime
            .load_workspace_configuration()
            .await
            .map_err(|_| VaultErrorCode::PersistenceUnavailable)?;
        if configuration.as_ref().map(|c| c.active_vault_path()) != Some(v.root_path.as_str()) {
            runtime.close().await;
            return Err(VaultErrorCode::VaultUnavailable.into());
        }
        let _ = runtime.process_pending_review_rewards(256).await;
        Ok(runtime)
    }
    fn status_locked(&self, s: &Session) -> ManagerStatus {
        let vaults: Vec<_> = s
            .registry
            .vaults
            .iter()
            .cloned()
            .map(|mut v| {
                v.is_current = Some(&v.id) == s.registry.current_id.as_ref();
                v
            })
            .collect();
        let current = match vaults.iter().find(|v| v.is_current) {
            None => CurrentVaultState::None,
            Some(v) if s.runtime.is_some() && self.validate(v).is_ok() => {
                CurrentVaultState::Ready { vault: v.clone() }
            }
            Some(v) => CurrentVaultState::Unavailable {
                vault: v.clone(),
                reason: VaultErrorCode::VaultUnavailable,
            },
        };
        let context = if matches!(current, CurrentVaultState::Ready { .. }) {
            s.registry.current_id.clone().map(|vault_id| VaultContext {
                vault_id,
                epoch: s.epoch.clone(),
            })
        } else {
            None
        };
        ManagerStatus {
            vaults,
            current,
            context,
        }
    }
    pub async fn status(&self) -> ManagerStatus {
        self.status_locked(&*self.session.read().await)
    }
    pub async fn database(&self, context: &VaultContext) -> Result<VaultLease<'_>> {
        let s = self.session.read().await;
        if s.registry.current_id.as_ref() != Some(&context.vault_id) || s.epoch != context.epoch {
            return Err(VaultErrorCode::StaleVaultContext.into());
        }
        let v = s
            .registry
            .vaults
            .iter()
            .find(|v| v.id == context.vault_id)
            .ok_or(VaultErrorCode::NoActiveVault)?;
        self.validate(v)?;
        if s.runtime.is_none() {
            return Err(VaultErrorCode::VaultUnavailable.into());
        }
        Ok(VaultLease { session: s })
    }
    pub async fn create(&self, input: CreateVaultRequest) -> Result<VaultSummary> {
        let mut s = self.session.write().await;
        let name = input.name.trim();
        if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
            return Err(VaultErrorCode::InvalidName.into());
        }
        let root = fs::canonicalize(&input.root_path).map_err(|_| VaultErrorCode::RootMissing)?;
        if !root.is_dir() {
            return Err(VaultErrorCode::RootMissing.into());
        }
        if self.protected(&root) {
            return Err(VaultErrorCode::LegacyProtected.into());
        }
        for v in &s.registry.vaults {
            if Path::new(&v.root_path) == root {
                return Err(VaultErrorCode::DuplicateRoot.into());
            }
            if overlaps(&root, Path::new(&v.root_path)) {
                return Err(VaultErrorCode::RootNested.into());
            }
        }
        let owned = root.join(".acm-os");
        fs::create_dir(&owned).map_err(|_| VaultErrorCode::ExistingData)?;
        let id = Uuid::now_v7().to_string();
        let v = VaultSummary {
            id: id.clone(),
            name: name.into(),
            root_path: root.to_string_lossy().into_owned(),
            database_path: owned
                .join("system-facts.sqlite3")
                .to_string_lossy()
                .into_owned(),
            created_at_utc: chrono::Utc::now().to_rfc3339(),
            is_current: false,
        };
        fs::write(owned.join("vault-id"), &id)
            .map_err(|_| VaultErrorCode::PersistenceUnavailable)?;
        for folder in ["Problems", "Knowledge"] {
            let path = root.join(folder);
            if !path.exists() {
                fs::create_dir(&path).map_err(|_| VaultErrorCode::PersistenceUnavailable)?;
            }
        }
        let runtime = start_database(&owned).await;
        if !matches!(runtime.status(), StartupGateStatus::Ready { .. }) {
            runtime.close().await;
            return Err(VaultErrorCode::PersistenceUnavailable.into());
        }
        let configured = acm_os_application::configure_workspace(
            &runtime,
            WorkspaceConfigurationDraft {
                active_vault_path: v.root_path.clone(),
                problem_root_path: root.join("Problems").to_string_lossy().into_owned(),
                knowledge_root_path: root.join("Knowledge").to_string_lossy().into_owned(),
            },
        )
        .await;
        runtime.close().await;
        drop(runtime);
        configured.map_err(|_| VaultErrorCode::PersistenceUnavailable)?;
        self.validate(&v)?;
        let mut next = s.registry.clone();
        next.vaults.push(v.clone());
        self.save(&next)?;
        s.registry = next;
        Ok(v)
    }
    pub async fn open_vault(&self, input: VaultRequest) -> Result<ManagerStatus> {
        self.open_vault_observed(input, |_| {}).await
    }
    pub async fn open_vault_observed(
        &self,
        input: VaultRequest,
        on_open: impl FnOnce(&ManagerStatus),
    ) -> Result<ManagerStatus> {
        let mut s = self.session.write().await;
        let v = s
            .registry
            .vaults
            .iter()
            .find(|v| v.id == input.vault_id)
            .cloned()
            .ok_or(VaultErrorCode::VaultNotFound)?;
        self.validate(&v)?;
        if s.registry.current_id.as_ref() == Some(&v.id) && s.runtime.is_some() {
            let status = self.status_locked(&s);
            on_open(&status);
            return Ok(status);
        }
        let runtime = Self::start(&v).await?;
        let mut next = s.registry.clone();
        next.current_id = Some(v.id.clone());
        if let Err(e) = self.save(&next) {
            runtime.close().await;
            return Err(e);
        }
        if let Some(old) = s.runtime.take() {
            old.close().await;
            drop(old)
        }
        s.runtime = Some(runtime);
        s.registry = next;
        s.epoch = Uuid::now_v7().to_string();
        s.deletion = None;
        let status = self.status_locked(&s);
        on_open(&status);
        Ok(status)
    }
    pub async fn preview_delete(&self, input: VaultRequest) -> Result<DeletePreview> {
        let mut s = self.session.write().await;
        if s.registry.current_id.as_ref() == Some(&input.vault_id) {
            return Err(VaultErrorCode::CurrentVaultDeleteForbidden.into());
        }
        let v = s
            .registry
            .vaults
            .iter()
            .find(|v| v.id == input.vault_id)
            .ok_or(VaultErrorCode::VaultNotFound)?;
        self.validate(v)?;
        let p = DeletePreview {
            vault_id: v.id.clone(),
            name: v.name.clone(),
            root_path: v.root_path.clone(),
            retained_data_path: Path::new(&v.root_path)
                .join(format!(".acm-os-deleted-{}", v.id))
                .to_string_lossy()
                .into_owned(),
            confirmation_text: format!("DELETE {}", v.name),
            token: Uuid::now_v7().to_string(),
        };
        s.deletion = Some((p.clone(), s.epoch.clone()));
        Ok(p)
    }
    pub async fn delete(&self, input: DeleteVaultRequest) -> Result<ManagerStatus> {
        let mut s = self.session.write().await;
        if s.registry.current_id.as_ref() == Some(&input.vault_id) {
            return Err(VaultErrorCode::CurrentVaultDeleteForbidden.into());
        }
        let (preview, epoch) = s
            .deletion
            .take()
            .ok_or(VaultErrorCode::InvalidConfirmationToken)?;
        if epoch != s.epoch || input.vault_id != preview.vault_id || input.token != preview.token {
            return Err(VaultErrorCode::InvalidConfirmationToken.into());
        }
        if input.confirmation != preview.confirmation_text {
            return Err(VaultErrorCode::ConfirmationMismatch.into());
        }
        let v = s
            .registry
            .vaults
            .iter()
            .find(|v| v.id == input.vault_id)
            .ok_or(VaultErrorCode::VaultNotFound)?;
        self.validate(v)?;
        let owned = Path::new(&v.root_path).join(".acm-os");
        let retained = Path::new(&preview.retained_data_path);
        if retained.exists() {
            return Err(VaultErrorCode::ExistingData.into());
        }
        // Acquire the same OS lock as the persistence runtime before renaming owned data.
        let deletion_lock = OpenOptions::new()
            .write(true)
            .open(owned.join(".database-startup.lock"))
            .map_err(|_| VaultErrorCode::VaultUnavailable)?;
        deletion_lock
            .try_lock()
            .map_err(|_| VaultErrorCode::ManagerInUse)?;
        let mut next = s.registry.clone();
        next.vaults.retain(|v| v.id != input.vault_id);
        drop(deletion_lock); // The manager process lock still serializes registered lifecycle operations.
        fs::rename(&owned, retained).map_err(|_| VaultErrorCode::PersistenceUnavailable)?;
        if let Err(e) = self.save(&next) {
            fs::rename(retained, &owned).map_err(|_| VaultErrorCode::RegistryUnavailable)?;
            return Err(e);
        }
        s.registry = next;
        Ok(self.status_locked(&s))
    }
    pub async fn shutdown(&self) {
        let mut s = self.session.write().await;
        if let Some(runtime) = s.runtime.take() {
            runtime.close().await;
        }
    }
}
fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

#[cfg(test)]
mod tests;
