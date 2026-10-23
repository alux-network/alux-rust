use crate::{Account, Chain, RESERVE};
use alux_stake::{
    DelegationAlg, EmissionAlg, HoldingsAlg, RewardPayoutAlg, RewardShareAlg, SnapshotLagAlg, StakeDelegationExt,
    WeightSourceAlg,
};
use alux_token::{BalancesAlg, TokenLedgerExt};

/// Selects delegated holdings, weights lagged two epochs, reserve rewards scaled by performance, and
/// no forfeiting.
#[derive(Debug, Default)]
pub struct Cardano;

/// Supplies delegated balances as weight.
impl WeightSourceAlg for Chain<Cardano> {
    type Validator = Account;
    type Amount = u128;

    fn source_weights(&self) -> Vec<(Account, u128)> {
        self.delegated_weights()
    }
}

/// Uses the snapshot taken two epochs before.
impl SnapshotLagAlg for Chain<Cardano> {
    type Epoch = u64;

    fn snapshot_governing(&self, epoch: &u64) -> u64 {
        epoch.saturating_sub(2)
    }
}

/// Emits three tenths of a percent of the reserve each epoch.
impl EmissionAlg for Chain<Cardano> {
    type Amount = u128;
    type Epoch = u64;

    fn emission(&self, _: &u64) -> u128 {
        self.balance(&RESERVE) * 3 / 1_000
    }
}

/// Shares the pot by weight, scaled by blocks produced out of those expected.
impl RewardShareAlg for Chain<Cardano> {
    type Validator = Account;
    type Amount = u128;
    type Performance = (u64, u64);

    fn reward_share(
        &self,
        _: &Account,
        pot: &u128,
        weight: &u128,
        total: &u128,
        (produced, expected): &(u64, u64),
    ) -> u128 {
        if *expected == 0 {
            return 0;
        }
        pot * weight / total * u128::from(*produced.min(expected)) / u128::from(*expected)
    }
}

/// Pays rewards from the reserve; what nobody earned stays there.
impl RewardPayoutAlg for Chain<Cardano> {
    type Validator = Account;
    type Amount = u128;

    fn reward_pay(&mut self, validator: &Account, amount: u128) {
        self.token_transfer(&RESERVE, validator, amount).expect("emission never exceeds the reserve");
    }

    fn reward_unpaid(&mut self, _: u128) {}
}

impl DelegationAlg for Chain<Cardano> {
    type Holder = Account;
    type Validator = Account;

    fn delegation(&self, holder: &Account) -> Option<Account> {
        self.delegations.get(holder).copied()
    }

    fn delegation_set(&mut self, holder: &Account, validator: Option<Account>) {
        match validator {
            Some(validator) => self.delegations.insert(*holder, validator),
            None => self.delegations.remove(holder),
        };
    }

    fn delegation_holders(&self) -> Vec<Account> {
        self.delegations.keys().copied().collect()
    }
}

/// Counts a holder's token balance as its holdings.
impl HoldingsAlg for Chain<Cardano> {
    type Holder = Account;
    type Amount = u128;

    fn holding(&self, holder: &Account) -> u128 {
        self.balance(holder)
    }
}
