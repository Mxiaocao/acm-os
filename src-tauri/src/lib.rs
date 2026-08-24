mod ipc;
mod vault_watcher;

use acm_os_application::{StartupRecoveryReason, StartupStatusQuery, WorkspaceConfigurationStatus};
use acm_os_infrastructure::DatabaseRuntime;
use std::path::PathBuf;
use tauri::Manager;

#[cfg(feature = "desktop-e2e")]
fn app_private_data_path(_app: &tauri::App) -> Result<PathBuf, ()> {
    std::env::var_os("ACM_OS_E2E_ROOT")
        .map(PathBuf::from)
        .ok_or(())
}

#[cfg(not(feature = "desktop-e2e"))]
fn app_private_data_path(app: &tauri::App) -> Result<PathBuf, ()> {
    app.path().app_local_data_dir().map_err(|_| ())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    builder
        .setup(|app| {
            let database = match app_private_data_path(app) {
                Ok(app_private_data) => {
                    let database = tauri::async_runtime::block_on(
                        acm_os_infrastructure::start_database(&app_private_data),
                    );
                    database
                }
                Err(_) => DatabaseRuntime::recovery(StartupRecoveryReason::AppDataUnavailable),
            };
            let startup_query = StartupStatusQuery::new(database.status().clone());
            let watcher = vault_watcher::VaultWatcher::new();
            if let Ok(WorkspaceConfigurationStatus::Configured(configuration)) =
                tauri::async_runtime::block_on(acm_os_application::query_workspace_configuration(
                    &database,
                ))
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
            ipc::system_health_snapshot,
            ipc::startup_status,
            ipc::app_shell_status,
            ipc::contest_shelf,
            ipc::contest_library_list_families,
            ipc::contest_library_create_family,
            ipc::contest_library_rename_family,
            ipc::contest_library_list_series,
            ipc::contest_library_create_series,
            ipc::contest_library_rename_series,
            ipc::contest_library_list_years,
            ipc::contest_library_list_contest_placements,
            ipc::contest_library_create_placement,
            ipc::contest_library_update_placement,
            ipc::contest_library_remove_placement,
            ipc::contest_library_list_contests,
            ipc::contest_detail,
            ipc::complete_contest_facts,
            ipc::correct_contest_problem_facts,
            ipc::set_contest_archived,
            ipc::preview_delete_contest,
            ipc::delete_contest,
            ipc::preview_contest_ai_analysis,
            ipc::save_contest_ai_analysis,
            ipc::import_codeforces_contest,
            ipc::import_manual_codeforces_contest,
            ipc::lightweight_problems,
            ipc::lightweight_problem_detail,
            ipc::lightweight_problem_detail_by_id,
            ipc::create_personal_note,
            ipc::transition_problem_lifecycle,
            ipc::transition_problem_lifecycle_by_id,
            ipc::start_or_resume_review,
            ipc::start_or_resume_review_by_id,
            ipc::review_focus,
            ipc::review_help_drawer,
            ipc::reveal_review_help,
            ipc::complete_review,
            ipc::void_review,
            ipc::review_attempt_history,
            ipc::review_history,
            ipc::review_history_by_id,
            ipc::update_problem_mastery_evidence,
            ipc::update_problem_mastery_evidence_by_id,
            ipc::today_snapshot,
            ipc::weekly_acm_budget,
            ipc::save_weekly_acm_budget,
            ipc::reorder_today,
            ipc::preview_today_replan,
            ipc::apply_today_replan,
            ipc::complete_today_entry,
            ipc::today_extra_suggestions,
            ipc::accept_today_extra_suggestion,
            ipc::delete_personal_note,
            ipc::delete_personal_note_by_id,
            ipc::personal_note_projection,
            ipc::personal_note_projection_by_id,
            ipc::personal_note_relocation_candidates,
            ipc::personal_note_relocation_candidates_by_id,
            ipc::rebind_personal_note,
            ipc::rebind_personal_note_by_id,
            ipc::confirm_personal_note_deleted,
            ipc::confirm_personal_note_deleted_by_id,
            ipc::open_personal_note_in_obsidian,
            ipc::open_personal_note_in_obsidian_by_id,
            ipc::open_original_oj,
            ipc::statement_assets_by_id,
            ipc::knowledge_index,
            ipc::knowledge_relocation_candidates,
            ipc::rebind_knowledge_node,
            ipc::confirm_knowledge_markdown_deleted,
            ipc::resolve_knowledge_identity_conflict,
            ipc::knowledge_detail,
            ipc::confirm_knowledge_understanding,
            ipc::knowledge_reevaluation_suggestion,
            ipc::knowledge_candidates,
            ipc::knowledge_candidates_by_id,
            ipc::register_knowledge_candidate,
            ipc::set_knowledge_candidate_disposition,
            ipc::set_knowledge_candidate_disposition_by_id,
            ipc::accept_existing_knowledge_candidate,
            ipc::accept_existing_knowledge_candidate_by_id,
            ipc::open_knowledge_in_obsidian,
            ipc::statement_assets,
            ipc::workspace_status,
            ipc::preview_manual_backup,
            ipc::create_manual_backup,
            ipc::backup_inventory,
            ipc::preview_system_restore_candidate,
            ipc::prepare_system_restore,
            ipc::restart_for_pending_restore,
            ipc::restore_diagnostics,
            ipc::confirm_restore_rollback_cleanup,
            ipc::preview_post_restore_rebuild,
            ipc::validate_post_restore_problem_bindings,
            ipc::validate_post_restore_knowledge_bindings,
            ipc::check_post_restore_rebuild_preconditions,
            ipc::apply_post_restore_rebuild,
            ipc::preview_diagnostic_export,
            ipc::create_diagnostic_export,
            ipc::create_weekly_backup,
            ipc::preview_backup_retention,
            ipc::apply_backup_retention,
            ipc::configure_workspace,
            #[cfg(feature = "desktop-e2e")]
            ipc::desktop_e2e_context,
            #[cfg(feature = "desktop-e2e")]
            ipc::desktop_e2e_set_date,
            #[cfg(feature = "desktop-e2e")]
            ipc::desktop_e2e_log,
            #[cfg(feature = "desktop-e2e")]
            ipc::desktop_e2e_finish,
            #[cfg(feature = "desktop-e2e")]
            ipc::desktop_e2e_exit
        ])
        .run(tauri::generate_context!())
        .expect("error while running ACM-OS");
}
