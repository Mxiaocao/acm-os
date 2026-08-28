import type { ReactNode } from "react";
import { t } from "../i18n";

export function LoadingState({ message = t("shell.loading") }: { message?: string }) {
  return <p aria-busy="true" aria-live="polite">{message}</p>;
}

export function EmptyState({ title, message, action }: { title?: string; message?: string; action?: ReactNode }) {
  return <section className="empty-state">{title ? <h2>{title}</h2> : null}{message ? <p>{message}</p> : null}{action}</section>;
}

export function ErrorState({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return <section className="empty-state" role="alert"><p className="error-message">{message}</p>{onRetry ? <button className="secondary-action" onClick={onRetry} type="button">{t("common.retry")}</button> : null}</section>;
}
