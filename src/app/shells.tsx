import {
  type FormEvent,
  type MouseEvent,
  type RefObject,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import katex from "katex";
import "katex/dist/katex.min.css";
import contestBookShell from "../assets/contest-book-shell.png";
import contestDisplayStandBack from "../assets/contest-display-stand-back.png";
import contestDisplayStandFront from "../assets/contest-display-stand-front.png";
import contestCabinetLeft from "../assets/contest-cabinet-left.png";
import contestCabinetCenter from "../assets/contest-cabinet-center.png";
import contestCabinetRight from "../assets/contest-cabinet-right.png";
import contestCabinetShelfForeground1 from "../assets/contest-cabinet-shelf-foreground-1.png";
import contestCabinetShelfForeground2 from "../assets/contest-cabinet-shelf-foreground-2.png";
import contestCabinetShelfForeground3 from "../assets/contest-cabinet-shelf-foreground-3.png";
import type { FoundationStatus } from "../ipc/foundation";
import {
  confirmKnowledgeUnderstanding,
  confirmKnowledgeMarkdownDeleted,
  acceptExistingKnowledgeCandidate,
  acceptExistingKnowledgeCandidateById,
  loadKnowledgeDetail,
  loadKnowledgeIndex,
  loadKnowledgeRelocationCandidates,
  loadKnowledgeCandidates,
  loadKnowledgeCandidatesById,
  loadKnowledgeReevaluationSuggestion,
  openKnowledgeInObsidian,
  rebindKnowledgeNode,
  resolveKnowledgeIdentityConflict,
  setKnowledgeCandidateDisposition,
  setKnowledgeCandidateDispositionById,
  type KnowledgeCandidateDto,
  type CanonicalKnowledgeCandidateDto,
  type KnowledgeDetailDto,
  type KnowledgeNodeDto,
  type KnowledgeRelocationCandidateDto,
  type KnowledgeIdentityConflictDto,
  type KnowledgeUnderstandingLevel,
} from "../ipc/knowledge";
import {
  createPersonalNote,
  createPersonalNoteById,
  completeReview,
  confirmPersonalNoteDeleted,
  confirmPersonalNoteDeletedById,
  deletePersonalNote,
  deletePersonalNoteById,
  getContestDetail,
  getContestShelf,
  listContestLibraryFamilies,
  createContestLibraryFamily,
  renameContestLibraryFamily,
  listContestLibrarySeries,
  createContestLibrarySeries,
  renameContestLibrarySeries,
  listContestLibraryYears,
  listContestLibraryPlacements,
  createContestLibraryPlacement,
  updateContestLibraryPlacement,
  removeContestLibraryPlacement,
  listContestLibraryContests,
  getCanonicalProblemDetail,
  getCanonicalStatementAssets,
  getLightweightProblemDetail,
  getLightweightProblems,
  getPersonalNoteProjection,
  getPersonalNoteProjectionById,
  getPersonalNoteRelocationCandidates,
  getPersonalNoteRelocationCandidatesById,
  getReviewFocus,
  getReviewHelpDrawer,
  getReviewAttemptHistory,
  getReviewHistory,
  getReviewHistoryById,
  getStatementAssets,
  importCodeforcesContest,
  importManualCodeforcesContest,
  completeContestFacts,
  correctContestProblemFacts,
  previewContestAiAnalysis,
  saveContestAiAnalysis,
  setContestArchived,
  previewDeleteContest,
  deleteContest,
  openPersonalNoteInObsidian,
  openPersonalNoteInObsidianById,
  openOriginalOj,
  rebindPersonalNote,
  rebindPersonalNoteById,
  revealReviewHelp,
  startOrResumeReview,
  startOrResumeReviewById,
  transitionProblemLifecycle,
  transitionProblemLifecycleById,
  updateProblemMasteryEvidence,
  updateProblemMasteryEvidenceById,
  voidReview,
  type CompleteReviewInputDto,
  type CompletedReviewAttemptDto,
  type ContestDetailDto,
  type ContestAiAnalysisPreviewDto,
  type ContestDeletePreviewDto,
  type ContestFinalResultDto,
  type ContestUpsolveDecisionDto,
  type ContestShelfItemDto,
  type ContestLibraryFamilyDto,
  type ContestLibrarySeriesDto,
  type ContestLibraryPlacementDto,
  type ContestLibrarySeriesFilterDto,
  type ContestLibraryYearFilterDto,
  type ContestLibraryScopeDto,
  type ContestLibraryArchiveFilterDto,
  type CanonicalProblemDetailDto,
  type LightweightProblemDetailDto,
  type LightweightProblemItemDto,
  type PersonalNoteReadStateDto,
  type PersonalNoteRelocationCandidateDto,
  type ProblemLifecycleActionDto,
  type ProblemMasteryEvidenceDto,
  type ReviewFocusDto,
  type RevealedReviewHelpDto,
  type ReviewHelpDrawerDto,
  type ReviewHelpItemDto,
  type ReviewHelpLevel,
  type ReviewHistoryDto,
  type ReviewHistoryItemDto,
  type CanonicalReviewHistoryDto,
  type CanonicalReviewHistoryItemDto,
  type ReviewFailureReasonCodeDto,
  type SubmissionResultDto,
} from "../ipc/contest";
import { onPersonalNoteInvalidated } from "../ipc/personal-note-events";
import {
  acceptTodayExtraSuggestion,
  applyTodayReplan,
  completeTodayEntry,
  loadToday,
  loadTodayExtraSuggestions,
  loadWeeklyAcmBudget,
  previewTodayReplan,
  reorderToday,
  saveWeeklyAcmBudget,
  type TodayEntryDto,
  type TodayExtraSuggestionsPreviewDto,
  type TodayReplanPreviewDto,
  type TodaySnapshotDto,
  type WeeklyAcmBudgetDto,
} from "../ipc/today";
import {
  createDiagnosticExport,
  previewDiagnosticExport,
  type DiagnosticExportPreviewDto,
} from "../ipc/startup";
import {
  activateReward,
  archiveCustomReward,
  createCustomReward,
  createRewardIntentId,
  getRewardAccountSummary,
  getRewardActivationState,
  getRewardRedemptionHistory,
  listCustomRewards,
  redeemCustomReward,
  refundCustomReward,
  updateCustomReward,
  type CustomRewardDto,
  type RedemptionHistoryItemDto,
  type RedemptionResultDto,
  type RewardAccountSummaryDto,
} from "../ipc/reward";
import type { StartupRecoveryReasonCode } from "../ipc/startup";
import {
  configureWorkspace,
  describeWorkspaceError,
  parseWorkspaceConfigurationError,
  type WorkspaceConfigurationDraft,
  type WorkspaceConfigurationErrorDto,
  type WorkspacePathField,
  type WorkspaceStatusDto,
} from "../ipc/workspace";
import {
  createManualBackup,
  loadBackupInventory,
  previewManualBackup,
  type BackupInventoryDto,
  type ManualBackupPreviewDto,
} from "../ipc/workspace";
import type { AppRoute, NormalPage } from "./routing";
import { displayProblemTitle } from "./translation";
import { t } from "./i18n";
import { getErrorPresentation } from "./i18n/errors";
import { LoadingState } from "./ui/states";

type Navigate = (pathname: string, options?: { replace?: boolean }) => void;
type ConfiguredWorkspace = Extract<WorkspaceStatusDto, { state: "configured" }>;

const EMPTY_WORKSPACE_DRAFT: WorkspaceConfigurationDraft = {
  activeVaultPath: "",
  problemRootPath: "",
  knowledgeRootPath: "",
};

export function LoadingShell() {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  return (
    <main aria-busy="true" className="gate-shell gate-shell--loading">
      <Brand />
      <p className="eyebrow">启动检查</p>
      <h1 ref={headingRef} tabIndex={-1}>正在检查系统事实</h1>
      <LoadingState message={t("shell.checkingFacts")} />
    </main>
  );
}

export function RecoveryShell({
  reason,
  supportedSchemaVersion,
  foundSchemaVersion,
}: {
  reason: StartupRecoveryReasonCode | "startup_status_unavailable";
  supportedSchemaVersion: number | null;
  foundSchemaVersion: number | null;
}) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [diagnosticPreview, setDiagnosticPreview] = useState<DiagnosticExportPreviewDto | null>(null);
  const [diagnosticResult, setDiagnosticResult] = useState<string | null>(null);
  const [diagnosticError, setDiagnosticError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  const inspectDiagnostics = () => {
    setDiagnosticError(null);
    previewDiagnosticExport().then(setDiagnosticPreview).catch(() => setDiagnosticError("Diagnostic export is unavailable while recovery state is being read."));
  };
  const exportDiagnostics = () => {
    setExporting(true);
    setDiagnosticError(null);
    createDiagnosticExport().then((result) => setDiagnosticResult(result.path)).catch(() => setDiagnosticError("Diagnostic export was not created; no recovery state was changed.")).finally(() => setExporting(false));
  };
  return (
    <main className="gate-shell gate-shell--recovery">
      <Brand />
      <p className="eyebrow">恢复模式</p>
      <h1 ref={headingRef} tabIndex={-1}>{t("shell.normalBlocked")}</h1>
      <p>
        ACM-OS could not prove that System Facts are safe to use. Normal navigation stays hidden
        so the application cannot continue in a partially valid state.
      </p>
      <section aria-labelledby="recovery-detail" className="gate-panel" role="alert">
        <h2 id="recovery-detail">诊断状态</h2>
        <dl className="detail-list">
          <dt>原因</dt>
          <dd>{reason}</dd>
          {supportedSchemaVersion !== null ? (
            <>
              <dt>支持的数据库结构版本</dt>
              <dd>{supportedSchemaVersion}</dd>
            </>
          ) : null}
          {foundSchemaVersion !== null ? (
            <>
              <dt>检测到的数据库结构版本</dt>
              <dd>{foundSchemaVersion}</dd>
            </>
          ) : null}
        </dl>
      </section>
      <p className="safe-note">No automatic repair or destructive action is performed in B0.4.</p>
      <section aria-labelledby="recovery-tools" className="gate-panel">
        <h2 id="recovery-tools">恢复诊断</h2>
        <p>生成经过隐私过滤的 JSON 诊断包，供人工检查或技术支持使用。</p>
        <div className="action-row">
          <button className="secondary-action" onClick={inspectDiagnostics} type="button">预览导出</button>
          <button className="primary-action" disabled={exporting} onClick={exportDiagnostics} type="button">{exporting ? "Exporting…" : "Create diagnostic export"}</button>
        </div>
        {diagnosticPreview ? <p className="system-caption">Output directory: {diagnosticPreview.outputDirectory}; sections: {diagnosticPreview.sections.length}</p> : null}
        {diagnosticResult ? <p aria-live="polite" className="safe-note">Created: {diagnosticResult}</p> : null}
        {diagnosticError ? <p role="alert" className="error-message">{diagnosticError}</p> : null}
      </section>
    </main>
  );
}

export function SetupShell({
  foundation,
  onConfigured,
}: {
  foundation: FoundationStatus;
  onConfigured: (workspace: ConfiguredWorkspace) => void;
}) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [draft, setDraft] = useState(EMPTY_WORKSPACE_DRAFT);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [issue, setIssue] = useState<WorkspaceConfigurationErrorDto | null>(null);
  const savingRef = useRef(false);
  const activeVaultRef = useRef<HTMLInputElement>(null);
  const problemRootRef = useRef<HTMLInputElement>(null);
  const knowledgeRootRef = useRef<HTMLInputElement>(null);

  const updateDraft = (field: keyof WorkspaceConfigurationDraft, value: string) => {
    setDraft((current) => ({ ...current, [field]: value }));
  };

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (savingRef.current) return;
    savingRef.current = true;
    setSaving(true);
    setError(null);
    setIssue(null);
    try {
      const workspace = await configureWorkspace(draft);
      if (workspace.state !== "configured") {
        throw new Error("Workspace configuration returned an invalid state");
      }
      onConfigured(workspace);
    } catch (cause: unknown) {
      const nextIssue = parseWorkspaceConfigurationError(cause);
      setIssue(nextIssue);
      setError(describeWorkspaceError(cause));
      focusField(nextIssue?.field ?? null, {
        active_vault: activeVaultRef,
        problem_root: problemRootRef,
        knowledge_root: knowledgeRootRef,
      });
    } finally {
      savingRef.current = false;
      setSaving(false);
    }
  };

  const fieldHasError = (field: WorkspacePathField) => issue?.field === field;

  return (
    <main className="gate-shell gate-shell--setup">
      <Brand />
      <p className="eyebrow">Setup shell · Workspace</p>
      <h1 ref={headingRef} tabIndex={-1}>Connect your workspace</h1>
      <p>
        Choose one existing Vault and two existing, non-overlapping folders inside it. ACM-OS
        will not scan or modify Markdown during setup.
      </p>
      <form className="workspace-form gate-panel" onSubmit={submit}>
        <WorkspaceField
          describedBy="active-vault-error"
          error={fieldHasError("active_vault") ? error : null}
          inputRef={activeVaultRef}
          label="Active Vault"
          onChange={(value) => updateDraft("activeVaultPath", value)}
          value={draft.activeVaultPath}
        />
        <WorkspaceField
          describedBy="problem-root-error"
          error={fieldHasError("problem_root") ? error : null}
          inputRef={problemRootRef}
          label="Problem Notes Root"
          onChange={(value) => updateDraft("problemRootPath", value)}
          value={draft.problemRootPath}
        />
        <WorkspaceField
          describedBy="knowledge-root-error"
          error={fieldHasError("knowledge_root") ? error : null}
          inputRef={knowledgeRootRef}
          label="Knowledge Root"
          onChange={(value) => updateDraft("knowledgeRootPath", value)}
          value={draft.knowledgeRootPath}
        />
        <button className="primary-action" disabled={saving} type="submit">
          {saving ? "Validating workspace…" : "Save and enter ACM-OS"}
        </button>
        {error && (!issue || issue.field === null) ? (
          <p aria-live="assertive" className="error-message">
            {error}
          </p>
        ) : null}
      </form>
      <p className="system-caption">
        Core boundary: {foundationCaption(foundation)}
      </p>
    </main>
  );
}

export function NormalAppShell({
  route,
  workspace,
  foundation,
  navigate,
}: {
  route: Exclude<AppRoute, { kind: "review" }>;
  workspace: ConfiguredWorkspace;
  foundation: FoundationStatus;
  navigate: Navigate;
}) {
  return (
    <div className="normal-shell">
      <aside className="app-sidebar">
        <Brand />
      <a className="skip-link" href="#main-content">{t("shell.skipContent")}</a>
        <nav aria-label={t("shell.primaryNav")}>
          <ShellLink active={route.kind === "normal" && route.page === "today"} href="/today" navigate={navigate}>{t("nav.today")}</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "contests"} href="/contests" navigate={navigate}>{t("nav.contests")}</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "problems"} href="/problems" navigate={navigate}>{t("nav.problems")}</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "knowledge"} href="/knowledge" navigate={navigate}>{t("nav.knowledge")}</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "reward"} href="/reward" navigate={navigate}>{t("nav.reward")}</ShellLink>
        </nav>
        <nav aria-label={t("shell.toolsNav")} className="tool-nav">
          <ShellLink active={route.kind === "normal" && route.page === "settings"} href="/settings" navigate={navigate}>{t("nav.settings")}</ShellLink>
        </nav>
        <p className="system-caption">
          System Facts {foundationCaption(foundation)}
        </p>
      </aside>
      <main className="normal-content" id="main-content" tabIndex={-1}>
        {route.kind === "normal" ? (
          <NormalPageContent navigate={navigate} page={route.page} workspace={workspace} />
        ) : route.kind === "contestDetail" ? (
          <ContestDetail contestId={route.contestId} navigate={navigate} />
        ) : route.kind === "problemDetail" ? (
          <ProblemDetail contestId={route.contestId} index={route.index} navigate={navigate} />
        ) : route.kind === "canonicalProblemDetail" ? (
          <ProblemDetail problemId={route.problemId} navigate={navigate} />
        ) : (
          <NotFoundContent pathname={route.pathname} navigate={navigate} />
        )}
      </main>
    </div>
  );
}

