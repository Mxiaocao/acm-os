import assert from "node:assert/strict";
import test from "node:test";
import path from "node:path";

import { JSDOM } from "jsdom";
import React, { act } from "react";
import { createRoot } from "react-dom/client";
import { createServer, createServerModuleRunner, loadConfigFromFile } from "vite";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

const vite = await createServer({
  configFile: false,
  optimizeDeps: { noDiscovery: true, include: [] },
  root: process.cwd(),
  cacheDir: ".dom-shells-vite-cache",
  plugins: [{
    name: "dom-shells-static-asset-stub",
    enforce: "pre",
    resolveId(source) {
      if (source === "katex/dist/katex.min.css") return `\0dom-shells-css:${source}`;
      return undefined;
    },
    load(id) {
      if (id.startsWith("\0dom-shells-css:")) return "export default {};";
      return undefined;
    },
    transform(code, id) {
      if (id.endsWith("/src/app/shells.tsx")) {
        return code
          .replace('import "katex/dist/katex.min.css";', "")
          .replace(
            /import (\w+) from "(\.\.\/assets\/[^\"]+\.(?:png|jpe?g|webp|svg))";/g,
            (_match, name, source) => `const ${name} = ${JSON.stringify(`dom-shells-asset://${source}`)};`,
          );
      }
      return undefined;
    },
  }],
  server: { middlewareMode: true, hmr: false },
});
const moduleRunner = createServerModuleRunner(vite.environments.ssr, { hmr: false });
moduleRunner.options.transport.timeout = 300_000;
moduleRunner.transport.timeout = 300_000;
const shells = await moduleRunner.import("/src/app/shells.tsx");

test("Contest artwork keeps production asset URLs relative to the Tauri bundle", async () => {
  const loaded = await loadConfigFromFile({ command: "build", mode: "production" }, path.resolve("vite.config.ts"));
  assert.equal(loaded.config.base, "./");
});

const vaultFixture = { id: "test-vault", name: "Test Vault", rootPath: "C:/Vault", databasePath: "C:/Vault/.acm-os/system-facts.sqlite3", createdAtUtc: "2026-09-15T00:00:00Z", isCurrent: true };
const managerFixture = { vaults: [vaultFixture], current: { state: "ready", vault: vaultFixture }, context: { vaultId: "test-vault", epoch: "test-session" } };

const configuredWorkspace = {
  state: "configured",
  activeVaultPath: "C:/Vault",
  problemRootPath: "C:/Vault/Problems",
  knowledgeRootPath: "C:/Vault/Knowledge",
};
const readyFoundation = {
  state: "ready",
  foundation: { status: "ready", core: "acm-os" },
};

test("Contest Detail focuses the final heading after async detail loading", { concurrency: false }, async () => {
  let resolveDetail;
  const detailPromise = new Promise((resolve) => { resolveDetail = resolve; });
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detailPromise;
    if (command === "contest_library_list_placements") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests/1979");
  try {
    await settle();
    assert.equal(view.document.querySelector("h1")?.textContent, "正在加载比赛");
    assert.equal(view.document.activeElement, view.document.querySelector("h1"));
    await act(async () => resolveDetail({ contestId: 1979, title: "Contest", sourceUrl: "https://codeforces.com/contest/1979", contestDate: "2026-08-10", importStatus: "complete", factsStatus: "completed", problems: [], corrections: [], aiAnalysis: null, archived: false }));
    await settle();
    const heading = view.document.querySelector("h1");
    assert.equal(heading?.textContent, "Contest");
    assert.equal(view.document.activeElement, heading);
  } finally { await view.cleanup(); }
});

test("Contest AI analysis previews before explicit save and preserves failed raw text", { concurrency: false }, async () => {
  const calls = [];
  let detail = { contestId: 1979, title: "Contest", sourceUrl: "https://codeforces.com/contest/1979", contestDate: "2026-08-10", importStatus: "complete", factsStatus: "completed", problems: [], corrections: [], aiAnalysis: null };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detail;
    if (command === "preview_contest_ai_analysis") { calls.push(command); return { rawText: args.input.rawText, parseStatus: "failed", parsedProjectionJson: "{}" }; }
    if (command === "save_contest_ai_analysis") { calls.push(command); detail = { ...detail, aiAnalysis: { rawText: args.input.rawText, parseStatus: "failed", parsedProjectionJson: "{}", updatedAtUtc: "2026-08-13T00:00:00Z" } }; return detail; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests/1979");
  try {
    await settle();
    const textarea = view.document.querySelector('textarea[aria-label="比赛 AI 分析原始文本"]');
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLTextAreaElement.prototype, "value").set.call(textarea, "unstructured raw"); textarea.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await settle();
    const buttons = [...view.document.querySelectorAll("button")];
    const preview = buttons.find((button) => button.textContent === "解析预览");
    let save = buttons.find((button) => button.textContent === "保存分析");
    assert.equal(save.disabled, true);
    await act(async () => preview.click()); await settle();
    assert.deepEqual(calls, ["preview_contest_ai_analysis"]);
    assert.match(view.document.body.textContent, /预览：FAILED/);
    save = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "保存分析");
    assert.equal(save.disabled, false);
    await act(async () => save.click()); await settle();
    assert.deepEqual(calls, ["preview_contest_ai_analysis", "save_contest_ai_analysis"]);
    assert.match(view.document.body.textContent, /已保存原始分析（FAILED）/);
  } finally { await view.cleanup(); }
});

for (const route of ["/problems/1979/A", "/problems/id/42", "/contests/1979"]) {
  test(`Original source opens through desktop IPC with failure feedback and retry (${route})`, { concurrency: false }, async () => {
    const url = "https://codeforces.com/gym/702542/problem/A";
    const calls = [];
    let finish;
    let fail = true;
    const view = await renderApp((command, args) => {
      if (command === "foundation_status") return { status: "ready", core: "acm-os" };
      if (command === "app_shell_status") return { state: "normal", workspace: configuredWorkspace };
      if (command === "lightweight_problem_detail" || command === "lightweight_problem_detail_by_id") return {
        problemId: "42", contestId: 1979, index: "A", title: "Problem A", rating: null,
        sourceUrl: url, statement: { state: "pending" }, identityType: "lightweight", personalNote: null,
        lifecycle: lightweightLifecycle,
      };
      if (command === "contest_detail") return { contestId: 1979, title: "Round", sourceUrl: url, contestDate: null, importStatus: "complete", factsStatus: "pending", problems: [], corrections: [], aiAnalysis: null, archived: false };
      if (command === "contest_library_list_contest_placements") return [];
      if (command === "open_original_oj") {
        calls.push(args.input);
        if (fail) throw "external_open_failed";
        return new Promise((resolve) => { finish = resolve; });
      }
      throw new Error(`unexpected command ${command}`);
    }, route);
    try {
      await settle();
      const link = [...view.document.querySelectorAll("a")].find((item) => item.textContent === (route.startsWith("/contests") ? "打开原比赛" : "打开原题"));
      assert.ok(link);
      assert.equal(link.href, url);
      const click = () => {
        const event = new view.window.MouseEvent("click", { bubbles: true, cancelable: true });
        link.dispatchEvent(event);
        assert.equal(event.defaultPrevented, true);
      };
      await act(async () => click()); await settle();
      assert.deepEqual(calls, [{ url }]);
      assert.match(view.document.querySelector('[role="alert"]').textContent, /系统浏览器/);
      assert.ok(view.document.querySelector('[role="alert"]').textContent.includes(url));
      fail = false;
      await act(async () => { click(); click(); }); await settle();
      assert.equal(calls.length, 2);
      assert.equal(view.document.querySelector('[role="alert"]')?.textContent ?? null, null);
      await act(async () => finish()); await settle();
    } finally { await view.cleanup(); }
  });
}

test("Manual Contest preserves failed drafts and clears all fields after a successful retry", { concurrency: false }, async () => {
  const calls = [];
  let failImport = true;
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_contests") return [];
    if (command === "preview_problem_library") return { title: "Manual Round", problems: [{ problemId: 1, index: "A", title: "Manual A", alreadyAdded: false }] };
    if (command === "import_manual_codeforces_contest") { calls.push(args.input); if (failImport) throw new Error("save_failed"); return { contestId: 1979, importStatus: "complete", missingSnapshotProblems: [], failedSnapshotProblems: [] }; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const addPanel = view.document.querySelector(".contest-add-panel");
    assert.ok(addPanel);
    assert.equal(view.document.querySelectorAll(".contest-add-panel").length, 1);
    assert.equal(view.document.querySelector(".content-panel.contest-import-form"), null);
    assert.equal(view.document.querySelector(".content-panel.manual-import-panel"), null);
    assert.match(addPanel.querySelector("h2").textContent, /添加比赛/);
    assert.ok(addPanel.querySelector('.contest-import-form input[placeholder="https://codeforces.com/contest/1979"]'));
    const details = addPanel.querySelector("details.manual-import-panel");
    assert.equal(details.open, false);
    await act(async () => { details.open = true; details.dispatchEvent(new view.window.Event("toggle")); });
    assert.equal(details.open, true);
    const inputs = [...details.querySelectorAll("input")];
    const values = ["1979", "Manual Round", "2026-08-13", "A", "Manual A", "https://codeforces.com/contest/1979/problem/A"];
    await act(async () => { inputs.forEach((input, index) => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, values[index]); input.dispatchEvent(new view.window.Event("input", { bubbles: true })); }); const textarea = details.querySelector("textarea"); Object.getOwnPropertyDescriptor(view.window.HTMLTextAreaElement.prototype, "value").set.call(textarea, "x < y"); textarea.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await act(async () => [...details.querySelectorAll("button")].find((button) => button.textContent === "添加题目").click());
    const secondValues = ["B", "Manual B", "https://codeforces.com/contest/1979/problem/B"];
    await act(async () => {
      [...details.querySelectorAll("fieldset")][1].querySelectorAll("input").forEach((input, index) => {
        Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, secondValues[index]);
        input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      });
      const textarea = details.querySelectorAll("textarea")[1];
      Object.getOwnPropertyDescriptor(view.window.HTMLTextAreaElement.prototype, "value").set.call(textarea, "second statement");
      textarea.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await act(async () => details.querySelector("form").dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.equal(calls.length, 1);
    assert.deepEqual([...details.querySelectorAll("input")].map((input) => input.value), [...values, ...secondValues]);
    assert.deepEqual([...details.querySelectorAll("textarea")].map((input) => input.value), ["x < y", "second statement"]);
    assert.equal(view.document.querySelector('[aria-label="加入我的题库"]'), null);
    failImport = false;
    await act(async () => details.querySelector("form").dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.equal(calls.length, 2);
    assert.deepEqual(calls[1], calls[0]);
    assert.equal(calls[0].contestId, 1979);
    assert.equal(calls[0].problems[0].index, "A");
    assert.equal(calls[0].problems[0].statementText, "x < y");
    assert.equal(calls[0].problems.length, 2);
    assert.equal(details.querySelectorAll("fieldset").length, 1);
    assert.deepEqual([...details.querySelectorAll("input")].map((input) => input.value), ["", "", "", "A", "", ""]);
    assert.equal(details.querySelector("textarea").value, "");
    assert.equal(view.document.querySelector('[aria-label="加入我的题库"] input[type="checkbox"]').checked, false);
    assert.match(view.document.body.textContent, /已通过标准导入和题面快照契约保存/);
  } finally { await view.cleanup(); }
});

test("Contest delete requires consequence preview and archive stays reversible", { concurrency: false }, async () => {
  const calls = [];
  let detail = { contestId: 1979, title: "Contest", sourceUrl: "https://codeforces.com/contest/1979", contestDate: "2026-08-10", importStatus: "complete", factsStatus: "completed", problems: [], corrections: [], aiAnalysis: null, archived: false };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detail;
    if (command === "set_contest_archived") { calls.push(command); detail = { ...detail, archived: args.input.archived }; return detail; }
    if (command === "preview_delete_contest") { calls.push(command); return { contestTitle: "Contest", relationshipCount: 2, cleanupProblemCount: 1, preservedProblemCount: 1 }; }
    if (command === "delete_contest") { calls.push(command); return { contestTitle: "Contest", relationshipCount: 2, cleanupProblemCount: 1, preservedProblemCount: 1 }; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests/1979");
  try {
    await settle();
    const archive = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "归档比赛");
    await act(async () => archive.click()); await settle();
    assert.deepEqual(calls, ["set_contest_archived"]);
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "恢复比赛"));
    const preview = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "预览删除影响");
    assert.equal(calls.includes("delete_contest"), false);
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /保留 1 道具有身份或历史的全局题目/);
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "删除比赛");
    await act(async () => confirm.click()); await settle();
    assert.deepEqual(calls, ["set_contest_archived", "preview_delete_contest", "delete_contest"]);
    assert.equal(view.window.location.pathname, "/contests");
  } finally { await view.cleanup(); }
});
const lightweightLifecycle = {
  learningStatus: "unstarted",
  learningStatusSinceUtc: "2026-08-11T00:00:00.000Z",
  nextReviewDueLocalDate: null,
  availableActions: [],
};

for (const canonical of [false, true]) {
  test(`Problem edit validates, cancels, retries and refreshes (${canonical ? "canonical" : "external"})`, { concurrency: false }, async () => {
    let detail = {
      problemId: "42", contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://example.test/a", statement: { state: "pending" },
      identityType: "lightweight", personalNote: null, lifecycle: lightweightLifecycle,
    };
    const writes = [];
    let failSave = true, failRefresh = false, finish;
    const view = await renderApp((command, args) => {
      if (command === "foundation_status") return { status: "ready", core: "acm-os" };
      if (command === "app_shell_status") return { state: "normal", workspace: configuredWorkspace };
      if (command === "lightweight_problem_detail" || command === "lightweight_problem_detail_by_id") {
        if (failRefresh) throw "problem_unavailable";
        return detail;
      }
      if (command === "update_problem") {
        writes.push(args.input);
        if (failSave) throw "problem_update_unavailable";
        return new Promise((resolve) => { finish = () => {
          detail = { ...detail, title: args.input.title, sourceUrl: args.input.sourceUrl, rating: args.input.rating };
          resolve();
        }; });
      }
      throw new Error(`unexpected command ${command}`);
    }, canonical ? "/problems/id/42" : "/problems/1979/A");
    const button = (label) => [...view.document.querySelectorAll("button")].find((b) => b.textContent === label);
    const fill = async (position, value) => act(async () => {
      const input = view.document.querySelectorAll('form[aria-label="编辑题目"] input')[position];
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, value);
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    const submit = async () => act(async () => view.document.querySelector('form[aria-label="编辑题目"]').dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    try {
      await settle();
      await act(async () => button("编辑题目").click());
      await fill(0, "Discarded");
      await act(async () => button("取消").click());
      assert.equal(writes.length, 0);
      assert.match(view.document.querySelector("h1").textContent, /Problem A/);
      await act(async () => button("编辑题目").click());
      assert.equal(view.document.querySelector('form[aria-label="编辑题目"] input').value, "Problem A");
      await fill(0, "   "); await submit();
      assert.match(view.document.querySelector('[role="alert"]').textContent, /标题不能为空/);
      await fill(0, " Problem B "); await fill(1, "javascript:alert(1)"); await submit();
      assert.match(view.document.querySelector('[role="alert"]').textContent, /HTTP/);
      await fill(1, "https://example.test/b"); await fill(2, "-1"); await submit();
      assert.match(view.document.querySelector('[role="alert"]').textContent, /正整数/);
      assert.equal(writes.length, 0);
      await fill(2, "1600"); await submit();
      assert.match(view.document.querySelector('[role="alert"]').textContent, /数据库或备份/);
      assert.equal(view.document.querySelector('form[aria-label="编辑题目"] input').value, " Problem B ");
      failSave = false;
      await submit(); await submit();
      assert.equal(writes.length, 2);
      assert.ok(button("处理中…").disabled);
      assert.ok(button("取消").disabled);
      assert.deepEqual(writes[1], { target: canonical ? { problemId: "42" } : { contestId: 1979, index: "A" }, title: "Problem B", sourceUrl: "https://example.test/b", rating: 1600 });
      failRefresh = true;
      await act(async () => finish()); await settle();
      assert.match(view.document.querySelector('[role="alert"]').textContent, /修改已保存.*刷新失败/);
      failRefresh = false;
      await act(async () => button("重试刷新").click()); await settle();
      assert.equal(writes.length, 2);
      assert.match(view.document.querySelector("h1").textContent, /Problem B/);
      assert.ok(view.document.querySelector('a[href="https://example.test/b"]'));
      await act(async () => button("编辑题目").click());
      await fill(2, ""); await submit();
      assert.equal(writes[2].rating, null);
      await act(async () => finish()); await settle();
      assert.ok(button("编辑题目"));
    } finally { await view.cleanup(); }
  });

  test(`Problem delete confirms, cancels, retries and refreshes library (${canonical ? "canonical" : "external"})`, { concurrency: false }, async () => {
    const calls = [];
    let fail = true;
    let finish;
    let reads = 0;
    const view = await renderApp((command, args) => {
      if (command === "foundation_status") return { status: "ready", core: "acm-os" };
      if (command === "app_shell_status") return { state: "normal", workspace: configuredWorkspace };
      if (command === "lightweight_problem_detail" || command === "lightweight_problem_detail_by_id") return {
        problemId: "42", contestId: 1979, index: "A", title: "Problem A", rating: 800,
        sourceUrl: "https://codeforces.com/contest/1979/problem/A",
        statement: { state: "pending" }, identityType: "lightweight", personalNote: null,
        lifecycle: lightweightLifecycle,
      };
      if (command === "delete_problem") {
        calls.push(args.input);
        if (fail) throw "problem_delete_unavailable";
        return new Promise((resolve) => { finish = resolve; });
      }
      if (command === "lightweight_problems") { reads++; return []; }
      throw new Error(`unexpected command ${command}`);
    }, canonical ? "/problems/id/42" : "/problems/1979/A");
    const button = (label) => [...view.document.querySelectorAll("button")].find((b) => b.textContent === label);
    try {
      await settle();
      await act(async () => button("删除题目").click());
      assert.match(view.document.querySelector('[role="alertdialog"]').textContent, /Problem A/);
      assert.equal(view.document.activeElement, button("取消"));
      assert.equal(calls.length, 0);
      await act(async () => button("取消").click());
      assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
      assert.equal(view.document.activeElement, button("删除题目"));
      await act(async () => button("删除题目").click());
      await act(async () => button("确认删除").click());
      assert.match(view.document.querySelector('[role="alert"]').textContent, /数据库或备份暂不可用/);
      assert.equal(calls.length, 1);
      fail = false;
      await act(async () => { button("确认删除").click(); button("确认删除").click(); });
      assert.equal(calls.length, 2);
      assert.ok(button("正在删除…").disabled);
      assert.ok(button("取消").disabled);
      assert.deepEqual(calls[1], canonical ? { problemId: "42" } : { contestId: 1979, index: "A" });
      await act(async () => finish());
      await settle();
      assert.equal(view.window.location.pathname, "/problems");
      assert.equal(reads, 1);
      assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
      assert.doesNotMatch(view.document.body.textContent, /Problem A/);
    } finally { await view.cleanup(); }
  });
}
const personalUnstartedLifecycle = {
  ...lightweightLifecycle,
  availableActions: ["joinUpsolve"],
};

async function render(component) {
  const dom = new JSDOM("<!doctype html><html><body><div id=\"root\"></div></body></html>", {
    url: "https://acm-os.test/today",
  });
  dom.window.HTMLElement.prototype.attachEvent ??= () => {};
  dom.window.HTMLElement.prototype.detachEvent ??= () => {};
  const globals = {
    window: dom.window,
    document: dom.window.document,
    navigator: dom.window.navigator,
    HTMLElement: dom.window.HTMLElement,
    HTMLAnchorElement: dom.window.HTMLAnchorElement,
    HTMLImageElement: dom.window.HTMLImageElement,
    DOMParser: dom.window.DOMParser,
    NodeFilter: dom.window.NodeFilter,
  };
  const previous = Object.fromEntries(
    Object.keys(globals).map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]),
  );
  for (const [key, value] of Object.entries(globals)) {
    Object.defineProperty(globalThis, key, { configurable: true, value, writable: true });
  }
  const root = createRoot(dom.window.document.getElementById("root"));
  await act(async () => root.render(component));
  return {
    document: dom.window.document,
    cleanup: async () => {
      await act(async () => root.unmount());
      for (const [key, descriptor] of Object.entries(previous)) {
        if (descriptor) Object.defineProperty(globalThis, key, descriptor);
        else Reflect.deleteProperty(globalThis, key);
      }
      dom.window.close();
    },
  };
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

async function renderApp(ipc, pathname = "/", strict = false, shelfWidth = 699, calendarIpc = null, independentIpc = null, participationIpc = null) {
  const dom = new JSDOM("<!doctype html><html><body><div id=\"root\"></div></body></html>", {
    url: `https://acm-os.test${pathname}`,
  });
  const resizeObservers = new Set();
  dom.window.ResizeObserver = class {
    constructor(callback) { this.callback = callback; resizeObservers.add(this); }
    observe() {}
    disconnect() { resizeObservers.delete(this); }
  };
  const originalRect = dom.window.HTMLElement.prototype.getBoundingClientRect;
  dom.window.HTMLElement.prototype.getBoundingClientRect = function () {
    const width = this.classList.contains("contest-shelf-books") ? shelfWidth
      : this.classList.contains("contest-book-slot") ? 168 : null;
    return width === null ? originalRect.call(this) : { width, height: 218.4, top: 0, left: 0, right: width, bottom: 218.4 };
  };
  const originalStyle = dom.window.getComputedStyle.bind(dom.window);
  dom.window.getComputedStyle = (element) => element.classList.contains("contest-shelf-books")
    ? { columnGap: "9px" } : originalStyle(element);
  dom.window.HTMLElement.prototype.attachEvent ??= () => {};
  dom.window.HTMLElement.prototype.detachEvent ??= () => {};
  const globals = {
    window: dom.window,
    document: dom.window.document,
    navigator: dom.window.navigator,
    HTMLElement: dom.window.HTMLElement,
    HTMLAnchorElement: dom.window.HTMLAnchorElement,
    HTMLImageElement: dom.window.HTMLImageElement,
    DOMParser: dom.window.DOMParser,
    NodeFilter: dom.window.NodeFilter,
  };
  const previous = Object.fromEntries(
    Object.keys(globals).map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]),
  );
  for (const [key, value] of Object.entries(globals)) {
    Object.defineProperty(globalThis, key, { configurable: true, value, writable: true });
  }
  const { clearMocks, mockIPC } = await import("@tauri-apps/api/mocks");
  mockIPC((command, args) => {
    if (command === "learning_calendar") return calendarIpc ? calendarIpc(args) : { today: "2026-08-11", entries: [] };
    if (command === "contest_participations") return participationIpc ? participationIpc(args) : [];
    if (command === "independent_problems") return independentIpc ? independentIpc(args) : [];
    if (command === "vault_manager_status") return managerFixture;
    if (command === "plugin:event|listen") return 1;
    if (command === "plugin:event|unlisten") return null;
    return ipc(command, args);
  });
  const { App } = await moduleRunner.import("/src/app/App.tsx");
  const root = createRoot(dom.window.document.getElementById("root"));
  const app = React.createElement(App);
  await act(async () => root.render(strict ? React.createElement(React.StrictMode, null, app) : app));
  await settle();
  return {
    resizeShelf: async (width) => {
      shelfWidth = width;
      await act(async () => {
        for (const observer of resizeObservers) observer.callback([]);
      });
      await settle();
    },
    document: dom.window.document,
    window: dom.window,
    cleanup: async () => {
      await act(async () => root.unmount());
      clearMocks();
      for (const [key, descriptor] of Object.entries(previous)) {
        if (descriptor) Object.defineProperty(globalThis, key, descriptor);
        else Reflect.deleteProperty(globalThis, key);
      }
      dom.window.close();
    },
  };
}


