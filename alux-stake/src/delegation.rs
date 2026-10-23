use alux_ext::ext;
use std::ops::Add;

/// Keeps which validator each holder points its stake at.
///
/// Delegating moves no value: the holder keeps its holdings and may spend them at any time.
pub trait DelegationAlg {
    /// Identifies a holder.
    type Holder;
    /// Identifies a validator, such as a stake pool.
    type Validator;

    /// Returns the validator the holder delegates to, if any.
    fn delegation(&self, holder: &Self::Holder) -> Option<Self::Validator>;

    /// Replaces the validator the holder delegates to.
    fn delegation_set(&mut self, holder: &Self::Holder, validator: Option<Self::Validator>);

    /// Returns every holder that delegates.
    fn delegation_holders(&self) -> Vec<Self::Holder>;
}

/// Tells how much value a holder has, as delegated stake counts it.
///
/// A chain whose stake is its native token answers with the holder's balance.
pub trait HoldingsAlg {
    /// Identifies a holder.
    type Holder;
    /// Measures a quantity of stake.
    type Amount;

    /// Returns how much the holder has.
    fn holding(&self, holder: &Self::Holder) -> Self::Amount;
}

/// Derives delegated stake: pointing holdings at validators, and the weight that supplies.
#[ext(name = StakeDelegationExt)]
pub impl<This, Holder, Validator, Amount> This
where
    This: DelegationAlg<Holder = Holder, Validator = Validator> + HoldingsAlg<Holder = Holder, Amount = Amount>,
    Validator: PartialEq,
    Amount: Copy + Default + Add<Output = Amount>,
{
    /// Points the holder's stake at the validator, or withdraws it with `None`.
    fn stake_delegate(&mut self, holder: &Holder, validator: Option<Validator>) {
        self.delegation_set(holder, validator);
    }

    /// Returns every validator's delegated holdings: the weight delegation supplies.
    fn delegated_weights(&self) -> Vec<(Validator, Amount)> {
        let mut weights: Vec<(Validator, Amount)> = Vec::new();
        for holder in self.delegation_holders() {
            let Some(validator) = self.delegation(&holder) else { continue };
            let holding = self.holding(&holder);
            match weights.iter_mut().find(|(kept, _)| *kept == validator) {
                Some((_, weight)) => *weight = *weight + holding,
                None => weights.push((validator, holding)),
            }
        }
        weights
    }
}
