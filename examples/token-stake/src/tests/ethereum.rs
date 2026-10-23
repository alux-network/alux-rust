use crate::tests::{ALICE, BOB, EscrowScenariosExt, EthereumChainAlg, OTHER_VALIDATOR, VALIDATOR};
use crate::{EthereumOffense, MAX_EFFECTIVE_BALANCE, STAKE_POOL};
use alux_ext::ext;
use alux_stake::{RewardSettlement, StakeEscrowExt, StakeForfeitExt, StakeRewardsExt, StakeWeightExt};
use alux_token::TokenLedgerExt;
use anyhow::{Result, ensure};

/// States what an Ethereum-like chain adds: the effective balance cap, equivocation and inactivity
/// forfeits, ejection, and rewards per duty.
#[ext(name = EthereumScenariosExt)]
pub(crate) impl<This> This
where
    This: EthereumChainAlg,
{
    fn scenario_weight_is_capped_at_the_effective_balance(&mut self) -> Result<()> {
        self.token_mint(&ALICE, 2_000);
        self.stake_bond(&ALICE, &VALIDATOR, 2_500).unwrap();
        self.stake_bond(&BOB, &OTHER_VALIDATOR, 500).unwrap();
        self.scenario_epoch_close();
        self.scenario_epoch_close();
        let expected = [(VALIDATOR, MAX_EFFECTIVE_BALANCE), (OTHER_VALIDATOR, 500)];
        ensure!(self.stake_weights(&1) == expected, "weight was not capped");
        ensure!(self.balance(&STAKE_POOL) == 3_000, "the cap changed what is escrowed");
        Ok(())
    }

    fn scenario_equivocation_forfeits_and_ejects(&mut self) -> Result<()> {
        let supply = self.supply();
        self.stake_bond(&ALICE, &VALIDATOR, 640).unwrap();
        self.stake_bond(&BOB, &OTHER_VALIDATOR, 640).unwrap();
        let forfeited = self.stake_forfeit(&VALIDATOR, &EthereumOffense::Equivocation);
        ensure!(forfeited == 20, "equivocation took {forfeited}, not a thirty-second");
        self.stake_exclude(&VALIDATOR, u64::MAX);
        (0..3).for_each(|_| self.scenario_epoch_close());
        ensure!(self.stake_weights(&2) == [(OTHER_VALIDATOR, 640)], "an ejected validator carried weight");
        ensure!(self.supply() == supply - 20, "forfeiting did not burn the tokens it took");
        self.pool_holds_what_is_escrowed()
    }

    fn scenario_inactivity_leak_grows_with_the_square_of_the_epochs(&mut self) -> Result<()> {
        self.token_mint(&ALICE, 3_000);
        let mut leaks = Vec::new();
        for (validator, epochs) in [(20, 16), (21, 32), (22, 64), (23, 128)] {
            self.stake_bond(&ALICE, &validator, 1_000).unwrap();
            leaks.push(self.stake_forfeit(&validator, &EthereumOffense::Inactivity { epochs }));
        }
        ensure!(leaks == [62, 250, 1_000, 1_000], "the leak from 1000 is not quadratic: {leaks:?}");
        self.pool_holds_what_is_escrowed()
    }

    fn scenario_rewards_pay_duties_performed_and_mint_only_what_is_paid(&mut self) -> Result<()> {
        let supply = self.supply();
        self.stake_bond(&ALICE, &VALIDATOR, 800).unwrap();
        self.stake_bond(&BOB, &OTHER_VALIDATOR, 800).unwrap();
        self.scenario_epoch_close();
        self.scenario_epoch_close();
        self.scenario_observe(VALIDATOR, 1, 32, 32);
        self.scenario_observe(OTHER_VALIDATOR, 1, 16, 32);
        let settlement = self.stake_reward(&1);
        ensure!(settlement == RewardSettlement { paid: 120, unpaid: 40 }, "the pot was settled as {settlement:?}");
        ensure!(self.supply() == supply + 120, "issuance minted more than it paid");
        Ok(())
    }
}