test("Contest Library D2-A preserves full-mode identity navigation and keeps overflow accessible", { concurrency: false }, async () => {
  const items = Array.from({ length: 13 }, (_, index) => ({
    contestId: 1979 + index,
    title: index === 0 ? "Codeforces Round 951 (Div. 2)" : index === 1 ? "Educational Codeforces Round 166" : `Codeforces Round ${953 + index}`,
    importStatus: index === 12 ? "incomplete" : "complete",
    problemCount: 6 + index,
    missingSnapshotCount: index === 12 ? 1 : 0,
    archived: false,
  }));
  const retryCalls = [];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_contests") return items;
    if (command === "import_codeforces_contest") {
      retryCalls.push(args.input.contestUrl);
      return { importStatus: "incomplete", missingSnapshotProblems: ["A"], failedSnapshotProblems: [] };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const books = [...view.document.querySelectorAll("button.contest-book")];
    const cabinet = view.document.querySelector('[aria-label="三层比赛收藏柜"]');
    assert.ok(cabinet);
    assert.match(view.document.querySelector(".contest-cabinet-prototype__heading")?.textContent, /13 场比赛/);
    assert.equal(view.document.querySelectorAll(".contest-cabinet__shell-piece").length, 6);
    assert.equal(view.document.querySelectorAll(".contest-cabinet__shell-piece[alt='']").length, 6);
    assert.equal(view.document.querySelector(".contest-cabinet__shell")?.getAttribute("aria-hidden"), "true");
    assert.equal(view.document.querySelector(".contest-cabinet__shell-piece--left")?.getAttribute("src")?.endsWith("contest-cabinet-left.png"), true);
    assert.equal(view.document.querySelector(".contest-cabinet__shell-piece--center")?.getAttribute("src")?.endsWith("contest-cabinet-center.png"), true);
    assert.equal(view.document.querySelector(".contest-cabinet__shell-piece--right")?.getAttribute("src")?.endsWith("contest-cabinet-right.png"), true);
    assert.equal(view.document.querySelectorAll(".contest-shelf__foreground").length, 6);
    assert.equal(view.document.querySelectorAll(".contest-shelf__foreground[alt='']").length, 6);
    assert.equal(view.document.querySelectorAll(".contest-shelf__foreground")[0]?.getAttribute("src")?.endsWith("contest-cabinet-shelf-foreground-1.png"), true);
    assert.equal(view.document.querySelectorAll(".contest-shelf__foreground")[1]?.getAttribute("src")?.endsWith("contest-cabinet-shelf-foreground-2.png"), true);
    assert.equal(view.document.querySelectorAll(".contest-shelf__foreground")[2]?.getAttribute("src")?.endsWith("contest-cabinet-shelf-foreground-3.png"), true);
    assert.ok(view.document.querySelector(".contest-cabinet__overlay"));
    assert.ok(cabinet.querySelector(".contest-cabinet__cornice"));
    assert.equal(cabinet.querySelectorAll(".contest-cabinet__stile").length, 2);
    assert.ok(cabinet.querySelector(".contest-cabinet__plinth"));
    const tiers = [...cabinet.querySelectorAll(".contest-cabinet__tier")];
    assert.equal(tiers.length, 3);
    assert.deepEqual(tiers.map((tier) => tier.querySelectorAll("button.contest-book").length), [4, 4, 4]);
    assert.equal(cabinet.querySelectorAll(".contest-cabinet__back").length, 3);
    assert.equal(cabinet.querySelectorAll(".contest-shelf").length, 3);
    assert.equal(cabinet.querySelectorAll(".contest-shelf__bottom-shadow").length, 3);
    assert.equal(books.length, 13);
    const slots = [...cabinet.querySelectorAll(".contest-book-slot")];
    assert.equal(slots.length, 12);
    assert.equal(cabinet.querySelectorAll(".contest-display-stand--rear").length, 12);
    assert.equal(cabinet.querySelectorAll(".contest-display-stand--front").length, 12);
    assert.ok(slots.every((slot) => slot.querySelectorAll(".contest-display-stand").length === 2));
    assert.deepEqual(tiers.map((tier) => [...tier.querySelectorAll(".contest-book-slot")].map((slot) => slot.dataset.logicalColumn)), [[
      "1", "2", "3", "4"
    ], [
      "1", "2", "3", "4"
    ], [
      "1", "2", "3", "4"
    ]]);
    assert.equal(books[0].dataset.contestId, "1979");
    assert.equal(tiers[1].querySelector("button.contest-book")?.dataset.contestId, "1983");
    assert.equal(tiers[2].querySelector("button.contest-book")?.dataset.contestId, "1987");
    assert.equal(books[0].getAttribute("type"), "button");
    const shell = books[0].querySelector(".contest-book__shell");
    assert.ok(shell);
    assert.equal(shell.getAttribute("alt"), "");
    assert.equal(shell.getAttribute("aria-hidden"), "true");
    assert.equal(shell.getAttribute("src"), "dom-shells-asset://../assets/contest-book-shell.png");
    assert.ok(books[0].querySelector(".contest-book__volume"));
    assert.ok(books[0].querySelector(".contest-book__spine"));
    assert.ok(books[0].querySelector(".contest-book__cover"));
    assert.ok(books[0].querySelector(".contest-book__content"));
    assert.ok(books[0].querySelector(".contest-book__masthead"));
    assert.ok(books[0].querySelector(".contest-book__footer"));
    assert.equal(books[0].querySelector(".contest-book__hinge"), null);
    assert.equal(books[0].querySelector(".contest-book__collection")?.textContent, "Codeforces");
    assert.equal(books[0].querySelector(".contest-book__series")?.textContent, "场次系列");
    assert.equal(books[0].querySelector(".contest-book__round-label")?.textContent, "场次");
    assert.equal(books[0].querySelector(".contest-book__round-number")?.textContent, "951");
    assert.equal(books[0].querySelector(".contest-book__subtitle")?.textContent, "Div. 2");
    assert.equal(books[0].querySelector(".contest-book__identity")?.textContent, "CF 1979");
    assert.equal(books[1].querySelector(".contest-book__series")?.textContent, "教育系列");
    assert.equal(books[1].querySelector(".contest-book__round-number")?.textContent, "166");
    assert.equal(view.document.querySelector(".contest-cabinet-pager"), null);
    const cabinets = [...view.document.querySelectorAll(".contest-cabinet--asset")];
    assert.equal(cabinets.length, 2);
    assert.deepEqual([...cabinets[1].querySelectorAll(".contest-cabinet__tier")].map((tier) => tier.querySelectorAll("button.contest-book").length), [1, 0, 0]);
    assert.equal(view.document.querySelector(".contest-library-remainder"), null);
    assert.ok(view.document.querySelector('[aria-label="比赛库导航"]'));
    assert.ok(view.document.querySelector(".contest-import-form"));

    const retry = view.document.querySelector(".contest-shelf-maintenance button");
    await act(async () => retry.click()); await settle();
    assert.equal(view.window.location.pathname, "/contests");
    assert.deepEqual(retryCalls, ["https://codeforces.com/contest/1991"]);

    for (const position of [0, 3, 7, 11]) {
      const item = items[position];
      const book = [...view.document.querySelectorAll("button.contest-book")][position];
      assert.equal(book.dataset.contestId, String(item.contestId));
      assert.equal(book.getAttribute("aria-label"), `打开比赛“${item.title}”`);
      await act(async () => book.click()); await settle();
      assert.equal(view.window.location.pathname, `/contests/${item.contestId}`);
      await act(async () => {
        view.window.history.pushState(null, "", "/contests");
        view.window.dispatchEvent(new view.window.PopStateEvent("popstate"));
      });
      await settle();
    }

    const remainderLink = [...view.document.querySelectorAll("button.contest-book")][12];
    await act(async () => remainderLink.click()); await settle();
    assert.equal(view.window.location.pathname, "/contests/1991");
  } finally { await view.cleanup(); }
});

test("Contest Library D2-A keeps one real contest in the first position of a complete cabinet", { concurrency: false }, async () => {
  const item = { contestId: 2256, title: "Codeforces Round 1116 (Div. 2)", importStatus: "complete", problemCount: 7, missingSnapshotCount: 0, archived: false };
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_contests") return [item];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const tiers = [...view.document.querySelectorAll(".contest-cabinet__tier")];
    assert.equal(tiers.length, 3);
    assert.deepEqual(tiers.map((tier) => tier.querySelectorAll("button.contest-book").length), [1, 0, 0]);
    assert.equal(tiers[0].querySelector("button.contest-book")?.dataset.contestId, "2256");
    assert.equal(view.document.querySelectorAll("button.contest-book").length, 1);
    assert.equal(tiers[0].querySelectorAll(".contest-book-slot").length, 1);
    assert.equal(tiers[0].querySelectorAll(".contest-display-stand").length, 2);
    assert.equal(tiers[0].querySelector(".contest-display-stand--rear")?.getAttribute("src")?.endsWith("contest-display-stand-back.png"), true);
    assert.equal(tiers[0].querySelector(".contest-display-stand--front")?.getAttribute("src")?.endsWith("contest-display-stand-front.png"), true);
    assert.equal(tiers[1].querySelector(".contest-display-stand"), null);
    assert.equal(tiers[2].querySelector(".contest-display-stand"), null);
  } finally { await view.cleanup(); }
});

test("Contest Library reflows all books on resize and fills three tiers before another cabinet", { concurrency: false }, async () => {
  const items = Array.from({ length: 16 }, (_, index) => ({
    contestId: 4101 + index, title: `Codeforces Round ${1200 + index}`,
    importStatus: "complete", problemCount: 1, missingSnapshotCount: 0, archived: false,
  }));
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_contests") return items;
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    // Include exact-fit and just-below boundaries, shrink and grow on the same mounted view.
    for (const [width, expected] of [
      [876, [[5, 5, 5], [1, 0, 0]]],
      [875, [[4, 4, 4], [4, 0, 0]]],
      [522, [[3, 3, 3], [3, 3, 1]]],
      [345, [[2, 2, 2], [2, 2, 2], [2, 2, 0]]],
      [168, [[1, 1, 1], [1, 1, 1], [1, 1, 1], [1, 1, 1], [1, 1, 1], [1, 0, 0]]],
      [1053, [[6, 6, 4]]],
      [876, [[5, 5, 5], [1, 0, 0]]],
    ]) {
      await view.resizeShelf(width);
      const cabinets = [...view.document.querySelectorAll(".contest-cabinet--asset")];
      assert.deepEqual(cabinets.map((cabinet) => [...cabinet.querySelectorAll(".contest-cabinet__tier")].map((tier) => tier.querySelectorAll("button.contest-book").length)), expected);
      assert.deepEqual([...view.document.querySelectorAll("button.contest-book")].map((book) => Number(book.dataset.contestId)), items.map((item) => item.contestId));
      assert.equal(view.document.querySelector(".contest-cabinet-pager"), null);
      assert.equal(view.document.querySelector('[data-compact-active="false"]'), null);
    }
    const lastBook = [...view.document.querySelectorAll("button.contest-book")].at(-1);
    await act(async () => lastBook.click()); await settle();
    assert.equal(view.window.location.pathname, "/contests/4116");
  } finally { await view.cleanup(); }
});

test("Contest Library D2-A preserves all three tiers for an empty filtered result", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_contests") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const cabinet = view.document.querySelector('[aria-label="三层比赛收藏柜"]');
    assert.ok(cabinet);
    assert.equal(cabinet.querySelectorAll(".contest-cabinet__tier").length, 3);
    assert.equal(cabinet.querySelectorAll("button.contest-book").length, 0);
    assert.equal(cabinet.querySelector(".contest-cabinet__empty")?.textContent, "当前视图没有比赛");
    assert.equal(view.document.querySelector(".empty-state"), null);
    assert.equal(view.document.querySelector('[aria-label="紧凑收藏柜列导航"]'), null);
  } finally { await view.cleanup(); }
});

test("Contest Library creates taxonomy from peer-level chips and preserves persisted IDs", { concurrency: false }, async () => {
  const calls = [];
  let families = [{ familyId: 1, displayName: "Codeforces" }];
  let series = [];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return families;
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_list_series") return series;
    if (command === "contest_library_list_years") return [];
    if (command === "contest_library_create_family") { calls.push([command, args.input]); families = [...families, { familyId: 2, displayName: args.input.displayName.trim() }]; return families[1]; }
    if (command === "contest_library_rename_family") { calls.push([command, args.input]); families = families.map((item) => item.familyId === args.input.familyId ? { ...item, displayName: args.input.displayName } : item); return families.find((item) => item.familyId === args.input.familyId); }
    if (command === "contest_library_create_series") { calls.push([command, args.input]); series = [{ seriesId: 21, familyId: args.input.familyId, displayName: args.input.displayName }]; return series[0]; }
    if (command === "contest_library_rename_series") { calls.push([command, args.input]); series = [{ ...series[0], displayName: args.input.displayName }]; return series[0]; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const createFamilyButton = view.document.querySelector('button.filter-option--create[aria-label="创建分类"]');
    await act(async () => createFamilyButton.click()); await settle();
    const familyDialog = view.document.querySelector('[role="dialog"][aria-modal="true"]');
    const familyForm = familyDialog.querySelector(".contest-taxonomy-create-form");
    assert.equal(view.document.querySelectorAll(".contest-taxonomy-create-form").length, 1);
    assert.equal(view.document.activeElement, familyForm.querySelector("input"));
    const familyInput = familyForm.querySelector("input");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(familyInput, " User Family "); familyInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await settle();
    await act(async () => familyForm.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    await settle();
    assert.equal(view.document.querySelector('[role="dialog"]'), null);
    assert.equal(view.document.activeElement, createFamilyButton);
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "User Family"));
    assert.equal(calls.find(([name]) => name === "contest_library_create_family")[1].displayName, " User Family ");
    const selected = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "User Family");
    await act(async () => selected.click()); await settle();
    const createSeriesButton = view.document.querySelector('button.filter-option--create[aria-label="创建系列"]');
    await act(async () => createSeriesButton.click()); await settle();
    const seriesForm = view.document.querySelector(".contest-taxonomy-create-form");
    assert.equal(view.document.querySelectorAll(".contest-taxonomy-create-form").length, 1);
    assert.equal(seriesForm.querySelector("select").value, "2");
    assert.equal(view.document.activeElement, seriesForm.querySelector("select"));
    const seriesInput = seriesForm.querySelector("input");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(seriesInput, "Rounds"); seriesInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await settle();
    await act(async () => seriesForm.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    await settle();
    assert.equal(view.document.activeElement, createSeriesButton);
    assert.ok(calls.some(([name]) => name === "contest_library_create_series"));
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Rounds"));
    const familyManage = view.document.querySelector('button[aria-label="管理分类"]');
    await act(async () => familyManage.click()); await settle();
    let managementDialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(managementDialog.getAttribute("aria-modal"), "true");
    assert.equal(managementDialog.querySelector(".taxonomy-management-current strong").textContent, "User Family");
    const familyRename = [...managementDialog.querySelectorAll("button")].find((button) => button.textContent === "重命名");
    await act(async () => familyRename.click()); await settle();
    const familyRenameForm = view.document.querySelector(".contest-taxonomy-management--rename");
    const familyRenameInput = familyRenameForm.querySelector("input");
    assert.equal(view.document.activeElement, familyRenameInput);
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(familyRenameInput, "Renamed Family"); familyRenameInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await settle();
    await act(async () => familyRenameForm.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.deepEqual(calls.find(([name]) => name === "contest_library_rename_family")[1], { familyId: 2, displayName: "Renamed Family" });
    assert.equal([...view.document.querySelectorAll("button.filter-option--selected")].some((button) => button.textContent === "Renamed Family"), true);
    assert.equal(view.document.activeElement.textContent, "重命名");
    await act(async () => [...managementDialog.querySelectorAll("button")].find((button) => button.textContent === "取消").click()); await settle();
    const seriesManage = view.document.querySelector('button[aria-label="管理系列"]');
    await act(async () => seriesManage.click()); await settle();
    managementDialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(managementDialog.querySelector("select").value, "2");
    const managedSeriesChip = [...managementDialog.querySelectorAll(".taxonomy-management-option")].find((button) => button.textContent === "Rounds");
    await act(async () => managedSeriesChip.click()); await settle();
    const seriesRename = [...managementDialog.querySelectorAll("button")].find((button) => button.textContent === "重命名");
    await act(async () => seriesRename.click()); await settle();
    const seriesRenameForm = view.document.querySelector(".contest-taxonomy-management--rename");
    assert.match(seriesRenameForm.textContent, /Renamed Family/);
    const seriesRenameInput = seriesRenameForm.querySelector("input");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(seriesRenameInput, "Renamed Series"); seriesRenameInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await settle();
    await act(async () => seriesRenameForm.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.deepEqual(calls.find(([name]) => name === "contest_library_rename_series")[1], { seriesId: 21, displayName: "Renamed Series" });
  } finally { await view.cleanup(); }
});

test("Contest Library peer groups expose accessible create dialogs and explicit Series context", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Codeforces" }];
    if (command === "contest_library_list_series") return [];
    if (command === "contest_library_list_years") return [];
    if (command === "contest_library_list_contests") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const groups = [...view.document.querySelectorAll(".taxonomy-filter-group")];
    const [createFamilyButton, createSeriesButton, createYearButton] = [...view.document.querySelectorAll("button.filter-option--create")];
    assert.equal(groups.length, 3);
    assert.deepEqual([createFamilyButton, createSeriesButton, createYearButton].map((button) => button.getAttribute("aria-label")), ["创建分类", "创建系列", "创建年份"]);
    assert.deepEqual([...view.document.querySelectorAll(".taxonomy-manage-trigger")].map((button) => button.getAttribute("aria-label")), ["管理分类", "管理系列", "管理年份"]);
    assert.equal(view.document.querySelector(".contest-library-create-panel"), null);
    assert.equal(view.document.querySelector(".management-list"), null);
    assert.equal(view.document.querySelector(".management-list__row"), null);

    await act(async () => createSeriesButton.click()); await settle();
    let dialog = view.document.querySelector('[role="dialog"]');
    const familySelect = dialog.querySelector("select");
    assert.equal(familySelect.value, "");
    assert.equal(view.document.activeElement, familySelect);
    assert.equal(dialog.querySelector('button[type="submit"]').disabled, true);
    const cancel = dialog.querySelector("button.secondary-action");
    await act(async () => cancel.click()); await settle();
    assert.equal(view.document.querySelector('[role="dialog"]'), null);
    assert.equal(view.document.activeElement, createSeriesButton);

    await act(async () => createFamilyButton.click()); await settle();
    dialog = view.document.querySelector('[role="dialog"]');
    assert.equal(view.document.activeElement, dialog.querySelector("input"));
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    await settle();
    assert.equal(view.document.querySelector('[role="dialog"]'), null);
    assert.equal(view.document.activeElement, createFamilyButton);

    await act(async () => createYearButton.click()); await settle();
    dialog = view.document.querySelector('[role="dialog"]');
    assert.equal(dialog.querySelector("input")?.getAttribute("type"), "number");
    const last = dialog.querySelector('button[type="submit"]');
    last.focus();
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement, dialog.querySelector("input"));
  } finally { await view.cleanup(); }
});

test("Contest Add panel keeps quick import visible, guarded, and refreshes through the existing payload", { concurrency: false }, async () => {
  const importCalls = [];
  let listCalls = 0;
  let finishImport;
  const pendingImport = new Promise((resolve) => { finishImport = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_contests") { listCalls += 1; return []; }
    if (command === "preview_problem_library") return { title: "Round 2256", problems: ["A", "B", "C", "D", "E1", "E2", "F"].map((index,i) => ({ problemId: i+1, index, title: "Problem " + index, alreadyAdded: false })) };
    if (command === "import_codeforces_contest") { importCalls.push(args.input); return pendingImport; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const baselineListCalls = listCalls;
    const panel = view.document.querySelector(".contest-add-panel");
    const form = panel.querySelector(".contest-import-form");
    const input = form.querySelector("input");
    const submit = form.querySelector('button[type="submit"]');
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, "https://codeforces.com/contest/2256");
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await act(async () => form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    assert.equal(importCalls.length, 1);
    assert.deepEqual(importCalls[0], { contestUrl: "https://codeforces.com/contest/2256" });
    assert.equal(submit.disabled, true);
    assert.equal(submit.textContent, "正在导入…");
    await act(async () => form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    assert.equal(importCalls.length, 1);
    await act(async () => finishImport({ contestId: 2256, importStatus: "complete", missingSnapshotProblems: [], failedSnapshotProblems: [] })); await settle();
    assert.match(panel.querySelector('[aria-live="polite"]').textContent, /比赛已导入/);
    assert.ok(listCalls > baselineListCalls);
    assert.equal(panel.querySelectorAll('[aria-live="polite"]').length, 1);
    const selection = view.document.querySelector('[aria-label="加入我的题库"]');
    assert.equal(selection.querySelectorAll('input[type="checkbox"]').length, 7);
    assert.ok([...selection.querySelectorAll('input[type="checkbox"]')].every((input) => !input.checked));
  } finally { await view.cleanup(); }
});

test("Contest Library taxonomy management keeps selection independent and gives Series an explicit Category context", { concurrency: false }, async () => {
  const calls = [];
  const families = [{ familyId: 1, displayName: "Codeforces" }, { familyId: 2, displayName: "XCPC" }];
  const seriesByFamily = new Map([
    [1, [{ seriesId: 11, familyId: 1, displayName: "Div.1" }, { seriesId: 12, familyId: 1, displayName: "Div.2" }]],
    [2, [{ seriesId: 21, familyId: 2, displayName: "Regional" }]],
  ]);
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return families;
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_series") return seriesByFamily.get(args.input.familyId) ?? [];
    if (command === "contest_library_list_years") return [];
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_delete_series") { calls.push([command, args.input]); return null; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const seriesManage = view.document.querySelector('button[aria-label="管理系列"]');
    await act(async () => seriesManage.click()); await settle();
    let dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    let context = dialog.querySelector("select");
    assert.equal(context.value, "");
    assert.equal(view.document.activeElement, context);
    assert.match(dialog.textContent, /请先选择分类/);
    const cancelManagement = [...dialog.querySelectorAll("button")].find((button) => button.textContent === "取消");
    cancelManagement.focus();
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement, context);
    assert.equal([...view.document.querySelectorAll(".taxonomy-filter-group")][0].querySelector(".filter-option--selected").textContent, "全部比赛");

    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(context, "1"); context.dispatchEvent(new view.window.Event("change", { bubbles: true })); }); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    const divOne = [...dialog.querySelectorAll(".taxonomy-management-option")].find((button) => button.textContent === "Div.1");
    await act(async () => divOne.click()); await settle();
    assert.equal(dialog.querySelector(".taxonomy-management-current strong").textContent, "Div.1");
    assert.equal([...view.document.querySelectorAll(".taxonomy-filter-group")][0].querySelector(".filter-option--selected").textContent, "全部比赛");
    assert.equal([...view.document.querySelectorAll(".taxonomy-filter-group")][1].querySelector(".filter-option--selected").textContent, "全部系列");

    context = dialog.querySelector("select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(context, "2"); context.dispatchEvent(new view.window.Event("change", { bubbles: true })); }); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(dialog.querySelector(".taxonomy-management-current strong").textContent, "尚未选择");
    assert.equal(dialog.querySelector('.taxonomy-management-option[aria-pressed="true"]'), null);

    context = dialog.querySelector("select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(context, "1"); context.dispatchEvent(new view.window.Event("change", { bubbles: true })); }); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    await act(async () => [...dialog.querySelectorAll(".taxonomy-management-option")].find((button) => button.textContent === "Div.1").click()); await settle();
    await act(async () => dialog.querySelector(".danger-action").click()); await settle();
    const deleteDialog = view.document.querySelector('[role="dialog"]');
    assert.equal(view.document.querySelector(".contest-taxonomy-management-dialog"), null);
    assert.match(deleteDialog.textContent, /当前系列Div\.1/);
    assert.match(deleteDialog.textContent, /所属分类Codeforces/);
    const replacement = deleteDialog.querySelector("select");
    assert.deepEqual([...replacement.options].map((option) => option.textContent), ["不设替代项（仅适用于未使用对象）", "Div.2"]);
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(replacement, "12"); replacement.dispatchEvent(new view.window.Event("change", { bubbles: true })); }); await settle();
    await act(async () => [...deleteDialog.querySelectorAll("button")].find((button) => button.textContent === "确认删除").click()); await settle();
    assert.deepEqual(calls[0], ["contest_library_delete_series", { seriesId: 11, replacementSeriesId: 12 }]);

    const xcpc = [...view.document.querySelectorAll(".taxonomy-filter-group:first-child .filter-option")].find((button) => button.textContent === "XCPC");
    await act(async () => xcpc.click()); await settle();
    const regional = [...view.document.querySelectorAll(".taxonomy-filter-group:nth-child(2) .filter-option")].find((button) => button.textContent === "Regional");
    await act(async () => regional.click()); await settle();
    await act(async () => seriesManage.click()); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(dialog.querySelector("select").value, "2");
    assert.equal(dialog.querySelector(".taxonomy-management-current strong").textContent, "Regional");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true }))); await settle();
    assert.equal(view.document.querySelector(".contest-taxonomy-management-dialog"), null);
    assert.equal(view.document.activeElement, seriesManage);
  } finally { await view.cleanup(); }
});

