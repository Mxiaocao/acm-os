use super::*;
use std::future::Future;
use tempfile::TempDir;
fn create(name: &str, root: &Path) -> CreateVaultRequest {
    CreateVaultRequest {
        name: name.into(),
        root_path: root.to_string_lossy().into_owned(),
    }
}
#[tokio::test]
async fn real_databases_switch_restart_and_delete() {
    let app = TempDir::new().unwrap();
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    let va = m.create(create("A", a.path())).await.unwrap();
    let vb = m.create(create("B", b.path())).await.unwrap();
    assert_ne!(va.database_path, vb.database_path);
    assert!(fs::metadata(&va.database_path).unwrap().len() > 4096);
    let sa = m
        .open_vault(VaultRequest {
            vault_id: va.id.clone(),
        })
        .await
        .unwrap();
    let ca = sa.context.unwrap();
    assert!(m.database(&ca).await.is_ok());
    let sb = m
        .open_vault(VaultRequest {
            vault_id: vb.id.clone(),
        })
        .await
        .unwrap();
    assert!(matches!(
        m.database(&ca).await,
        Err(VaultError {
            code: VaultErrorCode::StaleVaultContext
        })
    ));
    assert!(matches!(
        m.preview_delete(VaultRequest {
            vault_id: vb.id.clone()
        })
        .await,
        Err(VaultError {
            code: VaultErrorCode::CurrentVaultDeleteForbidden
        })
    ));
    let p = m
        .preview_delete(VaultRequest {
            vault_id: va.id.clone(),
        })
        .await
        .unwrap();
    m.delete(DeleteVaultRequest {
        vault_id: va.id,
        token: p.token,
        confirmation: p.confirmation_text,
    })
    .await
    .unwrap();
    assert!(Path::new(&p.retained_data_path)
        .join("system-facts.sqlite3")
        .exists());
    let old = sb.context.unwrap();
    m.shutdown().await;
    drop(m);
    let m = VaultManager::open(app.path()).await.unwrap();
    let status = m.status().await;
    assert_eq!(status.context.as_ref().unwrap().vault_id, vb.id);
    assert_ne!(status.context.unwrap().epoch, old.epoch);
    m.shutdown().await;
}
#[tokio::test]
async fn corrupt_registry_fails_closed() {
    let app = TempDir::new().unwrap();
    fs::write(app.path().join("vault-manager.json"), "{broken").unwrap();
    assert!(matches!(
        VaultManager::open(app.path()).await,
        Err(VaultError {
            code: VaultErrorCode::RegistryCorrupt
        })
    ));
}

