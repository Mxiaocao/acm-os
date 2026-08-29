import assert from "node:assert/strict";
import test from "node:test";
import { createServer } from "vite";

const vite = await createServer({ configFile: false, root: process.cwd(), server: { middlewareMode: true } });
const { DEFAULT_LOCALE, FALLBACK_LOCALE, t, validateCatalogs, validateChineseCatalogMojibake } = await vite.ssrLoadModule("/src/app/i18n/index.ts");
test.after(() => vite.close());

test("defaults to zh-CN and translates semantic keys", () => {
  assert.equal(DEFAULT_LOCALE, "zh-CN");
  assert.equal(FALLBACK_LOCALE, "en");
  assert.equal(t("common.save"), "保存");
  assert.equal(t("nav.today"), "今日");
  assert.equal(t("today.pageTitle"), "今日计划");
});

test("preserves context-specific reward terminology and interpolation", () => {
  assert.equal(t("reward.xpLabel"), "经验值（XP）");
  assert.equal(t("reward.xpShort"), "XP");
  assert.equal(t("reward.undoRedemption"), "撤销兑换");
  assert.equal(t("common.itemCount", { count: 3 }), "3 项");
});

test("catalog placeholders validate and unknown keys never return undefined", () => {
  assert.deepEqual(validateCatalogs(), []);
  assert.equal(t("missing.key"), "Something went wrong.");
});

test("zh-CN application catalog contains no known mojibake signatures", () => {
  assert.deepEqual(validateChineseCatalogMojibake(), []);
});
