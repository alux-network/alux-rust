use crate::{EpochAlg, ExclusionAlg, RewardSettlement, SnapshotAlg, SnapshotLagAlg, StakeWeightExt, WeightSourceAlg};
use alux_ext::ext;
use std::ops::{Add, Sub};

/// Tells how well a validator did its duties in an epoch.
///
/// Consensus observes performance, not stake. Blocks produced against blocks expected, or duties
/// done against duties assigned, are both answers.
pub trait PerformanceAlg {
    /// Identifies a validator.
    type Validator;
    /// Orders time in epochs.
    type Epoch;
    /// Describes how well a validator did.
    type Performance;

    /// Returns how well the validator did in the epoch.
    fn performance(&self, validator: &Self::Validator, epoch: &Self::Epoch) -> Self::Performance;
}

/// Tells how much is available to reward an epoch.
///
/// Inflation, a fraction of a reserve, collected fees, or their sum are all answers.
pub trait EmissionAlg {
    /// Measures a quantity of stake.
    type Amount;
    /// Orders time in epochs.
    type Epoch;

    /// Returns the epoch's reward pot.
    fn emission(&self, epoch: &Self::Epoch) -> Self::Amount;
}

/// Decides how much of a pot a validator earned.
///
/// Withholding is this policy, not a separate penalty: a validator that performed poorly earns less,
/// and its principal is not touched. Saturation, pledge, and margin are also this policy's.
pub trait RewardShareAlg {
    /// Identifies a validator.
    type Validator;
    /// Measures a quantity of stake.
    type Amount;
    /// Describes how well a validator did.
    type Performance;

    /// Returns what the validator earned from the pot, given its weight out of the total.
    fn reward_share(
        &self,
        validator: &Self::Validator,
        pot: &Self::Amount,
        weight: &Self::Amount,
        total: &Self::Amount,
        performance: &Self::Performance,
    ) -> Self::Amount;
}

/// Pays rewards with value.
///
/// Proof of stake does not decide what rewards are made of. A chain mints its native token, or moves
/// it from a reserve; what is unpaid stays in the reserve, or is never created.
pub trait RewardPayoutAlg {
    /// Identifies a validator.
    type Validator;
    /// Measures a quantity of stake.
    type Amount;

    /// Pays the validator an amount.
    fn reward_pay(&mut self, validator: &Self::Validator, amount: Self::Amount);

    /// Settles what nobody earned.
    fn reward_unpaid(&mut self, amount: Self::Amount);
}

/// Derives settling an epoch's rewards from weights, performance, emission, share, and payout.
///
/// TODO: split a validator's reward among its delegators.
#[ext(name = StakeRewardsExt)]
pub impl<This, Validator, Amount, Epoch, Performance> This
where
    This: EpochAlg<Epoch = Epoch>
        + WeightSourceAlg<Validator = Validator, Amount = Amount>
        + SnapshotAlg<Validator = Validator, Amount = Amount, Epoch = Epoch>
        + SnapshotLagAlg<Epoch = Epoch>
        + ExclusionAlg<Validator = Validator, Epoch = Epoch>
        + PerformanceAlg<Validator = Validator, Epoch = Epoch, Performance = Performance>
        + EmissionAlg<Amount = Amount, Epoch = Epoch>
        + RewardShareAlg<Validator = Validator, Amount = Amount, Performance = Performance>
        + RewardPayoutAlg<Validator = Validator, Amount = Amount>,
    Validator: PartialEq,
    Amount: Copy + Ord + Default + Add<Output = Amount> + Sub<Output = Amount>,
    Epoch: Clone + Ord,
{
    /// Pays every weighted validator its share of the epoch's pot; paid and unpaid sum to the pot.
    fn stake_reward(&mut self, epoch: &Epoch) -> RewardSettlement<Amount> {
        let pot = self.emission(epoch);
        let total = self.stake_total(epoch);
        let mut paid = Amount::default();
        for (validator, weight) in self.stake_weights(epoch) {
            let performance = self.performance(&validator, epoch);
            let share = self.reward_share(&validator, &pot, &weight, &total, &performance).min(pot - paid);
            self.reward_pay(&validator, share);
            paid = paid + share;
        }
        let unpaid = pot - paid;
        self.reward_unpaid(unpaid);
        RewardSettlement { paid, unpaid }
    }
}
