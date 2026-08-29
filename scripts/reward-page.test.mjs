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
    const rewardLink = view.document.querySelector('nav[aria-label="主导航"] a[href="/reward"]');
    assert.equal(rewardLink?.textContent, "奖励");
    assert.equal(rewardLink?.getAttribute("aria-current"), "page");
    assert.match(view.document.body.textContent, /奖励模式当前未启用/);
    assert.match(view.document.body.textContent, /无法关闭或重置/);
    assert.match(view.document.body.textContent, /历史活动不会追溯获得正向奖励/);

    const trigger = findButton(view.document, "Enable Reward Mode");
    await act(async () => trigger.click());
    const dialog = view.document.querySelector('[role="alertdialog"]');
    assert.ok(dialog);
    assert.equal(calls.includes("activate_reward"), false);
    assert.equal(view.document.activeElement?.textContent, "启用奖励模式");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true })));
    assert.equal(view.document.activeElement?.textContent, "取消");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement?.textContent, "启用奖励模式");

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
    assert.equal(confirm.textContent, "正在启用…");

    await act(async () => resolveActivation());
    await settle();
    assert.equal(calls.filter((command) => command === "reward_activation_state").length, 2);
    assert.equal(calls.filter((command) => command === "reward_account_summary").length, 1);
    const account = view.document.querySelector('[aria-labelledby="reward-account-heading"]');
    assert.ok(account);
    assert.deepEqual(
      [...account.querySelectorAll("dt")].map((node) => node.textContent),
      ["等级", "经验值（XP）", "金币"],
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
    assert.match(view.document.body.textContent, /正在加载奖励模式/);
    await act(async () => resolveRead());
    await settle();
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /奖励模式无法加载/);
    await act(async () => findButton(view.document, "Retry").click());
    await settle();
    assert.match(view.document.body.textContent, /奖励模式当前未启用/);
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
    assert.match(alert?.textContent ?? "", /奖励账户摘要无法加载/);
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
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /奖励模式未能启用/);
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
    assert.match(view.document.body.textContent, /自定义奖励/);
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
    assert.equal([...view.document.querySelectorAll("button")].filter((b) => b.textContent === "编辑").length, 1);

    const nameInput = view.document.querySelector('input[aria-label="自定义奖励名称"]');
    const costInput = view.document.querySelector('input[aria-label="自定义奖励所需金币"]');
    await act(async () => { setInput(nameInput, "Weekend walk"); setInput(costInput, "12"); });
    await act(async () => findButton(view.document, "Create reward").click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "create_custom_reward")[1].input, { name: "Weekend walk", coinCost: 12 });

    await act(async () => findButton(view.document, "Edit").click()); await settle();
    assert.equal(view.document.querySelector('input[aria-label="编辑自定义奖励名称"]').value, "Coffee");
    await act(async () => setInput(view.document.querySelector('input[aria-label="编辑自定义奖励名称"]'), "Tea"));
    await act(async () => setInput(view.document.querySelector('input[aria-label="编辑自定义奖励所需金币"]'), "7"));
    await act(async () => findButton(view.document, "Save changes").click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "update_custom_reward")[1].input, { customRewardId: "r1", name: "Tea", coinCost: 7 });
    assert.match(view.document.querySelector('[aria-labelledby="reward-actions-heading"]')?.textContent ?? "", /Tea/);

    await act(async () => findButton(view.document, "Archive").click()); await settle();
    assert.match(view.document.querySelector('[role="alertdialog"]')?.textContent ?? "", /Tea/);
    assert.equal(calls.some(([name]) => name === "archive_custom_reward"), false);
    await act(async () => findButton(view.document, "Cancel").click()); await settle();
    assert.equal(calls.some(([name]) => name === "archive_custom_reward"), false);
    await act(async () => findButton(view.document, "Archive").click());
    await act(async () => findButton(view.document, "Archive reward").click()); await settle();
    assert.deepEqual(calls.find(([name]) => name === "archive_custom_reward")[1].input, { customRewardId: "r1" });
    assert.doesNotMatch(view.document.querySelector('[aria-labelledby="reward-actions-heading"]')?.textContent ?? "", /Tea/);
  } finally { await view.cleanup(); }
});

