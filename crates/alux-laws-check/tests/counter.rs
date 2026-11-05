//! The design end to end on a toy counter, read top to bottom.
//!
//! A capability (`CounterAlg`) states its laws once, as an ordinary ext block lowered by
//! `via = laws` into a law program (`counter_laws()`). The program is read by several interpreters
//! without changing it: the checker runs it on a reference and on a mutant, `names()` and
//! `definitions()` read what it holds, and a test can add a reader of its own. Test data is an
//! extension over the subject maker, so each test is one row.

use alux_ext::{OperationAlg, ext};
use alux_laws::{LawAlg, LawDefinitionsExt, LawNamesExt, LawOp, LawsAlg, LawsCompositionExt, LawsProgramExt, laws};
use alux_laws_check::{Fresh, Run, SubjectLawsExt};
use core::any::type_name;

/// The capability whose meaning the laws state.
trait CounterAlg {
    fn bump(&mut self);
    fn add(&mut self, n: u32);
    fn count(&self) -> u32;
}

/// An input of `add_is_repeated_bump`. Laws take small input types of their own, so a run finds
/// each input by its type.
#[derive(Debug, Clone, Copy)]
struct Steps(u32);

/// An input of `bumps_never_lower`: a second input type in the same suite.
#[derive(Debug, Clone, Copy)]
struct Bumps(u32);

/// The laws of `CounterAlg`, written once beside the trait. Each method is an ordinary law over
/// any counter; `via = laws` also generates `Op_<law>` per method and `counter_laws()`, the suite's
/// program. The laws show the three shapes a law may take: no input, a borrowed input, and a
/// `Result` instead of a panic.
#[ext(name = CounterLaws, defunc(via = laws))]
impl<This> This
where
    This: CounterAlg,
{
    /// Bumping adds one.
    fn bump_adds_one(&mut self) {
        let before = self.count();
        self.bump();
        assert_eq!(self.count(), before + 1);
    }

    /// Adding `n` is bumping `n` times.
    fn add_is_repeated_bump(&mut self, steps: &Steps) {
        let start = self.count();
        self.add(steps.0);
        assert_eq!(self.count(), start + steps.0);
    }

    /// Bumping never lowers the count.
    fn bumps_never_lower(&mut self, bumps: &Bumps) -> Result<(), String> {
        let before = self.count();
        (0..bumps.0).for_each(|_| self.bump());
        if self.count() < before { Err(format!("{} < {before}", self.count())) } else { Ok(()) }
    }
}

/// The reference: a counter that obeys every law.
#[derive(Debug, Default)]
struct Good(u32);

impl CounterAlg for Good {
    fn bump(&mut self) {
        self.0 += 1;
    }
    fn add(&mut self, n: u32) {
        self.0 += n;
    }
    fn count(&self) -> u32 {
        self.0
    }
}

/// A mutant: built wrong on purpose, every bump adds two, so `bump_adds_one` must catch it.
#[derive(Debug, Default)]
struct BumpsTwice(u32);

impl CounterAlg for BumpsTwice {
    fn bump(&mut self) {
        self.0 += 2;
    }
    fn add(&mut self, n: u32) {
        self.0 += n;
    }
    fn count(&self) -> u32 {
        self.0
    }
}

/// The data a counter is checked on: an extension over whatever makes the counter, written once and
/// read by every row. `.input(value)` finds the laws of each value by its type.
#[ext(name = CounterTestData)]
impl<This, S> This
where
    This: Fn() -> S,
{
    fn with_counter_data(self) -> Run<(), Fresh<This>> {
        self.check().input(Steps(0)).input(Steps(1)).input(Steps(7)).input(Bumps(0)).input(Bumps(3))
    }
}

/// One test per row: the row's check, under the row's name. The macro knows nothing about laws.
macro_rules! counter_test {
    ($test:ident: $check:expr) => {
        #[test]
        fn $test() {
            $check;
        }
    };
}

// One suite, two regimes: the reference obeys every law, and the mutant is caught by one.
counter_test!(good_obeys_counter_laws: Good::default.with_counter_data().obeys(counter_laws()));
counter_test!(bumps_twice_is_caught_by_counter_laws: BumpsTwice::default.with_counter_data().caught_by(counter_laws()));

