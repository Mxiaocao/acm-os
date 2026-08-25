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

const configuredWorkspace = {
  state: "configured",
  activeVaultPath: "C:/Vault",
  problemRootPath: "C:/Vault/Problems",
  knowledgeRootPath: "C:/Vault/Knowledge",
};

test.after(async () => vite.close());

test("Reward navigation and inactive account require explicit confirmation", { concurrency: false }, async () => {
  const calls = [];
  const view = await renderApp((command) => {
    calls.push(command);
    if (command === "reward_activation_state") return { active: false };
    return baseIpc(command);
  });
  try {
    const rewardLink = view.document.querySelector('nav[aria-label="Primary"] a[href="/reward"]');
    assert.equal(rewardLink?.textContent, "Reward");
    assert.equal(rewardLink?.getAttribute("aria-current"), "page");
    assert.match(view.document.body.textContent, /Reward Mode is currently off/);
    assert.match(view.document.body.textContent, /cannot be turned off or reset/);
    assert.match(view.document.body.textContent, /historical activity.*does not receive positive rewards/i);

    const trigger = findButton(view.document, "Enable Reward Mode");
    await act(async () => trigger.click());
    const dialog = view.document.querySelector('[role="alertdialog"]');
    assert.ok(dialog);
    assert.equal(calls.includes("activate_reward"), false);
    assert.equal(view.document.activeElement?.textContent, "Enable Reward Mode");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true })));
    assert.equal(view.document.activeElement?.textContent, "Cancel");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement?.textContent, "Enable Reward Mode");

    await act(async () => findButton(view.document, "Cancel").click());
    await settle();
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
    assert.equal(calls.includes("activate_reward"), false);
    assert.equal(view.document.activeElement, trigger);

    await act(async () => trigger.click());
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    await settle();
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
    assert.equal(view.document.activeElement, trigger);
  } finally {
    await view.cleanup();
  }
});

test("Reward activation runs once while pending and refreshes the active account", { concurrency: false }, async () => {
  let active = false;
  let resolveActivation;
  const activation = new Promise((resolve) => { resolveActivation = resolve; });
  const calls = [];
  const view = await renderApp((command) => {
    calls.push(command);
    if (command === "reward_activation_state") return { active };
    if (command === "activate_reward") return activation.then(() => { active = true; });
    if (command === "reward_account_summary") return { level: 3, xp: 125, coin: 40 };
    return baseIpc(command);
  });
  try {
    await act(async () => findButton(view.document, "Enable Reward Mode").click());
    const confirm = findButton(view.document, "Enable Reward Mode", 1);
    await act(async () => {
      confirm.click();
      confirm.click();
      await Promise.resolve();
    });
    assert.equal(calls.filter((command) => command === "activate_reward").length, 1);
    assert.equal(confirm.disabled, true);
    assert.equal(confirm.textContent, "Enabling...");

    await act(async () => resolveActivation());
    await settle();
    assert.equal(calls.filter((command) => command === "reward_activation_state").length, 2);
    assert.equal(calls.filter((command) => command === "reward_account_summary").length, 1);
    const account = view.document.querySelector('[aria-labelledby="reward-account-heading"]');
    assert.ok(account);
    assert.deepEqual(
      [...account.querySelectorAll("dt")].map((node) => node.textContent),
      ["Level", "XP", "Coin"],
    );
    assert.deepEqual(
      [...account.querySelectorAll("dd")].map((node) => node.textContent),
      ["3", "125", "40"],
    );
    assert.doesNotMatch(account.textContent, /next|remaining|%/i);
  } finally {
    await view.cleanup();
  }
});

test("Reward shows initial loading and a retryable activation-state error", { concurrency: false }, async () => {
  let resolveRead;
  let attempts = 0;
  const firstRead = new Promise((resolve) => { resolveRead = resolve; });
  const view = await renderApp((command) => {
    if (command === "reward_activation_state") {
      attempts += 1;
      if (attempts === 1) return firstRead.then(() => Promise.reject(new Error("offline")));
      return { active: false };
    }
    return baseIpc(command);
  }, false);
  try {
    assert.match(view.document.body.textContent, /Loading Reward Mode/);
    await act(async () => resolveRead());
    await settle();
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /could not be loaded/i);
    await act(async () => findButton(view.document, "Retry").click());
    await settle();
    assert.match(view.document.body.textContent, /Reward Mode is currently off/);
  } finally {
    await view.cleanup();
  }
});

test("Reward account errors remain visible without fake account values", { concurrency: false }, async () => {
  const view = await renderApp((command) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") throw new Error("offline");
    return baseIpc(command);
  });
  try {
    const alert = view.document.querySelector('[role="alert"]');
    assert.match(alert?.textContent ?? "", /account summary could not be loaded/i);
    assert.equal(view.document.querySelector('[aria-labelledby="reward-account-heading"] dl'), null);
    assert.doesNotMatch(view.document.body.textContent, /Level\s*0|XP\s*0|Coin\s*0/);
  } finally {
    await view.cleanup();
  }
});

test("Reward activation failure is announced and remains retryable", { concurrency: false }, async () => {
  let activationCalls = 0;
  const view = await renderApp((command) => {
    if (command === "reward_activation_state") return { active: false };
    if (command === "activate_reward") {
      activationCalls += 1;
      throw new Error("failed");
    }
    return baseIpc(command);
  });
  try {
    await act(async () => findButton(view.document, "Enable Reward Mode").click());
    await act(async () => findButton(view.document, "Enable Reward Mode", 1).click());
    await settle();
    assert.equal(activationCalls, 1);
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /was not enabled/i);
    assert.ok(findButton(view.document, "Enable Reward Mode"));
  } finally {
    await view.cleanup();
  }
});

function baseIpc(command) {
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
  throw new Error(`unexpected command ${command}`);
}

function findButton(document, label, index = 0) {
  const matches = [...document.querySelectorAll("button")].filter((button) => button.textContent === label);
  assert.ok(matches[index], `missing button: ${label} at index ${index}`);
  return matches[index];
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

async function renderApp(ipc, settleInitial = true) {
  const dom = new JSDOM('<!doctype html><html><body><div id="root"></div></body></html>', {
    url: "https://acm-os.test/reward",
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
  mockIPC((command, args) => ipc(command, args));
  const { App } = await vite.ssrLoadModule("/src/app/App.tsx");
  const root = createRoot(dom.window.document.getElementById("root"));
  await act(async () => root.render(React.createElement(App)));
  if (settleInitial) await settle();
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
