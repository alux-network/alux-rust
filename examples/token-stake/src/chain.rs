use alux_stake::{
    EpochAlg, ExclusionAlg, PerformanceAlg, SnapshotAlg, SnapshotLagAlg, StakeWeightExt, Unbonding, WeightSourceAlg,
};
use alux_token::{BalancesAlg, SupplyAlg};
use std::{collections::BTreeMap, marker::PhantomData};

/// Identifies an account: a token holder, a delegator, or a validator.
pub type Account = u32;

/// Identifies the account that holds every escrowed token.
pub const STAKE_POOL: Account = 0;

/// Identifies the account that holds tokens not yet emitted as rewards.
pub const RESERVE: Account = u32::MAX;

/// Keeps one chain's token ledger and stake under a variant.
#[derive(Debug, Default)]
pub struct Chain<Variant> {
    balances: BTreeMap<Account, u128>,
    supply: u128,
    epoch: u64,
    snapshots: BTreeMap<u64, Vec<(Account, u128)>>,
    exclusions: BTreeMap<Account, u64>,
    pub(crate) bonds: BTreeMap<(Account, Account), u128>,
    pub(crate) unbonding: BTreeMap<Account, Vec<Unbonding<Account, u128, u64>>>,
    pub(crate) delegations: BTreeMap<Account, Account>,
    blocks: BTreeMap<(Account, u64), (u64, u64)>,
    variant: PhantomData<Variant>,
}

impl<Variant> Chain<Variant>
where
    Self: WeightSourceAlg<Validator = Account, Amount = u128> + SnapshotLagAlg<Epoch = u64>,
{
    /// Takes the closing epoch's snapshot and moves to the next epoch.
    pub fn epoch_close(&mut self) {
        self.stake_snapshot_take();
        self.epoch += 1;
    }
}

impl<Variant> Chain<Variant> {
    /// Records what consensus observed: blocks a validator produced out of those it was expected to.
    pub fn blocks_observe(&mut self, validator: Account, epoch: u64, produced: u64, expected: u64) {
        self.blocks.insert((validator, epoch), (produced, expected));
    }
}

impl<Variant> BalancesAlg for Chain<Variant> {
    type Holder = Account;
    type Amount = u128;

    fn balance(&self, holder: &Account) -> u128 {
        self.balances.get(holder).copied().unwrap_or_default()
    }

    fn balance_set(&mut self, holder: &Account, amount: u128) {
        self.balances.insert(*holder, amount);
        self.balances.retain(|_, kept| *kept != 0);
    }

    fn balance_holders(&self) -> Vec<Account> {
        self.balances.keys().copied().collect()
    }
}

impl<Variant> SupplyAlg for Chain<Variant> {
    type Amount = u128;

    fn supply(&self) -> u128 {
        self.supply
    }

    fn supply_set(&mut self, amount: u128) {
        self.supply = amount;
    }
}

impl<Variant> EpochAlg for Chain<Variant> {
    type Epoch = u64;

    fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl<Variant> SnapshotAlg for Chain<Variant> {
    type Validator = Account;
    type Amount = u128;
    type Epoch = u64;

    fn snapshot(&self, epoch: &u64) -> Vec<(Account, u128)> {
        self.snapshots.get(epoch).cloned().unwrap_or_default()
    }

    fn snapshot_record(&mut self, epoch: u64, weights: Vec<(Account, u128)>) {
        self.snapshots.insert(epoch, weights);
    }
}

impl<Variant> ExclusionAlg for Chain<Variant> {
    type Validator = Account;
    type Epoch = u64;

    fn excluded_until(&self, validator: &Account) -> Option<u64> {
        self.exclusions.get(validator).copied()
    }

    fn exclusion_set(&mut self, validator: &Account, until: Option<u64>) {
        match until {
            Some(until) => self.exclusions.insert(*validator, until),
            None => self.exclusions.remove(validator),
        };
    }
}

impl<Variant> PerformanceAlg for Chain<Variant> {
    type Validator = Account;
    type Epoch = u64;
    type Performance = (u64, u64);

    fn performance(&self, validator: &Account, epoch: &u64) -> (u64, u64) {
        self.blocks.get(&(*validator, *epoch)).copied().unwrap_or((0, 0))
    }
}
