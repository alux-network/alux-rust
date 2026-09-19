# alux-bench-criterion

`alux-bench-criterion` interprets an [`alux-bench`](https://docs.rs/alux-bench) bench with [criterion](https://docs.rs/criterion).

```rust ignore
use alux_bench::BenchSampling;
use alux_bench_criterion::CriterionBench;
use criterion::{Criterion, criterion_group, criterion_main};

fn closing(criterion: &mut Criterion) {
    CriterionBench::new(criterion).closing();
}

criterion_group! {
    name = closing_bench;
    config = CriterionBench::sampled(BenchSampling::new(3));
    targets = closing
}

criterion_main!(closing_bench);
```

A bench is stated first and measured after, so criterion's `BenchmarkGroup` is held here, in this crate's own loop, rather than while the bench is being written. `MeasureBenchAlg::measure` walks the stated groups in order, opens a `benchmark_group` for each, and states every case as one `bench_function` named by its subject. The routine is run through `iter_custom`, which hands it the rounds of one sample and reads back the time it answers.

A group stated as `together` is measured one case at a time here: criterion owns the loop, so it decides when a case runs. What such a group states is what measuring its cases at once would cost, not what the harness must do.

## Sampling

`CriterionBench::sampled` builds the `Criterion` a `criterion_group!` takes as its `config`:

- `sample_size` is the samples asked for, raised to ten. Criterion asserts at least ten.
- `warm_up_time` and `measurement_time` are what a sample spends, or one millisecond when the sampling states no spend. Criterion divides the measurement time by the round it timed during warm-up to pick rounds per sample, so one millisecond is one round for any round slower than that. It does not cut a round short: a round that waits a second still takes a second.
- Plots are off.