export function ReviewFocusShell({ attemptId, navigate }: { attemptId: string; navigate: Navigate }) {
  const mainRef = useRouteFocus<HTMLDivElement>();
  const [focus, setFocus] = useState<ReviewFocusDto | null>(null);
  const [renderedHtml, setRenderedHtml] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const [helpDrawer, setHelpDrawer] = useState<ReviewHelpDrawerDto | null>(null);
  const [helpError, setHelpError] = useState<string | null>(null);
  const [pendingHelp, setPendingHelp] = useState<ReviewHelpItemDto | null>(null);
  const [revealedHelp, setRevealedHelp] = useState<RevealedReviewHelpDto[]>([]);
  const [revealingLevel, setRevealingLevel] = useState<ReviewHelpLevel | null>(null);
  const [completedReview, setCompletedReview] = useState<CompletedReviewAttemptDto | null>(null);
  const [terminalHistory, setTerminalHistory] = useState<CanonicalReviewHistoryItemDto | null>(null);
  const [completion, setCompletion] = useState<CompleteReviewInputDto>(() => emptyReviewCompletion(attemptId));
  const [completionError, setCompletionError] = useState<string | null>(null);
  const [ojOpenError, setOjOpenError] = useState<string | null>(null);
  const [completing, setCompleting] = useState(false);
  const [voidOpen, setVoidOpen] = useState(false);
  const [voidReason, setVoidReason] = useState("");
  const helpHeadingRef = useRef<HTMLHeadingElement>(null);
  const helpButtonRef = useRef<HTMLButtonElement>(null);
  const helpDrawerRef = useRef<HTMLElement>(null);
  const helpConfirmRef = useRef<HTMLDivElement>(null);
  const helpConfirmButtonRef = useRef<HTMLButtonElement>(null);
  const voidButtonRef = useRef<HTMLButtonElement>(null);
  const voidDialogRef = useRef<HTMLDivElement>(null);
  const voidReasonRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    let active = true;
    const objectUrls: string[] = [];
    getReviewFocus(attemptId).then((nextFocus) => {
      if (!active) return;
      const assetUrls = new Map<string, string>();
      for (const asset of nextFocus.statementAssets) {
        const objectUrl = URL.createObjectURL(
          new Blob([new Uint8Array(asset.bytes)], { type: asset.mediaType }),
        );
        objectUrls.push(objectUrl);
        assetUrls.set(asset.localRef, objectUrl);
      }
      setFocus(nextFocus);
      setRenderedHtml(sanitizeStatementForRender(nextFocus.statementSanitizedHtml, assetUrls));
    }).catch(() => {
      if (!active) return;
      getReviewAttemptHistory(attemptId)
        .then((history) => { if (active && history.status !== "inProgress") setTerminalHistory(history); })
        .catch(() => { if (active) setFailed(true); });
    });
    return () => {
      active = false;
      for (const objectUrl of objectUrls) URL.revokeObjectURL(objectUrl);
    };
  }, [attemptId]);

  useEffect(() => {
    if (helpOpen) helpHeadingRef.current?.focus();
  }, [helpOpen, helpDrawer]);

  useEffect(() => {
    const container = voidOpen
      ? voidDialogRef.current
      : pendingHelp
        ? helpConfirmRef.current
        : helpOpen
          ? helpDrawerRef.current
          : null;
    if (!container) return;
    const initial = voidOpen
      ? voidReasonRef.current
      : pendingHelp
        ? helpConfirmButtonRef.current
        : helpHeadingRef.current;
    initial?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        if (voidOpen) {
          setVoidOpen(false);
          queueMicrotask(() => voidButtonRef.current?.focus());
        } else if (pendingHelp) {
          setPendingHelp(null);
          queueMicrotask(() => helpHeadingRef.current?.focus());
        } else {
          closeHelpDrawer();
        }
        return;
      }
      if (event.key !== "Tab") return;
      const focusable = [...container.querySelectorAll<HTMLElement>(
        'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])',
      )];
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [helpOpen, pendingHelp, voidOpen]);

  function openHelpDrawer() {
    setHelpOpen(true);
    setHelpDrawer(null);
    setHelpError(null);
    getReviewHelpDrawer(attemptId)
      .then(setHelpDrawer)
      .catch(() => setHelpError("Help availability could not be read. No help usage was recorded."));
  }

  function openOriginalOjFromReview(event: MouseEvent<HTMLAnchorElement>) {
    event.preventDefault();
    if (!focus) return;
    setOjOpenError(null);
    openOriginalOj(focus.sourceUrl).catch((error: unknown) => {
      setOjOpenError(String(error).includes("unsafe_external_url")
        ? "The original OJ link was rejected because it is not an HTTPS Codeforces URL."
        : "The original OJ could not be opened. The Review Attempt remains unchanged.");
    });
  }

  function closeHelpDrawer() {
    setHelpOpen(false);
    setPendingHelp(null);
    queueMicrotask(() => helpButtonRef.current?.focus());
  }

  function performReveal(item: ReviewHelpItemDto, impactAcknowledged: boolean) {
    setRevealingLevel(item.level);
    setHelpError(null);
    revealReviewHelp(attemptId, item.level, impactAcknowledged)
      .then((revealed) => {
        setRevealedHelp((current) => [
          ...current.filter((entry) => entry.level !== revealed.level),
          revealed,
        ]);
        setHelpDrawer((current) => current && ({
          ...current,
          items: current.items.map((entry) => entry.level === revealed.level
            ? { ...entry, revealedAtUtc: revealed.revealedAtUtc }
            : entry),
        }));
        setPendingHelp(null);
      })
      .catch((error: unknown) => {
        setHelpError(String(error).includes("review_help_confirmation_required")
          ? "Confirm the Review consequence before revealing this help."
          : "Help was not revealed and no usage content was released.");
      })
      .finally(() => setRevealingLevel(null));
  }

  function requestReveal(item: ReviewHelpItemDto) {
    if (item.revealedAtUtc) performReveal(item, false);
    else setPendingHelp(item);
  }

  function submitCompletion(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setCompleting(true);
    setCompletionError(null);
    completeReview(completion)
      .then((result) => {
        setCompletedReview(result);
        setHelpOpen(false);
      })
      .catch((error: unknown) => {
        const code = String(error);
        setCompletionError(code.includes("review_failure_reason_required")
          ? "This Review is Partial or Not passed. Select at least one failure reason; your other facts were preserved."
          : code.includes("review_completion_facts_invalid")
            ? "The submitted facts contradict each other or are incomplete. The Attempt remains in progress."
            : "Review completion failed. The Attempt remains in progress and the form was preserved.");
      })
      .finally(() => setCompleting(false));
  }

  function confirmVoid() {
    setCompletionError(null);
    voidReview(attemptId, voidReason)
      .then((history) => {
        setTerminalHistory(history);
        setVoidOpen(false);
      })
      .catch(() => setCompletionError("The Attempt was not voided. Use Void only for a genuinely mistaken start."));
  }
  return (
    <main className="review-shell" ref={mainRef} tabIndex={-1}>
      <header className="review-header">
        <div>
          <p className="eyebrow">M4 · 复习专注</p>
          <h1>{focus ? focus.title : t("review.focusWorkspace")}</h1>
        </div>
        <button className="secondary-action" onClick={() => navigate("/today")} type="button">
          返回今日计划
        </button>
      </header>
      {completedReview ? (
        <ReviewEvidenceCard completed={completedReview} />
      ) : terminalHistory ? (
        <ReviewHistoryEvidenceCard item={terminalHistory} />
      ) : failed ? (
        <section className="review-stage" role="alert">
          <h2>复习记录不可用</h2>
          <p>复习结果和学习状态均未改变。</p>
        </section>
      ) : !focus || renderedHtml === null ? (
        <section aria-busy="true" className="review-stage"><p>{t("review.loadingStatement")}</p></section>
      ) : (
        <>
          <section aria-labelledby="review-attempt-metadata" className="review-stage">
            <h2 id="review-attempt-metadata">首次冷启动复习</h2>
            <p>
              {reviewAttemptTypeLabel(focus.attempt.attemptType)} · 计划日期 {focus.attempt.scheduledDueLocalDate}
              {focus.attempt.startedEarly ? " · 已提前开始" : ""}
            </p>
            <a href={focus.sourceUrl} onClick={openOriginalOjFromReview} rel="noreferrer" target="_blank">{t("review.openOj")}</a>
            {ojOpenError ? <p role="alert">{ojOpenError}</p> : null}
            <button className="secondary-action" onClick={openHelpDrawer} ref={helpButtonRef} type="button">{t("review.openHelp")}</button>
            <p className="safe-note">旧笔记、提示、题解、比赛历史和复习历史不会加载到此专注视图。</p>
          </section>
          <section className="review-stage statement-view" aria-labelledby="review-statement-heading">
            <div className="statement-heading-row"><h2 id="review-statement-heading">题面快照</h2></div>
            <div dangerouslySetInnerHTML={{ __html: renderedHtml }} />
          </section>
          <form className="review-stage review-facts-form" onSubmit={submitCompletion}>
            <div>
              <p className="eyebrow">依据事实，而不是自选评分</p>
              <h2>完成本次复习</h2>
              <p>系统根据这些事实和已记录的帮助使用情况推导掌握、部分掌握或未通过。</p>
            </div>
            <fieldset>
              <legend>提交事实</legend>
              <label><input checked={completion.finalAc} onChange={(event) => setCompletion({ ...completion, finalAc: event.target.checked })} type="checkbox" /> 最终结果为 AC</label>
              <label>First submission result<select value={completion.firstSubmissionResult} onChange={(event) => { const result = event.target.value as SubmissionResultDto; setCompletion({ ...completion, firstSubmissionResult: result, firstSubmissionOther: result === "other" ? completion.firstSubmissionOther : null }); }}>{submissionResultOptions()}</select></label>
              {completion.firstSubmissionResult === "other" ? <label>First result detail<input maxLength={120} onChange={(event) => setCompletion({ ...completion, firstSubmissionOther: event.target.value })} required value={completion.firstSubmissionOther ?? ""} /></label> : null}
              <label>Final result<select value={completion.finalResult} onChange={(event) => { const result = event.target.value as SubmissionResultDto; setCompletion({ ...completion, finalResult: result, finalResultOther: result === "other" ? completion.finalResultOther : null }); }}>{submissionResultOptions()}</select></label>
              {completion.finalResult === "other" ? <label>Final result detail<input maxLength={120} onChange={(event) => setCompletion({ ...completion, finalResultOther: event.target.value })} required value={completion.finalResultOther ?? ""} /></label> : null}
              <label>Total submissions<input min="1" onChange={(event) => setCompletion({ ...completion, totalSubmissions: Number(event.target.value) })} required type="number" value={completion.totalSubmissions} /></label>
            </fieldset>
            <fieldset>
              <legend>独立性</legend>
              <label><input checked={completion.ideaIndependent} onChange={(event) => setCompletion({ ...completion, ideaIndependent: event.target.checked })} type="checkbox" /> Idea was independent</label>
              <label><input checked={completion.implementationIndependent} onChange={(event) => setCompletion({ ...completion, implementationIndependent: event.target.checked })} type="checkbox" /> Implementation was independent</label>
              <label>Debug<select value={completion.debugIndependence} onChange={(event) => setCompletion({ ...completion, debugIndependence: event.target.value as CompleteReviewInputDto["debugIndependence"] })}><option value="notNeeded">No debug needed</option><option value="independent">Debugged independently</option><option value="usedSolvingHelp">Used problem-solving help to debug</option></select></label>
              <label>Unrecorded external help<select value={completion.externalHelp} onChange={(event) => setCompletion({ ...completion, externalHelp: event.target.value as CompleteReviewInputDto["externalHelp"] })}><option value="none">None</option><option value="solvingHint">Problem-solving hint</option><option value="fullSolution">Full solution</option></select></label>
            </fieldset>
            <fieldset>
              <legend>失败原因</legend>
              <p>Select at least one when the derived result may be Partial or Not passed.</p>
              {reviewFailureReasonOptions.map(([code, label]) => <label key={code}><input checked={completion.failureReasons.some((reason) => reason.code === code)} onChange={(event) => setCompletion({ ...completion, failureReasons: event.target.checked ? [...completion.failureReasons, { code, otherText: null }] : completion.failureReasons.filter((reason) => reason.code !== code) })} type="checkbox" /> {label}</label>)}
              {completion.failureReasons.some((reason) => reason.code === "other") ? <label>Other reason<input maxLength={500} onChange={(event) => setCompletion({ ...completion, failureReasons: completion.failureReasons.map((reason) => reason.code === "other" ? { ...reason, otherText: event.target.value } : reason) })} required value={completion.failureReasons.find((reason) => reason.code === "other")?.otherText ?? ""} /></label> : null}
            </fieldset>
            {completionError ? <p role="alert">{completionError}</p> : null}
            <div className="button-row"><button disabled={completing} type="submit">{completing ? "Completing…" : "Complete from facts"}</button><button className="secondary-action" onClick={() => setVoidOpen(true)} ref={voidButtonRef} type="button">Void mistaken Attempt</button></div>
          </form>
          {[...revealedHelp]
            .sort((left, right) => left.level - right.level)
            .map((revealed) => (
              <section className="review-stage review-help-content" key={revealed.level} aria-labelledby={`revealed-help-${revealed.level}`}>
                <h2 id={`revealed-help-${revealed.level}`}>Level {revealed.level} · {revealed.title}</h2>
                <p>{t("review.usageRecorded")} {revealed.revealedAtUtc}。</p>
                <pre>{revealed.contentMarkdown}</pre>
              </section>
            ))}
        </>
      )}
      {helpOpen ? (
        <aside aria-describedby="review-help-description" aria-labelledby="review-help-title" aria-modal="true" className="review-help-drawer" ref={helpDrawerRef} role="dialog">
          <div className="review-help-drawer__header">
            <div>
              <p className="eyebrow">查看前的影响说明</p>
              <h2 id="review-help-title" ref={helpHeadingRef} tabIndex={-1}>{t("review.helpTitle")}</h2>
            </div>
            <button className="secondary-action" onClick={closeHelpDrawer} type="button">{t("review.close")}</button>
          </div>
          <p id="review-help-description">Opening this drawer records nothing. A successful Reveal creates an irreversible usage event before content appears.</p>
          {helpError ? <p role="alert">{helpError}</p> : null}
          {!helpDrawer && !helpError ? <p aria-busy="true">Checking current Markdown…</p> : null}
          <ol className="review-help-levels">
            {helpDrawer?.items.map((item) => (
              <li key={item.level}>
                <div><strong>Level {item.level} · {reviewHelpLevelLabel(item.level)}</strong><span>{reviewHelpConsequence(item.consequence)}</span></div>
                <button
                  disabled={!item.available || revealingLevel !== null}
                  onClick={() => requestReveal(item)}
                  type="button"
                >
                  {revealingLevel === item.level ? "Recording…" : item.revealedAtUtc ? "Open again" : item.available ? "Reveal" : "Unavailable"}
                </button>
              </li>
            ))}
          </ol>
          {pendingHelp ? (
            <div aria-describedby="review-help-confirm-description" aria-labelledby="review-help-confirm-title" aria-modal="true" className="review-help-confirm" ref={helpConfirmRef} role="alertdialog">
              <h3 id="review-help-confirm-title">Reveal Level {pendingHelp.level}?</h3>
              <p id="review-help-confirm-description">
                {pendingHelp.level === 5
                  ? "Viewing the full solution means this Attempt can only be judged Not passed."
                  : "Using this problem-solving help means this Attempt can be judged Partial at best."}
              </p>
              <div className="button-row">
                <button onClick={() => performReveal(pendingHelp, true)} ref={helpConfirmButtonRef} type="button">{t("review.confirmReveal")}</button>
                <button className="secondary-action" onClick={() => { setPendingHelp(null); queueMicrotask(() => helpHeadingRef.current?.focus()); }} type="button">Cancel</button>
              </div>
            </div>
          ) : null}
        </aside>
      ) : null}
      {voidOpen ? (
        <div className="modal-backdrop"><div aria-describedby="void-review-description" aria-labelledby="void-review-title" aria-modal="true" ref={voidDialogRef} role="alertdialog"><h2 id="void-review-title">Void this Attempt?</h2><p id="void-review-description">Only use this for an accidental start. A real attempt that did not succeed must be completed as Not passed. The Void record and any revealed help remain in history; scheduling is unchanged.</p><label>Reason<input onChange={(event) => setVoidReason(event.target.value)} ref={voidReasonRef} value={voidReason} /></label><div className="button-row"><button disabled={!voidReason.trim()} onClick={confirmVoid} type="button">Void mistaken Attempt</button><button className="secondary-action" onClick={() => { setVoidOpen(false); queueMicrotask(() => voidButtonRef.current?.focus()); }} type="button">Cancel</button></div></div></div>
      ) : null}
    </main>
  );
}

function NormalPageContent({ page, workspace, navigate }: { page: NormalPage; workspace: ConfiguredWorkspace; navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  if (page === "today") {
    return <TodayPage navigate={navigate} />;
  }
  if (page === "settings") {
    return (
      <>
        <PageHeader eyebrow="工具" headingRef={headingRef} title={t("nav.settings")} />
        <section aria-labelledby="workspace-settings" className="content-panel">
          <h2 id="workspace-settings">工作区</h2>
          <dl className="detail-list detail-list--paths">
             <dt>当前 Vault</dt><dd>{workspace.activeVaultPath}</dd>
             <dt>题目笔记目录</dt><dd>{workspace.problemRootPath}</dd>
             <dt>知识库目录</dt><dd>{workspace.knowledgeRootPath}</dd>
          </dl>
          <p className="safe-note">修改当前 Vault 需要经过预览和确认流程。</p>
         </section>
         <ManualBackupSettings />
         <WeeklyAcmBudgetSettings />
      </>
    );
  }
  if (page === "contests") return <ContestLibraryPage navigate={navigate} />;
  if (page === "problems") return <ProblemIndex navigate={navigate} />;
  if (page === "reward") return <RewardPage />;
  return <KnowledgePage navigate={navigate} />;
}

type RewardActivationView = "loading" | "inactive" | "active" | "error";

function RewardPage() {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const activationTriggerRef = useRef<HTMLButtonElement>(null);
  const activationDialogRef = useRef<HTMLDivElement>(null);
  const activationConfirmRef = useRef<HTMLButtonElement>(null);
  const activatingRef = useRef(false);
  const [activationView, setActivationView] = useState<RewardActivationView>("loading");
  const [account, setAccount] = useState<RewardAccountSummaryDto | null>(null);
  const [accountLoading, setAccountLoading] = useState(false);
  const [accountError, setAccountError] = useState(false);
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const [activating, setActivating] = useState(false);
  const [activationError, setActivationError] = useState(false);
  const [activationSuccess, setActivationSuccess] = useState(false);

  const loadAccount = useCallback(async () => {
    setAccountLoading(true);
    setAccountError(false);
    setAccount(null);
    try {
      setAccount(await getRewardAccountSummary());
    } catch {
      setAccountError(true);
    } finally {
      setAccountLoading(false);
    }
  }, []);

  const refreshRewardAccount = useCallback(() => { void loadAccount(); }, [loadAccount]);

  const loadActivation = useCallback(async () => {
    setActivationView("loading");
    setAccount(null);
    setAccountError(false);
    try {
      const state = await getRewardActivationState();
      setActivationView(state.active ? "active" : "inactive");
      if (state.active) await loadAccount();
    } catch {
      setActivationView("error");
    }
  }, [loadAccount]);

  useEffect(() => { void loadActivation(); }, [loadActivation]);

  const closeConfirmation = useCallback(() => {
    if (activatingRef.current) return;
    setConfirmationOpen(false);
    setActivationError(false);
    queueMicrotask(() => activationTriggerRef.current?.focus());
  }, []);

  useEffect(() => {
    if (!confirmationOpen) return;
    const dialog = activationDialogRef.current;
    activationConfirmRef.current?.focus();
    if (!dialog) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        closeConfirmation();
        return;
      }
      if (event.key !== "Tab") return;
      const focusable = [...dialog.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [tabindex]:not([tabindex="-1"])',
      )];
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [closeConfirmation, confirmationOpen]);

  const confirmActivation = async () => {
    if (activatingRef.current) return;
    activatingRef.current = true;
    setActivating(true);
    setActivationError(false);
    setActivationSuccess(false);
    try {
      await activateReward();
      setConfirmationOpen(false);
      setActivationSuccess(true);
      await loadActivation();
    } catch {
      setActivationError(true);
    } finally {
      activatingRef.current = false;
      setActivating(false);
    }
  };

  return <>
    <PageHeader eyebrow={t("reward.account")} headingRef={headingRef} title={t("reward.pageTitle")} />
    {activationSuccess ? <p aria-live="polite" className="safe-note">{t("reward.enabled")}</p> : null}
    {activationView === "loading" ? <p aria-live="polite">{t("reward.loadingMode")}</p> : null}
    {activationView === "error" ? (
      <section className="empty-state" role="alert">
        <h2>{t("reward.unavailable")}</h2>
        <p>{t("reward.settingsUnchanged")}</p>
        <button className="secondary-action" onClick={() => void loadActivation()} type="button">{t("reward.retry")}</button>
      </section>
    ) : null}
    {activationView === "inactive" ? (
      <section aria-labelledby="reward-inactive-heading" className="content-panel">
        <h2 id="reward-inactive-heading">{t("reward.off")}</h2>
        <p>{t("reward.enableDescription")}</p>
        <p>{t("reward.retroactiveDescription")}</p>
        <button className="primary-action" onClick={() => { setActivationError(false); setConfirmationOpen(true); }} ref={activationTriggerRef} type="button">{t("reward.enableMode")}</button>
      </section>
    ) : null}
    {activationView === "active" ? (
      <>
        <section aria-labelledby="reward-account-heading" className="content-panel">
          <h2 id="reward-account-heading">{t("reward.account")}</h2>
          {accountLoading ? <p aria-live="polite">{t("reward.accountLoading")}</p> : null}
          {accountError ? <div role="alert"><p>{t("reward.accountError")}</p><button className="secondary-action" onClick={() => void loadAccount()} type="button">{t("common.retry")}</button></div> : null}
          {account ? <dl className="detail-list"><dt>{t("reward.level")}</dt><dd>{account.level}</dd><dt>{t("reward.xpLabel")}</dt><dd>{account.xp}</dd><dt>{t("reward.coin")}</dt><dd>{account.coin}</dd></dl> : null}
        </section>
        <CustomRewardManagement account={account} onTransactionResolved={refreshRewardAccount} />
      </>
    ) : null}
    {confirmationOpen ? (
      <div className="modal-backdrop">
        <div aria-describedby="reward-activation-description" aria-labelledby="reward-activation-title" aria-modal="true" ref={activationDialogRef} role="alertdialog">
          <h2 id="reward-activation-title">{t("reward.enableModeQuestion")}</h2>
          <p id="reward-activation-description">{t("reward.activationDescription")}</p>
          {activationError ? <p className="error-message" role="alert">{t("reward.activationError")}</p> : null}
          <div className="button-row">
            <button className="primary-action" disabled={activating} onClick={() => void confirmActivation()} ref={activationConfirmRef} type="button">{activating ? t("reward.enabling") : t("reward.enableMode")}</button>
            <button className="secondary-action" disabled={activating} onClick={closeConfirmation} type="button">{t("common.cancel")}</button>
          </div>
        </div>
      </div>
    ) : null}
  </>;
}

function CustomRewardManagement({ account, onTransactionResolved }: { account: RewardAccountSummaryDto | null; onTransactionResolved: () => void }) {
  const archiveTriggerRef = useRef<HTMLButtonElement>(null);
  const archiveDialogRef = useRef<HTMLDivElement>(null);
  const archiveConfirmRef = useRef<HTMLButtonElement>(null);
  const mutationPendingRef = useRef(false);
  const [rewards, setRewards] = useState<CustomRewardDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [showArchived, setShowArchived] = useState(false);
  const [name, setName] = useState("");
  const [coinCost, setCoinCost] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editCoinCost, setEditCoinCost] = useState("");
  const [archiveTarget, setArchiveTarget] = useState<CustomRewardDto | null>(null);
  const [pending, setPending] = useState(false);
  const [mutationError, setMutationError] = useState<string | null>(null);
  const [invalidFields, setInvalidFields] = useState({ name: false, coinCost: false });
  const load = useCallback(async () => { setLoading(true); setError(false); try { setRewards(await listCustomRewards()); } catch { setError(true); } finally { setLoading(false); } }, []);
  useEffect(() => { void load(); }, [load]);
  const closeArchive = useCallback(() => {
    if (mutationPendingRef.current) return;
    setArchiveTarget(null);
    queueMicrotask(() => archiveTriggerRef.current?.focus());
  }, []);
  useEffect(() => {
    if (!archiveTarget) return;
    archiveConfirmRef.current?.focus();
    const dialog = archiveDialogRef.current;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") { event.preventDefault(); closeArchive(); return; }
      if (event.key !== "Tab" || !dialog) return;
      const focusable = [...dialog.querySelectorAll<HTMLElement>('button:not([disabled]), [tabindex]:not([tabindex="-1"])')];
      if (!focusable.length) return;
      const first = focusable[0]; const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [archiveTarget, closeArchive]);
  const parseCost = (value: string) => { if (!/^\d+$/.test(value)) return null; const n = Number(value); return Number.isSafeInteger(n) && n > 0 ? n : null; };
  const validate = (nextName: string, nextCost: string) => { const invalid = { name: !nextName.trim(), coinCost: parseCost(nextCost) === null }; setInvalidFields(invalid); if (invalid.name || invalid.coinCost) { setMutationError(t("reward.validation")); return null; } return { name: nextName.trim(), coinCost: Number(nextCost) }; };
  const create = async (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); if (mutationPendingRef.current) return; setMutationError(null); const input = validate(name, coinCost); if (!input) return; mutationPendingRef.current = true; setPending(true); try { await createCustomReward(input); setName(""); setCoinCost(""); setInvalidFields({ name: false, coinCost: false }); await load(); } catch { setMutationError(t("reward.changeError")); } finally { mutationPendingRef.current = false; setPending(false); } };
  const edit = (reward: CustomRewardDto) => { setEditingId(reward.customRewardId); setEditName(reward.name); setEditCoinCost(String(reward.coinCost)); setInvalidFields({ name: false, coinCost: false }); setMutationError(null); };
  const update = async (event: FormEvent<HTMLFormElement>, reward: CustomRewardDto) => { event.preventDefault(); if (mutationPendingRef.current) return; setMutationError(null); const values = validate(editName, editCoinCost); if (!values) return; mutationPendingRef.current = true; setPending(true); try { await updateCustomReward({ customRewardId: reward.customRewardId, ...values }); setEditingId(null); setInvalidFields({ name: false, coinCost: false }); await load(); } catch { setMutationError(t("reward.editConflict")); await load(); } finally { mutationPendingRef.current = false; setPending(false); } };
  const archive = async () => { if (!archiveTarget || mutationPendingRef.current) return; mutationPendingRef.current = true; setPending(true); setMutationError(null); try { await archiveCustomReward(archiveTarget.customRewardId); setArchiveTarget(null); await load(); } catch { setMutationError(t("reward.changeError")); await load(); } finally { mutationPendingRef.current = false; setPending(false); } };
  const visible = rewards.filter((reward) => showArchived || reward.status === "active");
  return <section aria-labelledby="custom-rewards-heading" className="content-panel">
    <div className="statement-heading-row"><h2 id="custom-rewards-heading">{t("reward.customRewards")}</h2><label><input checked={showArchived} onChange={(event) => setShowArchived(event.currentTarget.checked)} type="checkbox" /> {t("reward.showArchived")}</label></div>
    {mutationError ? <p aria-live="assertive" className="error-message" id="custom-reward-error" role="alert">{mutationError}</p> : null}
    <form className="action-row" noValidate onSubmit={create}><label>{t("reward.name")}<input aria-describedby={invalidFields.name ? "custom-reward-error" : undefined} aria-invalid={invalidFields.name} aria-label={t("reward.customNameAria")} onInput={(event) => { setName(event.currentTarget.value); if (invalidFields.name) setInvalidFields((current) => ({ ...current, name: false })); }} value={name} /></label><label>{t("reward.coinCost")}<input aria-describedby={invalidFields.coinCost ? "custom-reward-error" : undefined} aria-invalid={invalidFields.coinCost} aria-label={t("reward.customCoinCostAria")} inputMode="numeric" onInput={(event) => { setCoinCost(event.currentTarget.value); if (invalidFields.coinCost) setInvalidFields((current) => ({ ...current, coinCost: false })); }} value={coinCost} /></label><button className="primary-action" disabled={pending} type="submit">{t("reward.create")}</button></form>
    {loading ? <p aria-live="polite">{t("reward.loadingCustom")}</p> : null}
    {error ? <div role="alert"><p>{t("reward.customLoadError")}</p><button className="secondary-action" onClick={() => void load()} type="button">{t("common.retry")}</button></div> : null}
    {!loading && !error && visible.length === 0 ? <p>{showArchived ? t("reward.noCustomAll") : t("reward.noCustomActive")}</p> : null}
    {!loading && !error && visible.length > 0 ? <ul className="detail-list">{visible.map((reward) => <li key={reward.customRewardId}><div><strong>{reward.name}</strong><span>{t("reward.coinCostValue", { cost: reward.coinCost })}</span><span>{reward.status === "archived" ? t("reward.archived") : t("reward.active")}</span></div>{reward.status === "active" ? <div className="action-row"><button className="secondary-action" onClick={() => edit(reward)} type="button">{t("reward.edit")}</button><button className="danger-action" onClick={(event) => { archiveTriggerRef.current = event.currentTarget; setArchiveTarget(reward); }} type="button">{t("reward.archive")}</button></div> : null}{editingId === reward.customRewardId ? <form className="action-row" noValidate onSubmit={(event) => void update(event, reward)}><label>{t("reward.name")}<input aria-describedby={invalidFields.name ? "custom-reward-error" : undefined} aria-invalid={invalidFields.name} aria-label={t("reward.editNameAria")} onInput={(event) => { setEditName(event.currentTarget.value); if (invalidFields.name) setInvalidFields((current) => ({ ...current, name: false })); }} value={editName} /></label><label>{t("reward.coinCost")}<input aria-describedby={invalidFields.coinCost ? "custom-reward-error" : undefined} aria-invalid={invalidFields.coinCost} aria-label={t("reward.editCoinCostAria")} onInput={(event) => { setEditCoinCost(event.currentTarget.value); if (invalidFields.coinCost) setInvalidFields((current) => ({ ...current, coinCost: false })); }} value={editCoinCost} /></label><button className="primary-action" disabled={pending} type="submit">{t("reward.saveChanges")}</button><button className="secondary-action" disabled={pending} onClick={() => { setEditingId(null); setInvalidFields({ name: false, coinCost: false }); setMutationError(null); }} type="button">{t("common.cancel")}</button></form> : null}</li>)}</ul> : null}
    {archiveTarget ? <div className="modal-backdrop"><div aria-describedby="archive-reward-description" aria-labelledby="archive-reward-title" aria-modal="true" ref={archiveDialogRef} role="alertdialog"><h2 id="archive-reward-title">{t("reward.archiveQuestion", { name: archiveTarget.name })}</h2><p id="archive-reward-description">{t("reward.archiveWarning")}</p><div className="button-row"><button className="danger-action" disabled={pending} onClick={() => void archive()} ref={archiveConfirmRef} type="button">{pending ? t("reward.archiving") : t("reward.archiveReward")}</button><button className="secondary-action" disabled={pending} onClick={closeArchive} type="button">{t("common.cancel")}</button></div></div></div> : null}
    <RewardTransactions account={account} onTransactionResolved={onTransactionResolved} />
  </section>;
}
function rewardErrorCode(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") return error.message;
  return "unknown";
}

