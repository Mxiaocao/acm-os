export type NormalPage = "today" | "contests" | "problems" | "knowledge" | "settings";

export type AppRoute =
  | { kind: "normal"; page: NormalPage }
  | { kind: "contestDetail"; contestId: number }
  | { kind: "problemDetail"; contestId: number; index: string }
  | { kind: "canonicalProblemDetail"; problemId: string }
  | { kind: "review"; attemptId: string }
  | { kind: "notFound"; pathname: string };

const NORMAL_ROUTES: Readonly<Record<string, NormalPage>> = {
  "/today": "today",
  "/contests": "contests",
  "/problems": "problems",
  "/knowledge": "knowledge",
  "/settings": "settings",
};

export function parseAppRoute(pathname: string): AppRoute {
  const normalized = normalizePathname(pathname);
  if (normalized === "/") {
    return { kind: "normal", page: "today" };
  }
  const normalPage = NORMAL_ROUTES[normalized];
  if (normalPage) {
    return { kind: "normal", page: normalPage };
  }

  const contestMatch = /^\/contests\/([1-9][0-9]*)$/.exec(normalized);
  if (contestMatch) {
    return { kind: "contestDetail", contestId: Number(contestMatch[1]) };
  }

  const problemMatch = /^\/problems\/([1-9][0-9]*)\/([A-Z][0-9]?)$/.exec(normalized);
  if (problemMatch) {
    return { kind: "problemDetail", contestId: Number(problemMatch[1]), index: problemMatch[2] };
  }
  const canonicalProblemMatch = /^\/problems\/id\/([1-9][0-9]*)$/.exec(normalized);
  if (canonicalProblemMatch) {
    return { kind: "canonicalProblemDetail", problemId: canonicalProblemMatch[1] };
  }

  const reviewMatch = /^\/review\/([^/]+)$/.exec(normalized);
  if (reviewMatch) {
    try {
      const segment = reviewMatch[1];
      const attemptId = decodeURIComponent(segment);
      if (segment === attemptId && isUuidV7(attemptId)) {
        return { kind: "review", attemptId };
      }
    } catch {
      return { kind: "notFound", pathname: normalized };
    }
  }
  return { kind: "notFound", pathname: normalized };
}

function isUuidV7(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
}

function normalizePathname(pathname: string): string {
  if (!pathname.startsWith("/")) {
    return `/${pathname}`;
  }
  if (pathname.length > 1 && pathname.endsWith("/")) {
    return pathname.slice(0, -1);
  }
  return pathname;
}
