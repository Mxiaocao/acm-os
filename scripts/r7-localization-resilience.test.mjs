import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createServer, createServerModuleRunner } from "vite";

const root = new URL("../", import.meta.url);
const [app, shells, css, messages] = await Promise.all([
  readFile(new URL("src/app/App.tsx", root), "utf8"),
  readFile(new URL("src/app/shells.tsx", root), "utf8"),
  readFile(new URL("src/app/app.css", root), "utf8"),
  readFile(new URL("src/app/i18n/messages.ts", root), "utf8"),
]);

const vite = await createServer({ configFile: false, root: process.cwd(), server: { middlewareMode: true } });
const runner = createServerModuleRunner(vite.environments.ssr, { hmr: false });
const [{ t, validateCatalogs }, translation] = await Promise.all([
  runner.import("/src/app/i18n/index.ts"),
  runner.import("/src/app/translation.ts"),
]);
test.after(() => vite.close());

test("R7 keeps long external content unchanged and localizes long application presentation", () => {
  const longExternalTitle = "A. The Extremely Long Codeforces Problem Title With Mixed 中文 Content and an Identifier 1920F";
  assert.equal(translation.displayProblemTitle("1920F", longExternalTitle), longExternalTitle);
  assert.equal(t("review.planOrdinal", { count: 100 }), "第 100 次计划复习");
  assert.equal(
    t("errors.longContentFixture", { detail: "C:\\very\\long\\workspace\\path\\diagnostics.json" }),
    "操作未能完成。请检查以下详细信息：C:\\very\\long\\workspace\\path\\diagnostics.json",
  );
  assert.deepEqual(validateCatalogs(), []);
});

test("R7 application-owned accessible names and visible copy do not leak known English UI", () => {
  const forbiddenApplicationCopy = [
    "Connect your workspace",
    "First submission result",
    "Today plan summary",
    "Contest Library navigation",
    "Collection cabinet",
    "Problem index is unavailable",
    "Open original problem",
    "Learning lifecycle",
    "Note location needs attention",
    "Vault is unavailable",
  ];
  for (const copy of forbiddenApplicationCopy) assert.doesNotMatch(shells, new RegExp(copy.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  for (const copy of ["Contest detail", "Problem statement", "Page not found"]) {
    assert.doesNotMatch(app, new RegExp(copy));
  }
  for (const copy of ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday", " · Rating "]) {
    assert.doesNotMatch(shells, new RegExp(copy));
  }
});

test("R7 dialogs and long-content surfaces remain reachable at narrow viewport heights", () => {
  assert.match(css, /\.modal-backdrop\s*>\s*div\s*\{[\s\S]*?max-height:\s*calc\(100(?:dvh|vh)\s*-\s*40px\);[\s\S]*?overflow-y:\s*auto;/);
  assert.match(css, /\.modal-backdrop\s*>\s*div\s*\{[\s\S]*?min-width:\s*0;/);
  assert.match(css, /\.error-message\s*\{[\s\S]*?overflow-wrap:\s*anywhere;/);
  assert.match(css, /\.content-panel\s*\{[\s\S]*?min-width:\s*0;/);
});

test("R7 user-visible frontend sources contain no replacement-character mojibake", () => {
  for (const [name, source] of [["shells", shells], ["messages", messages]]) {
    assert.doesNotMatch(source, /\uFFFD|锟斤拷|閿熸枻鎷/, `${name} contains mojibake`);
  }
});
