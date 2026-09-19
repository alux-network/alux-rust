//! Interprets the stating and measurement of named cases with criterion.

use alux_bench::{BenchAlg, BenchCases, BenchGroup, BenchRounds, BenchSampling, BenchStated, MeasureBenchAlg};
use core::time::Duration;
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion};
use derive_new::new as New;

/// The fewest samples criterion takes. `Criterion::sample_size` asserts at least this many.
const SAMPLE_FLOOR: usize = 10;

/// What a sample spends when none is stated. Criterion picks rounds per sample by dividing this by
/// the round it timed during warm-up, so a millisecond is one round of anything slower.
const LEAST: Duration = Duration::from_millis(1);

/// Measures cases with the criterion a benchmark target is given.
#[derive(New)]
pub struct CriterionBench<'bench> {
    bench: &'bench mut Criterion,
}

impl CriterionBench<'_> {
    /// Builds the criterion a `criterion_group!` takes as its `config`, sampled as stated.
    ///
    /// A sample's spend is criterion's measurement time. Samples below criterion's floor of ten are
    /// raised to ten.
    pub fn sampled(sampling: BenchSampling) -> Criterion {
        let spend = sampling.spend().unwrap_or(LEAST);

        Criterion::default()
            .sample_size(sampling.samples().max(SAMPLE_FLOOR))
            .warm_up_time(spend)
            .measurement_time(spend)
            .without_plots()
    }

    /// Measures what `stating` states, with the criterion a benchmark target is given.
    ///
    /// ```rust ignore
    /// fn switches(criterion: &mut Criterion) {
    ///     CriterionBench::measuring(criterion, |bench| bench.switching());
    /// }
    /// ```
    pub fn measuring<Stating>(criterion: &mut Criterion, stating: Stating)
    where
        Stating: FnOnce(&CriterionBench<'_>) -> BenchStated<'static>,
    {
        let mut bench = CriterionBench::new(criterion);
        let stated = stating(&bench);

        bench.measure(stated);
    }
}

impl BenchAlg for CriterionBench<'_> {
    type Bench = BenchStated<'static>;

    /// States the bench as the value it is, which measuring folds into criterion's own groups.
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

impl MeasureBenchAlg for CriterionBench<'_> {
    type Bench = BenchStated<'static>;

    /// Measures every group as criterion's own `benchmark_group`, in the order stated.
    ///
    /// Criterion decides when a case runs, so a group stated to be measured at the same time is
    /// measured one case at a time here, like any other.
    fn measure(&mut self, bench: Self::Bench) {
        for stated in bench.into_groups() {
            let mut group = self.bench.benchmark_group(stated.group());
            match stated.cases() {
                BenchCases::OneAtATime(cases) => {
                    for (subject, routine) in cases {
                        case(&mut group, subject, routine);
                    }
                }
                BenchCases::Together(cases) => {
                    for (subject, routine) in cases {
                        case(&mut group, subject, routine);
                    }
                }
            }
        }
    }
}

/// States one case as a `bench_function` named by its subject.
///
/// `iter_custom` hands the routine the rounds of one sample and reads back the time it answers.
fn case<Routine>(group: &mut BenchmarkGroup<'_, WallTime>, subject: &'static str, mut routine: Routine)
where
    Routine: FnMut(BenchRounds) -> Duration,
{
    group.bench_function(subject, |sampling| {
        sampling.iter_custom(|count| routine(BenchRounds::new(count)));
    });
}
