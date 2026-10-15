import { useState } from "react";
import type { PersonalNoteReadStateDto } from "../ipc/contest";
import { DEFAULT_NOTE_PROMPT, isPromptSection } from "./note-prompt";
import "./personal-note.css";

export function NotePromptPanel({ readState }: { readState: PersonalNoteReadStateDto | null }) {
  const [message, setMessage] = useState("");
  const savedPrompt = readState?.state === "ready"
    ? readState.projection.documentSections?.find(section => isPromptSection(section.name)) : undefined;
  const prompt = savedPrompt?.contentMarkdown ?? DEFAULT_NOTE_PROMPT;
  const copy = async () => {
    try { await navigator.clipboard.writeText(prompt); setMessage("提示词已复制，可粘贴到 AI 对话中，再附上本次题目材料。"); }
    catch { setMessage("自动复制失败，请选中提示词框中的文字手动复制。"); }
  };
  return <details className="note-prompt-panel">
    <summary>Prompt</summary>
    <div className="note-ai-assistant">
      <p className="safe-note">先复制提示词给 AI，再附上题面、代码或截图；检查结果后创建个人笔记并保存到 Obsidian。</p>
      {savedPrompt ? <p className="safe-note">当前使用这份 Markdown 中保存的提示词。</p> : null}
      <label>提示词<textarea readOnly rows={10} value={prompt} onFocus={event => event.currentTarget.select()} /></label>
      <button className="text-button" type="button" onClick={() => void copy()}>复制提示词</button>
      {message ? <p role="status">{message}</p> : null}
    </div>
  </details>;
}
