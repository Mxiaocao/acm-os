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
  createPersonalNote,
  deletePersonalNote,
  getContestDetail,
  getContestShelf,
  getLightweightProblemDetail,
  getLightweightProblems,
  getPersonalNoteProjection,
  getStatementAssets,
  importCodeforcesContest,
  openPersonalNoteInObsidian,
  transitionProblemLifecycle,
  type ContestDetailDto,
  type ContestShelfItemDto,
  type LightweightProblemDetailDto,
  type LightweightProblemItemDto,
  type PersonalNoteReadStateDto,
  type ProblemLifecycleActionDto,
} from "../ipc/contest";
import { onPersonalNoteInvalidated } from "../ipc/personal-note-events";
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
  return (
    <main className="review-shell" ref={mainRef} tabIndex={-1}>
      <header className="review-header">
        <div>
          <p className="eyebrow">Review focus shell</p>
          <h1>Isolated review workspace</h1>
        </div>
        <button className="secondary-action" onClick={() => navigate("/today")} type="button">
          Return to Today
        </button>
      </header>
      <section aria-labelledby="review-boundary" className="review-stage">
        <h2 id="review-boundary">Review layout boundary</h2>
        <p>
          Route <code>/review/{attemptId}</code> is isolated from ordinary navigation. Review
          execution and knowledge reveal are intentionally unavailable until M4.
        </p>
      </section>
    </main>
  );
}

function NormalPageContent({ page, workspace, navigate }: { page: NormalPage; workspace: ConfiguredWorkspace; navigate: Navigate }) {
  const headingRef = useRouteFocus<HTMLHeadingElement>();
  if (page === "today") {
    return (
      <>
        <PageHeader eyebrow="Normal app shell" headingRef={headingRef} title="Today" />
        <section className="empty-state">
          <h2>Nothing planned yet</h2>
          <p>The workspace is ready. Today planning is introduced in a later milestone.</p>
        </section>
      </>
    );
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
      </>
    );
  }
  if (page === "contests") return <ContestShelf navigate={navigate} />;
  if (page === "problems") return <ProblemIndex navigate={navigate} />;
  const titles: Record<Exclude<NormalPage, "today" | "settings" | "contests" | "problems">, string> = {
    knowledge: "Knowledge",
  };
  return (
    <>
      <PageHeader eyebrow="Normal app shell" headingRef={headingRef} title={titles[page]} />
      <section className="empty-state">
        <h2>Surface boundary ready</h2>
        <p>This product surface is intentionally empty in M0.</p>
      </section>
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
    } catch {
      setNoteMessage("Personal Markdown was not deleted. The Personal Problem and its history were preserved.");
    } finally {
      setDeletingNote(false);
    }
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
          <p><strong>First cold-start due:</strong> {detail.lifecycle.nextReviewDueLocalDate}</p>
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
        </div>
        {lifecycleMessage ? <p aria-live="polite" className="system-caption">{lifecycleMessage}</p> : null}
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
    {detail.statement.state === "pending" ? <section className="empty-state"><h2>Statement capture is pending</h2><p>Retry the contest import to capture this statement. Existing data remains unchanged.</p></section> : renderedHtml === null ? <section className="empty-state" aria-busy="true"><p>Preparing the local statement…</p></section> : <section className="content-panel statement-view"><h2>Statement snapshot</h2><div dangerouslySetInnerHTML={{ __html: renderedHtml }} /></section>}
  </>;
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
