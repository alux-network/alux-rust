use crate::{Account, Chain, EthereumOffense};
use alux_stake::{
    BondsAlg, CustodyAlg, DelegationAlg, EmissionAlg, EpochAlg, ExclusionAlg, ForfeitAlg, HoldingsAlg, PerformanceAlg,
    QuorumAlg, RewardPayoutAlg, RewardShareAlg, SnapshotAlg, SnapshotLagAlg, UnbondingAlg, UnbondingDelayAlg,
    WeightSourceAlg,
};
use alux_token::{BalancesAlg, Shortfall, SupplyAlg};
use trait_set::trait_set;

pub(crate) const ALICE: Account = 1;
pub(crate) const BOB: Account = 2;
pub(crate) const VALIDATOR: Account = 10;
pub(crate) const OTHER_VALIDATOR: Account = 11;

/// Lets a scenario move time and report what consensus observed, which the specifications leave to
/// the chain.
pub(crate) trait ScenarioAlg {
    /// Takes the closing epoch's snapshot and moves to the next epoch.
    fn scenario_epoch_close(&mut self);

    /// Records blocks or duties a validator performed in an epoch out of those assigned.
    fn scenario_observe(&mut self, validator: Account, epoch: u64, performed: u64, assigned: u64);
}

impl<Variant> ScenarioAlg for Chain<Variant>
where
    Self: WeightSourceAlg<Validator = Account, Amount = u128> + SnapshotLagAlg<Epoch = u64>,
{
    fn scenario_epoch_close(&mut self) {
        self.epoch_close();
    }

    fn scenario_observe(&mut self, validator: Account, epoch: u64, performed: u64, assigned: u64) {
        self.blocks_observe(validator, epoch, performed, assigned);
    }
}

trait_set! {
    /// Combines a token ledger with epoch weights and rewards, as every chain has them.
    pub(crate) trait StakedChainAlg = ScenarioAlg
        + BalancesAlg<Holder = Account, Amount = u128>
        + SupplyAlg<Amount = u128>
        + EpochAlg<Epoch = u64>
        + WeightSourceAlg<Validator = Account, Amount = u128>
        + SnapshotAlg<Validator = Account, Amount = u128, Epoch = u64>
        + SnapshotLagAlg<Epoch = u64>
        + ExclusionAlg<Validator = Account, Epoch = u64>
        + PerformanceAlg<Validator = Account, Epoch = u64, Performance = (u64, u64)>
        + EmissionAlg<Amount = u128, Epoch = u64>
        + RewardShareAlg<Validator = Account, Amount = u128, Performance = (u64, u64)>
        + RewardPayoutAlg<Validator = Account, Amount = u128>;

    /// Adds escrowed stake held by the token ledger.
    pub(crate) trait EscrowChainAlg = StakedChainAlg
        + BondsAlg<Delegator = Account, Validator = Account, Amount = u128>
        + UnbondingAlg<Delegator = Account, Validator = Account, Amount = u128, Epoch = u64>
        + UnbondingDelayAlg<Epoch = u64>
        + CustodyAlg<Delegator = Account, Amount = u128, Denial = Shortfall<u128>>;

    /// Adds quorum finality and forfeits priced in basis points.
    pub(crate) trait CosmosChainAlg = EscrowChainAlg + QuorumAlg<Amount = u128> + ForfeitAlg<Offense = u128, Amount = u128>;

    /// Adds quorum finality and forfeits for Ethereum's offenses.
    pub(crate) trait EthereumChainAlg =
        EscrowChainAlg + QuorumAlg<Amount = u128> + ForfeitAlg<Offense = EthereumOffense, Amount = u128>;

    /// Adds delegated holdings.
    pub(crate) trait CardanoChainAlg = StakedChainAlg
        + DelegationAlg<Holder = Account, Validator = Account>
        + HoldingsAlg<Holder = Account, Amount = u128>;
}
