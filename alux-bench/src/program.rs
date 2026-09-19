//! States what a benchmark measures, as a value that names no harness.
//!
//! A bench states groups; a group states cases; a case states a subject and the routine measuring
//! it. Nothing here runs: what a stated bench denotes is read by whoever measures it, in whatever
//! order and on whatever threads that interpretation wants.

use crate::BenchRounds;
use core::time::Duration;

/// What one case runs: the rounds of one sample, answering how long they took.
pub type BenchRoutine<'routine> = Box<dyn FnMut(BenchRounds) -> Duration + 'routine>;

/// What one case runs, when it may be measured beside the cases stated with it.
pub type BenchSentRoutine = Box<dyn FnMut(BenchRounds) -> Duration + Send>;

/// The cases of one group, and what measuring them at once would cost.
pub enum BenchCases<'routine> {
    /// Measured one at a time, which is what a CPU-bound round needs: rounds running beside each
    /// other contend for cores, and what is measured is the contention.
    OneAtATime(Vec<(&'static str, BenchRoutine<'routine>)>),
    /// Measured at the same time, which an IO-bound round allows: time spent blocked costs the
    /// same blocked beside another round, so the group takes what its longest case takes.
    Together(Vec<(&'static str, BenchSentRoutine)>),
}

impl BenchCases<'_> {
    /// Answers how many cases the group states.
    pub fn len(&self) -> usize {
        match self {
            Self::OneAtATime(cases) => cases.len(),
            Self::Together(cases) => cases.len(),
        }
    }

    /// Answers whether the group states no case.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads the subject of every case, in the order stated.
    pub fn subjects(&self) -> impl Iterator<Item = &'static str> + '_ {
        match self {
            Self::OneAtATime(cases) => Subjects::OneAtATime(cases.iter()),
            Self::Together(cases) => Subjects::Together(cases.iter()),
        }
    }
}

/// Reads the subjects of either kind of group.
enum Subjects<'cases, 'routine> {
    OneAtATime(core::slice::Iter<'cases, (&'static str, BenchRoutine<'routine>)>),
    Together(core::slice::Iter<'cases, (&'static str, BenchSentRoutine)>),
}

impl Iterator for Subjects<'_, '_> {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::OneAtATime(cases) => cases.next().map(|(subject, _)| *subject),
            Self::Together(cases) => cases.next().map(|(subject, _)| *subject),
        }
    }
}

/// One group of a stated bench: what the cases are compared within, and the cases.
pub struct BenchGroup<'routine> {
    group: &'static str,
    cases: BenchCases<'routine>,
}

impl<'routine> BenchGroup<'routine> {
    /// States `cases` as one group, named `group`.
    pub const fn new(group: &'static str, cases: BenchCases<'routine>) -> Self {
        Self { group, cases }
    }

    /// Returns what the group is named.
    pub const fn group(&self) -> &'static str {
        self.group
    }

    /// Returns the cases the group states, and how they are measured.
    pub fn cases(self) -> BenchCases<'routine> {
        self.cases
    }

    /// Reads the cases the group states without taking them.
    pub const fn stated(&self) -> &BenchCases<'routine> {
        &self.cases
    }
}

/// A stated bench: its groups, in the order stated.
#[derive(Default)]
pub struct BenchStated<'routine> {
    groups: Vec<BenchGroup<'routine>>,
}

impl<'routine> BenchStated<'routine> {
    /// States a bench of no groups.
    pub const fn new() -> Self {
        Self { groups: Vec::new() }
    }

    /// States one group after the groups already stated.
    #[must_use]
    pub fn group(mut self, group: BenchGroup<'routine>) -> Self {
        self.groups.push(group);

        self
    }

    /// States the groups of `next` after the groups of this bench.
    #[must_use]
    pub fn then(mut self, next: Self) -> Self {
        self.groups.extend(next.groups);

        self
    }

    /// Reads the groups the bench states, in the order stated.
    pub fn groups(&self) -> impl Iterator<Item = &BenchGroup<'routine>> {
        self.groups.iter()
    }

    /// Separates the groups from the bench that states them.
    pub fn into_groups(self) -> impl Iterator<Item = BenchGroup<'routine>> {
        self.groups.into_iter()
    }
}
