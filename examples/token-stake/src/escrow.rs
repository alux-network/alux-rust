use crate::{Account, Chain, STAKE_POOL};
use alux_stake::{BondsAlg, CustodyAlg, Unbonding, UnbondingAlg};
use alux_token::{Shortfall, TokenLedgerExt};

/// Marks a variant whose stake is escrowed: bonded tokens move into the stake pool and leave through
/// unbonding.
pub trait Escrowed {}

impl<Variant> BondsAlg for Chain<Variant>
where
    Variant: Escrowed,
{
    type Delegator = Account;
    type Validator = Account;
    type Amount = u128;

    fn bond(&self, delegator: &Account, validator: &Account) -> u128 {
        self.bonds.get(&(*delegator, *validator)).copied().unwrap_or_default()
    }

    fn bond_set(&mut self, delegator: &Account, validator: &Account, amount: u128) {
        self.bonds.insert((*delegator, *validator), amount);
        self.bonds.retain(|_, kept| *kept != 0);
    }

    fn bond_delegators(&self, validator: &Account) -> Vec<Account> {
        self.bonds.keys().filter(|(_, kept)| kept == validator).map(|(delegator, _)| *delegator).collect()
    }

    fn bond_validators(&self) -> Vec<Account> {
        let mut validators: Vec<_> = self.bonds.keys().map(|(_, validator)| *validator).collect();
        validators.sort_unstable();
        validators.dedup();
        validators
    }
}

impl<Variant> UnbondingAlg for Chain<Variant>
where
    Variant: Escrowed,
{
    type Delegator = Account;
    type Validator = Account;
    type Amount = u128;
    type Epoch = u64;

    fn unbonding(&self, delegator: &Account) -> Vec<Unbonding<Account, u128, u64>> {
        self.unbonding.get(delegator).cloned().unwrap_or_default()
    }

    fn unbonding_set(&mut self, delegator: &Account, pending: Vec<Unbonding<Account, u128, u64>>) {
        self.unbonding.insert(*delegator, pending);
        self.unbonding.retain(|_, kept| !kept.is_empty());
    }

    fn unbonding_delegators(&self) -> Vec<Account> {
        self.unbonding.keys().copied().collect()
    }
}

/// Holds escrowed tokens in the stake pool's balance.
impl<Variant> CustodyAlg for Chain<Variant>
where
    Variant: Escrowed,
{
    type Delegator = Account;
    type Amount = u128;
    type Denial = Shortfall<u128>;

    fn custody_deposit(&mut self, from: &Account, amount: u128) -> Result<(), Shortfall<u128>> {
        self.token_transfer(from, &STAKE_POOL, amount)
    }

    fn custody_withdraw(&mut self, to: &Account, amount: u128) {
        self.token_transfer(&STAKE_POOL, to, amount).expect("the stake pool holds every unbonding");
    }

    fn custody_forfeit(&mut self, amount: u128) {
        self.token_burn(&STAKE_POOL, amount).expect("the stake pool holds every bond");
    }
}
