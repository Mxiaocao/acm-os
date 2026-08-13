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
  acceptExistingKnowledgeCandidate,
  loadKnowledgeDetail,
  loadKnowledgeIndex,
  loadKnowledgeCandidates,
  loadKnowledgeReevaluationSuggestion,
  openKnowledgeInObsidian,
  setKnowledgeCandidateDisposition,
  type KnowledgeCandidateDto,
  type KnowledgeDetailDto,
  type KnowledgeNodeDto,
  type KnowledgeUnderstandingLevel,
} from "../ipc/knowledge";
import {
  createPersonalNote,
  completeReview,
  deletePersonalNote,
  getContestDetail,
  getContestShelf,
  getLightweightProblemDetail,
  getLightweightProblems,
  getPersonalNoteProjection,
  getReviewFocus,
  getReviewHelpDrawer,
  getReviewAttemptHistory,
  getReviewHistory,
  getStatementAssets,
  importCodeforcesContest,
  openPersonalNoteInObsidian,
  revealReviewHelp,
  startOrResumeReview,
  transitionProblemLifecycle,
  updateProblemMasteryEvidence,
  voidReview,
  type CompleteReviewInputDto,
  type CompletedReviewAttemptDto,
  type ContestDetailDto,
  type ContestShelfItemDto,
  type LightweightProblemDetailDto,
  type LightweightProblemItemDto,
  type PersonalNoteReadStateDto,
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
import type { AppRoute, NormalPage } from "./routing";

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
      <p className="eyebrow">Startup gate</p>
      <h1 ref={headingRef} tabIndex={-1}>Checking system facts</h1>
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
  return (
    <main className="gate-shell gate-shell--recovery">
      <Brand />
      <p className="eyebrow">Recovery shell</p>
      <h1 ref={headingRef} tabIndex={-1}>Normal startup is blocked</h1>
      <p>
        ACM-OS could not prove that System Facts are safe to use. Normal navigation stays hidden
        so the application cannot continue in a partially valid state.
      </p>
      <section aria-labelledby="recovery-detail" className="gate-panel" role="alert">
        <h2 id="recovery-detail">Diagnostic status</h2>
        <dl className="detail-list">
          <dt>Reason</dt>
          <dd>{reason}</dd>
          {supportedSchemaVersion !== null ? (
            <>
              <dt>Supported schema</dt>
              <dd>{supportedSchemaVersion}</dd>
            </>
          ) : null}
          {foundSchemaVersion !== null ? (
            <>
              <dt>Found schema</dt>
              <dd>{foundSchemaVersion}</dd>
            </>
          ) : null}
        </dl>
      </section>
      <p className="safe-note">No automatic repair or destructive action is performed in B0.4.</p>
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
          <p className="eyebrow">M4 · Review Focus</p>
          <h1>{focus ? `${focus.attempt.index}. ${focus.title}` : "Isolated review workspace"}</h1>
        </div>
        <button className="secondary-action" onClick={() => navigate("/today")} type="button">
          Return to Today
        </button>
      </header>
      {completedReview ? (
        <ReviewEvidenceCard completed={completedReview} />
      ) : terminalHistory ? (
        <ReviewHistoryEvidenceCard item={terminalHistory} />
      ) : failed ? (
        <section className="review-stage" role="alert">
          <h2>Review Attempt is unavailable</h2>
          <p>No Review result or learning state was changed.</p>
        </section>
      ) : !focus || renderedHtml === null ? (
        <section aria-busy="true" className="review-stage"><p>Loading isolated statement…</p></section>
      ) : (
        <>
          <section aria-labelledby="review-attempt-metadata" className="review-stage">
            <h2 id="review-attempt-metadata">Cold-start attempt</h2>
            <p>
              {reviewAttemptTypeLabel(focus.attempt.attemptType)} · scheduled {focus.attempt.scheduledDueLocalDate}
              {focus.attempt.startedEarly ? " · started early" : ""}
            </p>
            <a href={focus.sourceUrl} rel="noreferrer" target="_blank">Open original OJ</a>
            <button className="secondary-action" onClick={openHelpDrawer} ref={helpButtonRef} type="button">Open controlled help</button>
            <p className="safe-note">Old notes, hints, solutions, Contest history, and Review history are not loaded into this Focus view.</p>
          </section>
          <section className="review-stage statement-view" aria-labelledby="review-statement-heading">
            <h2 id="review-statement-heading">Statement snapshot</h2>
            <div dangerouslySetInnerHTML={{ __html: renderedHtml }} />
          </section>
          <form className="review-stage review-facts-form" onSubmit={submitCompletion}>
            <div>
              <p className="eyebrow">Facts, not a self-selected grade</p>
              <h2>Finish this Review</h2>
              <p>The system derives Mastered, Partial, or Not passed from these facts and recorded help.</p>
            </div>
            <fieldset>
              <legend>Submission facts</legend>
              <label><input checked={completion.finalAc} onChange={(event) => setCompletion({ ...completion, finalAc: event.target.checked })} type="checkbox" /> Final result was AC</label>
              <label>First submission result<select value={completion.firstSubmissionResult} onChange={(event) => { const result = event.target.value as SubmissionResultDto; setCompletion({ ...completion, firstSubmissionResult: result, firstSubmissionOther: result === "other" ? completion.firstSubmissionOther : null }); }}>{submissionResultOptions()}</select></label>
              {completion.firstSubmissionResult === "other" ? <label>First result detail<input maxLength={120} onChange={(event) => setCompletion({ ...completion, firstSubmissionOther: event.target.value })} required value={completion.firstSubmissionOther ?? ""} /></label> : null}
              <label>Final result<select value={completion.finalResult} onChange={(event) => { const result = event.target.value as SubmissionResultDto; setCompletion({ ...completion, finalResult: result, finalResultOther: result === "other" ? completion.finalResultOther : null }); }}>{submissionResultOptions()}</select></label>
              {completion.finalResult === "other" ? <label>Final result detail<input maxLength={120} onChange={(event) => setCompletion({ ...completion, finalResultOther: event.target.value })} required value={completion.finalResultOther ?? ""} /></label> : null}
              <label>Total submissions<input min="1" onChange={(event) => setCompletion({ ...completion, totalSubmissions: Number(event.target.value) })} required type="number" value={completion.totalSubmissions} /></label>
            </fieldset>
            <fieldset>
              <legend>Independence</legend>
              <label><input checked={completion.ideaIndependent} onChange={(event) => setCompletion({ ...completion, ideaIndependent: event.target.checked })} type="checkbox" /> Idea was independent</label>
              <label><input checked={completion.implementationIndependent} onChange={(event) => setCompletion({ ...completion, implementationIndependent: event.target.checked })} type="checkbox" /> Implementation was independent</label>
              <label>Debug<select value={completion.debugIndependence} onChange={(event) => setCompletion({ ...completion, debugIndependence: event.target.value as CompleteReviewInputDto["debugIndependence"] })}><option value="notNeeded">No debug needed</option><option value="independent">Debugged independently</option><option value="usedSolvingHelp">Used problem-solving help to debug</option></select></label>
              <label>Unrecorded external help<select value={completion.externalHelp} onChange={(event) => setCompletion({ ...completion, externalHelp: event.target.value as CompleteReviewInputDto["externalHelp"] })}><option value="none">None</option><option value="solvingHint">Problem-solving hint</option><option value="fullSolution">Full solution</option></select></label>
            </fieldset>
            <fieldset>
              <legend>Failure reasons</legend>
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
        <aside aria-labelledby="review-help-title" aria-modal="true" className="review-help-drawer" ref={helpDrawerRef} role="dialog">
          <div className="review-help-drawer__header">
            <div>
              <p className="eyebrow">Evidence before reveal</p>
              <h2 id="review-help-title" ref={helpHeadingRef} tabIndex={-1}>Controlled help</h2>
            </div>
            <button className="secondary-action" onClick={closeHelpDrawer} type="button">Close</button>
          </div>
          <p>Opening this drawer records nothing. A successful Reveal creates an irreversible usage event before content appears.</p>
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
          <h2 id="workspace-settings">Workspace</h2>
          <dl className="detail-list detail-list--paths">
            <dt>Active Vault</dt><dd>{workspace.activeVaultPath}</dd>
            <dt>Problem Notes Root</dt><dd>{workspace.problemRootPath}</dd>
            <dt>Knowledge Root</dt><dd>{workspace.knowledgeRootPath}</dd>
          </dl>
          <p className="safe-note">Changing the Active Vault requires a future preview-and-confirm flow.</p>
        </section>
        <WeeklyAcmBudgetSettings />
      </>
    );
  }
  if (page === "contests") return <ContestShelf navigate={navigate} />;
  if (page === "problems") return <ProblemIndex navigate={navigate} />;
  return <KnowledgePage navigate={navigate} />;
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
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [detail, setDetail] = useState<KnowledgeDetailDto | null>(null);
  const [selectedLevel, setSelectedLevel] = useState<KnowledgeUnderstandingLevel>("notLearned");
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [reevaluation, setReevaluation] = useState<{ shouldSuggest: boolean; qualifyingProblemCount: number } | null>(null);

  const refresh = useCallback(async (nextQuery = query) => {
    setLoading(true);
    setError(null);
    try {
      const index = await loadKnowledgeIndex(nextQuery);
      setNodes(index.nodes);
      setAnomalies(index.locationAnomalies);
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

  return (
    <>
      <PageHeader eyebrow="Markdown authority" headingRef={headingRef} title="Knowledge" />
      <section aria-labelledby="knowledge-index-title" className="content-panel knowledge-index">
        <div className="knowledge-toolbar">
          <div><h2 id="knowledge-index-title">Knowledge index</h2><p>Only Markdown files currently found in Knowledge Root appear here.</p></div>
          <button className="secondary-action" disabled={loading} onClick={() => void refresh(query)} type="button">{loading ? "Re-indexing..." : "Re-index"}</button>
        </div>
        <form className="knowledge-search" onSubmit={(event) => { event.preventDefault(); void refresh(query); }}>
          <label htmlFor="knowledge-search">Search name or path</label>
          <div><input id="knowledge-search" onChange={(event) => setQuery(event.currentTarget.value)} value={query} /><button type="submit">Search</button></div>
        </form>
        {error ? <p aria-live="polite" className="error-copy">{error}</p> : null}
        {!loading && !error && nodes.length === 0 ? <p className="safe-note">No matching Markdown files were discovered.</p> : null}
        <ul className="knowledge-node-list">
          {nodes.map((node) => <li key={node.knowledgeNodeId}><button className="list-link" onClick={() => void openDetail(node)} type="button"><strong>{node.displayName}</strong><span>{node.vaultRelativePath}</span></button></li>)}
        </ul>
        {anomalies.length > 0 ? <p className="error-copy">{anomalies.length} bound Knowledge node(s) have a location anomaly and require recovery.</p> : null}
      </section>
      {detail ? (
        <section aria-labelledby="knowledge-detail-title" className="content-panel knowledge-detail">
          <div className="knowledge-toolbar"><div><p className="eyebrow">Fresh Markdown detail</p><h2 id="knowledge-detail-title">{detail.node.displayName}</h2><p><code>{detail.node.vaultRelativePath}</code></p></div><button className="secondary-action" onClick={() => void openKnowledgeInObsidian(detail.node.knowledgeNodeId).catch(() => setMessage("Obsidian could not open this file."))} type="button">Open in Obsidian</button></div>
          <div className="knowledge-understanding">
            <label>Current understanding<select aria-label="Current understanding" onChange={(event) => setSelectedLevel(event.currentTarget.value as KnowledgeUnderstandingLevel)} value={selectedLevel}>{knowledgeLevels.map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label>
            <button disabled={saving} onClick={() => void confirmUnderstanding()} type="button">{saving ? "Saving..." : "Confirm status"}</button>
            {detail.understanding ? <p>Historical highest: <strong>{knowledgeLevelLabel(detail.understanding.historicalHighest)}</strong> · first reached {detail.understanding.firstReachedHighestOn}</p> : <p>No user-confirmed status yet.</p>}
            {reevaluation?.shouldSuggest ? <p aria-live="polite" className="safe-note">Consider re-evaluating this Knowledge status: {reevaluation.qualifyingProblemCount} distinct related Problems gained new 真会 Review Evidence. Your current status was not changed.</p> : null}
            {message ? <p aria-live="polite" className="safe-note">{message}</p> : null}
          </div>
          <KnowledgeNeighborList heading="Outgoing knowledge" nodes={detail.outgoing} onOpen={openDetail} />
          <KnowledgeNeighborList heading="Incoming knowledge" nodes={detail.incoming} onOpen={openDetail} />
          <div><h3>Related problems</h3>{detail.relatedProblems.length === 0 ? <p>None.</p> : <ul>{detail.relatedProblems.map((problem) => <li key={problem.problemId}><button className="list-link" onClick={() => navigate(`/problems/${problem.contestId}/${problem.problemIndex}`)} type="button"><strong>{problem.contestId}{problem.problemIndex} · {problem.title}</strong></button></li>)}</ul>}</div>
        </section>
      ) : null}
    </>
  );
}

function KnowledgeNeighborList({ heading, nodes, onOpen }: { heading: string; nodes: KnowledgeNodeDto[]; onOpen: (node: KnowledgeNodeDto) => Promise<void> }) {
  return <div><h3>{heading}</h3>{nodes.length === 0 ? <p>None.</p> : <ul>{nodes.map((node) => <li key={node.knowledgeNodeId}><button className="list-link" onClick={() => void onOpen(node)} type="button"><strong>{node.displayName}</strong><span>{node.vaultRelativePath}</span></button></li>)}</ul>}</div>;
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
  const [busyEntry, setBusyEntry] = useState<string | null>(null);
  const [replan, setReplan] = useState<TodayReplanPreviewDto | null>(null);
  const [suggestions, setSuggestions] = useState<TodayExtraSuggestionsPreviewDto | null>(null);
  const draggedEntryIdRef = useRef<string | null>(null);
  const pointerTargetEntryIdRef = useRef<string | null>(null);

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

  const commitSnapshot = async (next: TodaySnapshotDto) => {
    setSnapshot(next); setBudgetDraft(String(next.budgetMinutes)); setReplan(null); setError(null);
    await refreshSuggestions(next);
  };

  const move = async (index: number, offset: -1 | 1) => {
    if (!snapshot) return;
    const target = index + offset;
    if (target < 0 || target >= snapshot.entries.length) return;
    const ids = snapshot.entries.map((entry) => entry.entryId);
    [ids[index], ids[target]] = [ids[target], ids[index]];
    try { await commitSnapshot(await reorderToday(snapshot.planId, ids)); }
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
    try { await commitSnapshot(await reorderToday(snapshot.planId, ids)); }
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
    try { await commitSnapshot(await completeTodayEntry(snapshot.planId, entry.entryId)); }
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
    try { await commitSnapshot(await applyTodayReplan(replan)); }
    catch (cause) { setError(todayErrorMessage(cause)); }
  };

  const acceptSuggestion = async (problemId: string) => {
    if (!suggestions) return;
    setBusyEntry(problemId);
    try { await commitSnapshot(await acceptTodayExtraSuggestion(suggestions, problemId)); }
    catch (cause) { setError(todayErrorMessage(cause)); }
    finally { setBusyEntry(null); }
  };

  return <>
    <PageHeader eyebrow="Daily execution" headingRef={headingRef} title="Today" />
    {loading ? <p aria-live="polite">Loading today plan...</p> : null}
    {error ? <p aria-live="assertive" className="error-message">{error}</p> : null}
    {!loading && needsBudget ? <section className="empty-state"><h2>Set today&apos;s budget</h2><p>No weekly default is set for this weekday. Enter any non-negative whole number of minutes; tasks still use complete 30 or 60 minute planning blocks.</p><form className="today-budget-start" noValidate onSubmit={(event) => { event.preventDefault(); const value = Number(initialBudgetDraft); if (initialBudgetDraft.trim() === "" || !Number.isInteger(value) || value < 0) { setError("Daily budget must be a non-negative whole number of minutes."); return; } void load(true); }}><label>Minutes<input min="0" onInput={(event) => setInitialBudgetDraft(event.currentTarget.value)} required type="number" value={initialBudgetDraft} /></label><button className="primary-action" type="submit">Create Today plan</button></form></section> : null}
    {!loading && !snapshot && !needsBudget ? <section className="empty-state"><h2>Plan unavailable</h2><button className="secondary-action" onClick={() => void load(false)} type="button">Retry</button></section> : null}
    {snapshot ? <>
      <section className="today-toolbar" aria-label="Today plan summary">
        <dl><div><dt>Date</dt><dd>{snapshot.localDate}</dd></div><div><dt>Planned</dt><dd>{snapshot.plannedMinutes} min</dd></div><div><dt>Budget</dt><dd>{snapshot.budgetMinutes} min</dd></div><div><dt>Over</dt><dd>{snapshot.overBudgetMinutes} min</dd></div></dl>
        <form noValidate onSubmit={previewBudget}><label>Today override<input aria-label="Daily budget in minutes" min="0" onInput={(event) => setBudgetDraft(event.currentTarget.value)} required type="number" value={budgetDraft} /></label><button className="secondary-action" type="submit">Preview replan</button></form>
      </section>
      {snapshot.entries.length === 0 ? <section className="empty-state"><h2>No tasks fit this budget</h2><p>Only complete 30 or 60 minute tasks are scheduled.</p></section> :
        <ol className="today-list">{snapshot.entries.map((entry, index) => <li className={`today-entry today-entry--${entry.status}`} data-entry-id={entry.entryId} key={entry.entryId} onKeyDown={(event) => { if (event.altKey && event.key === "ArrowUp") { event.preventDefault(); void move(index, -1); } if (event.altKey && event.key === "ArrowDown") { event.preventDefault(); void move(index, 1); } }} tabIndex={0}>
          <div className="today-entry__order"><button aria-label={`Drag ${todayReasonLabel(entry.reason)} to reorder`} className="today-drag-handle" onPointerCancel={clearPointerDrag} onPointerDown={(event) => startPointerDrag(event, entry.entryId)} onPointerMove={movePointerDrag} onPointerUp={finishPointerDrag} title="Drag to reorder" type="button">⋮⋮</button><button aria-label={`Move ${todayReasonLabel(entry.reason)} up`} disabled={index === 0} onClick={() => void move(index, -1)} type="button">↑</button><button aria-label={`Move ${todayReasonLabel(entry.reason)} down`} disabled={index === snapshot.entries.length - 1} onClick={() => void move(index, 1)} type="button">↓</button></div>
          <div className="today-entry__body"><div><span className="today-lane">{todayLaneLabel(entry.lane)}</span><span className={`today-status today-status--${entry.status}`}>{todayStatusLabel(entry.status)}</span>{entry.origin === "manual" ? <span className="today-origin">Manual</span> : null}</div><button className="today-problem-link" onClick={() => navigate(entry.reviewAttemptId && entry.status === "inProgress" ? `/review/${entry.reviewAttemptId}` : `/problems/${entry.contestId}/${entry.problemIndex}`)} type="button"><strong>{entry.problemTitle}</strong><span>CF {entry.contestId}{entry.problemIndex} · {todayReasonLabel(entry.reason)} · {entry.planningCostMinutes} min</span></button></div>
          {todayDoneAllowed(entry) && entry.status !== "completed" ? <button className="primary-action" disabled={busyEntry === entry.entryId || entry.status === "unavailable"} onClick={() => void done(entry)} type="button">Done for today</button> : null}
        </li>)}</ol>}
      {suggestions && suggestions.suggestions.length > 0 ? <section className="today-suggestions"><h2>Extra suggestions</h2><p>{suggestions.remainingBudgetMinutes} minutes remain. Nothing is added without your action.</p><ul>{suggestions.suggestions.map((item) => <li key={item.problemId}><span><strong>{item.problemTitle}</strong><small>CF {item.contestId}{item.problemIndex} · {todayReasonLabel(item.reason)} · {item.planningCostMinutes} min</small></span><button className="secondary-action" disabled={busyEntry === item.problemId} onClick={() => void acceptSuggestion(item.problemId)} type="button">Add to Today</button></li>)}</ul></section> : null}
    </> : null}
    {replan ? <div className="modal-backdrop"><div aria-labelledby="today-replan-title" aria-modal="true" role="dialog"><h2 id="today-replan-title">Apply this replan?</h2><p>Budget {replan.expectedSnapshot.budgetMinutes} → {replan.proposedBudgetMinutes} minutes. This is a one-day override; the weekly default and next week&apos;s same weekday remain unchanged. Planned work becomes {replan.proposedPlannedMinutes} minutes across {replan.entries.length} entries. Completed, in-progress, and manual entries stay protected.</p><div className="button-row"><button className="primary-action" onClick={() => void applyBudget()} type="button">Apply replan</button><button className="secondary-action" onClick={() => { setBudgetDraft(String(snapshot?.budgetMinutes ?? Number(initialBudgetDraft))); setReplan(null); }} type="button">Cancel</button></div></div></div> : null}
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
  return (
    <>
      <PageHeader eyebrow="M1 · 比赛导入" headingRef={headingRef} title="比赛" />
      <form className="content-panel" onSubmit={submitImport}>
        <label>Codeforces 公开比赛网址
          <input autoComplete="off" disabled={importing} onInput={(event) => { setContestUrl(event.currentTarget.value); setImportMessage(null); }} placeholder="https://codeforces.com/contest/1979" required value={contestUrl} />
        </label>
        <button className="primary-action" disabled={importing} type="submit">{importing ? "导入中…" : "导入比赛"}</button>
        {importMessage ? <p aria-live="polite" className="system-caption">{importMessage}</p> : null}
      </form>
      {failed ? <section className="empty-state" role="alert"><h2>比赛数据暂不可用</h2><p>无法读取本地系统事实，任何导入状态都没有改变。</p></section> : null}
      {items?.length === 0 ? <section className="empty-state"><h2>尚未导入比赛</h2><p>请输入完整的 Codeforces 比赛网址，例如 https://codeforces.com/contest/1979。</p></section> : null}
      {items?.length ? <section className="content-panel" aria-label="已导入比赛"><ul className="detail-list">{items.map((item) => <li key={item.contestId}><button className="list-link" onClick={() => navigate(`/contests/${item.contestId}`)} type="button"><strong>{item.title}</strong><span>Codeforces {item.contestId} · {item.problemCount} 道题 · {item.importStatus === "complete" ? "导入完整" : `${item.missingSnapshotCount} 道题面缺失`}</span></button>{item.importStatus === "incomplete" ? <button className="secondary-action" disabled={importing} onClick={() => retryMissing(item.contestId)} type="button">重试缺失题面</button> : null}</li>)}</ul></section> : null}
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
      {items?.length ? <section className="content-panel" aria-label="Lightweight problems"><ul className="detail-list">{items.map((item) => <li key={`${item.contestId}-${item.index}`}><button className="list-link" onClick={() => navigate(`/problems/${item.contestId}/${item.index}`)} type="button"><strong>{item.index}. {item.title}</strong><span>Codeforces {item.contestId}{item.rating ? ` · ${item.rating}` : ""} · {item.hasStatementSnapshot ? "statement captured" : "statement pending"}</span></button></li>)}</ul></section> : null}
      {items === null && !failed ? <section className="empty-state" aria-busy="true"><p>Loading local problems…</p></section> : null}
    </>
  );
}

function ContestDetail({ contestId, navigate }: { contestId: number; navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  const [detail, setDetail] = useState<ContestDetailDto | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => { getContestDetail(contestId).then(setDetail).catch(() => setFailed(true)); }, [contestId]);
  if (failed) return <section className="empty-state" role="alert"><h1 ref={headingRef} tabIndex={-1}>Contest is unavailable</h1><p>The local contest detail could not be read.</p></section>;
  if (!detail) return <section className="empty-state" aria-busy="true"><h1 ref={headingRef} tabIndex={-1}>Loading contest</h1></section>;
  return <>
    <PageHeader eyebrow="M1 · Contest detail" headingRef={headingRef} title={detail.title} />
    <section className="content-panel"><p>Codeforces {detail.contestId} · {detail.importStatus}</p><a href={detail.sourceUrl} rel="noreferrer" target="_blank">Open original contest</a></section>
    <section className="content-panel" aria-label="Contest problems"><h2>Problems</h2><ul className="detail-list">{detail.problems.map((problem) => <li key={problem.index}><button className="list-link" onClick={() => navigate(`/problems/${problem.contestId}/${problem.index}`)} type="button"><strong>{problem.index}. {problem.title}</strong><span>{problem.rating ? `Rating ${problem.rating} · ` : ""}{problem.hasStatementSnapshot ? "statement captured" : "statement pending"}</span></button></li>)}</ul></section>
  </>;
}

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
          ? "Suggestion ignored. No Markdown or relation was changed."
          : "Suggestion returned to pending.");
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
      setCandidateMessage("Intent saved only. No Markdown, Knowledge Node, or formal relation was created.");
    } catch {
      setCandidateMessage("The intent could not be saved.");
    } finally { setBusyCandidate(null); }
  };
  if (failed) return <section className="empty-state" role="alert"><h1 ref={headingRef} tabIndex={-1}>Problem is unavailable</h1><p>The local problem detail could not be read. No import data was changed.</p></section>;
  if (!detail) return <section className="empty-state" aria-busy="true"><h1 ref={headingRef} tabIndex={-1}>Loading problem</h1><p>Reading the local statement snapshot...</p></section>;
  return <>
    <PageHeader eyebrow="M1 local statement snapshot" headingRef={headingRef} title={detail.index + ". " + detail.title} />
    <section className="content-panel">
      <p>
        Codeforces {detail.contestId}{detail.rating ? " · Rating " + detail.rating : ""}
        {" · "}{detail.identityType === "personal" ? "Personal Problem" : "Lightweight Problem"}
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
        <p>Suggestions are not Knowledge Nodes or formal relations. Accepting requires target resolution and the separate Safe Patch flow.</p>
        {knowledgeCandidates.length === 0 ? <p className="safe-note">No suggestions for this Personal Problem.</p> : <ul>{knowledgeCandidates.map((candidate) => <li key={candidate.fingerprint}><div><strong>{candidate.targetRef}</strong><span>{candidate.disposition === "acceptedIntent" ? candidate.knowledgeNodeId ? "Accepted intent · existing Knowledge Markdown now found" : "Accepted intent" : candidate.disposition === "ignored" ? "Ignored" : candidate.knowledgeNodeId ? "Pending · existing Knowledge Markdown found" : "Pending · no unique Knowledge Node"}</span></div><div className="action-row">{candidate.disposition !== "ignored" && candidate.knowledgeNodeId ? <button disabled={busyCandidate !== null} onClick={() => void acceptCandidate(candidate)} type="button">Accept existing Knowledge</button> : null}{candidate.disposition === "pending" && !candidate.knowledgeNodeId ? <button disabled={busyCandidate !== null} onClick={() => void acceptCandidateIntent(candidate)} type="button">Save intent only</button> : null}{candidate.disposition !== "ignored" ? <button className="secondary-action" disabled={busyCandidate !== null} onClick={() => void updateCandidate(candidate, "ignored")} type="button">Do not suggest</button> : null}{candidate.disposition !== "pending" ? <button className="secondary-action" disabled={busyCandidate !== null} onClick={() => void updateCandidate(candidate, "pending")} type="button">Return to pending</button> : null}</div></li>)}</ul>}
        {candidateMessage ? <p aria-live="polite" className="safe-note">{candidateMessage}</p> : null}
      </section>
    ) : null}
    {detail.identityType === "personal" ? (
      <section className="content-panel" aria-labelledby="personal-note-danger-heading">
        <h2 id="personal-note-danger-heading">Personal note actions</h2>
        {!showDeletePreview ? (
          <button className="secondary-action" onClick={() => setShowDeletePreview(true)} type="button">
            Delete my personal note…
          </button>
        ) : (
          <div role="alertdialog" aria-labelledby="delete-note-preview-title" aria-describedby="delete-note-preview-description">
            <h3 id="delete-note-preview-title">Delete this Personal Markdown?</h3>
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
        <section className="empty-state" role="alert"><h2>Personal Markdown is unavailable</h2><p>The current bound file could not be read. System Facts were preserved.</p></section>
      ) : noteReadState === null ? (
        <section className="empty-state" aria-busy="true"><p>Reading current Personal Markdown...</p></section>
      ) : noteReadState.state === "vaultUnavailable" ? (
        <section className="empty-state" role="status"><h2>Vault is unavailable</h2><p>Live Markdown access is temporarily unavailable. The Personal Problem and its System Facts were preserved.</p></section>
      ) : noteReadState.state === "locationAnomaly" ? (
        <section className="empty-state" role="status"><h2>Note location needs attention</h2><p>The original path is missing and no unique relocation was found. The Personal Problem was not deleted or downgraded.</p></section>
      ) : (
        <section className="content-panel" aria-label="Personal Markdown projection">
          <h2>My note</h2>
          {noteReadState.relocated ? <p className="safe-note">The note binding was restored to its current location.</p> : null}
          <h3>Known sections</h3>
          {noteReadState.projection.knownSections.length ? <ul>{noteReadState.projection.knownSections.map((section, position) => <li key={`${section.name}-${position}`}>{section.name}</li>)}</ul> : <p>No known sections found.</p>}
          <h3>Solution routes</h3>
          {noteReadState.projection.solutionRoutes.length ? <ol>{noteReadState.projection.solutionRoutes.map((route, position) => <li key={`${route.name}-${position}`}>{route.name}</li>)}</ol> : <p>No solution routes found.</p>}
          {noteReadState.projection.warnings.map((warning) => <p className="safe-note" key={`${warning.code}-${warning.name}`}>Duplicate section: {warning.name} ({warning.count})</p>)}
        </section>
      )
    ) : null}
    <ProblemReviewHistory contestId={contestId} index={index} learningStatus={detail.lifecycle.learningStatus} />
    {detail.statement.state === "pending" ? <section className="empty-state"><h2>Statement capture is pending</h2><p>Retry the contest import to capture this statement. Existing data remains unchanged.</p></section> : renderedHtml === null ? <section className="empty-state" aria-busy="true"><p>Preparing the local statement…</p></section> : <section className="content-panel statement-view"><h2>Statement snapshot</h2><div dangerouslySetInnerHTML={{ __html: renderedHtml }} /></section>}
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
      <p className="eyebrow">Completed Review · Evidence Card</p>
      <h2 id="review-evidence-title">{reviewJudgementLabel(completed.judgement)}</h2>
      <p>Completed {completed.completedLocalDate}. This result was derived by judgement rule v{completed.attempt.judgementRuleVersion}; it was not selected directly.</p>
      <h3>Why</h3>
      <ul>{completed.evidenceCodes.map((code) => <li key={code}>{reviewEvidenceLabel(code)}</li>)}</ul>
      {completed.failureReasons.length ? <><h3>Failure reasons</h3><ul>{completed.failureReasons.map((reason) => <li key={reason.code}>{reviewFailureReasonLabel(reason)}</li>)}</ul></> : null}
      <h3>Next state</h3>
      <p>{learningStatusLabel(completed.lifecycle.learningStatus)}{completed.lifecycle.nextReviewDueLocalDate ? ` · due ${completed.lifecycle.nextReviewDueLocalDate}` : ""}</p>
    </section>
  );
}

