use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use acm_os_application::{
    StartupGateStatus, StartupRecoveryReason, WorkspaceConfiguration,
    WorkspaceConfigurationPort, WorkspacePathResolutionError, WorkspacePersistenceError,
};
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::SqlitePool;

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
}

impl DatabaseRuntime {
    pub fn recovery(reason: StartupRecoveryReason) -> Self {
        Self {
            _pool: None,
            _startup_lock: None,
            status: StartupGateStatus::RecoveryRequired { reason },
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

    if !matches!(schema_version, 1 | 2) {
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

    Ok(())
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
        configure_workspace, query_workspace_configuration, StartupGateStatus,
        StartupRecoveryReason, WorkspaceConfigurationDraft, WorkspaceConfigurationError,
        WorkspaceConfigurationStatus, WorkspacePathField,
    };
    use sqlx::Executor;
    use tempfile::TempDir;

    use super::*;

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
            &StartupGateStatus::Ready { schema_version: 2 }
        );
        let pool = runtime._pool.as_ref().expect("ready database pool");
        let ledger_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(pool)
            .await
            .expect("migration ledger");
        assert_eq!(ledger_count, 2);
        verify_integrity(pool).await.expect("database integrity");
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
                        CHECK (schema_generation < 3), \
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
            &StartupGateStatus::Ready { schema_version: 2 }
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
    async fn version_one_database_is_backed_up_then_migrated_to_version_two() {
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
            &StartupGateStatus::Ready { schema_version: 2 }
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
            &StartupGateStatus::Ready { schema_version: 2 }
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
