import { invoke } from "@tauri-apps/api/core";
import { v7 as uuidv7 } from "uuid";

export interface RewardActivationStateDto {
  active: boolean;
}

export interface RewardAccountSummaryDto {
  xp: number;
  coin: number;
  level: number;
}

export type CustomRewardStatusDto = "active" | "archived";

export interface CustomRewardDto {
  customRewardId: string;
  name: string;
  coinCost: number;
  status: CustomRewardStatusDto;
}

export interface RedemptionHistoryItemDto {
  redemptionId: string;
  customRewardId: string;
  rewardName: string;
  coinCostPaid: number;
  redeemedAtUtc: string;
  refundId: string | null;
  refundedAtUtc: string | null;
}

export type RedemptionDispositionDto = "processed" | "alreadyProcessed";
export type RefundDispositionDto = "processed" | "alreadyProcessed" | "alreadyRefunded";

export interface RedemptionResultDto {
  disposition: RedemptionDispositionDto;
  redemptionId: string;
  customRewardId: string;
  coinCostPaid: number;
  redeemedAtUtc: string;
}

export interface RefundResultDto {
  disposition: RefundDispositionDto;
  refundId: string;
  redemptionId: string;
  refundedAtUtc: string;
}

export interface CreateCustomRewardInputDto {
  name: string;
  coinCost: number;
}

export interface UpdateCustomRewardInputDto {
  customRewardId: string;
  name: string;
  coinCost: number;
}

export interface RedeemCustomRewardInputDto {
  redemptionId: string;
  customRewardId: string;
}

export interface RefundCustomRewardInputDto {
  refundId: string;
  redemptionId: string;
}

export function getRewardActivationState(): Promise<RewardActivationStateDto> {
  return invoke<RewardActivationStateDto>("reward_activation_state");
}

export function getRewardAccountSummary(): Promise<RewardAccountSummaryDto> {
  return invoke<RewardAccountSummaryDto>("reward_account_summary");
}

export function listCustomRewards(): Promise<CustomRewardDto[]> {
  return invoke<CustomRewardDto[]>("list_custom_rewards");
}

export function getRewardRedemptionHistory(): Promise<RedemptionHistoryItemDto[]> {
  return invoke<RedemptionHistoryItemDto[]>("reward_redemption_history");
}

export function activateReward(): Promise<void> {
  return invoke<void>("activate_reward");
}

export function createCustomReward(
  input: CreateCustomRewardInputDto,
): Promise<CustomRewardDto> {
  return invoke<CustomRewardDto>("create_custom_reward", { input });
}

export function updateCustomReward(
  input: UpdateCustomRewardInputDto,
): Promise<CustomRewardDto> {
  return invoke<CustomRewardDto>("update_custom_reward", { input });
}

export function archiveCustomReward(customRewardId: string): Promise<CustomRewardDto> {
  return invoke<CustomRewardDto>("archive_custom_reward", { input: { customRewardId } });
}

export function redeemCustomReward(
  input: RedeemCustomRewardInputDto,
): Promise<RedemptionResultDto> {
  return invoke<RedemptionResultDto>("redeem_custom_reward", { input });
}

export function refundCustomReward(
  input: RefundCustomRewardInputDto,
): Promise<RefundResultDto> {
  return invoke<RefundResultDto>("refund_custom_reward", { input });
}

export function createRewardIntentId(): string {
  return uuidv7();
}