test("Custom Reward create synchronizes the redemption area immediately", { concurrency: false }, async () => {
  let rewards = [{ customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" }];
  const view = await renderApp((command, args) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 20 };
    if (command === "list_custom_rewards") return rewards;
    if (command === "create_custom_reward") {
      const created = { customRewardId: "r2", name: args.input.name, coinCost: args.input.coinCost, status: "active" };
      rewards = [...rewards, created];
      return created;
    }
    if (command === "reward_redemption_history") return [];
    return baseIpc(command, args);
  });
  try {
    const management = view.document.querySelector('[aria-labelledby="custom-rewards-heading"]');
    const [name, cost] = management.querySelectorAll('input:not([type="checkbox"])');
    await act(async () => { setInput(name, "Tea"); setInput(cost, "7"); });
    await act(async () => findButton(view.document, "Create reward").click());
    await settle();
    const redemption = view.document.querySelector('[aria-labelledby="reward-actions-heading"]');
    assert.match(redemption?.textContent ?? "", /Tea/);
    assert.match(redemption?.textContent ?? "", /7 金币/);
  } finally { await view.cleanup(); }
});

test("Reward mutations never fake-add when create fails", { concurrency: false }, async () => {
  const reward = { customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" };
  const view = await renderApp((command) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 20 };
    if (command === "list_custom_rewards") return [reward];
    if (command === "reward_redemption_history") return [];
    if (command === "create_custom_reward") throw new Error("offline");
    return baseIpc(command);
  });
  try {
    const management = view.document.querySelector('[aria-labelledby="custom-rewards-heading"]');
    const [name, cost] = management.querySelectorAll('input:not([type="checkbox"])');
    await act(async () => { setInput(name, "Tea"); setInput(cost, "7"); });
    await act(async () => findButton(view.document, "Create reward").click()); await settle();
    assert.doesNotMatch(view.document.querySelector('[aria-labelledby="reward-actions-heading"]')?.textContent ?? "", /Tea/);
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /鏇存敼鏈繚瀛|奖励/);
  } finally { await view.cleanup(); }
});

test("Redeem refreshes balance, history count, and affordability", { concurrency: false }, async () => {
  const rewards = [
    { customRewardId: "r1", name: "Small", coinCost: 100, status: "active" },
    { customRewardId: "r2", name: "Large", coinCost: 250, status: "active" },
  ];
  let coin = 320;
  let history = [];
  const view = await renderApp((command, args) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin };
    if (command === "list_custom_rewards") return rewards;
    if (command === "reward_redemption_history") return history;
    if (command === "redeem_custom_reward") {
      coin -= 100;
      history = [{ redemptionId: args.input.redemptionId, customRewardId: "r1", rewardName: "Small", coinCostPaid: 100, redeemedAtUtc: "2026-08-25T00:00:00Z", refundId: null, refundedAtUtc: null }];
      return { disposition: "processed", redemptionId: args.input.redemptionId, customRewardId: "r1", coinCostPaid: 100, redeemedAtUtc: "2026-08-25T00:00:00Z" };
    }
    return baseIpc(command);
  });
  try {
    const buttons = [...view.document.querySelectorAll('[aria-labelledby="reward-actions-heading"] button')];
    assert.equal(buttons.length, 2);
    assert.equal(buttons[1].disabled, false);
    await act(async () => buttons[0].click());
    await act(async () => findButton(view.document, "Redeem reward").click()); await settle();
    assert.match(view.document.querySelector('[aria-labelledby="reward-account-heading"]')?.textContent ?? "", /220/);
    assert.equal(view.document.querySelector('[aria-labelledby="reward-actions-heading"] button[aria-label*="Large"]')?.disabled, true);
    assert.equal(view.document.querySelector('.reward-history .reward-section-summary > span')?.textContent, "1");
    assert.equal(view.document.querySelector('.reward-management')?.hasAttribute("open"), false);
    assert.equal(view.document.querySelector('.reward-history')?.hasAttribute("open"), false);
  } finally { await view.cleanup(); }
});

