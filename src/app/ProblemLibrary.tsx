import { ArrowUpRight, MagnifyingGlass } from "@phosphor-icons/react";
import katex from "katex";
import { useMemo, useState, type ReactNode } from "react";

export interface LibraryProblem {
  id: string;
  title: string;
  code: string | null;
  source: string;
  rating: number | null;
  importedAtUtc?: string | null;
  statementHtml?: string | null;
  hasStatementSnapshot: boolean;
  route: string;
}

function statementText(html?: string | null): string {
  if (!html) return "";
  const template = document.createElement("template");
  template.innerHTML = html;
  const doc = template.content;
  doc.querySelectorAll(".header, script, style, .MathJax_Preview").forEach((node) => node.remove());
  doc.querySelectorAll("p, div, br, li, tr, h1, h2, h3, pre").forEach((node) => node.append(" "));
  return (doc.textContent ?? "").replace(/\s+/g, " ").trim();
}

function importDate(value?: string | null): string | null {
  if (!value) return null;
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return null;
  const parts = new Intl.DateTimeFormat("zh-CN", { year: "numeric", month: "2-digit", day: "2-digit" }).formatToParts(date);
  return ["year", "month", "day"].map((type) => parts.find((part) => part.type === type)?.value).join("-");
}

function statementPreview(text: string): ReactNode[] {
  const limit = 180;
  const parts: ReactNode[] = [];
  const pattern = /\$\$\$([\s\S]+?)\$\$\$|\$\$([\s\S]+?)\$\$|\$([^$\n]+?)\$|\\\(([\s\S]+?)\\\)|\\\[([\s\S]+?)\\\]/g;
  let cursor = 0;
  let used = 0;
  for (const match of text.matchAll(pattern)) {
    const start = match.index ?? 0;
    const plain = text.slice(cursor, start);
    if (used + plain.length >= limit) {
      parts.push(plain.slice(0, limit - used), "…");
      return parts;
    }
    parts.push(plain);
    used += plain.length;
    const expression = match.slice(1).find((value) => value !== undefined) ?? "";
    // Keep an entire formula when the excerpt boundary falls inside it.
    parts.push(<span className="problem-library-math" key={start} dangerouslySetInnerHTML={{ __html: katex.renderToString(expression, {
      displayMode: false, strict: "ignore", throwOnError: false, trust: false, maxExpand: 1000, maxSize: 16,
    }) }} />);
    used += expression.length;
    cursor = start + match[0].length;
    if (used >= limit) {
      if (cursor < text.length) parts.push("…");
      return parts;
    }
  }
  const remaining = text.slice(cursor);
  parts.push(remaining.slice(0, Math.max(0, limit - used)));
  if (used + remaining.length > limit) parts.push("…");
  return parts;
}

export function ProblemLibrary({ items, navigate }: { items: LibraryProblem[]; navigate: (route: string) => void }) {
  const [query, setQuery] = useState("");
  const indexed = useMemo(() => items.map((item) => {
    const text = statementText(item.statementHtml);
    return { ...item, text, preview: statementPreview(text), date: importDate(item.importedAtUtc) };
  }), [items]);
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  const visible = indexed.filter((item) => {
    const text = `${item.code ?? ""} ${item.title} ${item.source} ${item.rating ?? ""} ${item.text}`.toLocaleLowerCase();
    return terms.every((term) => text.includes(term));
  });

  return <div className="problem-library">
    <div className="problem-library-toolbar">
      <label className="problem-library-search"><MagnifyingGlass size={19} aria-hidden="true" />
        <span className="sr-only">搜索题库</span>
        <input onInput={(event) => setQuery(event.currentTarget.value)} placeholder="搜索题号、标题或题面关键词" type="search" value={query} />
      </label>
      <span role="status">{query.trim() ? `找到 ${visible.length} / ${items.length} 道题` : `共 ${items.length} 道题`}</span>
    </div>
    {[true, false].map((codeforces) => {
      const group = visible.filter((item) => Boolean(item.code) === codeforces);
      if (!group.length) return null;
      const name = codeforces ? "Codeforces 题目" : "独立题目";
      return <section aria-label={name} className="content-panel problem-library-section" key={name}>
        <h2>{name}<span>{group.length}</span></h2>
        <ul className="problem-library-list">{group.map((item) => <li key={item.id}>
          <button className="problem-library-row" onClick={() => navigate(item.route)} type="button">
            <span className={`problem-library-code${item.code ? "" : " problem-library-code--manual"}`}>{item.code ?? "自录"}</span>
            <span className="problem-library-body">
              <strong className="problem-library-title">{item.title}</strong>
              <span className="problem-library-preview">{item.text ? item.preview : item.hasStatementSnapshot ? "题面包含图片或公式，打开查看完整内容" : "题面尚未获取，打开题目补充内容"}</span>
              <span className="problem-library-meta"><span>{item.source}</span>
                {item.rating !== null ? <span className="problem-library-rating">难度 {item.rating}</span> : null}
                <span>{item.date ? <>导入于 <time dateTime={item.importedAtUtc ?? undefined}>{item.date}</time></> : "导入日期未记录"}</span>
              </span>
            </span>
            <ArrowUpRight className="problem-library-arrow" size={20} aria-hidden="true" />
          </button>
        </li>)}</ul>
      </section>;
    })}
    {visible.length === 0 && items.length > 0 ? <section className="empty-state"><h2>没有找到匹配的题目</h2><p>试试完整题号、标题或题面中的关键词。</p></section> : null}
  </div>;
}
