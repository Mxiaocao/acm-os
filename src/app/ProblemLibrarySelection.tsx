import { useEffect, useRef, useState } from "react";
import { addProblemLibraryItems, previewProblemLibrary, type ProblemLibraryPreviewDto } from "../ipc/contest";

export function ProblemLibrarySelection({ contestId, onClose }: { contestId: number; onClose: () => void }) {
  const [preview, setPreview] = useState<ProblemLibraryPreviewDto | null>(null);
  const [selected, setSelected] = useState<number[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [retry, setRetry] = useState(0);
  const pending = useRef(false);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    let active = true;
    heading.current?.focus();
    setPreview(null); setSelected([]); setError(null); setMessage(null);
    previewProblemLibrary(contestId).then((value) => { if (active) setPreview(value); })
      .catch(() => { if (active) setError("比赛已保存，题库状态读取失败，请重试。"); });
    return () => { active = false; };
  }, [contestId, retry]);

  const add = async () => {
    if (pending.current || selected.length === 0) return;
    pending.current = true; setBusy(true); setError(null); setMessage(null);
    try {
      await addProblemLibraryItems(contestId, selected);
      setSelected([]);
      setMessage("所选题目已加入我的题库。重复题目不会再次创建。");
      // Re-read membership: another view may have added a selected problem.
      try { setPreview(await previewProblemLibrary(contestId)); }
      catch { setPreview(null); setError("加入已完成，状态刷新失败，请重试。"); }
    } catch {
      setError("加入未确认，请重试；重复提交不会创建重复记录。");
    } finally { pending.current = false; setBusy(false); }
  };

  return <section className="content-panel" aria-label="加入我的题库">
    <h2 ref={heading} tabIndex={-1}>加入我的题库</h2>
    <p>比赛题目已保存在比赛库。勾选你想学习、记录和管理的题目，再加入个人题库。</p>
    {preview ? <>
      <h3>{preview.title}</h3><p>共 {preview.problems.length} 道比赛题</p>
      <ul className="detail-list">{preview.problems.map((item) => <li key={item.problemId}>
        <label><input type="checkbox" disabled={busy || item.alreadyAdded}
          checked={item.alreadyAdded || selected.includes(item.problemId)}
          onChange={(event) => {
            const checked = event.currentTarget.checked;
            setSelected((current) => checked
              ? [...current, item.problemId] : current.filter((id) => id !== item.problemId));
          }} />
          <span>{item.alreadyAdded ? "已存在" : "未加入"} · {item.index}. {item.title}</span>
        </label>
      </li>)}</ul>
    </> : !error ? <p aria-busy="true">正在检查个人题库…</p> : null}
    {error ? <p role="alert">{error}</p> : null}
    {message ? <p role="status">{message}</p> : null}
    <div className="action-row">
      <button className="primary-action" type="button" disabled={busy || !preview || selected.length === 0} onClick={() => void add()}>
        {busy ? "加入中…" : `加入我的题库（${selected.length}）`}
      </button>
      {error && !preview ? <button className="secondary-action" type="button" disabled={busy} onClick={() => setRetry((value) => value + 1)}>重试</button> : null}
      <button className="secondary-action" type="button" disabled={busy} onClick={onClose}>{message ? "完成" : "取消"}</button>
    </div>
  </section>;
}
