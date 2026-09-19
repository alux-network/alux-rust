//! Measures how often one address can be handed from one server to the next, for every provider.
//!
//! What the bench measures is stated over [`BenchAlg`], so it names no harness. Which bench runs it
//! is stated by `bench_main!`, and what the cases are stated of is the example's providers.

use alux_bench::{BenchAlg, BenchExt, BenchSampling};
use alux_bench_direct::bench_main;
use alux_ext::ext;
use alux_http_conformance::Handover;
use core::time::Duration;
use http_providers::every_provider;

/// Measures how often one address is handed from one server to the next.
#[ext(name = HandoverBench)]
pub impl<This> This
where
    This: BenchAlg,
{
    /// One handover is `close` then `open` on the same address, under what is on the connection
    /// while it happens: nothing, a request answered, or one the server is still serving.
    ///
    /// The first two are CPU-bound, a bind and an accept, and are measured one at a time. The
    /// third blocks until the closed server's drain expires, so it is measured together.
    fn handover(&self) -> This::Bench {
        self.benches([
            self.one_at_a_time("no request", every_provider(Handover::NoRequest.into())),
            self.one_at_a_time("req OK", every_provider(Handover::RequestOk.into())),
            self.together("req OK 30 sec", every_provider(Handover::RequestHeld("/slow").into())),
        ])
    }
}

/// Three samples, each spending a tenth of a second on the handovers it fits.
///
/// A handover of a free address is microseconds, so a sample runs thousands; one waiting out the
/// drain is longer than the spend, so a sample runs one. Narrow a run with an argument, for example
/// `cargo bench --bench handover -- "30 sec"`.
const SAMPLING: BenchSampling = BenchSampling::new(3).spending(Duration::from_millis(100));

bench_main! {
    sampling = SAMPLING;
    bench = handover;
}