test("Redemption cards show six by default and support view-all collapse", { concurrency: false }, async () => {
  const rewards = Array.from({ length: 7 }, (_, index) => ({ customRewardId: `r${index}`, name: `Reward ${index}`, coinCost: 1, status: "active" }));
  const view = await renderApp((command) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 20 };
    if (command === "list_custom_rewards") return rewards;
    if (command === "reward_redemption_history") return [];
    return baseIpc(command);
  });
  try {
    assert.equal(view.document.querySelectorAll(".reward-card").length, 6);
    const expand = view.document.querySelector(".reward-expand");
    assert.ok(expand);
    await act(async () => expand.click());
    assert.equal(view.document.querySelectorAll(".reward-card").length, 7);
    await act(async () => expand.click());
    assert.equal(view.document.querySelectorAll(".reward-card").length, 6);
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
    const name = view.document.querySelector('input[aria-label="自定义奖励名称"]');
    const cost = view.document.querySelector('input[aria-label="自定义奖励所需金币"]');
    for (const value of ["", "0", "-1", "1.5", "9007199254740992"]) {
      await act(async () => { setInput(name, value === "" ? "" : "Valid"); setInput(cost, value); });
      await act(async () => findButton(view.document, "Create reward").click()); await settle();
      assert.equal(calls.length, 0);
      assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /请输入名称.*有效的正整数/);
      const invalidInput = value === "" ? name : cost;
      assert.equal(invalidInput.getAttribute("aria-invalid"), "true");
      assert.equal(invalidInput.getAttribute("aria-describedby"), "custom-reward-error");
    }

    await act(async () => { setInput(name, "Existing name"); setInput(cost, String(Number.MAX_SAFE_INTEGER)); });
    await act(async () => findButton(view.document, "Create reward").click()); await settle();
    assert.deepEqual(calls[0][1].input, { name: "Existing name", coinCost: Number.MAX_SAFE_INTEGER });
  } finally { await view.cleanup(); }
});

test("Custom Reward mutations reject rapid duplicate submits while pending", { concurrency: false }, async () => {
  let rewards = [{ customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" }];
  const calls = [];
  let resolveCreate;
  let resolveUpdate;
  let resolveArchive;
  const createPending = new Promise((resolve) => { resolveCreate = resolve; });
  const updatePending = new Promise((resolve) => { resolveUpdate = resolve; });
  const archivePending = new Promise((resolve) => { resolveArchive = resolve; });
  const view = await renderApp((command, args) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 20 };
    if (command === "list_custom_rewards") return rewards;
    if (command === "reward_redemption_history") return [];
    if (command === "create_custom_reward") { calls.push(command); return createPending; }
    if (command === "update_custom_reward") { calls.push(command); return updatePending; }
    if (command === "archive_custom_reward") { calls.push(command); return archivePending; }
    return baseIpc(command, args);
  });
  try {
    await act(async () => {
      setInput(view.document.querySelector('input[aria-label="自定义奖励名称"]'), "Coffee");
      setInput(view.document.querySelector('input[aria-label="自定义奖励所需金币"]'), "7");
    });
    const create = findButton(view.document, "Create reward");
    act(() => { create.click(); create.click(); });
    assert.equal(calls.filter((command) => command === "create_custom_reward").length, 1);
    rewards = [...rewards, { customRewardId: "r2", name: "Coffee", coinCost: 7, status: "active" }];
    await act(async () => resolveCreate(rewards[1])); await settle();

    await act(async () => findButton(view.document, "Edit").click());
    await act(async () => setInput(view.document.querySelector('input[aria-label="编辑自定义奖励名称"]'), "Tea"));
    const save = findButton(view.document, "Save changes");
    act(() => { save.click(); save.click(); });
    assert.equal(calls.filter((command) => command === "update_custom_reward").length, 1);
    rewards = rewards.map((reward) => reward.customRewardId === "r1" ? { ...reward, name: "Tea" } : reward);
    await act(async () => resolveUpdate(rewards[0])); await settle();

    await act(async () => findButton(view.document, "Archive").click());
    const archive = findButton(view.document, "Archive reward");
    act(() => { archive.click(); archive.click(); });
    assert.equal(calls.filter((command) => command === "archive_custom_reward").length, 1);
    rewards = rewards.map((reward) => reward.customRewardId === "r1" ? { ...reward, status: "archived" } : reward);
    await act(async () => resolveArchive(rewards[0])); await settle();
  } finally { await view.cleanup(); }
});

test("archive dialog traps focus and restores the originating action on cancel", { concurrency: false }, async () => {
  const reward = { customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" };
  const view = await renderApp((command) => {
    if (command === "reward_activation_state") return { active: true };
    if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 20 };
    if (command === "list_custom_rewards") return [reward];
    if (command === "reward_redemption_history") return [];
    return baseIpc(command);
  });
  try {
    const trigger = findButton(view.document, "Archive");
    await act(async () => trigger.click());
    assert.equal(view.document.activeElement?.textContent, "归档奖励");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true })));
    assert.equal(view.document.activeElement?.textContent, "取消");
    await act(async () => view.document.dispatchEvent(new view.window.KeyboardEvent("keydown", { key: "Tab", bubbles: true })));
    assert.equal(view.document.activeElement?.textContent, "归档奖励");
    await act(async () => findButton(view.document, "Cancel").click()); await settle();
    assert.equal(view.document.querySelector('[role="alertdialog"]'), null);
    assert.equal(view.document.activeElement, trigger);
  } finally { await view.cleanup(); }
});

