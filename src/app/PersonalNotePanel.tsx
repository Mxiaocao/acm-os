import { useEffect, useId, useRef, useState } from "react";
import type { PersonalNoteReadStateDto, SolutionRouteDto } from "../ipc/contest";
import { t } from "./i18n";
import "./personal-note.css";
import { isPromptSection, isSummarySection } from "./note-prompt";

export function PersonalNotePanel({ readState, refresh, renderContent }: {
  readState: Extract<PersonalNoteReadStateDto, { state: "ready" }>;
  refresh: () => Promise<void>;
  renderContent: (html: string) => string;
}) {
  const [selected, setSelected] = useState<SolutionRouteDto | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const sections = readState.projection.documentSections ?? [];
  const trigger = useRef<HTMLElement | null>(null);
  const dialog = useRef<HTMLDivElement>(null);
  const headingId = useId();
  const close = () => { setSelected(null); trigger.current?.focus(); };
  useEffect(() => {
    if (!selected) return;
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    dialog.current?.querySelector<HTMLButtonElement>("button")?.focus();
    const keydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") { event.preventDefault(); setSelected(null); trigger.current?.focus(); }
      if (event.key !== "Tab") return;
      const elements = Array.from(dialog.current?.querySelectorAll<HTMLElement>('button, a[href], [tabindex="0"]') ?? []);
      const first = elements[0];
      const last = elements[elements.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
    };
    document.addEventListener("keydown", keydown);
    return () => { document.body.style.overflow = previousOverflow; document.removeEventListener("keydown", keydown); };
  }, [selected]);
  // A refreshed projection invalidates the open route, including an externally deleted route.
  useEffect(() => { setSelected(null); if (selected) trigger.current?.focus(); }, [readState]);
  const routesContent = (routes: SolutionRouteDto[]) => routes.length ? <ol className="note-route-list">{routes.map((route, index) => <li key={index}><button className="text-button note-route-button" type="button" aria-haspopup="dialog" onClick={event => { trigger.current = event.currentTarget; setSelected(route); }}>{route.name}</button></li>)}</ol> : null;
  return <section className="content-panel personal-note-panel" aria-label={t("problem.projectionAria")}>
    <div className="note-heading-row"><h2>我的笔记</h2><button className="secondary-action" disabled={refreshing} type="button" onClick={async () => { setRefreshing(true); try { await refresh(); } finally { setRefreshing(false); } }}>{refreshing ? "正在刷新…" : "刷新笔记"}</button></div>
    {readState.relocated ? <p className="safe-note">{t("problem.bindingRestored")}</p> : null}
    {readState.projection.introductionHtml?.trim() ? <div className="note-introduction"><h3>大致题意</h3><div className="note-route-content" dangerouslySetInnerHTML={{ __html: renderContent(readState.projection.introductionHtml) }} /></div> : null}
    {sections.map((section, index) => {
      if (isSummarySection(section.name)) return <div className="note-summary-section" key={index}><h3>{section.name}</h3><div className="note-route-content" dangerouslySetInnerHTML={{ __html: renderContent(section.contentHtml) }} /></div>;
      if (isPromptSection(section.name)) return null;
      const routes = readState.projection.solutionRoutes.filter(route => route.startOffset >= section.startOffset && route.endOffset <= section.endOffset);
      return <details key={index} className="note-document-section"><summary>{section.name}</summary>
        {section.contentHtml?.trim() ? <div className="note-route-content" dangerouslySetInnerHTML={{ __html: renderContent(section.contentHtml) }} /> : null}
        {section.name === "题解" ? routesContent(routes) : null}
        {!section.contentHtml?.trim() && !routes.length ? <p className="safe-note">这一节还没有正文。</p> : null}
      </details>;
    })}
    {!readState.projection.documentSections ? <>
      {readState.projection.knownSections.filter(section => section.name !== "题解").map((section, index) => <details key={index}><summary>{section.name}</summary><p className="safe-note">刷新笔记以读取章节正文。</p></details>)}
      <details><summary>题解</summary>{routesContent(readState.projection.solutionRoutes) ?? <p>还没有题解方法。</p>}</details>
    </> : null}
    {readState.projection.warnings.map(warning => <p className="safe-note" key={`${warning.code}-${warning.name}`}>{t("problem.duplicateSection", { name: warning.name, count: warning.count })}</p>)}
    {selected ? <div className="modal-backdrop" onClick={event => { if (event.target === event.currentTarget) close(); }}><div className="note-route-dialog" role="dialog" aria-modal="true" aria-labelledby={headingId} ref={dialog}>
      <div className="note-heading-row"><h2 id={headingId}>{selected.name}</h2><button className="secondary-action" type="button" onClick={close}>关闭</button></div>
      {selected.contentMarkdown?.trim() ? <div className="note-route-content" dangerouslySetInnerHTML={{ __html: renderContent(selected.contentHtml) }} /> : <p>这个方法还没有正文。请在 Obsidian 中补充，或使用 AI 提示词编写题解后刷新。</p>}
    </div></div> : null}
  </section>;
}
