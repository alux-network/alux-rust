//! Measures `close` and `end` against a request in flight, for every provider.
//!
//! What the bench measures is stated over [`BenchAlg`], so it names no harness. Which bench runs it
//! is stated by `bench_main!`, and what the cases are stated of is the example's providers.

use alux_bench::{BenchAlg, BenchExt, BenchSampling};
use alux_bench_direct::bench_main;
use alux_ext::ext;
use alux_http_conformance::Closing;
use core::time::Duration;
use http_providers::{Measures, closed, composable, every_provider};

/// Measures `close` and `end` against one request in flight.
#[ext(name = ClosingBench)]
pub impl<This> This
where
    This: BenchAlg,
{
    /// `close` returns once the address is released, `end` once what it is still serving has
    /// finished. `/pause` answers 1s after the close begins, inside the drain; `/slow` answers 30s
    /// after, past it.
    ///
    /// A composable `close` is CPU-bound: it drops the listener and returns in microseconds, so
    /// those cases are measured one at a time. Every other case blocks until a drain expires, so
    /// they are measured together.
    fn closing(&self) -> This::Bench {
        let answered = Measures::from(Closing::new("/pause", false));
        let ended = Measures::from(Closing::new("/slow", false));

        self.benches([
            self.one_at_a_time("close, req OK 1 sec", composable(answered)),
            self.one_at_a_time("close, req OK 30 sec", composable(ended)),
            self.together("close, req OK 1 sec", closed(answered)),
            self.together("close, req OK 30 sec", closed(ended)),
            self.together("end, req OK 1 sec", every_provider(ending("/pause"))),
            self.together("end, req OK 30 sec", every_provider(ending("/slow"))),
        ])
    }
}

/// States closing a server holding `asking_for` and ending it too.
fn ending(asking_for: &'static str) -> Measures {
    Closing::new(asking_for, true).into()
}

/// Three samples, each spending a second on the closes it fits.
///
/// A round costs what it sets up: opening a server and waiting [`alux_http_conformance::SETTLE`]
/// for it to accept the held request, which is about a third of a second whatever the close costs.
/// So a composable close, measured in microseconds, runs a few rounds per sample and is averaged
/// over them, while one waiting out a drain is longer than the spend and runs once. Narrow a run
/// with an argument, for example `cargo bench --bench closing -- "30 sec"`.
const SAMPLING: BenchSampling = BenchSampling::new(3).spending(Duration::from_secs(1));

bench_main! {
    sampling = SAMPLING;
    bench = closing;
}