function ReviewHistoryEvidenceCard({ item }: { item: ReviewHistoryItemDto }) {
  if (item.status === "void") {
    return <section className="review-stage review-evidence-card"><p className="eyebrow">Review history</p><h2>Voided mistaken Attempt</h2><p>{item.voidReason}</p><p>Scheduling was unchanged. Revealed help history remains recorded.</p></section>;
  }
  return <section className="review-stage review-evidence-card"><p className="eyebrow">Completed Review · Evidence Card</p><h2>{item.judgement ? reviewJudgementLabel(item.judgement) : "Completed"}</h2><p>Completed {item.completedLocalDate}.</p><ul>{item.evidenceCodes.map((code) => <li key={code}>{reviewEvidenceLabel(code)}</li>)}</ul></section>;
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
  ["recallsProblem", "I can recall what the Problem asks me to solve"],
  ["multipleSolutionsClear", "Multiple solution routes are clear"],
  ["knowledgeUnderstood", "The related knowledge is genuinely understood"],
  ["implementationFluent", "I can implement it quickly and clearly"],
  ["canAdaptOrCreate", "I understand the setting and can adapt or create a related Problem"],
  ["transferSolvedIndependently", "I independently solved a related transfer Problem"],
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
      <h2 id="review-history-heading">Review history</h2>
      {!history ? <button className="secondary-action" disabled={loading} onClick={load} type="button">{loading ? "Loading…" : "Load Review history"}</button> : null}
      {error ? <p role="alert">Review history is temporarily unavailable; no history was changed.</p> : null}
      {history ? <>
        <p><strong>Historical best Review evidence:</strong> {history.historicalBestReview ? reviewJudgementLabel(history.historicalBestReview) : "None yet"}</p>
        <section className="mastery-evidence" aria-labelledby="mastery-evidence-heading">
          <h3 id="mastery-evidence-heading">Thorough digestion evidence</h3>
          <p><strong>Current:</strong> {achievedCount}/6 evidence criteria · {learningStatusLabel(learningStatus)}</p>
          <p><strong>Historical highest:</strong> {mastery?.historicalThoroughlyDigested ? "Thoroughly digested" : "Not yet thoroughly digested"}{mastery?.firstThoroughlyDigestedLocalDate ? ` · first reached ${mastery.firstThoroughlyDigestedLocalDate}` : ""}</p>
          <p className="safe-note">Only 6/6 is “Thoroughly digested”. Review Mastered does not automatically change these user-confirmed facts.</p>
          <fieldset>
            <legend>Current evidence</legend>
            {masteryEvidenceLabels.map(([key, label]) => <label key={key}><input checked={masteryDraft[key]} onChange={(event) => setMasteryDraft({ ...masteryDraft, [key]: event.target.checked })} type="checkbox" /> {label}</label>)}
          </fieldset>
          <button className="secondary-action" disabled={savingMastery} onClick={saveMastery} type="button">{savingMastery ? "Saving…" : "Save current evidence"}</button>
        </section>
        {history.attempts.length === 0 ? <p>No Review Attempts yet.</p> : <ol className="review-history-list">{history.attempts.map((item) => <li key={item.attempt.attemptId}><strong>{item.status === "void" ? "Void" : item.status === "inProgress" ? "In progress" : item.judgement ? reviewJudgementLabel(item.judgement) : "Completed"}</strong><span>{item.attempt.attemptType} · started {item.attempt.startedAtUtc}</span>{item.completionFacts ? <span>Final AC: {item.completionFacts.finalAc ? "yes" : "no"} · submissions: {item.completionFacts.totalSubmissions} · idea independent: {item.completionFacts.ideaIndependent ? "yes" : "no"} · implementation independent: {item.completionFacts.implementationIndependent ? "yes" : "no"}</span> : null}{item.helpLevels.length ? <span>Help levels: {item.helpLevels.join(", ")}</span> : null}{item.failureReasons.length ? <span>Reasons: {item.failureReasons.map(reviewFailureReasonLabel).join("; ")}</span> : null}</li>)}</ol>}
      </> : null}
    </section>
  );
}

