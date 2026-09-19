//! Names what is measured and how much of it one sample runs.

use core::ops::Range;

/// Names one measured case: the group it is read within, and the subject it measures.
///
/// A group holds the cases a run compares, such as one operation under one load. A subject is what
/// is measured under it, such as one provider.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BenchCase {
    group: &'static str,
    subject: &'static str,
}

impl BenchCase {
    /// States that `subject` is measured within `group`.
    pub const fn new(group: &'static str, subject: &'static str) -> Self {
        Self { group, subject }
    }

    /// Returns the group the case is read within.
    pub const fn group(self) -> &'static str {
        self.group
    }

    /// Returns the subject the case measures.
    pub const fn subject(self) -> &'static str {
        self.subject
    }
}

/// How many rounds one sample runs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BenchRounds(u64);

impl BenchRounds {
    /// States a sample of `count` rounds.
    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    /// Returns how many rounds the sample runs.
    pub const fn count(self) -> u64 {
        self.0
    }
}

impl IntoIterator for BenchRounds {
    type IntoIter = Range<u64>;
    type Item = u64;

    /// Iterates the rounds, so a routine reads `for _ in rounds`.
    fn into_iter(self) -> Self::IntoIter {
        0..self.0
    }
}

impl From<u64> for BenchRounds {
    fn from(count: u64) -> Self {
        Self::new(count)
    }
}