/// A law is still an ordinary method: the program is generated beside it, not instead of it, so
/// a law can be called directly on any counter, outside any run.
#[test]
fn a_law_is_an_ordinary_method() {
    let mut counter = Good::default();
    counter.bump_adds_one();
    counter.add_is_repeated_bump(&Steps(3));
    assert_eq!(counter.count(), 4);
    assert_eq!(counter.bumps_never_lower(&Bumps(2)), Ok(()));
}

/// The names of the laws are read off the program, with nothing run: the same value the checker
/// runs, read by another interpreter.
#[test]
fn names_lists_the_laws_in_block_order() {
    let names = counter_laws::<Good>().names();
    assert_eq!(
        names,
        [
            ("CounterLaws", "bump_adds_one"),
            ("CounterLaws", "add_is_repeated_bump"),
            ("CounterLaws", "bumps_never_lower"),
        ]
    );
}

/// What each law is, documentation and input type included, is read off the program too: enough
/// to document a capability's laws or list what a composition owes, with nothing run.
#[test]
fn definitions_read_each_law_with_its_doc_and_input() {
    let definitions = counter_laws::<Good>().definitions();
    let read: Vec<_> = definitions.iter().map(|law| (law.law, law.doc, law.input)).collect();
    assert_eq!(
        read,
        [
            ("bump_adds_one", "Bumping adds one.", "()"),
            ("add_is_repeated_bump", "Adding `n` is bumping `n` times.", "counter::Steps"),
            ("bumps_never_lower", "Bumping never lowers the count.", "counter::Bumps"),
        ]
    );
}

/// A reader of law programs written here, in a test: it counts the laws over each input type.
/// Nothing in the suite or the macro changes for it; it only answers the law algebra.
#[derive(Debug, Default)]
struct CountsByInput;

impl LawsAlg for CountsByInput {
    type Laws = Vec<&'static str>;

    fn laws_empty(&self) -> Self::Laws {
        Vec::new()
    }

    fn laws_merge(&self, mut left: Self::Laws, right: Self::Laws) -> Self::Laws {
        left.extend(right);
        left
    }
}

impl<Subject, Input> LawAlg<Subject, Input> for CountsByInput {
    fn finish_law<Op>(&self, _: &'static str, _: Op) -> Vec<&'static str>
    where
        Op: LawOp<Subject, Input = Input> + OperationAlg,
    {
        vec![type_name::<Input>().rsplit("::").next().unwrap_or_default()]
    }
}

/// A new interpreter compiles the same program as the built-in ones.
#[test]
fn a_test_can_add_its_own_interpreter() {
    let inputs = CountsByInput.compile_laws(&counter_laws::<Good>());
    assert_eq!(inputs, ["()", "Steps", "Bumps"]);
}

/// The checker reports a law no input reached instead of passing it: a run must feed every input
/// type its laws take.
#[test]
fn a_law_with_no_inputs_of_its_type_is_reported_unreached() {
    let verdicts = Good::default.check().input(Steps(1)).verdicts(&counter_laws());
    let unreached: Vec<_> = verdicts.iter().filter(|v| v.reached == 0).map(|v| v.law).collect();
    assert_eq!(unreached, ["bumps_never_lower"]);
}

/// A root that holds two counters, as a composition root holds the carriers it is made of.
#[derive(Debug, Default)]
struct Root {
    left: u32,
    right: u32,
}

/// A composition root checks the laws of what it holds: each suite is projected onto the part of
/// the root it is about with `over`, and the projections are joined into one program. The joined
/// program is the root's contract: each suite compiles only over a part that has its capability, and
/// `obeys` fails when a suite's laws were not all reached.
#[test]
fn a_root_obeys_the_suites_it_holds() {
    let laws =
        counter_laws().over(|root: &Root| Good(root.left)).join(counter_laws().over(|root: &Root| Good(root.right)));
    Root::default.with_counter_data().obeys(laws);
}

/// Sampled inputs are a run setting beside the pinned ones, in any order: every law over `Steps`
/// also runs on drawn steps.
#[cfg(feature = "proptest")]
#[test]
fn sampled_and_pinned_inputs_combine_in_any_order() {
    use proptest::prelude::*;
    let sampled = (0u32..100).prop_map(Steps);
    Good::default.with_counter_data().sampled(sampled).obeys(counter_laws());
}
