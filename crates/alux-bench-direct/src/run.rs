//! States the whole of a benchmark's `main`.

/// States `main` for a benchmark: state these benches, then measure them.
///
/// `sampling` is the [`crate::DirectBench`] sampling, and each `bench` is a method stating one: an
/// extension over [`alux_bench::BenchAlg`], written wherever the benchmark is authored. Stating
/// happens first and measuring after, so what a bench states is a value either interpretation can
/// measure. The filter comes from [`crate::DirectBench::from_args`].
///
/// ```rust ignore
/// bench_main! {
///     sampling = BenchSampling::new(3);
///     bench = closing;
///     bench = handover;
/// }
/// ```
#[macro_export]
macro_rules! bench_main {
    (
        sampling = $sampling:expr;
        $(bench = $bench:ident;)*
    ) => {
        fn main() {
            let mut bench = $crate::DirectBench::from_args($sampling);

            $(
                let stated = bench.$bench();
                $crate::MeasureBenchAlg::measure(&mut bench, stated);
            )*
        }
    };
}
