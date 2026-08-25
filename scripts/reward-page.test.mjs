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


test("active Reward page loads active custom rewards and hides archived by default", { concurrency: false }, async () => {
  const calls = [];
  const view = await renderApp((command) => {
    calls.push(command);
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 2, xp: 20, coin: 8 };
    if (command === "list_custom_rewards") return [
      { customRewardId: "r-active", name: "Coffee", coinCost: 5, status: "active" },
      { customRewardId: "r-archived", name: "Old prize", coinCost: 9, status: "archived" },
    ];
    return baseIpc(command);
  });
  try {
    assert.match(view.document.body.textContent, /Custom Rewards/);
    assert.match(view.document.body.textContent, /Coffee/);
    assert.doesNotMatch(view.document.body.textContent, /Old prize/);
    assert.equal(calls.includes("list_custom_rewards"), true);
  } finally { await view.cleanup(); }
});

test("Custom Reward create, edit, archived filter, and archive confirmation use exact authority inputs", { concurrency: false }, async () => {
  const calls = [];
  let rewards = [{ customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" }, { customRewardId: "r2", name: "Old prize", coinCost: 9, status: "archived" }];
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 1 };
    if (command === "list_custom_rewards") return rewards;
    if (command === "create_custom_reward") { calls.push([command, args]); rewards = [...rewards, { customRewardId: "r3", name: args.input.name, coinCost: args.input.coinCost, status: "active" }]; return rewards.at(-1); }
    if (command === "update_custom_reward") { calls.push([command, args]); rewards = rewards.map((r) => r.customRewardId === args.input.customRewardId ? { ...r, name: args.input.name, coinCost: args.input.coinCost } : r); return rewards.find((r) => r.customRewardId === args.input.customRewardId); }
    if (command === "archive_custom_reward") { calls.push([command, args]); rewards = rewards.map((r) => r.customRewardId === args.input.customRewardId ? { ...r, status: "archived" } : r); return rewards.find((r) => r.customRewardId === args.input.customRewardId); }
    return baseIpc(command);
  });
  try {
    const archivedToggle = view.document.querySelector('input[type="checkbox"]');
    assert.ok(archivedToggle);
    await act(async () => archivedToggle.click()); await settle();
    assert.match(view.document.body.textContent, /Old prize/);
    assert.equal([...view.document.querySelectorAll("button")].filter((b) => b.textContent === "Edit").length, 1);

    const nameInput = view.document.querySelector('input[aria-label="Custom reward name"]');
    const costInput = view.document.querySelector('input[aria-label="Custom reward coin cost"]');
    await act(async () => { setInput(nameInput, "Weekend walk"); setInput(costInput, "12"); });
    await act(async () => findButton(view.document, "Create reward").click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "create_custom_reward")[1].input, { name: "Weekend walk", coinCost: 12 });

    await act(async () => findButton(view.document, "Edit").click()); await settle();
    assert.equal(view.document.querySelector('input[aria-label="Edit custom reward name"]').value, "Coffee");
    await act(async () => setInput(view.document.querySelector('input[aria-label="Edit custom reward name"]'), "Tea"));
    await act(async () => setInput(view.document.querySelector('input[aria-label="Edit custom reward coin cost"]'), "7"));
    await act(async () => findButton(view.document, "Save changes").click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "update_custom_reward")[1].input, { customRewardId: "r1", name: "Tea", coinCost: 7 });

    await act(async () => findButton(view.document, "Archive").click()); await settle();
    assert.match(view.document.querySelector('[role="alertdialog"]')?.textContent ?? "", /Tea/);
    assert.equal(calls.some(([name]) => name === "archive_custom_reward"), false);
    await act(async () => findButton(view.document, "Cancel").click()); await settle();
    assert.equal(calls.some(([name]) => name === "archive_custom_reward"), false);
    await act(async () => findButton(view.document, "Archive").click());
    await act(async () => findButton(view.document, "Archive reward").click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "archive_custom_reward")[1].input, { customRewardId: "r1" });
  } finally { await view.cleanup(); }
});

test("Custom Reward validation rejects blank names and invalid coin costs before IPC", { concurrency: false }, async () => {
  const calls = [];
  const view = await renderApp((command, args) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 1 };
    if (command === "list_custom_rewards") return [];
    if (command === "create_custom_reward") { calls.push([command, args]); return null; }
    return baseIpc(command);
  });
  try {
    const name = view.document.querySelector('input[aria-label="Custom reward name"]');
    const cost = view.document.querySelector('input[aria-label="Custom reward coin cost"]');
    for (const value of ["", "0", "-1", "1.5", "9007199254740992"]) {
      await act(async () => { setInput(name, value === "" ? "" : "Valid"); setInput(cost, value); });
      await act(async () => findButton(view.document, "Create reward").click()); await settle();
      assert.equal(calls.length, 0);
      assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /positive safe whole-number/i);
    }
  } finally { await view.cleanup(); }
});

