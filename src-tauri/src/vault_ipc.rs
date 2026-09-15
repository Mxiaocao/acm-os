use acm_os_infrastructure::vault_manager::{
    CreateVaultRequest, DeletePreview, DeleteVaultRequest, ManagerStatus, VaultError, VaultManager,
    VaultRequest, VaultSummary,
};
use tauri::Manager;
pub type DesktopVaultManager = Result<VaultManager, VaultError>;
#[tauri::command]
pub async fn vault_manager_status(
    manager: tauri::State<'_, DesktopVaultManager>,
) -> Result<ManagerStatus, VaultError> {
    Ok(manager
        .inner()
        .as_ref()
        .map_err(Clone::clone)?
        .status()
        .await)
}
#[tauri::command]
pub async fn create_vault(
    app: tauri::AppHandle,
    input: CreateVaultRequest,
) -> Result<VaultSummary, VaultError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DesktopVaultManager>();
        let manager = state.inner().as_ref().map_err(Clone::clone)?;
        tauri::async_runtime::block_on(manager.create(input))
    })
    .await
    .map_err(|_| {
        VaultError::from(
            acm_os_infrastructure::vault_manager::VaultErrorCode::PersistenceUnavailable,
        )
    })?
}
#[tauri::command]
pub async fn open_vault(
    app: tauri::AppHandle,
    input: VaultRequest,
) -> Result<ManagerStatus, VaultError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DesktopVaultManager>();
        let manager = state.inner().as_ref().map_err(Clone::clone)?;
        tauri::async_runtime::block_on(manager.open_vault_observed(input, |status| {
            let watcher = app.state::<crate::vault_watcher::VaultWatcher>();
            watcher.stop();
            if let acm_os_infrastructure::vault_manager::CurrentVaultState::Ready { vault } =
                &status.current
            {
                let _ = watcher.watch(&vault.root_path, app.clone());
            }
        }))
    })
    .await
    .map_err(|_| {
        VaultError::from(
            acm_os_infrastructure::vault_manager::VaultErrorCode::PersistenceUnavailable,
        )
    })?
}
#[tauri::command]
pub async fn preview_delete_vault(
    manager: tauri::State<'_, DesktopVaultManager>,
    input: VaultRequest,
) -> Result<DeletePreview, VaultError> {
    manager
        .inner()
        .as_ref()
        .map_err(Clone::clone)?
        .preview_delete(input)
        .await
}
#[tauri::command]
pub async fn delete_vault(
    manager: tauri::State<'_, DesktopVaultManager>,
    input: DeleteVaultRequest,
) -> Result<ManagerStatus, VaultError> {
    manager
        .inner()
        .as_ref()
        .map_err(Clone::clone)?
        .delete(input)
        .await
}
