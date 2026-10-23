use crate::{BondShortfall, EpochAlg, Unbonding};
use alux_ext::ext;
use std::ops::{Add, Sub};

/// Keeps how much each delegator has bonded to each validator.
///
/// A pair never assigned a bond has bonded zero.
pub trait BondsAlg {
    /// Identifies whoever bonds stake.
    type Delegator;
    /// Identifies a validator stake is bonded to.
    type Validator;
    /// Measures a quantity of stake.
    type Amount;

    /// Returns how much the delegator has bonded to the validator.
    fn bond(&self, delegator: &Self::Delegator, validator: &Self::Validator) -> Self::Amount;

    /// Replaces how much the delegator has bonded to the validator.
    fn bond_set(&mut self, delegator: &Self::Delegator, validator: &Self::Validator, amount: Self::Amount);

    /// Returns every delegator with a bond to the validator that is not zero.
    fn bond_delegators(&self, validator: &Self::Validator) -> Vec<Self::Delegator>;

    /// Returns every validator with at least one bond that is not zero.
    fn bond_validators(&self) -> Vec<Self::Validator>;
}

/// Keeps the escrowed stake each delegator has asked to unbond and not yet withdrawn.
pub trait UnbondingAlg {
    /// Identifies whoever bonds stake.
    type Delegator;
    /// Identifies a validator stake is bonded to.
    type Validator;
    /// Measures a quantity of stake.
    type Amount;
    /// Orders time in epochs.
    type Epoch;

    /// Returns the delegator's pending unbondings, oldest first.
    fn unbonding(&self, delegator: &Self::Delegator) -> Vec<Unbonding<Self::Validator, Self::Amount, Self::Epoch>>;

    /// Replaces the delegator's pending unbondings.
    fn unbonding_set(
        &mut self,
        delegator: &Self::Delegator,
        pending: Vec<Unbonding<Self::Validator, Self::Amount, Self::Epoch>>,
    );

    /// Returns every delegator with a pending unbonding.
    fn unbonding_delegators(&self) -> Vec<Self::Delegator>;
}

/// Tells how long escrowed stake takes to leave.
pub trait UnbondingDelayAlg {
    /// Orders time in epochs.
    type Epoch;

    /// Returns the epoch at which stake that starts unbonding in the given epoch matures.
    fn unbonding_matures(&self, start: &Self::Epoch) -> Self::Epoch;
}

/// Holds the value backing escrowed stake on the stake's behalf.
///
/// Proof of stake does not decide what stake is made of. A chain lends custody from its native
/// token ledger; another lends it from a vault on another chain.
pub trait CustodyAlg {
    /// Identifies whoever bonds stake.
    type Delegator;
    /// Measures a quantity of stake.
    type Amount;
    /// Explains why a deposit was refused.
    type Denial;

    /// Takes an amount from the delegator into custody.
    ///
    /// # Errors
    ///
    /// Returns the custodian's denial, such as an insufficient balance; nothing moves then.
    fn custody_deposit(&mut self, from: &Self::Delegator, amount: Self::Amount) -> Result<(), Self::Denial>;

    /// Returns an amount from custody to the delegator.
    fn custody_withdraw(&mut self, to: &Self::Delegator, amount: Self::Amount);

    /// Destroys an amount held in custody.
    fn custody_forfeit(&mut self, amount: Self::Amount);
}

/// Decides how much of an escrowed stake a proven offense forfeits.
///
/// Consensus proves the offense, such as signing two blocks at one height; this policy prices it. A
/// fraction gives proportional slashing. A chain that does not forfeit stake does not answer this.
pub trait ForfeitAlg {
    /// Describes a proven offense.
    type Offense;
    /// Measures a quantity of stake.
    type Amount;

    /// Returns how much of the stake the offense forfeits; never more than the stake.
    fn forfeit_penalty(&self, offense: &Self::Offense, stake: &Self::Amount) -> Self::Amount;
}