function reviewJudgementLabel(judgement: CompletedReviewAttemptDto["judgement"]): string {
  return judgement === "mastered" ? "Mastered" : judgement === "partial" ? "Partial" : "Not passed";
}

function reviewFailureReasonLabel(reason: { code: ReviewFailureReasonCodeDto; otherText: string | null }): string {
  return reason.code === "other"
    ? `Other: ${reason.otherText ?? ""}`
    : reviewFailureReasonOptions.find(([code]) => code === reason.code)?.[1] ?? reason.code;
}

function reviewEvidenceLabel(code: string): string {
  const labels: Record<string, string> = {
    final_ac: "Final submission accepted",
    no_final_ac: "No final accepted submission",
    controlled_help_l1: "Prerequisite names revealed",
    controlled_help_l2: "Hint revealed",
    controlled_help_l3: "Prerequisite content revealed",
    controlled_help_l4: "Old idea or code revealed",
    controlled_help_l5: "Full solution revealed",
    external_solving_hint: "External problem-solving hint reported",
    external_full_solution: "External full solution reported",
    idea_not_independent: "Idea was not independent",
    implementation_not_independent: "Implementation was not independent",
    debug_not_needed: "No debugging was needed",
    debug_independent: "Debugging was independent",
    debug_solving_help: "Problem-solving help was used while debugging",
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
    firstColdStart: "First cold-start Review",
    longTermReview: "Long-term Review",
    earlyCheck: "Early check",
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
      const href = element.getAttribute("href") ?? "";
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
      <p className="eyebrow">Unknown route</p>
      <h1 ref={headingRef} tabIndex={-1}>Page not found</h1>
      <p><code>{pathname}</code> is not part of the current application map.</p>
      <button className="primary-action" onClick={() => navigate("/today")} type="button">Go to Today</button>
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
    case "checking": return "checking";
    case "unavailable": return "unavailable";
    case "ready": return `ready · ${foundation.foundation.core}`;
  }
}
