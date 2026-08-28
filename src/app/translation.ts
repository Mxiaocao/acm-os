/** Presentation helpers for external problem content.
 * External titles and statements are intentionally returned unchanged.
 */
export function displayProblemTitle(_index: string, title: string): string {
  return title;
}

export function buildChineseQuickView(html: string): string {
  const container = document.createElement("div");
  container.innerHTML = html;
  const text = (container.textContent ?? "").replace(/\s+/g, " ").trim();
  if (!text) return "题面没有可提取的文字。";
  return text.slice(0, 1400) + (text.length > 1400 ? "…" : "");
}
