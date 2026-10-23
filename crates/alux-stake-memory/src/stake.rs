use crate::{Policy, Source};
use alux_stake::{
    BondsAlg, CustodyAlg, DelegationAlg, EmissionAlg, EpochAlg, ExclusionAlg, ForfeitAlg, HoldingsAlg, PerformanceAlg,
    QuorumAlg, RewardPayoutAlg, RewardShareAlg, SnapshotAlg, SnapshotLagAlg, StakeDelegationExt, StakeEscrowExt,
    Unbonding, UnbondingAlg, UnbondingDelayAlg, WeightSourceAlg,
};
use std::collections::BTreeMap;

/// Keeps stake in memory over accounts that are both holders and validators.
#[derive(Clone, Debug)]
pub struct MemoryStake<Account> {
    policy: Policy,
    epoch: u64,
    wallets: BTreeMap<Account, u64>,
    vault: u64,
    reserve: u64,
    bonds: BTreeMap<(Account, Account), u64>,
    unbonding: BTreeMap<Account, Vec<Unbonding<Account, u64, u64>>>,
    delegations: BTreeMap<Account, Account>,
    snapshots: BTreeMap<u64, Vec<(Account, u64)>>,
    exclusions: BTreeMap<Account, u64>,
    blocks: BTreeMap<(Account, u64), (u64, u64)>,
}

impl<Account> MemoryStake<Account>
where
    Account: Clone + Ord,
{
    /// Creates empty stake under the policy, at epoch zero.
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            epoch: 0,
            wallets: BTreeMap::new(),
            vault: 0,
            reserve: 0,
            bonds: BTreeMap::new(),
            unbonding: BTreeMap::new(),
            delegations: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            exclusions: BTreeMap::new(),
            blocks: BTreeMap::new(),
        }
    }

    /// Adds an amount to the account's wallet.
    pub fn wallet_fund(&mut self, account: &Account, amount: u64) {
        *self.wallets.entry(account.clone()).or_default() += amount;
    }

    /// Returns the account's wallet, including rewards paid to it.
    pub fn wallet(&self, account: &Account) -> u64 {
        self.wallets.get(account).copied().unwrap_or_default()
    }

    /// Returns what custody holds for escrowed stake.
    pub fn vault(&self) -> u64 {
        self.vault
    }

    /// Returns what was emitted and earned by nobody.
    pub fn reserve(&self) -> u64 {
        self.reserve
    }

    /// Moves to the next epoch.
    pub fn epoch_advance(&mut self) {
        self.epoch += 1;
    }

    /// Records blocks a validator produced in an epoch out of those it was expected to.
    pub fn blocks_observe(&mut self, validator: Account, epoch: u64, produced: u64, expected: u64) {
        self.blocks.insert((validator, epoch), (produced, expected));
    }
}

impl<Account> EpochAlg for MemoryStake<Account> {
    type Epoch = u64;

    fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl<Account> WeightSourceAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Validator = Account;
    type Amount = u64;

    fn source_weights(&self) -> Vec<(Account, u64)> {
        match self.policy.source {
            Source::Escrow => self.escrow_weights(),
            Source::Delegation => self.delegated_weights(),
        }
    }
}

impl<Account> SnapshotAlg for MemoryStake<Account>
where
    Account: Clone,
{
    type Validator = Account;
    type Amount = u64;
    type Epoch = u64;

    fn snapshot(&self, epoch: &u64) -> Vec<(Account, u64)> {
        self.snapshots.get(epoch).cloned().unwrap_or_default()
    }

    fn snapshot_record(&mut self, epoch: u64, weights: Vec<(Account, u64)>) {
        self.snapshots.insert(epoch, weights);
    }
}

impl<Account> SnapshotLagAlg for MemoryStake<Account> {
    type Epoch = u64;

    fn snapshot_governing(&self, epoch: &u64) -> u64 {
        epoch.saturating_sub(self.policy.lag)
    }
}

impl<Account> ExclusionAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Validator = Account;
    type Epoch = u64;

    fn excluded_until(&self, validator: &Account) -> Option<u64> {
        self.exclusions.get(validator).copied()
    }

    fn exclusion_set(&mut self, validator: &Account, until: Option<u64>) {
        match until {
            Some(until) => self.exclusions.insert(validator.clone(), until),
            None => self.exclusions.remove(validator),
        };
    }
}

/// Decides with a weight above the policy's fraction of the total.
impl<Account> QuorumAlg for MemoryStake<Account> {
    type Amount = u64;

    fn quorum_reached(&self, weight: &u64, total: &u64) -> bool {
        let (numerator, denominator) = self.policy.quorum;
        u128::from(*weight) * u128::from(denominator) > u128::from(*total) * u128::from(numerator)
    }
}

impl<Account> PerformanceAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Validator = Account;
    type Epoch = u64;
    type Performance = (u64, u64);

    fn performance(&self, validator: &Account, epoch: &u64) -> (u64, u64) {
        self.blocks.get(&(validator.clone(), *epoch)).copied().unwrap_or((0, 0))
    }
}

impl<Account> EmissionAlg for MemoryStake<Account> {
    type Amount = u64;
    type Epoch = u64;

