//! Measures how many provider switches the example answers a second.
//!
//! What the bench measures is stated over [`BenchAlg`], so it names no harness. Criterion is the
//! harness here: one round is one `POST /fw` answered, so a sample runs the many that fit, and
//! picking the rounds, discarding outliers and comparing against the previous run are the point.

use alux_bench::{BenchAlg, BenchExt, BenchSampling};
use alux_bench_criterion::CriterionBench;
use alux_ext::ext;
use core::time::Duration;
use criterion::{Criterion, criterion_group, criterion_main};
use http_providers::Switching;

/// Measures the switches the example's front door answers.
#[ext(name = SwitchingBench)]
pub impl<This> This
where
    This: BenchAlg,
{
    /// Eight clients send `POST /fw` at once, each naming the next provider in turn.
    ///
    /// Starting the example is setup, so it happens off the clock and stops when the bench is done
    /// with it.
    fn switching(&self) -> This::Bench {
        let serving = Switching::started();

        self.one_at_a_time("provider switches", [("8 clients", move |rounds| serving.switches(rounds))])
    }
}

/// Ten samples, each spending fifteen seconds on the switches it fits.
const SAMPLING: BenchSampling = BenchSampling::new(10).spending(Duration::from_secs(15));

fn switches(criterion: &mut Criterion) {
    CriterionBench::measuring(criterion, |bench| bench.switching());
}

criterion_group! {
    name = benches;
    config = CriterionBench::sampled(SAMPLING);
    targets = switches
}
criterion_main!(benches);
