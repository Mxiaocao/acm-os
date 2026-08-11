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
pub struct ContestShelfItemDto {
    contest_id: u64,
    title: String,
    import_status: &'static str,
    problem_count: u32,
    missing_snapshot_count: u32,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestDetailInput {
    contest_id: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestDetailDto {
    contest_id: u64,
    title: String,
    source_url: String,
    import_status: &'static str,
    problems: Vec<LightweightProblemItemDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightweightProblemItemDto {
    contest_id: u64,
    index: String,
    title: String,
    rating: Option<u32>,
    has_statement_snapshot: bool,
    identity_type: &'static str,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightweightProblemDetailInput {
    contest_id: u64,
    index: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightweightProblemDetailDto {
    contest_id: u64,
    index: String,
    title: String,
    rating: Option<u32>,
    source_url: String,
    statement: StatementReadStateDto,
    identity_type: &'static str,
    personal_note: Option<PersonalNoteBindingDto>,
    lifecycle: ProblemLifecycleStateDto,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemLifecycleStateDto {
    learning_status: &'static str,
    learning_status_since_utc: String,
    next_review_due_local_date: Option<String>,
    available_actions: Vec<&'static str>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemLifecycleCommandInput {
    contest_id: u64,
    index: String,
    action: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalNoteBindingDto {
    vault_relative_path: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemMarkdownProjectionDto {
    content_digest: String,
    known_sections: Vec<KnownMarkdownSectionDto>,
    solution_routes: Vec<SolutionRouteDto>,
    warnings: Vec<MarkdownParseWarningDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum PersonalNoteReadStateDto {
    Ready {
        #[serde(rename = "vaultRelativePath")]
        vault_relative_path: String,
        relocated: bool,
        projection: ProblemMarkdownProjectionDto,
    },
    LocationAnomaly {
        #[serde(rename = "lastKnownPath")]
        last_known_path: String,
    },
    VaultUnavailable {
        #[serde(rename = "lastKnownPath")]
        last_known_path: String,
    },
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownMarkdownSectionDto {
    name: String,
    start_offset: usize,
    end_offset: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionRouteDto {
    name: String,
    start_offset: usize,
    end_offset: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownParseWarningDto {
    code: &'static str,
    name: String,
    count: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum StatementReadStateDto {
    Pending,
    Ready { #[serde(rename = "sanitizedHtml")] sanitized_html: String },
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStatementAssetDto {
    local_ref: String,
    media_type: String,
    bytes: Vec<u8>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestImportInput {
    contest_url: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestImportRunDto {
    import_status: &'static str,
    missing_snapshot_problems: Vec<String>,
    failed_snapshot_problems: Vec<String>,
}

#[tauri::command]
pub async fn import_codeforces_contest(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestImportInput,
) -> Result<ContestImportRunDto, &'static str> {
    use acm_os_application::import_codeforces_contest;

    let contest = acm_os_application::codeforces::locate_public_contest(&input.contest_url)
        .map_err(|_| "unsupported_contest_url")?;
    let source = acm_os_infrastructure::codeforces::CodeforcesHttpAdapter::new()
        .map_err(|_| "adapter_unavailable")?;
    let result = import_codeforces_contest(database.inner(), &source, contest)
        .await
        .map_err(|error| match error {
            acm_os_application::ContestImportSourceError::Unavailable => "import_unavailable",
            acm_os_application::ContestImportSourceError::InvalidRemoteData => "invalid_remote_data",
        })?;
    Ok(ContestImportRunDto {
        import_status: match result.persisted.status {
            acm_os_application::ContestImportStatus::Incomplete => "incomplete",
            acm_os_application::ContestImportStatus::Complete => "complete",
        },
        missing_snapshot_problems: result.persisted.missing_snapshot_problems.into_iter()
            .map(|problem| format!("{}{}", problem.contest().contest_id(), problem.index()))
            .collect(),
        failed_snapshot_problems: result.failed_snapshot_problems.into_iter()
            .map(|problem| format!("{}{}", problem.contest().contest_id(), problem.index()))
            .collect(),
    })
}

#[tauri::command]
pub async fn contest_shelf(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<Vec<ContestShelfItemDto>, ()> {
    use acm_os_application::ContestReadPort;
    database
        .list_contests()
        .await
        .map(|items| items.into_iter().map(contest_shelf_item_dto).collect())
        .map_err(|_| ())
}

#[tauri::command]
pub async fn contest_detail(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestDetailInput,
) -> Result<ContestDetailDto, &'static str> {
    use acm_os_application::ContestReadPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    database.contest_detail(&contest).await
        .map(contest_detail_dto)
        .map_err(contest_read_error_code)
}

#[tauri::command]
pub async fn lightweight_problems(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<Vec<LightweightProblemItemDto>, ()> {
    use acm_os_application::ContestReadPort;
    database
        .list_lightweight_problems()
        .await
        .map(|items| items.into_iter().map(lightweight_problem_item_dto).collect())
        .map_err(|_| ())
}

#[tauri::command]
pub async fn lightweight_problem_detail(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<LightweightProblemDetailDto, &'static str> {
    use acm_os_application::ContestReadPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    database.lightweight_problem_detail(&problem).await
        .map(lightweight_problem_detail_dto)
        .map_err(|error| match error {
            acm_os_application::ContestReadError::NotFound => "problem_not_found",
            acm_os_application::ContestReadError::Unavailable => "problem_unavailable",
        })
}

#[tauri::command]
pub async fn create_personal_note(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<PersonalNoteBindingDto, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    acm_os_application::create_personal_note(database.inner(), &problem)
        .await
        .map(personal_note_binding_dto)
        .map_err(acm_os_application::PersonalNoteError::code)
}

#[tauri::command]
pub async fn transition_problem_lifecycle(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ProblemLifecycleCommandInput,
) -> Result<ProblemLifecycleStateDto, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    let action = parse_problem_lifecycle_action(&input.action)?;
    let today = acm_os_infrastructure::current_local_date()
        .map_err(|_| "local_calendar_unavailable")?;
    acm_os_application::transition_problem_lifecycle(database.inner(), &problem, action, today)
        .await
        .map(problem_lifecycle_state_dto)
        .map_err(acm_os_application::ProblemLifecycleError::code)
}

#[tauri::command]
pub async fn delete_personal_note(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<ProblemLifecycleStateDto, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    acm_os_application::delete_personal_note(database.inner(), &problem)
        .await
        .map(problem_lifecycle_state_dto)
        .map_err(acm_os_application::PersonalNoteDeletionError::code)
}

#[tauri::command]
pub async fn personal_note_projection(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<PersonalNoteReadStateDto, &'static str> {
    use acm_os_application::PersonalNoteReadPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    database
        .read_personal_note_projection(&problem)
        .await
        .map(personal_note_read_state_dto)
        .map_err(acm_os_application::PersonalNoteReadError::code)
}

#[tauri::command]
pub async fn open_personal_note_in_obsidian(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    app: tauri::AppHandle,
    input: LightweightProblemDetailInput,
) -> Result<(), &'static str> {
    use acm_os_application::{
        PersonalNoteReadPort, PersonalNoteReadState, WorkspaceConfigurationPort,
    };
    use tauri_plugin_opener::OpenerExt;

    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    let state = database
        .read_personal_note_projection(&problem)
        .await
        .map_err(acm_os_application::PersonalNoteReadError::code)?;
    let binding = match state {
        PersonalNoteReadState::Ready { binding, .. } => binding,
        PersonalNoteReadState::LocationAnomaly { .. } => return Err("note_location_anomaly"),
        PersonalNoteReadState::VaultUnavailable { .. } => return Err("vault_unavailable"),
    };
    let workspace = database
        .load_workspace_configuration()
        .await
        .map_err(|_| "workspace_unavailable")?
        .ok_or("workspace_unavailable")?;
    let uri = obsidian_open_uri(
        workspace.active_vault_path(),
        &binding.vault_relative_path,
    )?;
    app.opener()
        .open_url(uri, None::<&str>)
        .map_err(|_| "obsidian_open_failed")
}

fn obsidian_open_uri(active_vault: &str, relative_path: &str) -> Result<String, &'static str> {
    let vault = std::fs::canonicalize(active_vault).map_err(|_| "vault_unavailable")?;
    let target = std::fs::canonicalize(vault.join(relative_path))
        .map_err(|_| "note_open_failed")?;
    if !target.starts_with(&vault) || !target.is_file() {
        return Err("note_open_failed");
    }
    let mut uri = url::Url::parse("obsidian://open").map_err(|_| "note_open_failed")?;
    uri.query_pairs_mut()
        .append_pair("path", &obsidian_external_path(&target)?);
    Ok(uri.into())
}

fn obsidian_external_path(path: &std::path::Path) -> Result<String, &'static str> {
    let raw = path.to_str().ok_or("note_open_failed")?;
    Ok(normalize_windows_verbatim_path(raw))
}

fn normalize_windows_verbatim_path(path: &str) -> String {
    if let Some(unc_path) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{unc_path}");
    }
    path.strip_prefix(r"\\?\").unwrap_or(path).to_owned()
}

fn personal_note_read_state_dto(
    state: acm_os_application::PersonalNoteReadState,
) -> PersonalNoteReadStateDto {
    match state {
        acm_os_application::PersonalNoteReadState::Ready {
            binding,
            projection,
            relocated,
        } => PersonalNoteReadStateDto::Ready {
            vault_relative_path: binding.vault_relative_path,
            relocated,
            projection: problem_markdown_projection_dto(projection),
        },
        acm_os_application::PersonalNoteReadState::LocationAnomaly { binding } => {
            PersonalNoteReadStateDto::LocationAnomaly {
                last_known_path: binding.vault_relative_path,
            }
        }
        acm_os_application::PersonalNoteReadState::VaultUnavailable { binding } => {
            PersonalNoteReadStateDto::VaultUnavailable {
                last_known_path: binding.vault_relative_path,
            }
        }
    }
}

#[tauri::command]
pub async fn statement_assets(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<Vec<LocalStatementAssetDto>, &'static str> {
    use acm_os_application::ContestReadPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    database.statement_assets(&problem).await
        .map(|assets| assets.into_iter().map(|asset| LocalStatementAssetDto {
            local_ref: asset.local_ref,
            media_type: asset.media_type,
            bytes: asset.bytes,
        }).collect())
        .map_err(contest_read_error_code)
}

fn contest_shelf_item_dto(item: acm_os_application::ContestShelfItem) -> ContestShelfItemDto {
    ContestShelfItemDto {
        contest_id: item.contest.contest_id(),
        title: item.title,
        import_status: match item.import_status {
            acm_os_application::ContestImportStatus::Incomplete => "incomplete",
            acm_os_application::ContestImportStatus::Complete => "complete",
        },
        problem_count: item.problem_count,
        missing_snapshot_count: item.missing_snapshot_count,
    }
}

fn contest_detail_dto(item: acm_os_application::ContestDetail) -> ContestDetailDto {
    ContestDetailDto {
        contest_id: item.contest.contest_id(),
        title: item.title,
        source_url: item.source_url,
        import_status: match item.import_status {
            acm_os_application::ContestImportStatus::Incomplete => "incomplete",
            acm_os_application::ContestImportStatus::Complete => "complete",
        },
        problems: item.problems.into_iter().map(lightweight_problem_item_dto).collect(),
    }
}

fn lightweight_problem_item_dto(item: acm_os_application::LightweightProblemItem) -> LightweightProblemItemDto {
    LightweightProblemItemDto {
        contest_id: item.problem.contest().contest_id(),
        index: item.problem.index().to_owned(),
        title: item.title,
        rating: item.rating,
        has_statement_snapshot: item.has_statement_snapshot,
        identity_type: problem_identity_type_dto(item.identity_type),
    }
}

fn lightweight_problem_detail_dto(item: acm_os_application::LightweightProblemDetail) -> LightweightProblemDetailDto {
    LightweightProblemDetailDto {
        contest_id: item.problem.contest().contest_id(),
        index: item.problem.index().to_owned(),
        title: item.title,
        rating: item.rating,
        source_url: item.source_url,
        statement: match item.statement {
            acm_os_application::StatementReadState::Pending => StatementReadStateDto::Pending,
            acm_os_application::StatementReadState::Ready { sanitized_html } => StatementReadStateDto::Ready { sanitized_html },
        },
        identity_type: problem_identity_type_dto(item.identity_type),
        personal_note: item.personal_note.map(personal_note_binding_dto),
        lifecycle: problem_lifecycle_state_dto(item.lifecycle),
    }
}

fn problem_lifecycle_state_dto(
    state: acm_os_application::ProblemLifecycleState,
) -> ProblemLifecycleStateDto {
    ProblemLifecycleStateDto {
        learning_status: learning_status_dto(state.learning_status),
        learning_status_since_utc: state.learning_status_since_utc,
        next_review_due_local_date: state
            .active_review_cycle
            .map(|cycle| cycle.next_due_local_date.to_iso_string()),
        available_actions: if state.identity_type == acm_os_application::ProblemIdentityType::Personal {
            acm_os_domain::ProblemLifecycleEngine::available_actions(state.learning_status)
                .iter()
                .copied()
                .map(problem_lifecycle_action_dto)
                .collect()
        } else {
            Vec::new()
        },
    }
}

fn learning_status_dto(status: acm_os_domain::LearningStatus) -> &'static str {
    match status {
        acm_os_domain::LearningStatus::Unstarted => "unstarted",
        acm_os_domain::LearningStatus::UpsolvePending => "upsolvePending",
        acm_os_domain::LearningStatus::Learning => "learning",
        acm_os_domain::LearningStatus::WaitingColdStart => "waitingColdStart",
        acm_os_domain::LearningStatus::Relearning => "relearning",
        acm_os_domain::LearningStatus::LongTermReview => "longTermReview",
    }
}

fn parse_problem_lifecycle_action(
    action: &str,
) -> Result<acm_os_domain::ProblemLifecycleAction, &'static str> {
    match action {
        "joinUpsolve" => Ok(acm_os_domain::ProblemLifecycleAction::JoinUpsolve),
        "startLearning" => Ok(acm_os_domain::ProblemLifecycleAction::StartLearning),
        "returnToPending" => Ok(acm_os_domain::ProblemLifecycleAction::ReturnToPending),
        "markUnderstood" => Ok(acm_os_domain::ProblemLifecycleAction::MarkUnderstood),
        "withdrawUnderstood" => Ok(acm_os_domain::ProblemLifecycleAction::WithdrawUnderstood),
        "startRelearning" => Ok(acm_os_domain::ProblemLifecycleAction::StartRelearning),
        "stopLearning" => Ok(acm_os_domain::ProblemLifecycleAction::StopLearning),
        _ => Err("invalid_lifecycle_action"),
    }
}

fn problem_lifecycle_action_dto(action: acm_os_domain::ProblemLifecycleAction) -> &'static str {
    match action {
        acm_os_domain::ProblemLifecycleAction::JoinUpsolve => "joinUpsolve",
        acm_os_domain::ProblemLifecycleAction::StartLearning => "startLearning",
        acm_os_domain::ProblemLifecycleAction::ReturnToPending => "returnToPending",
        acm_os_domain::ProblemLifecycleAction::MarkUnderstood => "markUnderstood",
        acm_os_domain::ProblemLifecycleAction::WithdrawUnderstood => "withdrawUnderstood",
        acm_os_domain::ProblemLifecycleAction::StartRelearning => "startRelearning",
        acm_os_domain::ProblemLifecycleAction::StopLearning => "stopLearning",
        acm_os_domain::ProblemLifecycleAction::DeletePersonalNote => "deletePersonalNote",
    }
}

fn problem_identity_type_dto(identity_type: acm_os_application::ProblemIdentityType) -> &'static str {
    match identity_type {
        acm_os_application::ProblemIdentityType::Lightweight => "lightweight",
        acm_os_application::ProblemIdentityType::Personal => "personal",
    }
}

fn personal_note_binding_dto(binding: acm_os_application::PersonalNoteBinding) -> PersonalNoteBindingDto {
    PersonalNoteBindingDto {
        vault_relative_path: binding.vault_relative_path,
    }
}

fn problem_markdown_projection_dto(
    projection: acm_os_application::ProblemMarkdownProjection,
) -> ProblemMarkdownProjectionDto {
    ProblemMarkdownProjectionDto {
        content_digest: projection.content_digest,
        known_sections: projection.known_sections.into_iter().map(|section| KnownMarkdownSectionDto {
            name: section.name,
            start_offset: section.start_offset,
            end_offset: section.end_offset,
        }).collect(),
        solution_routes: projection.solution_routes.into_iter().map(|route| SolutionRouteDto {
            name: route.name,
            start_offset: route.start_offset,
            end_offset: route.end_offset,
        }).collect(),
        warnings: projection.warnings.into_iter().map(|warning| match warning {
            acm_os_application::MarkdownParseWarning::DuplicateKnownSection { name, count } => MarkdownParseWarningDto {
                code: "duplicate_known_section",
                name,
                count,
            },
        }).collect(),
    }
}

fn contest_read_error_code(error: acm_os_application::ContestReadError) -> &'static str {
    match error {
        acm_os_application::ContestReadError::NotFound => "not_found",
        acm_os_application::ContestReadError::Unavailable => "unavailable",
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppShellStatusDto {
    state: &'static str,
    recovery_reason: Option<&'static str>,
    supported_schema_version: Option<i64>,
    found_schema_version: Option<i64>,
    workspace: Option<WorkspaceStatusDto>,
}

#[tauri::command]
pub async fn app_shell_status(
    startup: tauri::State<'_, acm_os_application::StartupStatusQuery>,
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<AppShellStatusDto, ()> {
    let workspace = match startup.execute() {
        acm_os_application::StartupGateStatus::Ready { .. } => {
            acm_os_application::query_workspace_configuration(database.inner())
                .await
                .ok()
        }
        acm_os_application::StartupGateStatus::RecoveryRequired { .. } => None,
    };
    let destination =
        acm_os_application::select_startup_destination(startup.execute(), workspace.as_ref());
    Ok(app_shell_status_dto(destination, workspace))
}

fn app_shell_status_dto(
    destination: acm_os_application::StartupDestination,
    workspace: Option<acm_os_application::WorkspaceConfigurationStatus>,
) -> AppShellStatusDto {
    match destination {
        acm_os_application::StartupDestination::Recovery { reason } => {
            let (supported_schema_version, found_schema_version) = match &reason {
                acm_os_application::StartupRecoveryReason::UnsupportedSchema {
                    found,
                    supported,
                } => (Some(*supported), Some(*found)),
                _ => (None, None),
            };
            AppShellStatusDto {
                state: "recovery",
                recovery_reason: Some(reason.code()),
                supported_schema_version,
                found_schema_version,
                workspace: None,
            }
        }
        acm_os_application::StartupDestination::Setup => AppShellStatusDto {
            state: "setup",
            recovery_reason: None,
            supported_schema_version: None,
            found_schema_version: None,
            workspace: workspace.map(workspace_status_dto),
        },
        acm_os_application::StartupDestination::Normal => AppShellStatusDto {
            state: "normal",
            recovery_reason: None,
            supported_schema_version: None,
            found_schema_version: None,
            workspace: workspace.map(workspace_status_dto),
        },
    }
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
    watcher: tauri::State<'_, crate::vault_watcher::VaultWatcher>,
    app: tauri::AppHandle,
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

    let _ = watcher.watch(configuration.active_vault_path(), app);

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
        ActiveReviewCycle,
        KnownMarkdownSection, MarkdownParseWarning, PersonalNoteBinding, PersonalNoteReadState,
        ProblemIdentityType, ProblemLifecycleState, ProblemMarkdownProjection, SolutionRoute,
        StartupDestination, StartupGateStatus, StartupRecoveryReason, WorkspaceConfiguration,
        WorkspaceConfigurationError, WorkspaceConfigurationStatus, WorkspacePathField,
    };
    use serde_json::json;

    use super::{
        app_shell_status_dto, normalize_windows_verbatim_path, obsidian_open_uri,
        personal_note_read_state_dto, problem_lifecycle_state_dto, startup_status_dto,
        workspace_error_dto, workspace_status_dto, LightweightProblemDetailDto,
        ProblemLifecycleStateDto, StatementReadStateDto,
    };

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

    #[test]
    fn serializes_problem_detail_without_source_html() {
        let dto = LightweightProblemDetailDto {
            contest_id: 1979,
            index: "A".to_owned(),
            title: "Problem A".to_owned(),
            rating: Some(800),
            source_url: "https://codeforces.com/contest/1979/problem/A".to_owned(),
            statement: StatementReadStateDto::Ready {
                sanitized_html: "<p>safe local statement</p>".to_owned(),
            },
            identity_type: "lightweight",
            personal_note: None,
            lifecycle: ProblemLifecycleStateDto {
                learning_status: "unstarted",
                learning_status_since_utc: "2026-08-11T00:00:00.000Z".to_owned(),
                next_review_due_local_date: None,
                available_actions: Vec::new(),
            },
        };
        assert_eq!(
            serde_json::to_value(dto).expect("serialize problem detail"),
            json!({
                "contestId": 1979,
                "index": "A",
                "title": "Problem A",
                "rating": 800,
                "sourceUrl": "https://codeforces.com/contest/1979/problem/A",
                "statement": {
                    "state": "ready",
                    "sanitizedHtml": "<p>safe local statement</p>"
                },
                "identityType": "lightweight",
                "personalNote": null,
                "lifecycle": {
                    "learningStatus": "unstarted",
                    "learningStatusSinceUtc": "2026-08-11T00:00:00.000Z",
                    "nextReviewDueLocalDate": null,
                    "availableActions": []
                }
            })
        );
    }

    #[test]
    fn lifecycle_dto_exposes_backend_actions_and_local_date_due() {
        let dto = problem_lifecycle_state_dto(ProblemLifecycleState {
            identity_type: ProblemIdentityType::Personal,
            learning_status: acm_os_domain::LearningStatus::WaitingColdStart,
            learning_status_since_utc: "2026-08-11T00:00:00.000Z".to_owned(),
            active_review_cycle: Some(ActiveReviewCycle {
                cycle_number: 1,
                stage: 0,
                schedule_rule_version: 1,
                next_due_local_date: acm_os_domain::LocalDate::parse_iso("2026-08-14")
                    .expect("local date"),
            }),
        });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize lifecycle"),
            json!({
                "learningStatus": "waitingColdStart",
                "learningStatusSinceUtc": "2026-08-11T00:00:00.000Z",
                "nextReviewDueLocalDate": "2026-08-14",
                "availableActions": ["withdrawUnderstood", "stopLearning"]
            })
        );
    }

    #[test]
    fn serializes_markdown_projection_contract() {
        let dto = personal_note_read_state_dto(PersonalNoteReadState::Ready {
            binding: PersonalNoteBinding {
                vault_relative_path: "Archive/note.md".to_owned(),
                content_digest: "abc123".to_owned(),
                windows_file_key: None,
            },
            relocated: true,
            projection: ProblemMarkdownProjection {
                content_digest: "abc123".to_owned(),
                known_sections: vec![KnownMarkdownSection {
                    name: "题解".to_owned(),
                    start_offset: 10,
                    end_offset: 42,
                }],
                solution_routes: vec![SolutionRoute {
                    name: "Route ×".to_owned(),
                    start_offset: 18,
                    end_offset: 42,
                }],
                warnings: vec![MarkdownParseWarning::DuplicateKnownSection {
                    name: "题解".to_owned(),
                    count: 2,
                }],
            },
        });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize Markdown projection"),
            json!({
                "state": "ready",
                "vaultRelativePath": "Archive/note.md",
                "relocated": true,
                "projection": {
                    "contentDigest": "abc123",
                    "knownSections": [{ "name": "题解", "startOffset": 10, "endOffset": 42 }],
                    "solutionRoutes": [{ "name": "Route ×", "startOffset": 18, "endOffset": 42 }],
                    "warnings": [{ "code": "duplicate_known_section", "name": "题解", "count": 2 }]
                }
            })
        );
    }

    #[test]
    fn serializes_affected_scope_markdown_states() {
        let binding = PersonalNoteBinding {
            vault_relative_path: "Problems/last-known.md".to_owned(),
            content_digest: "digest".to_owned(),
            windows_file_key: None,
        };
        assert_eq!(
            serde_json::to_value(personal_note_read_state_dto(
                PersonalNoteReadState::LocationAnomaly {
                    binding: binding.clone(),
                }
            ))
            .expect("serialize location anomaly"),
            json!({ "state": "locationAnomaly", "lastKnownPath": "Problems/last-known.md" })
        );
        assert_eq!(
            serde_json::to_value(personal_note_read_state_dto(
                PersonalNoteReadState::VaultUnavailable { binding }
            ))
            .expect("serialize Vault unavailable"),
            json!({ "state": "vaultUnavailable", "lastKnownPath": "Problems/last-known.md" })
        );
    }

    #[test]
    fn serializes_startup_shell_contracts() {
        let setup = app_shell_status_dto(
            StartupDestination::Setup,
            Some(WorkspaceConfigurationStatus::Unconfigured),
        );
        assert_eq!(
            serde_json::to_value(setup).expect("serialize setup shell"),
            json!({
                "state": "setup",
                "recoveryReason": null,
                "supportedSchemaVersion": null,
                "foundSchemaVersion": null,
                "workspace": {
                    "state": "unconfigured",
                    "activeVaultPath": null,
                    "problemRootPath": null,
                    "knowledgeRootPath": null
                }
            })
        );

        let recovery = app_shell_status_dto(
            StartupDestination::Recovery {
                reason: StartupRecoveryReason::UnsupportedSchema {
                    found: 3,
                    supported: 2,
                },
            },
            None,
        );
        assert_eq!(
            serde_json::to_value(recovery).expect("serialize recovery shell"),
            json!({
                "state": "recovery",
                "recoveryReason": "unsupported_schema",
                "supportedSchemaVersion": 2,
                "foundSchemaVersion": 3,
                "workspace": null
            })
        );

        let (vault, problem_root, knowledge_root) = if cfg!(windows) {
            ("C:\\Vault", "C:\\Vault\\Problems", "C:\\Vault\\Knowledge")
        } else {
            ("/Vault", "/Vault/Problems", "/Vault/Knowledge")
        };
        let normal = app_shell_status_dto(
            StartupDestination::Normal,
            Some(WorkspaceConfigurationStatus::Configured(
                WorkspaceConfiguration::from_resolved(
                    vault.to_owned(),
                    problem_root.to_owned(),
                    knowledge_root.to_owned(),
                )
                .expect("normal workspace"),
            )),
        );
        assert_eq!(
            serde_json::to_value(normal).expect("serialize normal shell"),
            json!({
                "state": "normal",
                "recoveryReason": null,
                "supportedSchemaVersion": null,
                "foundSchemaVersion": null,
                "workspace": {
                    "state": "configured",
                    "activeVaultPath": vault,
                    "problemRootPath": problem_root,
                    "knowledgeRootPath": knowledge_root
                }
            })
        );
    }

    #[test]
    fn obsidian_uri_uses_a_canonical_file_inside_the_active_vault() {
        let vault = tempfile::tempdir().expect("temporary vault");
        let problems = vault.path().join("Problems");
        std::fs::create_dir(&problems).expect("problem directory");
        std::fs::write(problems.join("A note.md"), "# Problem\n").expect("personal note");

        let uri = obsidian_open_uri(
            vault.path().to_str().expect("utf-8 vault"),
            "Problems/A note.md",
        )
        .expect("safe Obsidian URI");

        assert!(uri.starts_with("obsidian://open?path="));
        assert!(uri.contains("A+note.md") || uri.contains("A%20note.md"));
        let decoded_path = url::Url::parse(&uri)
            .expect("valid Obsidian URI")
            .query_pairs()
            .find_map(|(key, value)| (key == "path").then(|| value.into_owned()))
            .expect("path query parameter");
        assert!(!decoded_path.starts_with(r"\\?\"));
    }

    #[test]
    fn obsidian_uri_path_removes_windows_verbatim_prefixes() {
        assert_eq!(
            normalize_windows_verbatim_path(r"\\?\E:\ACM-Obsidian\题目笔记\CF-2256-C.md"),
            r"E:\ACM-Obsidian\题目笔记\CF-2256-C.md"
        );
        assert_eq!(
            normalize_windows_verbatim_path(r"\\?\UNC\server\share\Vault\Problems\A.md"),
            r"\\server\share\Vault\Problems\A.md"
        );
    }

    #[test]
    fn obsidian_uri_rejects_a_binding_outside_the_active_vault() {
        let parent = tempfile::tempdir().expect("temporary parent");
        let vault = parent.path().join("Vault");
        std::fs::create_dir(&vault).expect("vault");
        std::fs::write(parent.path().join("outside.md"), "# Outside\n")
            .expect("outside note");

        assert_eq!(
            obsidian_open_uri(vault.to_str().expect("utf-8 vault"), "../outside.md"),
            Err("note_open_failed")
        );
    }
}
