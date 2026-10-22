import { useEffect, useRef, useState, type FormEvent } from "react";
import { loadLibraryCompletionDates, saveLibraryCompletionDate, type LibraryCompletionDateDto } from "../ipc/learning-calendar";
import { t } from "./i18n";

export function CalendarCompletionEditor({ initialDate, initialProblemId, today, onSaved, onClose }: {
  initialDate: string; initialProblemId?: string; today: string;
  onSaved: (date: string) => void; onClose: () => void;
}) {
  const [items, setItems] = useState<LibraryCompletionDateDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadFailed, setLoadFailed] = useState(false);
  const [retry, setRetry] = useState(0);
  const [query, setQuery] = useState("");
  const [problemId, setProblemId] = useState(initialProblemId ?? "");
  const [date, setDate] = useState(initialDate);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inFlight = useRef(false);
  useEffect(() => {
    let cancelled = false;
    setLoading(true); setLoadFailed(false);
    loadLibraryCompletionDates().then(rows => { if (!cancelled) setItems(rows); })
      .catch(() => { if (!cancelled) setLoadFailed(true); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [retry]);
  const item = items.find(row => row.problemId === problemId);
  const visible = items.filter(row => row.problemId === problemId || row.problemTitle.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()));
  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (inFlight.current) return;
    if (!item || !/^\d{4}-\d{2}-\d{2}$/.test(date) || date < "1900-01-01" || date > today) { setError(t("calendar.completionInvalid")); return; }
    inFlight.current = true; setBusy(true); setError(null);
    try {
      await saveLibraryCompletionDate(problemId, date);
      onSaved(date);
    } catch (cause) {
      const code = typeof cause === "string" ? cause : cause && typeof cause === "object" && "code" in cause ? String(cause.code) : "";
      setError(t(code === "invalid_completion_date" ? "calendar.completionInvalid"
        : code === "completion_problem_not_found" ? "calendar.completionNotFound"
        : code === "stale_vault_context" ? "calendar.completionStale" : "calendar.completionError"));
    } finally { inFlight.current = false; setBusy(false); }
  };
  return <form aria-label={t("calendar.editCompletion")} aria-busy={loading || busy} className="calendar-completion-form manual-import-form" onSubmit={save}>
    <p>{t("calendar.completionHelp")}</p>
    {loading ? <p role="status">{t("calendar.loading")}</p> : loadFailed ? <div role="alert">
      <p>{t("calendar.error")}</p><button className="secondary-action" type="button" onClick={() => setRetry(v => v + 1)}>{t("common.retry")}</button>
    </div> : items.length ? <>
      <label>{t("calendar.searchProblem")}<input autoFocus type="search" disabled={busy} value={query} onInput={event => setQuery(event.currentTarget.value)} /></label>
      <label>{t("calendar.selectProblem")}<select disabled={busy} required value={problemId} onChange={event => { setProblemId(event.currentTarget.value); setError(null); }}>
        <option value="">{t("calendar.selectProblem")}</option>
        {visible.map(row => <option key={row.problemId} value={row.problemId}>{row.problemTitle} · {row.date ?? t("calendar.unrecorded")}</option>)}
      </select></label>
      {item ? <p>{t("calendar.currentCompletion", { date: item.date ?? t("calendar.unrecorded") })}</p> : null}
      <label>{t("calendar.actualCompletionDate")}<input type="date" required min="1900-01-01" max={today} disabled={busy} value={date} onInput={event => setDate(event.currentTarget.value)} /></label>
    </> : <p>{t("calendar.completionEmpty")}</p>}
    {error ? <p role="alert">{error}</p> : null}
    <div className="action-row">
      <button className="primary-action" type="submit" disabled={loading || loadFailed || busy || !item}>{busy ? t("calendar.savingCompletion") : t("calendar.saveCompletion")}</button>
      <button className="secondary-action" type="button" disabled={busy} onClick={onClose}>{t("calendar.cancelCompletion")}</button>
    </div>
  </form>;
}
