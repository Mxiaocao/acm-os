import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { loadLearningCalendar, type LearningCalendarDto, type LearningCalendarEntryDto } from "../ipc/learning-calendar";
import { t } from "./i18n";
import { displayProblemTitle } from "./translation";
import { calendarDateKey, calendarMonthCells, parseCalendarDate } from "./calendar-dates";
import { participationKindLabel, participationScore } from "../ipc/contest-participation";
import "./learning-calendar.css";
import { CalendarCompletionEditor } from "./CalendarCompletionEditor";

const kindLabels = {
  library_added: "calendar.libraryAdded",
  study: "calendar.study", first_cold_start: "calendar.coldStart",
  long_term_review: "calendar.review", early_check: "calendar.earlyCheck",
} as const;

// One problem can be added, studied and reviewed on the same day.
// Keep its activities, but count and list it only once in that day's work.
function completedProblems(entries: LearningCalendarEntryDto[]) {
  const groups = new Map<string, LearningCalendarEntryDto[]>();
  for (const entry of entries) {
    if (entry.status !== "completed") continue;
    const key = entry.date + ":" + entry.problemId;
    const group = groups.get(key) ?? [];
    group.push(entry);
    groups.set(key, group);
  }
  return [...groups.values()];
}
const judgementLabels = {
  mastered: "calendar.mastered", partial: "calendar.partial", fail: "calendar.fail",
} as const;