function transactionMessage(error: unknown, kind: "redeem" | "refund"): string {
  const code = rewardErrorCode(error);
  if (code === "reward_inactive") return t("reward.errorInactive");
  if (code === "custom_reward_not_found") return t("reward.errorNotFound");
  if (code === "custom_reward_archived") return t("reward.errorArchived");
  if (kind === "refund" && code === "redemption_not_found") return t("reward.errorRedemptionMissing");
  if (kind === "refund" && code === "already_refunded") return t("reward.errorAlreadyRefunded");
  if (kind === "redeem" && code === "insufficient_coin") return t("reward.errorInsufficient");
  if (kind === "redeem" && code === "redemption_intent_conflict") return t("reward.errorRedemptionConflict");
  if (kind === "refund" && code === "refund_intent_conflict") return t("reward.errorRefundConflict");
  if (code === "reward_integrity_violation") return t("reward.errorIntegrity");
  if (code === "reward_persistence_unavailable" || code === "reward_database_failure") return t("reward.errorStorage");
  return kind === "redeem" ? t("reward.errorRedeem") : t("reward.errorRefund");
}

function formatRewardDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN");
}

function RewardTransactions({ account, onTransactionResolved }: { account: RewardAccountSummaryDto | null; onTransactionResolved: () => void }) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const confirmRef = useRef<HTMLButtonElement>(null);
  const pendingRef = useRef(false);
  const redeemIntentRef = useRef<{ reward: CustomRewardDto; id: string } | null>(null);
  const refundIntentRef = useRef<{ item: RedemptionHistoryItemDto; id: string } | null>(null);
  const [rewards, setRewards] = useState<CustomRewardDto[]>([]);
  const [history, setHistory] = useState<RedemptionHistoryItemDto[]>([]);
  const [historyLoading, setHistoryLoading] = useState(true);
  const [historyError, setHistoryError] = useState(false);
  const [transaction, setTransaction] = useState<"redeem" | "refund" | null>(null);
  const [transactionError, setTransactionError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const loadRewards = useCallback(async () => { try { setRewards(await listCustomRewards()); } catch { setRewards([]); } }, []);
  const loadHistory = useCallback(async () => { setHistoryLoading(true); setHistoryError(false); try { setHistory(await getRewardRedemptionHistory()); } catch { setHistoryError(true); } finally { setHistoryLoading(false); } }, []);
  useEffect(() => { void loadRewards(); void loadHistory(); }, [loadRewards, loadHistory]);
  const close = useCallback(() => {
    if (pendingRef.current) return;
    setTransaction(null); setTransactionError(null); redeemIntentRef.current = null; refundIntentRef.current = null;
    queueMicrotask(() => triggerRef.current?.focus());
  }, []);
  useEffect(() => {
    if (!transaction) return;
    confirmRef.current?.focus();
    const dialog = dialogRef.current;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") { event.preventDefault(); close(); return; }
      if (event.key !== "Tab" || !dialog) return;
      const focusable = [...dialog.querySelectorAll<HTMLElement>('button:not([disabled]), [tabindex]:not([tabindex="-1"])')];
      if (!focusable.length) return;
      const first = focusable[0]; const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [close, transaction]);
  const beginRedeem = (reward: CustomRewardDto, event: MouseEvent<HTMLButtonElement>) => { triggerRef.current = event.currentTarget; redeemIntentRef.current = { reward, id: createRewardIntentId() }; setTransactionError(null); setTransaction("redeem"); };
  const beginRefund = (item: RedemptionHistoryItemDto, event: MouseEvent<HTMLButtonElement>) => { triggerRef.current = event.currentTarget; refundIntentRef.current = { item, id: createRewardIntentId() }; setTransactionError(null); setTransaction("refund"); };
  const refresh = async () => { await Promise.all([loadRewards(), loadHistory()]); onTransactionResolved(); };
  const confirm = async () => {
    if (!transaction || pendingRef.current) return;
    pendingRef.current = true; setPending(true); setTransactionError(null);
    try {
      if (transaction === "redeem") {
        const intent = redeemIntentRef.current;
        if (!intent) return;
        const result: RedemptionResultDto = await redeemCustomReward({ redemptionId: intent.id, customRewardId: intent.reward.customRewardId });
        setTransaction(null); redeemIntentRef.current = null; await refresh();
        setTransactionError(null);
      } else {
        const intent = refundIntentRef.current;
        if (!intent) return;
        await refundCustomReward({ refundId: intent.id, redemptionId: intent.item.redemptionId });
        setTransaction(null); refundIntentRef.current = null; await refresh();
      }
      queueMicrotask(() => triggerRef.current?.focus());
    } catch (error) { setTransactionError(transactionMessage(error, transaction)); }
    finally { pendingRef.current = false; setPending(false); }
  };
  const activeRewards = rewards.filter((reward) => reward.status === "active");
  const activeTransaction = transaction === "redeem" ? redeemIntentRef.current : null;
  const refundTransaction = transaction === "refund" ? refundIntentRef.current : null;
  const transactionTitle = transaction === "redeem"
    ? t("reward.redeemQuestion", { name: activeTransaction?.reward.name ?? "" })
    : t("reward.refundQuestion", { name: refundTransaction?.item.rewardName ?? "" });
  const transactionDescription = transaction === "redeem"
    ? t("reward.redeemDescription", { cost: activeTransaction?.reward.coinCost ?? 0 })
    : t("reward.refundDescription", { cost: refundTransaction?.item.coinCostPaid ?? 0, date: formatRewardDate(refundTransaction?.item.redeemedAtUtc ?? "") });
  return <>
    {activeRewards.length > 0 ? <section aria-labelledby="reward-actions-heading" className="content-panel"><h2 id="reward-actions-heading">{t("reward.redeemRewards")}</h2><ul className="detail-list">{activeRewards.map((reward) => { const insufficient = account !== null && account.coin < reward.coinCost; const descriptionId = "reward-insufficient-" + reward.customRewardId; return <li key={reward.customRewardId}><div><strong>{reward.name}</strong><span>{t("reward.coinCostValue", { cost: reward.coinCost })}</span>{insufficient ? <small id={descriptionId}>{t("reward.notEnoughCoin")}</small> : null}</div><button aria-describedby={insufficient ? descriptionId : undefined} className="primary-action" disabled={insufficient || pending} onClick={(event) => beginRedeem(reward, event)} type="button">{t("reward.redeem")}</button></li>; })}</ul></section> : null}
    <section aria-labelledby="redemption-history-heading" className="content-panel"><h2 id="redemption-history-heading">{t("reward.redemptionHistory")}</h2>{historyLoading ? <p aria-live="polite">{t("reward.historyLoading")}</p> : null}{historyError ? <div role="alert"><p>{t("reward.historyError")}</p><button className="secondary-action" onClick={() => void loadHistory()} type="button">{t("reward.retryHistory")}</button></div> : null}{!historyLoading && !historyError && history.length === 0 ? <p>{t("reward.noRedemptions")}</p> : null}{!historyLoading && !historyError && history.length > 0 ? <ul className="detail-list">{history.map((item) => <li key={item.redemptionId}><div><strong>{item.rewardName}</strong><span>{t("reward.coinPaidValue", { cost: item.coinCostPaid })}</span><span>{t("reward.redeemed", { date: formatRewardDate(item.redeemedAtUtc) })}</span><span>{item.refundedAtUtc ? t("reward.refunded", { date: formatRewardDate(item.refundedAtUtc) }) : t("reward.notRefunded")}</span></div>{item.refundId === null ? <button className="secondary-action" onClick={(event) => beginRefund(item, event)} type="button">{t("reward.refund")}</button> : <span aria-label={t("reward.refundedStatus")} className="safe-note">{t("reward.refundedStatus")}</span>}</li>)}</ul> : null}</section>
    {transaction ? <div className="modal-backdrop"><div aria-describedby="reward-transaction-description" aria-labelledby="reward-transaction-title" aria-modal="true" ref={dialogRef} role="alertdialog"><h2 id="reward-transaction-title">{transactionTitle}</h2><p id="reward-transaction-description">{transactionDescription}</p>{transactionError ? <p className="error-message" role="alert">{transactionError}</p> : null}<div className="button-row"><button className="primary-action" disabled={pending} onClick={() => void confirm()} ref={confirmRef} type="button">{pending ? (transaction === "redeem" ? t("reward.redeeming") : t("reward.refunding")) : (transaction === "redeem" ? t("reward.redeemReward") : t("reward.refundReward"))}</button><button className="secondary-action" disabled={pending} onClick={close} type="button">{t("common.cancel")}</button></div></div></div> : null}
  </>;
}

function ManualBackupSettings() {
  const [preview, setPreview] = useState<ManualBackupPreviewDto | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [inventory, setInventory] = useState<BackupInventoryDto | null>(null);
  const prepare = async () => {
    try { setPreview(await previewManualBackup()); setMessage(null); }
    catch (cause) { setMessage(getErrorPresentation(cause, "load")); }
  };
  const backup = async () => {
    setBusy(true);
    try {
      const result = await createManualBackup();
      setMessage(`备份已创建：${result.path}`);
      setPreview(null);
      setInventory(await loadBackupInventory());
    } catch {
      setMessage(getErrorPresentation(new Error("backup_save_failed"), "save") + " 未发布不完整的备份。");
    } finally { setBusy(false); }
  };
  const inspect = async () => {
    try { setInventory(await loadBackupInventory()); setMessage(null); }
    catch (cause) { setMessage(getErrorPresentation(cause, "load")); }
  };
  return (
    <section aria-labelledby="manual-backup" className="content-panel">
       <h2 id="manual-backup">系统事实备份</h2>
       <p>创建与 SQLite 一致的快照，不会复制或修改 Markdown 文件。</p>
       <button className="secondary-action" onClick={() => void prepare()} type="button">预览手动备份</button>
       <button className="secondary-action" onClick={() => void inspect()} type="button">查看备份清单</button>
      {preview ? <div role="alertdialog">
        <p>Schema {preview.schemaVersion}; destination <code>{preview.backupDirectory}</code>; filename prefix <code>{preview.filenamePrefix}</code>.</p>
        <button disabled={busy} onClick={() => void backup()} type="button">{busy ? "正在创建备份…" : "创建备份"}</button>
      </div> : null}
      {message ? <p aria-live="polite" className="safe-note">{message}</p> : null}
      {inventory ? <div>
        <p>保留预览：保留 {inventory.dailyKeep} 个每日快照和 {inventory.weeklyKeep} 个每周快照。手动备份和迁移备份受保护。</p>
         {inventory.entries.length === 0 ? <p>没有已发布的备份。</p> : <ul className="backup-inventory">
          {inventory.entries.map((entry) => <li key={entry.path}>
            <code>{entry.path}</code><span>{entry.category} · {entry.integrityVerified ? "完整性已验证" : "完整性验证失败"} · {entry.retention}</span>
          </li>)}
        </ul>}
      </div> : null}
    </section>
  );
}

const knowledgeLevels: Array<[KnowledgeUnderstandingLevel, string]> = [
  ["notLearned", "未学"],
  ["vague", "学过但模糊"],
  ["basic", "基本理解"],
  ["proficient", "熟练使用"],
  ["deep", "深入理解"],
];

function knowledgeLevelLabel(level: KnowledgeUnderstandingLevel): string {
  return knowledgeLevels.find(([value]) => value === level)?.[1] ?? level;
}

function KnowledgePage({ navigate }: { navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [nodes, setNodes] = useState<KnowledgeNodeDto[]>([]);
  const [anomalies, setAnomalies] = useState<KnowledgeNodeDto[]>([]);
  const [identityConflicts, setIdentityConflicts] = useState<KnowledgeIdentityConflictDto[]>([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [detail, setDetail] = useState<KnowledgeDetailDto | null>(null);
  const [selectedLevel, setSelectedLevel] = useState<KnowledgeUnderstandingLevel>("notLearned");
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [reevaluation, setReevaluation] = useState<{ shouldSuggest: boolean; qualifyingProblemCount: number } | null>(null);
  const [repairingNodeId, setRepairingNodeId] = useState<string | null>(null);
  const [relocationCandidates, setRelocationCandidates] = useState<Record<string, KnowledgeRelocationCandidateDto[]>>({});
  const [confirmingDeletedNodeId, setConfirmingDeletedNodeId] = useState<string | null>(null);
  const [deletePreviewNodeId, setDeletePreviewNodeId] = useState<string | null>(null);
  const [resolvingConflict, setResolvingConflict] = useState(false);

  const refresh = useCallback(async (nextQuery = query) => {
    setLoading(true);
    setError(null);
    try {
      const index = await loadKnowledgeIndex(nextQuery);
      setNodes(index.nodes);
      setAnomalies(index.locationAnomalies);
      setIdentityConflicts(index.identityConflicts ?? []);
    } catch {
      setError(t("knowledge.index"));
    } finally {
      setLoading(false);
    }
  }, [query]);

  useEffect(() => { void refresh(""); }, []);

  const openDetail = async (node: KnowledgeNodeDto) => {
    setError(null);
    setMessage(null);
    try {
      const next = await loadKnowledgeDetail(node.knowledgeNodeId);
      setDetail(next);
      setSelectedLevel(next.understanding?.current ?? "notLearned");
      try { setReevaluation(await loadKnowledgeReevaluationSuggestion(node.knowledgeNodeId)); }
      catch { setReevaluation(null); }
    } catch {
      setError(t("knowledge.empty"));
    }
  };

  const confirmUnderstanding = async () => {
    if (!detail) return;
    setSaving(true);
    setMessage(null);
    try {
      const understanding = await confirmKnowledgeUnderstanding(detail.node.knowledgeNodeId, selectedLevel);
      setDetail({ ...detail, understanding });
      setMessage(t("knowledge.confirmStatus"));
    } catch {
      setMessage(t("errors.unknown"));
    } finally {
      setSaving(false);
    }
  };

  const findKnowledgeRelocationCandidates = async (knowledgeNodeId: string) => {
    setError(null);
    try {
      const candidates = await loadKnowledgeRelocationCandidates(knowledgeNodeId);
      setRelocationCandidates((current) => ({ ...current, [knowledgeNodeId]: candidates }));
    } catch {
      setError("Possible Knowledge locations could not be read fresh.");
    }
  };

  const confirmKnowledgeRelocation = async (knowledgeNodeId: string, vaultRelativePath: string) => {
    setRepairingNodeId(knowledgeNodeId);
    setError(null);
    try {
      await rebindKnowledgeNode(knowledgeNodeId, vaultRelativePath);
      setRelocationCandidates((current) => {
        const next = { ...current };
        delete next[knowledgeNodeId];
        return next;
      });
      await refresh(query);
    } catch {
      setError("This Knowledge location could not be rebound. The binding was not changed.");
    } finally {
      setRepairingNodeId(null);
    }
  };

  const confirmKnowledgeDeleted = async (knowledgeNodeId: string) => {
    setConfirmingDeletedNodeId(knowledgeNodeId);
    setError(null);
    try {
      await confirmKnowledgeMarkdownDeleted(knowledgeNodeId);
      setDeletePreviewNodeId(null);
      setRelocationCandidates((current) => {
        const next = { ...current };
        delete next[knowledgeNodeId];
        return next;
      });
      await refresh(query);
    } catch {
      setError("Deletion could not be confirmed. The Knowledge identity and history were preserved.");
    } finally {
      setConfirmingDeletedNodeId(null);
    }
  };

  const resolveIdentityConflict = async (conflict: KnowledgeIdentityConflictDto, restoreOldIdentity: boolean) => {
    setResolvingConflict(true);
    setError(null);
    try {
      await resolveKnowledgeIdentityConflict(
        conflict.historicalKnowledgeNodeId,
        conflict.candidateVaultRelativePath,
        restoreOldIdentity,
      );
      await refresh(query);
    } catch {
      setError("This Knowledge identity conflict changed before confirmation. Nothing was reassigned.");
    } finally {
      setResolvingConflict(false);
    }
  };

  return (
    <>
      <PageHeader eyebrow="Markdown 权威来源" headingRef={headingRef} title="知识库" />
      <section aria-labelledby="knowledge-index-title" className="content-panel knowledge-index">
        <div className="knowledge-toolbar">
          <div><h2 id="knowledge-index-title">知识库索引</h2><p>这里只显示知识库目录中当前找到的 Markdown 文件。</p></div>
          <button className="secondary-action" disabled={loading} onClick={() => void refresh(query)} type="button">{loading ? "正在重新索引…" : "重新索引"}</button>
        </div>
        <form className="knowledge-search" onSubmit={(event) => { event.preventDefault(); void refresh(query); }}>
          <label htmlFor="knowledge-search">搜索名称或路径</label>
          <div><input id="knowledge-search" onChange={(event) => setQuery(event.currentTarget.value)} value={query} /><button type="submit">搜索</button></div>
        </form>
        {error ? <p aria-live="polite" className="error-copy">{error}</p> : null}
        {!loading && !error && nodes.length === 0 ? <p className="safe-note">没有找到匹配的 Markdown 文件。</p> : null}
        <ul className="knowledge-node-list">
          {nodes.map((node) => <li key={node.knowledgeNodeId}><button className="list-link" onClick={() => void openDetail(node)} type="button"><strong>{node.displayName}</strong><span>{node.vaultRelativePath}</span></button></li>)}
        </ul>
        {anomalies.length > 0 ? (
          <div className="recovery-actions">
            <p className="error-copy">有 {anomalies.length} 个已绑定的知识节点位置异常，需要恢复。</p>
            <ul className="knowledge-node-list">
              {anomalies.map((node) => (
                <li key={node.knowledgeNodeId}>
                  <strong>{node.displayName}</strong><span>{node.vaultRelativePath}</span>
                  <button className="secondary-action" onClick={() => void findKnowledgeRelocationCandidates(node.knowledgeNodeId)} type="button">查找可能的位置</button>
                  {relocationCandidates[node.knowledgeNodeId] ? (
                    <ul>
                      {relocationCandidates[node.knowledgeNodeId].map((candidate) => (
                        <li key={candidate.vaultRelativePath}>
                          <code>{candidate.vaultRelativePath}</code>{candidate.occupied ? <span>已绑定到其他主对象</span> : null}
                          <button disabled={candidate.occupied || repairingNodeId !== null} onClick={() => void confirmKnowledgeRelocation(node.knowledgeNodeId, candidate.vaultRelativePath)} type="button">使用此 Markdown</button>
                        </li>
                      ))}
                    </ul>
                  ) : null}
                  {deletePreviewNodeId === node.knowledgeNodeId ? (
                    <div role="alertdialog" aria-labelledby={`confirm-knowledge-delete-${node.knowledgeNodeId}`}>
                      <h3 id={`confirm-knowledge-delete-${node.knowledgeNodeId}`}>Confirm that this Knowledge Markdown was deleted?</h3>
                      <p>This does not delete any file. The formal Knowledge Node leaves the current index, remaining WikiLinks become unresolved, and understanding history is preserved.</p>
                      <button disabled={confirmingDeletedNodeId !== null} onClick={() => setDeletePreviewNodeId(null)} type="button">Cancel</button>
                      <button className="danger-action" disabled={confirmingDeletedNodeId !== null} onClick={() => void confirmKnowledgeDeleted(node.knowledgeNodeId)} type="button">
                        {confirmingDeletedNodeId === node.knowledgeNodeId ? "Revalidating absence…" : "Confirm deleted"}
                      </button>
                    </div>
                  ) : <button className="danger-action" onClick={() => setDeletePreviewNodeId(node.knowledgeNodeId)} type="button">Confirm file was deleted…</button>}
                </li>
              ))}
            </ul>
          </div>
        ) : null}
        {identityConflicts.length > 0 ? (
          <div className="recovery-actions">
            <p className="error-copy">某个 Markdown 名称与之前删除的知识节点相同，系统没有自动猜测其身份。</p>
            <ul className="knowledge-node-list">
              {identityConflicts.map((conflict) => (
                <li key={`${conflict.historicalKnowledgeNodeId}:${conflict.candidateVaultRelativePath}`}>
                  <strong>{conflict.displayName}</strong><span>{conflict.candidateVaultRelativePath}</span>
                  <p>恢复旧节点会保留历史身份和理解状态；无论选择哪一种，当前关系都只会根据这份 Markdown 重新建立。</p>
                  <button disabled={resolvingConflict} onClick={() => void resolveIdentityConflict(conflict, true)} type="button">恢复旧知识节点</button>
                  <button disabled={resolvingConflict} onClick={() => void resolveIdentityConflict(conflict, false)} type="button">创建新知识节点</button>
                </li>
              ))}
            </ul>
          </div>
        ) : null}
      </section>
      {detail ? (
        <section aria-labelledby="knowledge-detail-title" className="content-panel knowledge-detail">
          <div className="knowledge-toolbar"><div><p className="eyebrow">最新 Markdown 详情</p><h2 id="knowledge-detail-title">{detail.node.displayName}</h2><p><code>{detail.node.vaultRelativePath}</code></p></div><button className="secondary-action" onClick={() => void openKnowledgeInObsidian(detail.node.knowledgeNodeId).catch(() => setMessage("Obsidian 无法打开此文件。"))} type="button">在 Obsidian 中打开</button></div>
          <div className="knowledge-understanding">
            <label>当前理解程度<select aria-label="当前理解程度" onChange={(event) => setSelectedLevel(event.currentTarget.value as KnowledgeUnderstandingLevel)} value={selectedLevel}>{knowledgeLevels.map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label>
            <button disabled={saving} onClick={() => void confirmUnderstanding()} type="button">{saving ? "正在保存…" : "确认状态"}</button>
            {detail.understanding ? <p>历史最高：<strong>{knowledgeLevelLabel(detail.understanding.historicalHighest)}</strong> · 首次达到 {detail.understanding.firstReachedHighestOn}</p> : <p>尚未有用户确认的状态。</p>}
            {reevaluation?.shouldSuggest ? <p aria-live="polite" className="safe-note">建议重新评估此知识状态：{reevaluation.qualifyingProblemCount} 道相关题目获得了新的“真会”复习证据。当前状态没有改变。</p> : null}
            {message ? <p aria-live="polite" className="safe-note">{message}</p> : null}
          </div>
          <KnowledgeNeighborList heading="指向的知识" nodes={detail.outgoing} onOpen={openDetail} />
          <KnowledgeNeighborList heading="引用此知识" nodes={detail.incoming} onOpen={openDetail} />
<div><h3>相关题目</h3>{detail.relatedProblems.length === 0 ? <p>暂无。</p> : <ul>{detail.relatedProblems.map((problem) => <li key={problem.problemId}><button className="list-link" onClick={() => navigate(`/problems/id/${problem.problemId}`)} type="button"><strong>{problem.title}</strong></button></li>)}</ul>}</div>
        </section>
      ) : null}
    </>
  );
}

function KnowledgeNeighborList({ heading, nodes, onOpen }: { heading: string; nodes: KnowledgeNodeDto[]; onOpen: (node: KnowledgeNodeDto) => Promise<void> }) {
  return <div><h3>{heading}</h3>{nodes.length === 0 ? <p>暂无。</p> : <ul>{nodes.map((node) => <li key={node.knowledgeNodeId}><button className="list-link" onClick={() => void onOpen(node)} type="button"><strong>{node.displayName}</strong><span>{node.vaultRelativePath}</span></button></li>)}</ul>}</div>;
}

const weekBudgetFields: Array<[keyof WeeklyAcmBudgetDto, string]> = [
  ["monday", "Monday"], ["tuesday", "Tuesday"], ["wednesday", "Wednesday"],
  ["thursday", "Thursday"], ["friday", "Friday"], ["saturday", "Saturday"], ["sunday", "Sunday"],
];

function WeeklyAcmBudgetSettings() {
  const [draft, setDraft] = useState<Record<keyof WeeklyAcmBudgetDto, string>>({
    monday: "", tuesday: "", wednesday: "", thursday: "", friday: "", saturday: "", sunday: "",
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void loadWeeklyAcmBudget()
      .then((schedule) => setDraft(Object.fromEntries(weekBudgetFields.map(([key]) => [key, schedule[key] === null ? "" : String(schedule[key])])) as Record<keyof WeeklyAcmBudgetDto, string>))
      .catch(() => setMessage("Weekly budget is temporarily unavailable."))
      .finally(() => setLoading(false));
  }, []);

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); setMessage(null);
    const schedule = {} as WeeklyAcmBudgetDto;
    for (const [key] of weekBudgetFields) {
      const value = draft[key].trim();
      const minutes = value === "" ? null : Number(value);
      if (minutes !== null && (!Number.isInteger(minutes) || minutes < 0)) {
        setMessage("Each weekly budget must be blank or a non-negative whole number of minutes.");
        return;
      }
      schedule[key] = minutes;
    }
    setSaving(true);
    try {
      const saved = await saveWeeklyAcmBudget(schedule);
      setDraft(Object.fromEntries(weekBudgetFields.map(([key]) => [key, saved[key] === null ? "" : String(saved[key])])) as Record<keyof WeeklyAcmBudgetDto, string>);
      setMessage("Weekly ACM budget saved. Existing Today plans and one-day overrides were not changed.");
    } catch { setMessage("Weekly budget could not be saved."); }
    finally { setSaving(false); }
  };

  return <section aria-labelledby="weekly-acm-budget" className="content-panel"><h2 id="weekly-acm-budget">{t("today.weeklyBudget")}</h2><p>{t("today.weeklyBudgetHelp")}</p>{loading ? <p>{t("today.loadingPlan")}</p> : <form className="weekly-budget-form" noValidate onSubmit={submit}><div>{weekBudgetFields.map(([key, label]) => <label key={key}>{label}<input aria-label={`${label} ACM budget in minutes`} min="0" onInput={(event) => { const value = event.currentTarget.value; setDraft((current) => ({ ...current, [key]: value })); }} placeholder="未设置" type="number" value={draft[key]} /></label>)}</div><button className="primary-action" disabled={saving} type="submit">{saving ? "保存中…" : t("today.saveBudget")}</button></form>}{message ? <p aria-live="polite" className="safe-note">{message}</p> : null}</section>;
}

function TodayPage({ navigate }: { navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [snapshot, setSnapshot] = useState<TodaySnapshotDto | null>(null);
  const [needsBudget, setNeedsBudget] = useState(false);
  const [initialBudgetDraft, setInitialBudgetDraft] = useState("60");
  const [budgetDraft, setBudgetDraft] = useState("60");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [announcement, setAnnouncement] = useState<string | null>(null);
  const [busyEntry, setBusyEntry] = useState<string | null>(null);
  const [replan, setReplan] = useState<TodayReplanPreviewDto | null>(null);
  const [suggestions, setSuggestions] = useState<TodayExtraSuggestionsPreviewDto | null>(null);
  const draggedEntryIdRef = useRef<string | null>(null);
  const pointerTargetEntryIdRef = useRef<string | null>(null);
  const replanDialogRef = useRef<HTMLDivElement>(null);
  const replanApplyRef = useRef<HTMLButtonElement>(null);
  const replanTriggerRef = useRef<HTMLButtonElement>(null);

  const refreshSuggestions = useCallback(async (next: TodaySnapshotDto) => {
    if (next.entries.length > 0 && next.entries.every((entry) => entry.status === "completed")) {
      setSuggestions(await loadTodayExtraSuggestions());
    } else {
      setSuggestions(null);
    }
  }, []);

  const load = useCallback(async (create = false) => {
    setLoading(true); setError(null);
    try {
      const next = await loadToday(create ? Number(initialBudgetDraft) : null);
      if (!next) { setNeedsBudget(true); setSnapshot(null); return; }
      setNeedsBudget(false);
      setSnapshot(next); setBudgetDraft(String(next.budgetMinutes));
      await refreshSuggestions(next);
    } catch (cause) { setError(todayErrorMessage(cause)); }
    finally { setLoading(false); }
  }, [initialBudgetDraft, refreshSuggestions]);

  useEffect(() => { void load(false); }, []);

  const commitSnapshot = async (next: TodaySnapshotDto, nextAnnouncement?: string) => {
    const closingReplan = replan !== null;
    setSnapshot(next); setBudgetDraft(String(next.budgetMinutes)); setReplan(null); setError(null);
    setAnnouncement(nextAnnouncement ?? null);
    if (closingReplan) queueMicrotask(() => replanTriggerRef.current?.focus());
    await refreshSuggestions(next);
  };

  const move = async (index: number, offset: -1 | 1) => {
    if (!snapshot) return;
    const target = index + offset;
    if (target < 0 || target >= snapshot.entries.length) return;
    const ids = snapshot.entries.map((entry) => entry.entryId);
    [ids[index], ids[target]] = [ids[target], ids[index]];
    try { await commitSnapshot(await reorderToday(snapshot.planId, ids), "Today order updated."); }
    catch (cause) { setError(todayErrorMessage(cause)); }
  };

  const dropEntry = async (targetEntryId: string) => {
    const sourceEntryId = draggedEntryIdRef.current;
    if (!snapshot || !sourceEntryId || sourceEntryId === targetEntryId) {
      draggedEntryIdRef.current = null;
      return;
    }
    const ids = snapshot.entries.map((entry) => entry.entryId);
    const from = ids.indexOf(sourceEntryId);
    const target = ids.indexOf(targetEntryId);
    if (from < 0 || target < 0) { draggedEntryIdRef.current = null; return; }
    const [moved] = ids.splice(from, 1);
    ids.splice(target, 0, moved);
    draggedEntryIdRef.current = null;
    try { await commitSnapshot(await reorderToday(snapshot.planId, ids), "Today order updated."); }
    catch (cause) { setError(todayErrorMessage(cause)); }
  };

  const clearPointerDrag = () => {
    draggedEntryIdRef.current = null;
    pointerTargetEntryIdRef.current = null;
    document.querySelectorAll(".today-entry--dragging, .today-entry--drop-target")
      .forEach((item) => item.classList.remove("today-entry--dragging", "today-entry--drop-target"));
  };

  const startPointerDrag = (event: React.PointerEvent<HTMLButtonElement>, entryId: string) => {
    if (event.button !== 0) return;
    event.preventDefault();
    draggedEntryIdRef.current = entryId;
    pointerTargetEntryIdRef.current = entryId;
    event.currentTarget.setPointerCapture(event.pointerId);
    event.currentTarget.closest(".today-entry")?.classList.add("today-entry--dragging");
  };

  const movePointerDrag = (event: React.PointerEvent<HTMLButtonElement>) => {
    if (!draggedEntryIdRef.current) return;
    event.preventDefault();
    const target = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>(".today-entry");
    const targetEntryId = target?.dataset.entryId ?? null;
    document.querySelectorAll(".today-entry--drop-target")
      .forEach((item) => item.classList.remove("today-entry--drop-target"));
    if (target && targetEntryId && targetEntryId !== draggedEntryIdRef.current) {
      target.classList.add("today-entry--drop-target");
      pointerTargetEntryIdRef.current = targetEntryId;
    } else {
      pointerTargetEntryIdRef.current = draggedEntryIdRef.current;
    }
  };

  const finishPointerDrag = (event: React.PointerEvent<HTMLButtonElement>) => {
    const targetEntryId = pointerTargetEntryIdRef.current;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    if (targetEntryId && targetEntryId !== draggedEntryIdRef.current) {
      void dropEntry(targetEntryId);
    }
    clearPointerDrag();
  };

  const done = async (entry: TodayEntryDto) => {
    if (!snapshot || busyEntry) return;
    setBusyEntry(entry.entryId);
    try { await commitSnapshot(await completeTodayEntry(snapshot.planId, entry.entryId), `${entry.problemTitle} marked done for today.`); }
    catch (cause) { setError(todayErrorMessage(cause)); }
    finally { setBusyEntry(null); }
  };

  const previewBudget = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); setError(null);
    const proposedBudget = Number(budgetDraft);
    if (budgetDraft.trim() === "" || !Number.isInteger(proposedBudget) || proposedBudget < 0) {
      setError("Daily budget must be a non-negative whole number of minutes.");
      return;
    }
    try { setReplan(await previewTodayReplan(proposedBudget)); }
    catch (cause) { setError(todayErrorMessage(cause)); }
  };

  const applyBudget = async () => {
    if (!replan) return;
    try { await commitSnapshot(await applyTodayReplan(replan), "Today replan applied."); }
    catch (cause) { setError(todayErrorMessage(cause)); }
  };

  const acceptSuggestion = async (problemId: string) => {
    if (!suggestions) return;
    setBusyEntry(problemId);
    try { await commitSnapshot(await acceptTodayExtraSuggestion(suggestions, problemId), "Added the suggestion to Today."); }
    catch (cause) { setError(todayErrorMessage(cause)); }
    finally { setBusyEntry(null); }
  };

  useEffect(() => {
    if (!replan) return;
    const dialog = replanDialogRef.current;
    replanApplyRef.current?.focus();
    if (!dialog) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        setReplan(null);
        queueMicrotask(() => replanTriggerRef.current?.focus());
        return;
      }
      if (event.key !== "Tab") return;
      const focusable = [...dialog.querySelectorAll<HTMLElement>(
        'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])',
      )];
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [replan]);

  return <>
    <PageHeader eyebrow={t("today.dailyExecution")} headingRef={headingRef} title={t("today.pageTitle")} />
    <p aria-atomic="true" aria-live="polite" className="sr-only">{announcement}</p>
    {loading ? <p aria-live="polite">{t("today.loadingPlan")}</p> : null}
    {error ? <p aria-live="assertive" className="error-message">{error}</p> : null}
    {!loading && needsBudget ? <section className="empty-state"><h2>Set today&apos;s budget</h2><p>No weekly default is set for this weekday. Enter any non-negative whole number of minutes; tasks still use complete 30 or 60 minute planning blocks.</p><form className="today-budget-start" noValidate onSubmit={(event) => { event.preventDefault(); const value = Number(initialBudgetDraft); if (initialBudgetDraft.trim() === "" || !Number.isInteger(value) || value < 0) { setError("Daily budget must be a non-negative whole number of minutes."); return; } void load(true); }}><label>Minutes<input min="0" onInput={(event) => setInitialBudgetDraft(event.currentTarget.value)} required type="number" value={initialBudgetDraft} /></label><button className="primary-action" type="submit">Create Today plan</button></form></section> : null}
    {!loading && !snapshot && !needsBudget ? <section className="empty-state"><h2>{t("today.planUnavailable")}</h2><button className="secondary-action" onClick={() => void load(false)} type="button">{t("today.retry")}</button></section> : null}
    {snapshot ? <>
      <section className="today-toolbar" aria-label="Today plan summary">
        <dl><div><dt>Date</dt><dd>{snapshot.localDate}</dd></div><div><dt>Planned</dt><dd>{snapshot.plannedMinutes} min</dd></div><div><dt>Budget</dt><dd>{snapshot.budgetMinutes} min</dd></div><div><dt>Over</dt><dd>{snapshot.overBudgetMinutes} min</dd></div></dl>
        <form noValidate onSubmit={previewBudget}><label>Today override<input aria-label="Daily budget in minutes" min="0" onInput={(event) => setBudgetDraft(event.currentTarget.value)} required type="number" value={budgetDraft} /></label><button className="secondary-action" ref={replanTriggerRef} type="submit">Preview replan</button></form>
      </section>
      {snapshot.entries.length === 0 ? <section className="empty-state"><h2>{t("today.noTasks")}</h2><p>Only complete 30 or 60 minute tasks are scheduled.</p></section> :
        <ol className="today-list">{snapshot.entries.map((entry, index) => <li className={`today-entry today-entry--${entry.status}`} data-entry-id={entry.entryId} key={entry.entryId} onKeyDown={(event) => { if (event.altKey && event.key === "ArrowUp") { event.preventDefault(); void move(index, -1); } if (event.altKey && event.key === "ArrowDown") { event.preventDefault(); void move(index, 1); } }} tabIndex={0}>
          <div className="today-entry__order"><button aria-label={`Drag ${todayReasonLabel(entry.reason)} to reorder`} className="today-drag-handle" onPointerCancel={clearPointerDrag} onPointerDown={(event) => startPointerDrag(event, entry.entryId)} onPointerMove={movePointerDrag} onPointerUp={finishPointerDrag} title="Drag to reorder" type="button">⋮⋮</button><button aria-label={`Move ${todayReasonLabel(entry.reason)} up`} disabled={index === 0} onClick={() => void move(index, -1)} type="button">↑</button><button aria-label={`Move ${todayReasonLabel(entry.reason)} down`} disabled={index === snapshot.entries.length - 1} onClick={() => void move(index, 1)} type="button">↓</button></div>
          <div className="today-entry__body"><div><span className="today-lane">{todayLaneLabel(entry.lane)}</span><span className={`today-status today-status--${entry.status}`}>{todayStatusLabel(entry.status)}</span>{entry.origin === "manual" ? <span className="today-origin">Manual</span> : null}</div><button className="today-problem-link" onClick={() => navigate(entry.reviewAttemptId && entry.status === "inProgress" ? `/review/${entry.reviewAttemptId}` : `/problems/id/${entry.problemId}`)} type="button"><strong>{entry.problemTitle}</strong><span>{entry.problemRating === null ? "Unrated" : `Rating ${entry.problemRating}`} · {todayReasonLabel(entry.reason)} · {entry.planningCostMinutes} min</span></button></div>
          {todayDoneAllowed(entry) && entry.status !== "completed" ? <button className="primary-action" disabled={busyEntry === entry.entryId || entry.status === "unavailable"} onClick={() => void done(entry)} type="button">Done for today</button> : null}
        </li>)}</ol>}
      {suggestions && suggestions.suggestions.length > 0 ? <section className="today-suggestions"><h2>额外建议</h2><p>还剩 {suggestions.remainingBudgetMinutes} 分钟。未经确认不会自动加入。</p><ul>{suggestions.suggestions.map((item) => <li key={item.problemId}><span><strong>{item.problemTitle}</strong><small>{item.problemRating === null ? "未评级" : `评分 ${item.problemRating}`} · {todayReasonLabel(item.reason)} · {item.planningCostMinutes} 分钟</small></span><button className="secondary-action" disabled={busyEntry === item.problemId} onClick={() => void acceptSuggestion(item.problemId)} type="button">加入今日计划</button></li>)}</ul></section> : null}
    </> : null}
    {replan ? <div className="modal-backdrop"><div aria-describedby="today-replan-description" aria-labelledby="today-replan-title" aria-modal="true" ref={replanDialogRef} role="dialog"><h2 id="today-replan-title">应用这次重新规划</h2><p id="today-replan-description">预算从 {replan.expectedSnapshot.budgetMinutes} 调整为 {replan.proposedBudgetMinutes} 分钟。这是仅限今天的覆盖设置；每周默认值和下周同一天不变。计划任务将变为 {replan.proposedPlannedMinutes} 分钟，共 {replan.entries.length} 项。已完成、进行中和手动任务会受到保护。</p><div className="button-row"><button className="primary-action" onClick={() => void applyBudget()} ref={replanApplyRef} type="button">应用重新规划</button><button className="secondary-action" onClick={() => { setBudgetDraft(String(snapshot?.budgetMinutes ?? Number(initialBudgetDraft))); setReplan(null); queueMicrotask(() => replanTriggerRef.current?.focus()); }} type="button">取消</button></div></div></div> : null}
  </>;
}

function todayDoneAllowed(entry: TodayEntryDto) { return entry.reason === "continueLearning" || entry.reason === "relearn" || entry.reason === "upsolve"; }
function todayLaneLabel(lane: TodayEntryDto["lane"]) { return lane === "carryIn" ? "结转" : lane === "review" ? "复习" : "学习"; }
function todayStatusLabel(status: TodayEntryDto["status"]) { return ({ notStarted: "未开始", inProgress: "进行中", completed: "已完成", unavailable: "不可用" } as const)[status]; }
function todayReasonLabel(reason: TodayEntryDto["reason"]) { return ({ continueReview: "继续复习", continueLearning: "继续学习", dueFirstColdStart: "首次冷启动复习", dueLongTermReview: "长期复习", relearn: "重新学习", upsolve: "补题" } as const)[reason]; }
function todayErrorMessage(cause: unknown) { const code = String(cause); if (code.includes("stale_today")) return "The Today plan changed. Reload and try again."; if (code.includes("invalid_today_done")) return "This entry cannot be completed from Today."; if (code.includes("invalid_today_reorder")) return "The saved order changed. Reload and try again."; if (code.includes("today_integrity")) return "Today data failed an integrity check."; return "Today is temporarily unavailable."; }

function contestBookCoverTitle(title: string) {
  const match = /^(Educational\s+)?Codeforces(?:\s+(Global))?\s+Round\s+(\d+)(?:\s*\(([^)]+)\))?/i.exec(title.trim());
  if (!match) return { series: "Contest archive", roundNumber: null, subtitle: null };
  return {
    series: match[1] ? "Educational series" : match[2] ? "Global series" : "Round series",
    roundNumber: match[3],
    subtitle: match[4] ?? null,
  };
}

function ContestBookPrototype({ item, navigate }: { item: ContestShelfItemDto; navigate: Navigate }) {
  const title = displayProblemTitle(String(item.contestId), item.title);
  const coverTitle = contestBookCoverTitle(title);
  return (
    <button
      aria-label={`Open contest ${title}`}
      className="contest-book contest-book--asset"
      data-contest-id={item.contestId}
      onClick={() => navigate(`/contests/${item.contestId}`)}
      type="button"
    >
      <img aria-hidden="true" className="contest-book__shell" src={contestBookShell} alt="" />
      <span aria-hidden="true" className="contest-book__volume" />
      <span aria-hidden="true" className="contest-book__spine" />
      <span className="contest-book__cover">
        <span aria-hidden="true" className="contest-book__outer-edge" />
        <span aria-hidden="true" className="contest-book__frame" />
        <span className="contest-book__content">
          <span className="contest-book__masthead">
            <span className="contest-book__collection">Codeforces</span>
            <span className="contest-book__series">{coverTitle.series}</span>
          </span>
          {coverTitle.roundNumber ? <strong className="contest-book__title">
            <span className="contest-book__round-label">Round</span>
            <span className="contest-book__round-number">{coverTitle.roundNumber}</span>
          </strong> : <strong className="contest-book__title contest-book__title--fallback">{title}</strong>}
          {coverTitle.subtitle ? <span className="contest-book__subtitle">{coverTitle.subtitle}</span> : null}
          <span className="contest-book__footer">
            <span className="contest-book__identity">CF {item.contestId}</span>
            <span aria-hidden="true" className="contest-book__motif">ACM-OS</span>
          </span>
        </span>
      </span>
    </button>
  );
}

const CONTEST_CABINET_TIER_COUNT = 3;
const CONTEST_BOOKS_PER_TIER = 4;
const CONTEST_CABINET_CAPACITY = CONTEST_CABINET_TIER_COUNT * CONTEST_BOOKS_PER_TIER;
const CONTEST_CABINET_SHELF_FOREGROUNDS = [
  contestCabinetShelfForeground1,
  contestCabinetShelfForeground2,
  contestCabinetShelfForeground3,
] as const;

function ContestDisplayStand() {
  return <>
    <img aria-hidden="true" alt="" className="contest-display-stand contest-display-stand--rear" src={contestDisplayStandBack} />
    <img aria-hidden="true" alt="" className="contest-display-stand contest-display-stand--front" src={contestDisplayStandFront} />
  </>;
}

function ContestCabinetTier({ compactColumn, items, navigate, tier }: { compactColumn: number; items: ContestShelfItemDto[]; navigate: Navigate; tier: number }) {
  return (
    <section aria-label={`Shelf tier ${tier}`} className="contest-cabinet__tier" data-tier={tier}>
      <div className="contest-cabinet__back">
        <div className="contest-shelf-books">
          {items.map((item, index) => <div
            className="contest-book-slot"
            data-compact-active={index === compactColumn ? "true" : "false"}
            data-logical-column={index + 1}
            key={item.contestId}
          >
            <ContestDisplayStand />
            <ContestBookPrototype item={item} navigate={navigate} />
          </div>)}
        </div>
      </div>
      <div aria-hidden="true" className="contest-shelf">
        <img alt="" className="contest-shelf__foreground" src={CONTEST_CABINET_SHELF_FOREGROUNDS[tier - 1]} />
        <span className="contest-shelf__top" />
        <span className="contest-shelf__front" />
        <span className="contest-shelf__bottom-shadow" />
      </div>
    </section>
  );
}

function ContestCabinet({ items, navigate, totalCount }: { items: ContestShelfItemDto[]; navigate: Navigate; totalCount: number }) {
  const [compactColumn, setCompactColumn] = useState(0);
  const cabinetItemKey = items.map((item) => item.contestId).join(":");
  const tiers = Array.from({ length: CONTEST_CABINET_TIER_COUNT }, (_, tier) =>
    items.slice(tier * CONTEST_BOOKS_PER_TIER, (tier + 1) * CONTEST_BOOKS_PER_TIER));
  const compactPageCount = items.length === 0 ? 0 : Math.max(...tiers.map((tier) => tier.length));
  useEffect(() => {
    setCompactColumn((column) => Math.min(column, Math.max(0, compactPageCount - 1)));
  }, [cabinetItemKey, compactPageCount]);
  return (
    <section aria-labelledby="contest-cabinet-title" className="contest-cabinet-prototype">
      <div className="contest-cabinet-prototype__heading">
        <div>
          <p className="eyebrow">Contest archive</p>
          <h2 id="contest-cabinet-title">Collection cabinet</h2>
        </div>
        <p>{totalCount} {totalCount === 1 ? "contest" : "contests"}</p>
      </div>
      <div aria-label="Three-tier contest cabinet" className="contest-cabinet contest-cabinet--asset">
        <div aria-hidden="true" className="contest-cabinet__shell">
          <img alt="" className="contest-cabinet__shell-piece contest-cabinet__shell-piece--left" src={contestCabinetLeft} />
          <img alt="" className="contest-cabinet__shell-piece contest-cabinet__shell-piece--center" src={contestCabinetCenter} />
          <img alt="" className="contest-cabinet__shell-piece contest-cabinet__shell-piece--right" src={contestCabinetRight} />
        </div>
        <div className="contest-cabinet__overlay">
            <div aria-hidden="true" className="contest-cabinet__cornice"><span /></div>
            <div className="contest-cabinet__body">
              <div aria-hidden="true" className="contest-cabinet__stile contest-cabinet__stile--left" />
              <div className="contest-cabinet__tiers">
                {tiers.map((tierItems, index) => (
                  <ContestCabinetTier compactColumn={compactColumn} items={tierItems} key={index} navigate={navigate} tier={index + 1} />
                ))}
                {items.length === 0 ? <p className="contest-cabinet__empty">No contests in this view</p> : null}
              </div>
              <div aria-hidden="true" className="contest-cabinet__stile contest-cabinet__stile--right" />
            </div>
            <div aria-hidden="true" className="contest-cabinet__plinth"><span /></div>
        </div>
      </div>
      {compactPageCount > 1 ? <nav aria-label="Compact cabinet column navigation" className="contest-cabinet-pager">
        <button aria-label="Show previous cabinet column" className="secondary-action" disabled={compactColumn === 0} onClick={() => setCompactColumn((column) => Math.max(0, column - 1))} type="button">Previous</button>
        <output aria-live="polite" aria-label={`Cabinet column ${compactColumn + 1} of ${compactPageCount}`}>{compactColumn + 1} / {compactPageCount}</output>
        <button aria-label="Show next cabinet column" className="secondary-action" disabled={compactColumn === compactPageCount - 1} onClick={() => setCompactColumn((column) => Math.min(compactPageCount - 1, column + 1))} type="button">Next</button>
      </nav> : null}
    </section>
  );
}

function ContestLibraryPage({ navigate }: { navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [families, setFamilies] = useState<ContestLibraryFamilyDto[] | null>(null);
  const [series, setSeries] = useState<ContestLibrarySeriesDto[]>([]);
  const [years, setYears] = useState<Array<number | null>>([]);
  const [items, setItems] = useState<ContestShelfItemDto[] | null>(null);
  const [familyId, setFamilyId] = useState<number | null>(null);
  const [seriesFilter, setSeriesFilter] = useState<ContestLibrarySeriesFilterDto>({ kind: "any" });
  const [yearFilter, setYearFilter] = useState<ContestLibraryYearFilterDto>({ kind: "any" });
  const [archiveFilter, setArchiveFilter] = useState<ContestLibraryArchiveFilterDto>("active");
  const [loading, setLoading] = useState(true);
  const [failed, setFailed] = useState(false);
  const [retryNonce, setRetryNonce] = useState(0);
  const [managementMessage, setManagementMessage] = useState<string | null>(null);
  const [managementBusy, setManagementBusy] = useState(false);
  const [familyDraft, setFamilyDraft] = useState("");
  const [seriesDraft, setSeriesDraft] = useState("");
  const [editingFamily, setEditingFamily] = useState<number | null>(null);
  const [editingSeries, setEditingSeries] = useState<number | null>(null);
  const familyRequest = useRef(0);
  const libraryRequest = useRef(0);
  const [contestUrl, setContestUrl] = useState("");
  const [importing, setImporting] = useState(false);
  const [importMessage, setImportMessage] = useState<string | null>(null);
  const [manualContestId, setManualContestId] = useState("");
  const [manualTitle, setManualTitle] = useState("");
  const [manualDate, setManualDate] = useState("");
  const [manualProblems, setManualProblems] = useState([{ index: "A", title: "", sourceUrl: "", statementText: "" }]);

  useEffect(() => {
    let active = true;
    listContestLibraryFamilies().then((next) => { if (active) setFamilies(next); }).catch(() => { if (active) setFailed(true); });
    return () => { active = false; };
  }, [retryNonce]);

  useEffect(() => {
    const request = ++familyRequest.current;
    if (familyId === null) { setSeries([]); setYears([]); return; }
    Promise.all([listContestLibrarySeries(familyId), listContestLibraryYears(familyId, seriesFilter)])
      .then(([nextSeries, nextYears]) => {
        if (request !== familyRequest.current) return;
        setSeries(nextSeries); setYears(nextYears);
      })
      .catch(() => { if (request === familyRequest.current) setManagementMessage("Family navigation could not be loaded. Try again."); });
  }, [familyId, seriesFilter]);

  useEffect(() => {
    const request = ++libraryRequest.current;
    const scope: ContestLibraryScopeDto = familyId === null
      ? { kind: "all" }
      : { kind: "family", familyId, series: seriesFilter, year: yearFilter };
    setLoading(true); setFailed(false);
    listContestLibraryContests({ scope, archive: archiveFilter }).then((next) => {
      if (request === libraryRequest.current) { setItems(next); setLoading(false); }
    }).catch(() => { if (request === libraryRequest.current) { setFailed(true); setLoading(false); } });
  }, [familyId, seriesFilter, yearFilter, archiveFilter, retryNonce]);

  const selectFamily = (next: number | null) => {
    setFamilyId(next); setSeries([]); setYears([]); setSeriesFilter({ kind: "any" }); setYearFilter({ kind: "any" });
  };
  const selectSeries = (next: ContestLibrarySeriesFilterDto) => {
    setSeriesFilter(next); setYears([]); setYearFilter({ kind: "any" });
  };
  const refreshFamilies = async (selected: number | null = familyId) => {
    const next = await listContestLibraryFamilies();
    setFamilies(next);
    if (selected !== null && next.some((family) => family.familyId === selected)) setFamilyId(selected);
  };
  const createFamily = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (managementBusy) return;
    setManagementBusy(true); setManagementMessage(null);
    try { const created = await createContestLibraryFamily(familyDraft); await refreshFamilies(created.familyId); setFamilyDraft(""); }
    catch (error) { setManagementMessage(contestLibraryErrorMessage(error)); }
    finally { setManagementBusy(false); }
  };
  const renameFamily = async (event: FormEvent<HTMLFormElement>, id: number) => {
    event.preventDefault(); if (managementBusy) return;
    setManagementBusy(true); setManagementMessage(null);
    try { await renameContestLibraryFamily(id, familyDraft); await refreshFamilies(id); setFamilyDraft(""); setEditingFamily(null); }
    catch (error) { setManagementMessage(contestLibraryErrorMessage(error)); }
    finally { setManagementBusy(false); }
  };
  const createSeries = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (managementBusy || familyId === null) return;
    setManagementBusy(true); setManagementMessage(null);
    try { await createContestLibrarySeries(familyId, seriesDraft); setSeriesDraft(""); setSeries(await listContestLibrarySeries(familyId)); }
    catch (error) { setManagementMessage(contestLibraryErrorMessage(error)); }
    finally { setManagementBusy(false); }
  };
  const renameSeries = async (event: FormEvent<HTMLFormElement>, id: number) => {
    event.preventDefault(); if (managementBusy) return;
    setManagementBusy(true); setManagementMessage(null);
    try { await renameContestLibrarySeries(id, seriesDraft); setSeriesDraft(""); setEditingSeries(null); if (familyId !== null) setSeries(await listContestLibrarySeries(familyId)); }
    catch (error) { setManagementMessage(contestLibraryErrorMessage(error)); }
    finally { setManagementBusy(false); }
  };
  const submitImport = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (importing) return;
    setImporting(true); setImportMessage(null);
    try { const result = await importCodeforcesContest(contestUrl); setImportMessage(result.importStatus === "complete" ? "Contest imported." : `Contest saved; ${result.missingSnapshotProblems.length} snapshots remain.`); setRetryNonce((value) => value + 1); }
    catch (error) { setImportMessage(contestImportErrorMessage(error)); }
    finally { setImporting(false); }
  };
  const retryMissing = async (contestId: number) => {
    if (importing) return;
    setImporting(true); setImportMessage(null);
    try {
      const result = await importCodeforcesContest(`https://codeforces.com/contest/${contestId}`);
      setImportMessage(result.importStatus === "complete" ? "Missing snapshots were completed." : `Retry finished; ${result.missingSnapshotProblems.length} snapshots remain.`);
      setRetryNonce((value) => value + 1);
    } catch (error) { setImportMessage(contestImportErrorMessage(error)); }
    finally { setImporting(false); }
  };
  const submitManual = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (importing) return;
    setImporting(true); setImportMessage(null);
    try {
      const contestId = Number(manualContestId);
      await importManualCodeforcesContest({ contestId, title: manualTitle, sourceUrl: `https://codeforces.com/contest/${contestId}`, startsAtUtc: manualDate ? `${manualDate}T00:00:00Z` : null, problems: manualProblems });
      setImportMessage("Manual Contest saved through the canonical import and statement snapshot contract.");
      setRetryNonce((value) => value + 1);
    } catch (error) {
      const code = String(error);
      setImportMessage(code.includes("manual_manifest_conflict") ? "This Contest identity already has a different manifest. Existing data was not changed." : "Manual Contest was not saved. Check the explicit identities and all required fields.");
    } finally { setImporting(false); }
  };
  const updateManualProblem = (position: number, patch: Partial<(typeof manualProblems)[number]>) => setManualProblems((current) => current.map((item, index) => index === position ? { ...item, ...patch } : item));

  return (
    <>
      <PageHeader eyebrow="Contest Library" headingRef={headingRef} title="比赛" />
      <section className="content-panel contest-library-navigation" aria-label="Contest Library navigation">
        <div className="contest-library-navigation__header"><div><p className="eyebrow">Browse</p><h2>Contest archive</h2></div><label>Archive status<select value={archiveFilter} onChange={(event) => setArchiveFilter(event.currentTarget.value as ContestLibraryArchiveFilterDto)}><option value="active">Active contests</option><option value="archived">Archived contests</option><option value="all">All contests</option></select></label></div>
        <div className="contest-library-navigation__levels">
          <div><span className="filter-label">Family</span><div className="filter-options"><button className={familyId === null ? "filter-option filter-option--selected" : "filter-option"} onClick={() => selectFamily(null)} type="button">All contests</button>{families?.map((family) => <button className={familyId === family.familyId ? "filter-option filter-option--selected" : "filter-option"} key={family.familyId} onClick={() => selectFamily(family.familyId)} type="button">{family.displayName}</button>)}</div></div>
          {familyId !== null && series.length > 0 ? <div><span className="filter-label">Series</span><div className="filter-options"><button className={seriesFilter.kind === "any" ? "filter-option filter-option--selected" : "filter-option"} onClick={() => selectSeries({ kind: "any" })} type="button">All series</button><button className={seriesFilter.kind === "unassigned" ? "filter-option filter-option--selected" : "filter-option"} onClick={() => selectSeries({ kind: "unassigned" })} type="button">Unassigned series</button>{series.map((item) => <button className={seriesFilter.kind === "exact" && seriesFilter.seriesId === item.seriesId ? "filter-option filter-option--selected" : "filter-option"} key={item.seriesId} onClick={() => selectSeries({ kind: "exact", seriesId: item.seriesId })} type="button">{item.displayName}</button>)}</div></div> : null}
          {familyId !== null && years.length > 0 ? <div><span className="filter-label">Year</span><div className="filter-options"><button className={yearFilter.kind === "any" ? "filter-option filter-option--selected" : "filter-option"} onClick={() => setYearFilter({ kind: "any" })} type="button">All years</button>{years.includes(null) ? <button className={yearFilter.kind === "unassigned" ? "filter-option filter-option--selected" : "filter-option"} onClick={() => setYearFilter({ kind: "unassigned" })} type="button">Unassigned year</button> : null}{years.filter((year): year is number => year !== null).map((year) => <button className={yearFilter.kind === "exact" && yearFilter.year === year ? "filter-option filter-option--selected" : "filter-option"} key={year} onClick={() => setYearFilter({ kind: "exact", year })} type="button">{year}</button>)}</div></div> : null}
        </div>
        {families === null && !failed ? <p aria-busy="true">Loading contest families…</p> : null}
        {managementMessage ? <p aria-live="polite" className="error-message">{managementMessage}</p> : null}
        <div className="action-row"><details><summary className="secondary-action">Create family</summary><form className="inline-form" onSubmit={createFamily}><label>Family name<input onInput={(event) => setFamilyDraft(event.currentTarget.value)} required value={familyDraft} /></label><button className="primary-action" disabled={managementBusy} type="submit">Create family</button></form></details>{familyId !== null ? <details><summary className="secondary-action">Create series</summary><form className="inline-form" onSubmit={createSeries}><label>Series name<input onInput={(event) => setSeriesDraft(event.currentTarget.value)} required value={seriesDraft} /></label><button className="primary-action" disabled={managementBusy} type="submit">Create series</button></form></details> : null}</div>
        {familyId !== null && families?.find((family) => family.familyId === familyId) ? <div className="management-list"><div><strong>Selected family</strong><button className="text-button" onClick={() => { setEditingFamily(familyId); setFamilyDraft(families.find((family) => family.familyId === familyId)?.displayName ?? ""); }} type="button">Rename</button></div>{editingFamily === familyId ? <form className="inline-form" onSubmit={(event) => void renameFamily(event, familyId)}><label>Family name<input onInput={(event) => setFamilyDraft(event.currentTarget.value)} required value={familyDraft} /></label><button className="primary-action" disabled={managementBusy} type="submit">Save name</button><button className="secondary-action" onClick={() => setEditingFamily(null)} type="button">Cancel</button></form> : null}</div> : null}
        {familyId !== null && series.length > 0 ? <div className="management-list"><strong>Series management</strong>{series.map((item) => <div className="management-list__row" key={item.seriesId}><span>{item.displayName}</span><button className="text-button" onClick={() => { setEditingSeries(item.seriesId); setSeriesDraft(item.displayName); }} type="button">Rename</button>{editingSeries === item.seriesId ? <form className="inline-form" onSubmit={(event) => void renameSeries(event, item.seriesId)}><label>Series name<input onInput={(event) => setSeriesDraft(event.currentTarget.value)} required value={seriesDraft} /></label><button className="primary-action" disabled={managementBusy} type="submit">Save name</button><button className="secondary-action" onClick={() => setEditingSeries(null)} type="button">Cancel</button></form> : null}</div>)}</div> : null}
      </section>
      <form className="content-panel contest-import-form" onSubmit={submitImport}><label>Codeforces contest URL<input autoComplete="off" disabled={importing} onInput={(event) => { setContestUrl(event.currentTarget.value); setImportMessage(null); }} placeholder="https://codeforces.com/contest/1979" required value={contestUrl} /></label><button className="primary-action" disabled={importing} type="submit">{importing ? "Importing…" : "Import contest"}</button>{importMessage ? <p aria-live="polite" className="system-caption">{importMessage}</p> : null}</form>
      <details className="content-panel manual-import-panel"><summary>手动比赛导入</summary><form className="manual-import-form" onSubmit={submitManual}><p>Use explicit Codeforces Contest and Problem identities. Manual import does not guess or merge identities.</p><label>Contest ID<input inputMode="numeric" min="1" onInput={(event) => setManualContestId(event.currentTarget.value)} required type="number" value={manualContestId} /></label><label>Contest title<input onInput={(event) => setManualTitle(event.currentTarget.value)} required value={manualTitle} /></label><label>Contest date<input onInput={(event) => setManualDate(event.currentTarget.value)} required type="date" value={manualDate} /></label>{manualProblems.map((problem, position) => <fieldset className="manual-problem-card" key={position}><legend>Problem {position + 1}</legend><label>Index<input aria-label={`Manual problem ${position + 1} index`} onInput={(event) => updateManualProblem(position, { index: event.currentTarget.value })} required value={problem.index} /></label><label>English title<input aria-label={`Manual problem ${position + 1} title`} onInput={(event) => updateManualProblem(position, { title: event.currentTarget.value })} required value={problem.title} /></label><label>Problem URL<input aria-label={`Manual problem ${position + 1} URL`} onInput={(event) => updateManualProblem(position, { sourceUrl: event.currentTarget.value })} required type="url" value={problem.sourceUrl} /></label><label>Statement text<textarea aria-label={`Manual problem ${position + 1} statement`} onInput={(event) => updateManualProblem(position, { statementText: event.currentTarget.value })} required rows={8} value={problem.statementText} /></label></fieldset>)}<div className="action-row"><button className="secondary-action" onClick={() => setManualProblems((current) => [...current, { index: "", title: "", sourceUrl: "", statementText: "" }])} type="button">Add problem</button><button className="primary-action" disabled={importing} type="submit">Save manual Contest</button></div></form></details>
      {failed ? <section className="empty-state" role="alert"><h2>Contest Library is unavailable</h2><p>Nothing was changed. Retry after the local IPC becomes available.</p><button className="secondary-action" onClick={() => setRetryNonce((value) => value + 1)} type="button">Retry</button></section> : null}
      {loading && !failed ? <section className="empty-state" aria-busy="true"><p>Loading contests…</p></section> : null}
      {!loading && !failed && items ? (
        <>
          <ContestCabinet items={items.slice(0, CONTEST_CABINET_CAPACITY)} navigate={navigate} totalCount={items.length} />
          {items.slice(0, CONTEST_CABINET_CAPACITY).some((item) => !item.archived && item.importStatus === "incomplete") ? (
            <div aria-label="Prototype contest maintenance" className="contest-shelf-maintenance">
              {items.slice(0, CONTEST_CABINET_CAPACITY).filter((item) => !item.archived && item.importStatus === "incomplete").map((item) => (
                <button className="secondary-action" disabled={importing} key={item.contestId} onClick={() => void retryMissing(item.contestId)} type="button">
                  Retry missing snapshots for {displayProblemTitle(String(item.contestId), item.title)}
                </button>
              ))}
            </div>
          ) : null}
          {items.length > CONTEST_CABINET_CAPACITY ? <section className="content-panel contest-library-remainder" aria-label="Remaining contest list"><div><p className="eyebrow">More in this view</p><h2>Remaining contests</h2></div><ul className="detail-list">{items.slice(CONTEST_CABINET_CAPACITY).map((item) => <li key={item.contestId}><button className="list-link" onClick={() => navigate(`/contests/${item.contestId}`)} type="button"><strong>{displayProblemTitle(String(item.contestId), item.title)}</strong><span>Codeforces {item.contestId} · {item.problemCount} problems · {item.archived ? "Archived" : item.importStatus === "complete" ? "Imported" : `${item.missingSnapshotCount} snapshots missing`}</span></button>{!item.archived && item.importStatus === "incomplete" ? <button className="secondary-action" disabled={importing} onClick={() => void retryMissing(item.contestId)} type="button">Retry missing snapshots</button> : null}</li>)}</ul></section> : null}
        </>
      ) : null}
    </>
  );
}