test("Reward read failures preserve unrelated successful account, reward, and history data", { concurrency: false }, async () => {
  const reward = { customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" };
  const history = { redemptionId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab", customRewardId: "r1", rewardName: "Coffee history", coinCostPaid: 4, redeemedAtUtc: "2026-08-25T00:00:00Z", refundId: "refund-1", refundedAtUtc: "2026-08-25T01:00:00Z" };
  for (const failedRead of ["reward_account_summary", "list_custom_rewards", "reward_redemption_history"]) {
    const view = await renderApp((command) => {
      if (command === "reward_activation_state") return { active: true };
      if (command === failedRead) throw new Error("offline");
      if (command === "reward_account_summary") return { level: 3, xp: 30, coin: 20 };
      if (command === "list_custom_rewards") return [reward];
      if (command === "reward_redemption_history") return [history];
      return baseIpc(command);
    });
    try {
      if (failedRead !== "reward_account_summary") assert.match(view.document.body.textContent, /等级\s*3[\s\S]*经验值（XP）\s*30[\s\S]*金币\s*20/);
      if (failedRead !== "list_custom_rewards") assert.match(view.document.body.textContent, /Coffee/);
      if (failedRead !== "reward_redemption_history") assert.match(view.document.body.textContent, /Coffee history[\s\S]*已支付 4 金币/);
      assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /无法加载/);
    } finally { await view.cleanup(); }
  }
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
    assert.match(view.document.body.textContent, /已支付 12 金币/);
    assert.match(view.document.body.textContent, /未撤销/);
    await act(async () => findButton(view.document, "Refund").click());
    assert.match(view.document.querySelector('[role="alertdialog"]')?.textContent ?? "", /12 金币/);
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
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /兑换未能完成，请重试或取消/);
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
    assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", /撤销未能完成，请重试或取消/);
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

test("Reward transaction errors distinguish terminal stale state from retryable storage failures", { concurrency: false }, async () => {
  const reward = { customRewardId: "r1", name: "Coffee", coinCost: 5, status: "active" };
  const history = { redemptionId: "018f0d8e-4a5b-7c6d-8e9f-0123456789ab", customRewardId: "r1", rewardName: "Coffee", coinCostPaid: 5, redeemedAtUtc: "2026-08-25T00:00:00Z", refundId: null, refundedAtUtc: null };
  for (const [kind, code, message] of [
    ["redeem", "reward_inactive", /奖励模式未启用/],
    ["redeem", "custom_reward_not_found", /该自定义奖励服务已不存在/],
    ["redeem", "custom_reward_archived", /该奖励已归档，无法兑换/],
    ["redeem", "reward_integrity_violation", /奖励数据无法验证/],
    ["redeem", "reward_database_failure", /奖励存储不可用/],
    ["refund", "redemption_not_found", /该兑换记录不存在/],
    ["refund", "already_refunded", /该兑换已撤销/],
    ["refund", "refund_intent_conflict", /撤销意图与已有撤销冲突/],
  ]) {
    const view = await renderApp((command) => {
      if (command === "reward_activation_state") return { active: true };
      if (command === "reward_account_summary") return { level: 1, xp: 1, coin: 20 };
      if (command === "list_custom_rewards") return [reward];
      if (command === "reward_redemption_history") return [history];
      if (command === (kind === "redeem" ? "redeem_custom_reward" : "refund_custom_reward")) throw code;
      return baseIpc(command);
    });
    try {
      await act(async () => findButton(view.document, kind === "redeem" ? "Redeem" : "Refund").click());
      await act(async () => findButton(view.document, kind === "redeem" ? "Redeem reward" : "Refund reward").click()); await settle();
      assert.match(view.document.querySelector('[role="alert"]')?.textContent ?? "", message);
      assert.ok(findButton(view.document, "Cancel"));
    } finally { await view.cleanup(); }
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

function setInput(input, value) { input.value = value; input.dispatchEvent(new input.ownerDocument.defaultView.Event("input", { bubbles: true })); }

function findButton(document, label, index = 0) {
  const localized = {
    "Enable Reward Mode": "启用奖励模式",
    "Cancel": "取消",
    "Retry": "重试",
    "Edit": "编辑",
    "Create reward": "创建奖励",
    "Save changes": "保存更改",
    "Archive": "归档",
    "Archive reward": "归档奖励",
    "Redeem": "兑换",
    "Redeem reward": "兑换奖励",
    "Refund": "撤销兑换",
    "Refund reward": "确认撤销兑换",
  }[label] ?? label;
  const matches = [...document.querySelectorAll("button")].filter((button) => button.textContent === localized);
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
