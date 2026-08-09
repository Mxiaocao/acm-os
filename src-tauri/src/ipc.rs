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

#[cfg(test)]
mod tests {
    use acm_os_application::{StartupGateStatus, StartupRecoveryReason};
    use serde_json::json;

    use super::startup_status_dto;

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
}