test("Contest Library Series management ignores stale Category rejection", { concurrency: false }, async () => {
  let rejectA;
  let resolveB;
  const seriesA = new Promise((resolve, reject) => { rejectA = reject; });
  const seriesB = new Promise((resolve) => { resolveB = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Category A" }, { familyId: 2, displayName: "Category B" }];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_series") return args.input.familyId === 1 ? seriesA : seriesB;
    if (command === "contest_library_list_contests") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const pageCategory = [...view.document.querySelectorAll(".taxonomy-filter-group")][0].querySelector(".filter-option--selected").textContent;
    const pageSeries = [...view.document.querySelectorAll(".taxonomy-filter-group")][1].querySelector(".filter-option--selected").textContent;
    const seriesManage = view.document.querySelectorAll(".taxonomy-manage-trigger")[1];
    await act(async () => seriesManage.click()); await settle();
    let dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    let context = dialog.querySelector("select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(context, "1"); context.dispatchEvent(new view.window.Event("change", { bubbles: true })); });
    context = view.document.querySelector(".contest-taxonomy-management-dialog select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(context, "2"); context.dispatchEvent(new view.window.Event("change", { bubbles: true })); });
    await act(async () => resolveB([{ seriesId: 22, familyId: 2, displayName: "Series B" }])); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(dialog.querySelector("select").value, "2");
    assert.ok([...dialog.querySelectorAll(".taxonomy-management-option")].some((button) => button.textContent === "Series B"));
    await act(async () => rejectA(new Error("stale Category A failure"))); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(dialog.querySelector("select").value, "2");
    assert.ok([...dialog.querySelectorAll(".taxonomy-management-option")].some((button) => button.textContent === "Series B"));
    assert.equal(dialog.querySelector(".error-message")?.textContent ?? null, null);
    assert.equal([...view.document.querySelectorAll(".taxonomy-filter-group")][0].querySelector(".filter-option--selected").textContent, pageCategory);
    assert.equal([...view.document.querySelectorAll(".taxonomy-filter-group")][1].querySelector(".filter-option--selected").textContent, pageSeries);
  } finally { await view.cleanup(); }
});

test("Contest Library Series management shows the current Category rejection", { concurrency: false }, async () => {
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Category A" }];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_series") return Promise.reject(new Error(`Category ${args.input.familyId} unavailable`));
    if (command === "contest_library_list_contests") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const seriesManage = view.document.querySelectorAll(".taxonomy-manage-trigger")[1];
    await act(async () => seriesManage.click()); await settle();
    const context = view.document.querySelector(".contest-taxonomy-management-dialog select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(context, "1"); context.dispatchEvent(new view.window.Event("change", { bubbles: true })); }); await settle();
    assert.match(view.document.querySelector(".contest-taxonomy-management-dialog .error-message").textContent, /比赛库操作失败/);
  } finally { await view.cleanup(); }
});

test("Contest Library management exposes exact empty states without repair entities", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_contests") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    await act(async () => view.document.querySelector('button[aria-label="管理分类"]').click()); await settle();
    assert.match(view.document.querySelector(".contest-taxonomy-management-dialog").textContent, /暂无可编辑的分类/);
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true }))); await settle();
    await act(async () => view.document.querySelector('button[aria-label="管理年份"]').click()); await settle();
    assert.match(view.document.querySelector(".contest-taxonomy-management-dialog").textContent, /暂无可编辑的年份/);
    assert.doesNotMatch(view.document.body.textContent, /Unknown|Unassigned|未知年份|未分配系列/);
  } finally { await view.cleanup(); }
});

test("Contest Library rename blocks duplicate submits and retains failed input", { concurrency: false }, async () => {
  let rejectRename;
  let renameCalls = 0;
  const pendingRename = new Promise((_resolve, reject) => { rejectRename = reject; });
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Codeforces" }];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_rename_family") { renameCalls += 1; return pendingRename; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    await act(async () => view.document.querySelector('button[aria-label="管理分类"]').click()); await settle();
    let dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    await act(async () => dialog.querySelector(".taxonomy-management-option").click()); await settle();
    await act(async () => [...dialog.querySelectorAll("button")].find((button) => button.textContent === "重命名").click()); await settle();
    const form = view.document.querySelector(".contest-taxonomy-management--rename");
    const input = form.querySelector("input");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, "Persistent rename"); input.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await act(async () => {
      form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
      form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
    });
    assert.equal(renameCalls, 1);
    assert.equal(input.disabled, true);
    await act(async () => rejectRename(new Error("duplicate_family_name"))); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(dialog.querySelector("input").value, "Persistent rename");
    assert.ok(dialog.querySelector(".error-message"));
  } finally { await view.cleanup(); }
});

test("Contest Library create dialog blocks duplicate submits and preserves failed input", { concurrency: false }, async () => {
  let rejectCreate;
  let createCalls = 0;
  const pendingCreate = new Promise((_resolve, reject) => { rejectCreate = reject; });
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_create_family") { createCalls += 1; return pendingCreate; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const trigger = view.document.querySelector('button.filter-option--create[aria-label="创建分类"]');
    await act(async () => trigger.click()); await settle();
    const form = view.document.querySelector(".contest-taxonomy-create-form");
    const input = form.querySelector("input");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, "Persistent category");
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await act(async () => {
      form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
      form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
    });
    assert.equal(createCalls, 1);
    assert.equal(input.disabled, true);
    await act(async () => rejectCreate(new Error("duplicate_family_name")));
    await settle();
    assert.equal(view.document.querySelector(".contest-taxonomy-create-form input").value, "Persistent category");
    assert.ok(view.document.querySelector(".contest-taxonomy-create-form .error-message"));
  } finally { await view.cleanup(); }
});

test("Contest Library category deletion discloses series identity preservation before confirmation", { concurrency: false }, async () => {
  const calls = [];
  let families = [{ familyId: 1, displayName: "Family A" }, { familyId: 2, displayName: "Family B" }];
  const series = [{ seriesId: 11, familyId: 1, displayName: "Series A" }];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return families;
    if (command === "contest_library_list_series") return args.input.familyId === 1 ? series : [];
    if (command === "contest_library_list_years") return [];
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_delete_family") { calls.push([command, args.input]); families = families.filter((item) => item.familyId !== args.input.familyId); return null; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const family = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Family A");
    await act(async () => family.click()); await settle();
    await act(async () => view.document.querySelector('button[aria-label="管理分类"]').click()); await settle();
    const managementDialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(managementDialog.querySelector(".taxonomy-management-current strong").textContent, "Family A");
    const deleteButton = [...managementDialog.querySelectorAll("button")].find((button) => button.textContent === "删除");
    await act(async () => deleteButton.click()); await settle();
    assert.match(view.document.body.textContent, /全部 1 个系列会保留身份并迁移到替代分类/);
    const dialog = view.document.querySelector('[role="dialog"]');
    assert.ok(dialog);
    assert.equal(dialog.getAttribute("aria-labelledby"), "contest-taxonomy-delete-category-title");
    assert.match(dialog.getAttribute("aria-describedby"), /contest-taxonomy-delete-category-description/);
    assert.match(dialog.textContent, /当前分类Family A/);
    assert.match(dialog.textContent, /删除影响/);
    assert.match(dialog.textContent, /风险提示/);
    assert.equal(dialog.querySelectorAll("select option").length, 2);
    assert.equal(calls.length, 0);
    const replacement = dialog.querySelector("select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(replacement, "2"); replacement.dispatchEvent(new view.window.Event("change", { bubbles: true })); });
    const confirm = [...dialog.querySelectorAll("button")].find((button) => button.textContent === "确认删除");
    await act(async () => confirm.click()); await settle();
    assert.deepEqual(calls[0][1], { familyId: 1, replacementFamilyId: 2 });
  } finally { await view.cleanup(); }
});

test("Contest taxonomy delete retains validation state and blocks duplicate busy actions", { concurrency: false }, async () => {
  let families = [{ familyId: 1, displayName: "Family A" }, { familyId: 2, displayName: "Family B" }, { familyId: 3, displayName: "Family C" }];
  let deleteCalls = 0;
  let resolveDelete;
  const pendingDelete = new Promise((resolve) => { resolveDelete = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return families;
    if (command === "contest_library_list_series") return [];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_delete_family") {
      deleteCalls += 1;
      if (args.input.replacementFamilyId === null) throw new Error("family_in_use");
      if (args.input.replacementFamilyId === 2) throw new Error("family_in_use");
      families = families.filter((item) => item.familyId !== args.input.familyId);
      return pendingDelete;
    }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const manageTrigger = view.document.querySelector('button[aria-label="管理分类"]');
    await act(async () => manageTrigger.click()); await settle();
    let managementDialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    await act(async () => managementDialog.querySelector(".taxonomy-management-option").click()); await settle();
    await act(async () => managementDialog.querySelector(".danger-action").click()); await settle();
    let dialog = view.document.querySelector(".contest-taxonomy-delete-dialog");
    const select = dialog.querySelector("select");
    assert.equal(view.document.activeElement, select);
    const confirm = dialog.querySelector(".danger-action");
    confirm.focus();
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement, select);

    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true }))); await settle();
    managementDialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.ok(managementDialog);
    assert.equal(managementDialog.querySelector(".taxonomy-management-current strong").textContent, "Family A");
    await act(async () => managementDialog.querySelector(".danger-action").click()); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-delete-dialog");

    await act(async () => dialog.querySelector(".danger-action").click()); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-delete-dialog");
    assert.equal(deleteCalls, 1);
    assert.match(dialog.textContent, /当前对象正在使用/);
    assert.ok(dialog.querySelector('[role="alert"]'));
    assert.equal(dialog.querySelector(".danger-action").disabled, true);
    assert.equal(dialog.querySelector("select").value, "");

    const replacement = dialog.querySelector("select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(replacement, "2"); replacement.dispatchEvent(new view.window.Event("change", { bubbles: true })); }); await settle();
    await act(async () => dialog.querySelector(".danger-action").click()); await settle();
    dialog = view.document.querySelector(".contest-taxonomy-delete-dialog");
    assert.match(dialog.textContent, /替代分类中存在同名系列/);
    assert.equal(dialog.querySelector("select").value, "2");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(dialog.querySelector("select"), "3"); dialog.querySelector("select").dispatchEvent(new view.window.Event("change", { bubbles: true })); }); await settle();
    const readyConfirm = view.document.querySelector(".contest-taxonomy-delete-dialog .danger-action");
    await act(async () => { readyConfirm.click(); readyConfirm.click(); });
    assert.equal(deleteCalls, 3);
    dialog = view.document.querySelector(".contest-taxonomy-delete-dialog");
    assert.equal(dialog.querySelector(".danger-action").textContent, "删除中…");
    assert.equal(dialog.querySelector("select").disabled, true);
    assert.equal(dialog.querySelector(".secondary-action").disabled, true);
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    assert.ok(view.document.querySelector(".contest-taxonomy-delete-dialog"));
    await act(async () => resolveDelete()); await settle();
    assert.equal(view.document.querySelector(".contest-taxonomy-delete-dialog"), null);
    assert.equal(view.document.activeElement, manageTrigger);
  } finally { await view.cleanup(); }
});

test("Contest archive exposes explicit placement taxonomy for single and grouped contests", { concurrency: false }, async () => {
  const items = [
    {
      contestId: 4101,
      title: "Summer multi-campus",
      importStatus: "complete",
      problemCount: 3,
      missingSnapshotCount: 0,
      archived: false,
      placements: [
        { placementId: 1, familyId: 1, familyName: "NiuKe", seriesId: 11, seriesName: "Summer", year: 2026, ordinal: null },
        { placementId: 2, familyId: 2, familyName: "XCPC", seriesId: null, seriesName: null, year: 2025, ordinal: null },
      ],
    },
    {
      contestId: 4102,
      title: "Legacy contest",
      importStatus: "complete",
      problemCount: 2,
      missingSnapshotCount: 0,
      archived: false,
      placements: [{ placementId: 3, familyId: 3, familyName: "HangDian", seriesId: null, seriesName: null, year: null, ordinal: null }],
    },
  ];
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_contests") return items;
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const books = [...view.document.querySelectorAll("button.contest-book")];
    assert.equal(books.length, 2);
    assert.deepEqual([...books[0].querySelectorAll(".contest-book__placement")].map((node) => node.textContent.trim()), ["NiuKe \u00b7 Summer \u00b7 2026", "\u9700\u8981\u8865\u9f50"]);
    assert.equal(books[0].querySelector('[data-taxonomy-field="year"]')?.textContent, "2026");
    assert.equal(books[1].querySelector(".contest-book__placement")?.textContent.trim(), "\u9700\u8981\u8865\u9f50");
    assert.equal(books[1].querySelector('[data-taxonomy-field="series"]'), null);
    assert.equal(books[1].querySelector('[data-taxonomy-field="year"]'), null);
    const artwork = books[0].querySelector(".contest-book__shell");
    await act(async () => artwork.dispatchEvent(new view.window.Event("error")));
    await settle();
    assert.equal(books[0].querySelector(".contest-book__shell"), null);
    assert.equal(books[0].classList.contains("contest-book--asset"), false);
    assert.ok(books[0].querySelector(".contest-book__cover"));
    assert.equal(books[0].querySelectorAll(".contest-book__placement").length, 2);
  } finally { await view.cleanup(); }
});

test("Contest Library manages Year entities through stable IDs", { concurrency: false }, async () => {
  const calls = [];
  let years = [{ yearId: 1, value: 2025 }];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_year_entities") return years;
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_create_year") { const created = { yearId: 3, value: args.input.value }; calls.push([command, args.input]); years = [...years, created]; return created; }
    if (command === "contest_library_rename_year") { calls.push([command, args.input]); years = years.map((item) => item.yearId === args.input.yearId ? { ...item, value: args.input.value } : item); return years.find((item) => item.yearId === args.input.yearId); }
    if (command === "contest_library_delete_year") { calls.push([command, args.input]); years = years.filter((item) => item.yearId !== args.input.yearId); return null; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const create = view.document.querySelector('button.filter-option--create[aria-label="\u521b\u5efa\u5e74\u4efd"]');
    await act(async () => create.click()); await settle();
    const createForm = view.document.querySelector(".contest-taxonomy-create-form");
    const createInput = createForm.querySelector("input");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(createInput, "2027"); createInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await act(async () => createForm.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.deepEqual(calls.find(([name]) => name === "contest_library_create_year")[1], { value: 2027 });
    await act(async () => view.document.querySelector('button[aria-label="管理年份"]').click()); await settle();
    let managementDialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(managementDialog.querySelector(".taxonomy-management-current strong").textContent, "2027");
    await act(async () => [...managementDialog.querySelectorAll("button")].find((button) => button.textContent === "\u91cd\u547d\u540d").click()); await settle();
    const renameForm = view.document.querySelector(".contest-taxonomy-management--rename");
    const renameInput = renameForm.querySelector("input");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(renameInput, "2028"); renameInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await act(async () => renameForm.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.deepEqual(calls.find(([name]) => name === "contest_library_rename_year")[1], { yearId: 3, value: 2028 });
    managementDialog = view.document.querySelector(".contest-taxonomy-management-dialog");
    assert.equal(managementDialog.querySelector(".taxonomy-management-current strong").textContent, "2028");
    await act(async () => managementDialog.querySelector(".danger-action").click()); await settle();
    const dialog = view.document.querySelector('[role="dialog"]');
    assert.match(dialog.textContent, /当前年份2028/);
    assert.match(dialog.textContent, /未使用的年份可以不选替代项直接删除/);
    const replacement = dialog.querySelector("select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(replacement, "1"); replacement.dispatchEvent(new view.window.Event("change", { bubbles: true })); });
    await act(async () => [...dialog.querySelectorAll("button")].find((button) => button.textContent === "确认删除").click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "contest_library_delete_year")[1], { yearId: 3, replacementYearId: 1 });
  } finally { await view.cleanup(); }
});

test("Contest Library shares years across categories, reuses existing years and immediately shows unused new years", { concurrency: false }, async () => {
  let years = [{ yearId: 21, value: 2026 }];
  const created = [];
  const scopes = [];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Codeforces" }, { familyId: 2, displayName: "牛客" }];
    if (command === "contest_library_list_year_entities") return years;
    if (command === "contest_library_list_series" || command === "contest_library_list_years") return [];
    if (command === "contest_library_create_year") { const year = { yearId: 22, value: args.input.value }; created.push(year); years = [...years, year]; return year; }
    if (command === "contest_library_list_contests") { scopes.push(args.input.scope); return args.input.scope.kind === "all" ? [{ contestId: 2275, title: "Round", importStatus: "complete", problemCount: 1, missingSnapshotCount: 0, archived: false, placements: [{ placementId: 1, familyId: 1, familyName: "Codeforces", seriesId: null, seriesName: null, yearId: 21, year: 2026, ordinal: null }] }] : []; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    const yearButton = (value) => [...view.document.querySelectorAll("button.filter-option")].find((button) => button.textContent === String(value));
    const select = async (label) => { await act(async () => [...view.document.querySelectorAll("button.filter-option")].find((button) => button.textContent === label).click()); await settle(); };
    const createYear = async (value) => {
      await act(async () => view.document.querySelector('button[aria-label="创建年份"]').click()); await settle();
      const form = view.document.querySelector(".contest-taxonomy-create-form");
      const input = form.querySelector("input");
      await act(async () => { input.value = String(value); input.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
      await act(async () => form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    };
    await settle();
    assert.ok(yearButton(2026));
    await select("Codeforces");
    assert.ok(yearButton(2026));
    await select("牛客");
    assert.ok(yearButton(2026));
    await createYear(2026);
    assert.equal(created.length, 0);
    assert.equal(view.document.querySelector(".contest-taxonomy-create-form"), null);
    assert.equal(yearButton(2026).classList.contains("filter-option--selected"), true);
    assert.deepEqual(scopes.at(-1), { kind: "family", familyId: 2, series: { kind: "any" }, year: { kind: "exact", yearId: 21 } });
    await createYear(2027);
    assert.equal(created.length, 1);
    assert.ok(yearButton(2027));
    assert.equal(view.document.querySelectorAll("button.contest-book").length, 0);
    await select("全部比赛");
    assert.ok(yearButton(2027));
    assert.equal(view.document.querySelectorAll("button.contest-book").length, 1);
    await select("2027");
    assert.equal(view.document.querySelectorAll("button.contest-book").length, 0);
    await select("2026");
    assert.equal(view.document.querySelectorAll("button.contest-book").length, 1);
  } finally { await view.cleanup(); }
});

test("Contest Library filters with Category, Series, and Year IDs without repair buckets", { concurrency: false }, async () => {
  const contestCalls = [];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Category" }];
    if (command === "contest_library_list_year_entities") return [];
    if (command === "contest_library_list_series") return [{ seriesId: 11, familyId: 1, displayName: "Series" }];
    if (command === "contest_library_list_years") return [{ yearId: 21, value: 2026 }];
    if (command === "contest_library_list_contests") { contestCalls.push(args.input); return []; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    await act(async () => [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Category").click()); await settle();
    await act(async () => [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Series").click()); await settle();
    await act(async () => [...view.document.querySelectorAll("button")].find((button) => button.textContent === "2026").click()); await settle();
    assert.deepEqual(contestCalls.at(-1).scope, { kind: "family", familyId: 1, series: { kind: "exact", seriesId: 11 }, year: { kind: "exact", yearId: 21 } });
    assert.doesNotMatch(view.document.querySelector(".contest-library-navigation").textContent, /\u672a\u5206\u914d|\u672a\u77e5/);
  } finally { await view.cleanup(); }
});


test("Contest Library Retry reloads Families after the first Family request fails", { concurrency: false }, async () => {
  let familyCalls = 0;
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") {
      familyCalls += 1;
      if (familyCalls === 1) return Promise.reject(new Error("family unavailable"));
      return [{ familyId: 1, displayName: "Recovered Family" }];
    }
    if (command === "contest_library_list_contests") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const retry = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "重试");
    assert.ok(retry);
    await act(async () => retry.click()); await settle();
    assert.equal(familyCalls, 2);
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Recovered Family"));
  } finally { await view.cleanup(); }
});

test("Contest Library clears old child navigation immediately when Family changes", { concurrency: false }, async () => {
  let resolveB;
  const seriesB = new Promise((resolve) => { resolveB = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Family A" }, { familyId: 2, displayName: "Family B" }];
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_list_series") return args.input.familyId === 1 ? [{ seriesId: 11, familyId: 1, displayName: "Series A" }] : seriesB;
    if (command === "contest_library_list_years") return args.input.familyId === 1 ? [{ yearId: 15, value: 2025 }] : [];
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const familyA = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Family A");
    const familyB = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Family B");
    await act(async () => familyA.click()); await settle();
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Series A"));
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "2025"));
    await act(async () => familyB.click());
    assert.equal([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Series A"), false);
    assert.equal([...view.document.querySelectorAll("button")].some((button) => button.textContent === "2025"), false);
    await act(async () => resolveB([])); await settle();
  } finally { await view.cleanup(); }
});


test("Contest Library mutation refresh uses the latest selected scope", { concurrency: false }, async () => {
  let finishImport;
  const pendingImport = new Promise((resolve) => { finishImport = resolve; });
  const contestCalls = [];
  const item = (contestId, title) => ({ contestId, title, importStatus: "complete", problemCount: 1, missingSnapshotCount: 0, archived: false });
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Family A" }, { familyId: 2, displayName: "Family B" }];
    if (command === "contest_library_list_series") return [];
    if (command === "contest_library_list_years") return [];
    if (command === "contest_library_list_contests") {
      contestCalls.push(args.input);
      if (args.input.scope.kind === "family" && args.input.scope.familyId === 1) return [item(1001, "Contest A")];
      if (args.input.scope.kind === "family" && args.input.scope.familyId === 2) return [item(2002, "Contest B")];
      return [];
    }
    if (command === "import_codeforces_contest") return pendingImport;
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const familyA = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Family A");
    const familyB = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Family B");
    await act(async () => familyA.click()); await settle();
    const input = view.document.querySelector('input[placeholder="https://codeforces.com/contest/1979"]');
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, "https://codeforces.com/contest/1979");
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      input.closest("form").dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
    });
    await act(async () => familyB.click()); await settle();
    assert.match(view.document.body.textContent, /Contest B/);
    await act(async () => finishImport({ importStatus: "complete", missingSnapshotProblems: [], failedSnapshotProblems: [] })); await settle();
    assert.match(view.document.body.textContent, /Contest B/);
    assert.doesNotMatch(view.document.body.textContent, /Contest A/);
    assert.deepEqual(contestCalls.at(-1).scope, { kind: "family", familyId: 2, series: { kind: "any" }, year: { kind: "any" } });
  } finally { await view.cleanup(); }
});




test("Contest placement editor ignores stale Series responses after Family changes", { concurrency: false }, async () => {
  let resolveA;
  let resolveB;
  const seriesA = new Promise((resolve) => { resolveA = resolve; });
  const seriesB = new Promise((resolve) => { resolveB = resolve; });
  const detail = { contestId: 1979, title: "Contest", sourceUrl: "https://codeforces.com/contest/1979", contestDate: "2026-08-10", importStatus: "complete", factsStatus: "completed", problems: [], corrections: [], aiAnalysis: null, archived: false };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detail;
    if (command === "contest_library_list_contest_placements") return [];
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Family A" }, { familyId: 2, displayName: "Family B" }];
    if (command === "contest_library_list_series") return args.input.familyId === 1 ? seriesA : seriesB;
    throw new Error(`unexpected command ${command}`);
  }, "/contests/1979");
  try {
    await settle();
    const add = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "添加位置");
    await act(async () => add.click()); await settle();
    const familySelect = view.document.querySelector(".placement-form select");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(familySelect, "2");
      familySelect.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await act(async () => resolveB([{ seriesId: 22, familyId: 2, displayName: "Series B" }])); await settle();
    assert.ok([...view.document.querySelectorAll(".placement-form option")].some((option) => option.textContent === "Series B"));
    await act(async () => resolveA([{ seriesId: 11, familyId: 1, displayName: "Series A" }])); await settle();
    assert.ok([...view.document.querySelectorAll(".placement-form option")].some((option) => option.textContent === "Series B"));
    assert.equal([...view.document.querySelectorAll(".placement-form option")].some((option) => option.textContent === "Series A"), false);
    assert.equal(view.document.querySelectorAll(".placement-form select")[1].value, "");
  } finally { await view.cleanup(); }
});