function ContestShelf({ navigate }: { navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [items, setItems] = useState<ContestShelfItemDto[] | null>(null);
  const [failed, setFailed] = useState(false);
  const [contestUrl, setContestUrl] = useState("");
  const [importing, setImporting] = useState(false);
  const [importMessage, setImportMessage] = useState<string | null>(null);
  const [manualContestId, setManualContestId] = useState("");
  const [manualTitle, setManualTitle] = useState("");
  const [manualDate, setManualDate] = useState("");
  const [manualProblems, setManualProblems] = useState([{ index: "A", title: "", sourceUrl: "", statementText: "" }]);
  const [showArchived, setShowArchived] = useState(false);
  useEffect(() => { getContestShelf().then(setItems).catch(() => setFailed(true)); }, []);
  const submitImport = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (importing) return;
    setImporting(true); setImportMessage(null);
    try {
      const result = await importCodeforcesContest(contestUrl);
      setImportMessage(result.importStatus === "complete" ? "比赛已完整导入。" : `比赛已保存，仍有 ${result.missingSnapshotProblems.length} 道题的题面尚未导入。`);
      setItems(await getContestShelf());
    } catch (error) {
      setImportMessage(contestImportErrorMessage(error));
    } finally { setImporting(false); }
  };
  const retryMissing = async (contestId: number) => {
    if (importing) return;
    setImporting(true); setImportMessage(null);
    try {
      const result = await importCodeforcesContest(`https://codeforces.com/contest/${contestId}`);
      setImportMessage(result.importStatus === "complete" ? "缺失题面已补齐，比赛导入完成。" : `重试完成，仍有 ${result.missingSnapshotProblems.length} 道题的题面尚未导入。`);
      setItems(await getContestShelf());
    } catch (error) {
      setImportMessage(contestImportErrorMessage(error));
    } finally { setImporting(false); }
  };
  const submitManual = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (importing) return; setImporting(true); setImportMessage(null);
    try {
      const contestId = Number(manualContestId);
      await importManualCodeforcesContest({ contestId, title: manualTitle, sourceUrl: `https://codeforces.com/contest/${contestId}`, startsAtUtc: manualDate ? `${manualDate}T00:00:00Z` : null, problems: manualProblems });
      setImportMessage("Manual Contest saved through the canonical import and statement snapshot contract.");
      setItems(await getContestShelf());
    } catch (error) { const code = String(error); setImportMessage(code.includes("manual_manifest_conflict") ? "This Contest identity already has a different manifest. Existing data was not changed." : "Manual Contest was not saved. Check the explicit identities and all required fields."); }
    finally { setImporting(false); }
  };
  const updateManualProblem = (position: number, patch: Partial<(typeof manualProblems)[number]>) => setManualProblems((current) => current.map((item, index) => index === position ? { ...item, ...patch } : item));
  return (
    <>
      <PageHeader eyebrow="M1 · 比赛导入" headingRef={headingRef} title="比赛" />
      <form className="content-panel contest-import-form" onSubmit={submitImport}>
        <label>Codeforces 公开比赛网址
          <input autoComplete="off" disabled={importing} onInput={(event) => { setContestUrl(event.currentTarget.value); setImportMessage(null); }} placeholder="https://codeforces.com/contest/1979" required value={contestUrl} />
        </label>
        <button className="primary-action" disabled={importing} type="submit">{importing ? "导入中…" : "导入比赛"}</button>
        {importMessage ? <p aria-live="polite" className="system-caption">{importMessage}</p> : null}
      </form>
      <details className="content-panel manual-import-panel"><summary>手动比赛导入</summary><form className="manual-import-form" onSubmit={submitManual}><p>请填写明确的 Codeforces 比赛 ID 和标准题号。系统不会使用模糊相似度合并题目。</p><label>比赛 ID<input inputMode="numeric" min="1" onInput={(event) => setManualContestId(event.currentTarget.value)} required type="number" value={manualContestId} /></label><label>比赛标题<input onInput={(event) => setManualTitle(event.currentTarget.value)} required value={manualTitle} /></label><label>比赛日期<input onInput={(event) => setManualDate(event.currentTarget.value)} required type="date" value={manualDate} /></label>{manualProblems.map((problem, position) => <fieldset className="manual-problem-card" key={position}><legend>题目 {position + 1}</legend><label>题号<input aria-label={`手动题目 ${position + 1} 题号`} onInput={(event) => updateManualProblem(position, { index: event.currentTarget.value })} required value={problem.index} /></label><label>英文标题<input aria-label={`手动题目 ${position + 1} 标题`} onInput={(event) => updateManualProblem(position, { title: event.currentTarget.value })} required value={problem.title} /></label><label>题目链接<input aria-label={`手动题目 ${position + 1} 链接`} onInput={(event) => updateManualProblem(position, { sourceUrl: event.currentTarget.value })} required type="url" value={problem.sourceUrl} /></label><label>题面正文<textarea aria-label={`手动题目 ${position + 1} 题面`} onInput={(event) => updateManualProblem(position, { statementText: event.currentTarget.value })} required rows={8} value={problem.statementText} /></label></fieldset>)}<div className="action-row"><button className="secondary-action" onClick={() => setManualProblems((current) => [...current, { index: "", title: "", sourceUrl: "", statementText: "" }])} type="button">添加题目</button><button className="primary-action" disabled={importing} type="submit">保存手动比赛</button></div></form></details>
      {failed ? <section className="empty-state" role="alert"><h2>比赛数据暂不可用</h2><p>无法读取本地系统事实，任何导入状态都没有改变。</p></section> : null}
      {items?.length === 0 ? <section className="empty-state"><h2>尚未导入比赛</h2><p>请输入完整的 Codeforces 比赛网址，例如 https://codeforces.com/contest/1979。</p></section> : null}
      {items?.length ? <section className="content-panel" aria-label="已导入比赛"><button className="secondary-action" onClick={() => setShowArchived((current) => !current)} type="button">{showArchived ? "查看进行中的比赛" : "查看已归档比赛"}</button><ul className="detail-list">{items.filter((item) => item.archived === showArchived).map((item) => <li key={item.contestId}><button className="list-link" onClick={() => navigate(`/contests/${item.contestId}`)} type="button"><strong>{displayProblemTitle(String(item.contestId), item.title)}</strong><span>Codeforces {item.contestId} · {item.problemCount} 道题 · {item.archived ? "已归档" : item.importStatus === "complete" ? "导入完整" : `${item.missingSnapshotCount} 道题面缺失`}</span></button>{!item.archived && item.importStatus === "incomplete" ? <button className="secondary-action" disabled={importing} onClick={() => retryMissing(item.contestId)} type="button">重试缺失题面</button> : null}</li>)}</ul></section> : null}
      {items === null && !failed ? <section className="empty-state" aria-busy="true"><p>正在读取本地比赛…</p></section> : null}
    </>
  );
}

