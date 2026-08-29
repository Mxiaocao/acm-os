/** Presentation helpers for external problem content.
 * External titles and statements are intentionally returned unchanged.
 */
/** Existing authoritative compatibility pairs for historical Russian imports. */
const CANONICAL_ENGLISH_TITLES: Record<string, string> = {
  "Три числа на доске": "Three Numbers on the Blackboard",
  "Плитки домино": "Domino Tiles",
  "Горячая картошка на складе фей": "Hot Potatoes at the Fairy Warehouse",
  "Лента для завтрашнего дня": "A Ribbon for Tomorrow",
  "Даже если весь мир перевернётся": "Even If the World Turns",
  "Сколько времени пройдет, пока ничего не останется?": "How Long Until Nothing Remains?",
  "Сколько времени пройдёт, пока ничего не останется?": "How Long Until Nothing Remains?",
};

export function displayProblemTitle(_index: string, title: string): string {
  return CANONICAL_ENGLISH_TITLES[title.trim()] ?? title;
}

export function buildChineseQuickView(html: string): string {
  const container = document.createElement("div");
  container.innerHTML = html;
  const text = (container.textContent ?? "").replace(/\s+/g, " ").trim();
  if (!text) return "题面没有可提取的文字。";
  return text.slice(0, 1400) + (text.length > 1400 ? "…" : "");
}