test("Contest Library ignores stale Series responses after rapid Family switching", { concurrency: false }, async () => {
  let resolveA;
  let resolveB;
  const seriesA = new Promise((resolve) => { resolveA = resolve; });
  const seriesB = new Promise((resolve) => { resolveB = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_library_list_families") return [{ familyId: 1, displayName: "Family A" }, { familyId: 2, displayName: "Family B" }];
    if (command === "contest_library_list_contests") return [];
    if (command === "contest_library_list_years") return [];
    if (command === "contest_library_list_series") return args.input.familyId === 1 ? seriesA : seriesB;
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const familyA = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Family A");
    const familyB = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Family B");
    await act(async () => familyA.click());
    await act(async () => familyB.click());
    await act(async () => resolveB([{ seriesId: 22, familyId: 2, displayName: "Series B" }])); await settle();
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Series B"));
    await act(async () => resolveA([{ seriesId: 11, familyId: 1, displayName: "Series A" }])); await settle();
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Series B"));
    assert.equal([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Series A"), false);
  } finally { await view.cleanup(); }
});

test.after(async () => {
  moduleRunner.close();
  await vite.close();
});

test("Loading, Recovery, and Setup hide ordinary navigation and focus their content", { concurrency: false }, async () => {
  const states = [
    React.createElement(shells.LoadingShell),
    React.createElement(shells.RecoveryShell, {
      foundSchemaVersion: null,
      reason: "database_unavailable",
      supportedSchemaVersion: null,
    }),
    React.createElement(shells.SetupShell, {
      foundation: { state: "unavailable" },
      onConfigured: () => undefined,
    }),
  ];
  for (const component of states) {
    const view = await render(component);
    try {
      assert.equal(view.document.querySelector("nav"), null);
      assert.equal(view.document.activeElement?.tagName, "H1");
    } finally {
      await view.cleanup();
    }
  }
});

test("Normal shell exposes its frozen navigation, skip link, and focused route heading", { concurrency: false }, async () => {
  const view = await render(React.createElement(shells.NormalAppShell, {
    foundation: readyFoundation,
    navigate: () => undefined,
    route: { kind: "normal", page: "today" },
    workspace: configuredWorkspace,
  }));
  try {
    assert.deepEqual(
      [...view.document.querySelectorAll("nav a")].map((link) => link.textContent),
      ["今日", "学习月历", "比赛", "我的题库", "知识库", "奖励", "设置"],
    );
    assert.equal(view.document.querySelector(".skip-link")?.getAttribute("href"), "#main-content");
    assert.equal(view.document.activeElement?.textContent, "今日计划");
  } finally {
    await view.cleanup();
  }
});

test("Review focus is isolated and Return to Today requests the normal route", { concurrency: false }, async () => {
  const navigations = [];
  const view = await render(React.createElement(shells.ReviewFocusShell, {
    attemptId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
    navigate: (pathname) => navigations.push(pathname),
  }));
  try {
    assert.equal(view.document.querySelector("nav"), null);
    assert.equal(view.document.activeElement?.tagName, "MAIN");
    assert.doesNotMatch(view.document.body.textContent, /Open in Obsidian/);
    await act(async () => view.document.querySelector("button").click());
    assert.deepEqual(navigations, ["/today"]);
  } finally {
    await view.cleanup();
  }
});

test("App shell IPC rejection fails closed into Recovery", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return Promise.reject(new Error("unavailable"));
    throw new Error(`unexpected command ${command}`);
  });
  try {
    assert.equal(view.document.querySelector("nav"), null);
    assert.equal(view.document.querySelector("h1")?.textContent, "正常启动已被阻止");
    assert.equal(view.document.title, "恢复模式 · ACM-OS");
  } finally {
    await view.cleanup();
  }
});

test("Setup success replaces the URL with Today and focuses its new content", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") {
      return {
        state: "setup", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null,
        workspace: { state: "unconfigured", activeVaultPath: null, problemRootPath: null, knowledgeRootPath: null },
      };
    }
    if (command === "configure_workspace") return configuredWorkspace;
    throw new Error(`unexpected command ${command}`);
  }, "/setup");
  try {
    const inputs = [...view.document.querySelectorAll("input")];
    for (const [input, value] of inputs.map((input, index) => [input, ["C:/Vault", "C:/Vault/Problems", "C:/Vault/Knowledge"][index]])) {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, value);
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    }
    await act(async () => view.document.querySelector("form").dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    await settle();
    assert.equal(view.window.location.pathname, "/today");
    assert.equal(view.document.querySelector("h1")?.textContent, "今日计划");
    assert.equal(view.document.activeElement?.textContent, "今日计划");
  } finally {
    await view.cleanup();
  }
});

test("Normal navigation pushes and popstate enters Review focus with isolated chrome", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") {
      return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/today");
  try {
    const contests = [...view.document.querySelectorAll("a")].find((link) => link.textContent === "比赛");
    await act(async () => contests.dispatchEvent(new view.window.MouseEvent("click", { bubbles: true, button: 0 })));
    assert.equal(view.window.location.pathname, "/contests");
    assert.equal(view.document.querySelector("h1")?.textContent, "比赛");
    view.window.history.pushState(null, "", "/review/018f0d8e-4a5b-7c6d-8e9f-0123456789ab");
    await act(async () => view.window.dispatchEvent(new view.window.PopStateEvent("popstate")));
    await settle();
    assert.equal(view.document.querySelector("nav"), null);
    assert.equal(view.document.querySelector("h1")?.textContent, "独立复习空间");
    assert.equal(view.document.activeElement?.tagName, "MAIN");
  } finally {
    await view.cleanup();
  }
});

test("Contest import exposes a specific Chinese error instead of swallowing the IPC code", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") {
      return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    }
    if (command === "contest_library_list_families") return [];
    if (command === "contest_library_list_contests") return [];
    if (command === "import_codeforces_contest") throw new Error("unsupported_contest_url");
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    const input = view.document.querySelector('input[placeholder="https://codeforces.com/contest/1979"]');
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, "https://codeforces.com");
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await act(async () => {
      input.closest("form").dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
    });
    await settle();
    assert.match(view.document.body.textContent, /比赛网址格式不正确/);
    assert.match(view.document.body.textContent, /https:\/\/codeforces\.com\/contest\/1979/);

    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, "https://codeforces.com/contest/2256");
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    assert.doesNotMatch(view.document.body.textContent, /比赛网址格式不正确/);
  } finally {
    await view.cleanup();
  }
});

test("Contest facts snapshot preserves contest result beside live learning status", { concurrency: false }, async () => {
  let completed = false;
  let submitted;
  const detail = () => ({
    contestId: 1979, title: "Codeforces Round", sourceUrl: "https://codeforces.com/contest/1979",
    contestDate: "2026-08-10", importStatus: "complete", factsStatus: completed ? "completed" : "pending", corrections: [],
    problems: [{ contestId: 1979, index: "A", title: "Problem A", rating: 800, hasStatementSnapshot: true, identityType: "personal", finalContestResult: completed ? "wrongAnswer" : null, upsolveDecision: completed ? "planned" : "undecided", liveLearningStatus: "longTermReview" }],
  });
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detail();
    if (command === "complete_contest_facts") { submitted = args.input; completed = true; return detail(); }
    throw new Error(`unexpected command ${command}`);
  }, "/contests/1979");
  try {
    await settle();
    assert.match(view.document.body.textContent, /当前学习状态：长期复习/);
    const [resultSelect, upsolveSelect] = view.document.querySelectorAll("select");
    const form = view.document.querySelector("form");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(resultSelect, "wrongAnswer");
      resultSelect.dispatchEvent(new view.window.Event("change", { bubbles: true }));
      Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(upsolveSelect, "planned");
      upsolveSelect.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await act(async () => {
      form.dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
    });
    await settle();
    assert.equal(submitted.problems[0].finalContestResult, "wrongAnswer");
    assert.equal(submitted.problems[0].upsolveDecision, "planned");
    assert.match(view.document.body.textContent, /赛后整理已完成/);
    assert.equal([...view.document.querySelectorAll("select")].every((select) => !select.disabled), true);
    assert.equal([...view.document.querySelectorAll("button")].some((button) => button.textContent.includes("瀹屾垚璧涘悗鏁寸悊")), false);
    assert.match(view.document.body.textContent, /当前学习状态：长期复习/);
  } finally { await view.cleanup(); }
});

test("Contest detail displays canonical English problem titles", { concurrency: false }, async () => {
  const detail = {
    contestId: 2256,
    title: "Codeforces Round 1116 (Div. 2)",
    sourceUrl: "https://codeforces.com/contest/2256",
    contestDate: "2026-08-11",
    importStatus: "complete",
    factsStatus: "completed",
    archived: false,
    corrections: [],
    aiAnalysis: null,
    problems: [{
      contestId: 2256,
      index: "C",
      title: "Горячая картошка на складе фей",
      rating: null,
      hasStatementSnapshot: true,
      identityType: "lightweight",
      finalContestResult: "unknown",
      upsolveDecision: "undecided",
      liveLearningStatus: "unstarted",
    }],
  };
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detail;
    throw new Error(`unexpected command ${command}`);
  }, "/contests/2256");
  try {
    await settle();
    assert.match(view.document.body.textContent, /C\. Hot Potatoes at the Fairy Warehouse/);
  } finally { await view.cleanup(); }
});

test("Completed Contest correction updates facts and keeps an explicit history event", { concurrency: false }, async () => {
  let corrected = false;
  const detail = () => ({ contestId: 1979, title: "Round", sourceUrl: "https://codeforces.com/contest/1979", contestDate: "2026-08-10", importStatus: "complete", factsStatus: "completed",
    problems: [{ contestId: 1979, index: "A", title: "A", rating: 800, hasStatementSnapshot: true, identityType: "personal", finalContestResult: corrected ? "accepted" : "wrongAnswer", upsolveDecision: corrected ? "notPlanned" : "planned", liveLearningStatus: "longTermReview" }],
    corrections: corrected ? [{ correctionId: "c1", problemIndex: "A", field: "finalContestResult", oldValue: "wrong_answer", newValue: "accepted", correctedAtUtc: "2026-08-13T00:00:00Z" }] : [] });
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detail();
    if (command === "correct_contest_problem_facts") { corrected = true; return detail(); }
    throw new Error(`unexpected command ${command}`);
  }, "/contests/1979");
  try {
    await settle();
    const [resultSelect] = view.document.querySelectorAll("select");
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(resultSelect, "accepted"); resultSelect.dispatchEvent(new view.window.Event("change", { bubbles: true })); });
    const correct = view.document.querySelector(
      'form[aria-label="比赛事实快照"] button.secondary-action',
    );
    assert.ok(correct);
    await act(async () => correct.dispatchEvent(new view.window.MouseEvent("click", { bubbles: true })));
    await settle();
    assert.match(view.document.body.textContent, /纠错已保存/);
    assert.match(view.document.body.textContent, /纠错历史/);
    assert.match(view.document.body.textContent, /wrong_answer → accepted/);
    assert.match(view.document.body.textContent, /当前学习状态：长期复习/);
  } finally { await view.cleanup(); }
});

test("Problem detail creates a Personal Markdown through business IPC and re-queries Core", {
  concurrency: false,
}, async () => {
  let detailReads = 0;
  let createCalls = 0;
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") {
      return {
        state: "normal",
        recoveryReason: null,
        supportedSchemaVersion: null,
        foundSchemaVersion: null,
        workspace: configuredWorkspace,
      };
    }
    if (command === "lightweight_problem_detail") {
      detailReads += 1;
      const personal = detailReads > 1;
      return {
        contestId: 1979,
        index: "A",
        title: "Problem A",
        rating: 800,
        sourceUrl: "https://codeforces.com/contest/1979/problem/A",
        statement: { state: "pending" },
        identityType: personal ? "personal" : "lightweight",
        personalNote: personal ? { vaultRelativePath: "Problems/CF-1979-A.md" } : null,
        lifecycle: personal ? personalUnstartedLifecycle : lightweightLifecycle,
      };
    }
    if (command === "create_personal_note") {
      createCalls += 1;
      return { vaultRelativePath: "Problems/CF-1979-A.md" };
    }
    if (command === "personal_note_projection") {
      return {
        state: "ready",
        vaultRelativePath: "Archive/renamed.md",
        relocated: true,
        projection: {
          contentDigest: "fresh-digest",
          knownSections: [
            { name: "题解", startOffset: 11, endOffset: 44 },
            { name: "额外题目", startOffset: 44, endOffset: 55 },
          ],
          solutionRoutes: [
            { name: "External edit ×", startOffset: 20, endOffset: 44 },
          ],
          warnings: [],
        },
      };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    const createButton = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "创建个人 Markdown");
    assert.ok(createButton);
    const promptBeforeNote = view.document.querySelector('#external-personal-note-heading').closest('section').querySelector('.note-prompt-panel');
    assert.ok(promptBeforeNote);
    assert.match(promptBeforeNote.querySelector('textarea').value, /## 大致题意/);
    assert.equal(createCalls, 0);
    await act(async () => createButton.click());
    await settle();
    assert.equal(createCalls, 1);
    assert.equal(detailReads, 2);
    assert.match(view.document.body.textContent, /个人题目/);
    assert.match(view.document.body.textContent, /Archive\/renamed\.md/);
    assert.match(view.document.body.textContent, /笔记绑定已恢复到当前位置/);
    assert.match(view.document.body.textContent, /External edit ×/);
    assert.equal(
      [...view.document.querySelectorAll("button")]
        .some((button) => button.textContent === "创建个人 Markdown"),
      false,
    );
  } finally {
    await view.cleanup();
  }
});

test("Location Anomaly lists possible Markdown locations and explicitly rebinds an unoccupied file", {
  concurrency: false,
}, async () => {
  let rebound = false;
  const calls = [];
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "personal",
      personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" },
      lifecycle: personalUnstartedLifecycle,
    };
    if (command === "personal_note_projection") return rebound ? {
      state: "ready", vaultRelativePath: "Recovered/manual.md", relocated: false,
      projection: { contentDigest: "post", knownSections: [], solutionRoutes: [], warnings: [] },
    } : { state: "locationAnomaly", lastKnownPath: "Problems/CF-1979-A.md" };
    if (command === "knowledge_candidates") return [];
    if (command === "personal_note_relocation_candidates") return [
      { vaultRelativePath: "Problems/CF-1979-B.md", occupied: true },
      { vaultRelativePath: "Recovered/manual.md", occupied: false },
    ];
    if (command === "rebind_personal_note") {
      rebound = true;
      return { vaultRelativePath: args.input.vaultRelativePath };
    }
    if (command === "plugin:event|listen") return 1;
    if (command === "plugin:event|unlisten") return null;
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    await settle();
    assert.match(view.document.body.textContent, /笔记位置需要处理/);
    const find = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "查找可能的位置");
    await act(async () => find.click()); await settle();
    const occupied = [...view.document.querySelectorAll("li")]
      .find((item) => item.textContent.includes("CF-1979-B.md"));
    assert.ok(occupied.querySelector("button").disabled);
    const candidate = [...view.document.querySelectorAll("li")]
      .find((item) => item.textContent.includes("Recovered/manual.md"));
    await act(async () => candidate.querySelector("button").click()); await settle();
    assert.doesNotMatch(view.document.body.textContent, /笔记位置需要处理/);
    assert.match(view.document.body.textContent, /Recovered\/manual.md/);
    const rebindCall = calls.find(([command]) => command === "rebind_personal_note");
    assert.equal(rebindCall[1].input.vaultRelativePath, "Recovered/manual.md");
  } finally {
    await view.cleanup();
  }
});

test("Location Anomaly confirms a missing file only through an explicit consequence preview", {
  concurrency: false,
}, async () => {
  const calls = [];
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "personal",
      personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" },
      lifecycle: personalUnstartedLifecycle,
    };
    if (command === "personal_note_projection") return { state: "locationAnomaly", lastKnownPath: "Problems/CF-1979-A.md" };
    if (command === "knowledge_candidates") return [];
    if (command === "confirm_personal_note_deleted") return lightweightLifecycle;
    if (command === "plugin:event|listen") return 1;
    if (command === "plugin:event|unlisten") return null;
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    await settle();
    const preview = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "确认文件已删除…");
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /不会删除任何文件/);
    assert.equal(calls.some(([command]) => command === "confirm_personal_note_deleted"), false);
    const confirm = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "确认已删除");
    await act(async () => confirm.click()); await settle();
    assert.match(view.document.body.textContent, /我的题库/);
    assert.doesNotMatch(view.document.body.textContent, /笔记位置需要处理/);
    assert.equal(calls.filter(([command]) => command === "confirm_personal_note_deleted").length, 1);
  } finally {
    await view.cleanup();
  }
});

test("StrictMode keeps the ready Personal Markdown projection and exposes the Obsidian editor entry", {
  concurrency: false,
}, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "personal",
      personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" },
      lifecycle: personalUnstartedLifecycle,
    };
    if (command === "personal_note_projection") return {
      state: "ready", vaultRelativePath: "Problems/CF-1979-A.md", relocated: false,
      projection: { contentDigest: "fresh", knownSections: ["题解"], solutionRoutes: [], warnings: [] },
    };
    if (command === "plugin:event|listen") return 1;
    if (command === "plugin:event|unlisten") return null;
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A", true);
  try {
    await settle();
    const openButton = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "在 Obsidian 中打开并编辑题解");
    assert.ok(openButton);
    assert.match(view.document.body.textContent, /个人 Markdown：.*Problems\/CF-1979-A\.md/);
  } finally {
    await view.cleanup();
  }
});

test("Problem statement renders Codeforces LaTeX locally while preserving code blocks", {
  concurrency: false,
}, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 2256, index: "A", title: "Formula", rating: 800,
      sourceUrl: "https://codeforces.com/contest/2256/problem/A",
      statement: {
        state: "ready",
        sanitizedHtml: "<div class=\"problem-statement\"><p>For $$$a^2+b^2=c^2$$$.</p><p>$$$\\frac{1}{2}$$$</p><p><a href=\"javascript:alert(1)\">unsafe</a><a href=\"https://codeforces.com\">safe</a></p><pre>$$$keep_raw$$$</pre></div>",
      },
      identityType: "lightweight", personalNote: null, lifecycle: lightweightLifecycle,
    };
    if (command === "statement_assets") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/problems/2256/A");
  try {
    await settle();
    assert.equal(view.document.querySelectorAll(".statement-view .katex").length, 2);
    assert.ok(view.document.querySelector(".statement-view .katex-display"));
    assert.match(view.document.querySelector(".statement-view pre")?.textContent ?? "", /\$\$\$keep_raw\$\$\$/);
    assert.doesNotMatch(view.document.querySelector(".statement-view p")?.textContent ?? "", /\$\$\$/);
    const links = [...view.document.querySelectorAll(".statement-view a")];
    assert.equal(links[0]?.getAttribute("href"), "#");
    assert.equal(links[1]?.getAttribute("href"), "https://codeforces.com");
  } finally {
    await view.cleanup();
  }
});

test("Canonical Problem route loads by problem_id without projecting an external alias", {
  concurrency: false,
}, async () => {
  const calls = [];
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") return {
      problemId: "42", title: "Canonical Problem", rating: 1200,
      sourceUrl: "https://example.test/problem/42", statement: { state: "pending" },
      identityType: "personal", personalNote: { vaultRelativePath: "Problems/Canonical.md" },
      lifecycle: personalUnstartedLifecycle, reviewAction: null,
    };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/42");
  try {
    await settle();
    assert.equal(calls.some(([command]) => command === "lightweight_problem_detail_by_id"), true, JSON.stringify(calls));
    assert.match(view.document.body.textContent, /Canonical Problem/);
    assert.match(view.document.body.textContent, /Problems\/Canonical\.md/);
    assert.doesNotMatch(view.document.body.textContent, /Codeforces/);
  } finally {
    await view.cleanup();
  }
});

