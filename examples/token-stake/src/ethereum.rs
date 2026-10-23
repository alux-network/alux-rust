use crate::{Account, Chain, Escrowed};
use alux_stake::{
    EmissionAlg, ForfeitAlg, QuorumAlg, RewardPayoutAlg, RewardShareAlg, SnapshotLagAlg, StakeEscrowExt,
    StakeWeightExt, UnbondingDelayAlg, WeightSourceAlg,
};
use alux_token::TokenLedgerExt;

/// Caps how much of a validator's stake counts as weight.
pub const MAX_EFFECTIVE_BALANCE: u128 = 2_048;

/// Selects escrow with an effective balance cap, weights from the previous epoch, issuance that grows
/// with the square root of the stake, rewards per duty performed, and forfeiting for equivocation
/// and inactivity, with ejection.
///
/// TODO: rate-limit how much weight enters and leaves per epoch (the activation and exit churn
/// limit); the stake specification has no queue between the weight source and the snapshot.
///
/// TODO: let a forfeit grow with how much other stake was forfeited in the same window (the
/// correlation penalty); the stake specification has no observation of recent forfeits.
#[derive(Debug, Default)]
pub struct Ethereum;

impl Escrowed for Ethereum {}

/// Describes an offense consensus proved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EthereumOffense {
    /// Says the validator signed two conflicting messages.
    Equivocation,
    /// Says the validator missed its duties for this many epochs while the chain could not finalize.
    Inactivity {
        /// Counts the epochs of inactivity.
        epochs: u64,
    },
}

/// Supplies each validator's bonded stake as weight, capped at the maximum effective balance.
impl WeightSourceAlg for Chain<Ethereum> {
    type Validator = Account;
    type Amount = u128;

    fn source_weights(&self) -> Vec<(Account, u128)> {
        self.escrow_weights()
            .into_iter()
            .map(|(validator, bonded)| (validator, bonded.min(MAX_EFFECTIVE_BALANCE)))
            .collect()
    }
}

/// Uses the snapshot taken one epoch before.
impl SnapshotLagAlg for Chain<Ethereum> {
    type Epoch = u64;

    fn snapshot_governing(&self, epoch: &u64) -> u64 {
        epoch.saturating_sub(1)
    }
}

/// Decides with more than two thirds.
impl QuorumAlg for Chain<Ethereum> {
    type Amount = u128;

    fn quorum_reached(&self, weight: &u128, total: &u128) -> bool {
        3 * weight > 2 * total
    }
}

/// Issues four times the square root of the epoch's total weight.
impl EmissionAlg for Chain<Ethereum> {
    type Amount = u128;
    type Epoch = u64;

    fn emission(&self, epoch: &u64) -> u128 {
        4 * self.stake_total(epoch).isqrt()
    }
}

/// Shares the pot by weight, scaled by duties performed out of those assigned.
impl RewardShareAlg for Chain<Ethereum> {
    type Validator = Account;
    type Amount = u128;
    type Performance = (u64, u64);

    fn reward_share(
        &self,
        _: &Account,
        pot: &u128,
        weight: &u128,
        total: &u128,
        (performed, assigned): &(u64, u64),
    ) -> u128 {
        if *assigned == 0 {
            return 0;
        }
        pot * weight / total * u128::from(*performed.min(assigned)) / u128::from(*assigned)
    }
}

/// Mints rewards; what nobody earned is never created.
impl RewardPayoutAlg for Chain<Ethereum> {
    type Validator = Account;
    type Amount = u128;

    fn reward_pay(&mut self, validator: &Account, amount: u128) {
        self.token_mint(validator, amount);
    }

    fn reward_unpaid(&mut self, _: u128) {}
}

/// Matures unbonding four epochs after it starts.
impl UnbondingDelayAlg for Chain<Ethereum> {
    type Epoch = u64;

    fn unbonding_matures(&self, start: &u64) -> u64 {
        start + 4
    }
}

/// Forfeits a thirty-second of the stake for equivocation, and a share growing with the square of
/// the epochs of inactivity, up to the whole stake.
impl ForfeitAlg for Chain<Ethereum> {
    type Offense = EthereumOffense;
    type Amount = u128;

    fn forfeit_penalty(&self, offense: &EthereumOffense, stake: &u128) -> u128 {
        match offense {
            EthereumOffense::Equivocation => stake / 32,
            EthereumOffense::Inactivity { epochs } => {
                let epochs = u128::from(*epochs);
                (stake * epochs * epochs / 4_096).min(*stake)
            }
        }
    }
}
