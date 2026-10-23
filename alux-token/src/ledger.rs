use crate::Shortfall;
use alux_ext::ext;
use std::ops::{Add, Sub};

/// Keeps how much of the token each holder has.
///
/// A holder never assigned a balance holds zero.
pub trait BalancesAlg {
    /// Identifies a holder.
    type Holder;
    /// Measures a quantity of the token.
    type Amount;

    /// Returns how much the holder has.
    fn balance(&self, holder: &Self::Holder) -> Self::Amount;

    /// Replaces how much the holder has.
    fn balance_set(&mut self, holder: &Self::Holder, amount: Self::Amount);

    /// Returns every holder whose balance is not zero.
    fn balance_holders(&self) -> Vec<Self::Holder>;
}

/// Keeps how much of the token exists.
///
/// The supply equals the sum of every balance; the ledger operations preserve that equation.
pub trait SupplyAlg {
    /// Measures a quantity of the token.
    type Amount;

    /// Returns how much of the token exists.
    fn supply(&self) -> Self::Amount;

    /// Replaces how much of the token exists.
    fn supply_set(&mut self, amount: Self::Amount);
}

/// Derives moving, creating, and destroying tokens from balances and supply.
///
/// Zero is `Amount::default()`. Every operation either completes or changes nothing.
#[ext(name = TokenLedgerExt)]
pub impl<This, Holder, Amount> This
where
    This: BalancesAlg<Holder = Holder, Amount = Amount> + SupplyAlg<Amount = Amount>,
    Amount: Copy + Ord + Add<Output = Amount> + Sub<Output = Amount>,
{
    /// Moves an amount from one holder to another, keeping the supply.
    ///
    /// # Errors
    ///
    /// Returns the shortfall when the sender holds less than the amount.
    fn token_transfer(&mut self, from: &Holder, to: &Holder, amount: Amount) -> Result<(), Shortfall<Amount>> {
        let available = self.balance(from);
        if available < amount {
            return Err(Shortfall { available, requested: amount });
        }
        self.balance_set(from, available - amount);
        let received = self.balance(to);
        self.balance_set(to, received + amount);
        Ok(())
    }

    /// Creates an amount in a holder's balance, raising the supply by exactly that amount.
    fn token_mint(&mut self, to: &Holder, amount: Amount) {
        let received = self.balance(to);
        self.balance_set(to, received + amount);
        let supply = self.supply();
        self.supply_set(supply + amount);
    }

    /// Destroys an amount of a holder's balance, lowering the supply by exactly that amount.
    ///
    /// # Errors
    ///
    /// Returns the shortfall when the holder has less than the amount.
    fn token_burn(&mut self, from: &Holder, amount: Amount) -> Result<(), Shortfall<Amount>> {
        let available = self.balance(from);
        if available < amount {
            return Err(Shortfall { available, requested: amount });
        }
        self.balance_set(from, available - amount);
        let supply = self.supply();
        self.supply_set(supply - amount);
        Ok(())
    }
}
