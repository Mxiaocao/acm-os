import { useEffect, useRef, useState, type FormEvent } from "react";
import { v7 as uuid } from "uuid";
import { calendarDateKey } from "./calendar-dates";
import { deleteContestParticipation, listContestParticipations, saveContestParticipation, participationKindLabel, participationScore, type ContestParticipationDto, type ParticipationInput } from "../ipc/contest-participation";
import "./contest-participation.css";

const initial = (): ParticipationInput => {
  const now = new Date();
  return { id: uuid(), kind: "live", localDate: calendarDateKey(now), localTime: `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`, acceptedCount: null, totalCount: null };
};
function errorMessage(error: unknown) {
  const code = typeof error === "string" ? error : String((error as { code?: string })?.code ?? error);
  if (code.includes("duplicate_start")) return "这个时间已有参赛记录，请修改已有记录，或填写另一次参赛的实际开始时间。";
  if (code.includes("invalid_participation")) return "请检查参赛方式、日期、时间及成绩，AC数不能超过总题数。";
  return "操作未确认成功。请重试；重复保存同一条记录不会增加场次。";
}

export function ContestParticipationPanel({ contestId }: { contestId: number }) {
  const [records, setRecords] = useState<ContestParticipationDto[] | null>(null);
  const [draft, setDraft] = useState<ParticipationInput | null>(null);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  const [message, setMessage] = useState("");
  const [removeId, setRemoveId] = useState<string | null>(null);
  const [retry, setRetry] = useState(0);
  const lock = useRef(false);
  const formRef = useRef<HTMLFormElement>(null);
  useEffect(() => {
    let cancelled = false;
    setFailed(false); setRecords(null);
    listContestParticipations(contestId).then(value => { if (!cancelled) setRecords(value); }).catch(() => { if (!cancelled) setFailed(true); });
    return () => { cancelled = true; };
  }, [contestId, retry]);
  useEffect(() => { if (draft) formRef.current?.querySelector<HTMLSelectElement>("select")?.focus(); }, [draft?.id]);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!draft || lock.current) return;
    if (draft.totalCount !== null && (draft.totalCount < 1 || (draft.acceptedCount !== null && draft.acceptedCount > draft.totalCount))) { setMessage("AC数不能超过总题数，总题数须大于0。"); return; }
    lock.current = true; setBusy(true); setMessage("");
    try {
      await saveContestParticipation(contestId, draft);
      setDraft(null); setMessage("参赛记录已保存，学习月历已按实际开始日期计入。");
      setRetry(value => value + 1);
    } catch (error) { setMessage(errorMessage(error)); }
    finally { lock.current = false; setBusy(false); }
  };
  const remove = async () => {
    if (!removeId || lock.current) return;
    lock.current = true; setBusy(true); setMessage("");
    try { await deleteContestParticipation(contestId, removeId); setRemoveId(null); setRetry(value => value + 1); setMessage("参赛记录已删除，月历场次同步移除。"); }
    catch (error) { setMessage(errorMessage(error)); }
    finally { lock.current = false; setBusy(false); }
  };
  return <section className="content-panel contest-participations" aria-labelledby="participation-heading">
    <div className="participation-heading"><h2 id="participation-heading">参赛记录</h2><button className="primary-action" disabled={busy || draft !== null || removeId !== null || records === null} type="button" onClick={() => { setDraft(initial()); setMessage(""); }}>记录参赛</button></div>
    <p className="system-caption">现场与VP分别统计，按本地开始日期记一场，跨午夜不重复计数。同一场比赛可多次VP；导入比赛和赛后补题不增加场次。</p>
    {failed ? <div role="alert">参赛记录读取失败。<button className="secondary-action" type="button" onClick={() => setRetry(value => value + 1)}>重试读取参赛记录</button></div>
      : records === null ? <p role="status">正在读取参赛记录…</p> : records.length ? <ul className="participation-list">{records.map(record => <li key={record.id}>
        <div><strong>{participationKindLabel(record.kind)}</strong> <time dateTime={`${record.localDate}T${record.localTime}`}>{record.localDate} {record.localTime}</time><span>{participationScore(record)}</span></div>
        <div className="action-row"><button className="secondary-action" disabled={busy || draft !== null || removeId !== null} type="button" onClick={() => { setDraft({ id: record.id, kind: record.kind, localDate: record.localDate, localTime: record.localTime, acceptedCount: record.acceptedCount, totalCount: record.totalCount }); setMessage(""); }}>修改记录</button>
          <button className="danger-action" disabled={busy || draft !== null || removeId !== null} type="button" onClick={() => { setRemoveId(record.id); setMessage(""); }}>删除记录</button></div>
      </li>)}</ul> : <p className="safe-note">尚未记录参赛。已有比赛不会根据导入日期自动计入，可在这里补录。</p>}
    {draft ? <form className="participation-form" aria-label="记录参赛" ref={formRef} onSubmit={submit}>
      <fieldset disabled={busy}><legend>{records?.some(record => record.id === draft.id) ? "修改参赛记录" : "记录一次参赛"}</legend>
        <label>参赛方式<select value={draft.kind} onChange={event => setDraft({ ...draft, kind: event.currentTarget.value as ParticipationInput["kind"] })}><option value="live">现场／实时参赛</option><option value="virtual">赛后VP</option></select></label>
        <label>实际开始日期<input required type="date" min="1900-01-01" max="9998-12-31" value={draft.localDate} onInput={event => setDraft({ ...draft, localDate: event.currentTarget.value })} /></label>
        <label>实际开始时间<input required type="time" value={draft.localTime} onInput={event => setDraft({ ...draft, localTime: event.currentTarget.value })} /></label>
        <label>赛内AC数（可选）<input type="number" min="0" max={draft.totalCount ?? 4294967295} step="1" value={draft.acceptedCount ?? ""} onInput={event => setDraft({ ...draft, acceptedCount: event.currentTarget.value === "" ? null : Number(event.currentTarget.value) })} /></label>
        <label>总题数（可选）<input type="number" min="1" max="4294967295" step="1" value={draft.totalCount ?? ""} onInput={event => setDraft({ ...draft, totalCount: event.currentTarget.value === "" ? null : Number(event.currentTarget.value) })} /></label>
      </fieldset>
      <p className="system-caption">填写本次参赛成绩，不会覆盖原有赛后快照，也不额外增加当天做题数。</p>
      <div className="action-row"><button className="primary-action" disabled={busy} type="submit">{busy ? "保存中…" : "保存参赛记录"}</button><button className="secondary-action" disabled={busy} type="button" onClick={() => { setDraft(null); setMessage(""); }}>取消</button></div>
    </form> : null}
    {removeId ? <div role="alertdialog" aria-label="删除参赛记录"><p>删除这一次参赛记录？对应月历场次会移除，比赛、题目和笔记会保留。</p><div className="action-row"><button className="danger-action" disabled={busy} type="button" onClick={() => void remove()}>确认删除参赛记录</button><button className="secondary-action" disabled={busy} type="button" onClick={() => setRemoveId(null)}>取消</button></div></div> : null}
    {message ? <p role="status">{message}</p> : null}
  </section>;
}
