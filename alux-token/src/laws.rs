use crate::{BalancesAlg, IssuancePolicyAlg, Shortfall, SupplyAlg, TokenIssuanceExt, TokenLedgerExt};
use alux_ext::ext;
use anyhow::{Result, ensure};
use std::{
    fmt::Debug,
    ops::{Add, Sub},
};

/// Checks the obligations every token ledger keeps: moving keeps the supply, and minting and burning
/// change it by exactly their amount.
#[ext(name = TokenLawsExt)]
pub impl<This, Holder, Amount> This
where
    This: BalancesAlg<Holder = Holder, Amount = Amount> + SupplyAlg<Amount = Amount>,
    Holder: PartialEq + Debug,
    Amount: Copy + Ord + Default + Debug + Add<Output = Amount> + Sub<Output = Amount>,
{
    /// Checks that the supply equals the sum of every balance.
    fn check_supply_is_the_sum_of_balances(&self) -> Result<()> {
        let sum = self.balance_holders().iter().map(|holder| self.balance(holder)).fold(Amount::default(), Add::add);
        ensure!(self.supply() == sum, "the supply {:?} differs from the sum of balances {sum:?}", self.supply());
        Ok(())
    }

    /// Checks that a transfer moves exactly its amount and keeps the supply, or refuses an overdraft
    /// and changes nothing.
    fn check_transfer(&mut self, from: &Holder, to: &Holder, amount: Amount) -> Result<()> {
        let (sent, received, supply) = (self.balance(from), self.balance(to), self.supply());
        match self.token_transfer(from, to, amount) {
            Err(shortfall) => {
                ensure!(sent < amount, "a covered transfer of {amount:?} from {sent:?} was refused");
                ensure!(shortfall == Shortfall { available: sent, requested: amount }, "a wrong shortfall");
                ensure!((self.balance(from), self.balance(to)) == (sent, received), "a refused transfer moved value");
            }
            Ok(()) if from == to => ensure!(self.balance(from) == sent, "a transfer to oneself changed the balance"),
            Ok(()) => {
                ensure!(self.balance(from) == sent - amount, "the sender was not debited exactly {amount:?}");
                ensure!(self.balance(to) == received + amount, "the recipient was not credited exactly {amount:?}");
            }
        }
        ensure!(self.supply() == supply, "a transfer changed the supply");
        self.check_supply_is_the_sum_of_balances()
    }

    /// Checks that minting raises the balance and supply by exactly the amount, and burning it again
    /// restores both.
    fn check_mint_then_burn(&mut self, holder: &Holder, amount: Amount) -> Result<()> {
        let (balance, supply) = (self.balance(holder), self.supply());
        self.token_mint(holder, amount);
        ensure!(self.balance(holder) == balance + amount, "minting did not credit exactly {amount:?}");
        ensure!(self.supply() == supply + amount, "minting did not raise the supply by exactly {amount:?}");
        self.check_supply_is_the_sum_of_balances()?;
        ensure!(self.token_burn(holder, amount).is_ok(), "burning what was just minted was refused");
        ensure!((self.balance(holder), self.supply()) == (balance, supply), "burning did not undo minting");
        self.check_supply_is_the_sum_of_balances()
    }

    /// Checks that burning lowers the balance and supply by exactly the amount, or refuses more than
    /// the holder has and changes nothing.
    fn check_burn(&mut self, holder: &Holder, amount: Amount) -> Result<()> {
        let (balance, supply) = (self.balance(holder), self.supply());
        if let Err(shortfall) = self.token_burn(holder, amount) {
            ensure!(balance < amount, "a covered burn of {amount:?} from {balance:?} was refused");
            ensure!(shortfall == Shortfall { available: balance, requested: amount }, "a wrong shortfall");
            ensure!((self.balance(holder), self.supply()) == (balance, supply), "a refused burn changed state");
        } else {
            ensure!(self.balance(holder) == balance - amount, "burning did not debit exactly {amount:?}");
            ensure!(self.supply() == supply - amount, "burning did not lower the supply by exactly {amount:?}");
        }
        self.check_supply_is_the_sum_of_balances()
    }
}

/// Checks that issuance does exactly what the policy permits.
#[ext(name = TokenIssuanceLawsExt)]
pub impl<This, Holder, Amount, Denial> This
where
    This: BalancesAlg<Holder = Holder, Amount = Amount>
        + SupplyAlg<Amount = Amount>
        + IssuancePolicyAlg<Holder = Holder, Amount = Amount, Denial = Denial>,
    Holder: PartialEq + Debug,
    Amount: Copy + Ord + Default + Debug + Add<Output = Amount> + Sub<Output = Amount>,
{
    /// Checks that a permitted issue credits exactly its amount and a refused one changes nothing.
    fn check_issue(&mut self, issuer: &Holder, to: &Holder, amount: Amount) -> Result<()> {
        let permitted = self.issuance_require(issuer, &amount).is_ok();
        let (balance, supply) = (self.balance(to), self.supply());
        let accepted = self.token_issue(issuer, to, amount).is_ok();
        ensure!(accepted == permitted, "issuance disagreed with the policy");
        let expected = if accepted { (balance + amount, supply + amount) } else { (balance, supply) };
        ensure!((self.balance(to), self.supply()) == expected, "issuance changed the ledger by another amount");
        self.check_supply_is_the_sum_of_balances()
    }
}
