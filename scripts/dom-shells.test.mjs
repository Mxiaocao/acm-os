import assert from "node:assert/strict";
import test from "node:test";

import { JSDOM } from "jsdom";
import React, { act } from "react";
import { createRoot } from "react-dom/client";
import { createServer } from "vite";

globalThis.IS_REACT_ACT_ENVIRONMENT = true;

const vite = await createServer({
  configFile: false,
  root: process.cwd(),
  server: { middlewareMode: true },
});
const shells = await vite.ssrLoadModule("/src/app/shells.tsx");

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
    const textarea = view.document.querySelector('textarea[aria-label="Contest AI analysis raw text"]');
    await act(async () => { Object.getOwnPropertyDescriptor(view.window.HTMLTextAreaElement.prototype, "value").set.call(textarea, "unstructured raw"); textarea.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await settle();
    const buttons = [...view.document.querySelectorAll("button")];
    const preview = buttons.find((button) => button.textContent === "Parse preview");
    let save = buttons.find((button) => button.textContent === "Save analysis");
    assert.equal(save.disabled, true);
    await act(async () => preview.click()); await settle();
    assert.deepEqual(calls, ["preview_contest_ai_analysis"]);
    assert.match(view.document.body.textContent, /Preview: FAILED/);
    save = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Save analysis");
    assert.equal(save.disabled, false);
    await act(async () => save.click()); await settle();
    assert.deepEqual(calls, ["preview_contest_ai_analysis", "save_contest_ai_analysis"]);
    assert.match(view.document.body.textContent, /Saved raw analysis \(FAILED\)/);
  } finally { await view.cleanup(); }
});

test("Manual Contest submits explicit identities and statement text through one command", { concurrency: false }, async () => {
  const calls = [];
  const view = await renderApp((command, args) => {
    if (command === "foundation_status") return { status: "ready", core: "acm-os" };
    if (command === "app_shell_status") return { state: "normal", recoveryReason: null, supportedSchemaVersion: null, foundSchemaVersion: null, workspace: configuredWorkspace };
    if (command === "contest_shelf") return [];
    if (command === "import_manual_codeforces_contest") { calls.push(args.input); return { importStatus: "complete", missingSnapshotProblems: [], failedSnapshotProblems: [] }; }
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    await settle();
    const details = view.document.querySelector("details"); details.open = true;
    const inputs = [...details.querySelectorAll("input")];
    const values = ["1979", "Manual Round", "2026-08-13", "A", "Manual A", "https://codeforces.com/contest/1979/problem/A"];
    await act(async () => { inputs.forEach((input, index) => { Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, values[index]); input.dispatchEvent(new view.window.Event("input", { bubbles: true })); }); const textarea = details.querySelector("textarea"); Object.getOwnPropertyDescriptor(view.window.HTMLTextAreaElement.prototype, "value").set.call(textarea, "x < y"); textarea.dispatchEvent(new view.window.Event("input", { bubbles: true })); });
    await act(async () => details.querySelector("form").dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }))); await settle();
    assert.equal(calls.length, 1);
    assert.equal(calls[0].contestId, 1979);
    assert.equal(calls[0].problems[0].index, "A");
    assert.equal(calls[0].problems[0].statementText, "x < y");
    assert.match(view.document.body.textContent, /canonical import and statement snapshot contract/);
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
    const archive = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Archive Contest");
    await act(async () => archive.click()); await settle();
    assert.deepEqual(calls, ["set_contest_archived"]);
    assert.ok([...view.document.querySelectorAll("button")].some((button) => button.textContent === "Restore Contest"));
    const preview = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Preview delete");
    assert.equal(calls.includes("delete_contest"), false);
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /Preserve 1 global Problems/);
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Delete Contest");
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

