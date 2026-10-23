# alux-stake

States what proof of stake means in a form general enough for its variants. The core is one function: a validator has a weight in an epoch, recorded when a snapshot was taken. Everything that differs between chains is a separate algebra around that core: where weight comes from, which snapshot governs an epoch, who is excluded, how rewards are emitted and shared, and whether proven offenses forfeit stake.

The crate is a specification. It depends only on [`alux-ext`](https://docs.rs/alux-ext) and names no token, consensus protocol, or runtime. Consensus enters only as primitive observations: [`PerformanceAlg`] says how well a validator did its duties, and the offense a [`ForfeitAlg`] prices is evidence consensus proved. Leader selection and finality rules belong to a consensus specification that consumes epoch weights.

## Core

| Algebra | Meaning |
| --- | --- |
| [`EpochAlg`] | the current epoch |
| [`WeightSourceAlg`] | how much stake stands behind each validator right now |
| [`SnapshotAlg`] | the weights recorded when each epoch closed |
| [`SnapshotLagAlg`] | which snapshot governs weight in an epoch |
| [`ExclusionAlg`] | validators kept out of weight, and until when |

[`StakeWeightExt`] derives taking a snapshot, excluding a validator, and reading weights and their total in an epoch. [`StakeQuorumExt`] adds a [`QuorumAlg`] threshold for chains with quorum finality.

## Rewards

| Algebra | Meaning |
| --- | --- |
| [`PerformanceAlg`] | how well a validator did its duties, as consensus observed |
| [`EmissionAlg`] | the epoch's reward pot |
| [`RewardShareAlg`] | what a validator earned from the pot |
| [`RewardPayoutAlg`] | paying rewards with value |

[`StakeRewardsExt`] settles an epoch: paid and unpaid always sum to the pot. Withholding rewards for weak performance is a share policy, not a separate penalty, and it never touches principal.

## Weight sources

| Source | Algebras | Extension |
| --- | --- | --- |
| Escrow: tokens are locked, leave after a delay, and may be forfeited | [`BondsAlg`], [`UnbondingAlg`], [`UnbondingDelayAlg`], [`CustodyAlg`], [`ForfeitAlg`] | [`StakeEscrowExt`], [`StakeForfeitExt`] |
| Delegation: holders point their holdings at a validator and nothing moves | [`DelegationAlg`], [`HoldingsAlg`] | [`StakeDelegationExt`] |

An interpretation answers [`WeightSourceAlg`] by delegating to `escrow_weights` or `delegated_weights`.

## Variants

A variant is a choice of primitive interpretations, not a different specification:

| Variant | Weight source | Snapshot lag | Rewards | Misbehavior costs |
| --- | --- | --- | --- | --- |
| Cosmos-like | escrow | none | inflation, by weight | forfeit and exclusion (jailing) |
| Ethereum-like | escrow | per epoch | per duty performed | forfeit, exclusion (ejection), and slow forfeit for inactivity |
| Cardano-like | delegation | two epochs | reserve, by weight and performance | withheld rewards only |

TODO: add a queue between the weight source and the snapshot that limits how much weight enters and leaves per epoch, so an interpretation can state Ethereum's activation and exit churn limit. Variants without a limit answer with an unbounded queue.

TODO: record recent forfeits as a primitive observation, so a forfeit policy can grow with how much other stake was forfeited in the same window, as Ethereum's correlation penalty does.

## Laws

Each extension has a law extension that every interpretation runs over the values it chooses: [`StakeWeightLawsExt`], [`StakeQuorumLawsExt`], [`StakeRewardsLawsExt`], [`StakeEscrowLawsExt`], [`StakeForfeitLawsExt`], and [`StakeDelegationLawsExt`]. A snapshot records exactly the source's weights of validators not excluded; repeated votes do not count and more voters never lose a quorum; paid and unpaid rewards sum to the pot and leave weights unchanged; escrow moves exactly the amounts asked; forfeiting takes exactly what it reports and only from the offender; delegating moves no value. [`alux-stake-memory`](https://docs.rs/alux-stake-memory) is the reference interpretation that runs them.

A consensus specification consumes stake through its bounds alone. Accepting a block certificate is an extension over epoch weights and quorum:

```rust
use alux_ext::ext;
use alux_stake::{
    EpochAlg, ExclusionAlg, QuorumAlg, SnapshotAlg, SnapshotLagAlg, StakeQuorumExt, WeightSourceAlg,
};
use std::ops::Add;

/// Derives certificate acceptance from stake-weighted quorum.
#[ext(name = CertificateExt)]
pub impl<This, Validator, Amount, Epoch> This
where
    This: EpochAlg<Epoch = Epoch>
        + WeightSourceAlg<Validator = Validator, Amount = Amount>
        + SnapshotAlg<Validator = Validator, Amount = Amount, Epoch = Epoch>
        + SnapshotLagAlg<Epoch = Epoch>
        + ExclusionAlg<Validator = Validator, Epoch = Epoch>
        + QuorumAlg<Amount = Amount>,
    Validator: PartialEq,
    Amount: Copy + Ord + Default + Add<Output = Amount>,
    Epoch: Clone + Ord,
{
    /// Accepts a block when its signers reach quorum in its epoch.
    fn certificate_accepts<Block>(&self, block: Block, epoch: &Epoch, signers: &[Validator]) -> Option<Block> {
        self.stake_quorum(epoch, signers).then_some(block)
    }
}
```
