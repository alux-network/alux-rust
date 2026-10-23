use crate::{EpochAlg, ExclusionAlg, SnapshotAlg, SnapshotLagAlg, StakeWeightExt, WeightSourceAlg};
use alux_ext::ext;
use std::ops::Add;

/// Decides whether a weight out of a total is enough to decide.
///
/// Byzantine fault tolerant finality usually asks for more than two thirds. A chain with
/// probabilistic finality has no quorum and does not answer this.
pub trait QuorumAlg {
    /// Measures a quantity of stake.
    type Amount;

    /// Says whether the weight decides, out of the total.
    fn quorum_reached(&self, weight: &Self::Amount, total: &Self::Amount) -> bool;
}

/// Derives the quorum test from epoch weights and a threshold policy.
#[ext(name = StakeQuorumExt)]
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
    /// Says whether the voters' combined weight in the epoch decides; a repeated voter counts once.
    fn stake_quorum(&self, epoch: &Epoch, voters: &[Validator]) -> bool {
        let weight = voters
            .iter()
            .enumerate()
            .filter(|(position, voter)| !voters[..*position].contains(voter))
            .map(|(_, voter)| self.stake_weight(voter, epoch))
            .fold(Amount::default(), Add::add);
        self.quorum_reached(&weight, &self.stake_total(epoch))
    }
}
