# alux-bench

`alux-bench` states what a benchmark measures, independently of the harness that measures it.

[`BenchAlg`](https://docs.rs/alux-bench/latest/alux_bench/trait.BenchAlg.html) states a benchmark; [`MeasureBenchAlg`](https://docs.rs/alux-bench/latest/alux_bench/trait.MeasureBenchAlg.html) measures a stated one. They are separate on purpose: stating runs nothing, so a harness holding something mutable holds it in its own loop rather than while the bench is being written.

```rust ignore
use alux_bench::{BenchAlg, BenchExt};
use alux_ext::ext;

/// The benchmarks, stated over whatever measures them.
#[ext(name = ClosingBenches)]
pub impl<This> This
where
    This: BenchAlg,
{
    fn closing(&self) -> This::Bench {
        self.benches([
            self.one_at_a_time("close, req OK 1 sec", [("hyper", hyper_rounds), ("axum", axum_rounds)]),
            self.together("close, req OK 30 sec", [("poem", poem_rounds), ("rocket", rocket_rounds)]),
        ])
    }
}
```

Nothing there names a harness, so the same bench runs on any interpretation: `alux-bench-direct` runs the cases itself, `alux-bench-criterion` folds them into criterion's own groups.

## What a bench states

`BenchStated` is what a bench denotes: groups, in the order stated, each holding its cases. `StatingBench` is the interpretation that states exactly that and nothing else, which is what both interpreters carry.

A group states how measuring its cases at once would cost. `one_at_a_time` is for a CPU-bound round: rounds running beside each other contend for cores, and what is measured is the contention. `together` is for an IO-bound round: time spent blocked on a socket, or on a timer the code under test owns, costs the same blocked beside another round, so the group takes about what its longest case takes rather than the sum. An interpretation that cannot run cases at the same time, criterion among them, measures a `together` group one case at a time; what the group states is what it would cost, not what the harness must do.

## What a case is

`BenchCase` names a group and a subject. A group holds the cases a run compares: one load, one operation, one set of numbers to read side by side. A subject is what is measured under it, such as one provider.

## What a round is

A case is measured by its routine: it is handed a `BenchRounds`, the rounds this sample runs, and answers the time they took. The routine times itself, so setup a round needs but does not measure stays off the clock. A routine is boxed, `BenchRoutine`, or `BenchSentRoutine` where the case may be measured beside others.

`BenchSampling` states how many samples a case is measured over, and how long one sample may spend on rounds. A sample that states no spend runs the fewest rounds the interpreter can run, which is one round of a case whose round is the drain of a server. A sample that states a spend runs the rounds that fit, which is what a case whose round is one request wants. The interpreter maps both onto its harness.