/// Derives escrowed stake: bonding, unbonding, withdrawal, and the weight it supplies.
///
/// Custody holds exactly what is bonded plus what is unbonding; every operation here keeps that.
#[ext(name = StakeEscrowExt)]
pub impl<This, Delegator, Validator, Amount, Epoch, Denial> This
where
    This: BondsAlg<Delegator = Delegator, Validator = Validator, Amount = Amount>
        + UnbondingAlg<Delegator = Delegator, Validator = Validator, Amount = Amount, Epoch = Epoch>
        + EpochAlg<Epoch = Epoch>
        + UnbondingDelayAlg<Epoch = Epoch>
        + CustodyAlg<Delegator = Delegator, Amount = Amount, Denial = Denial>,
    Amount: Copy + Ord + Default + Add<Output = Amount> + Sub<Output = Amount>,
    Epoch: Ord,
{
    /// Deposits an amount into custody and bonds it to the validator.
    ///
    /// # Errors
    ///
    /// Returns the custodian's denial; no bond changes then.
    fn stake_bond(&mut self, delegator: &Delegator, validator: &Validator, amount: Amount) -> Result<(), Denial> {
        self.custody_deposit(delegator, amount)?;
        let bonded = self.bond(delegator, validator);
        self.bond_set(delegator, validator, bonded + amount);
        Ok(())
    }

    /// Moves an amount from the bond into a pending unbonding that matures after the unbonding delay.
    ///
    /// # Errors
    ///
    /// Returns the shortfall when less than the amount is bonded; nothing changes then.
    fn stake_unbond(
        &mut self,
        delegator: &Delegator,
        validator: Validator,
        amount: Amount,
    ) -> Result<(), BondShortfall<Amount>> {
        let bonded = self.bond(delegator, &validator);
        if bonded < amount {
            return Err(BondShortfall { bonded, requested: amount });
        }
        self.bond_set(delegator, &validator, bonded - amount);
        let matures = self.unbonding_matures(&self.epoch());
        let mut pending = self.unbonding(delegator);
        pending.push(Unbonding { validator, amount, matures });
        self.unbonding_set(delegator, pending);
        Ok(())
    }

    /// Returns every matured unbonding of the delegator from custody, and the amount returned.
    fn stake_withdraw(&mut self, delegator: &Delegator) -> Amount {
        let now = self.epoch();
        let (matured, pending): (Vec<_>, Vec<_>) =
            self.unbonding(delegator).into_iter().partition(|unbonding| unbonding.matures <= now);
        self.unbonding_set(delegator, pending);
        let amount = matured.into_iter().map(|unbonding| unbonding.amount).fold(Amount::default(), Add::add);
        self.custody_withdraw(delegator, amount);
        amount
    }

    /// Returns every validator's bonded stake: the weight escrow supplies.
    fn escrow_weights(&self) -> Vec<(Validator, Amount)> {
        self.bond_validators()
            .into_iter()
            .map(|validator| {
                let bonded = self
                    .bond_delegators(&validator)
                    .iter()
                    .map(|delegator| self.bond(delegator, &validator))
                    .fold(Amount::default(), Add::add);
                (validator, bonded)
            })
            .collect()
    }
}

/// Derives forfeiting escrowed stake for a proven offense.
///
/// An offense forfeits from every bond to the validator and from every unbonding from it that has
/// not matured, so leaving does not escape an offense committed while bonded.
#[ext(name = StakeForfeitExt)]
pub impl<This, Delegator, Validator, Amount, Epoch, Denial, Offense> This
where
    This: BondsAlg<Delegator = Delegator, Validator = Validator, Amount = Amount>
        + UnbondingAlg<Delegator = Delegator, Validator = Validator, Amount = Amount, Epoch = Epoch>
        + EpochAlg<Epoch = Epoch>
        + CustodyAlg<Delegator = Delegator, Amount = Amount, Denial = Denial>
        + ForfeitAlg<Offense = Offense, Amount = Amount>,
    Validator: PartialEq,
    Amount: Copy + Ord + Default + Add<Output = Amount> + Sub<Output = Amount>,
    Epoch: Ord,
{
    /// Forfeits the offense's penalty from the validator's escrowed stake and returns the total.
    fn stake_forfeit(&mut self, validator: &Validator, offense: &Offense) -> Amount {
        let mut forfeited = Amount::default();
        for delegator in self.bond_delegators(validator) {
            let bonded = self.bond(&delegator, validator);
            let penalty = self.forfeit_penalty(offense, &bonded).min(bonded);
            self.bond_set(&delegator, validator, bonded - penalty);
            forfeited = forfeited + penalty;
        }
        let now = self.epoch();
        for delegator in self.unbonding_delegators() {
            let pending: Vec<_> = self
                .unbonding(&delegator)
                .into_iter()
                .map(|unbonding| {
                    if unbonding.validator != *validator || unbonding.matures <= now {
                        return unbonding;
                    }
                    let penalty = self.forfeit_penalty(offense, &unbonding.amount).min(unbonding.amount);
                    forfeited = forfeited + penalty;
                    Unbonding { amount: unbonding.amount - penalty, ..unbonding }
                })
                .collect();
            self.unbonding_set(&delegator, pending);
        }
        self.custody_forfeit(forfeited);
        forfeited
    }
}
