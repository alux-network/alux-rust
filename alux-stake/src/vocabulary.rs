/// Holds escrowed stake leaving a validator: an amount that becomes withdrawable at an epoch.
///
/// Until it matures, it still answers for the validator's offenses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Unbonding<Validator, Amount, Epoch> {
    /// Identifies the validator the stake was bonded to.
    pub validator: Validator,
    /// Holds the amount leaving.
    pub amount: Amount,
    /// Holds the epoch from which the amount may be withdrawn.
    pub matures: Epoch,
}

/// Says that a delegator asked to unbond more than it has bonded to a validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BondShortfall<Amount> {
    /// Holds the amount bonded.
    pub bonded: Amount,
    /// Holds the amount asked for.
    pub requested: Amount,
}

/// Holds how an epoch's reward pot was settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RewardSettlement<Amount> {
    /// Holds what was paid to validators.
    pub paid: Amount,
    /// Holds what was earned by nobody: the share withheld for weak performance, plus any remainder.
    pub unpaid: Amount,
}
