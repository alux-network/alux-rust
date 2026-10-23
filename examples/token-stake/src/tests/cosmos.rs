use crate::tests::{ALICE, BOB, CosmosChainAlg, EscrowScenariosExt, OTHER_VALIDATOR, VALIDATOR};
use alux_ext::ext;
use alux_stake::{RewardSettlement, StakeEscrowExt, StakeForfeitExt, StakeQuorumExt, StakeRewardsExt, StakeWeightExt};
use anyhow::{Result, ensure};

/// States what a Cosmos-like chain adds: burned forfeits, jailing, and inflation.
#[ext(name = CosmosScenariosExt)]
pub(crate) impl<This> This
where
    This: CosmosChainAlg,
{
    fn scenario_forfeit_burns_and_jail_removes_weight(&mut self) -> Result<()> {
        let supply = self.supply();
        self.stake_bond(&ALICE, &VALIDATOR, 800).unwrap();
        self.stake_bond(&BOB, &OTHER_VALIDATOR, 200).unwrap();
        self.scenario_epoch_close();
        ensure!(self.stake_quorum(&0, &[VALIDATOR]), "four fifths of the weight did not decide");
        ensure!(self.stake_forfeit(&VALIDATOR, &5_000) == 400, "forfeiting half of 800 took another amount");
        self.stake_exclude(&VALIDATOR, 3);
        self.scenario_epoch_close();
        ensure!(self.supply() == supply - 400, "forfeiting did not burn the tokens it took");
        ensure!(self.stake_weights(&1) == [(OTHER_VALIDATOR, 200)], "a jailed validator carried weight");
        self.pool_holds_what_is_escrowed()
    }

    fn scenario_inflation_raises_the_supply_by_exactly_what_it_pays(&mut self) -> Result<()> {
        let supply = self.supply();
        self.stake_bond(&ALICE, &VALIDATOR, 600).unwrap();
        self.stake_bond(&BOB, &OTHER_VALIDATOR, 200).unwrap();
        self.scenario_epoch_close();
        let settlement = self.stake_reward(&0);
        ensure!(settlement == RewardSettlement { paid: 20, unpaid: 0 }, "one percent of 2000 was not paid out");
        ensure!((self.balance(&VALIDATOR), self.balance(&OTHER_VALIDATOR)) == (15, 5), "rewards ignored weight");
        ensure!(self.supply() == supply + 20, "inflation changed the supply by something else");
        Ok(())
    }
}
