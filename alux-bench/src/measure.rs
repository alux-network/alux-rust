//! States the stating and the measuring of a benchmark, which are separate.

use crate::{BenchCases, BenchGroup, BenchRounds, BenchRoutine, BenchSentRoutine, BenchStated};
use alux_ext::ext;
use core::time::Duration;

/// Interprets the stating of a benchmark.
///
/// What a bench denotes is the interpretation's: one states the groups as a value to fold later,
/// another states them straight into whatever its harness holds. Nothing is measured by stating
/// it, so what an interpretation borrows, and when, is never the statement's business.
pub trait BenchAlg {
    /// Carries a stated bench.
    type Bench;

    /// States a bench of no groups.
    fn nothing(&self) -> Self::Bench;

    /// States one group of cases, named `group`.
    fn group(&self, group: &'static str, cases: BenchCases<'static>) -> Self::Bench;

    /// States the groups of `next` after the groups of `first`.
    fn then(&self, first: Self::Bench, next: Self::Bench) -> Self::Bench;
}

/// Interprets the measuring of a stated bench.
///
/// Reading the bench is the interpretation's own loop, so a harness holding something mutable
/// holds it here rather than while the bench is being stated.
pub trait MeasureBenchAlg {
    /// Carries the bench this measures.
    type Bench;

    /// Measures every case of `bench`, in the order the bench states them.
    fn measure(&mut self, bench: Self::Bench);
}

/// Derives the ways a bench states a group.
#[ext(name = BenchExt)]
pub impl<This> This
where
    This: BenchAlg,
{
    /// States one group whose cases are measured one at a time.
    ///
    /// A routine stating `Send` is taken as well, since a case that may be measured beside others
    /// may also be measured alone.
    fn one_at_a_time<Stated, Routine>(&self, group: &'static str, cases: Stated) -> This::Bench
    where
        Stated: IntoIterator<Item = (&'static str, Routine)>,
        Routine: FnMut(BenchRounds) -> Duration + 'static,
    {
        let cases = cases.into_iter().map(|(subject, routine)| (subject, Box::new(routine) as BenchRoutine<'static>));

        self.group(group, BenchCases::OneAtATime(cases.collect()))
    }

    /// States one group whose cases are measured at the same time.
    fn together<Stated, Routine>(&self, group: &'static str, cases: Stated) -> This::Bench
    where
        Stated: IntoIterator<Item = (&'static str, Routine)>,
        Routine: FnMut(BenchRounds) -> Duration + Send + 'static,
    {
        let cases = cases.into_iter().map(|(subject, routine)| (subject, Box::new(routine) as BenchSentRoutine));

        self.group(group, BenchCases::Together(cases.collect()))
    }

    /// States every group of `stated`, in the order stated.
    fn benches<Stated>(&self, stated: Stated) -> This::Bench
    where
        Stated: IntoIterator<Item = This::Bench>,
    {
        stated.into_iter().fold(self.nothing(), |bench, next| self.then(bench, next))
    }
}

/// States a bench as the value it is, which is what an interpretation folds.
#[derive(Clone, Copy, Debug, Default)]
pub struct StatingBench;

impl BenchAlg for StatingBench {
    type Bench = BenchStated<'static>;

    fn nothing(&self) -> Self::Bench {
        BenchStated::new()
    }

    fn group(&self, group: &'static str, cases: BenchCases<'static>) -> Self::Bench {
        BenchStated::new().group(BenchGroup::new(group, cases))
    }

    fn then(&self, first: Self::Bench, next: Self::Bench) -> Self::Bench {
        first.then(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchRounds;
    use core::time::Duration;

    /// One case, measured by a routine that answers what it was asked for.
    fn case(subject: &'static str) -> (&'static str, BenchRoutine<'static>) {
        (subject, Box::new(|rounds: BenchRounds| Duration::from_millis(rounds.count())))
    }

    #[test]
    fn states_groups_in_the_order_stated() {
        let stating = StatingBench;

        let bench = stating.benches([
            stating.one_at_a_time("close", [case("hyper"), case("axum")]),
            stating.together("end", [("poem", Box::new(|_| Duration::from_secs(1)) as BenchSentRoutine)]),
        ]);

        let groups = bench.groups().map(|group| (group.group(), group.stated().len())).collect::<Vec<_>>();
        assert_eq!(groups, [("close", 2), ("end", 1)]);
    }

    #[test]
    fn states_what_measuring_a_group_at_once_would_cost() {
        let stating = StatingBench;

        let bench = stating.one_at_a_time("close", [case("hyper")]);

        let stated = bench.groups().next().expect("the group stated");
        assert!(matches!(stated.stated(), BenchCases::OneAtATime(_)));
        assert_eq!(stated.stated().subjects().collect::<Vec<_>>(), ["hyper"]);
    }
}