use acm_os_application::{ContestImportPort, ContestReadPort, ProblemLibraryPort};
async fn import(
    lease: &VaultLease<'_>,
    id: u64,
    title: &str,
    problem_title: &str,
) -> acm_os_domain::CodeforcesProblemIdentity {
    let plan = acm_os_application::build_manual_codeforces_contest(
        id,
        title,
        &format!("https://codeforces.com/contest/{id}"),
        None,
        &[acm_os_application::ManualProblemDraft {
            index: "A".into(),
            title: problem_title.into(),
            source_url: format!("https://codeforces.com/contest/{id}/problem/A"),
            statement_text: "Real isolated problem statement".into(),
        }],
    )
    .unwrap();
    lease.persist_manifest(&plan.manifest).await.unwrap();
    assert!(lease.list_lightweight_problems().await.unwrap().is_empty());
    let preview = lease
        .preview_problem_library(&plan.manifest.contest)
        .await
        .unwrap();
    let selected: Vec<_> = preview
        .problems
        .iter()
        .map(|item| item.problem_id)
        .collect();
    lease
        .add_problem_library_items(&plan.manifest.contest, &selected)
        .await
        .unwrap();
    for snapshot in &plan.snapshots {
        lease.persist_first_snapshot(snapshot).await.unwrap();
    }
    plan.manifest.slots[0].problem.clone()
}
async fn open(m: &VaultManager, v: &VaultSummary) -> VaultContext {
    m.open_vault(VaultRequest {
        vault_id: v.id.clone(),
    })
    .await
    .unwrap()
    .context
    .unwrap()
}
#[tokio::test]
async fn contest_problem_reward_isolation_a_b_a_b_and_restart_preserves_legacy_bytes() {
    let app = TempDir::new().unwrap();
    let legacy_root = TempDir::new().unwrap();
    // Build an actual Legacy database using the existing engine, then freeze its bytes.
    let legacy = start_database(app.path()).await;
    fs::create_dir(legacy_root.path().join("Problems")).unwrap();
    fs::create_dir(legacy_root.path().join("Knowledge")).unwrap();
    acm_os_application::configure_workspace(
        &legacy,
        WorkspaceConfigurationDraft {
            active_vault_path: legacy_root.path().to_string_lossy().into_owned(),
            problem_root_path: legacy_root
                .path()
                .join("Problems")
                .to_string_lossy()
                .into_owned(),
            knowledge_root_path: legacy_root
                .path()
                .join("Knowledge")
                .to_string_lossy()
                .into_owned(),
        },
    )
    .await
    .unwrap();
    acm_os_application::create_family(&legacy, "Legacy sentinel family")
        .await
        .unwrap();
    legacy.close().await;
    drop(legacy);
    let legacy_file = app.path().join("system-facts.sqlite3");
    let before = fs::read(&legacy_file).unwrap();
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    assert!(matches!(
        m.create(create("Forbidden", legacy_root.path())).await,
        Err(VaultError {
            code: VaultErrorCode::LegacyProtected
        })
    ));
    let va = m.create(create("A", a.path())).await.unwrap();
    let vb = m.create(create("B", b.path())).await.unwrap();
    let ca = open(&m, &va).await;
    let account_a = {
        let lease = m.database(&ca).await.unwrap();
        let problem = import(&lease, 88001, "Contest A", "Problem A").await;
        acm_os_application::activate_reward(lease.inner())
            .await
            .unwrap();
        acm_os_application::create_personal_note(lease.inner(), &problem)
            .await
            .unwrap();
        let problem_id: i64 = sqlx::query_scalar("SELECT id FROM problems LIMIT 1")
            .fetch_one(lease.vault_test_pool())
            .await
            .unwrap();
        let today = acm_os_domain::LocalDate::parse_iso("2026-09-15").unwrap();
        for action in [
            acm_os_domain::ProblemLifecycleAction::JoinUpsolve,
            acm_os_domain::ProblemLifecycleAction::StartLearning,
            acm_os_domain::ProblemLifecycleAction::MarkUnderstood,
        ] {
            acm_os_application::transition_problem_lifecycle_by_id(
                lease.inner(),
                problem_id,
                action,
                today,
            )
            .await
            .unwrap();
        }
        let occurrence_id: String =
            sqlx::query_scalar("SELECT id FROM problem_completion_occurrences LIMIT 1")
                .fetch_one(lease.vault_test_pool())
                .await
                .unwrap();
        acm_os_application::process_problem_solved_reward(
            lease.inner(),
            acm_os_application::ProblemSolvedRewardRequest {
                occurrence_id,
                qualification: Some(acm_os_application::ProblemSolvedQualificationInput {
                    qualification: acm_os_domain::ProblemSolvedQualification::Qualifying,
                    reason:
                        acm_os_domain::ProblemSolvedEvaluationReason::ExplicitlyAcceptedDigested,
                    evaluated_at_utc: chrono::Utc::now()
                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                }),
            },
        )
        .await
        .unwrap();
        let account = acm_os_application::load_reward_account(lease.inner())
            .await
            .unwrap();
        assert!(account.xp_balance > 0 && account.coin_balance > 0);
        acm_os_application::create_custom_reward(lease.inner(), "A only", 1)
            .await
            .unwrap();
        account
    };
    let cb = open(&m, &vb).await;
    {
        let lease = m.database(&cb).await.unwrap();
        assert!(
            lease.list_contests().await.unwrap().is_empty(),
            "Contest A leaked to B"
        );
        assert!(lease.list_lightweight_problems().await.unwrap().is_empty());
        let account = acm_os_application::load_reward_account(lease.inner())
            .await
            .unwrap();
        assert_eq!((account.xp_balance, account.coin_balance), (0, 0));
        assert!(acm_os_application::list_custom_rewards(lease.inner())
            .await
            .unwrap()
            .is_empty());
        let problem = import(&lease, 88002, "Contest B", "Problem B").await;
        acm_os_application::create_personal_note(lease.inner(), &problem)
            .await
            .unwrap();
    }
    assert!(matches!(
        m.database(&ca).await,
        Err(VaultError {
            code: VaultErrorCode::StaleVaultContext
        })
    ));
    let ca2 = open(&m, &va).await;
    assert!(
        matches!(
            m.database(&ca).await,
            Err(VaultError {
                code: VaultErrorCode::StaleVaultContext
            })
        ),
        "A→B→A must not revive an old request"
    );
    {
        let lease = m.database(&ca2).await.unwrap();
        let contests = lease.list_contests().await.unwrap();
        assert_eq!(contests.len(), 1);
        assert_eq!(contests[0].title, "Contest A");
        assert!(lease
            .list_lightweight_problems()
            .await
            .unwrap()
            .iter()
            .all(|p| p.title != "Problem B"));
        assert_eq!(
            acm_os_application::load_reward_account(lease.inner())
                .await
                .unwrap(),
            account_a
        );
    }
    let cb2 = open(&m, &vb).await;
    {
        let lease = m.database(&cb2).await.unwrap();
        let problems = lease.list_lightweight_problems().await.unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].title, "Problem B");
    }
    m.shutdown().await;
    drop(m);
    assert_eq!(before, fs::read(&legacy_file).unwrap());
    let restarted = VaultManager::open(app.path()).await.unwrap();
    let cb3 = restarted.status().await.context.unwrap();
    assert_eq!(cb3.vault_id, vb.id);
    assert_ne!(cb3.epoch, cb2.epoch);
    {
        let lease = restarted.database(&cb3).await.unwrap();
        assert_eq!(lease.list_contests().await.unwrap()[0].title, "Contest B");
        assert_eq!(
            acm_os_application::load_reward_account(lease.inner())
                .await
                .unwrap()
                .coin_balance,
            0
        );
    }
    assert!(matches!(
        restarted.database(&cb2).await,
        Err(VaultError {
            code: VaultErrorCode::StaleVaultContext
        })
    ));
    restarted.shutdown().await;
    drop(restarted);
    assert_eq!(before, fs::read(&legacy_file).unwrap());
}
#[tokio::test]
async fn switch_waits_for_in_flight_write_then_rejects_old_context() {
    let app = TempDir::new().unwrap();
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    let va = m.create(create("A", a.path())).await.unwrap();
    let vb = m.create(create("B", b.path())).await.unwrap();
    let ca = open(&m, &va).await;
    let lease = m.database(&ca).await.unwrap();
    let switch = m.open_vault(VaultRequest {
        vault_id: vb.id.clone(),
    });
    tokio::pin!(switch);
    // A single poll of the real switch future must be Pending while the command lease is held.
    std::future::poll_fn(|cx| {
        assert!(switch.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    acm_os_application::create_family(lease.inner(), "in flight A")
        .await
        .unwrap();
    drop(lease);
    let cb = switch.await.unwrap().context.unwrap();
    assert_eq!(cb.vault_id, vb.id);
    assert!(matches!(
        m.database(&ca).await,
        Err(VaultError {
            code: VaultErrorCode::StaleVaultContext
        })
    ));
    let lease = m.database(&cb).await.unwrap();
    use acm_os_application::ContestLibraryPort;
    assert!(!lease
        .list_families()
        .await
        .unwrap()
        .iter()
        .any(|f| f.display_name == "in flight A"));
    drop(lease);
    m.shutdown().await;
}
#[tokio::test]
async fn validation_confirmation_and_missing_root_fail_without_switching() {
    let app = TempDir::new().unwrap();
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    let va = m.create(create("A", a.path())).await.unwrap();
    let vb = m.create(create("B", b.path())).await.unwrap();
    let ca = open(&m, &va).await;
    assert!(matches!(
        m.create(create("A duplicate", a.path())).await,
        Err(VaultError {
            code: VaultErrorCode::DuplicateRoot
        })
    ));
    fs::create_dir(a.path().join("nested")).unwrap();
    assert!(matches!(
        m.create(create("nested", &a.path().join("nested"))).await,
        Err(VaultError {
            code: VaultErrorCode::RootNested
        })
    ));
    let p = m
        .preview_delete(VaultRequest {
            vault_id: vb.id.clone(),
        })
        .await
        .unwrap();
    let bad = DeleteVaultRequest {
        vault_id: vb.id.clone(),
        token: p.token,
        confirmation: "wrong".into(),
    };
    assert!(matches!(
        m.delete(bad.clone()).await,
        Err(VaultError {
            code: VaultErrorCode::ConfirmationMismatch
        })
    ));
    assert!(matches!(
        m.delete(bad).await,
        Err(VaultError {
            code: VaultErrorCode::InvalidConfirmationToken
        })
    ));
    let marker = b.path().join(".acm-os/vault-id");
    fs::write(&marker, "someone else").unwrap();
    assert!(m
        .open_vault(VaultRequest { vault_id: vb.id })
        .await
        .is_err());
    assert_eq!(m.status().await.context, Some(ca));
    assert!(b.path().join(".acm-os/system-facts.sqlite3").exists());
    m.shutdown().await;
}
#[tokio::test]
async fn registry_commit_failure_preserves_active_binding_and_data() {
    let app = TempDir::new().unwrap();
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    let va = m.create(create("A", a.path())).await.unwrap();
    let vb = m.create(create("B", b.path())).await.unwrap();
    let ca = open(&m, &va).await;
    let path = app.path().join("vault-manager.json");
    let saved = app.path().join("saved-registry.json");
    fs::rename(&path, &saved).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(matches!(
        m.open_vault(VaultRequest {
            vault_id: vb.id.clone()
        })
        .await,
        Err(VaultError {
            code: VaultErrorCode::RegistryUnavailable
        })
    ));
    assert_eq!(m.status().await.context, Some(ca));
    let preview = m
        .preview_delete(VaultRequest {
            vault_id: vb.id.clone(),
        })
        .await
        .unwrap();
    assert!(m
        .delete(DeleteVaultRequest {
            vault_id: vb.id,
            token: preview.token,
            confirmation: preview.confirmation_text
        })
        .await
        .is_err());
    assert!(b.path().join(".acm-os/system-facts.sqlite3").exists());
    fs::remove_dir(path).unwrap();
    m.shutdown().await;
}
#[tokio::test]
async fn unavailable_saved_vault_never_falls_back_to_legacy_or_other_vault() {
    let app = TempDir::new().unwrap();
    let a = TempDir::new().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    let va = m.create(create("A", a.path())).await.unwrap();
    open(&m, &va).await;
    m.shutdown().await;
    drop(m);
    fs::rename(a.path().join(".acm-os"), a.path().join("offline")).unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    assert!(matches!(
        m.status().await.current,
        CurrentVaultState::Unavailable { .. }
    ));
    assert!(m.status().await.context.is_none());
    assert!(!app.path().join("system-facts.sqlite3").exists());
    m.shutdown().await;
}
#[test]
fn typed_requests_reject_missing_or_extra_fields_and_use_camel_case() {
    assert!(serde_json::from_str::<VaultContext>(r#"{"vaultId":"a"}"#).is_err());
    assert!(serde_json::from_str::<CreateVaultRequest>(
        r#"{"name":"A","rootPath":"C:/A","databasePath":"C:/Legacy"}"#
    )
    .is_err());
    let context: VaultContext =
        serde_json::from_str(r#"{"vaultId":"a","epoch":"session"}"#).unwrap();
    assert_eq!(context.vault_id, "a");
    assert_eq!(
        serde_json::to_value(VaultError::from(VaultErrorCode::StaleVaultContext)).unwrap(),
        serde_json::json!({"code":"stale_vault_context"})
    );
}

#[tokio::test]
async fn copied_foreign_database_is_rejected_before_any_writable_startup() {
    let app = TempDir::new().unwrap();
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    let va = m.create(create("A", a.path())).await.unwrap();
    let vb = m.create(create("B", b.path())).await.unwrap();
    fs::copy(&va.database_path, &vb.database_path).unwrap();
    let foreign_before = fs::read(&vb.database_path).unwrap();
    assert!(matches!(
        m.open_vault(VaultRequest { vault_id: vb.id }).await,
        Err(VaultError {
            code: VaultErrorCode::VaultUnavailable
        })
    ));
    assert_eq!(foreign_before, fs::read(&vb.database_path).unwrap());
    assert!(matches!(m.status().await.current, CurrentVaultState::None));
    m.shutdown().await;
}

#[tokio::test]
async fn manager_recovery_files_are_not_note_relocation_candidates() {
    let app = TempDir::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let m = VaultManager::open(app.path()).await.unwrap();
    let v = m.create(create("A", root.path())).await.unwrap();
    let canonical = Path::new(&v.root_path);
    fs::write(canonical.join("Knowledge/Visible.md"), "visible").unwrap();
    fs::write(canonical.join(".acm-os/recovery.md"), "private").unwrap();
    fs::create_dir(canonical.join(".acm-os-deleted-old")).unwrap();
    fs::write(canonical.join(".acm-os-deleted-old/old.md"), "private old").unwrap();
    let files = crate::file_binding::markdown_files(canonical).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("Visible.md"));
    assert!(
        crate::file_binding::resolve_relative_markdown(&v.root_path, ".acm-os/recovery.md")
            .is_err()
    );
    m.shutdown().await;
}
