import assert from "node:assert/strict";
import test from "node:test";
import { JSDOM } from "jsdom";
import { createServer, createServerModuleRunner } from "vite";

const vite = await createServer({ configFile: false, root: process.cwd(), server: { middlewareMode: true } });
const runner = createServerModuleRunner(vite.environments.ssr, { hmr: false });
const translation = await runner.import("/src/app/translation.ts");

test.after(() => vite.close());

test("MutationObserver preserves dynamic external and user content inside skip boundary", async () => {
  const dom = new JSDOM("<body><p>Today</p><div data-i18n-skip></div></body>", { url: "http://localhost/" });
  const previous = { document: globalThis.document, MutationObserver: globalThis.MutationObserver, NodeFilter: globalThis.NodeFilter };
  Object.assign(globalThis, { document: dom.window.document, MutationObserver: dom.window.MutationObserver, NodeFilter: dom.window.NodeFilter });
  try {
    const dispose = translation.installChineseUiTranslation();
    assert.equal(dom.window.document.querySelector("body > p")?.textContent, "今日计划");
    const skip = dom.window.document.querySelector("[data-i18n-skip]");
    const child = dom.window.document.createElement("section");
    child.innerHTML = '<span>A. Save the City</span><p>Use Save and Retry as literal words here.</p>';
    skip.append(child);
    await new Promise((resolve) => dom.window.queueMicrotask(resolve));
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal(skip.textContent, "A. Save the CityUse Save and Retry as literal words here.");
    dispose();
  } finally {
    Object.assign(globalThis, previous);
    dom.window.close();
  }
});
