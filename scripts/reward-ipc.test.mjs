import assert from "node:assert/strict";
import test from "node:test";
import { JSDOM } from "jsdom";

const dom = new JSDOM("<!doctype html><html><body></body></html>", {
  url: "https://acm-os.test/reward",
});
for (const [key, value] of Object.entries({
  window: dom.window,
  document: dom.window.document,
  navigator: dom.window.navigator,
})) {
  Object.defineProperty(globalThis, key, { configurable: true, value, writable: true });
}

const { clearMocks, mockIPC } = await import("@tauri-apps/api/mocks");
const reward = await import("../src/ipc/reward.ts");

const redemptionId = "018f0d8e-4a5b-7c6d-8e9f-0123456789ab";
const refundId = "018f0d8e-4a5b-7c6d-8e9f-0123456789ac";

test.after(() => {
  clearMocks();
  dom.window.close();
});

test("Reward IPC wrappers preserve command names, DTOs, and exact request inputs", async () => {
  const calls = [];
  const responses = {
    reward_activation_state: { active: true },
    reward_account_summary: { xp: 42, coin: 17, level: 3 },
    list_custom_rewards: [{ customRewardId: "reward-1", name: "Coffee", coinCost: 25, status: "active" }],
    reward_redemption_history: [{
      redemptionId,
      customRewardId: "reward-1",
      rewardName: "Coffee",
      coinCostPaid: 25,
      redeemedAtUtc: "2026-08-25T00:00:00.000Z",
      refundId: null,
      refundedAtUtc: null,
    }],
    activate_reward: undefined,
    create_custom_reward: { customRewardId: "reward-2", name: "Tea", coinCost: 12, status: "active" },
    update_custom_reward: { customRewardId: "reward-2", name: "Tea", coinCost: 13, status: "active" },
    archive_custom_reward: { customRewardId: "reward-2", name: "Tea", coinCost: 13, status: "archived" },
    redeem_custom_reward: {
      disposition: "processed",
      redemptionId,
      customRewardId: "reward-2",
      coinCostPaid: 13,
      redeemedAtUtc: "2026-08-25T01:00:00.000Z",
    },
    refund_custom_reward: {
      disposition: "processed",
      refundId,
      redemptionId,
      refundedAtUtc: "2026-08-25T02:00:00.000Z",
    },
  };
  mockIPC((command, args) => {
    calls.push({ command, args });
    return responses[command];
  });

  assert.deepEqual(await reward.getRewardActivationState(), responses.reward_activation_state);
  assert.deepEqual(await reward.getRewardAccountSummary(), responses.reward_account_summary);
  assert.deepEqual(await reward.listCustomRewards(), responses.list_custom_rewards);
  assert.deepEqual(await reward.getRewardRedemptionHistory(), responses.reward_redemption_history);
  await reward.activateReward();
  await reward.createCustomReward({ name: "Tea", coinCost: 12 });
  await reward.updateCustomReward({ customRewardId: "reward-2", name: "Tea", coinCost: 13 });
  await reward.archiveCustomReward("reward-2");
  await reward.redeemCustomReward({ redemptionId, customRewardId: "reward-2" });
  await reward.refundCustomReward({ refundId, redemptionId });

  assert.deepEqual(calls.map(({ command }) => command), [
    "reward_activation_state",
    "reward_account_summary",
    "list_custom_rewards",
    "reward_redemption_history",
    "activate_reward",
    "create_custom_reward",
    "update_custom_reward",
    "archive_custom_reward",
    "redeem_custom_reward",
    "refund_custom_reward",
  ]);
  assert.deepEqual(calls[5].args, { input: { name: "Tea", coinCost: 12 } });
  assert.deepEqual(calls[6].args, { input: { customRewardId: "reward-2", name: "Tea", coinCost: 13 } });
  assert.deepEqual(calls[7].args, { input: { customRewardId: "reward-2" } });
  assert.deepEqual(calls[8].args, { input: { redemptionId, customRewardId: "reward-2" } });
  assert.deepEqual(calls[9].args, { input: { refundId, redemptionId } });
});

test("Reward mutation dispositions remain distinct runtime values", async () => {
  mockIPC((command) => {
    if (command === "redeem_custom_reward") {
      return { disposition: "alreadyProcessed", redemptionId, customRewardId: "reward-1", coinCostPaid: 25, redeemedAtUtc: "2026-08-25T00:00:00.000Z" };
    }
    return { disposition: "alreadyRefunded", refundId, redemptionId, refundedAtUtc: "2026-08-25T02:00:00.000Z" };
  });
  const redemption = await reward.redeemCustomReward({ redemptionId, customRewardId: "reward-1" });
  const refund = await reward.refundCustomReward({ refundId, redemptionId });
  assert.equal(redemption.disposition, "alreadyProcessed");
  assert.equal(refund.disposition, "alreadyRefunded");
  assert.notEqual("processed", redemption.disposition);
  assert.notEqual("alreadyProcessed", refund.disposition);
});

test("Reward IPC wrappers preserve backend error codes", async () => {
  mockIPC((command) => {
    if (command === "redeem_custom_reward") return Promise.reject("insufficient_coin");
    return Promise.reject("unexpected_command");
  });
  await assert.rejects(
    reward.redeemCustomReward({ redemptionId, customRewardId: "reward-1" }),
    (error) => error === "insufficient_coin",
  );
});

test("createRewardIntentId returns canonical UUIDv7 values", () => {
  const ids = Array.from({ length: 8 }, () => reward.createRewardIntentId());
  for (const id of ids) {
    assert.match(id, /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
  }
  assert.equal(new Set(ids).size, ids.length);
});
