import { useEffect, useRef, useState } from "react";
import { deleteProblem, type DeleteProblemTarget } from "../ipc/contest";

export function ProblemDeleteButton({ title, target, onDeleted }: {
  title: string;
  target: DeleteProblemTarget;
  onDeleted: () => void;
}) {
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const dialog = useRef<HTMLDivElement>(null);
  const inFlight = useRef(false);
  const close = () => {
    if (inFlight.current) return;
    setOpen(false);
    queueMicrotask(() => trigger.current?.focus());
  };
  useEffect(() => {
    if (!open) return;
    dialog.current?.querySelector<HTMLButtonElement>("button")?.focus();
    const keydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") { event.preventDefault(); close(); }
      if (event.key !== "Tab") return;
      const buttons = [...(dialog.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
      if (!buttons.length) { event.preventDefault(); return; }
      const first = buttons[0], last = buttons[buttons.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener("keydown", keydown);
    return () => document.removeEventListener("keydown", keydown);
  }, [open]);
  const confirm = async () => {
    if (inFlight.current) return;
    inFlight.current = true;
    setPending(true); setError(null);
    try {
      await deleteProblem(target);
      setOpen(false);
      onDeleted();
    } catch (cause) {
      const code = typeof cause === "string" ? cause : cause instanceof Error ? cause.message
        : cause && typeof cause === "object" && "code" in cause ? String(cause.code) : "unknown";
      const messages: Record<string, string> = {
        problem_not_found: "题目不存在，请刷新后重试。",
        invalid_problem_id: "题目编号无效。",
        invalid_problem_identity: "题目标识无效。",
        problem_delete_unavailable: "数据库或备份暂不可用，删除未完成，请重试。",
        no_active_vault: "当前没有打开的资料库，请打开资料库后重试。",
        stale_vault_context: "资料库已切换，请在当前资料库中刷新题库。",
      };
      setError(messages[code] ?? `删除失败：${code}`);
    } finally { inFlight.current = false; setPending(false); }
  };
  return <>
    <button className="danger-action" onClick={() => { setError(null); setOpen(true); }} ref={trigger} type="button">删除题目</button>
    {open ? <div className="modal-backdrop"><div aria-labelledby="delete-problem-title" aria-describedby="delete-problem-description" aria-modal="true" role="alertdialog" ref={dialog} aria-busy={pending}>
      <h2 id="delete-problem-title">确认删除？</h2>
      <p>题目：{title}</p>
      <p id="delete-problem-description">删除后该题不会继续显示在题库中。已有刷题记录、标签和比赛关联将保留。</p>
      {error ? <p role="alert">{error}</p> : null}
      <div className="button-row">
        <button className="secondary-action" disabled={pending} onClick={close} type="button">取消</button>
        <button className="danger-action" disabled={pending} onClick={() => void confirm()} type="button">{pending ? "正在删除…" : "确认删除"}</button>
      </div>
    </div></div> : null}
  </>;
}
