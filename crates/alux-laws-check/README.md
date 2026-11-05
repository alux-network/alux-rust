# alux-laws-check

`alux-laws-check` runs `alux-laws` suites: every law on a fresh subject, and one report of everything that failed.

The suite is the specification. Everything about running it belongs to the [`Run`]. A run starts from what makes its subjects, a fresh one per check:

```text
make.obeys(laws)                        every law holds on make()'s subjects
make.caught_by(laws)                    a mutant: a law of `laws` breaks on make()'s subjects
make.check()                            a run on make()'s subjects, to configure first
cases.check_each(|&C| -> S)             a run on a subject per case; a law holds when it holds
                                        on every one, and a break names its case
  .input(value)                         one pinned input for every law over its type
  .inputs(|| -> Vec<I>)                 pinned inputs for every law that takes an `I`
  .inputs_each(|&C| -> Vec<I>)          pinned inputs per case
  .sampled(strategy)                    every law over `I` also runs on drawn inputs
  .sampled_subjects(strategy, tier)     the case set also takes drawn cases
  .tier(Tier::Strict)                   how many sampled cases run (default: from the env)
  .verdicts(&laws)                      one Verdict per law
  .obeys(laws) / .caught_by(laws)       checks, and panics with one report
```

Inputs are found by type, so the order of `.inputs` and `.sampled` does not matter, and one set of laws may take different inputs. Sampling needs the `proptest` feature.

A mutant is a subject built wrong on purpose, and the same suite reads it in a second regime: `obeys` checks that every law holds on a reference, and `caught_by` checks that a law breaks on a mutant. The laws stay the same in both. To pin down which laws catch a mutant, state them in a block of their own, a suite drawn by meaning, and check the mutant `caught_by` that block and `obeys` the others.

The data a suite is checked on is an extension over the subject maker, written once beside the suite's tests, and a small `macro_rules!` gives each row its own `#[test]`. `.input(value)` finds each value's laws by its type:

```rust ignore
/// The data a counter is checked on.
#[ext(name = CounterTestData)]
impl<This, S> This
where
    This: Fn() -> S,
{
    fn with_counter_data(self) -> Run<(), Fresh<This>> {
        self.check().input(Steps(0)).input(Steps(7)).input(Bumps(3))
    }
}

macro_rules! counter_test {
    ($test:ident: $check:expr) => {
        #[test]
        fn $test() {
            $check;
        }
    };
}

counter_test!(good_obeys_counter_laws: Good::default.with_counter_data().obeys(counter_laws()));
counter_test!(bumps_twice_is_caught_by_counter_laws: BumpsTwice::default.with_counter_data().caught_by(counter_laws()));
```

A composition root checks the suites of what it holds, on the value it builds: each suite projected with `over` onto the part it is about, and joined. The joined program is the root's contract. A suite compiles only over a part that has its capability, and `obeys` fails when any law was not reached, so every suite in the program runs. Rust cannot list the impls a program is composed of, so the test states the link.

```rust ignore
build_storage.with_storage_data().obeys(
    db_read_laws().over(|s: &Storage| s.db()).join(kv_cache_laws().over(|s: &Storage| s.cache())),
);
```

```rust
use alux_ext::ext;
use alux_laws::laws;
use alux_laws_check::SubjectLawsExt;

pub trait CounterAlg {
    fn bump(&mut self);
    fn count(&self) -> u32;
}

#[ext(name = CounterLaws, defunc(via = laws))]
pub impl<This> This
where
    This: CounterAlg,
{
    /// Bumping adds one.
    fn bump_adds_one(&mut self) {
        let before = self.count();
        self.bump();
        assert_eq!(self.count(), before + 1);
    }
}

#[derive(Default)]
struct Counter(u32);

impl CounterAlg for Counter {
    fn bump(&mut self) {
        self.0 += 1;
    }

    fn count(&self) -> u32 {
        self.0
    }
}

#[derive(Default)]
struct BumpsTwice(u32);

impl CounterAlg for BumpsTwice {
    fn bump(&mut self) {
        self.0 += 2;
    }

    fn count(&self) -> u32 {
        self.0
    }
}

Counter::default.obeys(counter_laws());
BumpsTwice::default.caught_by(counter_laws());
```
