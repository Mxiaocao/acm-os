mod ipc;
mod vault_watcher;

use acm_os_application::{StartupRecoveryReason, StartupStatusQuery, WorkspaceConfigurationStatus};
use acm_os_infrastructure::DatabaseRuntime;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let database = match app.path().app_local_data_dir() {
                Ok(app_private_data) => tauri::async_runtime::block_on(
                    acm_os_infrastructure::start_database(&app_private_data),
                ),
                Err(_) => DatabaseRuntime::recovery(StartupRecoveryReason::AppDataUnavailable),
            };
            let startup_query = StartupStatusQuery::new(database.status().clone());
            let watcher = vault_watcher::VaultWatcher::new();
            if let Ok(WorkspaceConfigurationStatus::Configured(configuration)) =
                tauri::async_runtime::block_on(
                    acm_os_application::query_workspace_configuration(&database),
                )
            {
                let _ = watcher.watch(configuration.active_vault_path(), app.handle().clone());
            }
            app.manage(watcher);
            app.manage(database);
            app.manage(startup_query);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::foundation_status,
            ipc::startup_status,
            ipc::app_shell_status,
            ipc::contest_shelf,
            ipc::contest_detail,
            ipc::import_codeforces_contest,
            ipc::lightweight_problems,
            ipc::lightweight_problem_detail,
            ipc::create_personal_note,
            ipc::transition_problem_lifecycle,
            ipc::delete_personal_note,
            ipc::personal_note_projection,
            ipc::open_personal_note_in_obsidian,
            ipc::statement_assets,
            ipc::workspace_status,
            ipc::configure_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running ACM-OS");
}