    fn emission(&self, _: &u64) -> u64 {
        self.policy.emission
    }
}

/// Shares the pot by weight, scaled by blocks produced out of those expected.
impl<Account> RewardShareAlg for MemoryStake<Account> {
    type Validator = Account;
    type Amount = u64;
    type Performance = (u64, u64);

    fn reward_share(&self, _: &Account, pot: &u64, weight: &u64, total: &u64, performance: &(u64, u64)) -> u64 {
        let (produced, expected) = *performance;
        if *total == 0 || expected == 0 {
            return 0;
        }
        let share = u128::from(*pot) * u128::from(*weight) / u128::from(*total);
        u64::try_from(share * u128::from(produced.min(expected)) / u128::from(expected)).unwrap_or(u64::MAX)
    }
}

/// Credits rewards to the validator's wallet and keeps what nobody earned in the reserve.
impl<Account> RewardPayoutAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Validator = Account;
    type Amount = u64;

    fn reward_pay(&mut self, validator: &Account, amount: u64) {
        self.wallet_fund(validator, amount);
    }

    fn reward_unpaid(&mut self, amount: u64) {
        self.reserve += amount;
    }
}

impl<Account> BondsAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Delegator = Account;
    type Validator = Account;
    type Amount = u64;

    fn bond(&self, delegator: &Account, validator: &Account) -> u64 {
        self.bonds.get(&(delegator.clone(), validator.clone())).copied().unwrap_or_default()
    }

    fn bond_set(&mut self, delegator: &Account, validator: &Account, amount: u64) {
        let key = (delegator.clone(), validator.clone());
        if amount == 0 {
            self.bonds.remove(&key);
        } else {
            self.bonds.insert(key, amount);
        }
    }

    fn bond_delegators(&self, validator: &Account) -> Vec<Account> {
        self.bonds.keys().filter(|(_, kept)| kept == validator).map(|(delegator, _)| delegator.clone()).collect()
    }

    fn bond_validators(&self) -> Vec<Account> {
        let mut validators: Vec<_> = self.bonds.keys().map(|(_, validator)| validator.clone()).collect();
        validators.sort_unstable();
        validators.dedup();
        validators
    }
}

impl<Account> UnbondingAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Delegator = Account;
    type Validator = Account;
    type Amount = u64;
    type Epoch = u64;

    fn unbonding(&self, delegator: &Account) -> Vec<Unbonding<Account, u64, u64>> {
        self.unbonding.get(delegator).cloned().unwrap_or_default()
    }

    fn unbonding_set(&mut self, delegator: &Account, pending: Vec<Unbonding<Account, u64, u64>>) {
        if pending.is_empty() {
            self.unbonding.remove(delegator);
        } else {
            self.unbonding.insert(delegator.clone(), pending);
        }
    }

    fn unbonding_delegators(&self) -> Vec<Account> {
        self.unbonding.keys().cloned().collect()
    }
}

impl<Account> UnbondingDelayAlg for MemoryStake<Account> {
    type Epoch = u64;

    fn unbonding_matures(&self, start: &u64) -> u64 {
        start + self.policy.unbonding_delay
    }
}

/// Holds escrowed stake in a vault filled from wallets.
impl<Account> CustodyAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Delegator = Account;
    type Amount = u64;
    type Denial = ();

    fn custody_deposit(&mut self, from: &Account, amount: u64) -> Result<(), ()> {
        let remaining = self.wallet(from).checked_sub(amount).ok_or(())?;
        self.wallets.insert(from.clone(), remaining);
        self.vault += amount;
        Ok(())
    }

    fn custody_withdraw(&mut self, to: &Account, amount: u64) {
        self.vault -= amount;
        self.wallet_fund(to, amount);
    }

    fn custody_forfeit(&mut self, amount: u64) {
        self.vault -= amount;
    }
}

/// Forfeits the offense's share of a stake, in basis points.
impl<Account> ForfeitAlg for MemoryStake<Account> {
    type Offense = u64;
    type Amount = u64;

    fn forfeit_penalty(&self, basis_points: &u64, stake: &u64) -> u64 {
        u64::try_from(u128::from(*stake) * u128::from((*basis_points).min(10_000)) / 10_000).unwrap_or(*stake)
    }
}

impl<Account> DelegationAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Holder = Account;
    type Validator = Account;

    fn delegation(&self, holder: &Account) -> Option<Account> {
        self.delegations.get(holder).cloned()
    }

    fn delegation_set(&mut self, holder: &Account, validator: Option<Account>) {
        match validator {
            Some(validator) => self.delegations.insert(holder.clone(), validator),
            None => self.delegations.remove(holder),
        };
    }

    fn delegation_holders(&self) -> Vec<Account> {
        self.delegations.keys().cloned().collect()
    }
}

/// Counts a holder's wallet as its holdings.
impl<Account> HoldingsAlg for MemoryStake<Account>
where
    Account: Clone + Ord,
{
    type Holder = Account;
    type Amount = u64;

    fn holding(&self, holder: &Account) -> u64 {
        self.wallet(holder)
    }
}
