use acm_os_application::*;
use std::cell::RefCell;
use std::future::Future;
use std::task::{Context, Waker};

fn run_ready<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        std::task::Poll::Ready(value) => value,
        std::task::Poll::Pending => panic!("test future unexpectedly yielded"),
    }
}

struct FakeRewardPort {
    account: Result<RewardAccountRecord, RewardError>,
    rewards: Result<Vec<CustomRewardRecord>, RewardError>,
    history: Result<Vec<RedemptionHistoryRecord>, RewardError>,
    active: bool,
    calls: RefCell<Vec<String>>,
}

impl Default for FakeRewardPort {
    fn default() -> Self {
        Self {
            account: Ok(RewardAccountRecord {
                xp_balance: 0,
                coin_balance: 0,
            }),
            rewards: Ok(Vec::new()),
            history: Ok(Vec::new()),
            active: false,
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl RewardPort for FakeRewardPort {
    async fn is_reward_active(&self) -> Result<bool, RewardError> {
        Ok(self.active)
    }
    async fn activate_reward(&self) -> Result<(), RewardError> {
        self.calls.borrow_mut().push("activate".into());
        Ok(())
    }
    async fn load_reward_account(&self) -> Result<RewardAccountRecord, RewardError> {
        self.calls.borrow_mut().push("account".into());
        self.account.clone()
    }
    async fn list_custom_rewards(&self) -> Result<Vec<CustomRewardRecord>, RewardError> {
        self.calls.borrow_mut().push("rewards".into());
        self.rewards.clone()
    }
    async fn list_redemption_history(&self) -> Result<Vec<RedemptionHistoryRecord>, RewardError> {
        self.calls.borrow_mut().push("history".into());
        self.history.clone()
    }
    async fn create_custom_reward(
        &self,
        name: &str,
        cost: i64,
    ) -> Result<CustomRewardRecord, RewardError> {
        self.calls
            .borrow_mut()
            .push(format!("create:{name}:{cost}"));
        Err(RewardError::DatabaseFailure)
    }
    async fn update_custom_reward(
        &self,
        id: &str,
        name: &str,
        cost: i64,
    ) -> Result<CustomRewardRecord, RewardError> {
        self.calls
            .borrow_mut()
            .push(format!("update:{id}:{name}:{cost}"));
        Err(RewardError::CustomRewardArchived)
    }
    async fn archive_custom_reward(&self, id: &str) -> Result<CustomRewardRecord, RewardError> {
        self.calls.borrow_mut().push(format!("archive:{id}"));
        Err(RewardError::CustomRewardNotFound)
    }
    async fn redeem_custom_reward(
        &self,
        id: &str,
        reward: &str,
    ) -> Result<RedemptionResult, RewardError> {
        self.calls
            .borrow_mut()
            .push(format!("redeem:{id}:{reward}"));
        Ok(RedemptionResult {
            disposition: RedemptionDisposition::AlreadyProcessed,
            redemption_id: id.into(),
            custom_reward_id: reward.into(),
            coin_cost_paid: 7,
            redeemed_at_utc: "t".into(),
        })
    }
    async fn refund_custom_reward(
        &self,
        refund: &str,
        redemption: &str,
    ) -> Result<RefundResult, RewardError> {
        self.calls
            .borrow_mut()
            .push(format!("refund:{refund}:{redemption}"));
        Ok(RefundResult {
            disposition: RefundDisposition::AlreadyRefunded,
            refund_id: refund.into(),
            redemption_id: redemption.into(),
            refunded_at_utc: "t".into(),
        })
    }
}

#[test]
fn account_composes_domain_level_and_copies_coin() {
    let port = FakeRewardPort {
        account: Ok(RewardAccountRecord {
            xp_balance: 100,
            coin_balance: 37,
        }),
        ..Default::default()
    };
    let summary = run_ready(load_reward_account(&port)).expect("summary");
    assert_eq!(
        summary,
        RewardAccountSummary {
            xp_balance: 100,
            coin_balance: 37,
            level: 2
        }
    );
}

#[test]
fn negative_xp_is_fail_closed() {
    let port = FakeRewardPort {
        account: Ok(RewardAccountRecord {
            xp_balance: -1,
            coin_balance: 1,
        }),
        ..Default::default()
    };
    assert_eq!(
        run_ready(load_reward_account(&port)),
        Err(RewardError::IntegrityViolation)
    );
}

#[test]
fn list_and_history_are_single_port_calls_and_drop_infrastructure_fields() {
    let port = FakeRewardPort {
        rewards: Ok(vec![CustomRewardRecord {
            custom_reward_id: "r".into(),
            name: "Reward".into(),
            coin_cost: 9,
            status: CustomRewardStatus::Archived,
        }]),
        history: Ok(vec![RedemptionHistoryRecord {
            redemption_id: "d".into(),
            custom_reward_id: "r".into(),
            reward_name: "Current".into(),
            coin_cost_paid: 4,
            redeemed_at_utc: "redeemed".into(),
            refund_id: Some("f".into()),
            refunded_at_utc: Some("refunded".into()),
        }]),
        ..Default::default()
    };
    let rewards = run_ready(list_custom_rewards(&port)).expect("rewards");
    assert_eq!(rewards[0].name, "Reward");
    let history = run_ready(list_redemption_history(&port)).expect("history");
    assert_eq!(history[0].coin_cost_paid, 4);
    assert_eq!(history[0].reward_name, "Current");
    assert_eq!(port.calls.borrow().as_slice(), ["rewards", "history"]);
}

#[test]
fn activation_and_mutations_preserve_one_way_boundary_and_intent_ids() {
    let port = FakeRewardPort::default();
    assert!(!run_ready(is_reward_active(&port)).expect("inactive"));
    run_ready(activate_reward(&port)).expect("activate");
    let redemption =
        run_ready(redeem_custom_reward(&port, "redemption", "reward")).expect("redeem");
    assert_eq!(
        redemption.disposition,
        RedemptionDisposition::AlreadyProcessed
    );
    let refund = run_ready(refund_custom_reward(&port, "refund", "redemption")).expect("refund");
    assert_eq!(refund.disposition, RefundDisposition::AlreadyRefunded);
    assert_eq!(
        port.calls.borrow().as_slice(),
        [
            "activate",
            "redeem:redemption:reward",
            "refund:refund:redemption"
        ]
    );
}

#[test]
fn error_codes_are_stable_and_mutation_inputs_are_forwarded() {
    let port = FakeRewardPort::default();
    assert_eq!(
        RewardError::RedemptionIntentConflict.code(),
        "redemption_intent_conflict"
    );
    assert_eq!(
        run_ready(create_custom_reward(&port, "name", 12)),
        Err(RewardError::DatabaseFailure)
    );
    assert_eq!(
        run_ready(update_custom_reward(&port, "id", "new", 13)),
        Err(RewardError::CustomRewardArchived)
    );
    assert_eq!(
        run_ready(archive_custom_reward(&port, "id")),
        Err(RewardError::CustomRewardNotFound)
    );
    assert_eq!(
        port.calls.borrow().as_slice(),
        ["create:name:12", "update:id:new:13", "archive:id"]
    );
}
