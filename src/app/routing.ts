export type NormalPage = "today" | "contests" | "problems" | "knowledge" | "settings";

export type AppRoute =
  | { kind: "normal"; page: NormalPage }
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