export function LearningCalendar({ navigate }: { navigate: (path: string) => void }) {
  const initial = calendarDateKey(new Date());
  const [today, setToday] = useState(initial);
  const [selected, setSelected] = useState(initial);
  const [month, setMonth] = useState(initial.slice(0, 7));
  const [calendar, setCalendar] = useState<LearningCalendarDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [failed, setFailed] = useState(false);
  const [retry, setRetry] = useState(0);
  const [editor, setEditor] = useState<{ date: string; problemId?: string } | null>(null);
  const [savedMessage, setSavedMessage] = useState(false);
  const editorTrigger = useRef<HTMLButtonElement | null>(null);
  const openEditor = (date: string, trigger: HTMLButtonElement, problemId?: string) => {
    editorTrigger.current = trigger; setSavedMessage(false); setEditor({ date, problemId });
  };
  const closeEditor = () => { setEditor(null); queueMicrotask(() => editorTrigger.current?.isConnected && editorTrigger.current.focus()); };
  const touched = useRef(false);
  const focusDate = useRef<string | null>(null);
  const root = useRef<HTMLElement>(null);
  const detailRef = useRef<HTMLElement>(null);
  const [year, monthNumber] = month.split("-").map(Number);

  useEffect(() => {
    let cancelled = false;
    setLoading(true); setFailed(false); setCalendar(null);
    loadLearningCalendar(year, monthNumber).then((result) => {
      if (cancelled) return;
      setCalendar(result); setToday(result.today);
      if (!touched.current) { setMonth(result.today.slice(0, 7)); setSelected(result.today); }
    }).catch(() => { if (!cancelled) setFailed(true); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [year, monthNumber, retry]);

  useEffect(() => {
    const refresh = () => setRetry((value) => value + 1);
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, []);

  useEffect(() => {
    if (focusDate.current) {
      root.current?.querySelector<HTMLButtonElement>(`[data-calendar-date="${focusDate.current}"]`)?.focus();
      focusDate.current = null;
    }
  }, [selected, month, retry]);

  const choose = (date: string, focus = false) => {
    touched.current = true;
    if (focus) focusDate.current = date;
    setSelected(date); setMonth(date.slice(0, 7));
  };
  const moveMonth = (offset: number, focus = false) => {
    const date = parseCalendarDate(selected);
    const next = new Date(year, monthNumber - 1 + offset, 1, 12);
    if (next.getFullYear() < 1900 || next.getFullYear() > 9998) return;
    next.setDate(Math.min(date.getDate(), new Date(next.getFullYear(), next.getMonth() + 1, 0, 12).getDate()));
    choose(calendarDateKey(next), focus);
  };
  const handleDateKey = (event: KeyboardEvent<HTMLButtonElement>, key: string) => {
    if (event.key === "PageUp" || event.key === "PageDown") {
      event.preventDefault(); moveMonth(event.key === "PageUp" ? -1 : 1, true); return;
    }
    const date = parseCalendarDate(key);
    const weekday = (date.getDay() + 6) % 7;
    const offset = ({ ArrowLeft: -1, ArrowRight: 1, ArrowUp: -7, ArrowDown: 7, Home: -weekday, End: 6 - weekday } as Record<string, number>)[event.key];
    if (offset === undefined) return;
    event.preventDefault(); date.setDate(date.getDate() + offset);
    if (date.getFullYear() >= 1900 && date.getFullYear() <= 9998) choose(calendarDateKey(date), true);
  };

  const entries = calendar?.entries ?? [];
  const participations = calendar?.participations ?? [];
  const selectedContests = participations.filter(record => record.localDate === selected);
  const liveCount = participations.filter(record => record.kind === "live").length;
  const virtualCount = participations.length - liveCount;
  const selectedEntries = entries.filter((entry) => entry.date === selected);
  const completed = completedProblems(selectedEntries);
  const reviews = selectedEntries.filter((entry) => entry.status !== "completed");
  const cells = calendarMonthCells(year, monthNumber);
  const firstDayOffset = cells.findIndex(Boolean);
  const maxDayCount = Math.max(1, ...cells.map((date) => {
    const day = entries.filter((entry) => entry.date === date);
    return completedProblems(day).length + day.filter((entry) => entry.status !== "completed").length;
  }));
  const completedCount = completedProblems(entries).length;
  const reviewCount = entries.filter((entry) => entry.status !== "completed").length;
  const upcomingDates = [...new Set(entries.filter((entry) => entry.status !== "completed" && entry.date >= today).map((entry) => entry.date))].sort().slice(0, 7);
  const weekdayLabel = (date: string) => t((["calendar.sun", "calendar.mon", "calendar.tue", "calendar.wed", "calendar.thu", "calendar.fri", "calendar.sat"] as const)[parseCalendarDate(date).getDay()]);
  const dateLabel = (date: string) => t("calendar.dateLabel", { month: Number(date.slice(5, 7)), day: Number(date.slice(-2)), weekday: weekdayLabel(date) });
  const renderEntry = (entry: LearningCalendarEntryDto, index: number, activities = [entry]) => (
    <li key={entry.problemId + "-" + entry.kind + "-" + index}>
      <a className="calendar-problem-link" href={"/problems/id/" + entry.problemId} onClick={(event) => {
        if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
        event.preventDefault(); navigate("/problems/id/" + entry.problemId);
      }}>
        <span aria-hidden="true" className="calendar-problem-number">{String(index + 1).padStart(2, "0")}</span>
        <span className="calendar-problem-main">
          <strong>{displayProblemTitle("", entry.problemTitle)}</strong>
          <span>{[...new Set(activities.map((activity) => t(kindLabels[activity.kind]) + (activity.judgement ? " · " + t(judgementLabels[activity.judgement]) : "")))].join(" · ")}{entry.status === "inProgress" ? " · " + t("calendar.inProgress") : ""}</span>
          {entry.status !== "completed" && entry.dueDate && entry.dueDate < today
            ? <small className="calendar-overdue">{t("calendar.overdue", { date: entry.dueDate })}</small> : null}
        </span>
        <span className={"calendar-problem-status calendar-problem-status--" + entry.status}>
          {t("calendar.viewSolution")}
        </span>
      </a>
      {activities.some(activity => activity.kind === "library_added") ? <button className="secondary-action calendar-edit-date" type="button"
        disabled={editor !== null}
        aria-label={t("calendar.editProblemDate", { title: entry.problemTitle })}
        onClick={event => openEditor(entry.date, event.currentTarget, entry.problemId)}>{t("calendar.changeCompletionDate")}</button> : null}
    </li>
  );

  return <section aria-labelledby="learning-calendar-title" className="learning-calendar" ref={root}>
    <div className="calendar-overview">
      <p>{t("calendar.description")}</p>
      <p aria-live="polite">{loading ? t("calendar.loading") : failed ? t("calendar.error") : t("calendar.monthSummary", { done: completedCount, due: reviewCount })}</p>
      {!loading && !failed ? <p>{t("calendar.contestSummary", { live: liveCount, vp: virtualCount, total: participations.length })}</p> : null}
    </div>
    <header className="learning-calendar__header">
      <div className="calendar-month-nav">
        <button aria-label={t("calendar.previousMonth")} className="secondary-action" disabled={month === "1900-01"} onClick={() => moveMonth(-1)} type="button">← {t("calendar.previousMonth")}</button>
        <h2 aria-live="polite" id="learning-calendar-title">{t("calendar.month", { year, month: monthNumber })}</h2>
        <button aria-label={t("calendar.nextMonth")} className="secondary-action" disabled={month === "9998-12"} onClick={() => moveMonth(1)} type="button">{t("calendar.nextMonth")} →</button>
      </div>
      <div className="action-row">
        <button className="secondary-action" disabled={loading || failed || editor !== null || selected > today} type="button" onClick={event => {
          openEditor(selected, event.currentTarget); detailRef.current?.scrollIntoView?.({ block: "start" });
        }}>{t("calendar.editCompletion")}</button>
        <button className="secondary-action" onClick={() => choose(today)} type="button">{t("calendar.returnToday")}</button>
      </div>
    </header>
    <div className="learning-calendar__body">
      <div className="calendar-month" aria-busy={loading}>
        <div aria-hidden="true" className="calendar-weekdays">{(["calendar.mon", "calendar.tue", "calendar.wed", "calendar.thu", "calendar.fri", "calendar.sat", "calendar.sun"] as const).map((day) => <span key={day}>{t(day)}</span>)}</div>
        <div aria-label={t("calendar.dates")} className="calendar-days">
          {cells.map((date, index) => {
            if (!date) {
              const paddingDate = new Date(year, monthNumber - 1, index - firstDayOffset + 1, 12);
              return <div aria-hidden="true" className="calendar-day calendar-day--blank" key={"blank-" + index}>
                <span className="calendar-day__number">{paddingDate.getDate()}</span><span className="calendar-day__empty">—</span>
              </div>;
            }
            const dayEntries = entries.filter((entry) => entry.date === date);
            const done = completedProblems(dayEntries).length;
            const due = dayEntries.filter((entry) => entry.status !== "completed").length;
            const dayContests = participations.filter(record => record.localDate === date);
            const live = dayContests.filter(record => record.kind === "live").length;
            const vp = dayContests.length - live;
            return <button aria-current={date === today ? "date" : undefined} aria-label={loading || failed ? date : t("calendar.daySummary", { date, done, due }) + (dayContests.length ? "，" + t("calendar.contestCounts", { live, vp }) : "")}
              aria-pressed={date === selected} className={`calendar-day${date === today ? " calendar-day--today" : ""}`}
              data-calendar-date={date} key={date} onClick={() => choose(date)} onKeyDown={(event) => handleDateKey(event, date)} tabIndex={date === selected ? 0 : -1} type="button">
              <span className="calendar-day__number">{Number(date.slice(-2))}</span>
              <span className="calendar-day__counts">{done > 0 ? <span className="calendar-count--completed">{t("calendar.doneCount", { count: done })}</span> : null}
                {due > 0 ? <span className="calendar-count--review">{t("calendar.dueCount", { count: due })}</span> : null}
                {done + due + dayContests.length === 0 ? <span className="calendar-day__empty">—</span> : null}</span>
              {dayContests.length ? <span className="calendar-day__contests">
                {live > 0 ? <span>{t("calendar.liveCount", { count: live })}</span> : null}
                {vp > 0 ? <span>{t("calendar.virtualCount", { count: vp })}</span> : null}
              </span> : null}
              <span aria-hidden="true" className="calendar-day__bar">
                <span className="calendar-day__bar-completed" style={{ width: (done / maxDayCount * 100) + "%" }} />
                <span className="calendar-day__bar-review" style={{ width: (due / maxDayCount * 100) + "%" }} />
              </span>
            </button>;
          })}
        </div>
        <p className="calendar-legend"><span className="calendar-count--completed">{t("calendar.completed")}</span><span className="calendar-count--review">{t("calendar.toReview")}</span><span className="calendar-count--contest">{t("calendar.contests")}</span><small>{t("calendar.barExplanation")}</small></p>
      </div>
      <section aria-label={t("calendar.dayDetails")} className="calendar-detail" ref={detailRef}>
        <h3><time dateTime={selected}>{dateLabel(selected)}</time>{selected === today ? <span>{t("calendar.today")}</span> : null}</h3>
        {!editor ? <button className="secondary-action" disabled={loading || failed || selected > today} type="button"
          onClick={event => openEditor(selected, event.currentTarget)}>{t("calendar.editCompletion")}</button> : <CalendarCompletionEditor
          key={editor.date + ":" + (editor.problemId ?? "")} initialDate={editor.date} initialProblemId={editor.problemId} today={today}
          onClose={closeEditor} onSaved={date => { setEditor(null); setSavedMessage(true); choose(date, true); setRetry(value => value + 1); }} />}
        {savedMessage ? <p role="status">{t("calendar.completionSaved")}</p> : null}
        {loading ? <p role="status">{t("calendar.loading")}</p> : failed ? <div role="alert"><p>{t("calendar.error")}</p><button className="secondary-action" onClick={() => setRetry((value) => value + 1)} type="button">{t("common.retry")}</button></div> : <div aria-live="polite">
          <h4>{t("calendar.contests")} <span>{selectedContests.length}</span></h4>
          {selectedContests.length ? <ul className="calendar-contest-list">{selectedContests.map(record => <li key={record.id}>
            {record.contestId ? <a className="calendar-contest-link" href={"/contests/" + record.contestId} onClick={event => {
              if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
              event.preventDefault(); navigate("/contests/" + record.contestId);
            }}><strong>{record.contestTitle}</strong><span>{t("calendar.viewContest")} →</span></a> : <strong>{record.contestTitle} · {t("calendar.deletedContest")}</strong>}
            <p><span className="calendar-count--contest">{participationKindLabel(record.kind)}</span> · <time dateTime={`${record.localDate}T${record.localTime}`}>{record.localTime}</time> · {participationScore(record)}</p>
          </li>)}</ul> : <p className="calendar-empty">{t("calendar.noContests")}</p>}
          <h4>{t("calendar.completed")} <span>{completed.length}</span></h4>
          {completed.length ? <ul>{completed.map((activities, index) => renderEntry(activities[0], index, activities))}</ul> : <p className="calendar-empty">{t("calendar.noCompleted")}</p>}
          <h4>{t("calendar.toReview")} <span>{reviews.length}</span></h4>
          {reviews.length ? <ul>{reviews.map((entry, index) => renderEntry(entry, index))}</ul> : <p className="calendar-empty">{t("calendar.noReviews")}</p>}
        </div>}
      </section>
    </div>
    {!loading && !failed ? <section aria-labelledby="calendar-upcoming-title" className="calendar-upcoming">
      <h3 id="calendar-upcoming-title">{t("calendar.upcoming")}</h3>
      <p>{t("calendar.upcomingDescription")}</p>
      {upcomingDates.length ? <ul>{upcomingDates.map((date) => {
        const dueEntries = entries.filter((entry) => entry.date === date && entry.status !== "completed");
        return <li key={date}><button aria-label={t("calendar.openDate", { date })} aria-pressed={date === selected}
          className="calendar-upcoming-link" data-calendar-upcoming={date} onClick={() => { choose(date); detailRef.current?.scrollIntoView?.({ block: "start" }); }} type="button">
          <time dateTime={date}>{dateLabel(date)}</time><strong>{t("calendar.dueCount", { count: dueEntries.length })}</strong>
          <span>{dueEntries.map((entry) => displayProblemTitle("", entry.problemTitle)).join(" · ")}</span>
        </button></li>;
      })}</ul> : <p className="calendar-empty">{t("calendar.noUpcoming")}</p>}
    </section> : null}
    <p className="calendar-note">{t("calendar.scheduleNote")}</p>
  </section>;
}