test("R9E redeem uses one UUIDv7 intent, confirmation, and refresh", { concurrency: false }, async () => {
  const calls = [];
  const reward = { customRewardId: "r1", name: "Coffee", coinCost: 25, status: "active" };
  const view = await renderApp((command, args) => {
    calls.push([command, args]);
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 2, xp: 10, coin: 40 };
    if (command === "list_custom_rewards") return [reward];
    if (command === "reward_redemption_history") return [];
    if (command === "redeem_custom_reward") { return { disposition: "processed", redemptionId: args.input.redemptionId, customRewardId: "r1", coinCostPaid: 25, redeemedAtUtc: "2026-08-25T01:00:00Z" }; }
    return baseIpc(command);
  });
  try {
    const redeem = findButton(view.document, "Redeem");
    await act(async () => redeem.click());
    assert.match(view.document.querySelector('[role="alertdialog"]')?.textContent ?? "", /Coffee/);
    assert.equal(calls.filter(([name]) => name === "redeem_custom_reward").length, 0);
    const confirm = findButton(view.document, "Redeem reward");
    await act(async () => confirm.click());
    const redeemCalls = calls.filter(([name]) => name === "redeem_custom_reward");
    assert.equal(redeemCalls.length, 1);
    assert.equal(redeemCalls[0][1].input.customRewardId, "r1");
    assert.match(redeemCalls[0][1].input.redemptionId, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    await new Promise((resolve) => setTimeout(resolve, 10));
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
    assert.ok(calls.filter(([name]) => name === "reward_account_summary").length >= 2);
  } finally { await view.cleanup(); }
});

test("R9E history uses historical paid cost and refund sends only IDs", { concurrency: false }, async () => {
  const calls = [];
  const reads = [];
  const historyItem = { redemptionId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab", customRewardId: "r1", rewardName: "Coffee", coinCostPaid: 12, redeemedAtUtc: "2026-08-25T00:00:00Z", refundId: null, refundedAtUtc: null };
  const view = await renderApp((command, args) => {
    if (command === "reward_account_summary" || command === "reward_redemption_history") reads.push(command);
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 3 };
    if (command === "list_custom_rewards") return [{ customRewardId: "r1", name: "Coffee", coinCost: 99, status: "archived" }];
    if (command === "reward_redemption_history") return [historyItem];
    if (command === "refund_custom_reward") { calls.push([command, args]); return { disposition: "alreadyRefunded", refundId: args.input.refundId, redemptionId: args.input.redemptionId, refundedAtUtc: "2026-08-25T02:00:00Z" }; }
    return baseIpc(command);
  });
  try {
    assert.match(view.document.body.textContent, /12 Coin paid/);
    assert.match(view.document.body.textContent, /Not refunded/);
    await act(async () => findButton(view.document, "Refund").click());
    assert.match(view.document.querySelector('[role="alertdialog"]')?.textContent ?? "", /12 Coin/);
    await act(async () => findButton(view.document, "Refund reward").click()); await settle();
    assert.equal(calls.length, 1);
    assert.equal(calls[0][1].input.redemptionId, historyItem.redemptionId);
    assert.match(calls[0][1].input.refundId, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    assert.equal(Object.keys(calls[0][1].input).sort().join(","), "redemptionId,refundId");
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
    assert.ok(reads.filter((command) => command === "reward_account_summary").length >= 2);
    assert.ok(reads.filter((command) => command === "reward_redemption_history").length >= 2);
  } finally { await view.cleanup(); }
});

test("R9E redeem retries reuse one intent, pending duplicates are ignored, and a new action gets a new UUIDv7", { concurrency: false }, async () => {
  const reward = { customRewardId: "r1", name: "Coffee", coinCost: 25, status: "active" };
  const calls = [];
  let resolveRetry;
  let attempt = 0;
  const retryPending = new Promise((resolve) => { resolveRetry = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 2, xp: 10, coin: 40 };
    if (command === "list_custom_rewards") return [reward];
    if (command === "reward_redemption_history") return [];
    if (command === "redeem_custom_reward") {
      calls.push(args.input);
      attempt += 1;
      if (attempt === 1) return Promise.reject("temporary_failure");
      if (attempt === 2) return retryPending;
      return { disposition: "alreadyProcessed", redemptionId: args.input.redemptionId, customRewardId: "r1", coinCostPaid: 25, redeemedAtUtc: "2026-08-25T01:00:00Z" };
    }
    return baseIpc(command);
  });
  try {
    await act(async () => findButton(view.document, "Redeem").click());
    await act(async () => findButton(view.document, "Redeem reward").click());
    await settle();
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /Retry the same intent/);
    const firstId = calls[0].redemptionId;
    assert.match(firstId, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);

    const retry = findButton(view.document, "Redeem reward");
    act(() => { retry.click(); retry.click(); });
    assert.equal(calls.length, 2);
    assert.equal(calls[1].redemptionId, firstId);
    await act(async () => resolveRetry({ disposition: "processed", redemptionId: firstId, customRewardId: "r1", coinCostPaid: 25, redeemedAtUtc: "2026-08-25T01:00:00Z" }));
    await settle();
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);

    await act(async () => findButton(view.document, "Redeem").click());
    await act(async () => findButton(view.document, "Redeem reward").click());
    await settle();
    assert.equal(calls.length, 3);
    assert.match(calls[2].redemptionId, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    assert.notEqual(calls[2].redemptionId, firstId);
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
  } finally { await view.cleanup(); }
});

