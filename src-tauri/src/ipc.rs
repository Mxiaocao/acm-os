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
pub struct SystemHealthSnapshotDto {
    startup_state: &'static str,
    schema_version: Option<i64>,
    pending_critical_operation_count: u64,
    backup_file_count: u64,
    pending_restore_intent: bool,
    rollback_integrity_verified: Option<bool>,
}

#[tauri::command]
pub async fn system_health_snapshot(
    startup: tauri::State<'_, acm_os_application::StartupStatusQuery>,
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<SystemHealthSnapshotDto, ()> {
    let (startup_state, schema_version) = match startup.execute() {
        acm_os_application::StartupGateStatus::Ready { schema_version } => {
            ("ready", Some(*schema_version))
        }
        acm_os_application::StartupGateStatus::RecoveryRequired { .. } => {
            ("recoveryRequired", None)
        }
    };
    let health = database.system_health_snapshot().await.map_err(|_| ())?;
    Ok(SystemHealthSnapshotDto {
        startup_state,
        schema_version,
        pending_critical_operation_count: health.pending_critical_operation_count,
        backup_file_count: health.backup_file_count,
        pending_restore_intent: health.pending_restore_intent,
        rollback_integrity_verified: health.rollback_integrity_verified,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestShelfItemDto {
    contest_id: u64,
    title: String,
    import_status: &'static str,
    problem_count: u32,
    missing_snapshot_count: u32,
    archived: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryFamilyDto {
    family_id: i64,
    display_name: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibrarySeriesDto {
    series_id: i64,
    family_id: i64,
    display_name: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryPlacementDto {
    placement_id: i64,
    family_id: i64,
    family_name: String,
    series_id: Option<i64>,
    series_name: Option<String>,
    year: Option<u32>,
    ordinal: Option<u32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ContestLibrarySeriesFilterDto {
    Any,
    Unassigned,
    Exact {
        #[serde(rename = "seriesId")]
        series_id: i64,
    },
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ContestLibraryYearFilterDto {
    Any,
    Unassigned,
    Exact { year: u32 },
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ContestLibraryScopeDto {
    All,
    Family {
        #[serde(rename = "familyId")]
        family_id: i64,
        series: ContestLibrarySeriesFilterDto,
        year: ContestLibraryYearFilterDto,
    },
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContestLibraryArchiveFilterDto {
    All,
    Active,
    Archived,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryFamilyInput {
    display_name: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryFamilyRenameInput {
    family_id: i64,
    display_name: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryFamilyIdInput {
    family_id: i64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibrarySeriesInput {
    family_id: i64,
    display_name: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibrarySeriesRenameInput {
    series_id: i64,
    display_name: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryYearsInput {
    family_id: i64,
    series: ContestLibrarySeriesFilterDto,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryPlacementInput {
    contest_id: u64,
    family_id: i64,
    series_id: Option<i64>,
    year: Option<u32>,
    ordinal: Option<u32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryPlacementUpdateInput {
    placement_id: i64,
    family_id: i64,
    series_id: Option<i64>,
    year: Option<u32>,
    ordinal: Option<u32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryPlacementRemoveInput {
    placement_id: i64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestLibraryListInput {
    scope: ContestLibraryScopeDto,
    archive: ContestLibraryArchiveFilterDto,
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
    contest_date: Option<String>,
    import_status: &'static str,
    facts_status: &'static str,
    problems: Vec<ContestProblemDetailItemDto>,
    corrections: Vec<ContestCorrectionEventDto>,
    ai_analysis: Option<ContestAiAnalysisDto>,
    archived: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestDeletePreviewDto {
    contest_title: String,
    relationship_count: u32,
    cleanup_problem_count: u32,
    preserved_problem_count: u32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestAiAnalysisDto {
    raw_text: String,
    parse_status: &'static str,
    parsed_projection_json: String,
    updated_at_utc: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestCorrectionEventDto {
    correction_id: String,
    problem_index: String,
    field: &'static str,
    old_value: String,
    new_value: String,
    corrected_at_utc: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestProblemDetailItemDto {
    contest_id: u64,
    index: String,
    title: String,
    rating: Option<u32>,
    has_statement_snapshot: bool,
    identity_type: &'static str,
    final_contest_result: Option<&'static str>,
    upsolve_decision: &'static str,
    live_learning_status: &'static str,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteContestFactsInput {
    contest_id: u64,
    problems: Vec<ContestProblemFactInputDto>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestProblemFactInputDto {
    index: String,
    final_contest_result: String,
    upsolve_decision: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectContestProblemFactsInput {
    contest_id: u64,
    index: String,
    final_contest_result: String,
    upsolve_decision: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestAiAnalysisInput {
    contest_id: u64,
    raw_text: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestAiAnalysisPreviewDto {
    raw_text: String,
    parse_status: &'static str,
    parsed_projection_json: String,
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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebindPersonalNoteInput {
    contest_id: u64,
    index: String,
    vault_relative_path: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeNodeInput {
    knowledge_node_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSearchInput {
    query: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeUnderstandingInput {
    knowledge_node_id: String,
    level: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCandidateRegisterInput {
    contest_id: u64,
    index: String,
    fingerprint: String,
    target_ref: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCandidateDispositionInput {
    contest_id: u64,
    index: String,
    fingerprint: String,
    disposition: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCandidateAcceptInput {
    contest_id: u64,
    index: String,
    fingerprint: String,
    knowledge_node_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedKnowledgeCandidateDto {
    knowledge_node_id: String,
    target_ref: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCandidateDto {
    contest_id: u64,
    problem_index: String,
    fingerprint: String,
    target_ref: String,
    disposition: &'static str,
    knowledge_node_id: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeNodeDto {
    knowledge_node_id: String,
    display_name: String,
    vault_relative_path: String,
    content_digest: String,
    location_state: &'static str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeIndexDto {
    nodes: Vec<KnowledgeNodeDto>,
    location_anomalies: Vec<KnowledgeNodeDto>,
    identity_conflicts: Vec<KnowledgeIdentityConflictDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeIdentityConflictDto {
    historical_knowledge_node_id: String,
    display_name: String,
    candidate_vault_relative_path: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRelocationCandidateDto {
    vault_relative_path: String,
    occupied: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebindKnowledgeNodeInput {
    knowledge_node_id: String,
    vault_relative_path: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveKnowledgeIdentityConflictInput {
    historical_knowledge_node_id: String,
    candidate_vault_relative_path: String,
    restore_old_identity: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeUnderstandingDto {
    knowledge_node_id: String,
    current: &'static str,
    historical_highest: &'static str,
    first_reached_highest_on: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedKnowledgeProblemDto {
    problem_id: String,
    contest_id: u64,
    problem_index: String,
    title: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDetailDto {
    node: KnowledgeNodeDto,
    understanding: Option<KnowledgeUnderstandingDto>,
    incoming: Vec<KnowledgeNodeDto>,
    outgoing: Vec<KnowledgeNodeDto>,
    related_problems: Vec<RelatedKnowledgeProblemDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeReevaluationSuggestionDto {
    knowledge_node_id: String,
    should_suggest: bool,
    qualifying_problem_count: u32,
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
    review_action: Option<&'static str>,
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
pub struct TodayLoadInput {
    budget_minutes: Option<u32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayReorderInput {
    plan_id: String,
    ordered_entry_ids: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayDoneInput {
    plan_id: String,
    entry_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayReplanInput {
    budget_minutes: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyAcmBudgetDto {
    monday: Option<u32>,
    tuesday: Option<u32>,
    wednesday: Option<u32>,
    thursday: Option<u32>,
    friday: Option<u32>,
    saturday: Option<u32>,
    sunday: Option<u32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayAcceptSuggestionInput {
    preview: TodayExtraSuggestionsPreviewDto,
    problem_id: String,
}

#[cfg(feature = "desktop-e2e")]
fn command_local_date() -> Result<acm_os_domain::LocalDate, ()> {
    let path = std::env::var_os("ACM_OS_E2E_DATE_FILE").ok_or(())?;
    let value = std::fs::read_to_string(path).map_err(|_| ())?;
    acm_os_domain::LocalDate::parse_iso(value.trim()).map_err(|_| ())
}

#[cfg(not(feature = "desktop-e2e"))]
fn command_local_date() -> Result<acm_os_domain::LocalDate, ()> {
    acm_os_infrastructure::current_local_date()
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayEntryDto {
    entry_id: String,
    problem_id: String,
    contest_id: u64,
    problem_index: String,
    problem_title: String,
    review_attempt_id: Option<String>,
    lane: String,
    reason: String,
    planning_cost_minutes: u32,
    position: u32,
    origin: String,
    status: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodaySnapshotDto {
    plan_id: String,
    local_date: String,
    budget_minutes: u32,
    planned_minutes: u32,
    over_budget_minutes: u32,
    review_only_streak: u8,
    entries: Vec<TodayEntryDto>,
}

#[cfg(feature = "desktop-e2e")]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopE2eContextDto {
    vault: String,
    problems: String,
    knowledge: String,
    phase: String,
}

#[cfg(feature = "desktop-e2e")]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopE2eDateInput {
    local_date: String,
}

#[cfg(feature = "desktop-e2e")]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopE2eResultInput {
    result: String,
}

#[cfg(feature = "desktop-e2e")]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopE2eStageInput {
    stage: String,
}

#[cfg(feature = "desktop-e2e")]
#[tauri::command]
pub fn desktop_e2e_context() -> Result<DesktopE2eContextDto, &'static str> {
    let root = std::env::var_os("ACM_OS_E2E_ROOT")
        .map(std::path::PathBuf::from)
        .ok_or("desktop_e2e_unavailable")?;
    let vault = root.parent().unwrap_or(&root).join("vault");
    Ok(DesktopE2eContextDto {
        vault: vault.to_string_lossy().into_owned(),
        problems: vault.join("Problems").to_string_lossy().into_owned(),
        knowledge: vault.join("Knowledge").to_string_lossy().into_owned(),
        phase: std::env::var("ACM_OS_E2E_PHASE").unwrap_or_else(|_| "initial".to_owned()),
    })
}

#[cfg(feature = "desktop-e2e")]
#[tauri::command]
pub fn desktop_e2e_set_date(input: DesktopE2eDateInput) -> Result<(), &'static str> {
    acm_os_domain::LocalDate::parse_iso(&input.local_date)
        .map_err(|_| "invalid_desktop_e2e_date")?;
    let path = std::env::var_os("ACM_OS_E2E_DATE_FILE").ok_or("desktop_e2e_unavailable")?;
    std::fs::write(path, format!("{}\n", input.local_date)).map_err(|_| "desktop_e2e_write_failed")
}

#[cfg(feature = "desktop-e2e")]
#[tauri::command]
pub fn desktop_e2e_finish(input: DesktopE2eResultInput) -> Result<(), &'static str> {
    let root = std::env::var_os("ACM_OS_E2E_ROOT")
        .map(std::path::PathBuf::from)
        .ok_or("desktop_e2e_unavailable")?;
    std::fs::write(root.join("desktop-e2e-result.txt"), input.result)
        .map_err(|_| "desktop_e2e_write_failed")
}

#[cfg(feature = "desktop-e2e")]
#[tauri::command]
pub fn desktop_e2e_exit() {
    std::process::exit(0);
}

#[cfg(feature = "desktop-e2e")]
#[tauri::command]
pub fn desktop_e2e_log(input: DesktopE2eStageInput) -> Result<(), &'static str> {
    let root = std::env::var_os("ACM_OS_E2E_ROOT")
        .map(std::path::PathBuf::from)
        .ok_or("desktop_e2e_unavailable")?;
    std::fs::write(root.join("desktop-e2e-stage.txt"), input.stage)
        .map_err(|_| "desktop_e2e_write_failed")
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayReplanEntryDto {
    existing_entry_id: Option<String>,
    problem_id: String,
    review_attempt_id: Option<String>,
    lane: String,
    reason: String,
    planning_cost_minutes: u32,
    origin: String,
    status: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayReplanPreviewDto {
    expected_snapshot: TodaySnapshotDto,
    proposed_budget_minutes: u32,
    proposed_planned_minutes: u32,
    proposed_over_budget_minutes: u32,
    proposed_review_only_streak: u8,
    entries: Vec<TodayReplanEntryDto>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayExtraSuggestionDto {
    problem_id: String,
    contest_id: u64,
    problem_index: String,
    problem_title: String,
    review_attempt_id: Option<String>,
    lane: String,
    reason: String,
    planning_cost_minutes: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayExtraSuggestionsPreviewDto {
    expected_snapshot: TodaySnapshotDto,
    remaining_budget_minutes: u32,
    suggestions: Vec<TodayExtraSuggestionDto>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemLifecycleCommandInput {
    contest_id: u64,
    index: String,
    action: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFocusInput {
    attempt_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealReviewHelpInput {
    attempt_id: String,
    level: u8,
    impact_acknowledged: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteReviewInput {
    attempt_id: String,
    final_ac: bool,
    first_submission_result: String,
    first_submission_other: Option<String>,
    final_result: String,
    final_result_other: Option<String>,
    total_submissions: u32,
    idea_independent: bool,
    implementation_independent: bool,
    debug_independence: String,
    external_help: String,
    failure_reasons: Vec<ReviewFailureReasonInput>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFailureReasonInput {
    code: String,
    other_text: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoidReviewInput {
    attempt_id: String,
    reason: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewAttemptDto {
    attempt_id: String,
    contest_id: u64,
    index: String,
    attempt_type: &'static str,
    scheduled_due_local_date: String,
    started_early: bool,
    judgement_rule_version: u32,
    started_at_utc: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFocusDto {
    attempt: ReviewAttemptDto,
    title: String,
    source_url: String,
    statement_sanitized_html: String,
    statement_assets: Vec<LocalStatementAssetDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHelpItemDto {
    level: u8,
    consequence: &'static str,
    available: bool,
    revealed_at_utc: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHelpDrawerDto {
    attempt_id: String,
    items: Vec<ReviewHelpItemDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealedReviewHelpDto {
    event_id: String,
    attempt_id: String,
    level: u8,
    consequence: &'static str,
    title: String,
    content_markdown: String,
    source_digest: String,
    revealed_at_utc: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedReviewAttemptDto {
    attempt: ReviewAttemptDto,
    judgement: &'static str,
    evidence_codes: Vec<String>,
    failure_reasons: Vec<ReviewFailureReasonDto>,
    completed_at_utc: String,
    completed_local_date: String,
    lifecycle: ProblemLifecycleStateDto,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFailureReasonDto {
    code: &'static str,
    other_text: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCompletionFactsDto {
    final_ac: bool,
    first_submission_result: String,
    final_result: String,
    total_submissions: u32,
    idea_independent: bool,
    implementation_independent: bool,
    debug_independence: &'static str,
    external_help: &'static str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHistoryItemDto {
    attempt: ReviewAttemptDto,
    status: &'static str,
    judgement: Option<&'static str>,
    completion_facts: Option<ReviewCompletionFactsDto>,
    evidence_codes: Vec<String>,
    failure_reasons: Vec<ReviewFailureReasonDto>,
    help_levels: Vec<u8>,
    completed_at_utc: Option<String>,
    completed_local_date: Option<String>,
    void_reason: Option<String>,
    voided_at_utc: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewHistoryDto {
    contest_id: u64,
    index: String,
    historical_best_review: Option<&'static str>,
    mastery: ProblemMasteryProjectionDto,
    attempts: Vec<ReviewHistoryItemDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemMasteryProjectionDto {
    current: ProblemMasteryEvidenceDto,
    historical_thoroughly_digested: bool,
    first_thoroughly_digested_local_date: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemMasteryEvidenceDto {
    recalls_problem: bool,
    multiple_solutions_clear: bool,
    knowledge_understood: bool,
    implementation_fluent: bool,
    can_adapt_or_create: bool,
    transfer_solved_independently: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProblemMasteryEvidenceInput {
    contest_id: u64,
    index: String,
    evidence: ProblemMasteryEvidenceDto,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalNoteBindingDto {
    vault_relative_path: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalNoteRelocationCandidateDto {
    vault_relative_path: String,
    occupied: bool,
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
    Ready {
        #[serde(rename = "sanitizedHtml")]
        sanitized_html: String,
    },
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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualContestInput {
    contest_id: u64,
    title: String,
    source_url: String,
    starts_at_utc: Option<String>,
    problems: Vec<ManualProblemInput>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualProblemInput {
    index: String,
    title: String,
    source_url: String,
    statement_text: String,
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
    #[cfg(feature = "desktop-e2e")]
    let result = import_codeforces_contest(database.inner(), &DesktopE2eContestSource, contest)
        .await
        .map_err(contest_import_error_code)?;
    #[cfg(not(feature = "desktop-e2e"))]
    let result = {
        let source = acm_os_infrastructure::codeforces::CodeforcesHttpAdapter::new()
            .map_err(|_| "adapter_unavailable")?;
        import_codeforces_contest(database.inner(), &source, contest)
            .await
            .map_err(contest_import_error_code)?
    };
    Ok(ContestImportRunDto {
        import_status: match result.persisted.status {
            acm_os_application::ContestImportStatus::Incomplete => "incomplete",
            acm_os_application::ContestImportStatus::Complete => "complete",
        },
        missing_snapshot_problems: result
            .persisted
            .missing_snapshot_problems
            .into_iter()
            .map(|problem| format!("{}{}", problem.contest().contest_id(), problem.index()))
            .collect(),
        failed_snapshot_problems: result
            .failed_snapshot_problems
            .into_iter()
            .map(|problem| format!("{}{}", problem.contest().contest_id(), problem.index()))
            .collect(),
    })
}

#[tauri::command]
pub async fn import_manual_codeforces_contest(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ManualContestInput,
) -> Result<ContestImportRunDto, &'static str> {
    use acm_os_application::ContestImportPort;
    let problems = input
        .problems
        .into_iter()
        .map(|item| acm_os_application::ManualProblemDraft {
            index: item.index,
            title: item.title,
            source_url: item.source_url,
            statement_text: item.statement_text,
        })
        .collect::<Vec<_>>();
    let plan = acm_os_application::build_manual_codeforces_contest(
        input.contest_id,
        &input.title,
        &input.source_url,
        input.starts_at_utc,
        &problems,
    )
    .map_err(manual_contest_error_code)?;
    let mut persisted =
        database
            .persist_manifest(&plan.manifest)
            .await
            .map_err(|error| match error {
                acm_os_application::ContestImportPersistenceError::ManifestConflict => {
                    "manual_manifest_conflict"
                }
                _ => "manual_import_unavailable",
            })?;
    for snapshot in plan.snapshots_for_missing(&persisted.missing_snapshot_problems) {
        persisted = database
            .persist_first_snapshot(snapshot)
            .await
            .map_err(|_| "manual_import_unavailable")?;
    }
    Ok(ContestImportRunDto {
        import_status: match persisted.status {
            acm_os_application::ContestImportStatus::Complete => "complete",
            acm_os_application::ContestImportStatus::Incomplete => "incomplete",
        },
        missing_snapshot_problems: persisted
            .missing_snapshot_problems
            .iter()
            .map(|problem| format!("{}{}", problem.contest().contest_id(), problem.index()))
            .collect(),
        failed_snapshot_problems: Vec::new(),
    })
}

fn manual_contest_error_code(error: acm_os_application::ManualContestError) -> &'static str {
    match error {
        acm_os_application::ManualContestError::InvalidIdentity => "manual_invalid_identity",
        acm_os_application::ManualContestError::InvalidInput => "manual_invalid_input",
        acm_os_application::ManualContestError::DuplicateProblem => "manual_duplicate_problem",
    }
}

fn contest_import_error_code(error: acm_os_application::ContestImportSourceError) -> &'static str {
    match error {
        acm_os_application::ContestImportSourceError::Unavailable => "import_unavailable",
        acm_os_application::ContestImportSourceError::InvalidRemoteData => "invalid_remote_data",
    }
}

#[cfg(feature = "desktop-e2e")]
struct DesktopE2eContestSource;

#[cfg(feature = "desktop-e2e")]
impl acm_os_application::ContestImportSource for DesktopE2eContestSource {
    async fn fetch_manifest(
        &self,
        contest: &acm_os_domain::CodeforcesContestIdentity,
    ) -> Result<acm_os_application::ContestImportDraft, acm_os_application::ContestImportSourceError>
    {
        if contest.contest_id() != 1979 {
            return Err(acm_os_application::ContestImportSourceError::InvalidRemoteData);
        }
        let problem_a = acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "A")
            .map_err(|_| acm_os_application::ContestImportSourceError::InvalidRemoteData)?;
        let problem_b = acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "B")
            .map_err(|_| acm_os_application::ContestImportSourceError::InvalidRemoteData)?;
        let problem_c = acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), "C")
            .map_err(|_| acm_os_application::ContestImportSourceError::InvalidRemoteData)?;
        Ok(acm_os_application::ContestImportDraft {
            contest: contest.clone(),
            title: "Desktop E2E Contest".to_owned(),
            source_url: "https://codeforces.com/contest/1979".to_owned(),
            starts_at_utc: Some("2026-08-01T00:00:00Z".to_owned()),
            slots: vec![
                acm_os_application::ContestProblemSlotDraft {
                    ordinal: 1,
                    problem: problem_a,
                    title: "Desktop E2E Problem".to_owned(),
                    rating: Some(800),
                    source_url: "https://codeforces.com/contest/1979/problem/A".to_owned(),
                },
                acm_os_application::ContestProblemSlotDraft {
                    ordinal: 2,
                    problem: problem_b,
                    title: "Desktop E2E Study Problem".to_owned(),
                    rating: Some(900),
                    source_url: "https://codeforces.com/contest/1979/problem/B".to_owned(),
                },
                acm_os_application::ContestProblemSlotDraft {
                    ordinal: 3,
                    problem: problem_c,
                    title: "Desktop E2E Extra Study Problem".to_owned(),
                    rating: Some(1000),
                    source_url: "https://codeforces.com/contest/1979/problem/C".to_owned(),
                },
            ],
        })
    }

    async fn fetch_snapshot(
        &self,
        problem: &acm_os_domain::CodeforcesProblemIdentity,
    ) -> Result<
        acm_os_application::StatementSnapshotDraft,
        acm_os_application::ContestImportSourceError,
    > {
        if problem.contest().contest_id() != 1979 || !matches!(problem.index(), "A" | "B" | "C") {
            return Err(acm_os_application::ContestImportSourceError::InvalidRemoteData);
        }
        let html = "<div class=\"problem-statement\"><p>Desktop E2E statement.</p></div>";
        Ok(acm_os_application::StatementSnapshotDraft {
            problem: problem.clone(),
            source_html: html.to_owned(),
            sanitized_html: html.to_owned(),
            assets: Vec::new(),
        })
    }
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
pub async fn contest_library_list_families(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<Vec<ContestLibraryFamilyDto>, &'static str> {
    acm_os_application::list_families(database.inner())
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| ContestLibraryFamilyDto {
                    family_id: item.family_id,
                    display_name: item.display_name,
                })
                .collect()
        })
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_create_family(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryFamilyInput,
) -> Result<ContestLibraryFamilyDto, &'static str> {
    acm_os_application::create_family(database.inner(), &input.display_name)
        .await
        .map(|item| ContestLibraryFamilyDto {
            family_id: item.family_id,
            display_name: item.display_name,
        })
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_rename_family(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryFamilyRenameInput,
) -> Result<ContestLibraryFamilyDto, &'static str> {
    acm_os_application::rename_family(database.inner(), input.family_id, &input.display_name)
        .await
        .map(|item| ContestLibraryFamilyDto {
            family_id: item.family_id,
            display_name: item.display_name,
        })
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_list_series(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryFamilyIdInput,
) -> Result<Vec<ContestLibrarySeriesDto>, &'static str> {
    acm_os_application::list_series(database.inner(), input.family_id)
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| ContestLibrarySeriesDto {
                    series_id: item.series_id,
                    family_id: item.family_id,
                    display_name: item.display_name,
                })
                .collect()
        })
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_create_series(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibrarySeriesInput,
) -> Result<ContestLibrarySeriesDto, &'static str> {
    acm_os_application::create_series(database.inner(), input.family_id, &input.display_name)
        .await
        .map(|item| ContestLibrarySeriesDto {
            series_id: item.series_id,
            family_id: item.family_id,
            display_name: item.display_name,
        })
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_rename_series(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibrarySeriesRenameInput,
) -> Result<ContestLibrarySeriesDto, &'static str> {
    acm_os_application::rename_series(database.inner(), input.series_id, &input.display_name)
        .await
        .map(|item| ContestLibrarySeriesDto {
            series_id: item.series_id,
            family_id: item.family_id,
            display_name: item.display_name,
        })
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_list_years(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryYearsInput,
) -> Result<Vec<Option<u32>>, &'static str> {
    let series = contest_library_series_filter(input.series)?;
    acm_os_application::list_years(database.inner(), input.family_id, series)
        .await
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_list_contest_placements(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestDetailInput,
) -> Result<Vec<ContestLibraryPlacementDto>, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    acm_os_application::list_contest_placements(database.inner(), &contest)
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| ContestLibraryPlacementDto {
                    placement_id: item.placement_id,
                    family_id: item.family_id,
                    family_name: item.family_name,
                    series_id: item.series_id,
                    series_name: item.series_name,
                    year: item.year,
                    ordinal: item.ordinal,
                })
                .collect()
        })
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_create_placement(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryPlacementInput,
) -> Result<ContestLibraryPlacementDto, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    acm_os_application::create_placement(
        database.inner(),
        acm_os_application::CreateContestPlacement {
            contest,
            family_id: input.family_id,
            series_id: input.series_id,
            year: input.year,
            ordinal: input.ordinal,
        },
    )
    .await
    .map(contest_library_placement_dto)
    .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_update_placement(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryPlacementUpdateInput,
) -> Result<ContestLibraryPlacementDto, &'static str> {
    acm_os_application::update_placement(
        database.inner(),
        acm_os_application::UpdateContestPlacement {
            placement_id: input.placement_id,
            family_id: input.family_id,
            series_id: input.series_id,
            year: input.year,
            ordinal: input.ordinal,
        },
    )
    .await
    .map(contest_library_placement_dto)
    .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_remove_placement(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryPlacementRemoveInput,
) -> Result<(), &'static str> {
    acm_os_application::remove_placement(database.inner(), input.placement_id)
        .await
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_library_list_contests(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestLibraryListInput,
) -> Result<Vec<ContestShelfItemDto>, &'static str> {
    let scope = contest_library_scope(input.scope)?;
    let archive = contest_library_archive_filter(input.archive);
    acm_os_application::list_library_contests(database.inner(), scope, archive)
        .await
        .map(|items| items.into_iter().map(contest_shelf_item_dto).collect())
        .map_err(contest_library_error_code)
}

#[tauri::command]
pub async fn contest_detail(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestDetailInput,
) -> Result<ContestDetailDto, &'static str> {
    use acm_os_application::ContestReadPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    database
        .contest_detail(&contest)
        .await
        .map(contest_detail_dto)
        .map_err(contest_read_error_code)
}

#[tauri::command]
pub async fn complete_contest_facts(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: CompleteContestFactsInput,
) -> Result<ContestDetailDto, &'static str> {
    use acm_os_application::ContestFactsPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    let facts = input
        .problems
        .into_iter()
        .map(|item| {
            Ok(acm_os_application::ContestProblemFactInput {
                problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), item.index)
                    .map_err(|_| "invalid_problem_identity")?,
                final_contest_result: parse_contest_final_result_dto(&item.final_contest_result)?,
                upsolve_decision: parse_contest_upsolve_decision_dto(&item.upsolve_decision)?,
            })
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    database
        .complete_contest_facts(&contest, &facts)
        .await
        .map(contest_detail_dto)
        .map_err(contest_facts_error_code)
}

#[tauri::command]
pub async fn correct_contest_problem_facts(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: CorrectContestProblemFactsInput,
) -> Result<ContestDetailDto, &'static str> {
    use acm_os_application::ContestCorrectionPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    let correction = acm_os_application::ContestProblemCorrectionInput {
        problem: acm_os_domain::CodeforcesProblemIdentity::new(contest.clone(), input.index)
            .map_err(|_| "invalid_problem_identity")?,
        final_contest_result: parse_contest_final_result_dto(&input.final_contest_result)?,
        upsolve_decision: parse_contest_upsolve_decision_dto(&input.upsolve_decision)?,
    };
    database
        .correct_contest_problem_facts(&contest, &correction)
        .await
        .map(contest_detail_dto)
        .map_err(contest_correction_error_code)
}

#[tauri::command]
pub async fn set_contest_archived(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestArchiveInput,
) -> Result<ContestDetailDto, &'static str> {
    use acm_os_application::ContestManagementPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    database
        .set_contest_archived(&contest, input.archived)
        .await
        .map(contest_detail_dto)
        .map_err(contest_management_error_code)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContestArchiveInput {
    contest_id: u64,
    archived: bool,
}

#[tauri::command]
pub async fn preview_delete_contest(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestDetailInput,
) -> Result<ContestDeletePreviewDto, &'static str> {
    use acm_os_application::ContestManagementPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    database
        .preview_delete_contest(&contest)
        .await
        .map(contest_delete_preview_dto)
        .map_err(contest_management_error_code)
}

#[tauri::command]
pub async fn delete_contest(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestDetailInput,
) -> Result<ContestDeletePreviewDto, &'static str> {
    use acm_os_application::ContestManagementPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    database
        .delete_contest(&contest)
        .await
        .map(contest_delete_preview_dto)
        .map_err(contest_management_error_code)
}

#[tauri::command]
pub async fn preview_contest_ai_analysis(
    input: ContestAiAnalysisInput,
) -> Result<ContestAiAnalysisPreviewDto, &'static str> {
    let _ = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    acm_os_application::preview_contest_ai_analysis(&input.raw_text)
        .map(contest_ai_analysis_preview_dto)
        .map_err(contest_ai_analysis_error_code)
}

#[tauri::command]
pub async fn save_contest_ai_analysis(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ContestAiAnalysisInput,
) -> Result<ContestDetailDto, &'static str> {
    use acm_os_application::ContestAiAnalysisPort;
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_contest_identity")?;
    let preview = database
        .preview_contest_ai_analysis(&input.raw_text)
        .await
        .map_err(contest_ai_analysis_error_code)?;
    database
        .save_contest_ai_analysis(&contest, &preview)
        .await
        .map(contest_detail_dto)
        .map_err(contest_ai_analysis_error_code)
}

#[tauri::command]
pub async fn lightweight_problems(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<Vec<LightweightProblemItemDto>, ()> {
    use acm_os_application::ContestReadPort;
    database
        .list_lightweight_problems()
        .await
        .map(|items| {
            items
                .into_iter()
                .map(lightweight_problem_item_dto)
                .collect()
        })
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
    let today = command_local_date().ok();
    let detail = database
        .lightweight_problem_detail(&problem)
        .await
        .map_err(|error| match error {
            acm_os_application::ContestReadError::NotFound => "problem_not_found",
            acm_os_application::ContestReadError::Unavailable => "problem_unavailable",
        })?;
    use acm_os_application::ReviewAttemptPort;
    let review_in_progress = database
        .load_in_progress_review_attempt(&problem)
        .await
        .map_err(acm_os_application::ReviewAttemptError::code)?
        .is_some();
    Ok(lightweight_problem_detail_dto(
        detail,
        today,
        review_in_progress,
    ))
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
    let problem = codeforces_problem_identity(input.contest_id, input.index)?;
    let action = parse_problem_lifecycle_action(&input.action)?;
    let today = command_local_date().map_err(|_| "local_calendar_unavailable")?;
    acm_os_application::transition_problem_lifecycle(database.inner(), &problem, action, today)
        .await
        .map(problem_lifecycle_state_dto)
        .map_err(acm_os_application::ProblemLifecycleError::code)
}

fn codeforces_problem_identity(
    contest_id: u64,
    index: String,
) -> Result<acm_os_domain::ProblemIdentity, &'static str> {
    let legacy_contest = acm_os_domain::CodeforcesContestIdentity::new(contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let legacy_problem = acm_os_domain::CodeforcesProblemIdentity::new(legacy_contest, index)
        .map_err(|_| "invalid_problem_identity")?;
    let platform = acm_os_domain::PlatformKey::new(legacy_problem.contest().platform())
        .map_err(|_| "invalid_problem_identity")?;
    let contest_key = acm_os_domain::ExternalContestKey::new(
        legacy_problem.contest().contest_id().to_string(),
    )
    .map_err(|_| "invalid_problem_identity")?;
    acm_os_domain::ProblemIdentity::new(
        acm_os_domain::ContestIdentity::new(platform, contest_key),
        legacy_problem.index(),
    )
    .map_err(|_| "invalid_problem_identity")
}

#[tauri::command]
pub async fn today_snapshot(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: TodayLoadInput,
) -> Result<Option<TodaySnapshotDto>, &'static str> {
    let today = command_local_date().map_err(|_| "local_calendar_unavailable")?;
    use acm_os_application::TodaySnapshotPort;
    let existing = database
        .inner()
        .load_today_snapshot(today)
        .await
        .map_err(today_error_code)?;
    let budget_minutes = if existing.is_some() {
        input.budget_minutes.unwrap_or(0)
    } else if let Some(minutes) = input.budget_minutes {
        minutes
    } else {
        use acm_os_application::WeeklyAcmBudgetPort;
        let schedule = database
            .inner()
            .load_weekly_acm_budget()
            .await
            .map_err(today_error_code)?;
        let Some(minutes) = acm_os_application::weekly_acm_budget_for_date(&schedule, today) else {
            return Ok(None);
        };
        minutes
    };
    acm_os_application::load_or_generate_today_snapshot(database.inner(), today, budget_minutes)
        .await
        .map(today_snapshot_dto)
        .map(Some)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn weekly_acm_budget(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<WeeklyAcmBudgetDto, &'static str> {
    acm_os_application::load_weekly_acm_budget(database.inner())
        .await
        .map(weekly_acm_budget_dto)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn save_weekly_acm_budget(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    schedule: WeeklyAcmBudgetDto,
) -> Result<WeeklyAcmBudgetDto, &'static str> {
    let schedule = parse_weekly_acm_budget(schedule);
    acm_os_application::save_weekly_acm_budget(database.inner(), &schedule)
        .await
        .map(weekly_acm_budget_dto)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn reorder_today(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: TodayReorderInput,
) -> Result<TodaySnapshotDto, &'static str> {
    acm_os_application::reorder_today_snapshot(
        database.inner(),
        &input.plan_id,
        &input.ordered_entry_ids,
    )
    .await
    .map(today_snapshot_dto)
    .map_err(today_error_code)
}

#[tauri::command]
pub async fn complete_today_entry(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: TodayDoneInput,
) -> Result<TodaySnapshotDto, &'static str> {
    acm_os_application::complete_today_entry(database.inner(), &input.plan_id, &input.entry_id)
        .await
        .map(today_snapshot_dto)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn preview_today_replan(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: TodayReplanInput,
) -> Result<TodayReplanPreviewDto, &'static str> {
    let today = command_local_date().map_err(|_| "local_calendar_unavailable")?;
    acm_os_application::preview_today_replan(database.inner(), today, input.budget_minutes)
        .await
        .map(today_replan_preview_dto)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn apply_today_replan(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    preview: TodayReplanPreviewDto,
) -> Result<TodaySnapshotDto, &'static str> {
    let preview = parse_today_replan_preview(preview)?;
    acm_os_application::apply_today_replan(database.inner(), &preview)
        .await
        .map(today_snapshot_dto)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn today_extra_suggestions(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<TodayExtraSuggestionsPreviewDto, &'static str> {
    let today = command_local_date().map_err(|_| "local_calendar_unavailable")?;
    acm_os_application::preview_today_extra_suggestions(database.inner(), today)
        .await
        .map(today_extra_suggestions_preview_dto)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn accept_today_extra_suggestion(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: TodayAcceptSuggestionInput,
) -> Result<TodaySnapshotDto, &'static str> {
    let preview = parse_today_extra_suggestions_preview(input.preview)?;
    acm_os_application::accept_today_extra_suggestion(database.inner(), &preview, &input.problem_id)
        .await
        .map(today_snapshot_dto)
        .map_err(today_error_code)
}

#[tauri::command]
pub async fn start_or_resume_review(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<ReviewAttemptDto, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    let today = command_local_date().map_err(|_| "local_calendar_unavailable")?;
    acm_os_application::start_or_resume_review(database.inner(), &problem, today)
        .await
        .map(review_attempt_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn review_focus(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ReviewFocusInput,
) -> Result<ReviewFocusDto, &'static str> {
    if input.attempt_id.len() != 36 {
        return Err("invalid_review_attempt_identity");
    }
    acm_os_application::review_focus(database.inner(), &input.attempt_id)
        .await
        .map(review_focus_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn review_help_drawer(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ReviewFocusInput,
) -> Result<ReviewHelpDrawerDto, &'static str> {
    if input.attempt_id.len() != 36 {
        return Err("invalid_review_attempt_identity");
    }
    acm_os_application::review_help_drawer(database.inner(), &input.attempt_id)
        .await
        .map(review_help_drawer_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn reveal_review_help(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: RevealReviewHelpInput,
) -> Result<RevealedReviewHelpDto, &'static str> {
    if input.attempt_id.len() != 36 {
        return Err("invalid_review_attempt_identity");
    }
    let level = acm_os_domain::ReviewHelpLevel::from_number(input.level)
        .ok_or("invalid_review_help_level")?;
    acm_os_application::reveal_review_help(
        database.inner(),
        &input.attempt_id,
        level,
        input.impact_acknowledged,
    )
    .await
    .map(revealed_review_help_dto)
    .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn complete_review(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: CompleteReviewInput,
) -> Result<CompletedReviewAttemptDto, &'static str> {
    if input.attempt_id.len() != 36 {
        return Err("invalid_review_attempt_identity");
    }
    let completion = parse_review_completion_input(input)?;
    let completed_on = command_local_date().map_err(|_| "local_calendar_unavailable")?;
    acm_os_application::complete_review(database.inner(), &completion.0, completion.1, completed_on)
        .await
        .map(completed_review_attempt_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn void_review(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: VoidReviewInput,
) -> Result<ReviewHistoryItemDto, &'static str> {
    if input.attempt_id.len() != 36 {
        return Err("invalid_review_attempt_identity");
    }
    acm_os_application::void_review(database.inner(), &input.attempt_id, &input.reason)
        .await
        .map(review_history_item_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn review_attempt_history(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ReviewFocusInput,
) -> Result<ReviewHistoryItemDto, &'static str> {
    if input.attempt_id.len() != 36 {
        return Err("invalid_review_attempt_identity");
    }
    acm_os_application::review_attempt_history_item(database.inner(), &input.attempt_id)
        .await
        .map(review_history_item_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn review_history(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<ReviewHistoryDto, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    acm_os_application::review_history(database.inner(), &problem)
        .await
        .map(review_history_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn update_problem_mastery_evidence(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: UpdateProblemMasteryEvidenceInput,
) -> Result<ProblemMasteryProjectionDto, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(input.contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    let problem = acm_os_domain::CodeforcesProblemIdentity::new(contest, input.index)
        .map_err(|_| "invalid_problem_identity")?;
    let today = command_local_date().map_err(|_| "local_calendar_unavailable")?;
    let evidence = acm_os_domain::ProblemMasteryEvidence {
        recalls_problem: input.evidence.recalls_problem,
        multiple_solutions_clear: input.evidence.multiple_solutions_clear,
        knowledge_understood: input.evidence.knowledge_understood,
        implementation_fluent: input.evidence.implementation_fluent,
        can_adapt_or_create: input.evidence.can_adapt_or_create,
        transfer_solved_independently: input.evidence.transfer_solved_independently,
    };
    acm_os_application::update_problem_mastery_evidence(database.inner(), &problem, evidence, today)
        .await
        .map(problem_mastery_projection_dto)
        .map_err(acm_os_application::ReviewAttemptError::code)
}

#[tauri::command]
pub async fn delete_personal_note(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<ProblemLifecycleStateDto, &'static str> {
    let problem = codeforces_problem_identity(input.contest_id, input.index)?;
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
    let problem = codeforces_problem_identity(input.contest_id, input.index)?;
    database
        .read_personal_note_projection(&problem)
        .await
        .map(personal_note_read_state_dto)
        .map_err(acm_os_application::PersonalNoteReadError::code)
}

#[tauri::command]
pub async fn personal_note_relocation_candidates(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<Vec<PersonalNoteRelocationCandidateDto>, &'static str> {
    let problem = codeforces_problem_identity(input.contest_id, input.index)?;
    acm_os_application::personal_note_relocation_candidates(&*database, &problem)
        .await
        .map(|candidates| {
            candidates
                .into_iter()
                .map(|candidate| PersonalNoteRelocationCandidateDto {
                    vault_relative_path: candidate.vault_relative_path,
                    occupied: candidate.occupied,
                })
                .collect()
        })
        .map_err(acm_os_application::PersonalNoteBindingRepairError::code)
}

#[tauri::command]
pub async fn rebind_personal_note(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: RebindPersonalNoteInput,
) -> Result<PersonalNoteBindingDto, &'static str> {
    let problem = codeforces_problem_identity(input.contest_id, input.index)?;
    acm_os_application::rebind_personal_note(&*database, &problem, input.vault_relative_path)
        .await
        .map(personal_note_binding_dto)
        .map_err(acm_os_application::PersonalNoteBindingRepairError::code)
}

#[tauri::command]
pub async fn confirm_personal_note_deleted(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<ProblemLifecycleStateDto, &'static str> {
    let problem = codeforces_problem_identity(input.contest_id, input.index)?;
    acm_os_application::confirm_personal_note_deleted(&*database, &problem)
        .await
        .map(problem_lifecycle_state_dto)
        .map_err(acm_os_application::PersonalNoteBindingRepairError::code)
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

    let problem = codeforces_problem_identity(input.contest_id, input.index)?;
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
    let uri = obsidian_open_uri(workspace.active_vault_path(), &binding.vault_relative_path)?;
    app.opener()
        .open_url(uri, None::<&str>)
        .map_err(|_| "obsidian_open_failed")
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenOriginalOjInput {
    pub url: String,
}

#[tauri::command]
pub fn open_original_oj(
    app: tauri::AppHandle,
    input: OpenOriginalOjInput,
) -> Result<(), &'static str> {
    use tauri_plugin_opener::OpenerExt;

    let url = url::Url::parse(input.url.trim()).map_err(|_| "unsafe_external_url")?;
    let host = url.host_str().ok_or("unsafe_external_url")?;
    if url.scheme() != "https" || !matches!(host, "codeforces.com" | "www.codeforces.com") {
        return Err("unsafe_external_url");
    }
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|_| "external_open_failed")
}

#[tauri::command]
pub async fn open_knowledge_in_obsidian(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    app: tauri::AppHandle,
    input: KnowledgeNodeInput,
) -> Result<(), &'static str> {
    use acm_os_application::WorkspaceConfigurationPort;
    use tauri_plugin_opener::OpenerExt;

    let detail =
        acm_os_application::load_knowledge_detail(database.inner(), &input.knowledge_node_id)
            .await
            .map_err(knowledge_index_error_code)?;
    let workspace = database
        .load_workspace_configuration()
        .await
        .map_err(|_| "workspace_unavailable")?
        .ok_or("workspace_unavailable")?;
    let uri = obsidian_open_uri(
        workspace.active_vault_path(),
        &detail.node.vault_relative_path,
    )?;
    app.opener()
        .open_url(uri, None::<&str>)
        .map_err(|_| "obsidian_open_failed")
}

#[tauri::command]
pub async fn knowledge_index(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeSearchInput,
) -> Result<KnowledgeIndexDto, &'static str> {
    let projection = acm_os_application::rebuild_knowledge_index(database.inner())
        .await
        .map_err(knowledge_index_error_code)?;
    let nodes = if input.query.trim().is_empty() {
        projection.nodes
    } else {
        acm_os_application::search_knowledge_index(database.inner(), &input.query)
            .await
            .map_err(knowledge_index_error_code)?
    };
    Ok(KnowledgeIndexDto {
        nodes: nodes.into_iter().map(knowledge_node_dto).collect(),
        location_anomalies: projection
            .location_anomalies
            .into_iter()
            .map(knowledge_node_dto)
            .collect(),
        identity_conflicts: projection
            .identity_conflicts
            .into_iter()
            .map(|item| KnowledgeIdentityConflictDto {
                historical_knowledge_node_id: item.historical_knowledge_node_id,
                display_name: item.display_name,
                candidate_vault_relative_path: item.candidate_vault_relative_path,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn knowledge_relocation_candidates(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeNodeInput,
) -> Result<Vec<KnowledgeRelocationCandidateDto>, &'static str> {
    acm_os_application::knowledge_relocation_candidates(&*database, &input.knowledge_node_id)
        .await
        .map(|items| {
            items
                .into_iter()
                .map(|item| KnowledgeRelocationCandidateDto {
                    vault_relative_path: item.vault_relative_path,
                    occupied: item.occupied,
                })
                .collect()
        })
        .map_err(acm_os_application::KnowledgeBindingRepairError::code)
}

#[tauri::command]
pub async fn rebind_knowledge_node(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: RebindKnowledgeNodeInput,
) -> Result<KnowledgeNodeDto, &'static str> {
    acm_os_application::rebind_knowledge_node(
        &*database,
        &input.knowledge_node_id,
        input.vault_relative_path,
    )
    .await
    .map(knowledge_node_dto)
    .map_err(acm_os_application::KnowledgeBindingRepairError::code)
}

#[tauri::command]
pub async fn confirm_knowledge_markdown_deleted(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeNodeInput,
) -> Result<(), &'static str> {
    acm_os_application::confirm_knowledge_markdown_deleted(&*database, &input.knowledge_node_id)
        .await
        .map_err(acm_os_application::KnowledgeBindingRepairError::code)
}

#[tauri::command]
pub async fn resolve_knowledge_identity_conflict(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: ResolveKnowledgeIdentityConflictInput,
) -> Result<KnowledgeNodeDto, &'static str> {
    acm_os_application::resolve_knowledge_identity_conflict(
        &*database,
        &input.historical_knowledge_node_id,
        &input.candidate_vault_relative_path,
        input.restore_old_identity,
    )
    .await
    .map(knowledge_node_dto)
    .map_err(acm_os_application::KnowledgeBindingRepairError::code)
}

#[tauri::command]
pub async fn knowledge_detail(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeNodeInput,
) -> Result<KnowledgeDetailDto, &'static str> {
    acm_os_application::load_knowledge_detail(database.inner(), &input.knowledge_node_id)
        .await
        .map(knowledge_detail_dto)
        .map_err(knowledge_index_error_code)
}

#[tauri::command]
pub async fn confirm_knowledge_understanding(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeUnderstandingInput,
) -> Result<KnowledgeUnderstandingDto, &'static str> {
    let level = knowledge_understanding_level(&input.level)?;
    let date = command_local_date().map_err(|_| "local_date_unavailable")?;
    acm_os_application::confirm_knowledge_understanding(
        database.inner(),
        &input.knowledge_node_id,
        level,
        date,
    )
    .await
    .map(knowledge_understanding_dto)
    .map_err(knowledge_index_error_code)
}

#[tauri::command]
pub async fn knowledge_reevaluation_suggestion(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeNodeInput,
) -> Result<KnowledgeReevaluationSuggestionDto, &'static str> {
    acm_os_application::load_knowledge_reevaluation_suggestion(
        database.inner(),
        &input.knowledge_node_id,
    )
    .await
    .map(|v| KnowledgeReevaluationSuggestionDto {
        knowledge_node_id: v.knowledge_node_id,
        should_suggest: v.should_suggest,
        qualifying_problem_count: v.qualifying_problem_count,
    })
    .map_err(knowledge_index_error_code)
}

#[tauri::command]
pub async fn knowledge_candidates(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: LightweightProblemDetailInput,
) -> Result<Vec<KnowledgeCandidateDto>, &'static str> {
    let problem_index = input.index.clone();
    let generic_problem = codeforces_problem_identity(input.contest_id, input.index)?;
    let items = acm_os_application::list_knowledge_candidates(database.inner(), &generic_problem)
        .await
        .map_err(knowledge_candidate_error_code)?;
    let index = acm_os_application::rebuild_knowledge_index(database.inner())
        .await
        .map_err(knowledge_index_error_code)?;
    Ok(items
        .into_iter()
        .map(|item| {
            knowledge_candidate_read_dto(item, input.contest_id, &problem_index, &index.nodes)
        })
        .collect())
}

#[tauri::command]
pub async fn register_knowledge_candidate(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeCandidateRegisterInput,
) -> Result<KnowledgeCandidateDto, &'static str> {
    let problem = knowledge_candidate_problem(input.contest_id, input.index)?;
    acm_os_application::register_knowledge_candidate(
        database.inner(),
        &problem,
        &input.fingerprint,
        &input.target_ref,
    )
    .await
    .map(|item| knowledge_candidate_dto(item, &[]))
    .map_err(knowledge_candidate_error_code)
}

#[tauri::command]
pub async fn set_knowledge_candidate_disposition(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeCandidateDispositionInput,
) -> Result<KnowledgeCandidateDto, &'static str> {
    let problem = knowledge_candidate_problem(input.contest_id, input.index)?;
    let disposition = match input.disposition.as_str() {
        "pending" => acm_os_application::KnowledgeCandidateDisposition::Pending,
        "acceptedIntent" => acm_os_application::KnowledgeCandidateDisposition::AcceptedIntent,
        "ignored" => acm_os_application::KnowledgeCandidateDisposition::Ignored,
        _ => return Err("invalid_candidate_disposition"),
    };
    acm_os_application::set_knowledge_candidate_disposition(
        database.inner(),
        &problem,
        &input.fingerprint,
        disposition,
    )
    .await
    .map(|item| knowledge_candidate_dto(item, &[]))
    .map_err(knowledge_candidate_error_code)
}

#[tauri::command]
pub async fn accept_existing_knowledge_candidate(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: KnowledgeCandidateAcceptInput,
) -> Result<AcceptedKnowledgeCandidateDto, &'static str> {
    let problem = knowledge_candidate_problem(input.contest_id, input.index)?;
    acm_os_application::accept_existing_knowledge_candidate(
        database.inner(),
        &problem,
        &input.fingerprint,
        &input.knowledge_node_id,
    )
    .await
    .map(|value| AcceptedKnowledgeCandidateDto {
        knowledge_node_id: value.knowledge_node_id,
        target_ref: value.target_ref,
    })
    .map_err(knowledge_candidate_error_code)
}

fn knowledge_candidate_problem(
    contest_id: u64,
    index: String,
) -> Result<acm_os_domain::CodeforcesProblemIdentity, &'static str> {
    let contest = acm_os_domain::CodeforcesContestIdentity::new(contest_id)
        .map_err(|_| "invalid_problem_identity")?;
    acm_os_domain::CodeforcesProblemIdentity::new(contest, index)
        .map_err(|_| "invalid_problem_identity")
}

fn knowledge_candidate_read_dto(
    value: acm_os_application::KnowledgeCandidateReadProjection,
    contest_id: u64,
    problem_index: &str,
    nodes: &[acm_os_application::KnowledgeNodeProjection],
) -> KnowledgeCandidateDto {
    knowledge_candidate_dto(
        acm_os_application::KnowledgeCandidateProjection {
            problem: acm_os_domain::CodeforcesProblemIdentity::new(
                acm_os_domain::CodeforcesContestIdentity::new(contest_id)
                    .expect("validated contest"),
                problem_index.to_owned(),
            )
            .expect("validated problem"),
            fingerprint: value.fingerprint,
            target_ref: value.target_ref,
            disposition: value.disposition,
        },
        nodes,
    )
}

fn knowledge_candidate_dto(
    value: acm_os_application::KnowledgeCandidateProjection,
    nodes: &[acm_os_application::KnowledgeNodeProjection],
) -> KnowledgeCandidateDto {
    let matches = nodes
        .iter()
        .filter(|node| {
            if value.target_ref.contains('/') {
                node.vault_relative_path
                    .strip_suffix(".md")
                    .is_some_and(|path| path.eq_ignore_ascii_case(&value.target_ref))
            } else {
                node.display_name.eq_ignore_ascii_case(&value.target_ref)
            }
        })
        .collect::<Vec<_>>();
    KnowledgeCandidateDto {
        contest_id: value.problem.contest().contest_id(),
        problem_index: value.problem.index().to_owned(),
        fingerprint: value.fingerprint,
        target_ref: value.target_ref,
        disposition: match value.disposition {
            acm_os_application::KnowledgeCandidateDisposition::Pending => "pending",
            acm_os_application::KnowledgeCandidateDisposition::AcceptedIntent => "acceptedIntent",
            acm_os_application::KnowledgeCandidateDisposition::Ignored => "ignored",
        },
        knowledge_node_id: (matches.len() == 1).then(|| matches[0].knowledge_node_id.clone()),
    }
}

fn knowledge_candidate_error_code(
    error: acm_os_application::KnowledgeCandidateError,
) -> &'static str {
    match error {
        acm_os_application::KnowledgeCandidateError::ProblemNotFound => "problem_not_found",
        acm_os_application::KnowledgeCandidateError::NotPersonal => {
            "candidate_requires_personal_problem"
        }
        acm_os_application::KnowledgeCandidateError::CandidateNotFound => "candidate_not_found",
        acm_os_application::KnowledgeCandidateError::InvalidFingerprint => {
            "invalid_candidate_fingerprint"
        }
        acm_os_application::KnowledgeCandidateError::InvalidTarget => "invalid_candidate_target",
        acm_os_application::KnowledgeCandidateError::PersistenceUnavailable => {
            "persistence_unavailable"
        }
        acm_os_application::KnowledgeCandidateError::IntegrityViolation => "integrity_violation",
    }
}

fn knowledge_node_dto(node: acm_os_application::KnowledgeNodeProjection) -> KnowledgeNodeDto {
    KnowledgeNodeDto {
        knowledge_node_id: node.knowledge_node_id,
        display_name: node.display_name,
        vault_relative_path: node.vault_relative_path,
        content_digest: node.content_digest,
        location_state: match node.location_state {
            acm_os_application::KnowledgeLocationState::Ready => "ready",
            acm_os_application::KnowledgeLocationState::LocationAnomaly => "locationAnomaly",
        },
    }
}

fn knowledge_understanding_level(
    value: &str,
) -> Result<acm_os_domain::KnowledgeUnderstandingLevel, &'static str> {
    match value {
        "notLearned" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::NotLearned),
        "vague" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Vague),
        "basic" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Basic),
        "proficient" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Proficient),
        "deep" => Ok(acm_os_domain::KnowledgeUnderstandingLevel::Deep),
        _ => Err("invalid_understanding_level"),
    }
}

fn knowledge_understanding_dto(
    value: acm_os_application::KnowledgeUnderstandingProjection,
) -> KnowledgeUnderstandingDto {
    KnowledgeUnderstandingDto {
        knowledge_node_id: value.knowledge_node_id,
        current: knowledge_understanding_level_dto(value.current),
        historical_highest: knowledge_understanding_level_dto(value.historical_highest),
        first_reached_highest_on: value.first_reached_highest_on.to_iso_string(),
    }
}

fn knowledge_understanding_level_dto(
    value: acm_os_domain::KnowledgeUnderstandingLevel,
) -> &'static str {
    match value {
        acm_os_domain::KnowledgeUnderstandingLevel::NotLearned => "notLearned",
        acm_os_domain::KnowledgeUnderstandingLevel::Vague => "vague",
        acm_os_domain::KnowledgeUnderstandingLevel::Basic => "basic",
        acm_os_domain::KnowledgeUnderstandingLevel::Proficient => "proficient",
        acm_os_domain::KnowledgeUnderstandingLevel::Deep => "deep",
    }
}

fn knowledge_detail_dto(
    detail: acm_os_application::KnowledgeDetailProjection,
) -> KnowledgeDetailDto {
    KnowledgeDetailDto {
        node: knowledge_node_dto(detail.node),
        understanding: detail.understanding.map(knowledge_understanding_dto),
        incoming: detail
            .incoming
            .into_iter()
            .map(knowledge_node_dto)
            .collect(),
        outgoing: detail
            .outgoing
            .into_iter()
            .map(knowledge_node_dto)
            .collect(),
        related_problems: detail
            .related_problems
            .into_iter()
            .map(|item| RelatedKnowledgeProblemDto {
                problem_id: item.problem_id,
                contest_id: item.problem.contest().contest_id(),
                problem_index: item.problem.index().to_owned(),
                title: item.title,
            })
            .collect(),
    }
}

fn knowledge_index_error_code(error: acm_os_application::KnowledgeIndexError) -> &'static str {
    match error {
        acm_os_application::KnowledgeIndexError::WorkspaceUnavailable => "workspace_unavailable",
        acm_os_application::KnowledgeIndexError::KnowledgeRootUnavailable => {
            "knowledge_root_unavailable"
        }
        acm_os_application::KnowledgeIndexError::KnowledgeNodeNotFound => {
            "knowledge_node_not_found"
        }
        acm_os_application::KnowledgeIndexError::PersistenceUnavailable => {
            "persistence_unavailable"
        }
        acm_os_application::KnowledgeIndexError::IntegrityViolation => "integrity_violation",
    }
}

fn obsidian_open_uri(active_vault: &str, relative_path: &str) -> Result<String, &'static str> {
    let vault = std::fs::canonicalize(active_vault).map_err(|_| "vault_unavailable")?;
    let target =
        std::fs::canonicalize(vault.join(relative_path)).map_err(|_| "note_open_failed")?;
    if !target.starts_with(&vault) || !target.is_file() {
        return Err("note_open_failed");
    }
    let mut uri = url::Url::parse("obsidian://open").map_err(|_| "note_open_failed")?;
    uri.query_pairs_mut()
        .append_pair("path", &obsidian_external_path(&target)?);
    // Obsidian treats `+` as a literal filename character in `path`; use URI
    // percent encoding for spaces instead of form-url-encoded spaces.
    Ok(String::from(uri).replace('+', "%20"))
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
    database
        .statement_assets(&problem)
        .await
        .map(|assets| {
            assets
                .into_iter()
                .map(|asset| LocalStatementAssetDto {
                    local_ref: asset.local_ref,
                    media_type: asset.media_type,
                    bytes: asset.bytes,
                })
                .collect()
        })
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
        archived: item.archived,
    }
}

fn contest_detail_dto(item: acm_os_application::ContestDetail) -> ContestDetailDto {
    ContestDetailDto {
        contest_id: item.contest.contest_id(),
        title: item.title,
        source_url: item.source_url,
        contest_date: item.contest_date,
        import_status: match item.import_status {
            acm_os_application::ContestImportStatus::Incomplete => "incomplete",
            acm_os_application::ContestImportStatus::Complete => "complete",
        },
        facts_status: match item.facts_status {
            acm_os_application::ContestFactsStatus::Pending => "pending",
            acm_os_application::ContestFactsStatus::Completed => "completed",
        },
        problems: item
            .problems
            .into_iter()
            .map(contest_problem_detail_item_dto)
            .collect(),
        corrections: item
            .corrections
            .into_iter()
            .map(|event| ContestCorrectionEventDto {
                correction_id: event.correction_id,
                problem_index: event.problem_index,
                field: match event.field {
                    acm_os_application::ContestCorrectionField::FinalContestResult => {
                        "finalContestResult"
                    }
                    acm_os_application::ContestCorrectionField::UpsolveDecision => {
                        "upsolveDecision"
                    }
                },
                old_value: event.old_value,
                new_value: event.new_value,
                corrected_at_utc: event.corrected_at_utc,
            })
            .collect(),
        ai_analysis: item.ai_analysis.map(|analysis| ContestAiAnalysisDto {
            raw_text: analysis.raw_text,
            parse_status: contest_ai_parse_status_dto(analysis.parse_status),
            parsed_projection_json: analysis.parsed_projection_json,
            updated_at_utc: analysis.updated_at_utc,
        }),
        archived: item.archived,
    }
}

fn contest_delete_preview_dto(
    item: acm_os_application::ContestDeletePreview,
) -> ContestDeletePreviewDto {
    ContestDeletePreviewDto {
        contest_title: item.contest_title,
        relationship_count: item.relationship_count,
        cleanup_problem_count: item.cleanup_problem_count,
        preserved_problem_count: item.preserved_problem_count,
    }
}
fn contest_management_error_code(
    error: acm_os_application::ContestManagementError,
) -> &'static str {
    match error {
        acm_os_application::ContestManagementError::Unavailable => "contest_management_unavailable",
        acm_os_application::ContestManagementError::NotFound => "contest_not_found",
    }
}

fn contest_library_series_filter(
    value: ContestLibrarySeriesFilterDto,
) -> Result<acm_os_application::ContestLibrarySeriesFilter, &'static str> {
    Ok(match value {
        ContestLibrarySeriesFilterDto::Any => acm_os_application::ContestLibrarySeriesFilter::Any,
        ContestLibrarySeriesFilterDto::Unassigned => {
            acm_os_application::ContestLibrarySeriesFilter::Unassigned
        }
        ContestLibrarySeriesFilterDto::Exact { series_id } => {
            if series_id <= 0 {
                return Err("invalid_contest_library_id");
            }
            acm_os_application::ContestLibrarySeriesFilter::Exact(series_id)
        }
    })
}

fn contest_library_year_filter(
    value: ContestLibraryYearFilterDto,
) -> Result<acm_os_application::ContestLibraryYearFilter, &'static str> {
    Ok(match value {
        ContestLibraryYearFilterDto::Any => acm_os_application::ContestLibraryYearFilter::Any,
        ContestLibraryYearFilterDto::Unassigned => {
            acm_os_application::ContestLibraryYearFilter::Unassigned
        }
        ContestLibraryYearFilterDto::Exact { year } => {
            if year == 0 {
                return Err("invalid_year");
            }
            acm_os_application::ContestLibraryYearFilter::Exact(year)
        }
    })
}

fn contest_library_scope(
    value: ContestLibraryScopeDto,
) -> Result<acm_os_application::ContestLibraryScope, &'static str> {
    match value {
        ContestLibraryScopeDto::All => Ok(acm_os_application::ContestLibraryScope::All),
        ContestLibraryScopeDto::Family {
            family_id,
            series,
            year,
        } => {
            if family_id <= 0 {
                return Err("invalid_contest_library_id");
            }
            Ok(acm_os_application::ContestLibraryScope::Family {
                family_id,
                series: contest_library_series_filter(series)?,
                year: contest_library_year_filter(year)?,
            })
        }
    }
}

fn contest_library_archive_filter(
    value: ContestLibraryArchiveFilterDto,
) -> acm_os_application::ContestLibraryArchiveFilter {
    match value {
        ContestLibraryArchiveFilterDto::All => acm_os_application::ContestLibraryArchiveFilter::All,
        ContestLibraryArchiveFilterDto::Active => {
            acm_os_application::ContestLibraryArchiveFilter::Active
        }
        ContestLibraryArchiveFilterDto::Archived => {
            acm_os_application::ContestLibraryArchiveFilter::Archived
        }
    }
}

fn contest_library_placement_dto(
    item: acm_os_application::ContestPlacement,
) -> ContestLibraryPlacementDto {
    ContestLibraryPlacementDto {
        placement_id: item.placement_id,
        family_id: item.family_id,
        family_name: item.family_name,
        series_id: item.series_id,
        series_name: item.series_name,
        year: item.year,
        ordinal: item.ordinal,
    }
}

fn contest_library_error_code(error: acm_os_application::ContestLibraryError) -> &'static str {
    match error {
        acm_os_application::ContestLibraryError::InvalidId => "invalid_contest_library_id",
        acm_os_application::ContestLibraryError::InvalidName => "invalid_name",
        acm_os_application::ContestLibraryError::InvalidYear => "invalid_year",
        acm_os_application::ContestLibraryError::InvalidOrdinal => "invalid_ordinal",
        acm_os_application::ContestLibraryError::FamilyNotFound => "family_not_found",
        acm_os_application::ContestLibraryError::SeriesNotFound => "series_not_found",
        acm_os_application::ContestLibraryError::ContestNotFound => "contest_not_found",
        acm_os_application::ContestLibraryError::PlacementNotFound => "placement_not_found",
        acm_os_application::ContestLibraryError::DuplicateFamilyName => "duplicate_family_name",
        acm_os_application::ContestLibraryError::DuplicateSeriesName => "duplicate_series_name",
        acm_os_application::ContestLibraryError::DuplicatePlacement => "duplicate_placement",
        acm_os_application::ContestLibraryError::SeriesFamilyMismatch => "series_family_mismatch",
        acm_os_application::ContestLibraryError::PersistenceUnavailable => {
            "contest_library_persistence_unavailable"
        }
        acm_os_application::ContestLibraryError::IntegrityViolation => {
            "contest_library_integrity_violation"
        }
    }
}

fn contest_ai_analysis_preview_dto(
    item: acm_os_application::ContestAiAnalysisPreview,
) -> ContestAiAnalysisPreviewDto {
    ContestAiAnalysisPreviewDto {
        raw_text: item.raw_text,
        parse_status: contest_ai_parse_status_dto(item.parse_status),
        parsed_projection_json: item.parsed_projection_json,
    }
}

fn contest_ai_parse_status_dto(status: acm_os_application::ContestAiParseStatus) -> &'static str {
    match status {
        acm_os_application::ContestAiParseStatus::Complete => "complete",
        acm_os_application::ContestAiParseStatus::Partial => "partial",
        acm_os_application::ContestAiParseStatus::Failed => "failed",
    }
}

fn contest_ai_analysis_error_code(
    error: acm_os_application::ContestAiAnalysisError,
) -> &'static str {
    match error {
        acm_os_application::ContestAiAnalysisError::Unavailable => {
            "contest_ai_analysis_unavailable"
        }
        acm_os_application::ContestAiAnalysisError::NotFound => "contest_not_found",
        acm_os_application::ContestAiAnalysisError::Empty => "contest_ai_analysis_empty",
        acm_os_application::ContestAiAnalysisError::Invalid => "contest_ai_analysis_invalid",
    }
}

fn contest_problem_detail_item_dto(
    item: acm_os_application::ContestProblemDetailItem,
) -> ContestProblemDetailItemDto {
    ContestProblemDetailItemDto {
        contest_id: item.problem.problem.contest().contest_id(),
        index: item.problem.problem.index().to_owned(),
        title: item.problem.title,
        rating: item.problem.rating,
        has_statement_snapshot: item.problem.has_statement_snapshot,
        identity_type: problem_identity_type_dto(item.problem.identity_type),
        final_contest_result: item.final_contest_result.map(contest_final_result_dto),
        upsolve_decision: contest_upsolve_decision_dto(item.upsolve_decision),
        live_learning_status: learning_status_dto(item.live_learning_status),
    }
}

fn parse_contest_final_result_dto(
    value: &str,
) -> Result<acm_os_application::ContestFinalResult, &'static str> {
    match value {
        "unknown" => Ok(acm_os_application::ContestFinalResult::Unknown),
        "notAttempted" => Ok(acm_os_application::ContestFinalResult::NotAttempted),
        "accepted" => Ok(acm_os_application::ContestFinalResult::Accepted),
        "wrongAnswer" => Ok(acm_os_application::ContestFinalResult::WrongAnswer),
        "timeLimitExceeded" => Ok(acm_os_application::ContestFinalResult::TimeLimitExceeded),
        "memoryLimitExceeded" => Ok(acm_os_application::ContestFinalResult::MemoryLimitExceeded),
        "runtimeError" => Ok(acm_os_application::ContestFinalResult::RuntimeError),
        "compilationError" => Ok(acm_os_application::ContestFinalResult::CompilationError),
        "otherFailed" => Ok(acm_os_application::ContestFinalResult::OtherFailed),
        _ => Err("invalid_contest_result"),
    }
}
fn contest_final_result_dto(value: acm_os_application::ContestFinalResult) -> &'static str {
    match value {
        acm_os_application::ContestFinalResult::Unknown => "unknown",
        acm_os_application::ContestFinalResult::NotAttempted => "notAttempted",
        acm_os_application::ContestFinalResult::Accepted => "accepted",
        acm_os_application::ContestFinalResult::WrongAnswer => "wrongAnswer",
        acm_os_application::ContestFinalResult::TimeLimitExceeded => "timeLimitExceeded",
        acm_os_application::ContestFinalResult::MemoryLimitExceeded => "memoryLimitExceeded",
        acm_os_application::ContestFinalResult::RuntimeError => "runtimeError",
        acm_os_application::ContestFinalResult::CompilationError => "compilationError",
        acm_os_application::ContestFinalResult::OtherFailed => "otherFailed",
    }
}
fn parse_contest_upsolve_decision_dto(
    value: &str,
) -> Result<acm_os_application::ContestUpsolveDecision, &'static str> {
    match value {
        "planned" => Ok(acm_os_application::ContestUpsolveDecision::Planned),
        "notPlanned" => Ok(acm_os_application::ContestUpsolveDecision::NotPlanned),
        "undecided" => Ok(acm_os_application::ContestUpsolveDecision::Undecided),
        _ => Err("invalid_contest_upsolve_decision"),
    }
}
fn contest_upsolve_decision_dto(value: acm_os_application::ContestUpsolveDecision) -> &'static str {
    match value {
        acm_os_application::ContestUpsolveDecision::Planned => "planned",
        acm_os_application::ContestUpsolveDecision::NotPlanned => "notPlanned",
        acm_os_application::ContestUpsolveDecision::Undecided => "undecided",
    }
}
fn contest_facts_error_code(error: acm_os_application::ContestFactsError) -> &'static str {
    match error {
        acm_os_application::ContestFactsError::Unavailable => "contest_facts_unavailable",
        acm_os_application::ContestFactsError::NotFound => "contest_not_found",
        acm_os_application::ContestFactsError::ImportIncomplete => "contest_import_incomplete",
        acm_os_application::ContestFactsError::ContestDateMissing => "contest_date_missing",
        acm_os_application::ContestFactsError::ProblemSetMismatch => "contest_problem_set_mismatch",
        acm_os_application::ContestFactsError::AlreadyCompleted => {
            "contest_facts_already_completed"
        }
    }
}
fn contest_correction_error_code(
    error: acm_os_application::ContestCorrectionError,
) -> &'static str {
    match error {
        acm_os_application::ContestCorrectionError::Unavailable => "contest_correction_unavailable",
        acm_os_application::ContestCorrectionError::NotFound => "contest_problem_not_found",
        acm_os_application::ContestCorrectionError::FactsNotCompleted => {
            "contest_facts_not_completed"
        }
        acm_os_application::ContestCorrectionError::NoChange => "contest_correction_no_change",
    }
}

fn lightweight_problem_item_dto(
    item: acm_os_application::LightweightProblemItem,
) -> LightweightProblemItemDto {
    LightweightProblemItemDto {
        contest_id: item.problem.contest().contest_id(),
        index: item.problem.index().to_owned(),
        title: item.title,
        rating: item.rating,
        has_statement_snapshot: item.has_statement_snapshot,
        identity_type: problem_identity_type_dto(item.identity_type),
    }
}

fn lightweight_problem_detail_dto(
    item: acm_os_application::LightweightProblemDetail,
    today: Option<acm_os_domain::LocalDate>,
    review_in_progress: bool,
) -> LightweightProblemDetailDto {
    let review_action = if review_in_progress {
        Some("continueReview")
    } else {
        review_action_dto(&item.lifecycle, today)
    };
    LightweightProblemDetailDto {
        contest_id: item.problem.contest().contest_id(),
        index: item.problem.index().to_owned(),
        title: item.title,
        rating: item.rating,
        source_url: item.source_url,
        statement: match item.statement {
            acm_os_application::StatementReadState::Pending => StatementReadStateDto::Pending,
            acm_os_application::StatementReadState::Ready { sanitized_html } => {
                StatementReadStateDto::Ready { sanitized_html }
            }
        },
        identity_type: problem_identity_type_dto(item.identity_type),
        personal_note: item.personal_note.map(personal_note_binding_dto),
        lifecycle: problem_lifecycle_state_dto_with_review_state(
            item.lifecycle,
            review_in_progress,
        ),
        review_action,
    }
}

fn review_action_dto(
    lifecycle: &acm_os_application::ProblemLifecycleState,
    today: Option<acm_os_domain::LocalDate>,
) -> Option<&'static str> {
    if lifecycle.identity_type != acm_os_application::ProblemIdentityType::Personal {
        return None;
    }
    let cycle = lifecycle.active_review_cycle.as_ref()?;
    let decision = acm_os_domain::ReviewEligibilityEngine::decide(
        lifecycle.learning_status,
        cycle.next_due_local_date,
        today?,
    )
    .ok()?;
    Some(if decision.started_early {
        "earlyCheck"
    } else {
        "startReview"
    })
}

fn problem_lifecycle_state_dto(
    state: acm_os_application::ProblemLifecycleState,
) -> ProblemLifecycleStateDto {
    problem_lifecycle_state_dto_with_review_state(state, false)
}

fn problem_lifecycle_state_dto_with_review_state(
    state: acm_os_application::ProblemLifecycleState,
    review_in_progress: bool,
) -> ProblemLifecycleStateDto {
    ProblemLifecycleStateDto {
        learning_status: learning_status_dto(state.learning_status),
        learning_status_since_utc: state.learning_status_since_utc,
        next_review_due_local_date: state
            .active_review_cycle
            .map(|cycle| cycle.next_due_local_date.to_iso_string()),
        available_actions: if state.identity_type
            == acm_os_application::ProblemIdentityType::Personal
            && !review_in_progress
        {
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

fn review_attempt_dto(attempt: acm_os_application::ReviewAttempt) -> ReviewAttemptDto {
    ReviewAttemptDto {
        attempt_id: attempt.attempt_id,
        contest_id: attempt.problem.contest().contest_id(),
        index: attempt.problem.index().to_owned(),
        attempt_type: match attempt.attempt_type {
            acm_os_domain::ReviewAttemptType::FirstColdStart => "firstColdStart",
            acm_os_domain::ReviewAttemptType::LongTermReview => "longTermReview",
            acm_os_domain::ReviewAttemptType::EarlyCheck => "earlyCheck",
        },
        scheduled_due_local_date: attempt.scheduled_due_local_date.to_iso_string(),
        started_early: attempt.started_early,
        judgement_rule_version: attempt.judgement_rule_version,
        started_at_utc: attempt.started_at_utc,
    }
}

fn review_focus_dto(view: acm_os_application::ReviewFocusView) -> ReviewFocusDto {
    ReviewFocusDto {
        attempt: review_attempt_dto(view.attempt),
        title: view.title,
        source_url: view.source_url,
        statement_sanitized_html: view.statement_sanitized_html,
        statement_assets: view
            .statement_assets
            .into_iter()
            .map(|asset| LocalStatementAssetDto {
                local_ref: asset.local_ref,
                media_type: asset.media_type,
                bytes: asset.bytes,
            })
            .collect(),
    }
}

fn review_help_drawer_dto(view: acm_os_application::ReviewHelpDrawerView) -> ReviewHelpDrawerDto {
    ReviewHelpDrawerDto {
        attempt_id: view.attempt_id,
        items: view
            .items
            .into_iter()
            .map(|item| ReviewHelpItemDto {
                level: item.level.number(),
                consequence: item.level.consequence_code(),
                available: item.available,
                revealed_at_utc: item.revealed_at_utc,
            })
            .collect(),
    }
}

fn revealed_review_help_dto(
    revealed: acm_os_application::RevealedReviewHelp,
) -> RevealedReviewHelpDto {
    RevealedReviewHelpDto {
        event_id: revealed.event_id,
        attempt_id: revealed.attempt_id,
        level: revealed.level.number(),
        consequence: revealed.level.consequence_code(),
        title: revealed.title,
        content_markdown: revealed.content_markdown,
        source_digest: revealed.source_digest,
        revealed_at_utc: revealed.revealed_at_utc,
    }
}

fn parse_review_completion_input(
    input: CompleteReviewInput,
) -> Result<(String, acm_os_application::ReviewCompletionInput), &'static str> {
    let attempt_id = input.attempt_id;
    let first_submission =
        parse_submission_fact(&input.first_submission_result, input.first_submission_other)?;
    let final_submission = parse_submission_fact(&input.final_result, input.final_result_other)?;
    let debug_independence = match input.debug_independence.as_str() {
        "notNeeded" => acm_os_domain::DebugIndependence::NotNeeded,
        "independent" => acm_os_domain::DebugIndependence::Independent,
        "usedSolvingHelp" => acm_os_domain::DebugIndependence::UsedSolvingHelp,
        _ => return Err("review_completion_facts_invalid"),
    };
    let external_help = match input.external_help.as_str() {
        "none" => acm_os_domain::ExternalHelpLevel::None,
        "solvingHint" => acm_os_domain::ExternalHelpLevel::SolvingHint,
        "fullSolution" => acm_os_domain::ExternalHelpLevel::FullSolution,
        _ => return Err("review_completion_facts_invalid"),
    };
    let failure_reasons = input
        .failure_reasons
        .into_iter()
        .map(parse_failure_reason_input)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        attempt_id,
        acm_os_application::ReviewCompletionInput {
            final_ac: input.final_ac,
            first_submission,
            final_submission,
            total_submissions: input.total_submissions,
            idea_independent: input.idea_independent,
            implementation_independent: input.implementation_independent,
            debug_independence,
            external_help,
            failure_reasons,
        },
    ))
}

fn parse_submission_fact(
    result: &str,
    other_text: Option<String>,
) -> Result<acm_os_application::SubmissionFact, &'static str> {
    let result = match result {
        "accepted" => acm_os_domain::SubmissionResult::Accepted,
        "wrongAnswer" => acm_os_domain::SubmissionResult::WrongAnswer,
        "timeLimitExceeded" => acm_os_domain::SubmissionResult::TimeLimitExceeded,
        "memoryLimitExceeded" => acm_os_domain::SubmissionResult::MemoryLimitExceeded,
        "runtimeError" => acm_os_domain::SubmissionResult::RuntimeError,
        "compilationError" => acm_os_domain::SubmissionResult::CompilationError,
        "other" => acm_os_domain::SubmissionResult::Other,
        _ => return Err("review_completion_facts_invalid"),
    };
    Ok(acm_os_application::SubmissionFact { result, other_text })
}

fn parse_failure_reason_input(
    input: ReviewFailureReasonInput,
) -> Result<acm_os_application::ReviewFailureReason, &'static str> {
    match (input.code.as_str(), input.other_text) {
        ("noIdea", None) => Ok(acm_os_application::ReviewFailureReason::NoIdea),
        ("keyPropertyBlocked", None) => {
            Ok(acm_os_application::ReviewFailureReason::KeyPropertyBlocked)
        }
        ("derivationBlocked", None) => {
            Ok(acm_os_application::ReviewFailureReason::DerivationBlocked)
        }
        ("cannotImplement", None) => Ok(acm_os_application::ReviewFailureReason::CannotImplement),
        ("implementationError", None) => {
            Ok(acm_os_application::ReviewFailureReason::ImplementationError)
        }
        ("boundaryError", None) => Ok(acm_os_application::ReviewFailureReason::BoundaryError),
        ("complexityError", None) => Ok(acm_os_application::ReviewFailureReason::ComplexityError),
        ("other", Some(text)) => Ok(acm_os_application::ReviewFailureReason::Other(text)),
        _ => Err("invalid_review_failure_reason"),
    }
}

fn completed_review_attempt_dto(
    completed: acm_os_application::CompletedReviewAttempt,
) -> CompletedReviewAttemptDto {
    CompletedReviewAttemptDto {
        attempt: review_attempt_dto(completed.attempt),
        judgement: review_judgement_dto(completed.judgement),
        evidence_codes: completed.evidence_codes,
        failure_reasons: completed
            .failure_reasons
            .into_iter()
            .map(review_failure_reason_dto)
            .collect(),
        completed_at_utc: completed.completed_at_utc,
        completed_local_date: completed.completed_local_date.to_iso_string(),
        lifecycle: problem_lifecycle_state_dto(completed.lifecycle),
    }
}

fn review_history_dto(history: acm_os_application::ReviewHistoryView) -> ReviewHistoryDto {
    ReviewHistoryDto {
        contest_id: history.problem.contest().contest_id(),
        index: history.problem.index().to_owned(),
        historical_best_review: history.historical_best_review.map(review_judgement_dto),
        mastery: problem_mastery_projection_dto(history.mastery),
        attempts: history
            .attempts
            .into_iter()
            .map(review_history_item_dto)
            .collect(),
    }
}

fn problem_mastery_projection_dto(
    projection: acm_os_application::ProblemMasteryProjection,
) -> ProblemMasteryProjectionDto {
    ProblemMasteryProjectionDto {
        current: ProblemMasteryEvidenceDto {
            recalls_problem: projection.current.recalls_problem,
            multiple_solutions_clear: projection.current.multiple_solutions_clear,
            knowledge_understood: projection.current.knowledge_understood,
            implementation_fluent: projection.current.implementation_fluent,
            can_adapt_or_create: projection.current.can_adapt_or_create,
            transfer_solved_independently: projection.current.transfer_solved_independently,
        },
        historical_thoroughly_digested: projection.historical_thoroughly_digested,
        first_thoroughly_digested_local_date: projection
            .first_thoroughly_digested_local_date
            .map(|date| date.to_iso_string()),
    }
}

fn review_history_item_dto(item: acm_os_application::ReviewHistoryItem) -> ReviewHistoryItemDto {
    ReviewHistoryItemDto {
        attempt: review_attempt_dto(item.attempt),
        status: match item.status {
            acm_os_application::ReviewAttemptStatus::InProgress => "inProgress",
            acm_os_application::ReviewAttemptStatus::Completed => "completed",
            acm_os_application::ReviewAttemptStatus::Void => "void",
        },
        judgement: item.judgement.map(review_judgement_dto),
        completion_facts: item
            .completion_input
            .as_ref()
            .map(|input| ReviewCompletionFactsDto {
                final_ac: input.final_ac,
                first_submission_result: submission_fact_dto(&input.first_submission),
                final_result: submission_fact_dto(&input.final_submission),
                total_submissions: input.total_submissions,
                idea_independent: input.idea_independent,
                implementation_independent: input.implementation_independent,
                debug_independence: match input.debug_independence {
                    acm_os_domain::DebugIndependence::NotNeeded => "notNeeded",
                    acm_os_domain::DebugIndependence::Independent => "independent",
                    acm_os_domain::DebugIndependence::UsedSolvingHelp => "usedSolvingHelp",
                },
                external_help: match input.external_help {
                    acm_os_domain::ExternalHelpLevel::None => "none",
                    acm_os_domain::ExternalHelpLevel::SolvingHint => "solvingHint",
                    acm_os_domain::ExternalHelpLevel::FullSolution => "fullSolution",
                },
            }),
        evidence_codes: item.evidence_codes,
        failure_reasons: item
            .failure_reasons
            .into_iter()
            .map(review_failure_reason_dto)
            .collect(),
        help_levels: item
            .help_levels
            .into_iter()
            .map(|level| level.number())
            .collect(),
        completed_at_utc: item.completed_at_utc,
        completed_local_date: item.completed_local_date.map(|date| date.to_iso_string()),
        void_reason: item.void_reason,
        voided_at_utc: item.voided_at_utc,
    }
}

fn submission_fact_dto(fact: &acm_os_application::SubmissionFact) -> String {
    match fact.result {
        acm_os_domain::SubmissionResult::Accepted => "accepted".to_owned(),
        acm_os_domain::SubmissionResult::WrongAnswer => "wrongAnswer".to_owned(),
        acm_os_domain::SubmissionResult::TimeLimitExceeded => "timeLimitExceeded".to_owned(),
        acm_os_domain::SubmissionResult::MemoryLimitExceeded => "memoryLimitExceeded".to_owned(),
        acm_os_domain::SubmissionResult::RuntimeError => "runtimeError".to_owned(),
        acm_os_domain::SubmissionResult::CompilationError => "compilationError".to_owned(),
        acm_os_domain::SubmissionResult::Other => {
            format!("other:{}", fact.other_text.as_deref().unwrap_or_default())
        }
    }
}

fn review_failure_reason_dto(
    reason: acm_os_application::ReviewFailureReason,
) -> ReviewFailureReasonDto {
    match reason {
        acm_os_application::ReviewFailureReason::NoIdea => ReviewFailureReasonDto {
            code: "noIdea",
            other_text: None,
        },
        acm_os_application::ReviewFailureReason::KeyPropertyBlocked => ReviewFailureReasonDto {
            code: "keyPropertyBlocked",
            other_text: None,
        },
        acm_os_application::ReviewFailureReason::DerivationBlocked => ReviewFailureReasonDto {
            code: "derivationBlocked",
            other_text: None,
        },
        acm_os_application::ReviewFailureReason::CannotImplement => ReviewFailureReasonDto {
            code: "cannotImplement",
            other_text: None,
        },
        acm_os_application::ReviewFailureReason::ImplementationError => ReviewFailureReasonDto {
            code: "implementationError",
            other_text: None,
        },
        acm_os_application::ReviewFailureReason::BoundaryError => ReviewFailureReasonDto {
            code: "boundaryError",
            other_text: None,
        },
        acm_os_application::ReviewFailureReason::ComplexityError => ReviewFailureReasonDto {
            code: "complexityError",
            other_text: None,
        },
        acm_os_application::ReviewFailureReason::Other(text) => ReviewFailureReasonDto {
            code: "other",
            other_text: Some(text),
        },
    }
}

fn review_judgement_dto(judgement: acm_os_domain::ReviewJudgement) -> &'static str {
    match judgement {
        acm_os_domain::ReviewJudgement::Mastered => "mastered",
        acm_os_domain::ReviewJudgement::Partial => "partial",
        acm_os_domain::ReviewJudgement::Fail => "fail",
    }
}

fn problem_identity_type_dto(
    identity_type: acm_os_application::ProblemIdentityType,
) -> &'static str {
    match identity_type {
        acm_os_application::ProblemIdentityType::Lightweight => "lightweight",
        acm_os_application::ProblemIdentityType::Personal => "personal",
    }
}

fn personal_note_binding_dto(
    binding: acm_os_application::PersonalNoteBinding,
) -> PersonalNoteBindingDto {
    PersonalNoteBindingDto {
        vault_relative_path: binding.vault_relative_path,
    }
}

fn problem_markdown_projection_dto(
    projection: acm_os_application::ProblemMarkdownProjection,
) -> ProblemMarkdownProjectionDto {
    ProblemMarkdownProjectionDto {
        content_digest: projection.content_digest,
        known_sections: projection
            .known_sections
            .into_iter()
            .map(|section| KnownMarkdownSectionDto {
                name: section.name,
                start_offset: section.start_offset,
                end_offset: section.end_offset,
            })
            .collect(),
        solution_routes: projection
            .solution_routes
            .into_iter()
            .map(|route| SolutionRouteDto {
                name: route.name,
                start_offset: route.start_offset,
                end_offset: route.end_offset,
            })
            .collect(),
        warnings: projection
            .warnings
            .into_iter()
            .map(|warning| match warning {
                acm_os_application::MarkdownParseWarning::DuplicateKnownSection { name, count } => {
                    MarkdownParseWarningDto {
                        code: "duplicate_known_section",
                        name,
                        count,
                    }
                }
            })
            .collect(),
    }
}

fn contest_read_error_code(error: acm_os_application::ContestReadError) -> &'static str {
    match error {
        acm_os_application::ContestReadError::NotFound => "not_found",
        acm_os_application::ContestReadError::Unavailable => "unavailable",
    }
}

fn today_snapshot_dto(snapshot: acm_os_application::TodaySnapshot) -> TodaySnapshotDto {
    TodaySnapshotDto {
        plan_id: snapshot.plan_id,
        local_date: snapshot.local_date.to_iso_string(),
        budget_minutes: snapshot.budget_minutes,
        planned_minutes: snapshot.planned_minutes,
        over_budget_minutes: snapshot.over_budget_minutes,
        review_only_streak: snapshot.review_only_streak,
        entries: snapshot.entries.into_iter().map(today_entry_dto).collect(),
    }
}

fn weekly_acm_budget_dto(
    schedule: acm_os_application::WeeklyAcmBudgetSchedule,
) -> WeeklyAcmBudgetDto {
    WeeklyAcmBudgetDto {
        monday: schedule.monday,
        tuesday: schedule.tuesday,
        wednesday: schedule.wednesday,
        thursday: schedule.thursday,
        friday: schedule.friday,
        saturday: schedule.saturday,
        sunday: schedule.sunday,
    }
}

fn parse_weekly_acm_budget(
    schedule: WeeklyAcmBudgetDto,
) -> acm_os_application::WeeklyAcmBudgetSchedule {
    acm_os_application::WeeklyAcmBudgetSchedule {
        monday: schedule.monday,
        tuesday: schedule.tuesday,
        wednesday: schedule.wednesday,
        thursday: schedule.thursday,
        friday: schedule.friday,
        saturday: schedule.saturday,
        sunday: schedule.sunday,
    }
}

fn today_entry_dto(entry: acm_os_application::TodaySnapshotEntry) -> TodayEntryDto {
    TodayEntryDto {
        entry_id: entry.entry_id,
        problem_id: entry.problem_id,
        contest_id: entry.contest_id,
        problem_index: entry.problem_index,
        problem_title: entry.problem_title,
        review_attempt_id: entry.review_attempt_id,
        lane: today_lane_code(entry.lane).to_owned(),
        reason: today_reason_code(entry.reason).to_owned(),
        planning_cost_minutes: entry.planning_cost_minutes,
        position: entry.position,
        origin: today_origin_code(entry.origin).to_owned(),
        status: today_status_code(entry.status).to_owned(),
    }
}

fn today_replan_preview_dto(
    preview: acm_os_application::TodayReplanPreview,
) -> TodayReplanPreviewDto {
    TodayReplanPreviewDto {
        expected_snapshot: today_snapshot_dto(preview.expected_snapshot),
        proposed_budget_minutes: preview.proposed_budget_minutes,
        proposed_planned_minutes: preview.proposed_planned_minutes,
        proposed_over_budget_minutes: preview.proposed_over_budget_minutes,
        proposed_review_only_streak: preview.proposed_review_only_streak,
        entries: preview
            .entries
            .into_iter()
            .map(|entry| TodayReplanEntryDto {
                existing_entry_id: entry.existing_entry_id,
                problem_id: entry.problem_id,
                review_attempt_id: entry.review_attempt_id,
                lane: today_lane_code(entry.lane).to_owned(),
                reason: today_reason_code(entry.reason).to_owned(),
                planning_cost_minutes: entry.planning_cost_minutes,
                origin: today_origin_code(entry.origin).to_owned(),
                status: today_status_code(entry.status).to_owned(),
            })
            .collect(),
    }
}

fn today_extra_suggestions_preview_dto(
    preview: acm_os_application::TodayExtraSuggestionsPreview,
) -> TodayExtraSuggestionsPreviewDto {
    TodayExtraSuggestionsPreviewDto {
        expected_snapshot: today_snapshot_dto(preview.expected_snapshot),
        remaining_budget_minutes: preview.remaining_budget_minutes,
        suggestions: preview
            .suggestions
            .into_iter()
            .map(|suggestion| TodayExtraSuggestionDto {
                problem_id: suggestion.problem_id,
                contest_id: suggestion.contest_id,
                problem_index: suggestion.problem_index,
                problem_title: suggestion.problem_title,
                review_attempt_id: suggestion.review_attempt_id,
                lane: today_lane_code(suggestion.lane).to_owned(),
                reason: today_reason_code(suggestion.reason).to_owned(),
                planning_cost_minutes: suggestion.planning_cost_minutes,
            })
            .collect(),
    }
}

fn parse_today_replan_preview(
    preview: TodayReplanPreviewDto,
) -> Result<acm_os_application::TodayReplanPreview, &'static str> {
    Ok(acm_os_application::TodayReplanPreview {
        expected_snapshot: parse_today_snapshot(preview.expected_snapshot)?,
        proposed_budget_minutes: preview.proposed_budget_minutes,
        proposed_planned_minutes: preview.proposed_planned_minutes,
        proposed_over_budget_minutes: preview.proposed_over_budget_minutes,
        proposed_review_only_streak: preview.proposed_review_only_streak,
        entries: preview
            .entries
            .into_iter()
            .map(|entry| {
                Ok(acm_os_application::TodayReplanEntry {
                    existing_entry_id: entry.existing_entry_id,
                    problem_id: entry.problem_id,
                    review_attempt_id: entry.review_attempt_id,
                    lane: parse_today_lane_code(&entry.lane)?,
                    reason: parse_today_reason_code(&entry.reason)?,
                    planning_cost_minutes: entry.planning_cost_minutes,
                    origin: parse_today_origin_code(&entry.origin)?,
                    status: parse_today_status_code(&entry.status)?,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
    })
}

fn parse_today_extra_suggestions_preview(
    preview: TodayExtraSuggestionsPreviewDto,
) -> Result<acm_os_application::TodayExtraSuggestionsPreview, &'static str> {
    Ok(acm_os_application::TodayExtraSuggestionsPreview {
        expected_snapshot: parse_today_snapshot(preview.expected_snapshot)?,
        remaining_budget_minutes: preview.remaining_budget_minutes,
        suggestions: preview
            .suggestions
            .into_iter()
            .map(|suggestion| {
                Ok(acm_os_application::TodayExtraSuggestion {
                    problem_id: suggestion.problem_id,
                    contest_id: suggestion.contest_id,
                    problem_index: suggestion.problem_index,
                    problem_title: suggestion.problem_title,
                    review_attempt_id: suggestion.review_attempt_id,
                    lane: parse_today_lane_code(&suggestion.lane)?,
                    reason: parse_today_reason_code(&suggestion.reason)?,
                    planning_cost_minutes: suggestion.planning_cost_minutes,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
    })
}

fn parse_today_snapshot(
    snapshot: TodaySnapshotDto,
) -> Result<acm_os_application::TodaySnapshot, &'static str> {
    Ok(acm_os_application::TodaySnapshot {
        plan_id: snapshot.plan_id,
        local_date: acm_os_domain::LocalDate::parse_iso(&snapshot.local_date)
            .map_err(|_| "invalid_today_snapshot")?,
        budget_minutes: snapshot.budget_minutes,
        planned_minutes: snapshot.planned_minutes,
        over_budget_minutes: snapshot.over_budget_minutes,
        review_only_streak: snapshot.review_only_streak,
        entries: snapshot
            .entries
            .into_iter()
            .map(|entry| {
                Ok(acm_os_application::TodaySnapshotEntry {
                    entry_id: entry.entry_id,
                    problem_id: entry.problem_id,
                    contest_id: entry.contest_id,
                    problem_index: entry.problem_index,
                    problem_title: entry.problem_title,
                    review_attempt_id: entry.review_attempt_id,
                    lane: parse_today_lane_code(&entry.lane)?,
                    reason: parse_today_reason_code(&entry.reason)?,
                    planning_cost_minutes: entry.planning_cost_minutes,
                    position: entry.position,
                    origin: parse_today_origin_code(&entry.origin)?,
                    status: parse_today_status_code(&entry.status)?,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
    })
}

fn today_lane_code(value: acm_os_domain::TodayCandidateLane) -> &'static str {
    match value {
        acm_os_domain::TodayCandidateLane::CarryIn => "carryIn",
        acm_os_domain::TodayCandidateLane::Review => "review",
        acm_os_domain::TodayCandidateLane::Study => "study",
    }
}
fn parse_today_lane_code(value: &str) -> Result<acm_os_domain::TodayCandidateLane, &'static str> {
    match value {
        "carryIn" => Ok(acm_os_domain::TodayCandidateLane::CarryIn),
        "review" => Ok(acm_os_domain::TodayCandidateLane::Review),
        "study" => Ok(acm_os_domain::TodayCandidateLane::Study),
        _ => Err("invalid_today_snapshot"),
    }
}
fn today_reason_code(value: acm_os_domain::TodayCandidateReason) -> &'static str {
    match value {
        acm_os_domain::TodayCandidateReason::ContinueReview => "continueReview",
        acm_os_domain::TodayCandidateReason::ContinueLearning => "continueLearning",
        acm_os_domain::TodayCandidateReason::DueFirstColdStart => "dueFirstColdStart",
        acm_os_domain::TodayCandidateReason::DueLongTermReview => "dueLongTermReview",
        acm_os_domain::TodayCandidateReason::Relearn => "relearn",
        acm_os_domain::TodayCandidateReason::Upsolve => "upsolve",
    }
}
fn parse_today_reason_code(
    value: &str,
) -> Result<acm_os_domain::TodayCandidateReason, &'static str> {
    match value {
        "continueReview" => Ok(acm_os_domain::TodayCandidateReason::ContinueReview),
        "continueLearning" => Ok(acm_os_domain::TodayCandidateReason::ContinueLearning),
        "dueFirstColdStart" => Ok(acm_os_domain::TodayCandidateReason::DueFirstColdStart),
        "dueLongTermReview" => Ok(acm_os_domain::TodayCandidateReason::DueLongTermReview),
        "relearn" => Ok(acm_os_domain::TodayCandidateReason::Relearn),
        "upsolve" => Ok(acm_os_domain::TodayCandidateReason::Upsolve),
        _ => Err("invalid_today_snapshot"),
    }
}
fn today_origin_code(value: acm_os_application::TodayEntryOrigin) -> &'static str {
    match value {
        acm_os_application::TodayEntryOrigin::Auto => "auto",
        acm_os_application::TodayEntryOrigin::Manual => "manual",
    }
}
fn parse_today_origin_code(
    value: &str,
) -> Result<acm_os_application::TodayEntryOrigin, &'static str> {
    match value {
        "auto" => Ok(acm_os_application::TodayEntryOrigin::Auto),
        "manual" => Ok(acm_os_application::TodayEntryOrigin::Manual),
        _ => Err("invalid_today_snapshot"),
    }
}
fn today_status_code(value: acm_os_application::TodayEntryStatus) -> &'static str {
    match value {
        acm_os_application::TodayEntryStatus::NotStarted => "notStarted",
        acm_os_application::TodayEntryStatus::InProgress => "inProgress",
        acm_os_application::TodayEntryStatus::Completed => "completed",
        acm_os_application::TodayEntryStatus::Unavailable => "unavailable",
    }
}
fn parse_today_status_code(
    value: &str,
) -> Result<acm_os_application::TodayEntryStatus, &'static str> {
    match value {
        "notStarted" => Ok(acm_os_application::TodayEntryStatus::NotStarted),
        "inProgress" => Ok(acm_os_application::TodayEntryStatus::InProgress),
        "completed" => Ok(acm_os_application::TodayEntryStatus::Completed),
        "unavailable" => Ok(acm_os_application::TodayEntryStatus::Unavailable),
        _ => Err("invalid_today_snapshot"),
    }
}

fn today_error_code(error: acm_os_application::TodaySnapshotError) -> &'static str {
    match error {
        acm_os_application::TodaySnapshotError::PersistenceUnavailable => "today_unavailable",
        acm_os_application::TodaySnapshotError::IntegrityViolation => "today_integrity_violation",
        acm_os_application::TodaySnapshotError::InvalidReorder => "invalid_today_reorder",
        acm_os_application::TodaySnapshotError::InvalidTodayDone => "invalid_today_done",
        acm_os_application::TodaySnapshotError::InvalidExtraSuggestion => {
            "invalid_today_suggestion"
        }
        acm_os_application::TodaySnapshotError::StaleExtraSuggestions => "stale_today_suggestions",
        acm_os_application::TodaySnapshotError::StaleReplanPreview => "stale_today_replan",
        acm_os_application::TodaySnapshotError::Candidate(_)
        | acm_os_application::TodaySnapshotError::Ordering(_)
        | acm_os_application::TodaySnapshotError::Planning(_) => "today_planning_failed",
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualBackupPreviewDto {
    schema_version: i64,
    backup_directory: String,
    filename_prefix: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualBackupResultDto {
    path: String,
    schema_version: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInventoryEntryDto {
    path: String,
    category: String,
    size_bytes: u64,
    integrity_verified: bool,
    retention: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInventoryDto {
    entries: Vec<BackupInventoryEntryDto>,
    daily_keep: u32,
    weekly_keep: u32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemRestoreCandidatePreviewDto {
    source_path: String,
    schema_version: i64,
    supported_schema_version: i64,
    migration_required: bool,
    restores_system_facts: bool,
    overwrites_markdown: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemRestoreCandidateInput {
    source_path: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreIntentPreparationDto {
    staging_path: String,
    pre_restore_snapshot_path: String,
    candidate: SystemRestoreCandidatePreviewDto,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDiagnosticsDto {
    pending_intent: bool,
    rollback_artifact_path: Option<String>,
    rollback_integrity_verified: Option<bool>,
    startup_state: &'static str,
    current_schema_version: Option<i64>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRestoreRebuildPreviewDto {
    problem_binding_count: u64,
    knowledge_binding_count: u64,
    derived_relation_count: u64,
    revalidates_bindings: bool,
    rebuilds_derived_knowledge: bool,
    overwrites_markdown: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRestoreBindingAnomalyDto {
    problem_id: i64,
    vault_relative_path: String,
    reason: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRestoreProblemBindingValidationDto {
    total_count: u64,
    ready_count: u64,
    anomalies: Vec<PostRestoreBindingAnomalyDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRestoreKnowledgeBindingValidationDto {
    total_count: u64,
    ready_count: u64,
    confirmed_deleted_count: u64,
    anomalies: Vec<PostRestoreKnowledgeBindingAnomalyDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRestoreKnowledgeBindingAnomalyDto {
    knowledge_node_id: String,
    vault_relative_path: String,
    reason: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRestoreRebuildPreconditionCheckDto {
    eligible: bool,
    blockers: Vec<String>,
    problem_binding_anomaly_count: u64,
    knowledge_binding_anomaly_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRestoreRebuildApplyResultDto {
    knowledge_node_count: u64,
    relation_count: u64,
    location_anomaly_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticExportPreviewDto {
    output_directory: String,
    sections: Vec<String>,
    privacy_exclusions: Vec<String>,
    creates_files: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticExportResultDto {
    path: String,
    sections: Vec<String>,
}

#[tauri::command]
pub async fn preview_manual_backup(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<ManualBackupPreviewDto, &'static str> {
    acm_os_application::preview_manual_backup(database.inner())
        .await
        .map(|value| ManualBackupPreviewDto {
            schema_version: value.schema_version,
            backup_directory: value.backup_directory,
            filename_prefix: value.filename_prefix,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn create_manual_backup(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<ManualBackupResultDto, &'static str> {
    acm_os_application::create_manual_backup(database.inner())
        .await
        .map(|value| ManualBackupResultDto {
            path: value.path,
            schema_version: value.schema_version,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn backup_inventory(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<BackupInventoryDto, &'static str> {
    acm_os_application::backup_inventory(database.inner())
        .await
        .map(|value| BackupInventoryDto {
            entries: value
                .entries
                .into_iter()
                .map(|entry| BackupInventoryEntryDto {
                    path: entry.path,
                    category: entry.category,
                    size_bytes: entry.size_bytes,
                    integrity_verified: entry.integrity_verified,
                    retention: entry.retention,
                })
                .collect(),
            daily_keep: value.daily_keep,
            weekly_keep: value.weekly_keep,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn preview_system_restore_candidate(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: SystemRestoreCandidateInput,
) -> Result<SystemRestoreCandidatePreviewDto, &'static str> {
    acm_os_application::preview_system_restore_candidate(database.inner(), input.source_path)
        .await
        .map(|value| SystemRestoreCandidatePreviewDto {
            source_path: value.source_path,
            schema_version: value.schema_version,
            supported_schema_version: value.supported_schema_version,
            migration_required: value.migration_required,
            restores_system_facts: value.restores_system_facts,
            overwrites_markdown: value.overwrites_markdown,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn prepare_system_restore(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: SystemRestoreCandidateInput,
) -> Result<RestoreIntentPreparationDto, &'static str> {
    acm_os_application::prepare_restore_intent(database.inner(), input.source_path)
        .await
        .map(|value| RestoreIntentPreparationDto {
            staging_path: value.staging_path,
            pre_restore_snapshot_path: value.pre_restore_snapshot_path,
            candidate: SystemRestoreCandidatePreviewDto {
                source_path: value.candidate.source_path,
                schema_version: value.candidate.schema_version,
                supported_schema_version: value.candidate.supported_schema_version,
                migration_required: value.candidate.migration_required,
                restores_system_facts: value.candidate.restores_system_facts,
                overwrites_markdown: value.candidate.overwrites_markdown,
            },
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub fn restart_for_pending_restore(
    app: tauri::AppHandle,
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<(), &'static str> {
    if !database.has_pending_restore_intent() {
        return Err("restore_intent_missing");
    }
    app.request_restart();
    Ok(())
}

#[tauri::command]
pub async fn restore_diagnostics(
    startup: tauri::State<'_, acm_os_application::StartupStatusQuery>,
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<RestoreDiagnosticsDto, ()> {
    let (startup_state, current_schema_version) = match startup.execute() {
        acm_os_application::StartupGateStatus::Ready { schema_version } => {
            ("ready", Some(*schema_version))
        }
        acm_os_application::StartupGateStatus::RecoveryRequired { .. } => {
            ("recoveryRequired", None)
        }
    };
    let diagnostics = database.inspect_restore_diagnostics().await;
    Ok(RestoreDiagnosticsDto {
        pending_intent: diagnostics.pending_intent,
        rollback_artifact_path: diagnostics.rollback_artifact_path,
        rollback_integrity_verified: diagnostics.rollback_integrity_verified,
        startup_state,
        current_schema_version,
    })
}

#[tauri::command]
pub async fn confirm_restore_rollback_cleanup(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    rollback_artifact_path: String,
) -> Result<(), &'static str> {
    database
        .confirm_restore_rollback_cleanup(&rollback_artifact_path)
        .await
        .map_err(|error| match error {
            acm_os_infrastructure::RestoreRollbackCleanupError::Unavailable => {
                "restore_rollback_unavailable"
            }
            acm_os_infrastructure::RestoreRollbackCleanupError::PendingIntent => {
                "restore_intent_pending"
            }
            acm_os_infrastructure::RestoreRollbackCleanupError::InvalidPath => {
                "restore_rollback_invalid_path"
            }
            acm_os_infrastructure::RestoreRollbackCleanupError::IntegrityFailed => {
                "restore_rollback_integrity_failed"
            }
            acm_os_infrastructure::RestoreRollbackCleanupError::DeleteFailed => {
                "restore_rollback_delete_failed"
            }
        })
}

#[tauri::command]
pub async fn preview_post_restore_rebuild(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<PostRestoreRebuildPreviewDto, &'static str> {
    acm_os_application::preview_post_restore_rebuild(database.inner())
        .await
        .map(|value| PostRestoreRebuildPreviewDto {
            problem_binding_count: value.problem_binding_count,
            knowledge_binding_count: value.knowledge_binding_count,
            derived_relation_count: value.derived_relation_count,
            revalidates_bindings: value.revalidates_bindings,
            rebuilds_derived_knowledge: value.rebuilds_derived_knowledge,
            overwrites_markdown: value.overwrites_markdown,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn validate_post_restore_problem_bindings(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<PostRestoreProblemBindingValidationDto, &'static str> {
    acm_os_application::validate_post_restore_problem_bindings(database.inner())
        .await
        .map(|value| PostRestoreProblemBindingValidationDto {
            total_count: value.total_count,
            ready_count: value.ready_count,
            anomalies: value
                .anomalies
                .into_iter()
                .map(|item| PostRestoreBindingAnomalyDto {
                    problem_id: item.problem_id,
                    vault_relative_path: item.vault_relative_path,
                    reason: item.reason,
                })
                .collect(),
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn validate_post_restore_knowledge_bindings(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<PostRestoreKnowledgeBindingValidationDto, &'static str> {
    acm_os_application::validate_post_restore_knowledge_bindings(database.inner())
        .await
        .map(|value| PostRestoreKnowledgeBindingValidationDto {
            total_count: value.total_count,
            ready_count: value.ready_count,
            confirmed_deleted_count: value.confirmed_deleted_count,
            anomalies: value
                .anomalies
                .into_iter()
                .map(|item| PostRestoreKnowledgeBindingAnomalyDto {
                    knowledge_node_id: item.knowledge_node_id,
                    vault_relative_path: item.vault_relative_path,
                    reason: item.reason,
                })
                .collect(),
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn check_post_restore_rebuild_preconditions(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<PostRestoreRebuildPreconditionCheckDto, &'static str> {
    acm_os_application::check_post_restore_rebuild_preconditions(database.inner())
        .await
        .map(|value| PostRestoreRebuildPreconditionCheckDto {
            eligible: value.eligible,
            blockers: value.blockers,
            problem_binding_anomaly_count: value.problem_binding_anomaly_count,
            knowledge_binding_anomaly_count: value.knowledge_binding_anomaly_count,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn apply_post_restore_rebuild(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<PostRestoreRebuildApplyResultDto, &'static str> {
    acm_os_application::apply_post_restore_rebuild(database.inner())
        .await
        .map(|value| PostRestoreRebuildApplyResultDto {
            knowledge_node_count: value.knowledge_node_count,
            relation_count: value.relation_count,
            location_anomaly_count: value.location_anomaly_count,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn preview_diagnostic_export(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<DiagnosticExportPreviewDto, &'static str> {
    acm_os_application::preview_diagnostic_export(database.inner())
        .await
        .map(|value| DiagnosticExportPreviewDto {
            output_directory: value.output_directory,
            sections: value.sections,
            privacy_exclusions: value.privacy_exclusions,
            creates_files: value.creates_files,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn create_diagnostic_export(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<DiagnosticExportResultDto, &'static str> {
    acm_os_application::create_diagnostic_export(database.inner())
        .await
        .map(|value| DiagnosticExportResultDto {
            path: value.path,
            sections: value.sections,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[tauri::command]
pub async fn create_weekly_backup(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<ManualBackupResultDto, &'static str> {
    acm_os_application::create_weekly_backup(database.inner())
        .await
        .map(|value| ManualBackupResultDto {
            path: value.path,
            schema_version: value.schema_version,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRetentionPreviewDto {
    protected_paths: Vec<String>,
    prune_candidate_paths: Vec<String>,
    daily_keep: u32,
    weekly_keep: u32,
}

#[tauri::command]
pub async fn preview_backup_retention(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
) -> Result<BackupRetentionPreviewDto, &'static str> {
    acm_os_application::preview_backup_retention(database.inner())
        .await
        .map(|value| BackupRetentionPreviewDto {
            protected_paths: value.protected_paths,
            prune_candidate_paths: value.prune_candidate_paths,
            daily_keep: value.daily_keep,
            weekly_keep: value.weekly_keep,
        })
        .map_err(acm_os_application::ManualBackupError::code)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRetentionApplyInput {
    paths: Vec<String>,
}

#[tauri::command]
pub async fn apply_backup_retention(
    database: tauri::State<'_, acm_os_infrastructure::DatabaseRuntime>,
    input: BackupRetentionApplyInput,
) -> Result<u64, &'static str> {
    acm_os_application::apply_backup_retention(database.inner(), input.paths)
        .await
        .map_err(acm_os_application::ManualBackupError::code)
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
        ActiveReviewCycle, KnownMarkdownSection, LocalStatementAsset, MarkdownParseWarning,
        PersonalNoteBinding, PersonalNoteReadState, ProblemIdentityType, ProblemLifecycleState,
        ProblemMarkdownProjection, RevealedReviewHelp, ReviewAttempt, ReviewFocusView,
        ReviewHelpDrawerView, ReviewHelpItem, SolutionRoute, StartupDestination, StartupGateStatus,
        StartupRecoveryReason, WorkspaceConfiguration, WorkspaceConfigurationError,
        WorkspaceConfigurationStatus, WorkspacePathField,
    };
    use serde_json::json;

    use super::{
        app_shell_status_dto, contest_library_error_code, contest_library_placement_dto,
        contest_library_scope, contest_library_series_filter, knowledge_index_error_code,
        knowledge_understanding_dto, knowledge_understanding_level,
        normalize_windows_verbatim_path, obsidian_open_uri, parse_review_completion_input,
        personal_note_read_state_dto, problem_lifecycle_state_dto, revealed_review_help_dto,
        review_action_dto, review_focus_dto, review_help_drawer_dto, startup_status_dto,
        workspace_error_dto, workspace_status_dto, CompleteReviewInput, ContestLibraryScopeDto,
        ContestLibrarySeriesFilterDto, LightweightProblemDetailDto,
        PersonalNoteRelocationCandidateDto, ProblemLifecycleStateDto, ReviewFailureReasonInput,
        StatementReadStateDto, TodayExtraSuggestionsPreviewDto, TodayReplanPreviewDto,
    };

    #[test]
    fn codeforces_problem_identity_preserves_legacy_ipc_compatibility() {
        let problem = super::codeforces_problem_identity(1979, "A".to_owned())
            .expect("legacy IPC identity");
        assert_eq!(problem.contest().platform().as_str(), "codeforces");
        assert_eq!(problem.contest().external_contest_key().as_str(), "1979");
        assert_eq!(problem.external_problem_key(), "A");
        assert_eq!(
            super::codeforces_problem_identity(0, "A".to_owned()),
            Err("invalid_problem_identity")
        );
        assert_eq!(
            super::codeforces_problem_identity(1979, "a".to_owned()),
            Err("invalid_problem_identity")
        );
    }

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
    fn contest_library_ipc_contract_uses_camel_case_tagged_filters_and_stable_errors() {
        let series: ContestLibrarySeriesFilterDto = serde_json::from_value(json!({
            "kind": "exact",
            "seriesId": 42
        }))
        .expect("deserialize exact series filter");
        assert_eq!(
            contest_library_series_filter(series),
            Ok(acm_os_application::ContestLibrarySeriesFilter::Exact(42))
        );

        let scope: ContestLibraryScopeDto = serde_json::from_value(json!({
            "kind": "family",
            "familyId": 3,
            "series": { "kind": "unassigned" },
            "year": { "kind": "exact", "year": 2026 }
        }))
        .expect("deserialize family scope");
        assert_eq!(
            contest_library_scope(scope),
            Ok(acm_os_application::ContestLibraryScope::Family {
                family_id: 3,
                series: acm_os_application::ContestLibrarySeriesFilter::Unassigned,
                year: acm_os_application::ContestLibraryYearFilter::Exact(2026),
            })
        );

        let placement = contest_library_placement_dto(acm_os_application::ContestPlacement {
            placement_id: 9,
            family_id: 3,
            family_name: "Codeforces".to_owned(),
            series_id: None,
            series_name: None,
            year: Some(2026),
            ordinal: None,
        });
        assert_eq!(
            serde_json::to_value(placement).expect("serialize placement DTO"),
            json!({
                "placementId": 9,
                "familyId": 3,
                "familyName": "Codeforces",
                "seriesId": null,
                "seriesName": null,
                "year": 2026,
                "ordinal": null
            })
        );

        assert_eq!(
            contest_library_error_code(acm_os_application::ContestLibraryError::InvalidName),
            "invalid_name"
        );
        assert_eq!(
            contest_library_error_code(acm_os_application::ContestLibraryError::DuplicatePlacement),
            "duplicate_placement"
        );
        assert_eq!(
            contest_library_error_code(
                acm_os_application::ContestLibraryError::SeriesFamilyMismatch
            ),
            "series_family_mismatch"
        );
        assert_eq!(
            contest_library_error_code(acm_os_application::ContestLibraryError::PlacementNotFound),
            "placement_not_found"
        );
    }

    #[test]
    fn today_preview_contracts_round_trip_through_camel_case_json() {
        let snapshot = json!({
            "planId": "plan-1", "localDate": "2026-08-12", "budgetMinutes": 120,
            "plannedMinutes": 60, "overBudgetMinutes": 0, "reviewOnlyStreak": 0,
            "entries": [{
                "entryId": "entry-1", "problemId": "7", "contestId": 1979,
                "problemIndex": "A", "problemTitle": "Alpha", "reviewAttemptId": null,
                "lane": "study", "reason": "upsolve", "planningCostMinutes": 60,
                "position": 0, "origin": "auto", "status": "completed"
            }]
        });
        let replan_json = json!({
            "expectedSnapshot": snapshot.clone(), "proposedBudgetMinutes": 90,
            "proposedPlannedMinutes": 60, "proposedOverBudgetMinutes": 0,
            "proposedReviewOnlyStreak": 0,
            "entries": [{
                "existingEntryId": "entry-1", "problemId": "7", "reviewAttemptId": null,
                "lane": "study", "reason": "upsolve", "planningCostMinutes": 60,
                "origin": "auto", "status": "completed"
            }]
        });
        let replan: TodayReplanPreviewDto =
            serde_json::from_value(replan_json.clone()).expect("deserialize replan preview");
        assert_eq!(
            serde_json::to_value(replan).expect("serialize replan preview"),
            replan_json
        );

        let suggestions_json = json!({
            "expectedSnapshot": snapshot, "remainingBudgetMinutes": 60,
            "suggestions": [{
                "problemId": "8", "contestId": 1979, "problemIndex": "B",
                "problemTitle": "Beta", "reviewAttemptId": null, "lane": "study",
                "reason": "relearn", "planningCostMinutes": 60
            }]
        });
        let suggestions: TodayExtraSuggestionsPreviewDto =
            serde_json::from_value(suggestions_json.clone())
                .expect("deserialize suggestions preview");
        assert_eq!(
            serde_json::to_value(suggestions).expect("serialize suggestions preview"),
            suggestions_json
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
    fn serializes_unresolved_critical_operation_startup_contract() {
        let dto = startup_status_dto(&StartupGateStatus::RecoveryRequired {
            reason: StartupRecoveryReason::UnresolvedCriticalOperation,
        });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize critical operation recovery status"),
            json!({
                "state": "recoveryRequired",
                "schemaVersion": null,
                "recoveryReason": "unresolved_critical_operation",
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
            review_action: None,
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
                },
                "reviewAction": null
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
    fn review_contract_exposes_attempt_metadata_and_only_focus_statement_data() {
        let problem = acm_os_domain::CodeforcesProblemIdentity::new(
            acm_os_domain::CodeforcesContestIdentity::new(1979).expect("contest"),
            "A",
        )
        .expect("problem");
        let due = acm_os_domain::LocalDate::parse_iso("2026-08-14").expect("due");
        let lifecycle = ProblemLifecycleState {
            identity_type: ProblemIdentityType::Personal,
            learning_status: acm_os_domain::LearningStatus::WaitingColdStart,
            learning_status_since_utc: "2026-08-11T00:00:00.000Z".to_owned(),
            active_review_cycle: Some(ActiveReviewCycle {
                cycle_number: 1,
                stage: 0,
                schedule_rule_version: 1,
                next_due_local_date: due,
            }),
        };
        assert_eq!(
            review_action_dto(&lifecycle, Some(due)),
            Some("startReview")
        );
        assert_eq!(
            review_action_dto(
                &lifecycle,
                Some(acm_os_domain::LocalDate::parse_iso("2026-08-13").expect("early")),
            ),
            Some("earlyCheck")
        );

        let dto = review_focus_dto(ReviewFocusView {
            attempt: ReviewAttempt {
                attempt_id: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab".to_owned(),
                problem,
                attempt_type: acm_os_domain::ReviewAttemptType::FirstColdStart,
                scheduled_due_local_date: due,
                started_early: false,
                judgement_rule_version: 1,
                started_at_utc: "2026-08-14T00:00:00.000Z".to_owned(),
            },
            title: "Problem A".to_owned(),
            source_url: "https://codeforces.com/contest/1979/problem/A".to_owned(),
            statement_sanitized_html: "<p>statement only</p>".to_owned(),
            statement_assets: vec![LocalStatementAsset {
                local_ref: "acm-os-asset://sample".to_owned(),
                media_type: "image/png".to_owned(),
                bytes: vec![1, 2, 3],
            }],
        });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize focus"),
            json!({
                "attempt": {
                    "attemptId": "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
                    "contestId": 1979,
                    "index": "A",
                    "attemptType": "firstColdStart",
                    "scheduledDueLocalDate": "2026-08-14",
                    "startedEarly": false,
                    "judgementRuleVersion": 1,
                    "startedAtUtc": "2026-08-14T00:00:00.000Z"
                },
                "title": "Problem A",
                "sourceUrl": "https://codeforces.com/contest/1979/problem/A",
                "statementSanitizedHtml": "<p>statement only</p>",
                "statementAssets": [{
                    "localRef": "acm-os-asset://sample",
                    "mediaType": "image/png",
                    "bytes": [1, 2, 3]
                }]
            })
        );
    }

    #[test]
    fn review_help_contract_exposes_no_content_before_reveal() {
        let drawer = review_help_drawer_dto(ReviewHelpDrawerView {
            attempt_id: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab".to_owned(),
            items: vec![ReviewHelpItem {
                level: acm_os_domain::ReviewHelpLevel::Hints,
                available: true,
                revealed_at_utc: None,
            }],
        });
        assert_eq!(
            serde_json::to_value(drawer).expect("serialize drawer"),
            json!({
                "attemptId": "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
                "items": [{
                    "level": 2,
                    "consequence": "partial_at_best",
                    "available": true,
                    "revealedAtUtc": null
                }]
            })
        );
        let revealed = revealed_review_help_dto(RevealedReviewHelp {
            event_id: "018f0d8e-4a5b-7c6d-8e9f-0123456789ac".to_owned(),
            attempt_id: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab".to_owned(),
            level: acm_os_domain::ReviewHelpLevel::FullSolution,
            title: "Full solution".to_owned(),
            content_markdown: "## 题解\nanswer".to_owned(),
            source_digest: "a".repeat(64),
            revealed_at_utc: "2026-08-14T00:00:00.000Z".to_owned(),
        });
        assert_eq!(
            serde_json::to_value(revealed).expect("serialize revealed help"),
            json!({
                "eventId": "018f0d8e-4a5b-7c6d-8e9f-0123456789ac",
                "attemptId": "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
                "level": 5,
                "consequence": "fail_only",
                "title": "Full solution",
                "contentMarkdown": "## 题解\nanswer",
                "sourceDigest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "revealedAtUtc": "2026-08-14T00:00:00.000Z"
            })
        );
    }

    #[test]
    fn completion_contract_accepts_facts_without_accepting_a_user_selected_judgement() {
        let (attempt_id, input) = parse_review_completion_input(CompleteReviewInput {
            attempt_id: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab".to_owned(),
            final_ac: true,
            first_submission_result: "wrongAnswer".to_owned(),
            first_submission_other: None,
            final_result: "accepted".to_owned(),
            final_result_other: None,
            total_submissions: 2,
            idea_independent: false,
            implementation_independent: true,
            debug_independence: "independent".to_owned(),
            external_help: "none".to_owned(),
            failure_reasons: vec![ReviewFailureReasonInput {
                code: "keyPropertyBlocked".to_owned(),
                other_text: None,
            }],
        })
        .expect("valid completion facts");
        assert_eq!(attempt_id, "018f0d8e-4a5b-7c6d-8e9f-0123456789ab");
        assert!(!input.idea_independent);
        assert_eq!(input.total_submissions, 2);
        assert_eq!(
            input.failure_reasons,
            [acm_os_application::ReviewFailureReason::KeyPropertyBlocked]
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
    fn serializes_personal_note_relocation_candidate_contract() {
        let dto = PersonalNoteRelocationCandidateDto {
            vault_relative_path: "Recovered/manual.md".to_owned(),
            occupied: false,
        };
        assert_eq!(
            serde_json::to_value(dto).expect("serialize relocation candidate"),
            json!({
                "vaultRelativePath": "Recovered/manual.md",
                "occupied": false
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
        assert!(uri.contains("A%20note.md"));
        assert!(!uri.contains("A+note.md"));
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
        std::fs::write(parent.path().join("outside.md"), "# Outside\n").expect("outside note");

        assert_eq!(
            obsidian_open_uri(vault.to_str().expect("utf-8 vault"), "../outside.md"),
            Err("note_open_failed")
        );
    }

    #[test]
    fn knowledge_open_errors_remain_specific_and_non_mutating() {
        assert_eq!(
            knowledge_index_error_code(
                acm_os_application::KnowledgeIndexError::KnowledgeNodeNotFound
            ),
            "knowledge_node_not_found"
        );
        assert_eq!(
            knowledge_index_error_code(
                acm_os_application::KnowledgeIndexError::KnowledgeRootUnavailable
            ),
            "knowledge_root_unavailable"
        );
    }

    #[test]
    fn knowledge_understanding_ipc_accepts_only_frozen_levels_and_serializes_history() {
        assert_eq!(
            knowledge_understanding_level("deep"),
            Ok(acm_os_domain::KnowledgeUnderstandingLevel::Deep)
        );
        assert_eq!(
            knowledge_understanding_level("mastered"),
            Err("invalid_understanding_level")
        );
        let dto =
            knowledge_understanding_dto(acm_os_application::KnowledgeUnderstandingProjection {
                knowledge_node_id: "node-1".to_owned(),
                current: acm_os_domain::KnowledgeUnderstandingLevel::Basic,
                historical_highest: acm_os_domain::KnowledgeUnderstandingLevel::Deep,
                first_reached_highest_on: acm_os_domain::LocalDate::parse_iso("2026-08-13")
                    .expect("valid date"),
            });
        assert_eq!(
            serde_json::to_value(dto).expect("serialize understanding"),
            json!({
                "knowledgeNodeId": "node-1",
                "current": "basic",
                "historicalHighest": "deep",
                "firstReachedHighestOn": "2026-08-13"
            })
        );
    }
}
