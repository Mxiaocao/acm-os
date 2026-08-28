import assert from "node:assert/strict";
import test from "node:test";
import { readFile } from "node:fs/promises";
import { createServer } from "vite";

const vite = await createServer({ configFile: false, root: process.cwd(), server: { middlewareMode: true } });
const { t } = await vite.ssrLoadModule("/src/app/i18n/index.ts");
test.after(() => vite.close());

test("R2 navigation labels come from typed i18n", () => {
  assert.deepEqual(
    [t("nav.today"), t("nav.contests"), t("nav.problems"), t("nav.review"), t("nav.knowledge"), t("nav.reward"), t("nav.settings")],
    ["今日", "比赛", "我的题库", "复习", "知识库", "奖励", "设置"],
  );
});

test("shared states expose accessible presentation semantics", async () => {
  const source = await readFile(new URL("../src/app/ui/states.tsx", import.meta.url), "utf8");
  assert.match(source, /aria-busy/);
  assert.match(source, /role="alert"/);
  assert.equal(t("common.retry"), "重试");
});
