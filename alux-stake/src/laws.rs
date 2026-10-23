use crate::{
    BondShortfall, BondsAlg, CustodyAlg, DelegationAlg, EmissionAlg, EpochAlg, ExclusionAlg, ForfeitAlg, HoldingsAlg,
    PerformanceAlg, QuorumAlg, RewardPayoutAlg, RewardShareAlg, SnapshotAlg, SnapshotLagAlg, StakeDelegationExt,
    StakeEscrowExt, StakeForfeitExt, StakeQuorumExt, StakeRewardsExt, StakeWeightExt, Unbonding, UnbondingAlg,
    UnbondingDelayAlg, WeightSourceAlg,
};
use alux_ext::ext;
use anyhow::{Result, ensure};
use std::{
    fmt::Debug,
    ops::{Add, Sub},
};

/// Checks the core every variant shares: a snapshot records the source's weights of validators not
/// excluded, and exclusion removes weight without touching the source.
#[ext(name = StakeWeightLawsExt)]
pub impl<This, Validator, Amount, Epoch> This
where
    This: EpochAlg<Epoch = Epoch>
        + WeightSourceAlg<Validator = Validator, Amount = Amount>
        + SnapshotAlg<Validator = Validator, Amount = Amount, Epoch = Epoch>
        + SnapshotLagAlg<Epoch = Epoch>
        + ExclusionAlg<Validator = Validator, Epoch = Epoch>,
    Validator: PartialEq + Debug,
    Amount: Copy + Ord + Default + Debug + Add<Output = Amount>,
    Epoch: Clone + Ord + Debug,
{
    /// Checks that taking a snapshot records exactly the source's weights of validators with stake
    /// that are not excluded in the current epoch.
    fn check_snapshot(&mut self) -> Result<()> {
        let epoch = self.epoch();
        let source = self.source_weights();
        self.stake_snapshot_take();
        let recorded = self.snapshot(&epoch);
        for (validator, weight) in &source {
            let excluded = self.excluded_until(validator).is_some_and(|until| until > epoch);
            let kept = recorded.iter().find(|(kept, _)| kept == validator).map(|(_, kept)| *kept);
            let expected = (*weight > Amount::default() && !excluded).then_some(*weight);
            ensure!(kept == expected, "the snapshot recorded {kept:?} for {validator:?} with {weight:?}");
        }
        ensure!(recorded.len() <= source.len(), "the snapshot recorded a validator the source does not know");
        Ok(())
    }

    /// Checks that an epoch's weights are ordered heaviest first and sum to its total.
    fn check_weights(&self, epoch: &Epoch) -> Result<()> {
        let weights = self.stake_weights(epoch);
        ensure!(weights.windows(2).all(|pair| pair[0].1 >= pair[1].1), "weights are not ordered: {weights:?}");
        let sum = weights.iter().map(|(_, weight)| *weight).fold(Amount::default(), Add::add);
        ensure!(self.stake_total(epoch) == sum, "the total differs from the sum of weights");
        Ok(())
    }

    /// Checks that excluding a validator keeps the source and leaves the validator out of every
    /// snapshot taken before the exclusion ends.
    fn check_exclusion(&mut self, validator: &Validator, until: Epoch) -> Result<()> {
        let source = self.source_weights();
        self.stake_exclude(validator, until.clone());
        ensure!(self.source_weights() == source, "exclusion touched the stake behind validators");
        self.check_snapshot()?;
        let epoch = self.epoch();
        let present = self.snapshot(&epoch).iter().any(|(kept, _)| kept == validator);
        ensure!(!(present && epoch < until), "an excluded validator carried weight in {epoch:?}");
        Ok(())
    }
}