test("Canonical bound Personal Note exposes read, open, and delete through problem_id IPC", {
  concurrency: false,
}, async () => {
  const calls = [];
  const lifecycle = { ...personalUnstartedLifecycle };
  const detail = {
    problemId: "42", title: "Bound Canonical Problem", rating: 1200,
    sourceUrl: "https://example.test/problem/42", statement: { state: "pending" },
    identityType: "personal", personalNote: { vaultRelativePath: "Problems/Canonical.md" },
    lifecycle, reviewAction: null,
  };
  const projection = {
    state: "ready", vaultRelativePath: "Problems/Canonical.md", relocated: false,
    projection: { contentDigest: "a".repeat(64), knownSections: [], solutionRoutes: [], warnings: [] },
  };
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") return detail;
    if (command === "personal_note_projection_by_id") return projection;
    if (command === "knowledge_candidates_by_id") return [];
    if (command === "open_personal_note_in_obsidian_by_id") return null;
    if (command === "delete_personal_note_by_id") return { ...lifecycle, identityType: "lightweight", learningStatus: "unstarted", availableActions: [] };
    if (command === "review_history_by_id") return { items: [], mastery: null };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/42");
  try {
    await settle();
    assert.match(view.document.body.textContent, /Problems\/Canonical\.md/);
    assert.equal([...view.document.querySelectorAll("button")].some((button) => button.textContent === "创建个人 Markdown"), false);
    const open = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "在 Obsidian 中打开");
    assert.ok(open);
    await act(async () => open.click()); await settle();
    const deleteButton = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "删除笔记");
    assert.ok(deleteButton);
    await act(async () => deleteButton.click()); await settle();
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "确认删除");
    assert.ok(confirm);
    await act(async () => confirm.click()); await settle();
    const personalCommands = calls.filter(([command]) => command.includes("personal_note"));
    assert.ok(personalCommands.some(([command]) => command === "personal_note_projection_by_id"));
    assert.ok(personalCommands.some(([command]) => command === "open_personal_note_in_obsidian_by_id"));
    assert.ok(personalCommands.some(([command]) => command === "delete_personal_note_by_id"));
    assert.equal(personalCommands.some(([command]) => ["personal_note_projection", "open_personal_note_in_obsidian", "delete_personal_note"].includes(command)), false);
    for (const [, args] of personalCommands) assert.deepEqual(args?.input?.problemId, "42");
  } finally {
    await view.cleanup();
  }
});

test("Canonical Personal Note anomaly exposes relocation, rebind, and missing confirmation by problem_id", {
  concurrency: false,
}, async () => {
  const calls = [];
  const detail = {
    problemId: "43", title: "Anomalous Canonical Problem", rating: null,
    sourceUrl: "https://example.test/problem/43", statement: { state: "pending" },
    identityType: "personal", personalNote: { vaultRelativePath: "Problems/Missing.md" },
    lifecycle: personalUnstartedLifecycle, reviewAction: null,
  };
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") return detail;
    if (command === "personal_note_projection_by_id") return { state: "locationAnomaly", lastKnownPath: "Problems/Missing.md" };
    if (command === "knowledge_candidates_by_id") return [];
    if (command === "personal_note_relocation_candidates_by_id") return [{ vaultRelativePath: "Archive/Recovered.md", occupied: false }];
    if (command === "rebind_personal_note_by_id") return { vaultRelativePath: "Archive/Recovered.md" };
    if (command === "confirm_personal_note_deleted_by_id") return { identityType: "lightweight", learningStatus: "unstarted", learningStatusSinceUtc: "2026-08-24T00:00:00Z", nextReviewDueLocalDate: null, availableActions: [] };
    if (command === "review_history_by_id") return { items: [], mastery: null };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/43");
  try {
    await settle();
    const find = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "查找笔记位置");
    assert.ok(find);
    await act(async () => find.click()); await settle();
    const rebind = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "重新绑定");
    assert.ok(rebind);
    await act(async () => rebind.click()); await settle();
    const missing = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "确认缺失笔记已删除");
    assert.ok(missing);
    await act(async () => missing.click()); await settle();
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "确认");
    assert.ok(confirm);
    await act(async () => confirm.click()); await settle();
    const commands = calls.map(([command]) => command);
    assert.ok(commands.includes("personal_note_relocation_candidates_by_id"));
    assert.ok(commands.includes("rebind_personal_note_by_id"));
    assert.ok(commands.includes("confirm_personal_note_deleted_by_id"));
    assert.equal(commands.some((command) => ["personal_note_relocation_candidates", "rebind_personal_note", "confirm_personal_note_deleted"].includes(command)), false);
    for (const [command, args] of calls.filter(([name]) => name.includes("personal_note"))) {
      if (command.endsWith("_by_id")) assert.equal(args.input.problemId, "43");
    }
  } finally {
    await view.cleanup();
  }
});

test("Unbound canonical Problem creates Personal Markdown only through problem_id IPC", {
  concurrency: false,
}, async () => {
  const calls = [];
  let detailReads = 0;
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") {
      detailReads += 1;
      const personal = detailReads > 1;
      return {
        problemId: "44", title: "Unbound Canonical Problem", rating: null,
        sourceUrl: "https://example.test/problem/44", statement: { state: "pending" },
        identityType: personal ? "personal" : "lightweight",
        personalNote: personal ? { vaultRelativePath: "Problems/Problem-44.md" } : null,
        lifecycle: personal ? personalUnstartedLifecycle : lightweightLifecycle,
        reviewAction: null,
      };
    }
    if (command === "create_personal_note_by_id") return { vaultRelativePath: "Problems/Problem-44.md" };
    if (command === "personal_note_projection_by_id") return {
      state: "ready", vaultRelativePath: "Problems/Problem-44.md", relocated: false,
      projection: { contentDigest: "a".repeat(64), knownSections: [], solutionRoutes: [], warnings: [] },
    };
    if (command === "review_history_by_id") return { items: [], mastery: null };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/44");
  try {
    await settle();
    assert.match(view.document.body.textContent, /Unbound Canonical Problem/);
    const promptBeforeNote = view.document.querySelector('#canonical-personal-note-heading').closest('section').querySelector('.note-prompt-panel');
    assert.ok(promptBeforeNote);
    const copyBeforeNote = [...promptBeforeNote.querySelectorAll('button')].find(button => button.textContent === '复制提示词');
    await act(async () => copyBeforeNote.click());
    assert.match(promptBeforeNote.querySelector('textarea').value, /## 大致题意/);
    assert.equal(calls.filter(([command]) => command === 'create_personal_note_by_id').length, 0);
    const create = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "创建个人 Markdown");
    assert.ok(create);
    await act(async () => create.click());
    await settle();
    assert.match(view.document.body.textContent, /Problems\/Problem-44\.md/);
    assert.ok([...view.document.querySelectorAll("button")]
      .some((button) => button.textContent === "在 Obsidian 中打开"));
    assert.equal(calls.filter(([command]) => command === "create_personal_note_by_id").length, 1);
    assert.equal(calls.some(([command]) => command === "create_personal_note"), false);
    const createCall = calls.find(([command]) => command === "create_personal_note_by_id");
    assert.deepEqual(createCall[1].input, { problemId: "44" });
    assert.equal(calls.some(([, args]) => args?.input?.contestId !== undefined || args?.input?.index !== undefined), false);
  } finally {
    await view.cleanup();
  }
});

test("Canonical Problem renders the next Review due date from canonical lifecycle state", {
  concurrency: false,
}, async () => {
  const calls = [];
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") return {
      problemId: "45", title: "Due Canonical Problem", rating: 1500,
      sourceUrl: "https://example.test/problem/45", statement: { state: "pending" },
      identityType: "personal", personalNote: { vaultRelativePath: "Problems/Problem-45.md" },
      lifecycle: { ...personalUnstartedLifecycle, nextReviewDueLocalDate: "2026-09-03" }, reviewAction: "earlyCheck",
    };
    if (command === "personal_note_projection_by_id") return {
      state: "ready", vaultRelativePath: "Problems/Problem-45.md", relocated: false,
      projection: { contentDigest: "a".repeat(64), knownSections: [], solutionRoutes: [], warnings: [] },
    };
    if (command === "knowledge_candidates_by_id") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/45");
  try {
    await settle();
    assert.match(view.document.body.textContent, /下次复习到期：\s*2026-09-03/);
    assert.equal(calls.some(([command]) => command === "lightweight_problem_detail"), false);
    assert.deepEqual(calls.find(([command]) => command === "lightweight_problem_detail_by_id")?.[1]?.input, { problemId: "45" });
  } finally {
    await view.cleanup();
  }
});

test("Canonical Knowledge candidate can return to pending only through problem_id mutation", {
  concurrency: false,
}, async () => {
  const calls = [];
  const ignored = {
    problemId: "46", fingerprint: "candidate-46", targetRef: "Graph Basics",
    disposition: "ignored", knowledgeNodeId: null,
  };
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") return {
      problemId: "46", title: "Knowledge Canonical Problem", rating: null,
      sourceUrl: "https://example.test/problem/46", statement: { state: "pending" },
      identityType: "personal", personalNote: { vaultRelativePath: "Problems/Problem-46.md" },
      lifecycle: personalUnstartedLifecycle, reviewAction: null,
    };
    if (command === "personal_note_projection_by_id") return {
      state: "ready", vaultRelativePath: "Problems/Problem-46.md", relocated: false,
      projection: { contentDigest: "b".repeat(64), knownSections: [], solutionRoutes: [], warnings: [] },
    };
    if (command === "knowledge_candidates_by_id") return [ignored];
    if (command === "set_knowledge_candidate_disposition_by_id") return { ...ignored, disposition: "pending" };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/46");
  try {
    await settle();
    const returnToPending = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "退回待处理");
    assert.ok(returnToPending);
    await act(async () => returnToPending.click());
    await settle();
    const mutation = calls.find(([command]) => command === "set_knowledge_candidate_disposition_by_id");
    assert.deepEqual(mutation?.[1]?.input, {
      problemId: "46", fingerprint: "candidate-46", disposition: "pending",
    });
    assert.equal(calls.some(([command]) => command === "set_knowledge_candidate_disposition"), false);
  } finally {
    await view.cleanup();
  }
});

test("Canonical Personal Note renders parsed sections routes and warnings", {
  concurrency: false,
}, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") return {
      problemId: "47", title: "Projection Canonical Problem", rating: 1800,
      sourceUrl: "https://example.test/problem/47", statement: { state: "pending" },
      identityType: "personal", personalNote: { vaultRelativePath: "Problems/Problem-47.md" },
      lifecycle: personalUnstartedLifecycle, reviewAction: null,
    };
    if (command === "personal_note_projection_by_id") return {
      state: "ready", vaultRelativePath: "Problems/Problem-47.md", relocated: true,
      projection: {
        contentDigest: "c".repeat(64),
        introductionHtml: "<p>完整题意摘要</p>",
        documentSections: [
          { name: "前置知识", startOffset: 0, endOffset: 10, contentMarkdown: "二分图", contentHtml: "<p>二分图</p>" },
          { name: "Hints", startOffset: 10, endOffset: 30, contentMarkdown: "先染色", contentHtml: "<p>先染色</p>" },
          { name: "题解", startOffset: 30, endOffset: 80, contentMarkdown: "", contentHtml: "" },
          { name: "额外题目", startOffset: 80, endOffset: 90, contentMarkdown: "暂无", contentHtml: "<p>暂无</p>" },
          { name: "Prompt", startOffset: 90, endOffset: 120, contentMarkdown: "用户保存的原始 Prompt\n请保留我的思路。", contentHtml: "<p>用户保存的原始 Prompt</p>" },
        ],
        knownSections: [{ name: "Prerequisite Knowledge", startOffset: 10, endOffset: 30 }],
        solutionRoutes: [{ name: "Binary search route", startOffset: 31, endOffset: 60, contentMarkdown: "#### Proof\nOnly this route $n^2$\n```cpp\nint n;\n```", contentHtml: '<h4>Proof</h4><p>Only this route $n^2$ <span class="math math-inline">a_i+b_j</span></p><span class="math math-display">x^2</span><pre><code>int n;</code></pre><img src="https://tracker.test/x" onerror="alert(1)"><a href="javascript:alert(1)">unsafe link</a>' }, { name: "Empty alternative", startOffset: 60, endOffset: 70, contentMarkdown: "", contentHtml: "" }],
        warnings: [{ code: "duplicate_known_section", name: "Solution", count: 2 }],
      },
    };
    if (command === "knowledge_candidates_by_id") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/47");
  try {
    await settle();
    const projection = view.document.querySelector('[aria-label="个人 Markdown 内容"]');
    assert.ok(projection);
    assert.match(projection.textContent, /完整题意摘要/);
    assert.deepEqual([...projection.querySelectorAll("summary")].map(summary => summary.textContent), ["前置知识", "Hints", "题解", "额外题目"]);
    assert.equal(projection.querySelector('.note-prompt-panel'), null);
    assert.equal(projection.querySelector("details[open]"), null);
    const solutionSection = [...projection.querySelectorAll("details")].find(details => details.querySelector("summary").textContent === "题解");
    solutionSection.open = true;
    assert.match(projection.textContent, /Binary search route/);
    assert.match(projection.textContent, /重复章节：Solution（2 处）/);
    assert.match(projection.textContent, /笔记绑定已恢复到当前位置/);
    assert.equal(view.document.querySelector('[role="dialog"]'), null);
    const route = [...projection.querySelectorAll("button")].find(button => button.textContent.includes("Binary search route"));
    await act(async () => route.click());
    const dialog = view.document.querySelector('[role="dialog"]');
    assert.match(dialog.textContent, /Only this route/);
    assert.equal(dialog.querySelector("pre code").textContent, "int n;");
    assert.ok(dialog.querySelector(".katex"));
    assert.ok(dialog.querySelector(".math-inline .katex"));
    assert.ok(dialog.querySelector(".math-display .katex-display"));
    assert.equal(dialog.querySelector("img"), null);
    assert.equal(dialog.querySelector("a").getAttribute("href"), "#");
    assert.equal(view.document.activeElement.textContent, "关闭");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    assert.equal(view.document.querySelector('[role="dialog"]'), null);
    assert.equal(view.document.activeElement, route);
    const emptyRoute = [...projection.querySelectorAll("button")].find(button => button.textContent.includes("Empty alternative"));
    await act(async () => emptyRoute.click());
    assert.match(view.document.querySelector('[role="dialog"]').textContent, /这个方法还没有正文/);
    assert.doesNotMatch(view.document.querySelector('[role="dialog"]').textContent, /Only this route/);
    await act(async () => [...view.document.querySelectorAll('[role="dialog"] button')].find(button => button.textContent === "关闭").click());
    const promptPanel = view.document.querySelector('#canonical-personal-note-heading').closest('section').querySelector('.note-prompt-panel');
    const copy = [...promptPanel.querySelectorAll("button")].find(button => button.textContent === "复制提示词");
    await act(async () => copy.click());
    await settle();
    const prompt = promptPanel.querySelector("textarea[readonly]").value;
    assert.equal(prompt, "用户保存的原始 Prompt\n请保留我的思路。");
  } finally {
    await view.cleanup();
  }
});

test("Canonical Vault-unavailable state preserves identity and disables Markdown operations", {
  concurrency: false,
}, async () => {
  const calls = [];
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail_by_id") return {
      problemId: "48", title: "Unavailable Vault Canonical Problem", rating: null,
      sourceUrl: "https://example.test/problem/48", statement: { state: "pending" },
      identityType: "personal", personalNote: { vaultRelativePath: "Problems/Problem-48.md" },
      lifecycle: personalUnstartedLifecycle, reviewAction: null,
    };
    if (command === "personal_note_projection_by_id") return {
      state: "vaultUnavailable", lastKnownPath: "Problems/Problem-48.md",
    };
    if (command === "knowledge_candidates_by_id") return [];
    throw new Error(`unexpected command ${command}`);
  }, "/problems/id/48");
  try {
    await settle();
    assert.match(view.document.body.textContent, /Vault 当前不可用/);
    assert.match(view.document.body.textContent, /个人题目及其系统事实已保留/);
    assert.equal([...view.document.querySelectorAll("button")].some((button) => button.textContent === "在 Obsidian 中打开"), false);
    assert.equal(calls.some(([command]) => command === "personal_note_projection"), false);
    assert.ok(calls.some(([command, args]) => command === "personal_note_projection_by_id" && args.input.problemId === "48"));
  } finally {
    await view.cleanup();
  }
});

test("Problem lifecycle actions and personal-note consequence preview use authoritative IPC results", {
  concurrency: false,
}, async () => {
  let lifecycle = personalUnstartedLifecycle;
  let transitionCalls = 0;
  let deleteCalls = 0;
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "personal",
      personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" }, lifecycle,
    };
    if (command === "personal_note_projection") return {
      state: "ready", vaultRelativePath: "Problems/CF-1979-A.md", relocated: false,
      projection: { contentDigest: "fresh", knownSections: [], solutionRoutes: [], warnings: [] },
    };
    if (command === "transition_problem_lifecycle") {
      transitionCalls += 1;
      assert.equal(args.input.action, "joinUpsolve");
      lifecycle = {
        ...personalUnstartedLifecycle,
        learningStatus: "upsolvePending",
        availableActions: ["startLearning", "stopLearning"],
      };
      return lifecycle;
    }
    if (command === "delete_personal_note") {
      deleteCalls += 1;
      return lightweightLifecycle;
    }
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    const join = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "加入补题");
    assert.ok(join);
    await act(async () => join.click());
    await settle();
    assert.equal(transitionCalls, 1);
    assert.match(view.document.body.textContent, /待补/);

    const beginDelete = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "删除我的个人笔记…");
    await act(async () => beginDelete.click());
    assert.match(view.document.body.textContent, /比赛历史、已完成的复习历史/);
    const confirmDelete = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "删除笔记");
    await act(async () => confirmDelete.click());
    await settle();
    assert.equal(deleteCalls, 1);
    assert.match(view.document.body.textContent, /我的题库/);
    assert.match(view.document.body.textContent, /历史事实已保留/);
  } finally {
    await view.cleanup();
  }
});

test("Due Review starts once and Focus renders only statement, OJ, and Attempt metadata", {
  concurrency: false,
}, async () => {
  const attemptId = "018f0d8e-4a5b-7c6d-8e9f-0123456789ab";
  let startCalls = 0;
  let focusCalls = 0;
  let drawerCalls = 0;
  const openOjCalls = [];
  const revealCalls = [];
  const completeCalls = [];
  const waitingLifecycle = {
    learningStatus: "waitingColdStart",
    learningStatusSinceUtc: "2026-08-11T00:00:00.000Z",
    nextReviewDueLocalDate: "2026-08-14",
    availableActions: ["withdrawUnderstood", "stopLearning"],
  };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "personal",
      personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" },
      lifecycle: waitingLifecycle, reviewAction: "startReview",
    };
    if (command === "personal_note_projection") return {
      state: "ready", vaultRelativePath: "Problems/CF-1979-A.md", relocated: false,
      projection: {
        contentDigest: "secret-note-digest",
        knownSections: [{ name: "题解", startOffset: 1, endOffset: 10 }],
        solutionRoutes: [{ name: "SECRET SOLUTION", startOffset: 2, endOffset: 9 }],
        warnings: [],
      },
    };
    if (command === "start_or_resume_review") {
      startCalls += 1;
      return {
        attemptId, contestId: 1979, index: "A", attemptType: "firstColdStart",
        scheduledDueLocalDate: "2026-08-14", startedEarly: false,
        judgementRuleVersion: 1, startedAtUtc: "2026-08-14T00:00:00.000Z",
      };
    }
    if (command === "review_focus") {
      focusCalls += 1;
      return {
        attempt: {
          attemptId, problemId: "1", attemptType: "firstColdStart",
          scheduledDueLocalDate: "2026-08-14", startedEarly: false,
          judgementRuleVersion: 1, startedAtUtc: "2026-08-14T00:00:00.000Z",
        },
        title: "Problem A",
        sourceUrl: "https://codeforces.com/contest/1979/problem/A",
        statementSanitizedHtml: "<div class=\"problem-statement\"><p>SAFE STATEMENT</p></div>",
        statementAssets: [],
      };
    }
    if (command === "open_original_oj") {
      openOjCalls.push(args.input.url);
      return undefined;
    }
    if (command === "review_help_drawer") {
      drawerCalls += 1;
      return {
        attemptId,
        items: [
          { level: 1, consequence: "partial_at_best", available: false, revealedAtUtc: null },
          { level: 2, consequence: "partial_at_best", available: true, revealedAtUtc: null },
          { level: 3, consequence: "partial_at_best", available: false, revealedAtUtc: null },
          { level: 4, consequence: "partial_at_best", available: false, revealedAtUtc: null },
          { level: 5, consequence: "fail_only", available: true, revealedAtUtc: null },
        ],
      };
    }
    if (command === "reveal_review_help") {
      revealCalls.push(args.input);
      return {
        eventId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ac",
        attemptId,
        level: args.input.level,
        consequence: args.input.level === 5 ? "fail_only" : "partial_at_best",
        title: args.input.level === 5 ? "Full solution" : "Hints",
        contentMarkdown: args.input.level === 5
          ? "## 题解\nFULL SOLUTION AFTER EVIDENCE"
          : "### Hint 1\nREVEALED ONLY AFTER EVIDENCE",
        sourceDigest: "a".repeat(64),
        revealedAtUtc: "2026-08-14T00:05:00.000Z",
      };
    }
    if (command === "complete_review") {
      completeCalls.push(args.input);
      if (args.input.failureReasons.length === 0) {
        return Promise.reject("review_failure_reason_required");
      }
      return {
        attempt: {
          attemptId, problemId: "1", attemptType: "firstColdStart",
          scheduledDueLocalDate: "2026-08-14", startedEarly: false,
          judgementRuleVersion: 1, startedAtUtc: "2026-08-14T00:00:00.000Z",
        },
        judgement: "partial",
        evidenceCodes: ["final_ac", "controlled_help_l2", "debug_not_needed"],
        failureReasons: [{ code: "keyPropertyBlocked", otherText: null }],
        completedAtUtc: "2026-08-14T00:10:00.000Z",
        completedLocalDate: "2026-08-14",
        lifecycle: {
          learningStatus: "relearning",
          learningStatusSinceUtc: "2026-08-14T00:10:00.000Z",
          nextReviewDueLocalDate: null,
          availableActions: ["startRelearning", "stopLearning"],
        },
      };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    const start = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "开始复习");
    assert.ok(start);
    await act(async () => start.click());
    await settle();
    assert.equal(startCalls, 1);
    assert.equal(focusCalls, 1);
    assert.equal(view.window.location.pathname, `/review/${attemptId}`);
    assert.equal(view.document.querySelector("nav"), null);
    assert.match(view.document.body.textContent, /SAFE STATEMENT/);
    assert.match(view.document.body.textContent, /打开原始 OJ/);
    assert.doesNotMatch(view.document.body.textContent, /SECRET SOLUTION/);
    assert.doesNotMatch(view.document.body.textContent, /Obsidian/);
    const originalOj = [...view.document.querySelectorAll("a")]
      .find((link) => link.textContent === "打开原始 OJ");
    await act(async () => originalOj.click());
    await settle();
    assert.deepEqual(openOjCalls, ["https://codeforces.com/contest/1979/problem/A"]);
    const openHelp = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "打开受控帮助");
    await act(async () => openHelp.click());
    await settle();
    assert.equal(drawerCalls, 1);
    assert.match(view.document.body.textContent, /打开此面板不会产生记录/);
    assert.equal(view.document.activeElement?.textContent, "受控帮助");
    assert.doesNotMatch(view.document.body.textContent, /REVEALED ONLY AFTER EVIDENCE/);
    assert.equal(revealCalls.length, 0);
    const hintRow = [...view.document.querySelectorAll(".review-help-levels li")]
      .find((row) => row.textContent.includes("第 2 级"));
    await act(async () => hintRow.querySelector("button").click());
    assert.match(view.document.body.textContent, /最多只能判定为部分掌握/);
    assert.equal(view.document.activeElement?.textContent, "确认并查看");
    assert.equal(revealCalls.length, 0, "confirmation precedes reveal IPC");
    const confirm = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "确认并查看");
    await act(async () => confirm.click());
    await settle();
    assert.deepEqual(revealCalls, [{ attemptId, level: 2, impactAcknowledged: true }]);
    assert.match(view.document.body.textContent, /REVEALED ONLY AFTER EVIDENCE/);
    const solutionRow = [...view.document.querySelectorAll(".review-help-levels li")]
      .find((row) => row.textContent.includes("第 5 级"));
    await act(async () => solutionRow.querySelector("button").click());
    assert.match(view.document.body.textContent, /只能判定为未通过/);
    assert.equal(revealCalls.length, 1, "Level 5 needs its own confirmation");
    const cancel = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "取消");
    await act(async () => cancel.click());
    assert.doesNotMatch(view.document.body.textContent, /FULL SOLUTION AFTER EVIDENCE/);
    assert.equal(view.document.activeElement?.textContent, "受控帮助");
    const closeHelp = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "关闭");
    await act(async () => closeHelp.click());
    assert.equal(view.document.activeElement, openHelp);
    const voidTrigger = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "作废误开的复习");
    await act(async () => voidTrigger.click());
    const voidDialog = view.document.querySelector('[aria-labelledby="void-review-title"]');
    const voidReason = voidDialog.querySelector("input");
    assert.equal(view.document.activeElement, voidReason);
    const voidCancel = [...voidDialog.querySelectorAll("button")]
      .find((button) => button.textContent === "取消");
    view.document.querySelector('[aria-labelledby="void-review-title"] input')?.focus();
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement, voidReason, "Tab stays inside the modal");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    assert.equal(view.document.querySelector('[aria-labelledby="void-review-title"]'), null);
    assert.equal(view.document.activeElement, voidTrigger);
    const complete = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "根据事实完成复习");
    await act(async () => [...view.document.querySelectorAll("form.review-facts-form button[type=submit]")][0].click());
    await settle();
    assert.equal(completeCalls.length, 1);
    assert.match(view.document.body.textContent, /请至少选择一个失败原因/);
    const reason = [...view.document.querySelectorAll("label")]
      .find((label) => label.textContent.includes("找到方向，但卡在关键性质"));
    await act(async () => reason.querySelector("input").click());
    await act(async () => [...view.document.querySelectorAll("form.review-facts-form button[type=submit]")][0].click());
    await settle();
    assert.equal(completeCalls.length, 2);
    assert.deepEqual(completeCalls[1].failureReasons, [{ code: "keyPropertyBlocked", otherText: null }]);
    assert.match(view.document.body.textContent, /复习已完成/);
    assert.match(view.document.body.textContent, /部分掌握/);
    assert.match(view.document.body.textContent, /回炉中/);
  } finally {
    await view.cleanup();
  }
});

