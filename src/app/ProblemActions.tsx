import { useRef, useState, type FormEvent } from "react";
import { updateProblem, type ProblemTarget, type ProblemUpdateFields } from "../ipc/contest";
import { ProblemDeleteButton } from "./ProblemDeleteButton";

export function ProblemActions({ value, target, onSaved, onDeleted }: {
  value: ProblemUpdateFields;
  target: ProblemTarget;
  onSaved: () => Promise<void>;
  onDeleted: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState("");
  const [sourceUrl, setSourceUrl] = useState("");
  const [rating, setRating] = useState("");
  const [pending, setPending] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inFlight = useRef(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const close = () => {
    if (inFlight.current) return;
    setEditing(false);
    queueMicrotask(() => trigger.current?.focus());
  };
  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (inFlight.current) return;
    let changes: ProblemUpdateFields | null = null;
    if (!saved) {
      if (!title.trim()) { setError("标题不能为空。"); return; }
      try {
        const url = new URL(sourceUrl.trim());
        if (!/^https?:$/.test(url.protocol) || !url.hostname || url.username || url.password
          || /\s|[\u0000-\u001f\u007f]/.test(sourceUrl.trim())) throw new Error();
      } catch { setError("请输入有效的 HTTP 或 HTTPS 题目链接。"); return; }
      const text = rating.trim();
      const numericRating = text === "" ? null : Number(text);
      if (numericRating !== null && (!/^\d+$/.test(text) || !Number.isInteger(numericRating)
        || numericRating <= 0 || numericRating > 4294967295)) {
        setError("难度必须是正整数，或留空。"); return;
      }
      changes = { title: title.trim(), sourceUrl: sourceUrl.trim(), rating: numericRating };
    }
    inFlight.current = true; setPending(true); setError(null);
    let persisted = saved;
    try {
      if (changes) { await updateProblem(target, changes); persisted = true; setSaved(true); }
      await onSaved();
      setEditing(false);
      queueMicrotask(() => trigger.current?.focus());
    } catch (cause) {
      const code = typeof cause === "string" ? cause : cause instanceof Error ? cause.message
        : cause && typeof cause === "object" && "code" in cause ? String(cause.code) : "unknown";
      const messages: Record<string, string> = {
        problem_title_required: "标题不能为空。",
        problem_url_invalid: "请输入有效的 HTTP 或 HTTPS 题目链接。",
        problem_rating_invalid: "难度必须是正整数，或留空。",
        problem_not_found_or_deleted: "题目不存在或已删除，无法保存。",
        problem_update_unavailable: "数据库或备份暂不可用，请重试。",
        stale_vault_context: "资料库已切换，请重新打开题目。",
        no_active_vault: "请先打开资料库。",
      };
      setError(persisted ? "修改已保存，但详情刷新失败。请重试刷新。" : messages[code] ?? `保存失败：${code}`);
    } finally { inFlight.current = false; setPending(false); }
  };
  if (!editing) return <div className="action-row">
    <button className="secondary-action" ref={trigger} type="button" onClick={() => {
      setTitle(value.title); setSourceUrl(value.sourceUrl); setRating(value.rating?.toString() ?? "");
      setSaved(false); setError(null); setEditing(true);
    }}>编辑题目</button>
    <ProblemDeleteButton title={value.title} target={target} onDeleted={onDeleted} />
  </div>;
  return <form className="manual-import-form" aria-label="编辑题目" aria-busy={pending} noValidate onSubmit={save}>
    <label>标题<input autoFocus disabled={pending || saved} required value={title} onInput={(event) => setTitle(event.currentTarget.value)} /></label>
    <label>链接<input disabled={pending || saved} required type="url" value={sourceUrl} onInput={(event) => setSourceUrl(event.currentTarget.value)} /></label>
    <label>难度（可留空）<input disabled={pending || saved} inputMode="numeric" value={rating} onInput={(event) => setRating(event.currentTarget.value)} /></label>
    {error ? <p role="alert">{error}</p> : null}
    <div className="action-row">
      <button className="primary-action" disabled={pending} type="submit">{pending ? "处理中…" : saved ? "重试刷新" : "保存"}</button>
      {!saved ? <button className="secondary-action" disabled={pending} type="button" onClick={close}>取消</button> : null}
    </div>
  </form>;
}
