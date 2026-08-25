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
import type { FoundationStatus } from "../ipc/foundation";
import {
  confirmKnowledgeUnderstanding,
  confirmKnowledgeMarkdownDeleted,
  acceptExistingKnowledgeCandidate,
  loadKnowledgeDetail,
  loadKnowledgeIndex,
  loadKnowledgeRelocationCandidates,
  loadKnowledgeCandidates,
  loadKnowledgeReevaluationSuggestion,
  openKnowledgeInObsidian,
  rebindKnowledgeNode,
  resolveKnowledgeIdentityConflict,
  setKnowledgeCandidateDisposition,
  type KnowledgeCandidateDto,
  type KnowledgeDetailDto,
  type KnowledgeNodeDto,
  type KnowledgeRelocationCandidateDto,
  type KnowledgeIdentityConflictDto,
  type KnowledgeUnderstandingLevel,
} from "../ipc/knowledge";
import {
  createPersonalNote,
  completeReview,
  confirmPersonalNoteDeleted,
  deletePersonalNote,
  getContestDetail,
  getContestShelf,
  getLightweightProblemDetail,
  getLightweightProblems,
  getPersonalNoteProjection,
  getPersonalNoteRelocationCandidates,
  getReviewFocus,
  getReviewHelpDrawer,
  getReviewAttemptHistory,
  getReviewHistory,
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
  openOriginalOj,
  rebindPersonalNote,
  revealReviewHelp,
  startOrResumeReview,
  transitionProblemLifecycle,
  updateProblemMasteryEvidence,
  voidReview,
  type CompleteReviewInputDto,
  type CompletedReviewAttemptDto,
  type ContestDetailDto,
  type ContestAiAnalysisPreviewDto,
  type ContestDeletePreviewDto,
  type ContestFinalResultDto,
  type ContestUpsolveDecisionDto,
  type ContestShelfItemDto,
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
  getRewardAccountSummary,
  getRewardActivationState,
  listCustomRewards,
  updateCustomReward,
  type CustomRewardDto,
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
      <p aria-live="polite">Validating the local database and workspace configuration…</p>
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
      <h1 ref={headingRef} tabIndex={-1}>Normal startup is blocked</h1>
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
        <a className="skip-link" href="#main-content">Skip to content</a>
        <nav aria-label="Primary">
          <ShellLink active={route.kind === "normal" && route.page === "today"} href="/today" navigate={navigate}>Today</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "contests"} href="/contests" navigate={navigate}>Contests</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "problems"} href="/problems" navigate={navigate}>我的题库</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "knowledge"} href="/knowledge" navigate={navigate}>Knowledge</ShellLink>
          <ShellLink active={route.kind === "normal" && route.page === "reward"} href="/reward" navigate={navigate}>Reward</ShellLink>
        </nav>
        <nav aria-label="Tools" className="tool-nav">
          <ShellLink active={route.kind === "normal" && route.page === "settings"} href="/settings" navigate={navigate}>Settings</ShellLink>
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
  const [terminalHistory, setTerminalHistory] = useState<ReviewHistoryItemDto | null>(null);
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
          <h1>{focus ? `${focus.attempt.index}. ${displayProblemTitle(focus.attempt.index, focus.title)}` : "Isolated review workspace"}</h1>
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
        <section aria-busy="true" className="review-stage"><p>Loading isolated statement…</p></section>
      ) : (
        <>
          <section aria-labelledby="review-attempt-metadata" className="review-stage">
            <h2 id="review-attempt-metadata">首次冷启动复习</h2>
            <p>
              {reviewAttemptTypeLabel(focus.attempt.attemptType)} · 计划日期 {focus.attempt.scheduledDueLocalDate}
              {focus.attempt.startedEarly ? " · 已提前开始" : ""}
            </p>
            <a href={focus.sourceUrl} onClick={openOriginalOjFromReview} rel="noreferrer" target="_blank">Open original OJ</a>
            {ojOpenError ? <p role="alert">{ojOpenError}</p> : null}
            <button className="secondary-action" onClick={openHelpDrawer} ref={helpButtonRef} type="button">Open controlled help</button>
            <p className="safe-note">Old notes, hints, solutions, Contest history, and Review history are not loaded into this Focus view.</p>
          </section>
          <section className="review-stage statement-view" aria-labelledby="review-statement-heading">
            <div className="statement-heading-row"><h2 id="review-statement-heading">题面快照</h2></div>
            <div dangerouslySetInnerHTML={{ __html: renderedHtml }} />
          </section>
          <form className="review-stage review-facts-form" onSubmit={submitCompletion}>
            <div>
              <p className="eyebrow">依据事实，而不是自选评分</p>
              <h2>完成本次复习</h2>
              <p>The system derives Mastered, Partial, or Not passed from these facts and recorded help.</p>
            </div>
            <fieldset>
              <legend>提交事实</legend>
              <label><input checked={completion.finalAc} onChange={(event) => setCompletion({ ...completion, finalAc: event.target.checked })} type="checkbox" /> Final result was AC</label>
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
                <p>Usage recorded at {revealed.revealedAtUtc}.</p>
                <pre>{revealed.contentMarkdown}</pre>
              </section>
            ))}
        </>
      )}
      {helpOpen ? (
        <aside aria-describedby="review-help-description" aria-labelledby="review-help-title" aria-modal="true" className="review-help-drawer" ref={helpDrawerRef} role="dialog">
          <div className="review-help-drawer__header">
            <div>
              <p className="eyebrow">Evidence before reveal</p>
              <h2 id="review-help-title" ref={helpHeadingRef} tabIndex={-1}>Controlled help</h2>
            </div>
            <button className="secondary-action" onClick={closeHelpDrawer} type="button">Close</button>
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
                <button onClick={() => performReveal(pendingHelp, true)} ref={helpConfirmButtonRef} type="button">Confirm and reveal</button>
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
        <PageHeader eyebrow="Tool" headingRef={headingRef} title="Settings" />
        <section aria-labelledby="workspace-settings" className="content-panel">
          <h2 id="workspace-settings">工作区</h2>
          <dl className="detail-list detail-list--paths">
             <dt>当前 Vault</dt><dd>{workspace.activeVaultPath}</dd>
             <dt>题目笔记目录</dt><dd>{workspace.problemRootPath}</dd>
             <dt>知识库目录</dt><dd>{workspace.knowledgeRootPath}</dd>
          </dl>
          <p className="safe-note">Changing the Active Vault requires a future preview-and-confirm flow.</p>
         </section>
         <ManualBackupSettings />
         <WeeklyAcmBudgetSettings />
      </>
    );
  }
  if (page === "contests") return <ContestShelf navigate={navigate} />;
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
    <PageHeader eyebrow="Account" headingRef={headingRef} title="Reward" />
    {activationSuccess ? <p aria-live="polite" className="safe-note">Reward Mode enabled.</p> : null}
    {activationView === "loading" ? <p aria-live="polite">Loading Reward Mode...</p> : null}
    {activationView === "error" ? (
      <section className="empty-state" role="alert">
        <h2>Reward Mode could not be loaded</h2>
        <p>Your Reward settings and account have not been changed.</p>
        <button className="secondary-action" onClick={() => void loadActivation()} type="button">Retry</button>
      </section>
    ) : null}
    {activationView === "inactive" ? (
      <section aria-labelledby="reward-inactive-heading" className="content-panel">
        <h2 id="reward-inactive-heading">Reward Mode is currently off</h2>
        <p>Enabling Reward Mode is explicit and cannot be turned off or reset in Reward V1.</p>
        <p>Historical activity from before activation does not receive positive rewards retroactively.</p>
        <button className="primary-action" onClick={() => { setActivationError(false); setConfirmationOpen(true); }} ref={activationTriggerRef} type="button">Enable Reward Mode</button>
      </section>
    ) : null}
    {activationView === "active" ? (
      <>
        <section aria-labelledby="reward-account-heading" className="content-panel">
          <h2 id="reward-account-heading">Account</h2>
          {accountLoading ? <p aria-live="polite">Loading account summary...</p> : null}
          {accountError ? <div role="alert"><p>Reward account summary could not be loaded.</p><button className="secondary-action" onClick={() => void loadAccount()} type="button">Retry</button></div> : null}
          {account ? <dl className="detail-list"><dt>Level</dt><dd>{account.level}</dd><dt>XP</dt><dd>{account.xp}</dd><dt>Coin</dt><dd>{account.coin}</dd></dl> : null}
        </section>
        <CustomRewardManagement />
      </>
    ) : null}
    {confirmationOpen ? (
      <div className="modal-backdrop">
        <div aria-describedby="reward-activation-description" aria-labelledby="reward-activation-title" aria-modal="true" ref={activationDialogRef} role="alertdialog">
          <h2 id="reward-activation-title">Enable Reward Mode?</h2>
          <p id="reward-activation-description">This is a one-way action in Reward V1. Reward Mode cannot be turned off or reset, and earlier activity will not receive positive rewards retroactively.</p>
          {activationError ? <p className="error-message" role="alert">Reward Mode was not enabled. Try again.</p> : null}
          <div className="button-row">
            <button className="primary-action" disabled={activating} onClick={() => void confirmActivation()} ref={activationConfirmRef} type="button">{activating ? "Enabling..." : "Enable Reward Mode"}</button>
            <button className="secondary-action" disabled={activating} onClick={closeConfirmation} type="button">Cancel</button>
          </div>
        </div>
      </div>
    ) : null}
  </>;
}