function contestImportErrorMessage(error: unknown): string {
  const code = error instanceof Error ? error.message : String(error);
  if (code.includes("unsupported_contest_url")) {
    return "比赛网址格式不正确。请输入完整地址，例如 https://codeforces.com/contest/1979。";
  }
  if (code.includes("invalid_remote_data")) {
    return "Codeforces 返回的比赛数据无法识别，本地已有导入数据保持不变。";
  }
  if (code.includes("adapter_unavailable")) {
    return "Codeforces 导入组件无法启动，请重新启动 ACM-OS 后再试。";
  }
  return "连接或读取 Codeforces 失败，请检查网络后重试；本地已有导入数据保持不变。";
}

function contestLibraryErrorMessage(error: unknown): string {
  const code = String(error);
  const messages: Record<string, string> = {
    invalid_name: "Name cannot be empty or contain control characters.",
    duplicate_family_name: "That Family name already exists.",
    duplicate_series_name: "That Series name already exists in this Family.",
    family_not_found: "The selected Family no longer exists. Reload and try again.",
    series_not_found: "The selected Series no longer exists. Reload and try again.",
    contest_not_found: "The Contest no longer exists. Return to the Library and reload.",
    placement_not_found: "That archive placement no longer exists. Reload and try again.",
    series_family_mismatch: "That Series belongs to a different Family.",
    duplicate_placement: "This Contest already has the same Family, Series, Year, and ordinal placement.",
    invalid_year: "Year must be empty or a positive whole number.",
    invalid_ordinal: "Ordinal must be empty or a positive whole number.",
    contest_library_persistence_unavailable: "Contest Library is temporarily unavailable. Retry without changing local data.",
    contest_library_integrity_violation: "Contest Library data failed an integrity check. No local data was changed.",
  };
  return messages[code] ?? "Contest Library operation failed. Existing data was not changed.";
}

