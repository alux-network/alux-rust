use crate::RESERVE;
use crate::tests::{ALICE, BOB, CardanoChainAlg, OTHER_VALIDATOR, VALIDATOR};
use alux_ext::ext;
use alux_stake::{RewardSettlement, StakeDelegationExt, StakeRewardsExt, StakeWeightExt};
use alux_token::TokenLedgerExt;
use anyhow::{Result, ensure};

/// States what a Cardano-like chain adds: delegation that moves nothing, lagged weights, and
/// rewards from a reserve that are withheld, never forfeited.
#[ext(name = CardanoScenariosExt)]
pub(crate) impl<This> This
where
    This: CardanoChainAlg,
{
    fn scenario_delegation_moves_nothing_and_weight_lags_two_epochs(&mut self) -> Result<()> {
        self.stake_delegate(&ALICE, Some(VALIDATOR));
        ensure!(self.balance(&ALICE) == 1_000, "delegating moved tokens");
        self.scenario_epoch_close();
        self.stake_delegate(&BOB, Some(VALIDATOR));
        self.token_transfer(&ALICE, &BOB, 300).unwrap();
        self.scenario_epoch_close();
        self.scenario_epoch_close();
        ensure!(self.stake_weight(&VALIDATOR, &2) == 1_000, "epoch 2 did not use the snapshot of epoch 0");
        ensure!(self.stake_weight(&VALIDATOR, &3) == 2_000, "epoch 3 did not use the snapshot of epoch 1");
        Ok(())
    }

    fn scenario_withholds_rewards_from_weak_pools_and_keeps_the_supply(&mut self) -> Result<()> {
        self.token_mint(&RESERVE, 100_000);
        let supply = self.supply();
        self.stake_delegate(&ALICE, Some(VALIDATOR));
        self.stake_delegate(&BOB, Some(OTHER_VALIDATOR));
        (0..3).for_each(|_| self.scenario_epoch_close());
        self.scenario_observe(VALIDATOR, 2, 10, 10);
        self.scenario_observe(OTHER_VALIDATOR, 2, 4, 10);
        let settlement = self.stake_reward(&2);
        ensure!(settlement == RewardSettlement { paid: 210, unpaid: 90 }, "the pot was settled as {settlement:?}");
        ensure!(self.balance(&RESERVE) == 100_000 - 210, "what nobody earned left the reserve");
        ensure!((self.balance(&ALICE), self.balance(&BOB)) == (1_000, 1_000), "withholding touched principal");
        ensure!(self.supply() == supply, "rewards changed the supply");
        Ok(())
    }
}
