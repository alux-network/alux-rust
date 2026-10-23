/// Says that an amount asked for exceeds the amount available.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shortfall<Amount> {
    /// Holds the amount available.
    pub available: Amount,
    /// Holds the amount asked for.
    pub requested: Amount,
}
