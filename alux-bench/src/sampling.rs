//! States how thoroughly a suite is sampled.

use core::time::Duration;

/// How many samples one case is measured over, and how long one sample may spend on rounds.
///
/// A sample is one call of a case's routine, running the rounds that fit [`Self::spend`], or the
/// fewest the interpreter can run where no spend is stated.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BenchSampling {
    samples: usize,
    spend: Option<Duration>,
}

impl BenchSampling {
    /// States that each case is measured over `samples` samples of the fewest rounds.
    pub const fn new(samples: usize) -> Self {
        Self { samples, spend: None }
    }

    /// States that one sample runs the rounds it can run in about `spend`.
    #[must_use]
    pub const fn spending(mut self, spend: Duration) -> Self {
        self.spend = Some(spend);

        self
    }

    /// Returns how many samples each case is measured over.
    pub const fn samples(self) -> usize {
        self.samples
    }

    /// Returns how long one sample may spend on rounds, or `None` for the fewest rounds.
    pub const fn spend(self) -> Option<Duration> {
        self.spend
    }
}

impl From<usize> for BenchSampling {
    fn from(samples: usize) -> Self {
        Self::new(samples)
    }
}