function ProblemIndex({ navigate }: { navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [items, setItems] = useState<LightweightProblemItemDto[] | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => { getLightweightProblems().then(setItems).catch(() => setFailed(true)); }, []);
  return (
    <>
      <PageHeader eyebrow="M1 · Lightweight Problems" headingRef={headingRef} title="我的题库" />
      {failed ? <section className="empty-state" role="alert"><h2>Problem index is unavailable</h2><p>No local learning state was changed.</p></section> : null}
      {items?.length === 0 ? <section className="empty-state"><h2>No lightweight problems yet</h2><p>Imported contest problems will appear here without creating Markdown notes.</p></section> : null}
      {items?.length ? <section className="content-panel" aria-label="轻量题目"><ul className="detail-list">{items.map((item) => <li key={`${item.contestId}-${item.index}`}><button className="list-link" onClick={() => navigate(`/problems/${item.contestId}/${item.index}`)} type="button"><strong>{item.index}. {displayProblemTitle(item.index, item.title)}</strong><span>Codeforces {item.contestId}{item.rating ? ` · ${item.rating}` : ""} · {item.hasStatementSnapshot ? "题面已保存" : "题面待获取"}</span></button></li>)}</ul></section> : null}
      {items === null && !failed ? <section className="empty-state" aria-busy="true"><p>Loading local problems…</p></section> : null}
    </>
  );
}

function ContestPlacementPanel({
  contestId,
  placements,
  error,
  editor,
  busy,
  onRetry,
  onEdit,
  onSaved,
  onBusy,
}: {
  contestId: number;
  placements: ContestLibraryPlacementDto[] | null;
  error: string | null;
  editor: ContestLibraryPlacementDto | "new" | null;
  busy: boolean;
  onRetry: () => void;
  onEdit: (value: ContestLibraryPlacementDto | "new" | null) => void;
  onSaved: () => void;
  onBusy: (value: boolean) => void;
}) {
  const [families, setFamilies] = useState<ContestLibraryFamilyDto[]>([]);
  const [series, setSeries] = useState<ContestLibrarySeriesDto[]>([]);
  const [familyId, setFamilyId] = useState(0);
  const [seriesId, setSeriesId] = useState<number | null>(null);
  const [year, setYear] = useState("");
  const [ordinal, setOrdinal] = useState("");
  const [formError, setFormError] = useState<string | null>(null);
  const [removeTarget, setRemoveTarget] = useState<ContestLibraryPlacementDto | null>(null);
  const removeButtonRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const seriesRequest = useRef(0);

  useEffect(() => {
    if (editor === null) return;
    let active = true;
    listContestLibraryFamilies().then((next) => {
      if (!active) return;
      setFamilies(next);
      const nextFamily = editor === "new" ? next[0]?.familyId ?? 0 : editor.familyId;
      setFamilyId(nextFamily);
      setSeriesId(editor === "new" ? null : editor.seriesId);
      setYear(editor === "new" || editor.year === null ? "" : String(editor.year));
      setOrdinal(editor === "new" || editor.ordinal === null ? "" : String(editor.ordinal));
      setSeries([]);
      const request = ++seriesRequest.current;
      if (nextFamily > 0) listContestLibrarySeries(nextFamily)
        .then((rows) => { if (active && request === seriesRequest.current) setSeries(rows); })
        .catch((cause) => { if (active && request === seriesRequest.current) setFormError(contestLibraryErrorMessage(cause)); });
    }).catch((cause) => { if (active) setFormError(contestLibraryErrorMessage(cause)); });
    return () => { active = false; };
  }, [editor]);

  useEffect(() => {
    if (!removeTarget) return;
    const dialog = dialogRef.current;
    dialog?.querySelector<HTMLElement>("button")?.focus();
    const close = (event: KeyboardEvent) => {
      if (event.key === "Escape") { setRemoveTarget(null); queueMicrotask(() => removeButtonRef.current?.focus()); }
    };
    document.addEventListener("keydown", close);
    return () => document.removeEventListener("keydown", close);
  }, [removeTarget]);

  const changeFamily = (next: number) => {
    setFamilyId(next); setSeriesId(null); setSeries([]); setFormError(null);
    const request = ++seriesRequest.current;
    if (next > 0) listContestLibrarySeries(next).then((rows) => { if (request === seriesRequest.current) setSeries(rows); }).catch((cause) => { if (request === seriesRequest.current) setFormError(contestLibraryErrorMessage(cause)); });
  };
  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); if (busy || familyId <= 0) return;
    onBusy(true); setFormError(null);
    try {
      const values = { familyId, seriesId, year: year === "" ? null : Number(year), ordinal: ordinal === "" ? null : Number(ordinal) };
      if (editor === "new") await createContestLibraryPlacement({ contestId, ...values });
      else if (editor) await updateContestLibraryPlacement({ placementId: editor.placementId, ...values });
      onSaved();
    } catch (cause) { setFormError(contestLibraryErrorMessage(cause)); }
    finally { onBusy(false); }
  };
  const remove = async () => {
    if (!removeTarget || busy) return;
    onBusy(true); setFormError(null);
    try { await removeContestLibraryPlacement(removeTarget.placementId); setRemoveTarget(null); onSaved(); }
    catch (cause) { setFormError(contestLibraryErrorMessage(cause)); }
    finally { onBusy(false); }
  };

  return <section className="content-panel contest-placement-panel" aria-label="Contest archive placements">
    <div className="contest-placement-panel__header"><div><h2>Archive placements</h2><p>Removing a placement removes only this archive location, never the Contest.</p></div><button className="primary-action" onClick={() => onEdit("new")} type="button">Add placement</button></div>
    {error ? <div role="alert"><p>{error}</p><button className="secondary-action" onClick={onRetry} type="button">Retry</button></div> : null}
    {placements === null && !error ? <p aria-busy="true">Loading archive placements…</p> : null}
    {placements?.length === 0 ? <p className="safe-note">No archive placement yet. This Contest remains available in All contests.</p> : null}
    {placements?.length ? <ul className="placement-list">{placements.map((item) => <li key={item.placementId}><div><strong>{item.familyName}</strong><span>{item.seriesName ?? "No series"} · {item.year ?? "Unassigned year"}{item.ordinal === null ? "" : ` · #${String(item.ordinal).padStart(2, "0")}`}</span></div><div className="action-row"><button className="secondary-action" onClick={() => onEdit(item)} type="button">Edit</button><button className="danger-action" onClick={() => setRemoveTarget(item)} ref={removeButtonRef} type="button">Remove placement</button></div></li>)}</ul> : null}
    {editor ? <form className="placement-form" onSubmit={submit}><h3>{editor === "new" ? "Add archive placement" : "Edit archive placement"}</h3><label>Family<select disabled={busy} onChange={(event) => changeFamily(Number(event.currentTarget.value))} required value={familyId}>{families.map((family) => <option key={family.familyId} value={family.familyId}>{family.displayName}</option>)}</select></label><label>Series<select disabled={busy} onChange={(event) => setSeriesId(event.currentTarget.value === "" ? null : Number(event.currentTarget.value))} value={seriesId ?? ""}><option value="">No series</option>{series.map((item) => <option key={item.seriesId} value={item.seriesId}>{item.displayName}</option>)}</select></label><label>Year<input disabled={busy} inputMode="numeric" min="1" onChange={(event) => setYear(event.currentTarget.value)} placeholder="Unassigned" type="number" value={year} /></label><label>Ordinal<input disabled={busy} inputMode="numeric" min="1" onChange={(event) => setOrdinal(event.currentTarget.value)} placeholder="Optional" type="number" value={ordinal} /></label>{formError ? <p className="error-message" role="alert">{formError}</p> : null}<div className="action-row"><button className="primary-action" disabled={busy || familyId <= 0} type="submit">{busy ? "Saving…" : "Save placement"}</button><button className="secondary-action" disabled={busy} onClick={() => onEdit(null)} type="button">Cancel</button></div></form> : null}
    {removeTarget ? <div className="modal-backdrop"><div aria-describedby="remove-placement-description" aria-labelledby="remove-placement-title" aria-modal="true" ref={dialogRef} role="dialog"><h2 id="remove-placement-title">Remove this archive placement?</h2><p id="remove-placement-description">This removes the {removeTarget.familyName} archive location only. The Contest, Problems, Facts, Review history, and Markdown remain unchanged.</p><div className="action-row"><button className="danger-action" disabled={busy} onClick={() => void remove()} type="button">Remove placement</button><button className="secondary-action" disabled={busy} onClick={() => { setRemoveTarget(null); queueMicrotask(() => removeButtonRef.current?.focus()); }} type="button">Cancel</button></div></div></div> : null}
  </section>;
}

