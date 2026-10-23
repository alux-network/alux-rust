use crate::{MemoryStake, Policy, Source};
use alux_stake::{
    StakeDelegationExt, StakeDelegationLawsExt, StakeEscrowExt, StakeEscrowLawsExt, StakeForfeitLawsExt,
    StakeQuorumLawsExt, StakeRewardsLawsExt, StakeWeightExt, StakeWeightLawsExt, UnbondingAlg,
};
use anyhow::{Result, ensure};

const DELEGATORS: [char; 3] = ['x', 'y', 'z'];
const VALIDATORS: [char; 3] = ['a', 'b', 'c'];
const AMOUNTS: [u64; 4] = [0, 1, 40, 1_000];

fn bonded(policy: Policy) -> MemoryStake<char> {
    let mut stake = MemoryStake::new(policy);
    for (delegator, validator, amount) in [('x', 'a', 40), ('y', 'a', 20), ('y', 'b', 30), ('z', 'c', 10)] {
        stake.wallet_fund(&delegator, 100);
        stake.stake_bond(&delegator, &validator, amount).unwrap();
    }
    stake
}

fn delegated(policy: Policy) -> MemoryStake<char> {
    let mut stake = MemoryStake::new(Policy { source: Source::Delegation, ..policy });
    for (holder, validator, amount) in [('x', 'a', 40), ('y', 'a', 20), ('z', 'b', 30)] {
        stake.wallet_fund(&holder, amount);
        stake.stake_delegate(&holder, Some(validator));
    }
    stake
}

fn escrow_is_held(stake: &MemoryStake<char>) -> Result<()> {
    let pending: u64 =
        DELEGATORS.iter().flat_map(|delegator| stake.unbonding(delegator)).map(|unbonding| unbonding.amount).sum();
    let bonded: u64 = stake.escrow_weights().iter().map(|(_, weight)| weight).sum();
    ensure!(stake.vault() == bonded + pending, "the vault holds {} for {}", stake.vault(), bonded + pending);
    Ok(())
}

#[test]
fn snapshots_satisfy_the_weight_laws_for_both_sources() -> Result<()> {
    for lag in [0, 2] {
        for mut stake in [bonded(Policy { lag, ..Policy::default() }), delegated(Policy { lag, ..Policy::default() })] {
            for epoch in 0..4 {
                stake.check_snapshot()?;
                stake.check_weights(&epoch)?;
                stake.epoch_advance();
            }
        }
    }
    Ok(())
}

#[test]
fn exclusion_satisfies_the_weight_laws() -> Result<()> {
    for validator in VALIDATORS {
        for until in 0..3 {
            let mut stake = bonded(Policy::default());
            stake.check_exclusion(&validator, until)?;
            stake.epoch_advance();
            stake.check_snapshot()?;
        }
    }
    Ok(())
}

#[test]
fn quorum_satisfies_its_laws() -> Result<()> {
    let mut stake = bonded(Policy::default());
    stake.stake_snapshot_take();
    let subsets: Vec<Vec<char>> = (0..8)
        .map(|mask| VALIDATORS.iter().enumerate().filter(|(bit, _)| mask >> bit & 1 == 1).map(|(_, v)| *v).collect())
        .collect();
    for voters in &subsets {
        stake.check_repeated_votes(&0, voters)?;
        for more in &subsets {
            stake.check_quorum_grows(&0, voters, more)?;
        }
    }
    Ok(())
}

#[test]
fn rewards_satisfy_their_laws() -> Result<()> {
    for performance in [(0, 0), (4, 10), (10, 10), (12, 10)] {
        let mut stake = bonded(Policy::default());
        stake.stake_snapshot_take();
        for validator in VALIDATORS {
            stake.blocks_observe(validator, 0, performance.0, performance.1);
        }
        stake.check_reward(&0)?;
        escrow_is_held(&stake)?;
    }
    Ok(())
}

#[test]
fn escrow_satisfies_its_laws() -> Result<()> {
    for delegator in DELEGATORS {
        for validator in VALIDATORS {
            for amount in AMOUNTS {
                let mut stake = bonded(Policy::default());
                stake.check_bond(&delegator, &validator, amount)?;
                stake.check_unbond(&delegator, &validator, amount)?;
                escrow_is_held(&stake)?;
                for _ in 0..3 {
                    stake.check_withdraw(&delegator)?;
                    stake.epoch_advance();
                }
                escrow_is_held(&stake)?;
            }
        }
    }
    Ok(())
}

#[test]
fn forfeiting_satisfies_its_laws() -> Result<()> {
    for validator in VALIDATORS {
        for basis_points in [0, 2_500, 10_000, 20_000] {
            let mut stake = bonded(Policy::default());
            stake.check_unbond(&'y', &'a', 10)?;
            stake.check_forfeit(&validator, &basis_points)?;
            escrow_is_held(&stake)?;
            stake.epoch_advance();
            stake.epoch_advance();
            stake.check_forfeit(&validator, &basis_points)?;
            escrow_is_held(&stake)?;
        }
    }
    Ok(())
}

#[test]
fn delegation_satisfies_its_laws() -> Result<()> {
    for holder in DELEGATORS {
        for validator in VALIDATORS.iter().copied().map(Some).chain([None]) {
            delegated(Policy::default()).check_delegate(&holder, validator)?;
        }
    }
    Ok(())
}
