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
        sanitizedHtml: "<div class=\"problem-statement\"><p>For $$$a^2+b^2=c^2$$$.</p><p>$$$\\frac{1}{2}$$$</p><pre>$$$keep_raw$$$</pre></div>",
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
