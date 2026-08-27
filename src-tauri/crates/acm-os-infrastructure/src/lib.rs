#![forbid(unsafe_code)]

pub mod codeforces;
mod file_binding;
mod knowledge_index;
mod markdown;
mod persistence;
mod safe_patch;

pub use persistence::{
    start_database, CustomReward, CustomRewardHistoryError, CustomRewardRedemption,
    CustomRewardRedemptionDisposition, CustomRewardRedemptionError, CustomRewardRedemptionHistory,
    CustomRewardRedemptionResult, CustomRewardRefund, CustomRewardRefundDisposition,
    CustomRewardRefundError, CustomRewardRefundResult, CustomRewardStatus, DatabaseRuntime,
    RestoreRollbackCleanupError, RewardProcessingError, SystemHealthSnapshot,
};

pub fn current_local_date() -> Result<acm_os_domain::LocalDate, ()> {
    let value = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    acm_os_domain::LocalDate::parse_iso(&value).map_err(|_| ())
}
