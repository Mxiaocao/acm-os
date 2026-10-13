import { useRef, useState, type FormEvent } from "react";
import { v7 as uuid } from "uuid";
import { importSingleProblem } from "../ipc/single-problem";

export function SingleProblemImport({ onImported }: { onImported: (problemId: string) => Promise<void> }) {
  const [mode, setMode] = useState<"manual" | "codeforces">("manual");
  const [title, setTitle] = useState("");
  const [sourceUrl, setSourceUrl] = useState("");
  const [rating, setRating] = useState("");
  const [statement, setStatement] = useState("");
  const [busy, setBusy] = useState(false);
  const [savedId, setSavedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const requestId = useRef(uuid());
  const inFlight = useRef(false);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (inFlight.current) return;
    const source = sourceUrl.trim();
    let numericRating: number | null = null;
    if (!savedId) {
      if (mode === "manual" && (!title.trim() || !statement.trim())) { setError("请填写题目标题和题面正文。"); return; }
      if (source || mode === "codeforces") {
        try {
          const url = new URL(source);
          if (!/^https?:$/.test(url.protocol) || !url.hostname || url.username || url.password || /\s|[\u0000-\u001f\u007f]/.test(source)) throw new Error();
          if (mode === "codeforces" && (!/^(www\.)?codeforces\.com$/.test(url.hostname) || url.port
            || !/^\/(?:problemset\/problem\/\d+\/|contest\/\d+\/problem\/)[A-Z][0-9]?\/?$/.test(url.pathname))) {
            setError("请输入 Codeforces 单道题目链接，例如 https://codeforces.com/problemset/problem/1900/A。"); return;
          }
        } catch { setError("请输入有效的 HTTP 或 HTTPS 题目链接，手动录入可留空。"); return; }
      }
      if (mode === "manual" && rating.trim()) {
        numericRating = Number(rating);
        if (!/^\d+$/.test(rating.trim()) || !Number.isInteger(numericRating) || numericRating <= 0 || numericRating > 4294967295) {
          setError("难度必须是正整数，或留空。"); return;
        }
      }
    }
    inFlight.current = true; setBusy(true); setError(null);
    let persistedId = savedId;
    try {
      if (!persistedId) {
        const result = await importSingleProblem({ mode, requestId: requestId.current, title: title.trim(), sourceUrl: source, rating: numericRating, statementText: statement.trim() });
        persistedId = result.problemId; setSavedId(persistedId);
      }
      await onImported(persistedId);
      setSavedId(null); setTitle(""); setStatement(""); setSourceUrl(""); setRating(""); requestId.current = uuid();
    } catch (cause) {
      const code = typeof cause === "string" ? cause : cause && typeof cause === "object" && "code" in cause ? String(cause.code) : "unknown";
      const messages: Record<string, string> = {
        single_problem_invalid_input: "请检查标题、题面和难度后重试。",
        single_problem_invalid_url: "题目链接格式不正确，请使用 HTTP 或 HTTPS 链接。",
        single_problem_invalid_codeforces_url: "请填写 Codeforces 单道题目的链接。",
        single_problem_remote_unavailable: "自动获取题面失败。可以重试，或切换到手动录入并粘贴题面。",
        single_problem_remote_not_found: "没有找到这道题，请检查题目链接。",
        single_problem_unavailable: "暂时无法保存题目，请稍后重试。",
        stale_vault_context: "工作区已切换，请在当前工作区重新导入。",
      };
      setError(persistedId ? "题目已导入，但题库刷新失败。点击重试刷新即可。" : messages[code] ?? "单题导入未完成，请重试。");
    } finally { inFlight.current = false; setBusy(false); }
  };
  const locked = busy || savedId !== null;
  return <details className="content-panel single-problem-import">
    <summary>单题导入<span>独立加入一道题，不需要先导入比赛</span></summary>
    <div className="single-problem-modes" role="group" aria-label="单题导入方式">
      <button aria-pressed={mode === "manual"} disabled={locked} onClick={() => { setMode("manual"); setError(null); }} type="button">手动录入</button>
      <button aria-pressed={mode === "codeforces"} disabled={locked} onClick={() => { setMode("codeforces"); setError(null); }} type="button">Codeforces 题目链接</button>
    </div>
    <form aria-label="单题导入" aria-busy={busy} className="manual-import-form" noValidate onSubmit={submit}>
      {mode === "manual" ? <><p>适用于其他平台、书本或自拟题。来源链接可留空。</p>
        <label>题目标题<input disabled={locked} maxLength={200} onInput={(event) => setTitle(event.currentTarget.value)} required value={title} /></label></> : <p>仅获取这一道题的题面。支持题库链接和比赛中的单题链接。</p>}
      <label>{mode === "manual" ? "来源链接（可选）" : "Codeforces 题目链接"}<input disabled={locked} onInput={(event) => setSourceUrl(event.currentTarget.value)} placeholder={mode === "manual" ? "可留空" : "https://codeforces.com/problemset/problem/1900/A"} required={mode === "codeforces"} type="url" value={sourceUrl} /></label>
      {mode === "manual" ? <><label>难度（可选）<input disabled={locked} inputMode="numeric" onInput={(event) => setRating(event.currentTarget.value)} value={rating} /></label>
        <label className="single-problem-statement">题面正文<textarea disabled={locked} maxLength={2 * 1024 * 1024} onInput={(event) => setStatement(event.currentTarget.value)} required rows={7} value={statement} /></label></> : null}
      {error ? <p role="alert">{error}</p> : null}
      <div className="action-row"><button className="primary-action" disabled={busy} type="submit">{busy ? "处理中…" : savedId ? "重试刷新" : "导入单题"}</button></div>
    </form>
  </details>;
}
