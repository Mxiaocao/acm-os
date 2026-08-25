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
                    if matches!(
                        database.status(),
                        acm_os_application::StartupGateStatus::Ready { .. }
                    ) {
                        let _ = tauri::async_runtime::block_on(
                            database.process_pending_review_rewards(256),
                        );
                    }
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
            ipc::reward_activation_state,
            ipc::reward_account_summary,
            ipc::list_custom_rewards,
            ipc::reward_redemption_history,
            ipc::activate_reward,
            ipc::create_custom_reward,
            ipc::update_custom_reward,
            ipc::archive_custom_reward,
            ipc::redeem_custom_reward,
            ipc::refund_custom_reward,
            ipc::system_health_snapshot,
            ipc::startup_status,
            ipc::app_shell_status,
            ipc::contest_shelf,
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
            ipc::create_personal_note,
            ipc::transition_problem_lifecycle,
            ipc::start_or_resume_review,
            ipc::review_focus,
            ipc::review_help_drawer,
            ipc::reveal_review_help,
            ipc::complete_review,
            ipc::void_review,
            ipc::review_attempt_history,
            ipc::review_history,
            ipc::update_problem_mastery_evidence,
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
            ipc::personal_note_projection,
            ipc::personal_note_relocation_candidates,
            ipc::rebind_personal_note,
            ipc::confirm_personal_note_deleted,
            ipc::open_personal_note_in_obsidian,
            ipc::open_original_oj,
            ipc::knowledge_index,
            ipc::knowledge_relocation_candidates,
            ipc::rebind_knowledge_node,
            ipc::confirm_knowledge_markdown_deleted,
            ipc::resolve_knowledge_identity_conflict,
            ipc::knowledge_detail,
            ipc::confirm_knowledge_understanding,
            ipc::knowledge_reevaluation_suggestion,
            ipc::knowledge_candidates,
            ipc::register_knowledge_candidate,
            ipc::set_knowledge_candidate_disposition,
            ipc::accept_existing_knowledge_candidate,
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
