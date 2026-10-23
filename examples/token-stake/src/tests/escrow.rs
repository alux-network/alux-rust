use crate::STAKE_POOL;
use crate::tests::{ALICE, BOB, EscrowChainAlg, OTHER_VALIDATOR, VALIDATOR};
use alux_ext::ext;
use alux_stake::StakeEscrowExt;
use alux_token::Shortfall;
use anyhow::{Result, ensure};

/// States what every chain whose stake is escrowed in its token ledger keeps.
#[ext(name = EscrowScenariosExt)]
pub(crate) impl<This> This
where
    This: EscrowChainAlg,
{
    /// Checks that the stake pool's balance is exactly what is bonded plus what is unbonding.
    fn pool_holds_what_is_escrowed(&self) -> Result<()> {
        let unbonding: u128 = self
            .unbonding_delegators()
            .iter()
            .flat_map(|delegator| self.unbonding(delegator))
            .map(|unbonding| unbonding.amount)
            .sum();
        let escrowed = self.escrow_weights().iter().map(|(_, weight)| weight).sum::<u128>() + unbonding;
        ensure!(self.balance(&STAKE_POOL) == escrowed, "the pool holds {} for {escrowed}", self.balance(&STAKE_POOL));
        Ok(())
    }

    fn scenario_escrow_keeps_the_supply(&mut self) -> Result<()> {
        let supply = self.supply();
        self.stake_bond(&ALICE, &VALIDATOR, 400).unwrap();
        self.stake_bond(&BOB, &OTHER_VALIDATOR, 300).unwrap();
        ensure!(self.balance(&ALICE) == 600, "bonding did not spend the delegator's tokens");
        self.stake_unbond(&ALICE, VALIDATOR, 100).unwrap();
        self.pool_holds_what_is_escrowed()?;
        let matures = self.unbonding(&ALICE)[0].matures;
        while self.epoch() < matures {
            self.scenario_epoch_close();
        }
        ensure!(self.stake_withdraw(&ALICE) == 100, "matured stake was not returned");
        ensure!(self.supply() == supply, "escrow changed the supply");
        self.pool_holds_what_is_escrowed()
    }

    fn scenario_custody_refuses_what_the_ledger_refuses(&mut self) -> Result<()> {
        let available = self.balance(&ALICE);
        let refused = self.stake_bond(&ALICE, &VALIDATOR, available + 1);
        ensure!(refused == Err(Shortfall { available, requested: available + 1 }), "bonded beyond the balance");
        self.pool_holds_what_is_escrowed()
    }
}
