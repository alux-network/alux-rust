use crate::{Account, Chain, Escrowed};
use alux_stake::{
    EmissionAlg, ForfeitAlg, QuorumAlg, RewardPayoutAlg, RewardShareAlg, SnapshotLagAlg, StakeEscrowExt,
    UnbondingDelayAlg, WeightSourceAlg,
};
use alux_token::{SupplyAlg, TokenLedgerExt};

/// Selects escrow, live weights, inflation rewards, and forfeiting with jailing.
#[derive(Debug, Default)]
pub struct Cosmos;

impl Escrowed for Cosmos {}

/// Supplies the bonded stake as weight.
impl WeightSourceAlg for Chain<Cosmos> {
    type Validator = Account;
    type Amount = u128;

    fn source_weights(&self) -> Vec<(Account, u128)> {
        self.escrow_weights()
    }
}

/// Uses live weights.
impl SnapshotLagAlg for Chain<Cosmos> {
    type Epoch = u64;

    fn snapshot_governing(&self, epoch: &u64) -> u64 {
        *epoch
    }
}

/// Decides with more than two thirds.
impl QuorumAlg for Chain<Cosmos> {
    type Amount = u128;

    fn quorum_reached(&self, weight: &u128, total: &u128) -> bool {
        3 * weight > 2 * total
    }
}

/// Inflates the supply by one percent each epoch.
impl EmissionAlg for Chain<Cosmos> {
    type Amount = u128;
    type Epoch = u64;

    fn emission(&self, _: &u64) -> u128 {
        self.supply() / 100
    }
}

/// Shares the pot by weight alone; missed blocks cost a validator its place, not its reward.
impl RewardShareAlg for Chain<Cosmos> {
    type Validator = Account;
    type Amount = u128;
    type Performance = (u64, u64);

    fn reward_share(&self, _: &Account, pot: &u128, weight: &u128, total: &u128, _: &(u64, u64)) -> u128 {
        pot * weight / total
    }
}

/// Mints rewards; what nobody earned is never created.
impl RewardPayoutAlg for Chain<Cosmos> {
    type Validator = Account;
    type Amount = u128;

    fn reward_pay(&mut self, validator: &Account, amount: u128) {
        self.token_mint(validator, amount);
    }

    fn reward_unpaid(&mut self, _: u128) {}
}

/// Matures unbonding three epochs after it starts.
impl UnbondingDelayAlg for Chain<Cosmos> {
    type Epoch = u64;

    fn unbonding_matures(&self, start: &u64) -> u64 {
        start + 3
    }
}

/// Forfeits the offense's share of a stake, in basis points.
impl ForfeitAlg for Chain<Cosmos> {
    type Offense = u128;
    type Amount = u128;

    fn forfeit_penalty(&self, basis_points: &u128, stake: &u128) -> u128 {
        stake * basis_points.min(&10_000) / 10_000
    }
}
