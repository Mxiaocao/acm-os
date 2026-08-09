#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundationStatusDto {
    status: &'static str,
    core: &'static str,
}

#[tauri::command]
pub fn foundation_status() -> FoundationStatusDto {
    let result = acm_os_application::foundation_status();

    FoundationStatusDto {
        status: result.status,
        core: result.core,
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatusDto {
    state: &'static str,
    schema_version: Option<i64>,
    recovery_reason: Option<&'static str>,
    supported_schema_version: Option<i64>,
    found_schema_version: Option<i64>,
}

#[tauri::command]
pub fn startup_status(
    startup: tauri::State<'_, acm_os_application::StartupStatusQuery>,
) -> StartupStatusDto {
    startup_status_dto(startup.execute())
}

fn startup_status_dto(status: &acm_os_application::StartupGateStatus) -> StartupStatusDto {
    use acm_os_application::{StartupGateStatus, StartupRecoveryReason};

    match status {
        StartupGateStatus::Ready { schema_version } => StartupStatusDto {
            state: "ready",
            schema_version: Some(*schema_version),
            recovery_reason: None,
            supported_schema_version: Some(*schema_version),
            found_schema_version: None,
        },
        StartupGateStatus::RecoveryRequired { reason } => {
            let (supported_schema_version, found_schema_version) = match reason {
                StartupRecoveryReason::UnsupportedSchema { found, supported } => {
                    (Some(*supported), Some(*found))
                }
                _ => (None, None),
            };

            StartupStatusDto {
                state: "recoveryRequired",
                schema_version: None,
                recovery_reason: Some(reason.code()),
                supported_schema_version,
                found_schema_version,
            }
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusDto {
    state: &'static str,
    active_vault_path: Option<String>,
    problem_root_path: Option<String>,
    knowledge_root_path: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfigurationInput {
    active_vault_path: String,
    problem_root_path: String,
    knowledge_root_path: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfigurationErrorDto {
    code: &'static str,
    field: Option<&'static str>,
}

#[tauri::command]
pub async fn workspace_status(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<WorkspaceStatusDto, WorkspaceConfigurationErrorDto> {
    acm_os_application::query_workspace_configuration(database.inner())
        .await
        .map(workspace_status_dto)
        .map_err(workspace_error_dto)
}

#[tauri::command]
pub async fn configure_workspace(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    draft: WorkspaceConfigurationInput,
) -> Result<WorkspaceStatusDto, WorkspaceConfigurationErrorDto> {
    let configuration = acm_os_application::configure_workspace(
        database.inner(),
        acm_os_application::WorkspaceConfigurationDraft {
            active_vault_path: draft.active_vault_path,
            problem_root_path: draft.problem_root_path,
            knowledge_root_path: draft.knowledge_root_path,
        },
    )
    .await
    .map_err(workspace_error_dto)?;

    Ok(workspace_status_dto(
        acm_os_application::WorkspaceConfigurationStatus::Configured(configuration),
    ))
}

fn workspace_status_dto(
    status: acm_os_application::WorkspaceConfigurationStatus,
) -> WorkspaceStatusDto {
    match status {
        acm_os_application::WorkspaceConfigurationStatus::Unconfigured => WorkspaceStatusDto {
            state: "unconfigured",
            active_vault_path: None,
            problem_root_path: None,
            knowledge_root_path: None,
        },
        acm_os_application::WorkspaceConfigurationStatus::Configured(configuration) => {
            WorkspaceStatusDto {
                state: "configured",
                active_vault_path: Some(configuration.active_vault_path().to_owned()),
                problem_root_path: Some(configuration.problem_root_path().to_owned()),
                knowledge_root_path: Some(configuration.knowledge_root_path().to_owned()),
            }
        }
    }
}

fn workspace_error_dto(
    error: acm_os_application::WorkspaceConfigurationError,
) -> WorkspaceConfigurationErrorDto {
    WorkspaceConfigurationErrorDto {
        code: error.code(),
        field: error.field().map(|field| field.code()),
    }
}

#[cfg(test)]
mod tests {
    use acm_os_application::{
        StartupGateStatus, StartupRecoveryReason, WorkspaceConfiguration,
        WorkspaceConfigurationError, WorkspaceConfigurationStatus, WorkspacePathField,
    };
    use serde_json::json;

    use super::{startup_status_dto, workspace_error_dto, workspace_status_dto};

    #[test]
    fn serializes_ready_startup_contract() {
        let dto = startup_status_dto(&StartupGateStatus::Ready { schema_version: 1 });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize ready startup status"),
            json!({
                "state": "ready",
                "schemaVersion": 1,
                "recoveryReason": null,
                "supportedSchemaVersion": 1,
                "foundSchemaVersion": null
            })
        );
    }

    #[test]
    fn serializes_recovery_startup_contract() {
        let dto = startup_status_dto(&StartupGateStatus::RecoveryRequired {
            reason: StartupRecoveryReason::MigrationFailed,
        });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize recovery startup status"),
            json!({
                "state": "recoveryRequired",
                "schemaVersion": null,
                "recoveryReason": "migration_failed",
                "supportedSchemaVersion": null,
                "foundSchemaVersion": null
            })
        );
    }

    #[test]
    fn serializes_unsupported_schema_contract() {
        let dto = startup_status_dto(&StartupGateStatus::RecoveryRequired {
            reason: StartupRecoveryReason::UnsupportedSchema {
                found: 4,
                supported: 1,
            },
        });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize unsupported schema status"),
            json!({
                "state": "recoveryRequired",
                "schemaVersion": null,
                "recoveryReason": "unsupported_schema",
                "supportedSchemaVersion": 1,
                "foundSchemaVersion": 4
            })
        );
    }

    #[test]
    fn serializes_workspace_status_contract() {
        let (vault, problem_root, knowledge_root) = if cfg!(windows) {
            ("C:\\Vault", "C:\\Vault\\Problems", "C:\\Vault\\Knowledge")
        } else {
            ("/Vault", "/Vault/Problems", "/Vault/Knowledge")
        };
        let dto = workspace_status_dto(WorkspaceConfigurationStatus::Configured(
            WorkspaceConfiguration::from_resolved(
                vault.to_owned(),
                problem_root.to_owned(),
                knowledge_root.to_owned(),
            )
            .expect("valid resolved workspace"),
        ));
        assert_eq!(
            serde_json::to_value(dto).expect("serialize workspace status"),
            json!({
                "state": "configured",
                "activeVaultPath": vault,
                "problemRootPath": problem_root,
                "knowledgeRootPath": knowledge_root
            })
        );
    }

    #[test]
    fn serializes_workspace_validation_error_contract() {
        let dto = workspace_error_dto(WorkspaceConfigurationError::RootOutsideVault {
            field: WorkspacePathField::KnowledgeRoot,
        });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize workspace error"),
            json!({
                "code": "root_outside_vault",
                "field": "knowledge_root"
            })
        );
    }
}