test("Review history preserves an earlier Mastered result after a later Partial result", {
  concurrency: false,
}, async () => {
  const attempt = (attemptId, startedAtUtc) => ({
    attemptId, contestId: 1979, index: "A", attemptType: "longTermReview",
    scheduledDueLocalDate: "2026-08-24", startedEarly: false,
    judgementRuleVersion: 1, startedAtUtc,
  });
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "lightweight", personalNote: null,
      lifecycle: lightweightLifecycle, reviewAction: null,
    };
    if (command === "review_history") return {
      contestId: 1979,
      index: "A",
      historicalBestReview: "mastered",
      attempts: [
        {
          attempt: attempt("018f0d8e-4a5b-7c6d-8e9f-0123456789ad", "2026-08-24T00:00:00.000Z"),
          status: "completed", judgement: "partial", completionFacts: null,
          evidenceCodes: ["final_ac", "external_solving_hint"],
          failureReasons: [{ code: "keyPropertyBlocked", otherText: null }], helpLevels: [],
          completedAtUtc: "2026-08-24T01:00:00.000Z", completedLocalDate: "2026-08-24",
          voidReason: null, voidedAtUtc: null,
        },
        {
          attempt: attempt("018f0d8e-4a5b-7c6d-8e9f-0123456789ab", "2026-08-14T00:00:00.000Z"),
          status: "completed", judgement: "mastered", completionFacts: null,
          evidenceCodes: ["final_ac"], failureReasons: [], helpLevels: [],
          completedAtUtc: "2026-08-14T01:00:00.000Z", completedLocalDate: "2026-08-14",
          voidReason: null, voidedAtUtc: null,
        },
      ],
    };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    const load = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "加载复习历史");
    await act(async () => load.click());
    await settle();
    assert.match(view.document.body.textContent, /历史最佳复习证据：\s*已掌握/);
    assert.match(view.document.body.textContent, /部分掌握/);
    assert.match(view.document.body.textContent, /找到方向，但卡在关键性质/);
  } finally {
    await view.cleanup();
  }
});

test("Problem detail preserves Personal identity when the Vault is unavailable", {
  concurrency: false,
}, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") {
      return {
        state: "normal",
        recoveryReason: null,
        supportedSchemaVersion: null,
        foundSchemaVersion: null,
        workspace: configuredWorkspace,
      };
    }
    if (command === "lightweight_problem_detail") {
      return {
        contestId: 1979,
        index: "A",
        title: "Problem A",
        rating: 800,
        sourceUrl: "https://codeforces.com/contest/1979/problem/A",
        statement: { state: "pending" },
        identityType: "personal",
        personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" },
        lifecycle: personalUnstartedLifecycle,
      };
    }
    if (command === "personal_note_projection") {
      return { state: "vaultUnavailable", lastKnownPath: "Problems/CF-1979-A.md" };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    await settle();
    assert.match(view.document.body.textContent, /个人题目/);
    assert.match(view.document.body.textContent, /Vault 当前不可用/);
    assert.match(view.document.body.textContent, /系统事实已保留/);
    assert.equal(
      [...view.document.querySelectorAll("button")]
        .some((button) => button.textContent === "创建个人 Markdown"),
      false,
    );
  } finally {
    await view.cleanup();
  }
});

test("six mastery evidence items preserve historical thorough digestion after current regression", {
  concurrency: false,
}, async () => {
  const updates = [];
  const blank = {
    recallsProblem: false, multipleSolutionsClear: false, knowledgeUnderstood: false,
    implementationFluent: false, canAdaptOrCreate: false, transferSolvedIndependently: false,
  };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "lightweight", personalNote: null,
      lifecycle: { ...lightweightLifecycle, learningStatus: "relearning" }, reviewAction: null,
    };
    if (command === "review_history") return {
      contestId: 1979, index: "A", historicalBestReview: null, attempts: [],
      mastery: { current: blank, historicalThoroughlyDigested: false, firstThoroughlyDigestedLocalDate: null },
    };
    if (command === "update_problem_mastery_evidence") {
      updates.push(args.input.evidence);
      const all = Object.values(args.input.evidence).every(Boolean);
      return {
        current: args.input.evidence,
        historicalThoroughlyDigested: true,
        firstThoroughlyDigestedLocalDate: "2026-08-12",
        ...(all ? {} : {}),
      };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    const load = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "加载复习历史");
    await act(async () => load.click());
    await settle();
    const checks = [...view.document.querySelectorAll(".mastery-evidence input[type='checkbox']")];
    assert.equal(checks.length, 6);
    for (const check of checks) await act(async () => check.click());
    const save = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "保存当前证据");
    await act(async () => save.click());
    await settle();
    assert.equal(updates.length, 1);
    assert.ok(Object.values(updates[0]).every(Boolean));
    assert.match(view.document.body.textContent, /历史最高：\s*已彻底掌握 · 首次达到 2026-08-12/);
    await act(async () => checks[5].click());
    await act(async () => save.click());
    await settle();
    assert.equal(updates.length, 2);
    assert.equal(updates[1].transferSolvedIndependently, false);
    assert.match(view.document.body.textContent, /当前：\s*5\/6 项证据/);
    assert.match(view.document.body.textContent, /历史最高：\s*已彻底掌握/);
    assert.match(view.document.body.textContent, /回炉中/);
  } finally {
    await view.cleanup();
  }
});

test("Problem detail performs a Fresh Read when the window regains focus", {
  concurrency: false,
}, async () => {
  let projectionReads = 0;
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "personal",
      personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" },
      lifecycle: personalUnstartedLifecycle,
    };
    if (command === "personal_note_projection") {
      projectionReads += 1;
      const name = projectionReads === 1 ? "Before focus" : "After external edit";
      return { state: "ready", vaultRelativePath: "Problems/CF-1979-A.md", relocated: false, projection: {
        contentDigest: String(projectionReads), knownSections: [],
        solutionRoutes: [{ name, startOffset: 0, endOffset: 1 }], warnings: [],
      } };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    assert.match(view.document.body.textContent, /Before focus/);
    await act(async () => window.dispatchEvent(new window.Event("focus")));
    await settle();
    assert.equal(projectionReads, 2);
    assert.match(view.document.body.textContent, /After external edit/);
    assert.doesNotMatch(view.document.body.textContent, /Before focus/);
  } finally {
    await view.cleanup();
  }
});

test("Obsidian open failure is scoped and exposes recovery actions", {
  concurrency: false,
}, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return {
      contestId: 1979, index: "A", title: "Problem A", rating: 800,
      sourceUrl: "https://codeforces.com/contest/1979/problem/A",
      statement: { state: "pending" }, identityType: "personal",
      personalNote: { vaultRelativePath: "Problems/CF-1979-A.md" },
      lifecycle: personalUnstartedLifecycle,
    };
    if (command === "personal_note_projection") return { state: "ready", vaultRelativePath: "Problems/CF-1979-A.md", relocated: false, projection: { contentDigest: "fresh", knownSections: [], solutionRoutes: [], warnings: [] } };
    if (command === "open_personal_note_in_obsidian") throw "obsidian_open_failed";
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1979/A");
  try {
    const openButton = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "在 Obsidian 中打开并编辑题解");
    assert.ok(openButton);
    await act(async () => openButton.click());
    await settle();
    assert.match(view.document.body.textContent, /学习状态没有改变/);
    assert.match(view.document.body.textContent, /重试/);
    assert.match(view.document.body.textContent, /复制路径/);
    assert.match(view.document.body.textContent, /检查设置/);
    assert.match(view.document.body.textContent, /个人题目/);
  } finally {
    await view.cleanup();
  }
});

test("Today drives stable reorder, Done, explicit suggestions, and confirmed replanning", {
  concurrency: false,
}, async () => {
  const calls = [];
  const entry = (id, problemId, reason, status = "notStarted", origin = "auto") => ({
    entryId: id, problemId, reviewAttemptId: null, lane: "study", reason,
    problemTitle: `Problem ${problemId}`, problemRating: 1200,
    planningCostMinutes: 60, position: 0, origin, status,
  });
  let snapshot = {
    planId: "plan-today", localDate: "2026-08-12", budgetMinutes: 120,
    plannedMinutes: 120, overBudgetMinutes: 0, reviewOnlyStreak: 0,
    entries: [entry("entry-a", "1", "upsolve"), { ...entry("entry-b", "2", "relearn"), position: 1 }],
  };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    calls.push([command, args]);
    if (command === "today_snapshot") return snapshot;
    if (command === "reorder_today") {
      snapshot = { ...snapshot, entries: args.input.orderedEntryIds.map((id, position) => ({ ...snapshot.entries.find((item) => item.entryId === id), position })) };
      return snapshot;
    }
    if (command === "complete_today_entry") {
      snapshot = { ...snapshot, entries: snapshot.entries.map((item) => ({ ...item, status: "completed" })) };
      return snapshot;
    }
    if (command === "today_extra_suggestions") return {
      expectedSnapshot: snapshot, remainingBudgetMinutes: 60,
      suggestions: [{ problemId: "3", problemTitle: "Problem 3", problemRating: null, reviewAttemptId: null, lane: "study", reason: "upsolve", planningCostMinutes: 60 }],
    };
    if (command === "accept_today_extra_suggestion") {
      snapshot = { ...snapshot, plannedMinutes: 180, overBudgetMinutes: 60, entries: [...snapshot.entries, { ...entry("entry-c", "3", "upsolve", "notStarted", "manual"), position: 2 }] };
      return snapshot;
    }
    if (command === "preview_today_replan") return {
      expectedSnapshot: snapshot, proposedBudgetMinutes: args.input.budgetMinutes,
      proposedPlannedMinutes: 120, proposedOverBudgetMinutes: 25,
      proposedReviewOnlyStreak: 0,
      entries: snapshot.entries.map(({ entryId, position: _position, ...item }) => ({ ...item, existingEntryId: entryId })),
    };
    if (command === "apply_today_replan") {
      snapshot = { ...snapshot, budgetMinutes: args.preview.proposedBudgetMinutes, overBudgetMinutes: 25 };
      return snapshot;
    }
    throw new Error(`unexpected command ${command}`);
  }, "/today");
  try {
    assert.match(view.document.body.textContent, /补题/);
    const down = view.document.querySelector('button[aria-label="将“补题”任务下移"]');
    await act(async () => down.click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "reorder_today")[1].input.orderedEntryIds, ["entry-b", "entry-a"]);
    const firstEntry = view.document.querySelector(".today-entry");
    await act(async () => firstEntry.dispatchEvent(new view.window.KeyboardEvent("keydown", { altKey: true, key: "ArrowDown", bubbles: true })));
    await settle();
    assert.equal(calls.filter(([name]) => name === "reorder_today").length, 2);

    const entries = view.document.querySelectorAll(".today-entry");
    const handle = entries[0].querySelector(".today-drag-handle");
    let capturedPointer = null;
    handle.setPointerCapture = (pointerId) => { capturedPointer = pointerId; };
    handle.hasPointerCapture = (pointerId) => capturedPointer === pointerId;
    handle.releasePointerCapture = () => { capturedPointer = null; };
    view.document.elementFromPoint = () => entries[1];
    const pointerEvent = (type, fields = {}) => {
      const event = new view.window.Event(type, { bubbles: true, cancelable: true });
      Object.assign(event, { button: 0, pointerId: 7, clientX: 100, clientY: 200, ...fields });
      return event;
    };
    await act(async () => {
      handle.dispatchEvent(pointerEvent("pointerdown"));
      handle.dispatchEvent(pointerEvent("pointermove"));
      handle.dispatchEvent(pointerEvent("pointerup"));
    });
    await settle();
    const reorderCalls = calls.filter(([name]) => name === "reorder_today");
    assert.equal(reorderCalls.length, 3);
    assert.deepEqual(reorderCalls[2][1].input, {
      planId: "plan-today",
      orderedEntryIds: ["entry-b", "entry-a"],
    });

    const done = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "今日完成");
    await act(async () => done.click()); await settle();
    assert.ok(calls.some(([name]) => name === "complete_today_entry"));
    assert.match(view.document.body.textContent, /额外建议/);
    const add = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "加入今日计划");
    await act(async () => add.click()); await settle();
    assert.ok(calls.some(([name, args]) => name === "accept_today_extra_suggestion" && args.input.problemId === "3"));
    assert.match(view.document.body.textContent, /手动/);

    const budget = view.document.querySelector('.today-toolbar input[type="number"]');
    await act(async () => {
      budget.value = "";
      budget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    assert.equal(budget.value, "", "the controlled budget input must remain clear while the user replaces its value");
    await act(async () => {
      budget.value = "95";
      budget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    assert.equal(budget.value, "95");
    const preview = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "预览重新规划");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(budget, "-1");
      budget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      budget.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await settle();
    await act(async () => preview.click());
    await settle();
    assert.match(view.document.body.textContent, /每日预算必须是非负整数分钟数/);
    assert.equal(calls.filter(([name]) => name === "preview_today_replan").length, 0);
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(budget, "95");
      budget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      budget.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /应用这次重新规划/);
    const apply = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "应用重新规划");
    assert.equal(view.document.activeElement, apply, "replan dialog focuses its primary action");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    assert.equal(view.document.querySelector('[role="dialog"]'), null, "Escape closes the replan dialog");
    assert.equal(view.document.activeElement, preview, "closing the replan returns focus to its trigger");
    await act(async () => preview.click()); await settle();
    const reopenedApply = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "应用重新规划");
    await act(async () => reopenedApply.click()); await settle();
    assert.ok(calls.some(([name]) => name === "apply_today_replan"));
    assert.ok([...view.document.querySelectorAll(".sr-only")].some((node) => /今日重新规划已应用/.test(node.textContent ?? "")));
  } finally {
    await view.cleanup();
  }
});

test("Today navigation uses canonical problem ids and review attempt ids without aliases", {
  concurrency: false,
}, async () => {
  const attemptId = "018f0d8e-4a5b-7c6d-8e9f-0123456789ab";
  const snapshot = {
    planId: "plan-navigation", localDate: "2026-08-12", budgetMinutes: 90,
    plannedMinutes: 90, overBudgetMinutes: 0, reviewOnlyStreak: 0,
    entries: [
      {
        entryId: "ordinary", problemId: "canonical-42", problemTitle: "Opaque identity",
        problemRating: null, reviewAttemptId: null, lane: "study", reason: "upsolve",
        planningCostMinutes: 60, position: 0, origin: "auto", status: "notStarted",
      },
      {
        entryId: "review", problemId: "canonical-43", problemTitle: "Continue review",
        problemRating: 1700, reviewAttemptId: attemptId, lane: "carryIn", reason: "continueReview",
        planningCostMinutes: 30, position: 1, origin: "auto", status: "inProgress",
      },
    ],
  };
  const invoke = (command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "today_snapshot") return snapshot;
    throw new Error(`unexpected command ${command}`);
  };

  const ordinary = await renderApp(invoke, "/today");
  try {
    const links = ordinary.document.querySelectorAll(".today-problem-link");
    await act(async () => links[0].click());
    assert.equal(ordinary.window.location.pathname, "/problems/id/canonical-42");
  } finally {
    await ordinary.cleanup();
  }

  const review = await renderApp(invoke, "/today");
  try {
    const links = review.document.querySelectorAll(".today-problem-link");
    await act(async () => links[1].click());
    assert.equal(review.window.location.pathname, `/review/${attemptId}`);
  } finally {
    await review.cleanup();
  }
});

test("Today asks for a budget before creating the first daily snapshot", {
  concurrency: false,
}, async () => {
  const loads = [];
  const empty = {
    planId: "new-plan", localDate: "2026-08-12", budgetMinutes: 90,
    plannedMinutes: 0, overBudgetMinutes: 0, reviewOnlyStreak: 0, entries: [],
  };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "today_snapshot") {
      loads.push(args.input.budgetMinutes);
      return args.input.budgetMinutes === null ? null : { ...empty, budgetMinutes: args.input.budgetMinutes };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/today");
  try {
    assert.deepEqual(loads, [null]);
    assert.match(view.document.body.textContent, /设置今日预算/);
    const create = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "创建今日计划");
    const initialBudget = view.document.querySelector(".today-budget-start input");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(initialBudget, "");
      initialBudget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      initialBudget.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await settle();
    await act(async () => create.click());
    await settle();
    assert.deepEqual(loads, [null]);
    assert.match(view.document.body.textContent, /每日预算必须是非负整数分钟数/);
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(initialBudget, "60");
      initialBudget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      initialBudget.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await act(async () => create.click()); await settle();
    assert.deepEqual(loads, [null, 60]);
    assert.match(view.document.body.textContent, /没有任务适合当前预算/);
  } finally { await view.cleanup(); }
});

test("Problem Detail can save a missing Knowledge intent without creating authority", {
  concurrency: false,
}, async () => {
  const calls = [];
  const candidate = {
    contestId: 1, problemIndex: "A", fingerprint: "a".repeat(64), targetRef: "Segment Tree", disposition: "pending",
  };
  const detail = {
    contestId: 1, index: "A", title: "Candidate Problem", rating: null,
    sourceUrl: "https://codeforces.com/contest/1/problem/A", identityType: "personal",
    statement: { state: "pending" }, personalNote: { vaultRelativePath: "Problems/1-A.md" },
    lifecycle: { learningStatus: "unstarted", learningStatusSinceUtc: "2026-08-13T00:00:00.000Z", nextReviewDueLocalDate: null, availableActions: [] }, reviewAction: null,
  };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return detail;
    if (command === "personal_note_projection") return { state: "ready", vaultRelativePath: "Problems/1-A.md", relocated: false, projection: { knownSections: [], solutionRoutes: [], warnings: [] } };
    if (command === "knowledge_candidates") { calls.push([command, args]); return [candidate]; }
    if (command === "set_knowledge_candidate_disposition") { calls.push([command, args]); return { ...candidate, disposition: args.input.disposition }; }
    if (command === "review_history") return { items: [], mastery: null };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1/A");
  try {
    await settle();
    assert.equal(calls.filter(([name]) => name === "knowledge_candidates").length, 1);
    assert.match(view.document.body.textContent, /Segment Tree/);
    const saveIntent = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "只保存意图");
    assert.ok(saveIntent);
    await act(async () => saveIntent.click()); await settle();
    assert.equal(calls.at(-1)[0], "set_knowledge_candidate_disposition");
    assert.equal(calls.at(-1)[1].input.disposition, "acceptedIntent");
    assert.match(view.document.body.textContent, /仅保存意图/);
    const ignore = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "不再建议");
    await act(async () => ignore.click()); await settle();
    assert.equal(calls.at(-1)[0], "set_knowledge_candidate_disposition");
    assert.equal(calls.at(-1)[1].input.disposition, "ignored");
    assert.equal(calls.some(([, args]) => args?.input?.disposition === "acceptedIntent"), true);
    assert.match(view.document.body.textContent, /已忽略建议/);
  } finally { await view.cleanup(); }
});

test("Problem Detail accepts only a uniquely resolved existing Knowledge Node through Safe Patch", {
  concurrency: false,
}, async () => {
  const calls = [];
  const candidate = {
    contestId: 1, problemIndex: "A", fingerprint: "b".repeat(64), targetRef: "Segment Tree",
    disposition: "pending", knowledgeNodeId: "018f0d8e-4a5b-7c6d-8e9f-1123456789ab",
  };
  let candidates = [candidate];
  const detail = {
    contestId: 1, index: "A", title: "Safe Patch Problem", rating: null,
    sourceUrl: "https://codeforces.com/contest/1/problem/A", identityType: "personal",
    statement: { state: "pending" }, personalNote: { vaultRelativePath: "Problems/1-A.md" },
    lifecycle: { learningStatus: "unstarted", learningStatusSinceUtc: "2026-08-13T00:00:00.000Z", nextReviewDueLocalDate: null, availableActions: [] }, reviewAction: null,
  };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return detail;
    if (command === "personal_note_projection") return { state: "ready", vaultRelativePath: "Problems/1-A.md", relocated: false, projection: { knownSections: [{ name: "前置知识" }], solutionRoutes: [], warnings: [] } };
    if (command === "knowledge_candidates") return candidates;
    if (command === "accept_existing_knowledge_candidate") {
      calls.push([command, args]); candidates = [];
      return { knowledgeNodeId: candidate.knowledgeNodeId, targetRef: candidate.targetRef };
    }
    if (command === "review_history") return { items: [], mastery: null };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1/A");
  try {
    await settle();
    const accept = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "接受现有知识");
    assert.ok(accept);
    await act(async () => accept.click()); await settle();
    assert.equal(calls.length, 1);
    assert.equal(calls[0][1].input.knowledgeNodeId, candidate.knowledgeNodeId);
    assert.match(view.document.body.textContent, /知识链接已写入当前 Markdown，并经重新读取验证为正式关系/);
    assert.match(view.document.body.textContent, /当前个人题目没有前置知识建议/);
  } finally { await view.cleanup(); }
});

