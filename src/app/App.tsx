import { useEffect, useState } from "react";
import { getAppShellStatus, type AppShellStatusDto } from "../ipc/app-shell";
import { getFoundationStatus, type FoundationStatus } from "../ipc/foundation";
import type { WorkspaceStatusDto } from "../ipc/workspace";
import { parseAppRoute, type NormalPage } from "./routing";
import { installChineseUiTranslation } from "./translation";
import {
  LoadingShell,
  NormalAppShell,
  RecoveryShell,
  ReviewFocusShell,
  SetupShell,
} from "./shells";

type ConfiguredWorkspace = Extract<WorkspaceStatusDto, { state: "configured" }>;

export function App() {
  const [shell, setShell] = useState<AppShellStatusDto | null>(null);
  const [foundation, setFoundation] = useState<FoundationStatus>({ state: "checking" });
  const [startupUnavailable, setStartupUnavailable] = useState(false);
  const [pathname, setPathname] = useState(window.location.pathname);

  useEffect(() => {
    return installChineseUiTranslation();
  }, []);

  useEffect(() => {
    let active = true;
    getFoundationStatus()
      .then((result) => {
        if (active) setFoundation({ state: "ready", foundation: result });
      })
      .catch(() => {
        if (active) setFoundation({ state: "unavailable" });
      });
    getAppShellStatus()
      .then((result) => {
        if (active) {
          setShell(result);
          if (result.state === "normal" && window.location.pathname === "/") {
            window.history.replaceState(null, "", "/today");
            setPathname("/today");
          }
        }
      })
      .catch(() => {
        if (active) setStartupUnavailable(true);
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const syncPathname = () => setPathname(window.location.pathname);
    window.addEventListener("popstate", syncPathname);
    return () => window.removeEventListener("popstate", syncPathname);
  }, []);

  useEffect(() => {
    document.title = `${routeAnnouncement(shell, pathname, startupUnavailable)} · ACM-OS`;
  }, [pathname, shell, startupUnavailable]);

  const navigate = (nextPathname: string, options?: { replace?: boolean }) => {
    if (nextPathname === window.location.pathname) return;
    if (options?.replace) {
      window.history.replaceState(null, "", nextPathname);
    } else {
      window.history.pushState(null, "", nextPathname);
    }
    setPathname(nextPathname);
  };

  if (startupUnavailable) {
    return (
      <RecoveryShell
        foundSchemaVersion={null}
        reason="startup_status_unavailable"
        supportedSchemaVersion={null}
      />
    );
  }
  if (!shell) return <LoadingShell />;
  if (shell.state === "recovery") {
    return (
      <RecoveryShell
        foundSchemaVersion={shell.foundSchemaVersion}
        reason={shell.recoveryReason}
        supportedSchemaVersion={shell.supportedSchemaVersion}
      />
    );
  }
  if (shell.state === "setup") {
    return (
      <SetupShell
        foundation={foundation}
        onConfigured={(workspace) => {
          enterNormalShell(workspace, setShell);
          navigate("/today", { replace: true });
        }}
      />
    );
  }

  const route = parseAppRoute(pathname);
  if (route.kind === "review") {
    return (
      <>
        <RouteAnnouncement message="Review focus" />
        <ReviewFocusShell attemptId={route.attemptId} navigate={navigate} />
      </>
    );
  }
  return (
    <>
      <RouteAnnouncement message={route.kind === "normal" ? normalPageLabel(route.page) : route.kind === "contestDetail" ? "Contest detail" : route.kind === "problemDetail" || route.kind === "canonicalProblemDetail" ? "Problem statement" : "Page not found"} />
      <NormalAppShell
        foundation={foundation}
        key={route.kind === "normal" ? route.page : route.kind === "contestDetail" ? `contest-${route.contestId}` : route.kind === "problemDetail" ? `${route.contestId}-${route.index}` : route.kind === "canonicalProblemDetail" ? `problem-id-${route.problemId}` : route.pathname}
        navigate={navigate}
        route={route}
        workspace={shell.workspace}
      />
    </>
  );
}

function RouteAnnouncement({ message }: { message: string }) {
  return <p aria-atomic="true" aria-live="polite" className="sr-only">{message}</p>;
}

function routeAnnouncement(
  shell: AppShellStatusDto | null,
  pathname: string,
  startupUnavailable: boolean,
): string {
  if (startupUnavailable || shell?.state === "recovery") return "Recovery";
  if (!shell) return "Checking system facts";
  if (shell.state === "setup") return "Connect your workspace";
  const route = parseAppRoute(pathname);
  if (route.kind === "review") return "Review focus";
  return route.kind === "normal" ? normalPageLabel(route.page) : route.kind === "contestDetail" ? "Contest detail" : route.kind === "problemDetail" || route.kind === "canonicalProblemDetail" ? "Problem statement" : "Page not found";
}

function normalPageLabel(page: NormalPage): string {
  const labels = {
    today: "Today",
    contests: "Contests",
    problems: "我的题库",
    knowledge: "Knowledge",
    reward: "Reward",
    settings: "Settings",
  };
  return labels[page];
}

function enterNormalShell(
  workspace: ConfiguredWorkspace,
  setShell: (status: AppShellStatusDto) => void,
) {
  setShell({
    state: "normal",
    recoveryReason: null,
    supportedSchemaVersion: null,
    foundSchemaVersion: null,
    workspace,
  });
}