/// Checks that quorum counts each validator once and only grows with more voters.
#[ext(name = StakeQuorumLawsExt)]
pub impl<This, Validator, Amount, Epoch> This
where
    This: EpochAlg<Epoch = Epoch>
        + WeightSourceAlg<Validator = Validator, Amount = Amount>
        + SnapshotAlg<Validator = Validator, Amount = Amount, Epoch = Epoch>
        + SnapshotLagAlg<Epoch = Epoch>
        + ExclusionAlg<Validator = Validator, Epoch = Epoch>
        + QuorumAlg<Amount = Amount>,
    Validator: Clone + PartialEq + Debug,
    Amount: Copy + Ord + Default + Add<Output = Amount>,
    Epoch: Clone + Ord,
{
    /// Checks that repeating every vote does not change the outcome.
    fn check_repeated_votes(&self, epoch: &Epoch, voters: &[Validator]) -> Result<()> {
        let repeated: Vec<_> = voters.iter().chain(voters).cloned().collect();
        ensure!(self.stake_quorum(epoch, voters) == self.stake_quorum(epoch, &repeated), "a repeated vote counted");
        Ok(())
    }

    /// Checks that adding voters never loses a quorum.
    fn check_quorum_grows(&self, epoch: &Epoch, voters: &[Validator], more: &[Validator]) -> Result<()> {
        let all: Vec<_> = voters.iter().chain(more).cloned().collect();
        ensure!(
            !self.stake_quorum(epoch, voters) || self.stake_quorum(epoch, &all),
            "adding {more:?} to {voters:?} lost a quorum"
        );
        Ok(())
    }
}

/// Checks that settling rewards splits the pot exactly and never touches the stake behind validators.
#[ext(name = StakeRewardsLawsExt)]
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
    Validator: PartialEq + Debug,
    Amount: Copy + Ord + Default + Debug + Add<Output = Amount> + Sub<Output = Amount>,
    Epoch: Clone + Ord,
{
    /// Checks that paid and unpaid sum to the epoch's pot, and that the snapshot is unchanged.
    fn check_reward(&mut self, epoch: &Epoch) -> Result<()> {
        let pot = self.emission(epoch);
        let weights = self.stake_weights(epoch);
        let settlement = self.stake_reward(epoch);
        ensure!(settlement.paid + settlement.unpaid == pot, "{settlement:?} does not settle the pot {pot:?}");
        ensure!(self.stake_weights(epoch) == weights, "settling rewards changed the epoch's weights");
        Ok(())
    }
}

/// Checks that escrow moves stake between bonds and unbondings by exactly the amounts asked.
#[ext(name = StakeEscrowLawsExt)]
pub impl<This, Delegator, Validator, Amount, Epoch, Denial> This
where
    This: BondsAlg<Delegator = Delegator, Validator = Validator, Amount = Amount>
        + UnbondingAlg<Delegator = Delegator, Validator = Validator, Amount = Amount, Epoch = Epoch>
        + EpochAlg<Epoch = Epoch>
        + UnbondingDelayAlg<Epoch = Epoch>
        + CustodyAlg<Delegator = Delegator, Amount = Amount, Denial = Denial>,
    Validator: Clone + PartialEq + Debug,
    Amount: Copy + Ord + Default + Debug + Add<Output = Amount> + Sub<Output = Amount>,
    Epoch: Ord + Debug,
{
    /// Checks that a bond grows by exactly the amount custody accepted, and not at all otherwise.
    fn check_bond(&mut self, delegator: &Delegator, validator: &Validator, amount: Amount) -> Result<()> {
        let bonded = self.bond(delegator, validator);
        let expected = if self.stake_bond(delegator, validator, amount).is_ok() { bonded + amount } else { bonded };
        ensure!(self.bond(delegator, validator) == expected, "bonding changed the bond by another amount");
        Ok(())
    }

    /// Checks that unbonding moves exactly the amount into a pending unbonding maturing after the
    /// delay, or refuses more than is bonded and changes nothing.
    fn check_unbond(&mut self, delegator: &Delegator, validator: &Validator, amount: Amount) -> Result<()> {
        let bonded = self.bond(delegator, validator);
        let mut pending = self.unbonding(delegator);
        if let Err(shortfall) = self.stake_unbond(delegator, validator.clone(), amount) {
            ensure!(shortfall == BondShortfall { bonded, requested: amount }, "a wrong shortfall");
            ensure!(bonded < amount, "a covered unbonding was refused");
            ensure!(self.bond(delegator, validator) == bonded, "a refused unbonding changed the bond");
        } else {
            ensure!(self.bond(delegator, validator) == bonded - amount, "unbonding took another amount");
            let matures = self.unbonding_matures(&self.epoch());
            pending.push(Unbonding { validator: validator.clone(), amount, matures });
        }
        ensure!(self.unbonding(delegator) == pending, "pending unbondings changed by something else");
        Ok(())
    }

    /// Checks that withdrawing returns exactly the matured unbondings and keeps the immature ones.
    fn check_withdraw(&mut self, delegator: &Delegator) -> Result<()> {
        let now = self.epoch();
        let (matured, immature): (Vec<_>, Vec<_>) =
            self.unbonding(delegator).into_iter().partition(|unbonding| unbonding.matures <= now);
        let expected = matured.iter().map(|unbonding| unbonding.amount).fold(Amount::default(), Add::add);
        let withdrawn = self.stake_withdraw(delegator);
        ensure!(withdrawn == expected, "withdrew {withdrawn:?} of {expected:?} matured");
        ensure!(self.unbonding(delegator) == immature, "withdrawing touched immature unbondings");
        Ok(())
    }
}

