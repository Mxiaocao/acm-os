import { type FormEvent, useEffect, useRef, useState } from "react";
import { getFoundationStatus, type FoundationStatusDto } from "../ipc/foundation";
import { getStartupStatus, type StartupStatusDto } from "../ipc/startup";
import {
  configureWorkspace,
  describeWorkspaceError,
  getWorkspaceStatus,
  parseWorkspaceConfigurationError,
  type WorkspaceConfigurationDraft,
  type WorkspaceConfigurationErrorDto,
  type WorkspacePathField,
  type WorkspaceStatusDto,
} from "../ipc/workspace";

const EMPTY_WORKSPACE_DRAFT: WorkspaceConfigurationDraft = {
  activeVaultPath: "",
  problemRootPath: "",
  knowledgeRootPath: "",
};

export function App() {
  const [foundation, setFoundation] = useState<FoundationStatusDto | null>(null);
  const [startup, setStartup] = useState<StartupStatusDto | null>(null);
  const [workspace, setWorkspace] = useState<WorkspaceStatusDto | null>(null);
  const [workspaceDraft, setWorkspaceDraft] = useState(EMPTY_WORKSPACE_DRAFT);
  const [savingWorkspace, setSavingWorkspace] = useState(false);
  const [foundationError, setFoundationError] = useState<string | null>(null);
  const [startupError, setStartupError] = useState<string | null>(null);
  const [workspaceError, setWorkspaceError] = useState<string | null>(null);
  const [workspaceIssue, setWorkspaceIssue] =
    useState<WorkspaceConfigurationErrorDto | null>(null);
  const savingWorkspaceRef = useRef(false);
  const activeVaultRef = useRef<HTMLInputElement>(null);
  const problemRootRef = useRef<HTMLInputElement>(null);
  const knowledgeRootRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let active = true;

    getFoundationStatus()
      .then((result) => {
        if (active) {
          setFoundation(result);
        }
      })
      .catch((cause: unknown) => {
        if (active) {
          setFoundationError(cause instanceof Error ? cause.message : String(cause));
        }
      });

    getStartupStatus()
      .then((result) => {
        if (active) {
          setStartup(result);
        }
      })
      .catch((cause: unknown) => {
        if (active) {
          setStartupError(cause instanceof Error ? cause.message : String(cause));
        }
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (startup?.state !== "ready") {
      return undefined;
    }

    let active = true;
    getWorkspaceStatus()
      .then((result) => {
        if (active) {
          setWorkspace(result);
        }
      })
      .catch((cause: unknown) => {
        if (active) {
          setWorkspaceError(describeWorkspaceError(cause));
        }
      });

    return () => {
      active = false;
    };
  }, [startup]);

  const updateWorkspaceDraft = (field: keyof WorkspaceConfigurationDraft, value: string) => {
    setWorkspaceDraft((current) => ({ ...current, [field]: value }));
  };

  const submitWorkspace = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (savingWorkspaceRef.current) {
      return;
    }
    savingWorkspaceRef.current = true;
    setSavingWorkspace(true);
    setWorkspaceError(null);
    setWorkspaceIssue(null);
    try {
      const configured = await configureWorkspace(workspaceDraft);
      setWorkspace(configured);
    } catch (cause: unknown) {
      const issue = parseWorkspaceConfigurationError(cause);
      setWorkspaceIssue(issue);
      setWorkspaceError(describeWorkspaceError(cause));
      focusWorkspaceField(issue?.field ?? null);
    } finally {
      savingWorkspaceRef.current = false;
      setSavingWorkspace(false);
    }
  };

  const focusWorkspaceField = (field: WorkspacePathField | null) => {
    const fields = {
      active_vault: activeVaultRef,
      problem_root: problemRootRef,
      knowledge_root: knowledgeRootRef,
    };
    if (field) {
      fields[field].current?.focus();
    }
  };

  const fieldHasError = (field: WorkspacePathField) => workspaceIssue?.field === field;

  return (
    <main className="app-shell">
      <p className="eyebrow">ACM-OS</p>
      <h1>BUILD foundation</h1>
      <p>This screen proves the repository, IPC, SQLite startup, and workspace boundaries.</p>
      <section aria-live="polite" className="status-card">
        {foundation ? (
          <p>
            Core status: <strong>{foundation.status}</strong> ({foundation.core})
          </p>
        ) : foundationError ? (
          <p>Core unavailable: {foundationError}</p>
        ) : (
          <p>Checking authoritative core...</p>
        )}
        {startup?.state === "ready" ? (
          <p>
            Database startup gate: <strong>ready</strong> (schema {startup.schemaVersion})
          </p>
        ) : startup?.state === "recoveryRequired" ? (
          <p>
            Database startup gate: <strong>recovery required</strong> ({startup.recoveryReason})
          </p>
        ) : startupError ? (
          <p>Database startup gate unavailable: {startupError}</p>
        ) : (
          <p>Checking database startup gate...</p>
        )}
      </section>
      {startup?.state === "ready" ? (
        <section aria-labelledby="workspace-heading" className="status-card workspace-card">
          <h2 id="workspace-heading">Workspace configuration</h2>
          {workspace?.state === "configured" ? (
            <dl className="workspace-summary">
              <dt>Active Vault</dt>
              <dd>{workspace.activeVaultPath}</dd>
              <dt>Problem Notes Root</dt>
              <dd>{workspace.problemRootPath}</dd>
              <dt>Knowledge Root</dt>
              <dd>{workspace.knowledgeRootPath}</dd>
            </dl>
          ) : workspace?.state === "unconfigured" ? (
            <form className="workspace-form" onSubmit={submitWorkspace}>
              <p>
                Connect one existing Vault and two existing, non-overlapping folders inside it.
              </p>
              <label>
                Active Vault
                <input
                  aria-describedby={fieldHasError("active_vault") ? "active-vault-error" : undefined}
                  aria-invalid={fieldHasError("active_vault")}
                  autoComplete="off"
                  onChange={(event) =>
                    updateWorkspaceDraft("activeVaultPath", event.currentTarget.value)
                  }
                  required
                  ref={activeVaultRef}
                  value={workspaceDraft.activeVaultPath}
                />
                {fieldHasError("active_vault") ? (
                  <span className="field-error" id="active-vault-error">
                    {workspaceError}
                  </span>
                ) : null}
              </label>
              <label>
                Problem Notes Root
                <input
                  aria-describedby={fieldHasError("problem_root") ? "problem-root-error" : undefined}
                  aria-invalid={fieldHasError("problem_root")}
                  autoComplete="off"
                  onChange={(event) =>
                    updateWorkspaceDraft("problemRootPath", event.currentTarget.value)
                  }
                  required
                  ref={problemRootRef}
                  value={workspaceDraft.problemRootPath}
                />
                {fieldHasError("problem_root") ? (
                  <span className="field-error" id="problem-root-error">
                    {workspaceError}
                  </span>
                ) : null}
              </label>
              <label>
                Knowledge Root
                <input
                  aria-describedby={fieldHasError("knowledge_root") ? "knowledge-root-error" : undefined}
                  aria-invalid={fieldHasError("knowledge_root")}
                  autoComplete="off"
                  onChange={(event) =>
                    updateWorkspaceDraft("knowledgeRootPath", event.currentTarget.value)
                  }
                  required
                  ref={knowledgeRootRef}
                  value={workspaceDraft.knowledgeRootPath}
                />
                {fieldHasError("knowledge_root") ? (
                  <span className="field-error" id="knowledge-root-error">
                    {workspaceError}
                  </span>
                ) : null}
              </label>
              <button disabled={savingWorkspace} type="submit">
                {savingWorkspace ? "Validating workspace..." : "Save workspace"}
              </button>
            </form>
          ) : workspaceError ? null : (
            <p>Checking workspace configuration...</p>
          )}
          {workspaceError && (!workspaceIssue || workspaceIssue.field === null) ? (
            <p aria-live="assertive" className="error-message">
              {workspaceError}
            </p>
          ) : null}
        </section>
      ) : null}
    </main>
  );
}
