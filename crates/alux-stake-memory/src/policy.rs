/// Selects where weight comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    /// Weighs what is bonded.
    #[default]
    Escrow,
    /// Weighs the wallets of holders delegating to each validator.
    Delegation,
}

/// Holds the policies the specification leaves to an interpretation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    /// Selects where weight comes from.
    pub source: Source,
    /// Counts how many epochs before an epoch its governing snapshot was taken.
    pub lag: u64,
    /// Counts how many epochs unbonding stake takes to mature.
    pub unbonding_delay: u64,
    /// Holds the fraction of the total a weight must exceed to decide, as numerator and denominator.
    pub quorum: (u64, u64),
    /// Holds the pot each epoch emits.
    pub emission: u64,
}

impl Default for Policy {
    fn default() -> Self {
        Self { source: Source::Escrow, lag: 0, unbonding_delay: 2, quorum: (2, 3), emission: 1_000 }
    }
}
