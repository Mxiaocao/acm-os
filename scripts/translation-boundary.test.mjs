import assert from "node:assert/strict";
import test from "node:test";
import { JSDOM } from "jsdom";
import { createServer, createServerModuleRunner } from "vite";

const vite = await createServer({ configFile: false, root: process.cwd(), server: { middlewareMode: true } });
const runner = createServerModuleRunner(vite.environments.ssr, { hmr: false });
const translation = await runner.import("/src/app/translation.ts");

test.after(() => vite.close());

test("canonical English title is preferred for known historical imports", () => {
  assert.equal(translation.displayProblemTitle("A", "Три числа на доске"), "Three Numbers on the Blackboard");
  assert.equal(translation.displayProblemTitle("B", "Неизвестное название"), "Неизвестное название");
});

test("R6 retires the runtime translator while preserving external titles", () => {
  assert.equal(translation.installChineseUiTranslation, undefined);
  assert.equal(translation.displayProblemTitle("A", "A. Save the City"), "A. Save the City");
  assert.equal(translation.displayProblemTitle("B", "Задача"), "Задача");
});
