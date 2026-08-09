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