test("Problem Detail requires a second explicit action after accepted intent later resolves", {
  concurrency: false,
}, async () => {
  const calls = [];
  const candidate = {
    contestId: 1, problemIndex: "A", fingerprint: "c".repeat(64), targetRef: "Fenwick Tree",
    disposition: "acceptedIntent", knowledgeNodeId: "018f0d8e-4a5b-7c6d-8e9f-2123456789ab",
  };
  let candidates = [candidate];
  const detail = {
    contestId: 1, index: "A", title: "Intent Problem", rating: null,
    sourceUrl: "https://codeforces.com/contest/1/problem/A", identityType: "personal",
    statement: { state: "pending" }, personalNote: { vaultRelativePath: "Problems/1-A.md" },
    lifecycle: { learningStatus: "unstarted", learningStatusSinceUtc: "2026-08-13T00:00:00.000Z", nextReviewDueLocalDate: null, availableActions: [] }, reviewAction: null,
  };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "lightweight_problem_detail") return detail;
    if (command === "personal_note_projection") return { state: "ready", vaultRelativePath: "Problems/1-A.md", relocated: false, projection: { knownSections: [], solutionRoutes: [], warnings: [] } };
    if (command === "knowledge_candidates") return candidates;
    if (command === "accept_existing_knowledge_candidate") { calls.push([command, args]); candidates = []; return { knowledgeNodeId: candidate.knowledgeNodeId, targetRef: candidate.targetRef }; }
    if (command === "review_history") return { items: [], mastery: null };
    throw new Error(`unexpected command ${command}`);
  }, "/problems/1/A");
  try {
    await settle();
    assert.match(view.document.body.textContent, /已接受意图 · 已找到对应知识 Markdown/);
    assert.equal(calls.length, 0);
    const accept = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "接受现有知识");
    assert.ok(accept);
    await act(async () => accept.click()); await settle();
    assert.equal(calls.length, 1);
  } finally { await view.cleanup(); }
});

test("Settings saves optional arbitrary-minute weekly defaults without touching Today", {
  concurrency: false,
}, async () => {
  const calls = [];
  let schedule = { monday: null, tuesday: null, wednesday: 95, thursday: null, friday: null, saturday: 101, sunday: 0 };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "weekly_acm_budget") return schedule;
    if (command === "save_weekly_acm_budget") {
      calls.push(args.schedule);
      schedule = args.schedule;
      return schedule;
    }
    throw new Error(`unexpected command ${command}`);
  }, "/settings");
  try {
    await settle();
    const wednesday = view.document.querySelector('input[aria-label="星期三 的 ACM 预算分钟数"]');
    const thursday = view.document.querySelector('input[aria-label="星期四 的 ACM 预算分钟数"]');
    assert.equal(wednesday.value, "95");
    assert.equal(thursday.value, "");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(wednesday, "73");
      wednesday.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await settle();
    assert.equal(wednesday.value, "73");
    const save = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "保存每周预算");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(wednesday, "-1");
      wednesday.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await settle();
    await act(async () => save.click());
    await settle();
    assert.equal(calls.length, 0);
    assert.match(view.document.body.textContent, /每周预算必须留空或填写非负整数分钟数/);
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(wednesday, "73");
      wednesday.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await act(async () => save.click()); await settle();
    assert.equal(calls.length, 1);
    assert.equal(calls[0].wednesday, 73);
    assert.equal(calls[0].thursday, null);
    assert.match(view.document.body.textContent, /现有今日计划和单日覆盖值没有改变/);
  } finally { await view.cleanup(); }
});

test("Knowledge discovers Markdown, loads Fresh detail, and changes understanding only after confirmation", {
  concurrency: false,
}, async () => {
  const calls = [];
  const node = {
    knowledgeNodeId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
    displayName: "Segment Tree",
    vaultRelativePath: "Knowledge/Data Structures/Segment Tree.md",
    contentDigest: "a".repeat(64),
    locationState: "ready",
  };
  let understanding = null;
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "knowledge_index") { calls.push([command, args]); return { nodes: [node], locationAnomalies: [], identityConflicts: [] }; }
    if (command === "knowledge_detail") { calls.push([command, args]); return { node, understanding, incoming: [], outgoing: [], relatedProblems: [{ problemId: "1", title: "Theatre Square" }] }; }
    if (command === "knowledge_reevaluation_suggestion") { calls.push([command, args]); return { knowledgeNodeId: node.knowledgeNodeId, shouldSuggest: true, qualifyingProblemCount: 3 }; }
    if (command === "confirm_knowledge_understanding") {
      calls.push([command, args]);
      understanding = { knowledgeNodeId: node.knowledgeNodeId, current: args.input.level, historicalHighest: args.input.level, firstReachedHighestOn: "2026-08-13" };
      return understanding;
    }
    throw new Error(`unexpected command ${command}`);
  }, "/knowledge");
  try {
    assert.equal(calls[0][0], "knowledge_index");
    assert.equal(calls[0][1].input.query, "");
    assert.match(view.document.body.textContent, /Segment Tree/);
    const nodeButton = [...view.document.querySelectorAll("button")].find((button) => button.textContent.includes("Segment Tree"));
    await act(async () => nodeButton.click()); await settle();
    assert.equal(calls.filter(([name]) => name === "knowledge_detail").length, 1);
    assert.equal(calls.filter(([name]) => name === "knowledge_reevaluation_suggestion").length, 1);
    const relatedProblem = [...view.document.querySelectorAll("button")].find((button) => button.textContent.includes("Theatre Square"));
    assert.ok(relatedProblem);
    assert.match(view.document.body.textContent, /建议重新评估此知识状态：3 道相关题目获得了新的“真会”复习证据/);
    assert.match(view.document.body.textContent, /当前状态没有改变/);
    assert.match(view.document.body.textContent, /尚未有用户确认的状态/);
    const select = view.document.querySelector('select[aria-label="当前理解程度"]');
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(select, "basic");
      select.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await settle();
    assert.equal(calls.filter(([name]) => name === "confirm_knowledge_understanding").length, 0);
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "确认状态");
    await act(async () => confirm.click()); await settle();
    assert.equal(calls.at(-1)[0], "confirm_knowledge_understanding");
    assert.equal(calls.at(-1)[1].input.level, "basic");
    assert.match(view.document.body.textContent, /历史最高：\s*基本理解/);
    await act(async () => relatedProblem.click()); await settle();
    assert.equal(view.window.location.pathname, "/problems/id/1");
  } finally { await view.cleanup(); }
});

test("Knowledge opens the configured Vault's native Obsidian Graph command", {
  concurrency: false,
}, async () => {
  const calls = [];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "knowledge_index") return { nodes: [], locationAnomalies: [], identityConflicts: [] };
    if (command === "open_obsidian_graph") { calls.push([command, args]); return "confirmedSuccess"; }
    throw new Error(`unexpected command ${command}`);
  }, "/knowledge");
  try {
    const openGraph = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "打开 Obsidian 图谱");
    assert.ok(openGraph);
    await act(async () => openGraph.click()); await settle();
    assert.deepEqual(calls, [["open_obsidian_graph", { vaultContext: managerFixture.context }]]);
  } finally { await view.cleanup(); }
});

test("Knowledge reports an Obsidian Graph open failure without hiding the page", {
  concurrency: false,
}, async () => {
  let attempts = 0;
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "knowledge_index") return { nodes: [], locationAnomalies: [], identityConflicts: [] };
    if (command === "open_obsidian_graph") {
      attempts += 1;
      if (attempts === 1) throw "obsidian_graph_open_failed";
      return "submittedUnconfirmed";
    }
    throw new Error(`unexpected command ${command}`);
  }, "/knowledge");
  try {
    const openGraph = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "打开 Obsidian 图谱");
    await act(async () => openGraph.click()); await settle();
    assert.match(view.document.body.textContent, /Obsidian 图谱无法打开/);
    assert.match(view.document.body.textContent, /知识库索引/);
    await act(async () => openGraph.click()); await settle();
    const notice = view.document.querySelector(".safe-note");
    assert.ok(notice);
    assert.match(notice.textContent, /尚未收到完成确认/);
    assert.equal(view.document.querySelector(".error-copy"), null);
  } finally { await view.cleanup(); }
});

test("Knowledge reports disabled Obsidian CLI with actionable guidance", {
  concurrency: false,
}, async () => {
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "knowledge_index") return { nodes: [], locationAnomalies: [], identityConflicts: [] };
    if (command === "open_obsidian_graph") throw "obsidian_cli_disabled";
    throw new Error(`unexpected command ${command}`);
  }, "/knowledge");
  try {
    const openGraph = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "打开 Obsidian 图谱");
    await act(async () => openGraph.click()); await settle();
    assert.match(view.document.body.textContent, /Obsidian/);
    assert.match(view.document.body.textContent, /命令行界面/);
    assert.match(view.document.body.textContent, /设置/);
    assert.match(view.document.body.textContent, /重试/);
  } finally { await view.cleanup(); }
});

test("Knowledge location anomaly requires explicit candidate selection before rebind", {
  concurrency: false,
}, async () => {
  const calls = [];
  const anomaly = {
    knowledgeNodeId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
    displayName: "Old Segment Tree",
    vaultRelativePath: "Knowledge/Old Segment Tree.md",
    contentDigest: "a".repeat(64),
    locationState: "locationAnomaly",
  };
  let repaired = false;
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "knowledge_index") {
      calls.push([command, args]);
      return repaired ? { nodes: [{ ...anomaly, vaultRelativePath: "Archive/Segment Tree.md", locationState: "ready" }], locationAnomalies: [] } : { nodes: [], locationAnomalies: [anomaly] };
    }
    if (command === "knowledge_relocation_candidates") {
      calls.push([command, args]);
      return [
        { vaultRelativePath: "Knowledge/Occupied.md", occupied: true },
        { vaultRelativePath: "Archive/Segment Tree.md", occupied: false },
      ];
    }
    if (command === "rebind_knowledge_node") {
      calls.push([command, args]);
      repaired = true;
      return { ...anomaly, vaultRelativePath: args.input.vaultRelativePath, locationState: "ready" };
    }
    throw new Error(`unexpected command ${command}`);
  }, "/knowledge");
  try {
    assert.match(view.document.body.textContent, /Old Segment Tree/);
    assert.equal(calls.filter(([name]) => name === "knowledge_relocation_candidates").length, 0);
    const find = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "查找可能的位置");
    await act(async () => find.click()); await settle();
    assert.equal(calls.at(-1)[0], "knowledge_relocation_candidates");
    assert.equal(calls.at(-1)[1].input.knowledgeNodeId, anomaly.knowledgeNodeId);
    const occupied = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "使用此 Markdown" && button.parentElement.textContent.includes("Occupied.md"));
    assert.equal(occupied.disabled, true);
    assert.match(view.document.body.textContent, /已绑定到其他主对象/);
    const use = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "使用此 Markdown" && button.parentElement.textContent.includes("Archive/Segment Tree.md"));
    await act(async () => use.click()); await settle();
    const rebind = calls.find(([name]) => name === "rebind_knowledge_node");
    assert.equal(rebind[1].input.knowledgeNodeId, anomaly.knowledgeNodeId);
    assert.equal(rebind[1].input.vaultRelativePath, "Archive/Segment Tree.md");
    assert.doesNotMatch(view.document.body.textContent, /location anomaly and require recovery/);
  } finally { await view.cleanup(); }
});

test("Knowledge deletion confirmation previews consequences before mutating", {
  concurrency: false,
}, async () => {
  const calls = [];
  const anomaly = {
    knowledgeNodeId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
    displayName: "Deleted Knowledge",
    vaultRelativePath: "Knowledge/Deleted Knowledge.md",
    contentDigest: "a".repeat(64),
    locationState: "locationAnomaly",
  };
  let deleted = false;
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "knowledge_index") {
      calls.push([command, args]);
      return deleted ? { nodes: [], locationAnomalies: [], identityConflicts: [] } : { nodes: [], locationAnomalies: [anomaly], identityConflicts: [] };
    }
    if (command === "knowledge_relocation_candidates") return [];
    if (command === "confirm_knowledge_markdown_deleted") { calls.push([command, args]); deleted = true; return null; }
    throw new Error(`unexpected command ${command}`);
  }, "/knowledge");
  try {
    const preview = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "确认文件已删除…");
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /此操作不会删除任何文件/);
    assert.equal(calls.some(([name]) => name === "confirm_knowledge_markdown_deleted"), false);
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "确认已删除");
    await act(async () => confirm.click()); await settle();
    assert.equal(calls.find(([name]) => name === "confirm_knowledge_markdown_deleted")[1].input.knowledgeNodeId, anomaly.knowledgeNodeId);
    assert.doesNotMatch(view.document.body.textContent, /location anomaly and require recovery/);
  } finally { await view.cleanup(); }
});

test("Knowledge same-name rebuild requires explicit identity choice", {
  concurrency: false,
}, async () => {
  const calls = [];
  const conflict = {
    historicalKnowledgeNodeId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
    displayName: "Segment Tree",
    candidateVaultRelativePath: "Knowledge/Segment Tree.md",
  };
  const node = {
    knowledgeNodeId: conflict.historicalKnowledgeNodeId,
    displayName: conflict.displayName,
    vaultRelativePath: conflict.candidateVaultRelativePath,
    contentDigest: "a".repeat(64),
    locationState: "ready",
  };
  let resolved = false;
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "knowledge_index") {
      calls.push([command, args]);
      return resolved ? { nodes: [{ ...node, vaultRelativePath: conflict.candidateVaultRelativePath }], locationAnomalies: [], identityConflicts: [] } : { nodes: [], locationAnomalies: [], identityConflicts: [conflict] };
    }
    if (command === "resolve_knowledge_identity_conflict") { calls.push([command, args]); resolved = true; return { ...node, vaultRelativePath: conflict.candidateVaultRelativePath }; }
    throw new Error(`unexpected command ${command}`);
  }, "/knowledge");
  try {
    assert.match(view.document.body.textContent, /系统没有自动猜测其身份/);
    assert.match(view.document.body.textContent, /恢复旧知识节点/);
    assert.match(view.document.body.textContent, /创建新知识节点/);
    const restore = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "恢复旧知识节点");
    await act(async () => restore.click()); await settle();
    const call = calls.find(([name]) => name === "resolve_knowledge_identity_conflict");
    assert.equal(call[1].input.historicalKnowledgeNodeId, conflict.historicalKnowledgeNodeId);
    assert.equal(call[1].input.candidateVaultRelativePath, conflict.candidateVaultRelativePath);
    assert.equal(call[1].input.restoreOldIdentity, true);
    assert.doesNotMatch(view.document.body.textContent, /系统没有自动猜测其身份/);
  } finally { await view.cleanup(); }
});

test("Settings previews manual backup before creating it", { concurrency: false }, async () => {
  const calls = [];
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "preview_manual_backup") { calls.push(command); return { schemaVersion: 23, backupDirectory: "backups/manual", filenamePrefix: "manual-schema-23-" }; }
    if (command === "create_manual_backup") { calls.push(command); return { path: "backups/manual/manual-schema-23-1.sqlite3", schemaVersion: 23 }; }
    if (command === "backup_inventory") { calls.push(command); return { dailyKeep: 7, weeklyKeep: 4, entries: [{ path: "backups/manual/manual-schema-23-1.sqlite3", category: "manual", sizeBytes: 4096, integrityVerified: true, retention: "protected" }] }; }
    if (command === "weekly_acm_budget") return { monday: null, tuesday: null, wednesday: null, thursday: null, friday: null, saturday: null, sunday: null };
    throw new Error(`unexpected command ${command}`);
  }, "/settings");
  try {
    const preview = [...view.document.querySelectorAll("button")].find((b) => b.textContent === "预览手动备份");
    await act(async () => preview.click()); await settle();
    assert.equal(calls[0], "preview_manual_backup");
    assert.match(view.document.body.textContent, /数据库结构版本 23/);
    assert.equal(calls.includes("create_manual_backup"), false);
    const create = [...view.document.querySelectorAll("button")].find((b) => b.textContent === "创建备份");
    await act(async () => create.click()); await settle();
    const createIndex = calls.indexOf("create_manual_backup");
    const inventoryIndex = calls.lastIndexOf("backup_inventory");
    assert.ok(createIndex >= 0);
    assert.ok(inventoryIndex > createIndex);
    assert.match(view.document.body.textContent, /备份已创建/);
    assert.match(view.document.body.textContent, /保留 7 个每日快照和 4 个每周快照/);
    assert.match(view.document.body.textContent, /完整性已验证/);
    assert.match(view.document.body.textContent, /protected/);
    assert.equal([...view.document.querySelectorAll("button")].filter((button) => !button.closest("[data-testid=vault-manager]")).some((button) => /delete|prune/i.test(button.textContent)), false);
  } finally { await view.cleanup(); }
});


test("Problem library selection marks membership, batches only checked items, and cancels without writes", async () => {
  const added = [];
  const members = new Set([1]);
  const detail = { contestId: 1979, title: "Contest", sourceUrl: "https://codeforces.com/contest/1979", contestDate: null, importStatus: "complete", factsStatus: "pending", problems: [], corrections: [], aiAnalysis: null, archived: false };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_detail") return detail;
    if (command === "preview_problem_library") return { title: "Contest", problems: ["A", "B", "C"].map((index, i) => ({ problemId: i + 1, index, title: "Problem " + index, alreadyAdded: members.has(i + 1) })) };
    if (command === "add_problem_library_items") { added.push(args.input); args.input.problemIds.forEach((id) => members.add(id)); return null; }
    throw new Error("unexpected command " + command);
  }, "/contests/1979");
  try {
    await settle();
    const open = () => [...view.document.querySelectorAll("button")].find((b) => b.textContent === "选择比赛题加入我的题库");
    await act(async () => open().click()); await settle();
    let panel = view.document.querySelector('[aria-label="加入我的题库"]');
    assert.match(panel.textContent, /共 3 道比赛题/);
    const inputs = panel.querySelectorAll('input[type="checkbox"]');
    assert.equal(inputs[0].disabled, true);
    assert.equal(inputs[0].checked, true);
    assert.equal(inputs[1].checked, false);
    assert.equal(inputs[2].checked, false);
    await act(async () => inputs[1].click());
    const completionInput = panel.querySelector('input[type="date"]');
    assert.match(completionInput.value, /^\d{4}-\d{2}-\d{2}$/);
    await act(async () => { completionInput.value = "9998-12-31"; completionInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await act(async () => [...panel.querySelectorAll("button")].find((b) => b.textContent === "加入我的题库（1）").click());
    assert.equal(added.length, 0);
    assert.match(panel.textContent, /不晚于今天/);
    await act(async () => { completionInput.value = "2026-09-30"; completionInput.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await act(async () => [...panel.querySelectorAll("button")].find((b) => b.textContent === "加入我的题库（1）").click()); await settle();
    assert.deepEqual(added, [{ contestId: 1979, problemIds: [2], completionDate: "2026-09-30" }]);
    assert.equal(panel.querySelectorAll('input[type="checkbox"]')[1].disabled, true);
    await act(async () => [...panel.querySelectorAll("button")].find((b) => b.textContent === "完成").click());
    await act(async () => open().click()); await settle();
    panel = view.document.querySelector('[aria-label="加入我的题库"]');
    await act(async () => panel.querySelectorAll('input[type="checkbox"]')[2].click());
    await act(async () => [...panel.querySelectorAll("button")].find((b) => b.textContent === "取消").click());
    assert.equal(added.length, 1);
    assert.equal(view.document.querySelector('[aria-label="加入我的题库"]'), null);
  } finally { await view.cleanup(); }
});


test("Contest participation supports manual VP, edits, retry-safe saves and confirmed removal", { concurrency: false }, async () => {
  let records = [];
  const saves = [];
  let failSave = true;
  let resolveSave;
  const detail = { contestId: 1979, title: "Participation round", sourceUrl: "https://codeforces.com/contest/1979", contestDate: "2020-01-01", importStatus: "complete", factsStatus: "pending", problems: [], corrections: [], aiAnalysis: null, archived: false };
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", workspace: configuredWorkspace };
    if (command === "contest_detail") return detail;
    if (command === "contest_library_list_placements") return [];
    if (command === "save_contest_participation") {
      saves.push(args.input);
      if (failSave) return Promise.reject("participation_unavailable");
      return new Promise(resolve => { resolveSave = () => { records = [{ ...args.input, contestId: "1979", contestTitle: detail.title }]; resolve(null); }; });
    }
    if (command === "delete_contest_participation") { records = []; return null; }
    throw Error(`unexpected command ${command}`);
  }, "/contests/1979", false, 699, null, null, () => records);
  try {
    await settle();
    const panel = () => view.document.querySelector(".contest-participations");
    const button = label => [...panel().querySelectorAll("button")].find(node => node.textContent === label);
    assert.match(panel().textContent, /尚未记录参赛/);
    await act(async () => button("记录参赛").click());
    const form = () => panel().querySelector("form");
    const select = form().querySelector("select");
    assert.equal(view.document.activeElement, select);
    await act(async () => { select.value = "virtual"; select.dispatchEvent(new view.window.Event("change", { bubbles: true })); });
    const fill = async (selector, value) => {
      const input = form().querySelector(selector);
      await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, value); input.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    };
    await fill('[type="date"]', "2026-10-15");
    await fill('[type="time"]', "23:30");
    await fill('[type="number"]', "3");
    await fill('fieldset label:last-child input', "7");
    const submit = async () => act(async () => form().dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    await submit(); await settle();
    assert.match(panel().textContent, /操作未确认成功/);
    assert.equal(saves[0].kind, "virtual");
    assert.equal(saves[0].localDate, "2026-10-15", "original contest date is not used for VP");
    assert.equal(saves[0].localTime, "23:30");
    failSave = false;
    await submit();
    assert.equal(button("保存中…").disabled, true);
    await submit();
    assert.equal(saves.length, 2, "busy save cannot be submitted twice");
    assert.equal(saves[0].id, saves[1].id, "failed save retry keeps its record identity");
    await act(async () => resolveSave()); await settle();
    assert.match(panel().textContent, /VP.*2026-10-15 23:30.*赛内AC 3/);
    await act(async () => button("修改记录").click());
    const editSelect = form().querySelector("select");
    await act(async () => { editSelect.value = "live"; editSelect.dispatchEvent(new view.window.Event("change", { bubbles: true })); });
    await submit(); await act(async () => resolveSave()); await settle();
    assert.equal(saves[2].id, saves[1].id);
    assert.equal(saves[2].kind, "live");
    await act(async () => button("删除记录").click());
    assert.equal(records.length, 1, "opening delete confirmation does not remove history");
    await act(async () => button("取消").click());
    assert.equal(records.length, 1);
    await act(async () => button("删除记录").click());
    await act(async () => button("确认删除参赛记录").click()); await settle();
    assert.equal(records.length, 0);
    assert.match(panel().textContent, /尚未记录参赛/);
  } finally { await view.cleanup(); }
});