function ContestDetail({ contestId, navigate }: { contestId: number; navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [detail, setDetail] = useState<ContestDetailDto | null>(null);
  const [failed, setFailed] = useState(false);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [facts, setFacts] = useState<Record<string, ContestFinalResultDto>>({});
  const [upsolveDecisions, setUpsolveDecisions] = useState<Record<string, ContestUpsolveDecisionDto>>({});
  const [correctingIndex, setCorrectingIndex] = useState<string | null>(null);
  const [analysisRaw, setAnalysisRaw] = useState("");
  const [analysisPreview, setAnalysisPreview] = useState<ContestAiAnalysisPreviewDto | null>(null);
  const [analysisBusy, setAnalysisBusy] = useState(false);
  const [deletePreview, setDeletePreview] = useState<ContestDeletePreviewDto | null>(null);
  const [managing, setManaging] = useState(false);
  const [placements, setPlacements] = useState<ContestLibraryPlacementDto[] | null>(null);
  const [placementError, setPlacementError] = useState<string | null>(null);
  const [placementEditor, setPlacementEditor] = useState<ContestLibraryPlacementDto | "new" | null>(null);
  const [placementBusy, setPlacementBusy] = useState(false);
  const loadPlacements = () => {
    setPlacementError(null);
    listContestLibraryPlacements(contestId).then(setPlacements).catch((error) => setPlacementError(contestLibraryErrorMessage(error)));
  };
  useEffect(() => { getContestDetail(contestId).then(setDetail).catch(() => setFailed(true)); loadPlacements(); }, [contestId]);
  useEffect(() => { if (detail) headingRef.current?.focus(); }, [detail]);
  if (failed) return <section className="empty-state" role="alert"><h1 ref={headingRef} tabIndex={-1}>Contest is unavailable</h1><p>The local contest detail could not be read.</p></section>;
  if (!detail) return <section className="empty-state" aria-busy="true"><h1 ref={headingRef} tabIndex={-1}>Loading contest</h1></section>;
  const submitFacts = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault(); setSaving(true); setMessage(null);
    try {
      const next = await completeContestFacts(contestId, detail.problems.map((problem) => ({ index: problem.index, finalContestResult: facts[problem.index] ?? "unknown", upsolveDecision: upsolveDecisions[problem.index] ?? "undecided" })));
      setDetail(next); setMessage("赛后事实快照已完成。比赛结果将保留为历史事实。");
    } catch (error) { setMessage(contestFactsErrorMessage(error)); }
    finally { setSaving(false); }
  };
  const correctFacts = async (problem: ContestDetailDto["problems"][number]) => {
    const finalContestResult = facts[problem.index] ?? problem.finalContestResult ?? "unknown";
    const upsolveDecision = upsolveDecisions[problem.index] ?? problem.upsolveDecision;
    setCorrectingIndex(problem.index); setMessage(null);
    try { setDetail(await correctContestProblemFacts(contestId, problem.index, finalContestResult, upsolveDecision)); setMessage("纠错已保存，并保留 Correction Event。"); }
    catch (error) { setMessage(contestCorrectionErrorMessage(error)); }
    finally { setCorrectingIndex(null); }
  };
  const previewAnalysis = async () => {
    setAnalysisBusy(true); setMessage(null);
    try { setAnalysisPreview(await previewContestAiAnalysis(contestId, analysisRaw)); }
    catch { setMessage("Analysis preview failed. Raw text was not saved."); }
    finally { setAnalysisBusy(false); }
  };
  const saveAnalysis = async () => {
    setAnalysisBusy(true); setMessage(null);
    try { const next = await saveContestAiAnalysis(contestId, analysisRaw); setDetail(next); setAnalysisPreview(null); setMessage("Post-contest analysis saved without changing contest facts or learning state."); }
    catch { setMessage("Analysis was not saved. Existing analysis and contest facts are unchanged."); }
    finally { setAnalysisBusy(false); }
  };
  const toggleArchive = async () => { setManaging(true); setMessage(null); try { setDetail(await setContestArchived(contestId, !detail.archived)); } catch { setMessage("Contest archive state was not changed."); } finally { setManaging(false); } };
  const loadDeletePreview = async () => { setManaging(true); setMessage(null); try { setDeletePreview(await previewDeleteContest(contestId)); } catch { setMessage("Delete preview is unavailable; nothing was deleted."); } finally { setManaging(false); } };
  const confirmDelete = async () => { setManaging(true); setMessage(null); try { await deleteContest(contestId); navigate("/contests", { replace: true }); } catch { setMessage("Contest was not deleted. Existing facts and Problems are unchanged."); setManaging(false); } };
  return <>
    <PageHeader eyebrow="M7 · 比赛事实" headingRef={headingRef} title={displayProblemTitle(String(detail.contestId), detail.title)} />
    <section className="content-panel"><p>Codeforces {detail.contestId} · {detail.contestDate ?? "日期缺失"} · {detail.importStatus === "complete" ? "导入完整" : "导入不完整"} · {detail.factsStatus === "completed" ? "赛后整理已完成" : "待赛后整理"}</p><a href={detail.sourceUrl} rel="noreferrer" target="_blank">Open original contest</a></section>
    <ContestPlacementPanel contestId={contestId} placements={placements} error={placementError} editor={placementEditor} busy={placementBusy} onRetry={loadPlacements} onEdit={setPlacementEditor} onSaved={() => { setPlacementEditor(null); loadPlacements(); }} onBusy={setPlacementBusy} />
    <section className="content-panel contest-management" aria-label="Contest management"><h2>Contest management</h2><div className="action-row"><button className="secondary-action" disabled={managing} onClick={() => void toggleArchive()} type="button">{detail.archived ? "Restore Contest" : "Archive Contest"}</button>{deletePreview ? null : <button className="danger-action" disabled={managing} onClick={() => void loadDeletePreview()} type="button">Preview delete</button>}</div>{deletePreview ? <div role="alert"><p>Delete {deletePreview.contestTitle}: remove the Contest, its Facts, Analysis, and {deletePreview.relationshipCount} Contest-Problem relationships.</p><p>Preserve {deletePreview.preservedProblemCount} global Problems with identity or history. Clean up {deletePreview.cleanupProblemCount} unreferenced history-free Lightweight Problems.</p><div className="action-row"><button className="secondary-action" onClick={() => setDeletePreview(null)} type="button">Cancel</button><button className="danger-action" disabled={managing} onClick={() => void confirmDelete()} type="button">Delete Contest</button></div></div> : null}</section>
    <form className="content-panel contest-facts" onSubmit={submitFacts} aria-label="Contest facts snapshot"><h2>Problems</h2><p>比赛结果与赛后补题决策是历史快照；当前学习状态始终实时读取，不会覆盖它们。</p><ul className="contest-facts-list">{detail.problems.map((problem) => <li key={problem.index}><button className="list-link" onClick={() => navigate(`/problems/${problem.contestId}/${problem.index}`)} type="button"><strong>{problem.index}. {displayProblemTitle(problem.index, problem.title)}</strong><span>当前学习状态：{learningStatusLabel(problem.liveLearningStatus)}</span></button><label>比赛最终结果<select disabled={saving || correctingIndex === problem.index} value={facts[problem.index] ?? problem.finalContestResult ?? "unknown"} onChange={(event) => { const value = event.currentTarget.value as ContestFinalResultDto; setFacts((current) => ({ ...current, [problem.index]: value })); }}>{contestResultOptions.map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label><label>比赛结束时补题决策<select disabled={saving || correctingIndex === problem.index} value={upsolveDecisions[problem.index] ?? problem.upsolveDecision} onChange={(event) => { const value = event.currentTarget.value as ContestUpsolveDecisionDto; setUpsolveDecisions((current) => ({ ...current, [problem.index]: value })); }}>{contestUpsolveOptions.map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label>{detail.factsStatus === "completed" ? <button className="secondary-action" disabled={correctingIndex === problem.index} onClick={() => void correctFacts(problem)} type="button">{correctingIndex === problem.index ? "保存纠错中…" : "保存纠错"}</button> : null}</li>)}</ul>{detail.factsStatus === "pending" ? <button className="primary-action" disabled={saving || detail.importStatus !== "complete" || detail.contestDate === null} type="submit">{saving ? "保存中…" : "完成赛后整理"}</button> : null}{message ? <p aria-live="polite" className="system-caption">{message}</p> : null}</form>
    {detail.corrections.length ? <section className="content-panel"><h2>Correction history</h2><ul className="detail-list">{detail.corrections.map((event) => <li key={event.correctionId}><strong>{event.problemIndex} · {event.field === "finalContestResult" ? "比赛结果" : "补题决策"}</strong><span>{event.oldValue} → {event.newValue} · {event.correctedAtUtc}</span></li>)}</ul></section> : null}
    <section className="content-panel contest-analysis" aria-label="Post-contest AI analysis"><h2>Post-Contest AI Analysis</h2><p>Paste the fixed external AI template. Preview never saves; Save/Replace stores raw text and parsed sections only.</p><label>Raw text<textarea aria-label="Contest AI analysis raw text" rows={8} value={analysisRaw} onInput={(event) => { setAnalysisRaw(event.currentTarget.value); setAnalysisPreview(null); }} /></label><div className="action-row"><button className="secondary-action" disabled={analysisBusy || analysisRaw.trim() === ""} onClick={() => void previewAnalysis()} type="button">Parse preview</button><button className="primary-action" disabled={analysisBusy || !analysisPreview} onClick={() => void saveAnalysis()} type="button">{detail.aiAnalysis ? "Replace analysis" : "Save analysis"}</button></div>{analysisPreview ? <div aria-live="polite"><strong>Preview: {analysisPreview.parseStatus.toUpperCase()}</strong><pre>{analysisPreview.parsedProjectionJson}</pre></div> : null}{detail.aiAnalysis ? <details><summary>Saved raw analysis ({detail.aiAnalysis.parseStatus.toUpperCase()})</summary><pre>{detail.aiAnalysis.rawText}</pre><p>Updated {detail.aiAnalysis.updatedAtUtc}</p></details> : <p>No saved analysis.</p>}</section>
  </>;
}

const contestResultOptions: Array<[ContestFinalResultDto, string]> = [["unknown", "未知 / 未记录"], ["notAttempted", "未尝试"], ["accepted", "AC"], ["wrongAnswer", "WA"], ["timeLimitExceeded", "TLE"], ["memoryLimitExceeded", "MLE"], ["runtimeError", "RE"], ["compilationError", "CE"], ["otherFailed", "其他未通过"]];
const contestUpsolveOptions: Array<[ContestUpsolveDecisionDto, string]> = [["undecided", "未决定"], ["planned", "计划补"], ["notPlanned", "暂不补"]];
function contestFactsErrorMessage(error: unknown): string { const code = String(error); if (code.includes("contest_import_incomplete")) return "全部题面导入完整后才能完成赛后整理。"; if (code.includes("contest_date_missing")) return "比赛日期缺失，暂时不能形成正式快照。"; if (code.includes("contest_problem_set_mismatch")) return "题目列表发生变化，请重新加载后再试。"; if (code.includes("contest_facts_already_completed")) return "赛后事实快照已经完成；后续修改必须通过纠错事件。"; return "赛后事实未保存，本地已有历史保持不变。"; }
function contestCorrectionErrorMessage(error: unknown): string { const code = String(error); if (code.includes("contest_correction_no_change")) return "没有需要保存的事实变化。"; if (code.includes("contest_facts_not_completed")) return "完成赛后整理后才能记录纠错。"; return "纠错未保存，现有比赛事实保持不变。"; }

type ProblemKnowledgeCandidate = KnowledgeCandidateDto | CanonicalKnowledgeCandidateDto;

function ProblemDetail({ contestId = 0, index = "", problemId, navigate }: { contestId?: number; index?: string; problemId?: string; navigate: Navigate }) {
  const canonical = problemId !== undefined;
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [detail, setDetail] = useState<LightweightProblemDetailDto | null>(null);
  const [canonicalDetail, setCanonicalDetail] = useState<CanonicalProblemDetailDto | null>(null);
  const [renderedHtml, setRenderedHtml] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  const [creatingNote, setCreatingNote] = useState(false);
  const [noteMessage, setNoteMessage] = useState<string | null>(null);
  const [noteReadState, setNoteReadState] = useState<PersonalNoteReadStateDto | null>(null);
  const [noteReadFailed, setNoteReadFailed] = useState(false);
  const [openingNote, setOpeningNote] = useState(false);
  const [openNoteFailed, setOpenNoteFailed] = useState(false);
  const [copyMessage, setCopyMessage] = useState<string | null>(null);
  const [relocationCandidates, setRelocationCandidates] = useState<PersonalNoteRelocationCandidateDto[] | null>(null);
  const [relocationMessage, setRelocationMessage] = useState<string | null>(null);
  const [repairingPath, setRepairingPath] = useState<string | null>(null);
  const [showMissingNoteDeleteConfirm, setShowMissingNoteDeleteConfirm] = useState(false);
  const [confirmingMissingNoteDelete, setConfirmingMissingNoteDelete] = useState(false);
  const [lifecycleAction, setLifecycleAction] = useState<ProblemLifecycleActionDto | null>(null);
  const [lifecycleMessage, setLifecycleMessage] = useState<string | null>(null);
  const [showDeletePreview, setShowDeletePreview] = useState(false);
  const [deletingNote, setDeletingNote] = useState(false);
  const [startingReview, setStartingReview] = useState(false);
  const [reviewMessage, setReviewMessage] = useState<string | null>(null);
  const [knowledgeCandidates, setKnowledgeCandidates] = useState<ProblemKnowledgeCandidate[]>([]);
  const [candidateMessage, setCandidateMessage] = useState<string | null>(null);
  const [busyCandidate, setBusyCandidate] = useState<string | null>(null);
  const noteReadSequence = useRef(0);
  const mounted = useRef(true);
  const displayedNotePath = noteReadState?.state === "ready"
    ? noteReadState.vaultRelativePath
    : noteReadState?.state === "locationAnomaly" || noteReadState?.state === "vaultUnavailable"
      ? noteReadState.lastKnownPath
      : (canonical ? canonicalDetail?.personalNote?.vaultRelativePath : detail?.personalNote?.vaultRelativePath);
  const refreshPersonalNote = useCallback(async () => {
    const sequence = ++noteReadSequence.current;
    try {
      const readState = canonical
        ? await getPersonalNoteProjectionById(problemId!)
        : await getPersonalNoteProjection(contestId, index);
      if (!mounted.current || sequence !== noteReadSequence.current) return;
      setNoteReadState(readState);
      setNoteReadFailed(false);
    } catch {
      if (!mounted.current || sequence !== noteReadSequence.current) return;
      setNoteReadFailed(true);
    }
  }, [canonical, contestId, index, problemId]);
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);
  useEffect(() => {
    let active = true;
    const objectUrls: string[] = [];
    setNoteReadState(null);
    setNoteReadFailed(false);
    const detailRequest = canonical
      ? getCanonicalProblemDetail(problemId!)
      : getLightweightProblemDetail(contestId, index);
    detailRequest.then(async (nextDetail) => {
      if (!active) return;
      if (canonical) {
        setCanonicalDetail(nextDetail as CanonicalProblemDetailDto);
      } else {
        setDetail(nextDetail as LightweightProblemDetailDto);
      }
      if (nextDetail.identityType === "personal") {
        if (canonical) {
          try { setKnowledgeCandidates(await loadKnowledgeCandidatesById(problemId!)); }
          catch { setCandidateMessage("Knowledge suggestions are temporarily unavailable."); }
        }
      }
      if (nextDetail.identityType === "personal") {
        await refreshPersonalNote();
        try { setKnowledgeCandidates(canonical ? await loadKnowledgeCandidatesById(problemId!) : await loadKnowledgeCandidates(contestId, index)); }
        catch { setCandidateMessage("Knowledge suggestions are temporarily unavailable."); }
      }
      if (nextDetail.statement.state !== "ready") return;
      const assets = canonical
        ? await getCanonicalStatementAssets(problemId!)
        : await getStatementAssets(contestId, index);
      if (!active) return;
      const assetUrls = new Map<string, string>();
      for (const asset of assets) {
        const objectUrl = URL.createObjectURL(new Blob([new Uint8Array(asset.bytes)], { type: asset.mediaType }));
        objectUrls.push(objectUrl);
        assetUrls.set(asset.localRef, objectUrl);
      }
      setRenderedHtml(sanitizeStatementForRender(nextDetail.statement.sanitizedHtml, assetUrls));
    }).catch(() => { if (active) setFailed(true); });
    return () => {
      active = false;
      noteReadSequence.current += 1;
      for (const objectUrl of objectUrls) URL.revokeObjectURL(objectUrl);
    };
  }, [canonical, contestId, index, problemId, refreshPersonalNote]);
  useEffect(() => {
    if ((canonical ? canonicalDetail?.identityType : detail?.identityType) !== "personal") return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const revalidate = () => { void refreshPersonalNote(); };
    window.addEventListener("focus", revalidate);
    onPersonalNoteInvalidated(revalidate)
      .then((nextUnlisten) => {
        if (disposed) nextUnlisten();
        else unlisten = nextUnlisten;
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      window.removeEventListener("focus", revalidate);
      unlisten?.();
    };
  }, [canonical, canonicalDetail?.identityType, detail?.identityType, refreshPersonalNote]);
  const createNote = async () => {
    if (creatingNote) return;
    setCreatingNote(true);
    setNoteMessage(null);
    try {
      if (canonical) {
        await createPersonalNoteById(problemId!);
        const nextDetail = await getCanonicalProblemDetail(problemId!);
        setCanonicalDetail(nextDetail);
        setNoteMessage("Personal Markdown created and verified.");
        if (nextDetail.identityType === "personal") await refreshPersonalNote();
        return;
      }
      await createPersonalNote(contestId, index);
      const nextDetail = await getLightweightProblemDetail(contestId, index);
      setDetail(nextDetail);
      setNoteMessage("Personal Markdown created and verified.");
      if (nextDetail.identityType === "personal") {
        await refreshPersonalNote();
      }
    } catch (error) {
      setNoteMessage(
        error === "note_target_exists"
          ? "The target Markdown filename already exists. No file or Problem identity was overwritten."
          : "Personal Markdown could not be created. The Problem remains lightweight.",
      );
    } finally {
      setCreatingNote(false);
    }
  };
  const openInObsidian = async () => {
    if (openingNote) return;
    setOpeningNote(true);
    setOpenNoteFailed(false);
    setCopyMessage(null);
    try {
      if (canonical) await openPersonalNoteInObsidianById(problemId!);
      else await openPersonalNoteInObsidian(contestId, index);
    } catch {
      setOpenNoteFailed(true);
    } finally {
      setOpeningNote(false);
    }
  };
  const copyNotePath = async () => {
    if (!displayedNotePath) return;
    try {
      await navigator.clipboard.writeText(displayedNotePath);
      setCopyMessage("Note path copied.");
    } catch {
      setCopyMessage(`Copy this note path: ${displayedNotePath}`);
    }
  };
  const findRelocationCandidates = async () => {
    setRelocationMessage(null);
    try {
      setRelocationCandidates(canonical
        ? await getPersonalNoteRelocationCandidatesById(problemId!)
        : await getPersonalNoteRelocationCandidates(contestId, index));
    } catch {
      setRelocationMessage("Possible locations could not be listed. The existing binding and System Facts were not changed.");
    }
  };
  const confirmRelocationCandidate = async (vaultRelativePath: string) => {
    if (repairingPath) return;
    setRepairingPath(vaultRelativePath);
    setRelocationMessage(null);
    try {
      if (canonical) await rebindPersonalNoteById(problemId!, vaultRelativePath);
      else await rebindPersonalNote(contestId, index, vaultRelativePath);
      setRelocationCandidates(null);
      setRelocationMessage("The selected Markdown was revalidated and the binding was restored.");
      await refreshPersonalNote();
    } catch (error) {
      const code = String(error);
      setRelocationMessage(code.includes("occupied")
        ? "That Markdown is already bound to another Problem and cannot be taken over."
        : "The selected Markdown could not be safely rebound. The previous binding and System Facts were preserved.");
    } finally {
      setRepairingPath(null);
    }
  };
  const confirmMissingNoteDeleted = async () => {
    if (confirmingMissingNoteDelete) return;
    setConfirmingMissingNoteDelete(true);
    setRelocationMessage(null);
    try {
      const lifecycle = canonical
        ? await confirmPersonalNoteDeletedById(problemId!)
        : await confirmPersonalNoteDeleted(contestId, index);
      setDetail((current) => current ? {
        ...current,
        identityType: "lightweight",
        personalNote: null,
        lifecycle,
      } : current);
      if (canonical) setCanonicalDetail((current) => current ? { ...current, identityType: "lightweight", personalNote: null, lifecycle } : current);
      setNoteReadState(null);
      setShowMissingNoteDeleteConfirm(false);
      setNoteMessage("The missing Personal Markdown was confirmed deleted. Historical facts were preserved.");
    } catch (error) {
      const code = String(error);
      setRelocationMessage(code.includes("review_in_progress")
        ? "The note cannot be confirmed deleted while a Review Attempt is in progress."
        : code.includes("vault_unavailable")
          ? "The Vault is unavailable, so absence cannot be confirmed as deletion."
          : "Deletion could not be confirmed. The Personal identity and existing System Facts were preserved.");
    } finally {
      setConfirmingMissingNoteDelete(false);
    }
  };
  const runLifecycleAction = async (action: ProblemLifecycleActionDto) => {
    if (lifecycleAction) return;
    setLifecycleAction(action);
    setLifecycleMessage(null);
    try {
      const lifecycle = canonical
        ? await transitionProblemLifecycleById(problemId!, action)
        : await transitionProblemLifecycle(contestId, index, action);
      if (canonical) setCanonicalDetail((current) => current ? { ...current, lifecycle } : current);
      else setDetail((current) => current ? { ...current, lifecycle } : current);
      setLifecycleMessage("Learning status updated and persisted.");
    } catch {
      setLifecycleMessage("Learning status could not be changed. The previous state was preserved.");
    } finally {
      setLifecycleAction(null);
    }
  };
  const confirmDeletePersonalNote = async () => {
    if (deletingNote) return;
    setDeletingNote(true);
    setNoteMessage(null);
    try {
      const lifecycle = canonical
        ? await deletePersonalNoteById(problemId!)
        : await deletePersonalNote(contestId, index);
      setDetail((current) => current ? {
        ...current,
        identityType: "lightweight",
        personalNote: null,
        lifecycle,
      } : current);
      if (canonical) setCanonicalDetail((current) => current ? { ...current, identityType: "lightweight", personalNote: null, lifecycle } : current);
      setNoteReadState(null);
      setShowDeletePreview(false);
      setNoteMessage("Personal Markdown deleted. Contest and historical facts were preserved.");
    } catch (error) {
      setNoteMessage(
        String(error).includes("review_in_progress")
          ? "Personal Markdown cannot be deleted while this Review Attempt is in progress."
          : "Personal Markdown was not deleted. The Personal Problem and its history were preserved.",
      );
    } finally {
      setDeletingNote(false);
    }
  };
  const beginReview = async () => {
    if (startingReview) return;
    setStartingReview(true);
    setReviewMessage(null);
    try {
      const attempt = canonical
        ? await startOrResumeReviewById(problemId!)
        : await startOrResumeReview(contestId, index);
      navigate(`/review/${attempt.attemptId}`);
    } catch {
      setReviewMessage("Review could not be started. The learning state and schedule were preserved.");
    } finally {
      setStartingReview(false);
    }
  };
  const updateCandidate = async (candidate: ProblemKnowledgeCandidate, disposition: ProblemKnowledgeCandidate["disposition"]) => {
    if (busyCandidate) return;
    setBusyCandidate(candidate.fingerprint);
    setCandidateMessage(null);
    try {
      const updated = canonical
        ? await setKnowledgeCandidateDispositionById(problemId!, candidate.fingerprint, disposition)
        : await setKnowledgeCandidateDisposition(contestId, index, candidate.fingerprint, disposition);
      setKnowledgeCandidates((current) => current.map((item) => item.fingerprint === updated.fingerprint ? updated : item));
      setCandidateMessage(disposition === "ignored"
          ? "已忽略建议，没有修改 Markdown 或关系。"
          : "建议已退回待处理。" );
    } catch {
      setCandidateMessage("The suggestion state could not be changed.");
    } finally { setBusyCandidate(null); }
  };
  const acceptCandidate = async (candidate: ProblemKnowledgeCandidate) => {
    if (busyCandidate || !candidate.knowledgeNodeId) return;
    setBusyCandidate(candidate.fingerprint);
    setCandidateMessage(null);
    try {
      if (canonical) await acceptExistingKnowledgeCandidateById(problemId!, candidate.fingerprint, candidate.knowledgeNodeId);
      else await acceptExistingKnowledgeCandidate(contestId, index, candidate.fingerprint, candidate.knowledgeNodeId);
      setCandidateMessage("Knowledge link was written to current Markdown, re-read, and verified as a formal relation.");
      try {
        setKnowledgeCandidates(canonical ? await loadKnowledgeCandidatesById(problemId!) : await loadKnowledgeCandidates(contestId, index));
        if (!canonical) await refreshPersonalNote();
      } catch {
        setKnowledgeCandidates((current) => current.filter((item) => item.fingerprint !== candidate.fingerprint));
      }
    } catch {
      setCandidateMessage("The current Markdown could not be safely patched. No formal relation was accepted.");
    } finally { setBusyCandidate(null); }
  };
  const acceptCandidateIntent = async (candidate: ProblemKnowledgeCandidate) => {
    if (busyCandidate || candidate.knowledgeNodeId || candidate.disposition !== "pending") return;
    setBusyCandidate(candidate.fingerprint);
    setCandidateMessage(null);
    try {
      const updated = canonical
        ? await setKnowledgeCandidateDispositionById(problemId!, candidate.fingerprint, "acceptedIntent")
        : await setKnowledgeCandidateDisposition(contestId, index, candidate.fingerprint, "acceptedIntent");
      setKnowledgeCandidates((current) => current.map((item) => item.fingerprint === updated.fingerprint ? { ...item, ...updated } : item));
      setCandidateMessage("仅保存意图，没有创建 Markdown、知识节点或正式关系。");
    } catch {
      setCandidateMessage("The intent could not be saved.");
    } finally { setBusyCandidate(null); }
  };
  if (failed) return <section className="empty-state" role="alert"><h1 ref={headingRef} tabIndex={-1}>Problem is unavailable</h1><p>The local problem detail could not be read. No import data was changed.</p></section>;
  if (canonical) {
    if (!canonicalDetail) return <section className="empty-state" aria-busy="true"><h1 ref={headingRef} tabIndex={-1}>Loading problem</h1><p>Reading the local statement snapshot...</p></section>;
    return <>
      <PageHeader eyebrow="M1 canonical Problem" headingRef={headingRef} title={displayProblemTitle("", canonicalDetail.title)} />
      <section className="content-panel">
        <p>{canonicalDetail.rating ? `Rating ${canonicalDetail.rating}` : ""} · {canonicalDetail.identityType === "personal" ? "Personal Problem" : "Lightweight Problem"}</p>
        <a href={canonicalDetail.sourceUrl} rel="noreferrer" target="_blank">Open original problem</a>
        {canonicalDetail.identityType === "lightweight" ? <button className="primary-action" disabled={creatingNote} onClick={() => void createNote()} type="button">{creatingNote ? "Creating…" : "Create Personal Markdown"}</button> : null}
        {canonicalDetail.personalNote ? <p className="safe-note">Personal Markdown: <code>{canonicalDetail.personalNote.vaultRelativePath}</code></p> : null}
        {canonicalDetail.identityType === "personal" ? <p><strong>Current status:</strong> {learningStatusLabel(canonicalDetail.lifecycle.learningStatus)}</p> : null}
        {noteMessage ? <p aria-live="polite" className="system-caption">{noteMessage}</p> : null}
      </section>
      {canonicalDetail.identityType === "personal" ? <>
        {canonicalDetail.personalNote ? <section className="content-panel" aria-labelledby="canonical-personal-note-heading">
          <h2 id="canonical-personal-note-heading">Personal Markdown</h2>
          <p className="safe-note"><code>{displayedNotePath}</code></p>
          {noteReadState?.state === "ready" ? <div className="action-row"><button className="secondary-action" disabled={openingNote} onClick={() => void openInObsidian()} type="button">{openingNote ? "Opening..." : "Open in Obsidian"}</button><button className="secondary-action" onClick={() => void copyNotePath()} type="button">Copy path</button><button className="secondary-action" onClick={() => setShowDeletePreview(true)} type="button">Delete note</button></div> : null}
          {noteReadState?.state === "locationAnomaly" ? <div className="action-row"><button className="secondary-action" onClick={() => void findRelocationCandidates()} type="button">Find note locations</button><button className="secondary-action" onClick={() => setShowMissingNoteDeleteConfirm(true)} type="button">Confirm missing note deleted</button></div> : null}
          {relocationCandidates ? <ul>{relocationCandidates.map((candidate) => <li key={candidate.vaultRelativePath}><code>{candidate.vaultRelativePath}</code>{candidate.occupied ? " (occupied)" : ""}{!candidate.occupied ? <button className="secondary-action" disabled={repairingPath !== null} onClick={() => void confirmRelocationCandidate(candidate.vaultRelativePath)} type="button">Rebind</button> : null}</li>)}</ul> : null}
          {showMissingNoteDeleteConfirm ? <div className="action-row"><button className="primary-action" disabled={confirmingMissingNoteDelete} onClick={() => void confirmMissingNoteDeleted()} type="button">{confirmingMissingNoteDelete ? "Confirming..." : "Confirm"}</button><button className="secondary-action" onClick={() => setShowMissingNoteDeleteConfirm(false)} type="button">Cancel</button></div> : null}
          {openNoteFailed ? <p role="alert" className="system-caption">Obsidian could not open this note.</p> : null}
          {noteReadFailed ? <p role="alert" className="system-caption">The bound Markdown could not be read.</p> : null}
          {showDeletePreview ? <div className="action-row"><button className="primary-action" disabled={deletingNote} onClick={() => void confirmDeletePersonalNote()} type="button">{deletingNote ? "Deleting..." : "Confirm delete"}</button><button className="secondary-action" onClick={() => setShowDeletePreview(false)} type="button">Cancel</button></div> : null}
        </section> : null}
        {noteReadState?.state === "vaultUnavailable" ? <VaultUnavailableNotice /> : null}
        {noteReadState?.state === "ready" ? <PersonalNoteProjectionPanel readState={noteReadState} /> : null}
        <section className="content-panel" aria-labelledby="canonical-learning-lifecycle-heading">
          <h2 id="canonical-learning-lifecycle-heading">Learning lifecycle</h2>
          <p><strong>Current status:</strong> {learningStatusLabel(canonicalDetail.lifecycle.learningStatus)}</p>
          <NextReviewDue localDate={canonicalDetail.lifecycle.nextReviewDueLocalDate} />
          <div className="action-row">{canonicalDetail.lifecycle.availableActions.map((action) => <button className="secondary-action" disabled={lifecycleAction !== null} key={action} onClick={() => void runLifecycleAction(action)} type="button">{lifecycleAction === action ? "Updating…" : lifecycleActionLabel(action)}</button>)}{canonicalDetail.reviewAction ? <button className="primary-action" disabled={startingReview} onClick={() => void beginReview()} type="button">{startingReview ? "Opening Review…" : canonicalDetail.reviewAction === "earlyCheck" ? "Start early check" : canonicalDetail.reviewAction === "continueReview" ? "Continue Review" : "Start Review"}</button> : null}</div>
          {lifecycleMessage ? <p aria-live="polite" className="system-caption">{lifecycleMessage}</p> : null}
          {reviewMessage ? <p aria-live="polite" className="system-caption">{reviewMessage}</p> : null}
        </section>
        <section className="content-panel knowledge-candidates" aria-labelledby="canonical-knowledge-candidates-heading">
          <h2 id="canonical-knowledge-candidates-heading">Prerequisite knowledge suggestions</h2>
          {knowledgeCandidates.length === 0 ? <p className="safe-note">No current suggestions.</p> : <ul>{knowledgeCandidates.map((candidate) => <li key={candidate.fingerprint}><div><strong>{candidate.targetRef}</strong><span>{candidate.disposition}</span></div><div className="action-row">{candidate.disposition !== "ignored" && candidate.knowledgeNodeId ? <button disabled={busyCandidate !== null} onClick={() => void acceptCandidate(candidate)} type="button">Accept existing knowledge</button> : null}{candidate.disposition === "pending" && !candidate.knowledgeNodeId ? <button disabled={busyCandidate !== null} onClick={() => void acceptCandidateIntent(candidate)} type="button">Save intent</button> : null}{candidate.disposition !== "ignored" ? <button className="secondary-action" disabled={busyCandidate !== null} onClick={() => void updateCandidate(candidate, "ignored")} type="button">Ignore</button> : null}{candidate.disposition !== "pending" ? <button className="secondary-action" disabled={busyCandidate !== null} onClick={() => void updateCandidate(candidate, "pending")} type="button">Return to pending</button> : null}</div></li>)}</ul>}
          {candidateMessage ? <p aria-live="polite" className="safe-note">{candidateMessage}</p> : null}
        </section>
        <ProblemReviewHistory problemId={problemId} learningStatus={canonicalDetail.lifecycle.learningStatus} />
      </> : null}
      {canonicalDetail.statement.state === "pending" ? <section className="empty-state"><h2>Statement capture is pending</h2><p>Retry the contest import to capture this statement. Existing data remains unchanged.</p></section> : renderedHtml === null ? <section className="empty-state" aria-busy="true"><p>Preparing the local statement...</p></section> : <section className="content-panel statement-view"><div dangerouslySetInnerHTML={{ __html: renderedHtml }} /></section>}
    </>;
  }
  if (!detail) return <section className="empty-state" aria-busy="true"><h1 ref={headingRef} tabIndex={-1}>Loading problem</h1><p>Reading the local statement snapshot...</p></section>;
  return <>
      <PageHeader eyebrow="M1 · 本地题面快照" headingRef={headingRef} title={detail.index + ". " + displayProblemTitle(detail.index, detail.title)} />
    <section className="content-panel">
      <p>
        Codeforces {detail.contestId}{detail.rating ? " · Rating " + detail.rating : ""}
        {" · "}{detail.identityType === "personal" ? "个人题目" : "轻量题目"}
      </p>
      <a href={detail.sourceUrl} rel="noreferrer" target="_blank">Open original problem</a>
      {detail.identityType === "lightweight" ? (
        <button className="primary-action" disabled={creatingNote} onClick={createNote} type="button">
          {creatingNote ? "Creating note…" : "Create my note"}
        </button>
      ) : null}
      {detail.personalNote ? (
        <>
          <p className="safe-note">
            Personal Markdown: <code>{displayedNotePath}</code>
          </p>
          {noteReadState?.state === "ready" ? (
            <button className="secondary-action" disabled={openingNote} onClick={openInObsidian} type="button">
              {openingNote ? "正在打开…" : "在 Obsidian 中打开并编辑题解"}
            </button>
          ) : null}
          {openNoteFailed ? (
            <div className="external-open-error" role="alert">
              <p>Obsidian could not open this note. Your Personal Problem and learning state were not changed.</p>
              <div className="action-row">
                <button onClick={openInObsidian} type="button">Retry</button>
                <button onClick={copyNotePath} type="button">Copy path</button>
                <button onClick={() => navigate("/settings")} type="button">Check settings</button>
              </div>
              {copyMessage ? <p aria-live="polite" className="system-caption">{copyMessage}</p> : null}
            </div>
          ) : null}
        </>
      ) : null}
      {noteMessage ? <p aria-live="polite" className="system-caption">{noteMessage}</p> : null}
    </section>
    {detail.identityType === "personal" ? (
      <section className="content-panel" aria-labelledby="learning-lifecycle-heading">
        <h2 id="learning-lifecycle-heading">Learning lifecycle</h2>
        <p><strong>Current status:</strong> {learningStatusLabel(detail.lifecycle.learningStatus)}</p>
        <NextReviewDue localDate={detail.lifecycle.nextReviewDueLocalDate} />
        <div className="action-row">
          {detail.lifecycle.availableActions.map((action) => (
            <button
              className={action === "markUnderstood" || action === "startLearning" || action === "joinUpsolve" ? "primary-action" : "secondary-action"}
              disabled={lifecycleAction !== null}
              key={action}
              onClick={() => void runLifecycleAction(action)}
              type="button"
            >
              {lifecycleAction === action ? "Updating…" : lifecycleActionLabel(action)}
            </button>
          ))}
          {detail.reviewAction ? (
            <button className="primary-action" disabled={startingReview} onClick={() => void beginReview()} type="button">
              {startingReview
                ? "Opening Review…"
                : detail.reviewAction === "earlyCheck"
                  ? "Start early check"
                  : detail.reviewAction === "continueReview"
                    ? "Continue Review"
                    : "Start Review"}
            </button>
          ) : null}
        </div>
        {lifecycleMessage ? <p aria-live="polite" className="system-caption">{lifecycleMessage}</p> : null}
        {reviewMessage ? <p aria-live="polite" className="system-caption">{reviewMessage}</p> : null}
      </section>
    ) : null}
    {detail.identityType === "personal" ? (
      <section className="content-panel knowledge-candidates" aria-labelledby="knowledge-candidates-heading">
        <h2 id="knowledge-candidates-heading">Prerequisite knowledge suggestions</h2>
        <p>这些只是前置知识建议，不是知识节点，也不会直接创建正式关系。接受建议前必须先解析目标，并通过独立的安全补丁流程。</p>
        {knowledgeCandidates.length === 0 ? <p className="safe-note">当前个人题目没有前置知识建议。</p> : <ul>{knowledgeCandidates.map((candidate) => <li key={candidate.fingerprint}><div><strong>{candidate.targetRef}</strong><span>{candidate.disposition === "acceptedIntent" ? candidate.knowledgeNodeId ? "已接受意图 · 已找到对应知识 Markdown" : "已接受意图" : candidate.disposition === "ignored" ? "已忽略" : candidate.knowledgeNodeId ? "待处理 · 已找到对应知识 Markdown" : "待处理 · 没有唯一知识节点"}</span></div><div className="action-row">{candidate.disposition !== "ignored" && candidate.knowledgeNodeId ? <button disabled={busyCandidate !== null} onClick={() => void acceptCandidate(candidate)} type="button">接受现有知识</button> : null}{candidate.disposition === "pending" && !candidate.knowledgeNodeId ? <button disabled={busyCandidate !== null} onClick={() => void acceptCandidateIntent(candidate)} type="button">只保存意图</button> : null}{candidate.disposition !== "ignored" ? <button className="secondary-action" disabled={busyCandidate !== null} onClick={() => void updateCandidate(candidate, "ignored")} type="button">不再建议</button> : null}{candidate.disposition !== "pending" ? <button className="secondary-action" disabled={busyCandidate !== null} onClick={() => void updateCandidate(candidate, "pending")} type="button">退回待处理</button> : null}</div></li>)}</ul>}
        {candidateMessage ? <p aria-live="polite" className="safe-note">{candidateMessage}</p> : null}
      </section>
    ) : null}
    {detail.identityType === "personal" ? (
      <section className="content-panel" aria-labelledby="personal-note-danger-heading">
         <h2 id="personal-note-danger-heading">个人笔记操作</h2>
        {!showDeletePreview ? (
          <button className="secondary-action" onClick={() => setShowDeletePreview(true)} type="button">
            Delete my personal note…
          </button>
        ) : (
          <div role="alertdialog" aria-labelledby="delete-note-preview-title" aria-describedby="delete-note-preview-description">
             <h3 id="delete-note-preview-title">删除这份个人 Markdown？</h3>
            <div id="delete-note-preview-description">
              <p>This will delete the bound Markdown, downgrade the Problem to Lightweight, exit its current learning lifecycle, and cancel its active Review schedule.</p>
              <p>Contest history, completed Review history, and historical highest evidence will be preserved.</p>
            </div>
            <div className="action-row">
              <button disabled={deletingNote} onClick={() => setShowDeletePreview(false)} type="button">Cancel</button>
              <button disabled={deletingNote} onClick={() => void confirmDeletePersonalNote()} type="button">
                {deletingNote ? "Deleting…" : "Delete personal note"}
              </button>
            </div>
          </div>
        )}
      </section>
    ) : null}
    {detail.identityType === "personal" ? (
      noteReadFailed ? (
         <section className="empty-state" role="alert"><h2>个人 Markdown 不可用</h2><p>当前绑定文件无法读取，系统事实已保留。</p></section>
      ) : noteReadState === null ? (
         <section className="empty-state" aria-busy="true"><p>正在读取当前个人 Markdown…</p></section>
      ) : noteReadState.state === "vaultUnavailable" ? (
        <VaultUnavailableNotice />
      ) : noteReadState.state === "locationAnomaly" ? (
        <section className="empty-state" role="status">
           <h2>Note location needs attention</h2>
          <p>The original path is missing and no unique relocation was found. The Personal Problem was not deleted or downgraded.</p>
          <button className="secondary-action" onClick={() => void findRelocationCandidates()} type="button">查找可能的位置</button>
          {relocationCandidates ? relocationCandidates.length ? (
            <ul className="detail-list" aria-label="Possible note locations">
              {relocationCandidates.map((candidate) => <li key={candidate.vaultRelativePath}>
                <span>{candidate.vaultRelativePath}{candidate.occupied ? " · already bound" : ""}</span>
                <button disabled={candidate.occupied || repairingPath !== null} onClick={() => void confirmRelocationCandidate(candidate.vaultRelativePath)} type="button">
                  {repairingPath === candidate.vaultRelativePath ? "正在重新验证…" : "使用此 Markdown"}
                </button>
              </li>)}
            </ul>
           ) : <p>当前没有可用于手动重新绑定的 Markdown 文件。</p> : null}
          {relocationMessage ? <p aria-live="polite" className="system-caption">{relocationMessage}</p> : null}
          {showMissingNoteDeleteConfirm ? (
            <div role="alertdialog" aria-labelledby="confirm-missing-note-title">
              <h3 id="confirm-missing-note-title">Confirm that this Markdown was deleted?</h3>
              <p>This does not delete any file. It removes the missing binding, returns the Problem to Lightweight, exits its learning lifecycle, and preserves Contest and Review history.</p>
              <div className="action-row">
                <button disabled={confirmingMissingNoteDelete} onClick={() => setShowMissingNoteDeleteConfirm(false)} type="button">Cancel</button>
                <button className="danger-action" disabled={confirmingMissingNoteDelete} onClick={() => void confirmMissingNoteDeleted()} type="button">
                  {confirmingMissingNoteDelete ? "Revalidating absence…" : "Confirm deleted"}
                </button>
              </div>
            </div>
          ) : <button className="danger-action" onClick={() => setShowMissingNoteDeleteConfirm(true)} type="button">Confirm file was deleted…</button>}
        </section>
      ) : <PersonalNoteProjectionPanel readState={noteReadState} />
    ) : null}
    <ProblemReviewHistory contestId={contestId} index={index} learningStatus={detail.lifecycle.learningStatus} />
    {detail.statement.state === "pending" ? <section className="empty-state"><h2>Statement capture is pending</h2><p>Retry the contest import to capture this statement. Existing data remains unchanged.</p></section> : renderedHtml === null ? <section className="empty-state" aria-busy="true"><p>Preparing the local statement…</p></section> : <section className="content-panel statement-view"><div className="statement-heading-row"><h2>Statement snapshot</h2></div><div dangerouslySetInnerHTML={{ __html: renderedHtml }} /></section>}
  </>;
}

function NextReviewDue({ localDate }: { localDate: string | null }) {
  return localDate ? <p><strong>Next Review due:</strong> {localDate}</p> : null;
}

function VaultUnavailableNotice() {
  return <section className="empty-state" role="status"><h2>Vault is unavailable</h2><p>Live Markdown access is temporarily unavailable. The Personal Problem and its System Facts were preserved.</p></section>;
}

function PersonalNoteProjectionPanel({ readState }: { readState: Extract<PersonalNoteReadStateDto, { state: "ready" }> }) {
  return <section className="content-panel" aria-label="Personal Markdown projection">
    <h2>我的笔记</h2>
    {readState.relocated ? <p className="safe-note">The note binding was restored to its current location.</p> : null}
    <h3>已识别章节</h3>
    {readState.projection.knownSections.length ? <ul>{readState.projection.knownSections.map((section, position) => <li key={`${section.name}-${position}`}>{section.name}</li>)}</ul> : <p>没有找到已识别章节。</p>}
    <h3>解题路线</h3>
    {readState.projection.solutionRoutes.length ? <ol>{readState.projection.solutionRoutes.map((route, position) => <li key={`${route.name}-${position}`}>{route.name}</li>)}</ol> : <p>没有找到解题路线。</p>}
    {readState.projection.warnings.map((warning) => <p className="safe-note" key={`${warning.code}-${warning.name}`}>Duplicate section: {warning.name} ({warning.count})</p>)}
  </section>;
}

const reviewFailureReasonOptions: ReadonlyArray<readonly [ReviewFailureReasonCodeDto, string]> = [
  ["noIdea", "No idea"],
  ["keyPropertyBlocked", "Direction found, key property blocked"],
  ["derivationBlocked", "Formula or derivation blocked"],
  ["cannotImplement", "Algorithm known, could not implement"],
  ["implementationError", "Implementation error"],
  ["boundaryError", "Boundary error"],
  ["complexityError", "Complexity judgement error"],
  ["other", "Other"],
];

function emptyReviewCompletion(attemptId: string): CompleteReviewInputDto {
  return {
    attemptId,
    finalAc: true,
    firstSubmissionResult: "accepted",
    firstSubmissionOther: null,
    finalResult: "accepted",
    finalResultOther: null,
    totalSubmissions: 1,
    ideaIndependent: true,
    implementationIndependent: true,
    debugIndependence: "notNeeded",
    externalHelp: "none",
    failureReasons: [],
  };
}

function submissionResultOptions() {
  const options: Array<[SubmissionResultDto, string]> = [
    ["accepted", "Accepted"],
    ["wrongAnswer", "Wrong answer"],
    ["timeLimitExceeded", "Time limit exceeded"],
    ["memoryLimitExceeded", "Memory limit exceeded"],
    ["runtimeError", "Runtime error"],
    ["compilationError", "Compilation error"],
    ["other", "Other"],
  ];
  return options.map(([value, label]) => <option key={value} value={value}>{label}</option>);
}

function ReviewEvidenceCard({ completed }: { completed: CompletedReviewAttemptDto }) {
  return (
    <section className="review-stage review-evidence-card" aria-labelledby="review-evidence-title">
      <p className="eyebrow">复习已完成 · 证据卡片</p>
      <h2 id="review-evidence-title">{reviewJudgementLabel(completed.judgement)}</h2>
      <p>完成日期：{completed.completedLocalDate}。结果由判定规则 v{completed.attempt.judgementRuleVersion} 根据事实推导，不是手动选择。</p>
      <h3>依据</h3>
      <ul>{completed.evidenceCodes.map((code) => <li key={code}>{reviewEvidenceLabel(code)}</li>)}</ul>
      {completed.failureReasons.length ? <><h3>失败原因</h3><ul>{completed.failureReasons.map((reason) => <li key={reason.code}>{reviewFailureReasonLabel(reason)}</li>)}</ul></> : null}
      <h3>下一状态</h3>
      <p>{learningStatusLabel(completed.lifecycle.learningStatus)}{completed.lifecycle.nextReviewDueLocalDate ? ` · 到期日 ${completed.lifecycle.nextReviewDueLocalDate}` : ""}</p>
    </section>
  );
}

function ReviewHistoryEvidenceCard({ item }: { item: ReviewHistoryItemDto | CanonicalReviewHistoryItemDto }) {
  if (item.status === "void") {
    return <section className="review-stage review-evidence-card"><p className="eyebrow">复习历史</p><h2>已作废的误开复习</h2><p>{item.voidReason}</p><p>复习排程没有改变，已查看的帮助仍保留在历史中。</p></section>;
  }
  return <section className="review-stage review-evidence-card"><p className="eyebrow">复习已完成 · 证据卡片</p><h2>{item.judgement ? reviewJudgementLabel(item.judgement) : "已完成"}</h2><p>完成日期：{item.completedLocalDate}。</p><ul>{item.evidenceCodes.map((code) => <li key={code}>{reviewEvidenceLabel(code)}</li>)}</ul></section>;
}

const emptyMasteryEvidence: ProblemMasteryEvidenceDto = {
  recallsProblem: false,
  multipleSolutionsClear: false,
  knowledgeUnderstood: false,
  implementationFluent: false,
  canAdaptOrCreate: false,
  transferSolvedIndependently: false,
};

const masteryEvidenceLabels: ReadonlyArray<readonly [keyof ProblemMasteryEvidenceDto, string]> = [
  ["recallsProblem", "我能回忆起这道题要解决什么问题"],
  ["multipleSolutionsClear", "我能清楚说出多种解题路线"],
  ["knowledgeUnderstood", "相关知识已经真正理解"],
  ["implementationFluent", "我能快速、清晰地完成实现"],
  ["canAdaptOrCreate", "我理解适用场景，并能迁移或创造相关题目"],
  ["transferSolvedIndependently", "我能独立解决相关迁移题"],
];

function ProblemReviewHistory({ contestId = 0, index = "", problemId, learningStatus }: {
  contestId?: number;
  index?: string;
  problemId?: string;
  learningStatus: LightweightProblemDetailDto["lifecycle"]["learningStatus"];
}) {
  const [history, setHistory] = useState<ReviewHistoryDto | CanonicalReviewHistoryDto | null>(null);
  const [masteryDraft, setMasteryDraft] = useState<ProblemMasteryEvidenceDto>(emptyMasteryEvidence);
  const [loading, setLoading] = useState(false);
  const [savingMastery, setSavingMastery] = useState(false);
  const [error, setError] = useState(false);
  const load = () => {
    setLoading(true);
    setError(false);
    (problemId ? getReviewHistoryById(problemId) : getReviewHistory(contestId, index))
      .then((next) => {
        setHistory(next);
        setMasteryDraft(next.mastery?.current ?? emptyMasteryEvidence);
      })
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  };
  const saveMastery = () => {
    if (!history) return;
    setSavingMastery(true);
    setError(false);
    (problemId ? updateProblemMasteryEvidenceById(problemId, masteryDraft) : updateProblemMasteryEvidence(contestId, index, masteryDraft))
      .then((mastery) => setHistory({ ...history, mastery }))
      .catch(() => setError(true))
      .finally(() => setSavingMastery(false));
  };
  const mastery = history?.mastery;
  const achievedCount = Object.values(masteryDraft).filter(Boolean).length;
  return (
    <section className="content-panel" aria-labelledby="review-history-heading">
      <h2 id="review-history-heading">复习历史</h2>
      {!history ? <button className="secondary-action" disabled={loading} onClick={load} type="button">{loading ? "正在加载…" : "加载复习历史"}</button> : null}
      {error ? <p role="alert">复习历史暂时不可用，历史记录没有改变。</p> : null}
      {history ? <>
        <p><strong>历史最佳复习证据：</strong> {history.historicalBestReview ? reviewJudgementLabel(history.historicalBestReview) : "暂无"}</p>
        <section className="mastery-evidence" aria-labelledby="mastery-evidence-heading">
          <h3 id="mastery-evidence-heading">彻底掌握证据</h3>
          <p><strong>当前：</strong> {achievedCount}/6 项证据 · {learningStatusLabel(learningStatus)}</p>
          <p><strong>历史最高：</strong> {mastery?.historicalThoroughlyDigested ? "已彻底掌握" : "尚未彻底掌握"}{mastery?.firstThoroughlyDigestedLocalDate ? ` · 首次达到 ${mastery.firstThoroughlyDigestedLocalDate}` : ""}</p>
          <p className="safe-note">只有 6/6 才算“彻底掌握”。复习结果为“已掌握”不会自动修改这些由用户确认的事实。</p>
          <fieldset>
            <legend>当前证据</legend>
            {masteryEvidenceLabels.map(([key, label]) => <label key={key}><input checked={masteryDraft[key]} onChange={(event) => setMasteryDraft({ ...masteryDraft, [key]: event.target.checked })} type="checkbox" /> {label}</label>)}
          </fieldset>
          <button className="secondary-action" disabled={savingMastery} onClick={saveMastery} type="button">{savingMastery ? "正在保存…" : "保存当前证据"}</button>
        </section>
        {history.attempts.length === 0 ? <p>暂无复习记录。</p> : <ol className="review-history-list">{history.attempts.map((item) => <li key={item.attempt.attemptId}><strong>{item.status === "void" ? "已作废" : item.status === "inProgress" ? "进行中" : item.judgement ? reviewJudgementLabel(item.judgement) : "已完成"}</strong><span>{reviewAttemptTypeLabel(item.attempt.attemptType)} · 开始时间：{item.attempt.startedAtUtc}</span>{item.completionFacts ? <span>最终 AC：{item.completionFacts.finalAc ? "是" : "否"} · 提交次数：{item.completionFacts.totalSubmissions} · 思路独立：{item.completionFacts.ideaIndependent ? "是" : "否"} · 实现独立：{item.completionFacts.implementationIndependent ? "是" : "否"}</span> : null}{item.helpLevels.length ? <span>使用帮助等级：{item.helpLevels.join(", ")}</span> : null}{item.failureReasons.length ? <span>失败原因：{item.failureReasons.map(reviewFailureReasonLabel).join("；")}</span> : null}</li>)}</ol>}
      </> : null}
    </section>
  );
}

function reviewJudgementLabel(judgement: CompletedReviewAttemptDto["judgement"]): string {
  return judgement === "mastered" ? "已掌握" : judgement === "partial" ? "部分掌握" : "未通过";
}

function reviewFailureReasonLabel(reason: { code: ReviewFailureReasonCodeDto; otherText: string | null }): string {
  return reason.code === "other"
    ? `其他：${reason.otherText ?? ""}`
    : reviewFailureReasonOptions.find(([code]) => code === reason.code)?.[1] ?? reason.code;
}

function reviewEvidenceLabel(code: string): string {
  const labels: Record<string, string> = {
    final_ac: "最终提交通过",
    no_final_ac: "没有最终通过的提交",
    controlled_help_l1: "查看了前置知识名称",
    controlled_help_l2: "查看了提示",
    controlled_help_l3: "查看了前置知识内容",
    controlled_help_l4: "查看了旧思路或旧代码",
    controlled_help_l5: "查看了完整题解",
    external_solving_hint: "记录了外部解题提示",
    external_full_solution: "记录了外部完整题解",
    idea_not_independent: "思路不是独立完成",
    implementation_not_independent: "实现不是独立完成",
    debug_not_needed: "不需要调试",
    debug_independent: "独立完成调试",
    debug_solving_help: "调试时使用了解题帮助",
  };
  return labels[code] ?? code;
}

function learningStatusLabel(status: LightweightProblemDetailDto["lifecycle"]["learningStatus"]): string {
  const labels = {
    unstarted: "未进入学习",
    upsolvePending: "待补",
    learning: "补题中",
    waitingColdStart: "已补懂，等待冷启动验证",
    relearning: "回炉中",
    longTermReview: "长期复习",
  } satisfies Record<LightweightProblemDetailDto["lifecycle"]["learningStatus"], string>;
  return labels[status];
}

function lifecycleActionLabel(action: ProblemLifecycleActionDto): string {
  const labels = {
    joinUpsolve: "加入补题",
    startLearning: "开始学习",
    returnToPending: "放回待补",
    markUnderstood: "我已经补懂",
    withdrawUnderstood: "撤回补懂",
    startRelearning: "重新学习",
    stopLearning: "停止学习此题",
  } satisfies Record<ProblemLifecycleActionDto, string>;
  return labels[action];
}

function reviewAttemptTypeLabel(type: ReviewFocusDto["attempt"]["attemptType"]): string {
  const labels = {
    firstColdStart: "首次冷启动复习",
    longTermReview: "长期复习",
    earlyCheck: "提前检查",
  } satisfies Record<ReviewFocusDto["attempt"]["attemptType"], string>;
  return labels[type];
}

function reviewHelpLevelLabel(level: ReviewHelpLevel): string {
  const labels = {
    1: "Prerequisite names",
    2: "Hints",
    3: "Prerequisite content",
    4: "Old idea / code",
    5: "Full solution",
  } satisfies Record<ReviewHelpLevel, string>;
  return labels[level];
}

function reviewHelpConsequence(consequence: ReviewHelpItemDto["consequence"]): string {
  return consequence === "fail_only" ? "Not passed only" : "Partial at best";
}

function sanitizeStatementForRender(html: string, assetUrls: ReadonlyMap<string, string>): string {
  const document = new DOMParser().parseFromString(html, "text/html");
  const allowedTags = new Set(["DIV", "P", "SPAN", "H1", "H2", "H3", "H4", "PRE", "CODE", "UL", "OL", "LI", "TABLE", "THEAD", "TBODY", "TR", "TH", "TD", "BR", "STRONG", "B", "EM", "I", "SUP", "SUB", "A", "IMG", "HR"]);
  for (const element of Array.from(document.body.querySelectorAll("*"))) {
    if (!allowedTags.has(element.tagName)) {
      element.replaceWith(...Array.from(element.childNodes));
      continue;
    }
    for (const attribute of Array.from(element.attributes)) {
      if (!["class", "title", "alt", "href", "src"].includes(attribute.name.toLowerCase())) {
        element.removeAttribute(attribute.name);
      }
    }
    if (element instanceof HTMLImageElement) {
      const localUrl = assetUrls.get(element.getAttribute("src") ?? "");
      if (!localUrl) {
        element.remove();
        continue;
      }
      element.src = localUrl;
    }
    if (element instanceof HTMLAnchorElement) {
      const href = element.getAttribute("href")?.trim() ?? "";
      if (href !== "#" && !/^https:\/\//i.test(href)) element.setAttribute("href", "#");
      element.target = "_blank";
      element.rel = "noreferrer noopener";
    }
  }
  renderCodeforcesMath(document);
  return document.body.innerHTML;
}

function renderCodeforcesMath(document: Document): void {
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  const textNodes: Text[] = [];
  while (walker.nextNode()) {
    const node = walker.currentNode as Text;
    if (node.data.includes("$$$") && !node.parentElement?.closest("pre, code, .katex")) {
      textNodes.push(node);
    }
  }

  for (const node of textNodes) {
    const source = node.data;
    const pattern = /\$\$\$([\s\S]+?)\$\$\$/g;
    const fragment = document.createDocumentFragment();
    let cursor = 0;
    let rendered = false;
    for (const match of source.matchAll(pattern)) {
      const start = match.index ?? 0;
      fragment.append(source.slice(cursor, start));
      const formula = document.createElement("span");
      formula.className = "codeforces-math";
      formula.innerHTML = katex.renderToString(match[1], {
        displayMode: source.trim() === match[0],
        strict: "ignore",
        throwOnError: false,
        trust: false,
      });
      fragment.append(formula);
      cursor = start + match[0].length;
      rendered = true;
    }
    if (rendered) {
      fragment.append(source.slice(cursor));
      node.replaceWith(fragment);
    }
  }
}

function NotFoundContent({ pathname, navigate }: { pathname: string; navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  return (
    <section className="empty-state">
       <p className="eyebrow">未知路径</p>
       <h1 ref={headingRef} tabIndex={-1}>页面不存在</h1>
      <p><code>{pathname}</code> is not part of the current application map.</p>
       <button className="primary-action" onClick={() => navigate("/today")} type="button">返回今日计划</button>
    </section>
  );
}

function WorkspaceField({
  label,
  value,
  onChange,
  error,
  describedBy,
  inputRef,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  error: string | null;
  describedBy: string;
  inputRef: RefObject<HTMLInputElement | null>;
}) {
  return (
    <label>
      {label}
      <input
        aria-describedby={error ? describedBy : undefined}
        aria-invalid={Boolean(error)}
        autoComplete="off"
        onChange={(event) => onChange(event.currentTarget.value)}
        ref={inputRef}
        required
        value={value}
      />
      {error ? <span className="field-error" id={describedBy}>{error}</span> : null}
    </label>
  );
}

function ShellLink({
  href,
  active,
  navigate,
  children,
}: {
  href: string;
  active: boolean;
  navigate: Navigate;
  children: string;
}) {
  const follow = (event: MouseEvent<HTMLAnchorElement>) => {
    if (event.button === 0 && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey) {
      event.preventDefault();
      navigate(href);
    }
  };
  return <a aria-current={active ? "page" : undefined} href={href} onClick={follow}>{children}</a>;
}

function PageHeader({ eyebrow, title, headingRef }: { eyebrow: string; title: string; headingRef: RefObject<HTMLHeadingElement | null> }) {
  return <header className="page-header"><p className="eyebrow">{eyebrow}</p><h1 ref={headingRef} tabIndex={-1}>{title}</h1></header>;
}

function Brand() {
  return <div className="brand-mark" aria-label="ACM-OS">ACM<span>OS</span></div>;
}

function focusField(
  field: WorkspacePathField | null,
  refs: Record<WorkspacePathField, RefObject<HTMLInputElement | null>>,
) {
  if (field) refs[field].current?.focus();
}

function useRouteFocus<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  useEffect(() => {
    ref.current?.focus();
  }, []);
  return ref;
}

function foundationCaption(foundation: FoundationStatus): string {
  switch (foundation.state) {
    case "checking": return "检查中";
    case "unavailable": return "不可用";
    case "ready": return `已就绪 · ${foundation.foundation.core}`;
  }
}