async function renderApp(ipc, pathname = "/", strict = false) {
  const dom = new JSDOM("<!doctype html><html><body><div id=\"root\"></div></body></html>", {
    url: `https://acm-os.test${pathname}`,
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
  const { clearMocks, mockIPC } = await import("@tauri-apps/api/mocks");
  mockIPC((command, args) => {
    if (command === "plugin:event|listen") return 1;
    if (command === "plugin:event|unlisten") return null;
    return ipc(command, args);
  });
  const { App } = await vite.ssrLoadModule("/src/app/App.tsx");
  const root = createRoot(dom.window.document.getElementById("root"));
  const app = React.createElement(App);
  await act(async () => root.render(strict ? React.createElement(React.StrictMode, null, app) : app));
  await settle();
  return {
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

test.after(async () => vite.close());

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
      ["Today", "Contests", "我的题库", "Knowledge", "Settings"],
    );
    assert.equal(view.document.querySelector(".skip-link")?.getAttribute("href"), "#main-content");
    assert.equal(view.document.activeElement?.textContent, "Today");
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
    assert.equal(view.document.querySelector("h1")?.textContent, "Normal startup is blocked");
    assert.equal(view.document.title, "Recovery · ACM-OS");
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
    assert.equal(view.document.querySelector("h1")?.textContent, "Today");
    assert.equal(view.document.activeElement?.textContent, "Today");
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
    const contests = [...view.document.querySelectorAll("a")].find((link) => link.textContent === "Contests");
    await act(async () => contests.dispatchEvent(new view.window.MouseEvent("click", { bubbles: true, button: 0 })));
    assert.equal(view.window.location.pathname, "/contests");
    assert.equal(view.document.querySelector("h1")?.textContent, "比赛");
    view.window.history.pushState(null, "", "/review/018f0d8e-4a5b-7c6d-8e9f-0123456789ab");
    await act(async () => view.window.dispatchEvent(new view.window.PopStateEvent("popstate")));
    await settle();
    assert.equal(view.document.querySelector("nav"), null);
    assert.equal(view.document.querySelector("h1")?.textContent, "Isolated review workspace");
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
    if (command === "contest_shelf") return [];
    if (command === "import_codeforces_contest") throw new Error("unsupported_contest_url");
    throw new Error(`unexpected command ${command}`);
  }, "/contests");
  try {
    const input = view.document.querySelector("input");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(input, "https://codeforces.com");
      input.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await act(async () => {
      view.document.querySelector("form").dispatchEvent(new view.window.Event("submit", { bubbles: true, cancelable: true }));
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
      'form[aria-label="Contest facts snapshot"] button.secondary-action',
    );
    assert.ok(correct);
    await act(async () => correct.dispatchEvent(new view.window.MouseEvent("click", { bubbles: true })));
    await settle();
    assert.match(view.document.body.textContent, /纠错已保存/);
    assert.match(view.document.body.textContent, /Correction history/);
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
      .find((button) => button.textContent === "Create my note");
    assert.ok(createButton);
    await act(async () => createButton.click());
    await settle();
    assert.equal(createCalls, 1);
    assert.equal(detailReads, 2);
    assert.match(view.document.body.textContent, /Personal Problem/);
    assert.match(view.document.body.textContent, /Archive\/renamed\.md/);
    assert.match(view.document.body.textContent, /binding was restored/);
    assert.match(view.document.body.textContent, /External edit ×/);
    assert.equal(
      [...view.document.querySelectorAll("button")]
        .some((button) => button.textContent === "Create my note"),
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
    assert.match(view.document.body.textContent, /Note location needs attention/);
    const find = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Find possible locations");
    await act(async () => find.click()); await settle();
    const occupied = [...view.document.querySelectorAll("li")]
      .find((item) => item.textContent.includes("CF-1979-B.md"));
    assert.ok(occupied.querySelector("button").disabled);
    const candidate = [...view.document.querySelectorAll("li")]
      .find((item) => item.textContent.includes("Recovered/manual.md"));
    await act(async () => candidate.querySelector("button").click()); await settle();
    assert.doesNotMatch(view.document.body.textContent, /Note location needs attention/);
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
      .find((button) => button.textContent === "Confirm file was deleted…");
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /does not delete any file/);
    assert.equal(calls.some(([command]) => command === "confirm_personal_note_deleted"), false);
    const confirm = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Confirm deleted");
    await act(async () => confirm.click()); await settle();
    assert.match(view.document.body.textContent, /Lightweight Problem/);
    assert.doesNotMatch(view.document.body.textContent, /Note location needs attention/);
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
    assert.match(view.document.body.textContent, /Personal Markdown:.*Problems\/CF-1979-A\.md/);
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
      .find((button) => button.textContent === "Delete my personal note…");
    await act(async () => beginDelete.click());
    assert.match(view.document.body.textContent, /Contest history, completed Review history/);
    const confirmDelete = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Delete personal note");
    await act(async () => confirmDelete.click());
    await settle();
    assert.equal(deleteCalls, 1);
    assert.match(view.document.body.textContent, /Lightweight Problem/);
    assert.match(view.document.body.textContent, /historical facts were preserved/);
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
          attemptId, contestId: 1979, index: "A", attemptType: "firstColdStart",
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
          attemptId, contestId: 1979, index: "A", attemptType: "firstColdStart",
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
      .find((button) => button.textContent === "Start Review");
    assert.ok(start);
    await act(async () => start.click());
    await settle();
    assert.equal(startCalls, 1);
    assert.equal(focusCalls, 1);
    assert.equal(view.window.location.pathname, `/review/${attemptId}`);
    assert.equal(view.document.querySelector("nav"), null);
    assert.match(view.document.body.textContent, /SAFE STATEMENT/);
    assert.match(view.document.body.textContent, /Open original OJ/);
    assert.doesNotMatch(view.document.body.textContent, /SECRET SOLUTION/);
    assert.doesNotMatch(view.document.body.textContent, /Obsidian/);
    const originalOj = [...view.document.querySelectorAll("a")]
      .find((link) => link.textContent === "Open original OJ");
    await act(async () => originalOj.click());
    await settle();
    assert.deepEqual(openOjCalls, ["https://codeforces.com/contest/1979/problem/A"]);
    const openHelp = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Open controlled help");
    await act(async () => openHelp.click());
    await settle();
    assert.equal(drawerCalls, 1);
    assert.match(view.document.body.textContent, /Opening this drawer records nothing/);
    assert.equal(view.document.activeElement?.textContent, "Controlled help");
    assert.doesNotMatch(view.document.body.textContent, /REVEALED ONLY AFTER EVIDENCE/);
    assert.equal(revealCalls.length, 0);
    const hintRow = [...view.document.querySelectorAll(".review-help-levels li")]
      .find((row) => row.textContent.includes("Level 2"));
    await act(async () => hintRow.querySelector("button").click());
    assert.match(view.document.body.textContent, /Partial at best/);
    assert.equal(view.document.activeElement?.textContent, "Confirm and reveal");
    assert.equal(revealCalls.length, 0, "confirmation precedes reveal IPC");
    const confirm = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Confirm and reveal");
    await act(async () => confirm.click());
    await settle();
    assert.deepEqual(revealCalls, [{ attemptId, level: 2, impactAcknowledged: true }]);
    assert.match(view.document.body.textContent, /REVEALED ONLY AFTER EVIDENCE/);
    const solutionRow = [...view.document.querySelectorAll(".review-help-levels li")]
      .find((row) => row.textContent.includes("Level 5"));
    await act(async () => solutionRow.querySelector("button").click());
    assert.match(view.document.body.textContent, /can only be judged Not passed/);
    assert.equal(revealCalls.length, 1, "Level 5 needs its own confirmation");
    const cancel = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Cancel");
    await act(async () => cancel.click());
    assert.doesNotMatch(view.document.body.textContent, /FULL SOLUTION AFTER EVIDENCE/);
    assert.equal(view.document.activeElement?.textContent, "Controlled help");
    const closeHelp = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Close");
    await act(async () => closeHelp.click());
    assert.equal(view.document.activeElement, openHelp);
    const voidTrigger = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Void mistaken Attempt");
    await act(async () => voidTrigger.click());
    const voidDialog = view.document.querySelector('[aria-labelledby="void-review-title"]');
    const voidReason = voidDialog.querySelector("input");
    assert.equal(view.document.activeElement, voidReason);
    const voidCancel = [...voidDialog.querySelectorAll("button")]
      .find((button) => button.textContent === "Cancel");
    voidCancel.focus();
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement, voidReason, "Tab stays inside the modal");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    assert.equal(view.document.querySelector('[aria-labelledby="void-review-title"]'), null);
    assert.equal(view.document.activeElement, voidTrigger);
    const complete = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Complete from facts");
    await act(async () => complete.click());
    await settle();
    assert.equal(completeCalls.length, 1);
    assert.match(view.document.body.textContent, /Select at least one failure reason/);
    const reason = [...view.document.querySelectorAll("label")]
      .find((label) => label.textContent.includes("Direction found, key property blocked"));
    await act(async () => reason.querySelector("input").click());
    await act(async () => complete.click());
    await settle();
    assert.equal(completeCalls.length, 2);
    assert.deepEqual(completeCalls[1].failureReasons, [{ code: "keyPropertyBlocked", otherText: null }]);
    assert.match(view.document.body.textContent, /Completed Review/);
    assert.match(view.document.body.textContent, /Partial/);
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
      .find((button) => button.textContent === "Load Review history");
    await act(async () => load.click());
    await settle();
    assert.match(view.document.body.textContent, /Historical best Review evidence:\s*Mastered/);
    assert.match(view.document.body.textContent, /Partial/);
    assert.match(view.document.body.textContent, /Direction found, key property blocked/);
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
    assert.match(view.document.body.textContent, /Personal Problem/);
    assert.match(view.document.body.textContent, /Vault is unavailable/);
    assert.match(view.document.body.textContent, /System Facts were preserved/);
    assert.equal(
      [...view.document.querySelectorAll("button")]
        .some((button) => button.textContent === "Create my note"),
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
      .find((button) => button.textContent === "Load Review history");
    await act(async () => load.click());
    await settle();
    const checks = [...view.document.querySelectorAll(".mastery-evidence input[type='checkbox']")];
    assert.equal(checks.length, 6);
    for (const check of checks) await act(async () => check.click());
    const save = [...view.document.querySelectorAll("button")]
      .find((button) => button.textContent === "Save current evidence");
    await act(async () => save.click());
    await settle();
    assert.equal(updates.length, 1);
    assert.ok(Object.values(updates[0]).every(Boolean));
    assert.match(view.document.body.textContent, /Historical highest:\s*Thoroughly digested · first reached 2026-08-12/);
    await act(async () => checks[5].click());
    await act(async () => save.click());
    await settle();
    assert.equal(updates.length, 2);
    assert.equal(updates[1].transferSolvedIndependently, false);
    assert.match(view.document.body.textContent, /Current:\s*5\/6 evidence criteria/);
    assert.match(view.document.body.textContent, /Historical highest:\s*Thoroughly digested/);
    assert.match(view.document.body.textContent, /回炉中|鍥炵倝涓?/);
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
    assert.match(view.document.body.textContent, /learning state were not changed/);
    assert.match(view.document.body.textContent, /Retry/);
    assert.match(view.document.body.textContent, /Copy path/);
    assert.match(view.document.body.textContent, /Check settings/);
    assert.match(view.document.body.textContent, /Personal Problem/);
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
    contestId: 1979, problemIndex: problemId === "1" ? "A" : problemId === "2" ? "B" : "C",
    problemTitle: `Problem ${problemId}`,
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
      suggestions: [{ problemId: "3", contestId: 1979, problemIndex: "C", problemTitle: "Problem 3", reviewAttemptId: null, lane: "study", reason: "upsolve", planningCostMinutes: 60 }],
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
    assert.match(view.document.body.textContent, /Upsolve/);
    const down = view.document.querySelector('button[aria-label="Move Upsolve down"]');
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

    const done = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Done for today");
    await act(async () => done.click()); await settle();
    assert.ok(calls.some(([name]) => name === "complete_today_entry"));
    assert.match(view.document.body.textContent, /Extra suggestions/);
    const add = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Add to Today");
    await act(async () => add.click()); await settle();
    assert.ok(calls.some(([name, args]) => name === "accept_today_extra_suggestion" && args.input.problemId === "3"));
    assert.match(view.document.body.textContent, /Manual/);

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
    const preview = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Preview replan");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(budget, "-1");
      budget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      budget.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await settle();
    await act(async () => preview.click());
    await settle();
    assert.match(view.document.body.textContent, /Daily budget must be a non-negative whole number/);
    assert.equal(calls.filter(([name]) => name === "preview_today_replan").length, 0);
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(budget, "95");
      budget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      budget.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /Apply this replan/);
    const apply = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Apply replan");
    assert.equal(view.document.activeElement, apply, "replan dialog focuses its primary action");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    assert.equal(view.document.querySelector('[role="dialog"]'), null, "Escape closes the replan dialog");
    assert.equal(view.document.activeElement, preview, "closing the replan returns focus to its trigger");
    await act(async () => preview.click()); await settle();
    const reopenedApply = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Apply replan");
    await act(async () => reopenedApply.click()); await settle();
    assert.ok(calls.some(([name]) => name === "apply_today_replan"));
    assert.ok([...view.document.querySelectorAll(".sr-only")].some((node) => /Today replan applied/.test(node.textContent ?? "")));
  } finally {
    await view.cleanup();
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
    assert.match(view.document.body.textContent, /Set today's budget/);
    const create = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Create Today plan");
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
    assert.match(view.document.body.textContent, /Daily budget must be a non-negative whole number/);
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(initialBudget, "60");
      initialBudget.dispatchEvent(new view.window.Event("input", { bubbles: true }));
      initialBudget.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await act(async () => create.click()); await settle();
    assert.deepEqual(loads, [null, 60]);
    assert.match(view.document.body.textContent, /No tasks fit this budget/);
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
    const saveIntent = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Save intent only");
    assert.ok(saveIntent);
    await act(async () => saveIntent.click()); await settle();
    assert.equal(calls.at(-1)[0], "set_knowledge_candidate_disposition");
    assert.equal(calls.at(-1)[1].input.disposition, "acceptedIntent");
    assert.match(view.document.body.textContent, /Intent saved only/);
    const ignore = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Do not suggest");
    await act(async () => ignore.click()); await settle();
    assert.equal(calls.at(-1)[0], "set_knowledge_candidate_disposition");
    assert.equal(calls.at(-1)[1].input.disposition, "ignored");
    assert.equal(calls.some(([, args]) => args?.input?.disposition === "acceptedIntent"), true);
    assert.match(view.document.body.textContent, /Suggestion ignored/);
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
    const accept = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Accept existing Knowledge");
    assert.ok(accept);
    await act(async () => accept.click()); await settle();
    assert.equal(calls.length, 1);
    assert.equal(calls[0][1].input.knowledgeNodeId, candidate.knowledgeNodeId);
    assert.match(view.document.body.textContent, /written to current Markdown, re-read, and verified as a formal relation/);
    assert.match(view.document.body.textContent, /No suggestions for this Personal Problem/);
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
    assert.match(view.document.body.textContent, /Accepted intent · existing Knowledge Markdown now found/);
    assert.equal(calls.length, 0);
    const accept = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Accept existing Knowledge");
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
    const wednesday = view.document.querySelector('input[aria-label="Wednesday ACM budget in minutes"]');
    const thursday = view.document.querySelector('input[aria-label="Thursday ACM budget in minutes"]');
    assert.equal(wednesday.value, "95");
    assert.equal(thursday.value, "");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(wednesday, "73");
      wednesday.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await settle();
    assert.equal(wednesday.value, "73");
    const save = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Save weekly budget");
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(wednesday, "-1");
      wednesday.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await settle();
    await act(async () => save.click());
    await settle();
    assert.equal(calls.length, 0);
    assert.match(view.document.body.textContent, /Each weekly budget must be blank or a non-negative whole number/);
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLInputElement.prototype, "value").set.call(wednesday, "73");
      wednesday.dispatchEvent(new view.window.Event("input", { bubbles: true }));
    });
    await act(async () => save.click()); await settle();
    assert.equal(calls.length, 1);
    assert.equal(calls[0].wednesday, 73);
    assert.equal(calls[0].thursday, null);
    assert.match(view.document.body.textContent, /Existing Today plans and one-day overrides were not changed/);
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
    if (command === "knowledge_detail") { calls.push([command, args]); return { node, understanding, incoming: [], outgoing: [], relatedProblems: [{ problemId: "problem-1", contestId: 1, problemIndex: "A", title: "Theatre Square" }] }; }
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
    assert.match(view.document.body.textContent, /3 distinct related Problems gained new 真会 Review Evidence/);
    assert.match(view.document.body.textContent, /Your current status was not changed/);
    assert.match(view.document.body.textContent, /No user-confirmed status yet/);
    const select = view.document.querySelector('select[aria-label="Current understanding"]');
    await act(async () => {
      Object.getOwnPropertyDescriptor(view.window.HTMLSelectElement.prototype, "value").set.call(select, "basic");
      select.dispatchEvent(new view.window.Event("change", { bubbles: true }));
    });
    await settle();
    assert.equal(calls.filter(([name]) => name === "confirm_knowledge_understanding").length, 0);
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Confirm status");
    await act(async () => confirm.click()); await settle();
    assert.equal(calls.at(-1)[0], "confirm_knowledge_understanding");
    assert.equal(calls.at(-1)[1].input.level, "basic");
    assert.match(view.document.body.textContent, /Historical highest: 基本理解/);
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
    const find = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Find possible locations");
    await act(async () => find.click()); await settle();
    assert.equal(calls.at(-1)[0], "knowledge_relocation_candidates");
    assert.equal(calls.at(-1)[1].input.knowledgeNodeId, anomaly.knowledgeNodeId);
    const occupied = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Use this Markdown" && button.parentElement.textContent.includes("Occupied.md"));
    assert.equal(occupied.disabled, true);
    assert.match(view.document.body.textContent, /Already bound to another Primary Object/);
    const use = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Use this Markdown" && button.parentElement.textContent.includes("Archive/Segment Tree.md"));
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
    const preview = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Confirm file was deleted…");
    await act(async () => preview.click()); await settle();
    assert.match(view.document.body.textContent, /This does not delete any file/);
    assert.equal(calls.some(([name]) => name === "confirm_knowledge_markdown_deleted"), false);
    const confirm = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Confirm deleted");
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
    assert.match(view.document.body.textContent, /Identity was not chosen automatically/);
    assert.match(view.document.body.textContent, /Restore old Knowledge Node/);
    assert.match(view.document.body.textContent, /Create new Knowledge Node/);
    const restore = [...view.document.querySelectorAll("button")].find((button) => button.textContent === "Restore old Knowledge Node");
    await act(async () => restore.click()); await settle();
    const call = calls.find(([name]) => name === "resolve_knowledge_identity_conflict");
    assert.equal(call[1].input.historicalKnowledgeNodeId, conflict.historicalKnowledgeNodeId);
    assert.equal(call[1].input.candidateVaultRelativePath, conflict.candidateVaultRelativePath);
    assert.equal(call[1].input.restoreOldIdentity, true);
    assert.doesNotMatch(view.document.body.textContent, /Identity was not chosen automatically/);
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
    const preview = [...view.document.querySelectorAll("button")].find((b) => b.textContent === "Preview manual backup");
    await act(async () => preview.click()); await settle();
    assert.equal(calls[0], "preview_manual_backup");
    assert.match(view.document.body.textContent, /Schema 23/);
    assert.equal(calls.includes("create_manual_backup"), false);
    const create = [...view.document.querySelectorAll("button")].find((b) => b.textContent === "Create backup");
    await act(async () => create.click()); await settle();
    const createIndex = calls.indexOf("create_manual_backup");
    const inventoryIndex = calls.lastIndexOf("backup_inventory");
    assert.ok(createIndex >= 0);
    assert.ok(inventoryIndex > createIndex);
    assert.match(view.document.body.textContent, /Backup created/);
    assert.match(view.document.body.textContent, /keep 7 daily and 4 weekly/);
    assert.match(view.document.body.textContent, /integrity verified/);
    assert.match(view.document.body.textContent, /protected/);
    assert.equal([...view.document.querySelectorAll("button")].some((button) => /delete|prune/i.test(button.textContent)), false);
  } finally { await view.cleanup(); }
});