test("Learning calendar adjusts a late entry across months, retries failures and prevents duplicate saves", { concurrency: false }, async () => {
  let date = "2026-10-20";
  let writes = 0;
  let failSave = true;
  let release;
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", workspace: configuredWorkspace };
    if (command === "library_completion_dates") return [{ problemId: "51", problemTitle: "Late problem", date }, { problemId: "52", problemTitle: "Other problem", date: null }];
    if (command === "save_library_completion_date") {
      writes++;
      assert.equal(args.input.problemId, "51");
      assert.equal(args.input.date, "2026-09-30");
      assert.ok(args.vaultContext, "writes must carry the vault context");
      if (failSave) throw "completion_date_unavailable";
      return new Promise(resolve => { release = () => { date = args.input.date; resolve(null); }; });
    }
    throw Error(`unexpected command ${command}`);
  }, "/calendar", false, 699, ({ input }) => ({ today: "2026-10-22", entries: [
    { date, problemId: "51", problemTitle: "Late problem", kind: "library_added", status: "completed", dueDate: null, judgement: null },
    { date: "2026-09-30", problemId: "51", problemTitle: "Late problem", kind: "study", status: "completed", dueDate: null, judgement: null },
  ].filter(entry => Number(entry.date.slice(5, 7)) === input.month) }));
  try {
    await settle();
    const root = view.document.querySelector(".learning-calendar");
    await act(async () => root.querySelector('[data-calendar-date="2026-10-20"]').click());
    await act(async () => root.querySelector(".calendar-edit-date").click()); await settle();
    const form = () => root.querySelector(".calendar-completion-form");
    assert.equal(form().querySelector("select").value, "51");
    assert.equal(form().querySelector('[type="date"]').max, "2026-10-22");
    const input = async (selector, value) => act(async () => {
      const node = form().querySelector(selector);
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(node, value);
      node.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await input('[type="search"]', "Late");
    assert.equal(form().querySelectorAll("option").length, 2);
    await input('[type="date"]', "2026-10-23");
    await act(async () => form().dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true })));
    assert.equal(writes, 0);
    assert.match(form().textContent, /不晚于今天/);
    await input('[type="date"]', "2026-09-30");
    await act(async () => form().dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.equal(writes, 1);
    assert.match(form().textContent, /输入已保留/);
    assert.equal(form().querySelector('[type="date"]').value, "2026-09-30");
    failSave = false;
    await act(async () => {
      form().dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
      form().dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
    });
    assert.equal(writes, 2);
    assert.equal(form().querySelector('[type="submit"]').disabled, true);
    assert.equal([...form().querySelectorAll("button")].find(b => b.textContent === "取消").disabled, true);
    await act(async () => release()); await settle();
    assert.equal(form(), null);
    assert.match(root.querySelector("h2").textContent, /2026 年 9 月/);
    assert.equal(root.querySelector('[data-calendar-date="2026-09-30"]').getAttribute("aria-pressed"), "true");
    assert.match(root.querySelector(".calendar-overview").textContent, /本月做题 1 道/);
    assert.equal(root.querySelectorAll(".calendar-problem-link").length, 1, "same-day activities still deduplicate");
    assert.match(root.textContent, /做题日期已保存/);
    const next = [...root.querySelectorAll("button")].find(b => b.getAttribute("aria-label") === "下个月");
    await act(async () => next.click()); await settle();
    assert.match(root.querySelector(".calendar-overview").textContent, /本月做题 0 道/);
    await act(async () => root.querySelector('[data-calendar-date="2026-10-23"]').click());
    assert.equal([...root.querySelectorAll("button")].find(b => b.textContent === "补录 / 调整做题日期").disabled, true);
  } finally { await view.cleanup(); }
});

test("Learning calendar counts repeated VP separately, preserves daily problem totals and links contests", { concurrency: false }, async () => {
  const base = { contestId: "1979", contestTitle: "Training round", localDate: "2026-10-15", localTime: "23:30", acceptedCount: 0, totalCount: 7 };
  const participations = [{ ...base, id: "live", kind: "live" }, { ...base, id: "vp1", kind: "virtual", localTime: "08:30" }, { ...base, id: "vp2", kind: "virtual", localTime: "16:30" }, { ...base, id: "orphan", contestId: null, contestTitle: "Old round", kind: "virtual", localDate: "2026-10-14", acceptedCount: null }];
  const view = await renderApp(command => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", workspace: configuredWorkspace };
    if (command === "contest_detail") return { contestId: 1979, title: "Training round", sourceUrl: "https://codeforces.com/contest/1979", contestDate: "2020-01-01", importStatus: "complete", factsStatus: "pending", problems: [], corrections: [], aiAnalysis: null, archived: false };
    if (command === "contest_library_list_placements") return [];
    throw Error(`unexpected command ${command}`);
  }, "/calendar", false, 699, ({ input }) => ({ today: "2026-10-15", entries: [], participations: input.month === 10 ? participations : [] }));
  try {
    await settle();
    const root = view.document.querySelector(".learning-calendar");
    assert.match(root.querySelector(".calendar-overview").textContent, /本月做题 0 道.*现场 1 场 · VP 3 场 · 共 4 次参赛/);
    const day = root.querySelector('[data-calendar-date="2026-10-15"]');
    assert.match(day.textContent, /现场 1 场.*VP 2 场/);
    assert.match(day.getAttribute("aria-label"), /做题 0 道.*现场 1 场，VP 2 场/);
    const selected = () => root.querySelector(".calendar-detail");
    assert.equal(selected().querySelectorAll(".calendar-contest-link").length, 3);
    assert.match(selected().textContent, /23:30.*赛内AC 0\/7/);
    await act(async () => root.querySelector('[data-calendar-date="2026-10-14"]').click());
    assert.match(selected().textContent, /Old round · 比赛已删除.*赛内AC未记录/);
    assert.equal(selected().querySelectorAll("a").length, 0);
    await act(async () => day.click());
    await act(async () => selected().querySelector(".calendar-contest-link").click()); await settle();
    assert.equal(view.window.location.pathname, "/contests/1979");
  } finally { await view.cleanup(); }
});

test("Learning calendar shows history, overdue and future reviews and ignores stale month replies", { concurrency: false }, async () => {
  const snapshot = { planId: "calendar-plan", localDate: "2026-10-09", budgetMinutes: 60, plannedMinutes: 0, overBudgetMinutes: 0, reviewOnlyStreak: 0, entries: [] };
  const entries = [
    { date: "2026-10-08", problemId: "51", problemTitle: "Recorded learning", kind: "study", status: "completed", dueDate: null, judgement: null },
    { date: "2026-10-08", problemId: "51", problemTitle: "Recorded learning", kind: "library_added", status: "completed", dueDate: null, judgement: null },
    { date: "2026-10-08", problemId: "51", problemTitle: "Recorded learning", kind: "early_check", status: "completed", dueDate: "2026-10-08", judgement: "partial" },
    { date: "2026-10-08", problemId: "54", problemTitle: "Just added", kind: "library_added", status: "completed", dueDate: null, judgement: null },
    { date: "2026-10-07", problemId: "51", problemTitle: "Recorded learning", kind: "study", status: "completed", dueDate: null, judgement: null },
    { date: "2026-10-09", problemId: "52", problemTitle: "Overdue review", kind: "long_term_review", status: "scheduled", dueDate: "2026-10-05", judgement: null },
    { date: "2026-10-10", problemId: "53", problemTitle: "Next cold start", kind: "first_cold_start", status: "scheduled", dueDate: "2026-10-10", judgement: null },
  ];
  let resolveNovember;
  let rejectJanuary = true;
  const calls = [];
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "today_snapshot") return snapshot;
    throw new Error(`unexpected command ${command}`);
  }, "/calendar", false, 699, ({ input }) => {
    calls.push(input);
    if (input.month === 11) return new Promise((resolve) => { resolveNovember = resolve; });
    if (input.month === 1 && rejectJanuary) return Promise.reject(new Error("calendar unavailable"));
    return { today: snapshot.localDate, entries: input.month === 10 ? entries : [] };
  });
  try {
    await settle();
    assert.equal(view.document.querySelector("h1")?.textContent, "学习月历");
    assert.equal(view.document.activeElement, view.document.querySelector("h1"));
    assert.match(view.document.title, /学习月历/);
    assert.equal(view.document.querySelector('a[href="/calendar"]').getAttribute("aria-current"), "page");
    assert.equal(view.document.querySelector(".today-toolbar"), null);
    const root = view.document.querySelector(".learning-calendar");
    const detail = () => root.querySelector(".calendar-detail");
    const day = (key) => root.querySelector(`[data-calendar-date="${key}"]`);
    assert.equal(day("2026-10-09").getAttribute("aria-current"), "date");
    assert.match(detail().textContent, /已逾期.*2026-10-05/);
    const upcoming = [...root.querySelectorAll('[data-calendar-upcoming]')];
    assert.deepEqual(upcoming.map((row) => row.dataset.calendarUpcoming), ["2026-10-09", "2026-10-10"]);
    assert.match(upcoming[1].textContent, /Next cold start/);
    await act(async () => upcoming[1].click());
    assert.equal(day("2026-10-10").getAttribute("aria-pressed"), "true");
    assert.match(detail().textContent, /Next cold start/);

    await act(async () => day("2026-10-08").click());
    assert.match(detail().textContent, /Recorded learning/);
    assert.equal(day("2026-10-08").getAttribute("aria-label"), "2026-10-08，做题 2 道，待复习 0 道");
    assert.match(root.querySelector(".calendar-overview").textContent, /本月做题 3 道 · 待复习 2 道/);
    const doneLinks = detail().querySelectorAll("a.calendar-problem-link");
    assert.equal(doneLinks.length, 2, "same-day activities share one problem row");
    assert.match(doneLinks[0].textContent, /学习完成.*首次做题记录.*提前检验.*部分掌握/);
    assert.equal(doneLinks[1].getAttribute("href"), "/problems/id/54");
    assert.match(doneLinks[1].textContent, /Just added.*查看题解/);
    assert.doesNotMatch(detail().textContent, /Next cold start/);
    await act(async () => day("2026-10-10").click());
    assert.match(detail().textContent, /Next cold start.*首次冷启动/);
    await act(async () => day("2026-10-10").dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true })));
    assert.equal(view.document.activeElement, day("2026-10-09"));
    const next = () => root.querySelector('[aria-label="下个月"]');
    await act(async () => next().click()); await settle();
    assert.equal(root.querySelectorAll("[data-calendar-date]").length, 30);
    assert.match(detail().textContent, /正在读取/);
    await act(async () => next().click()); await settle();
    assert.equal(root.querySelectorAll("[data-calendar-date]").length, 31);
    await act(async () => resolveNovember({ today: snapshot.localDate, entries: [{ ...entries[0], date: "2026-12-09", problemTitle: "Stale response" }] }));
    assert.doesNotMatch(root.textContent, /Stale response/);
    await act(async () => next().click()); await settle();
    assert.match(root.textContent, /2027 年 1 月/);
    assert.match(detail().textContent, /月历读取失败/);
    assert.doesNotMatch(detail().textContent, /没有已完成/);
    rejectJanuary = false;
    await act(async () => [...detail().querySelectorAll("button")].find((button) => button.textContent === "重试").click()); await settle();
    assert.match(detail().textContent, /没有已完成/);
    await act(async () => [...root.querySelectorAll("button")].find((button) => button.textContent === "回到今天").click()); await settle();
    assert.equal(day("2026-10-09").getAttribute("aria-pressed"), "true");
    assert.ok(calls.some((call) => call.year === 2027 && call.month === 1));
    await act(async () => day("2026-10-10").click());
    await act(async () => detail().querySelector(".calendar-problem-link").click()); await settle();
    assert.equal(view.window.location.pathname, "/problems/id/53");
  } finally { await view.cleanup(); }
});


test("Today keeps its plan and opens the calendar through the sidebar without generating another plan", { concurrency: false }, async () => {
  let planReads = 0;
  let calendarReads = 0;
  const view = await renderApp((command) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "today_snapshot") { planReads++; return { planId: "calendar-nav", localDate: "2026-10-09", budgetMinutes: 60, plannedMinutes: 0, overBudgetMinutes: 0, reviewOnlyStreak: 0, entries: [] }; }
    throw new Error(`unexpected command ${command}`);
  }, "/today", false, 699, () => { calendarReads++; return { today: "2026-10-09", entries: [] }; });
  try {
    assert.ok(view.document.querySelector(".today-toolbar"));
    assert.equal(view.document.querySelector(".learning-calendar"), null);
    assert.equal(calendarReads, 0);
    const before = planReads;
    await act(async () => view.document.querySelector('a[href="/calendar"]').click()); await settle();
    assert.equal(view.window.location.pathname, "/calendar");
    assert.ok(view.document.querySelector(".learning-calendar"));
    assert.equal(planReads, before);
    assert.ok(calendarReads > 0);
    await act(async () => { view.window.history.back(); await new Promise((resolve) => view.window.addEventListener("popstate", resolve, { once: true })); });
    await settle();
    assert.equal(view.window.location.pathname, "/today");
    assert.ok(view.document.querySelector(".today-toolbar"));
    assert.equal(view.document.querySelector(".learning-calendar"), null);
  } finally { await view.cleanup(); }
});


test("Today overview derives real counts and resumes the canonical active review", { concurrency: false }, async () => {
  const { TodayOverview } = await moduleRunner.import("/src/app/TodayOverview.tsx");
  const navigations=[];
  const snapshot={planId:"overview",localDate:"2026-10-12",budgetMinutes:90,plannedMinutes:90,overBudgetMinutes:0,reviewOnlyStreak:0,entries:[
    {entryId:"completed",problemId:"51",problemTitle:"Completed study",problemRating:null,lane:"study",reason:"upsolve",planningCostMinutes:60,position:0,origin:"auto",status:"completed",reviewAttemptId:null},
    {entryId:"active",problemId:"52",problemTitle:"Active review",problemRating:1300,lane:"carryIn",reason:"continueReview",planningCostMinutes:30,position:1,origin:"auto",status:"inProgress",reviewAttemptId:"018f0d8e-4a5b-7c6d-8e9f-0123456789ab"}
  ]};
  const view=await render(React.createElement(TodayOverview,{snapshot,navigate:(path)=>navigations.push(path)}));
  try {
    const metrics=[...view.document.querySelectorAll(".today-metrics > div")].map(item=>item.textContent);
    assert.deepEqual(metrics,["计划任务2项","已完成1项","进行中1项","待处理1项","计划用时90分钟","剩余预算0分钟"]);
    assert.equal(view.document.querySelector(".today-composition").getAttribute("aria-label"),"学习 60 分钟，复习 30 分钟");
    await act(async()=>[...view.document.querySelectorAll("button")].find(button=>button.textContent==="继续学习").click());
    assert.deepEqual(navigations,["/review/018f0d8e-4a5b-7c6d-8e9f-0123456789ab"]);
  } finally {await view.cleanup();}
});


test("Problem library searches statement text and shows real import dates without injecting HTML", { concurrency: false }, async () => {
  const { ProblemLibrary } = await moduleRunner.import("/src/app/ProblemLibrary.tsx");
  const routes = [];
  const items = [
    { id: "cf", code: "2275A", title: "In Search of Convenience", source: "Codeforces", rating: 1200, importedAtUtc: "2026-10-09T04:05:06Z", statementHtml: '<div class="header">Duplicated title and limits</div><p>Find the shortest path &amp; its length.</p><img src="https://invalid.example/tracker.png"><script>bad()</script>', hasStatementSnapshot: true, route: "/problems/2275/A" },
    { id: "local", code: null, title: "Book exercise", source: "单题录入", rating: null, importedAtUtc: null, statementHtml: "<p>Count intervals</p>", hasStatementSnapshot: true, route: "/problems/id/77" },
  ];
  const view = await render(React.createElement(ProblemLibrary, { items, navigate: (route) => routes.push(route) }));
  try {
    assert.match(view.document.querySelector("time").textContent, /2026-10-09/);
    assert.equal(view.document.querySelector("time").getAttribute("datetime"), items[0].importedAtUtc);
    assert.match(view.document.body.textContent, /导入日期未记录/);
    assert.match(view.document.body.textContent, /2275A/);
    assert.match(view.document.body.textContent, /难度 1200/);
    assert.match(view.document.querySelector(".problem-library-preview").textContent, /path & its length/);
    assert.doesNotMatch(view.document.body.textContent, /Duplicated title|bad\(\)/);
    assert.equal(view.document.querySelector("img, script"), null);
    const input = view.document.querySelector('input[type="search"]');
    const search = async (value) => act(async () => { input.value = value; input.dispatchEvent(new view.document.defaultView.Event("input", { bubbles: true })); });
    await search("SHORTEST path");
    assert.equal(view.document.querySelectorAll(".problem-library-row").length, 1);
    await act(async () => view.document.querySelector(".problem-library-row").click());
    assert.deepEqual(routes, ["/problems/2275/A"]);
    await search("2275A");
    assert.equal(view.document.querySelectorAll(".problem-library-row").length, 1);
    await search("intervals");
    await act(async () => view.document.querySelector(".problem-library-row").click());
    assert.equal(routes[1], "/problems/id/77");
    await search("absent");
    assert.match(view.document.body.textContent, /没有找到匹配/);
  } finally { await view.cleanup(); }
});

test("Problem library renders LaTeX and keeps a complete formula across the preview cutoff", { concurrency: false }, async () => {
  const { ProblemLibrary } = await moduleRunner.import("/src/app/ProblemLibrary.tsx");
  const item = { id: "formula", code: "2275A", title: "Formula", source: "Codeforces", rating: null, hasStatementSnapshot: true, route: "/problems/2275/A" };
  const items = [
    { ...item, statementHtml: '<p>Point $$$(x_0,y_0)$$$, $n$, $$a^2$$ and \\(\\frac{1}{2}\\).</p>' },
    { ...item, id: "cutoff", statementHtml: `<p>${"x".repeat(174)} $$$\\frac{a_1+a_2}{b_1+b_2}$$$ trailing text.</p>` },
    { ...item, id: "unsafe", statementHtml: '<p>$$$\\href{javascript:alert(1)}{x}$$$ and $$$\\invalidcommand$$$</p>' },
  ];
  const view = await render(React.createElement(ProblemLibrary, { items, navigate: () => {} }));
  try {
    const previews = view.document.querySelectorAll(".problem-library-preview");
    assert.equal(previews[0].querySelectorAll(".katex").length, 4);
    assert.doesNotMatch(previews[0].textContent, /\$/);
    assert.equal(previews[1].querySelectorAll(".katex").length, 1);
    assert.equal(previews[1].querySelector("annotation").textContent, "\\frac{a_1+a_2}{b_1+b_2}");
    assert.match(previews[1].textContent, /…$/);
    assert.equal(view.document.querySelector('a[href^="javascript:"], img, script'), null);
    assert.equal(previews[2].querySelectorAll(".problem-library-math").length, 2);
    assert.match(previews[2].textContent, /invalidcommand/);
  } finally { await view.cleanup(); }
});

test("Single problem import accepts a book exercise without a contest and blocks duplicate submits", { concurrency: false }, async () => {
  const calls=[];
  let finish;
  const independent=[{problemId:"77",title:"Earlier book exercise",sourceUrl:"acmos://local-problem/earlier",rating:null,hasStatementSnapshot:true}];
  const view=await renderApp((command,args)=>{
    if(command==="foundation_status")return{status:"ready",core:"acm-os"};
    if(command==="app_shell_status")return{state:"normal",recoveryReason:null,supportedSchemaVersion:null,foundSchemaVersion:null,workspace:configuredWorkspace};
    if(command==="lightweight_problems")return[];
    if(command==="import_single_problem"){calls.push(args.input);return new Promise(resolve=>{finish=resolve;});}
    if(command==="statement_assets_by_id")return[];
    if(command==="lightweight_problem_detail_by_id")return{problemId:"88",externalIdentities:[],title:"My book exercise",rating:null,sourceUrl:"acmos://local-problem/local",statement:{state:"ready",sanitizedHtml:"<p>Find x</p>"},identityType:"lightweight",personalNote:null,lifecycle:lightweightLifecycle};
    throw new Error('unexpected command '+command);
  },"/problems",false,699,null,()=>independent);
  try {
    assert.match(view.document.querySelector('[aria-label="独立题目"]').textContent,/Earlier book exercise/);
    assert.equal(view.document.querySelector('.empty-state'),null);
    const form=view.document.querySelector('form[aria-label="单题导入"]');
    const field=(text)=>[...form.querySelectorAll('label')].find(label=>label.textContent.startsWith(text)).querySelector('input,textarea');
    assert.match(field('实际做题日期').value, /^\d{4}-\d{2}-\d{2}$/);
    await act(async()=>{field('题目标题').value='My book exercise';field('题目标题').dispatchEvent(new view.window.Event('input',{bubbles:true}));field('题面正文').value='Find x';field('题面正文').dispatchEvent(new view.window.Event('input',{bubbles:true}));});
    await act(async()=>{field('实际做题日期').value='9998-12-31';field('实际做题日期').dispatchEvent(new view.window.Event('input',{bubbles:true}));});
    await act(async()=>form.dispatchEvent(new view.window.Event('submit',{bubbles:true,cancelable:true})));
    assert.equal(calls.length, 0);
    assert.match(form.textContent, /不晚于今天/);
    await act(async()=>{field('实际做题日期').value='2026-09-30';field('实际做题日期').dispatchEvent(new view.window.Event('input',{bubbles:true}));});
    await act(async()=>{form.dispatchEvent(new view.window.Event('submit',{bubbles:true,cancelable:true}));form.dispatchEvent(new view.window.Event('submit',{bubbles:true,cancelable:true}));});
    assert.equal(calls.length,1);
    assert.equal(calls[0].mode,'manual');assert.equal(calls[0].title,'My book exercise');assert.equal(calls[0].sourceUrl,'');assert.equal(calls[0].statementText,'Find x');
    assert.equal(calls[0].completionDate,'2026-09-30');
    assert.equal(field('实际做题日期').disabled, true);
    assert.equal('contestId' in calls[0],false);
    assert.match(calls[0].requestId,/^[0-9a-f-]{36}$/);
    await act(async()=>finish({problemId:'88'}));await settle();
    assert.equal(view.window.location.pathname,'/problems/id/88');
    assert.match(view.document.body.textContent,/本地录入题目/);
    assert.equal(view.document.querySelector('a[href^="acmos:"]'),null);
  }finally{await view.cleanup();}
});

test("Single problem link import validates its URL and preserves the input after a remote error", { concurrency: false }, async () => {
  const calls=[];
  const view=await renderApp((command,args)=>{
    if(command==="foundation_status")return{status:"ready",core:"acm-os"};
    if(command==="app_shell_status")return{state:"normal",recoveryReason:null,supportedSchemaVersion:null,foundSchemaVersion:null,workspace:configuredWorkspace};
    if(command==="lightweight_problems")return[];
    if(command==="import_single_problem"){calls.push(args.input);throw 'single_problem_remote_unavailable';}
    throw new Error('unexpected command '+command);
  },"/problems");
  try{
    const root=view.document.querySelector('.single-problem-import');
    await act(async()=>[...root.querySelectorAll('button')].find(button=>button.textContent==='Codeforces 题目链接').click());
    const form=root.querySelector('form');const input=form.querySelector('input[type="url"]');
    const dateInput=form.querySelector('input[type="date"]');
    await act(async()=>{dateInput.value='2026-09-30';dateInput.dispatchEvent(new view.window.Event('input',{bubbles:true}));});
    const setUrl=async(value)=>act(async()=>{input.value=value;input.dispatchEvent(new view.window.Event('input',{bubbles:true}));});
    const submit=async()=>{await act(async()=>form.dispatchEvent(new view.window.Event('submit',{bubbles:true,cancelable:true})));await settle();};
    await setUrl('https://example.com/problem/1');await submit();assert.equal(calls.length,0);
    await setUrl('https://codeforces.com/problemset/problem/1900/A');await submit();assert.equal(calls.length,1);
    assert.equal(calls[0].mode,'codeforces');assert.match(root.textContent,/自动获取题面失败/);
    assert.equal(calls[0].completionDate,'2026-09-30');
    assert.equal(dateInput.value,'2026-09-30');
    assert.equal(input.value,'https://codeforces.com/problemset/problem/1900/A');
    await submit();assert.equal(calls.length,2);assert.equal(calls[1].requestId,calls[0].requestId);
  }finally{await view.cleanup();}
});