function CustomRewardManagement() {
  const archiveTriggerRef = useRef<HTMLButtonElement>(null);
  const archiveDialogRef = useRef<HTMLDivElement>(null);
  const archiveConfirmRef = useRef<HTMLButtonElement>(null);
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
  const load = useCallback(async () => { setLoading(true); setError(false); try { setRewards(await listCustomRewards()); } catch { setError(true); } finally { setLoading(false); } }, []);
  useEffect(() => { void load(); }, [load]);  useEffect(() => {
    if (!archiveTarget) return;
    archiveConfirmRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !pending) { event.preventDefault(); setArchiveTarget(null); queueMicrotask(() => archiveTriggerRef.current?.focus()); }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [archiveTarget, pending]);
  const parseCost = (value: string) => { if (!/^\d+$/.test(value)) return null; const n = Number(value); return Number.isSafeInteger(n) && n > 0 ? n : null; };
  const validate = (nextName: string, nextCost: string) => { const cost = parseCost(nextCost); if (!nextName.trim() || cost === null) { setMutationError("Enter a name and a positive safe whole-number coin cost."); return null; } return { name: nextName.trim(), coinCost: cost }; };
  const create = async (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); if (pending) return; setMutationError(null); const input = validate(name, coinCost); if (!input) return; setPending(true); try { await createCustomReward(input); setName(""); setCoinCost(""); await load(); } catch { setMutationError("Custom Reward change could not be saved. No other rewards were changed."); } finally { setPending(false); } };
  const edit = (reward: CustomRewardDto) => { setEditingId(reward.customRewardId); setEditName(reward.name); setEditCoinCost(String(reward.coinCost)); setMutationError(null); };
  const update = async (event: FormEvent<HTMLFormElement>, reward: CustomRewardDto) => { event.preventDefault(); if (pending) return; setMutationError(null); const values = validate(editName, editCoinCost); if (!values) return; setPending(true); try { await updateCustomReward({ customRewardId: reward.customRewardId, ...values }); setEditingId(null); await load(); } catch { setMutationError("This reward changed elsewhere and is no longer editable. The list was refreshed."); await load(); } finally { setPending(false); } };
  const archive = async () => { if (!archiveTarget || pending) return; setPending(true); setMutationError(null); try { await archiveCustomReward(archiveTarget.customRewardId); setArchiveTarget(null); await load(); } catch { setMutationError("Custom Reward change could not be saved. No other rewards were changed."); await load(); } finally { setPending(false); } };
  const visible = rewards.filter((reward) => showArchived || reward.status === "active");
  return <section aria-labelledby="custom-rewards-heading" className="content-panel">
    <div className="statement-heading-row"><h2 id="custom-rewards-heading">Custom Rewards</h2><label><input checked={showArchived} onChange={(event) => setShowArchived(event.currentTarget.checked)} type="checkbox" /> Show archived</label></div>
    {mutationError ? <p aria-live="assertive" className="error-message" role="alert">{mutationError}</p> : null}
    <form className="action-row" noValidate onSubmit={create}><label>Name<input aria-label="Custom reward name" onInput={(event) => setName(event.currentTarget.value)} value={name} /></label><label>Coin cost<input aria-label="Custom reward coin cost" inputMode="numeric" onInput={(event) => setCoinCost(event.currentTarget.value)} value={coinCost} /></label><button className="primary-action" disabled={pending} type="submit">Create reward</button></form>
    {loading ? <p aria-live="polite">Loading custom rewards...</p> : null}
    {error ? <div role="alert"><p>Custom Rewards could not be loaded.</p><button className="secondary-action" onClick={() => void load()} type="button">Retry</button></div> : null}
    {!loading && !error && visible.length === 0 ? <p>{showArchived ? "No custom rewards yet." : "No active custom rewards yet."}</p> : null}
    {!loading && !error && visible.length > 0 ? <ul className="detail-list">{visible.map((reward) => <li key={reward.customRewardId}><div><strong>{reward.name}</strong><span>{reward.coinCost} Coin</span><span>{reward.status === "archived" ? "Archived" : "Active"}</span></div>{reward.status === "active" ? <div className="action-row"><button className="secondary-action" onClick={() => edit(reward)} type="button">Edit</button><button className="danger-action" onClick={(event) => { archiveTriggerRef.current = event.currentTarget; setArchiveTarget(reward); }} type="button">Archive</button></div> : null}{editingId === reward.customRewardId ? <form className="action-row" noValidate onSubmit={(event) => void update(event, reward)}><label>Name<input aria-label="Edit custom reward name" onInput={(event) => setEditName(event.currentTarget.value)} value={editName} /></label><label>Coin cost<input aria-label="Edit custom reward coin cost" onInput={(event) => setEditCoinCost(event.currentTarget.value)} value={editCoinCost} /></label><button className="primary-action" disabled={pending} type="submit">Save changes</button><button className="secondary-action" disabled={pending} onClick={() => setEditingId(null)} type="button">Cancel</button></form> : null}</li>)}</ul> : null}
    {archiveTarget ? <div className="modal-backdrop"><div aria-describedby="archive-reward-description" aria-labelledby="archive-reward-title" aria-modal="true" ref={archiveDialogRef} role="alertdialog"><h2 id="archive-reward-title">Archive {archiveTarget.name}?</h2><p id="archive-reward-description">Archiving is irreversible in Reward V1. This reward remains readable but cannot be edited, redeemed, or restored.</p><div className="button-row"><button className="danger-action" disabled={pending} onClick={() => void archive()} ref={archiveConfirmRef} type="button">{pending ? "Archiving..." : "Archive reward"}</button><button className="secondary-action" disabled={pending} onClick={() => setArchiveTarget(null)} type="button">Cancel</button></div></div></div> : null}
  </section>;
}
function ManualBackupSettings() {
  const [preview, setPreview] = useState<ManualBackupPreviewDto | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [inventory, setInventory] = useState<BackupInventoryDto | null>(null);
  const prepare = async () => {
    try { setPreview(await previewManualBackup()); setMessage(null); }
    catch { setMessage("Backup preview is temporarily unavailable."); }
  };
  const backup = async () => {
    setBusy(true);
    try {
      const result = await createManualBackup();
      setMessage(`Backup created: ${result.path}`);
      setPreview(null);
      setInventory(await loadBackupInventory());
    } catch {
      setMessage("Backup could not be created. No partial backup was published.");
    } finally { setBusy(false); }
  };
  const inspect = async () => {
    try { setInventory(await loadBackupInventory()); setMessage(null); }
    catch { setMessage("Backup inventory is temporarily unavailable."); }
  };
  return (
    <section aria-labelledby="manual-backup" className="content-panel">
       <h2 id="manual-backup">系统事实备份</h2>
       <p>创建与 SQLite 一致的快照，不会复制或修改 Markdown 文件。</p>
       <button className="secondary-action" onClick={() => void prepare()} type="button">Preview manual backup</button>
       <button className="secondary-action" onClick={() => void inspect()} type="button">Inspect backup inventory</button>
      {preview ? <div role="alertdialog">
        <p>Schema {preview.schemaVersion}; destination <code>{preview.backupDirectory}</code>; filename prefix <code>{preview.filenamePrefix}</code>.</p>
        <button disabled={busy} onClick={() => void backup()} type="button">{busy ? "Creating backup…" : "Create backup"}</button>
      </div> : null}
      {message ? <p aria-live="polite" className="safe-note">{message}</p> : null}
      {inventory ? <div>
        <p>Retention preview: keep {inventory.dailyKeep} daily and {inventory.weeklyKeep} weekly snapshots. Manual and migration backups are protected.</p>
         {inventory.entries.length === 0 ? <p>没有已发布的备份。</p> : <ul className="backup-inventory">
          {inventory.entries.map((entry) => <li key={entry.path}>
            <code>{entry.path}</code><span>{entry.category} · {entry.integrityVerified ? "integrity verified" : "integrity failed"} · {entry.retention}</span>
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
      setError("Knowledge index is temporarily unavailable.");
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
      setError("This Knowledge Markdown could not be read fresh.");
    }
  };

  const confirmUnderstanding = async () => {
    if (!detail) return;
    setSaving(true);
    setMessage(null);
    try {
      const understanding = await confirmKnowledgeUnderstanding(detail.node.knowledgeNodeId, selectedLevel);
      setDetail({ ...detail, understanding });
      setMessage("Understanding status confirmed by you.");
    } catch {
      setMessage("Understanding status could not be saved.");
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
          <div><h3>相关题目</h3>{detail.relatedProblems.length === 0 ? <p>暂无。</p> : <ul>{detail.relatedProblems.map((problem) => <li key={problem.problemId}><button className="list-link" onClick={() => navigate(`/problems/${problem.contestId}/${problem.problemIndex}`)} type="button"><strong>{problem.contestId}{problem.problemIndex} · {problem.title}</strong></button></li>)}</ul>}</div>
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

  return <section aria-labelledby="weekly-acm-budget" className="content-panel"><h2 id="weekly-acm-budget">Weekly ACM budget</h2><p>These defaults repeat every week. Leave a day blank to ask for that day&apos;s budget when Today is first opened.</p>{loading ? <p>Loading weekly budget...</p> : <form className="weekly-budget-form" noValidate onSubmit={submit}><div>{weekBudgetFields.map(([key, label]) => <label key={key}>{label}<input aria-label={`${label} ACM budget in minutes`} min="0" onInput={(event) => { const value = event.currentTarget.value; setDraft((current) => ({ ...current, [key]: value })); }} placeholder="Not set" type="number" value={draft[key]} /></label>)}</div><button className="primary-action" disabled={saving} type="submit">{saving ? "Saving..." : "Save weekly budget"}</button></form>}{message ? <p aria-live="polite" className="safe-note">{message}</p> : null}</section>;
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
    <PageHeader eyebrow="Daily execution" headingRef={headingRef} title="Today" />
    <p aria-atomic="true" aria-live="polite" className="sr-only">{announcement}</p>
    {loading ? <p aria-live="polite">Loading today plan...</p> : null}
    {error ? <p aria-live="assertive" className="error-message">{error}</p> : null}
    {!loading && needsBudget ? <section className="empty-state"><h2>Set today&apos;s budget</h2><p>No weekly default is set for this weekday. Enter any non-negative whole number of minutes; tasks still use complete 30 or 60 minute planning blocks.</p><form className="today-budget-start" noValidate onSubmit={(event) => { event.preventDefault(); const value = Number(initialBudgetDraft); if (initialBudgetDraft.trim() === "" || !Number.isInteger(value) || value < 0) { setError("Daily budget must be a non-negative whole number of minutes."); return; } void load(true); }}><label>Minutes<input min="0" onInput={(event) => setInitialBudgetDraft(event.currentTarget.value)} required type="number" value={initialBudgetDraft} /></label><button className="primary-action" type="submit">Create Today plan</button></form></section> : null}
    {!loading && !snapshot && !needsBudget ? <section className="empty-state"><h2>Plan unavailable</h2><button className="secondary-action" onClick={() => void load(false)} type="button">Retry</button></section> : null}
    {snapshot ? <>
      <section className="today-toolbar" aria-label="Today plan summary">
        <dl><div><dt>Date</dt><dd>{snapshot.localDate}</dd></div><div><dt>Planned</dt><dd>{snapshot.plannedMinutes} min</dd></div><div><dt>Budget</dt><dd>{snapshot.budgetMinutes} min</dd></div><div><dt>Over</dt><dd>{snapshot.overBudgetMinutes} min</dd></div></dl>
        <form noValidate onSubmit={previewBudget}><label>Today override<input aria-label="Daily budget in minutes" min="0" onInput={(event) => setBudgetDraft(event.currentTarget.value)} required type="number" value={budgetDraft} /></label><button className="secondary-action" ref={replanTriggerRef} type="submit">Preview replan</button></form>
      </section>
      {snapshot.entries.length === 0 ? <section className="empty-state"><h2>No tasks fit this budget</h2><p>Only complete 30 or 60 minute tasks are scheduled.</p></section> :
        <ol className="today-list">{snapshot.entries.map((entry, index) => <li className={`today-entry today-entry--${entry.status}`} data-entry-id={entry.entryId} key={entry.entryId} onKeyDown={(event) => { if (event.altKey && event.key === "ArrowUp") { event.preventDefault(); void move(index, -1); } if (event.altKey && event.key === "ArrowDown") { event.preventDefault(); void move(index, 1); } }} tabIndex={0}>
          <div className="today-entry__order"><button aria-label={`Drag ${todayReasonLabel(entry.reason)} to reorder`} className="today-drag-handle" onPointerCancel={clearPointerDrag} onPointerDown={(event) => startPointerDrag(event, entry.entryId)} onPointerMove={movePointerDrag} onPointerUp={finishPointerDrag} title="Drag to reorder" type="button">⋮⋮</button><button aria-label={`Move ${todayReasonLabel(entry.reason)} up`} disabled={index === 0} onClick={() => void move(index, -1)} type="button">↑</button><button aria-label={`Move ${todayReasonLabel(entry.reason)} down`} disabled={index === snapshot.entries.length - 1} onClick={() => void move(index, 1)} type="button">↓</button></div>
          <div className="today-entry__body"><div><span className="today-lane">{todayLaneLabel(entry.lane)}</span><span className={`today-status today-status--${entry.status}`}>{todayStatusLabel(entry.status)}</span>{entry.origin === "manual" ? <span className="today-origin">Manual</span> : null}</div><button className="today-problem-link" onClick={() => navigate(entry.reviewAttemptId && entry.status === "inProgress" ? `/review/${entry.reviewAttemptId}` : `/problems/${entry.contestId}/${entry.problemIndex}`)} type="button"><strong>{displayProblemTitle(entry.problemIndex, entry.problemTitle)}</strong><span>CF {entry.contestId}{entry.problemIndex} · {todayReasonLabel(entry.reason)} · {entry.planningCostMinutes} min</span></button></div>
          {todayDoneAllowed(entry) && entry.status !== "completed" ? <button className="primary-action" disabled={busyEntry === entry.entryId || entry.status === "unavailable"} onClick={() => void done(entry)} type="button">Done for today</button> : null}
        </li>)}</ol>}
      {suggestions && suggestions.suggestions.length > 0 ? <section className="today-suggestions"><h2>Extra suggestions</h2><p>{suggestions.remainingBudgetMinutes} minutes remain. Nothing is added without your action.</p><ul>{suggestions.suggestions.map((item) => <li key={item.problemId}><span><strong>{displayProblemTitle(item.problemIndex, item.problemTitle)}</strong><small>CF {item.contestId}{item.problemIndex} · {todayReasonLabel(item.reason)} · {item.planningCostMinutes} min</small></span><button className="secondary-action" disabled={busyEntry === item.problemId} onClick={() => void acceptSuggestion(item.problemId)} type="button">Add to Today</button></li>)}</ul></section> : null}
    </> : null}
    {replan ? <div className="modal-backdrop"><div aria-describedby="today-replan-description" aria-labelledby="today-replan-title" aria-modal="true" ref={replanDialogRef} role="dialog"><h2 id="today-replan-title">Apply this replan?</h2><p id="today-replan-description">Budget {replan.expectedSnapshot.budgetMinutes} → {replan.proposedBudgetMinutes} minutes. This is a one-day override; the weekly default and next week&apos;s same weekday remain unchanged. Planned work becomes {replan.proposedPlannedMinutes} minutes across {replan.entries.length} entries. Completed, in-progress, and manual entries stay protected.</p><div className="button-row"><button className="primary-action" onClick={() => void applyBudget()} ref={replanApplyRef} type="button">Apply replan</button><button className="secondary-action" onClick={() => { setBudgetDraft(String(snapshot?.budgetMinutes ?? Number(initialBudgetDraft))); setReplan(null); queueMicrotask(() => replanTriggerRef.current?.focus()); }} type="button">Cancel</button></div></div></div> : null}
  </>;
}

function todayDoneAllowed(entry: TodayEntryDto) { return entry.reason === "continueLearning" || entry.reason === "relearn" || entry.reason === "upsolve"; }
function todayLaneLabel(lane: TodayEntryDto["lane"]) { return lane === "carryIn" ? "Carry-in" : lane === "review" ? "Review" : "Study"; }
function todayStatusLabel(status: TodayEntryDto["status"]) { return ({ notStarted: "Not started", inProgress: "In progress", completed: "Completed", unavailable: "Unavailable" } as const)[status]; }
function todayReasonLabel(reason: TodayEntryDto["reason"]) { return ({ continueReview: "Continue Review", continueLearning: "Continue learning", dueFirstColdStart: "First cold-start Review", dueLongTermReview: "Long-term Review", relearn: "Relearn", upsolve: "Upsolve" } as const)[reason]; }
function todayErrorMessage(cause: unknown) { const code = String(cause); if (code.includes("stale_today")) return "The Today plan changed. Reload and try again."; if (code.includes("invalid_today_done")) return "This entry cannot be completed from Today."; if (code.includes("invalid_today_reorder")) return "The saved order changed. Reload and try again."; if (code.includes("today_integrity")) return "Today data failed an integrity check."; return "Today is temporarily unavailable."; }

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
  useEffect(() => { getContestDetail(contestId).then(setDetail).catch(() => setFailed(true)); }, [contestId]);
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

function ProblemDetail({ contestId, index, navigate }: { contestId: number; index: string; navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [detail, setDetail] = useState<LightweightProblemDetailDto | null>(null);
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
  const [knowledgeCandidates, setKnowledgeCandidates] = useState<KnowledgeCandidateDto[]>([]);
  const [candidateMessage, setCandidateMessage] = useState<string | null>(null);
  const [busyCandidate, setBusyCandidate] = useState<string | null>(null);
  const noteReadSequence = useRef(0);
  const mounted = useRef(true);
  const displayedNotePath = noteReadState?.state === "ready"
    ? noteReadState.vaultRelativePath
    : noteReadState?.state === "locationAnomaly" || noteReadState?.state === "vaultUnavailable"
      ? noteReadState.lastKnownPath
      : detail?.personalNote?.vaultRelativePath;
  const refreshPersonalNote = useCallback(async () => {
    const sequence = ++noteReadSequence.current;
    try {
      const readState = await getPersonalNoteProjection(contestId, index);
      if (!mounted.current || sequence !== noteReadSequence.current) return;
      setNoteReadState(readState);
      setNoteReadFailed(false);
    } catch {
      if (!mounted.current || sequence !== noteReadSequence.current) return;
      setNoteReadFailed(true);
    }
  }, [contestId, index]);
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);
  useEffect(() => {
    let active = true;
    const objectUrls: string[] = [];
    setNoteReadState(null);
    setNoteReadFailed(false);
    getLightweightProblemDetail(contestId, index).then(async (nextDetail) => {
      if (!active) return;
      setDetail(nextDetail);
      if (nextDetail.identityType === "personal") {
        await refreshPersonalNote();
        try { setKnowledgeCandidates(await loadKnowledgeCandidates(contestId, index)); }
        catch { setCandidateMessage("Knowledge suggestions are temporarily unavailable."); }
      }
      if (nextDetail.statement.state !== "ready") return;
      const assets = await getStatementAssets(contestId, index);
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
  }, [contestId, index, refreshPersonalNote]);
  useEffect(() => {
    if (detail?.identityType !== "personal") return;
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
  }, [detail?.identityType, refreshPersonalNote]);
  const createNote = async () => {
    if (creatingNote) return;
    setCreatingNote(true);
    setNoteMessage(null);
    try {
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
      await openPersonalNoteInObsidian(contestId, index);
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
      setRelocationCandidates(await getPersonalNoteRelocationCandidates(contestId, index));
    } catch {
      setRelocationMessage("Possible locations could not be listed. The existing binding and System Facts were not changed.");
    }
  };
  const confirmRelocationCandidate = async (vaultRelativePath: string) => {
    if (repairingPath) return;
    setRepairingPath(vaultRelativePath);
    setRelocationMessage(null);
    try {
      await rebindPersonalNote(contestId, index, vaultRelativePath);
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
      const lifecycle = await confirmPersonalNoteDeleted(contestId, index);
      setDetail((current) => current ? {
        ...current,
        identityType: "lightweight",
        personalNote: null,
        lifecycle,
      } : current);
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
      const lifecycle = await transitionProblemLifecycle(contestId, index, action);
      setDetail((current) => current ? { ...current, lifecycle } : current);
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
      const lifecycle = await deletePersonalNote(contestId, index);
      setDetail((current) => current ? {
        ...current,
        identityType: "lightweight",
        personalNote: null,
        lifecycle,
      } : current);
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
      const attempt = await startOrResumeReview(contestId, index);
      navigate(`/review/${attempt.attemptId}`);
    } catch {
      setReviewMessage("Review could not be started. The learning state and schedule were preserved.");
    } finally {
      setStartingReview(false);
    }
  };
  const updateCandidate = async (candidate: KnowledgeCandidateDto, disposition: KnowledgeCandidateDto["disposition"]) => {
    if (busyCandidate) return;
    setBusyCandidate(candidate.fingerprint);
    setCandidateMessage(null);
    try {
      const updated = await setKnowledgeCandidateDisposition(contestId, index, candidate.fingerprint, disposition);
      setKnowledgeCandidates((current) => current.map((item) => item.fingerprint === updated.fingerprint ? updated : item));
      setCandidateMessage(disposition === "ignored"
          ? "已忽略建议，没有修改 Markdown 或关系。"
          : "建议已退回待处理。" );
    } catch {
      setCandidateMessage("The suggestion state could not be changed.");
    } finally { setBusyCandidate(null); }
  };
  const acceptCandidate = async (candidate: KnowledgeCandidateDto) => {
    if (busyCandidate || !candidate.knowledgeNodeId) return;
    setBusyCandidate(candidate.fingerprint);
    setCandidateMessage(null);
    try {
      await acceptExistingKnowledgeCandidate(contestId, index, candidate.fingerprint, candidate.knowledgeNodeId);
      setCandidateMessage("Knowledge link was written to current Markdown, re-read, and verified as a formal relation.");
      try {
        setKnowledgeCandidates(await loadKnowledgeCandidates(contestId, index));
        await refreshPersonalNote();
      } catch {
        setKnowledgeCandidates((current) => current.filter((item) => item.fingerprint !== candidate.fingerprint));
      }
    } catch {
      setCandidateMessage("The current Markdown could not be safely patched. No formal relation was accepted.");
    } finally { setBusyCandidate(null); }
  };
  const acceptCandidateIntent = async (candidate: KnowledgeCandidateDto) => {
    if (busyCandidate || candidate.knowledgeNodeId || candidate.disposition !== "pending") return;
    setBusyCandidate(candidate.fingerprint);
    setCandidateMessage(null);
    try {
      const updated = await setKnowledgeCandidateDisposition(contestId, index, candidate.fingerprint, "acceptedIntent");
      setKnowledgeCandidates((current) => current.map((item) => item.fingerprint === updated.fingerprint ? { ...item, ...updated } : item));
      setCandidateMessage("仅保存意图，没有创建 Markdown、知识节点或正式关系。");
    } catch {
      setCandidateMessage("The intent could not be saved.");
    } finally { setBusyCandidate(null); }
  };
  if (failed) return <section className="empty-state" role="alert"><h1 ref={headingRef} tabIndex={-1}>Problem is unavailable</h1><p>The local problem detail could not be read. No import data was changed.</p></section>;
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
        {detail.lifecycle.nextReviewDueLocalDate ? (
          <p><strong>Next Review due:</strong> {detail.lifecycle.nextReviewDueLocalDate}</p>
        ) : null}
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
         <section className="empty-state" role="status"><h2>Vault is unavailable</h2><p>Live Markdown access is temporarily unavailable. The Personal Problem and its System Facts were preserved.</p></section>
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
      ) : (
        <section className="content-panel" aria-label="Personal Markdown projection">
           <h2>我的笔记</h2>
          {noteReadState.relocated ? <p className="safe-note">The note binding was restored to its current location.</p> : null}
           <h3>已识别章节</h3>
           {noteReadState.projection.knownSections.length ? <ul>{noteReadState.projection.knownSections.map((section, position) => <li key={`${section.name}-${position}`}>{section.name}</li>)}</ul> : <p>没有找到已识别章节。</p>}
           <h3>解题路线</h3>
           {noteReadState.projection.solutionRoutes.length ? <ol>{noteReadState.projection.solutionRoutes.map((route, position) => <li key={`${route.name}-${position}`}>{route.name}</li>)}</ol> : <p>没有找到解题路线。</p>}
          {noteReadState.projection.warnings.map((warning) => <p className="safe-note" key={`${warning.code}-${warning.name}`}>Duplicate section: {warning.name} ({warning.count})</p>)}
        </section>
      )
    ) : null}
    <ProblemReviewHistory contestId={contestId} index={index} learningStatus={detail.lifecycle.learningStatus} />
    {detail.statement.state === "pending" ? <section className="empty-state"><h2>Statement capture is pending</h2><p>Retry the contest import to capture this statement. Existing data remains unchanged.</p></section> : renderedHtml === null ? <section className="empty-state" aria-busy="true"><p>Preparing the local statement…</p></section> : <section className="content-panel statement-view"><div className="statement-heading-row"><h2>Statement snapshot</h2></div><div dangerouslySetInnerHTML={{ __html: renderedHtml }} /></section>}
  </>;
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

function ReviewHistoryEvidenceCard({ item }: { item: ReviewHistoryItemDto }) {
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

function ProblemReviewHistory({ contestId, index, learningStatus }: {
  contestId: number;
  index: string;
  learningStatus: LightweightProblemDetailDto["lifecycle"]["learningStatus"];
}) {
  const [history, setHistory] = useState<ReviewHistoryDto | null>(null);
  const [masteryDraft, setMasteryDraft] = useState<ProblemMasteryEvidenceDto>(emptyMasteryEvidence);
  const [loading, setLoading] = useState(false);
  const [savingMastery, setSavingMastery] = useState(false);
  const [error, setError] = useState(false);
  const load = () => {
    setLoading(true);
    setError(false);
    getReviewHistory(contestId, index)
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
    updateProblemMasteryEvidence(contestId, index, masteryDraft)
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