test("R9E refund retries reuse one intent, pending duplicates are ignored, and a new action gets a new UUIDv7", { concurrency: false }, async () => {
  const first = { redemptionId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab", customRewardId: "r1", rewardName: "Coffee", coinCostPaid: 12, redeemedAtUtc: "2026-08-25T00:00:00Z", refundId: null, refundedAtUtc: null };
  const second = { redemptionId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ac", customRewardId: "r2", rewardName: "Tea", coinCostPaid: 8, redeemedAtUtc: "2026-08-25T00:30:00Z", refundId: null, refundedAtUtc: null };
  let history = [first, second];
  const calls = [];
  let resolveRetry;
  let attempt = 0;
  const retryPending = new Promise((resolve) => { resolveRetry = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 10 };
    if (command === "list_custom_rewards") return [];
    if (command === "reward_redemption_history") return history;
    if (command === "refund_custom_reward") {
      calls.push(args.input);
      attempt += 1;
      if (attempt === 1) return Promise.reject("temporary_failure");
      if (attempt === 2) return retryPending;
      history = [{ ...first, refundId: "settled-first", refundedAtUtc: "2026-08-25T02:00:00Z" }, { ...second, refundId: args.input.refundId, refundedAtUtc: "2026-08-25T03:00:00Z" }];
      return { disposition: "alreadyProcessed", refundId: args.input.refundId, redemptionId: args.input.redemptionId, refundedAtUtc: "2026-08-25T03:00:00Z" };
    }
    return baseIpc(command);
  });
  try {
    await act(async () => findButton(view.document, "Refund").click());
    await act(async () => findButton(view.document, "Refund reward").click());
    await settle();
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /Retry the same intent/);
    const firstId = calls[0].refundId;
    assert.match(firstId, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);

    const retry = findButton(view.document, "Refund reward");
    act(() => { retry.click(); retry.click(); });
    assert.equal(calls.length, 2);
    assert.equal(calls[1].refundId, firstId);
    history = [{ ...first, refundId: firstId, refundedAtUtc: "2026-08-25T02:00:00Z" }, second];
    await act(async () => resolveRetry({ disposition: "processed", refundId: firstId, redemptionId: first.redemptionId, refundedAtUtc: "2026-08-25T02:00:00Z" }));
    await settle();
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);

    await act(async () => findButton(view.document, "Refund").click());
    await act(async () => findButton(view.document, "Refund reward").click());
    await settle();
    assert.equal(calls.length, 3);
    assert.equal(calls[2].redemptionId, second.redemptionId);
    assert.match(calls[2].refundId, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    assert.notEqual(calls[2].refundId, firstId);
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
  } finally { await view.cleanup(); }
});

test("R9E transaction dialog Escape restores the originating action", { concurrency: false }, async () => {
  const reward = { customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" };
  const view = await renderApp((command) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 10 };
    if (command === "list_custom_rewards") return [reward];
    if (command === "reward_redemption_history") return [];
    return baseIpc(command);
  });
  try {
    const redeem = findButton(view.document, "Redeem");
    await act(async () => redeem.click());
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    await settle();
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
    assert.equal(view.document.activeElement, redeem);
  } finally { await view.cleanup(); }
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

function setInput(input, value) { input.value = value; input.dispatchEvent(new input.ownerDocument.defaultView.Event("input", { bubbles: true })); }

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
