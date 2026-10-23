use alux_ext::ext;
use std::ops::Add;

/// Tells the current epoch.
pub trait EpochAlg {
    /// Orders time in epochs.
    type Epoch;

    /// Returns the current epoch.
    fn epoch(&self) -> Self::Epoch;
}

/// Tells how much stake stands behind each validator right now.
///
/// This is where variants differ. Escrowed stake answers with what is bonded; delegated stake answers
/// with the holdings of whoever points at the validator. The answer is not yet weight: weight is
/// what an epoch's snapshot recorded.
pub trait WeightSourceAlg {
    /// Identifies a validator.
    type Validator;
    /// Measures a quantity of stake.
    type Amount;

    /// Returns every validator with stake behind it, and how much.
    fn source_weights(&self) -> Vec<(Self::Validator, Self::Amount)>;
}

/// Keeps the weight distribution recorded when each epoch closed.
///
/// An epoch never recorded has no weight.
pub trait SnapshotAlg {
    /// Identifies a validator.
    type Validator;
    /// Measures a quantity of stake.
    type Amount;
    /// Orders time in epochs.
    type Epoch;

    /// Returns the weights recorded for the epoch.
    fn snapshot(&self, epoch: &Self::Epoch) -> Vec<(Self::Validator, Self::Amount)>;

    /// Records the weights for the epoch, replacing any earlier record.
    fn snapshot_record(&mut self, epoch: Self::Epoch, weights: Vec<(Self::Validator, Self::Amount)>);
}

/// Tells which snapshot governs weight in an epoch.
///
/// A chain that uses live stake answers with the epoch itself. One that lets stake settle first
/// answers with an earlier epoch; Cardano uses the snapshot taken two epochs before.
pub trait SnapshotLagAlg {
    /// Orders time in epochs.
    type Epoch;

    /// Returns the epoch whose snapshot governs weight in the given epoch.
    fn snapshot_governing(&self, epoch: &Self::Epoch) -> Self::Epoch;
}

/// Keeps which validators are kept out of the weight distribution, and until when.
///
/// Exclusion removes weight without touching value: jailing, ejection, or retirement.
pub trait ExclusionAlg {
    /// Identifies a validator.
    type Validator;
    /// Orders time in epochs.
    type Epoch;

    /// Returns the first epoch the validator may carry weight again, if it is excluded.
    fn excluded_until(&self, validator: &Self::Validator) -> Option<Self::Epoch>;

    /// Replaces the validator's exclusion.
    fn exclusion_set(&mut self, validator: &Self::Validator, until: Option<Self::Epoch>);
}

/// Derives epoch weights from a weight source, snapshots, their lag, and exclusion.
///
/// This is the core every variant shares: a validator has a weight in an epoch, and the weights of
/// an epoch sum to its total.
#[ext(name = StakeWeightExt)]
pub impl<This, Validator, Amount, Epoch> This
where
    This: EpochAlg<Epoch = Epoch>
        + WeightSourceAlg<Validator = Validator, Amount = Amount>
        + SnapshotAlg<Validator = Validator, Amount = Amount, Epoch = Epoch>
        + SnapshotLagAlg<Epoch = Epoch>
        + ExclusionAlg<Validator = Validator, Epoch = Epoch>,
    Validator: PartialEq,
    Amount: Copy + Ord + Default + Add<Output = Amount>,
    Epoch: Clone + Ord,
{
    /// Records the current epoch's snapshot: every validator with stake that is not excluded in it.
    fn stake_snapshot_take(&mut self) {
        let epoch = self.epoch();
        let weights = self
            .source_weights()
            .into_iter()
            .filter(|(validator, weight)| {
                *weight > Amount::default() && self.excluded_until(validator).is_none_or(|until| until <= epoch)
            })
            .collect();
        self.snapshot_record(epoch, weights);
    }

    /// Excludes a validator until an epoch; snapshots taken before then leave it out.
    fn stake_exclude(&mut self, validator: &Validator, until: Epoch) {
        self.exclusion_set(validator, Some(until));
    }

    /// Returns every validator's weight in the epoch, most weighty first.
    fn stake_weights(&self, epoch: &Epoch) -> Vec<(Validator, Amount)> {
        let mut weights = self.snapshot(&self.snapshot_governing(epoch));
        weights.sort_by(|(_, left), (_, right)| right.cmp(left));
        weights
    }

    /// Returns the validator's weight in the epoch.
    fn stake_weight(&self, validator: &Validator, epoch: &Epoch) -> Amount {
        self.stake_weights(epoch)
            .into_iter()
            .find(|(kept, _)| kept == validator)
            .map_or_else(Amount::default, |(_, weight)| weight)
    }

    /// Returns the sum of every weight in the epoch.
    fn stake_total(&self, epoch: &Epoch) -> Amount {
        self.stake_weights(epoch).into_iter().map(|(_, weight)| weight).fold(Amount::default(), Add::add)
    }
}
