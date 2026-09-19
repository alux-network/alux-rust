# alux-bench-direct

`alux-bench-direct` interprets an [`alux-bench`](https://docs.rs/alux-bench) bench by running it, with no harness in between.

```rust ignore
use alux_bench::BenchSampling;
use alux_bench_direct::bench_main;

bench_main! {
    sampling = BenchSampling::new(3);
    bench = closing;
    bench = handover;
}
```

Each `bench` is a method of the bench being run: an extension over `BenchAlg`, stated wherever the benchmark is authored. `bench_main!` builds the bench with `DirectBench::from_args`, which takes the filter from the first argument that is not a flag, and runs each in the order stated.

A bench is stated first and measured after. `MeasureBenchAlg::measure` walks the stated groups in order: a `one_at_a_time` group runs its cases one after another, a `together` group runs each on a thread of its own and joins them in the order the group states. A thread rather than a task, because an IO-bound routine blocks rather than yielding.

Each case says what it measured on stdout as soon as it has it, so an interrupted run has said everything it finished. What is being measured is said on stderr, which a terminal repaints in place, so the two streams never repeat each other and piping stdout gives the lines alone.

One case is run `samples` times. What is said is one round: a sample of many rounds is divided by the rounds it ran, and the line carries the case, its samples, then the shortest, the median and the longest. Nothing is kept after a case has said it. `DirectBench::only` skips the cases whose group and subject do not contain a filter, which is what a `cargo bench -- "6 sec"` argument states.

A sample that states a spend runs the rounds that fit it: one round is run first, and the spend is divided by what that round cost. Cost rather than measurement, so a case that opens a server off the clock still spends about what it states.

It runs the samples asked for. [`alux-bench-criterion`](https://docs.rs/alux-bench-criterion) raises anything below ten to ten, because `Criterion::sample_size` asserts at least ten, so three samples of a one second round cost ten seconds there and three here.

What it does not do is what criterion is for: warm-up, rounds per sample picked from a measurement time, outlier detection, comparison against the previous run.
