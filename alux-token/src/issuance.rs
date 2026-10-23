use crate::{BalancesAlg, SupplyAlg, TokenLedgerExt};
use alux_ext::ext;
use std::ops::{Add, Sub};

/// Decides who may create new tokens, and how many.
///
/// A token does not decide why an issuer may mint. One policy names a single minter; another caps
/// the supply; another lets a governance vote decide.
pub trait IssuancePolicyAlg {
    /// Identifies a holder acting as issuer.
    type Holder;
    /// Measures a quantity of the token.
    type Amount;
    /// Explains why issuance was refused.
    type Denial;

    /// Refuses unless the issuer may create the amount now.
    ///
    /// # Errors
    ///
    /// Returns the policy's denial when the issuer may not create it.
    fn issuance_require(&self, issuer: &Self::Holder, amount: &Self::Amount) -> Result<(), Self::Denial>;
}

/// Derives governed issuance from the ledger and an issuance policy.
#[ext(name = TokenIssuanceExt)]
pub impl<This, Holder, Amount, Denial> This
where
    This: BalancesAlg<Holder = Holder, Amount = Amount>
        + SupplyAlg<Amount = Amount>
        + IssuancePolicyAlg<Holder = Holder, Amount = Amount, Denial = Denial>,
    Amount: Copy + Ord + Add<Output = Amount> + Sub<Output = Amount>,
{
    /// Creates an amount for a recipient when the issuer may create it.
    ///
    /// # Errors
    ///
    /// Returns the policy's denial; the ledger does not change then.
    fn token_issue(&mut self, issuer: &Holder, to: &Holder, amount: Amount) -> Result<(), Denial> {
        self.issuance_require(issuer, &amount)?;
        self.token_mint(to, amount);
        Ok(())
    }
}