/// Checks that forfeiting takes exactly what it reports, only from the offending validator's
/// bonds and immature unbondings.
#[ext(name = StakeForfeitLawsExt)]
pub impl<This, Delegator, Validator, Amount, Epoch, Denial, Offense> This
where
    This: BondsAlg<Delegator = Delegator, Validator = Validator, Amount = Amount>
        + UnbondingAlg<Delegator = Delegator, Validator = Validator, Amount = Amount, Epoch = Epoch>
        + EpochAlg<Epoch = Epoch>
        + UnbondingDelayAlg<Epoch = Epoch>
        + CustodyAlg<Delegator = Delegator, Amount = Amount, Denial = Denial>
        + ForfeitAlg<Offense = Offense, Amount = Amount>,
    Validator: PartialEq + Debug,
    Amount: Copy + Ord + Default + Debug + Add<Output = Amount> + Sub<Output = Amount>,
    Epoch: Ord,
{
    /// Checks that the amount forfeited equals what left the validator's answerable stake, and that
    /// every other validator keeps its stake.
    fn check_forfeit(&mut self, validator: &Validator, offense: &Offense) -> Result<()> {
        let answerable = |this: &Self| {
            let now = this.epoch();
            let pending = this
                .unbonding_delegators()
                .iter()
                .flat_map(|delegator| this.unbonding(delegator))
                .filter(|unbonding| unbonding.validator == *validator && unbonding.matures > now)
                .map(|unbonding| unbonding.amount)
                .fold(Amount::default(), Add::add);
            let bonded = this
                .escrow_weights()
                .into_iter()
                .filter(|(kept, _)| kept == validator)
                .map(|(_, weight)| weight)
                .fold(Amount::default(), Add::add);
            bonded + pending
        };
        let others = |this: &Self| -> Vec<_> {
            this.escrow_weights().into_iter().filter(|(kept, _)| kept != validator).collect()
        };
        let (before, untouched) = (answerable(self), others(self));
        let forfeited = self.stake_forfeit(validator, offense);
        ensure!(before - answerable(self) == forfeited, "forfeiting reported {forfeited:?} but took another amount");
        ensure!(others(self) == untouched, "forfeiting reached another validator");
        Ok(())
    }
}

/// Checks that delegation moves no value and supplies exactly the delegators' holdings.
#[ext(name = StakeDelegationLawsExt)]
pub impl<This, Holder, Validator, Amount> This
where
    This: DelegationAlg<Holder = Holder, Validator = Validator> + HoldingsAlg<Holder = Holder, Amount = Amount>,
    Validator: Clone + PartialEq + Debug,
    Amount: Copy + Ord + Default + Debug + Add<Output = Amount>,
{
    /// Checks that delegating keeps the holder's holdings and points them at the validator.
    fn check_delegate(&mut self, holder: &Holder, validator: Option<Validator>) -> Result<()> {
        let holding = self.holding(holder);
        self.stake_delegate(holder, validator.clone());
        ensure!(self.holding(holder) == holding, "delegating moved value");
        ensure!(self.delegation(holder) == validator, "delegating pointed elsewhere");
        self.check_delegated_weights()
    }

    /// Checks that delegated weights sum to the holdings of every delegating holder.
    fn check_delegated_weights(&self) -> Result<()> {
        let weights = self.delegated_weights().into_iter().map(|(_, weight)| weight).fold(Amount::default(), Add::add);
        let holdings = self
            .delegation_holders()
            .iter()
            .filter(|holder| self.delegation(holder).is_some())
            .map(|holder| self.holding(holder))
            .fold(Amount::default(), Add::add);
        ensure!(weights == holdings, "delegated weights {weights:?} differ from delegated holdings {holdings:?}");
        Ok(())
    }
}
