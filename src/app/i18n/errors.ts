import { t, type MessageKey, type MessageParams } from "./index";

export type UiErrorContext = "load" | "save" | "retry" | "validation" | "notFound" | "permission" | "unknown";

export type UiErrorPresentation = { key: MessageKey; params?: MessageParams; raw: unknown };

export function mapErrorToMessageKey(error: unknown, context: UiErrorContext = "unknown"): UiErrorPresentation {
  const code = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  const key: MessageKey = code.includes("not_found") || code.includes("not found")
    ? "errors.notFound"
    : code.includes("permission") || code.includes("unsupported") || code.includes("unsafe_external")
      ? "errors.permission"
      : context === "load" ? "errors.load"
        : context === "save" ? "errors.save"
          : context === "validation" ? "errors.validation"
            : context === "retry" ? "errors.retry" : "errors.unknown";
  return { key, raw: error };
}

export function getErrorPresentation(error: unknown, context: UiErrorContext = "unknown"): string {
  return t(mapErrorToMessageKey(error, context).key, mapErrorToMessageKey(error, context).params);
}
