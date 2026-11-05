//! The checker's behavior, one toy suite per shape the `laws` backend accepts.
//!
//! `counter.rs` shows the design end to end; this file pins down what the checker does at each
//! edge: what a report says, which subject each law gets, how inputs reach laws, and which
//! authored shapes (generic keys, case structs, unsized parameters, associated-type inputs,
//! `Result` laws, laws named alike) lower and run.

use alux_ext::ext;
use alux_laws::{LawDefinitionsExt, LawNamesExt, LawsCompositionExt, laws};
use alux_laws_check::{CaseLawsExt, SubjectLawsExt};
use std::any::Any;
use std::cell::Cell;
use std::fmt::Debug;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The capability most suites here are over.
trait Counter {
    fn bump(&mut self);
    fn get(&self) -> u32;
}

/// Two laws over the subject alone, one returning a `Result` and one panicking, so reports of both
/// shapes can be compared.
#[ext(name = CounterLaws, defunc(via = laws))]
impl<This> This
where
    This: Counter,
{
    fn bump_adds_one(&mut self) -> Result<(), String> {
        let before = self.get();
        self.bump();
        let after = self.get();
        (after == before + 1).then_some(()).ok_or_else(|| format!("{before} bumped to {after}"))
    }

    fn starts_at_zero(&self) {
        assert_eq!(self.get(), 0, "a new counter reads zero");
    }
}

/// Two laws over one input type, each counting its calls, so tests can see exactly which inputs
/// reached which law.
#[ext(name = CounterStepLaws, defunc(via = laws))]
impl<This> This
where
    This: Counter,
{
    fn bump_n_adds_n(&mut self, n: &u8) -> Result<(), String> {
        STEP_CALLS.with(|calls| calls.set(calls.get() + 1));
        let before = self.get();
        (0..*n).for_each(|_| self.bump());
        let after = self.get();
        (after == before + u32::from(*n)).then_some(()).ok_or_else(|| format!("{before} + {n} read {after}"))
    }

    fn get_is_stable(&self, _n: &u8) -> Result<(), String> {
        STABLE_CALLS.with(|calls| calls.set(calls.get() + 1));
        (self.get() == self.get()).then_some(()).ok_or_else(|| "two reads differ".into())
    }
}

/// Laws whose input is where the subject was built to start: an input that belongs to one subject,
/// which no other subject can be checked against.
#[ext(name = StartLaws, defunc(via = laws))]
impl<This> This
where
    This: Counter,
{
    fn reads_where_it_starts(&self, at: &u32) -> Result<(), String> {
        (self.get() == *at).then_some(()).ok_or_else(|| format!("built at {at}, reads {}", self.get()))
    }

    fn one_bump_moves_one_past_it(&mut self, at: &u32) -> Result<(), String> {
        self.bump();
        (self.get() == at + 1).then_some(()).ok_or_else(|| format!("{at} bumped reads {}", self.get()))
    }
}

// Call and construction counts, per test thread, for the tests that check how often laws run and
// subjects are made.
thread_local! {
    static STEP_CALLS: Cell<usize> = const { Cell::new(0) };
    static STABLE_CALLS: Cell<usize> = const { Cell::new(0) };
    static MADE: Cell<usize> = const { Cell::new(0) };
}

/// A counter that starts at `at` and adds `step` per bump. `Good`, `(0, 1)`, is the reference; any
/// other step breaks `bump_adds_one`, any other start `starts_at_zero`.
#[derive(Debug, Clone)]
struct Stepper {
    at: u32,
    step: u32,
}

/// The reference counter, under the name the tests read best with.
type Good = Stepper;

impl Default for Stepper {
    fn default() -> Self {
        Self { at: 0, step: 1 }
    }
}

impl Counter for Stepper {
    fn bump(&mut self) {
        self.at += self.step;
    }

    fn get(&self) -> u32 {
        self.at
    }
}

/// Mutant: every bump adds two.
fn bumps_twice() -> Stepper {
    Stepper { at: 0, step: 2 }
}

/// Mutant: starts at one, so `starts_at_zero` panics.
fn starts_at_one() -> Stepper {
    Stepper { at: 1, step: 1 }
}

/// A `Stepper` from its case, counted in `MADE`. A case is borrowed because `check_each` lends it.
#[allow(clippy::trivially_copy_pass_by_ref, reason = "`check_each` lends each case")]
fn stepper(&(at, step): &(u32, u32)) -> Stepper {
    MADE.with(|made| made.set(made.get() + 1));
    Stepper { at, step }
}

/// The pinned inputs of the `u8` laws.
fn pinned() -> Vec<u8> {
    vec![1, 3]
}

/// The report `f` panics with, or `None` when it returns.
fn report_of(f: impl FnOnce()) -> Option<String> {
    let payload: Box<dyn Any + Send> = catch_unwind(AssertUnwindSafe(f)).err()?;
    let text = payload.downcast_ref::<String>().cloned();
    Some(text.or_else(|| payload.downcast_ref::<&str>().map(ToString::to_string)).unwrap_or_default())
}

/// The two regimes of one suite: a mutant is caught (`caught_by`), the same mutant fails as a
/// reference (`obeys`), and a reference no law catches fails as a mutant.
#[test]
fn a_mutant_is_caught_by_the_law_it_breaks() {
    bumps_twice.caught_by(counter_laws());

    let report = report_of(|| bumps_twice.obeys(counter_laws())).expect("a break fails a subject that must obey");
    assert!(report.contains("bump_adds_one"), "{report}");
    assert!(!report.contains("starts_at_zero"), "{report}");

    let report = report_of(|| Good::default.caught_by(counter_laws())).expect("a subject no law catches fails");
    assert!(report.contains("no law caught the subject: 2 held, 0 reached by no input"), "{report}");
}

/// What one bump does, apart from where a counter starts.
#[ext(name = BumpLaws, defunc(via = laws))]
impl<This> This
where
    This: Counter,
{
    fn one_bump_adds_one(&mut self) {
        let before = self.get();
        self.bump();
        assert_eq!(self.get(), before + 1);
    }
}

/// Where a counter starts, apart from what bumping does.
#[ext(name = OriginLaws, defunc(via = laws))]
impl<This> This
where
    This: Counter,
{
    fn a_new_counter_reads_zero(&self) {
        assert_eq!(self.get(), 0);
    }
}

/// Precision without naming laws: each property is a block of its own, so a mutant is caught by the
/// block it targets and still obeys the others.
#[test]
fn a_mutant_is_caught_by_its_block_and_obeys_the_others() {
    Good::default.obeys(bump_laws().join(origin_laws()));
    bumps_twice.caught_by(bump_laws());
    bumps_twice.obeys(origin_laws());
    starts_at_one.caught_by(origin_laws());
    starts_at_one.obeys(bump_laws());
}

/// Laws never share a subject, so one law's mutation cannot make another pass or fail.
#[test]
fn each_law_gets_a_fresh_subject() {
    // `bump_adds_one` runs first and leaves the counter at 1; `starts_at_zero` holds only on a new one.
    MADE.with(|made| made.set(0));
    (|| stepper(&(0, 1))).obeys(counter_laws());
    assert_eq!(MADE.with(Cell::get), 2);
}

/// A panicking law is a verdict, not a crash: the report names the law and the line it panicked
/// at, and keeps the assertion's own lines readable.
#[test]
fn a_panicking_law_is_reported_with_where_it_panicked() {
    let report = report_of(|| starts_at_one.obeys(counter_laws())).expect("a panicking law fails the check");
    let source = include_str!("check.rs").lines();
    let line = 1 + source.take_while(|line| !line.contains("\"a new counter reads zero\");")).count();
    let at = format!("\n  broke CounterLaws::starts_at_zero: panicked at {}:{line}: ", file!());
    assert!(report.contains(&at), "{report}");
    // An entry continues deeper than the next entry starts: the assertion's own `  left:` /
    // ` right:` lines, under an entry indented by two.
    assert!(report.contains("\n      left: 1\n     right: 0"), "{report}");
}

/// A break on an input names that input, so the failing case can be reproduced.
#[test]
fn a_pinned_break_shows_the_input_it_broke_on() {
    let report =
        report_of(|| bumps_twice.check().inputs(pinned).obeys(counter_step_laws())).expect("a mutant fails the check");
    assert!(report.contains("broke CounterStepLaws::bump_n_adds_n: input 1: "), "{report}");
}

/// A text input.
#[derive(Debug)]
struct Text(String);

/// Laws over a text input, to show how long inputs are cut.
#[ext(name = TextLaws, defunc(via = laws))]
impl<This> This
where
    This: Counter,
{
    fn text_is_short(&self, text: &Text) {
        assert!(text.0.len() < 10, "too long");
    }
}

/// A large input is cut in the verdict, so one break cannot flood the report.
#[test]
fn a_long_input_is_cut_in_the_verdict() {
    let verdicts = Good::default.check().inputs(|| vec![Text("x".repeat(400))]).verdicts(&text_laws());
    let shown = verdicts[0].broke.as_ref().and_then(|broke| broke.input.clone()).expect("the law broke on its input");
    assert_eq!(shown.chars().count(), 161);
    assert!(shown.ends_with('…'));
}

/// The catalog interpreter reads the argument names and input types the backend recorded, with
/// nothing run.
#[test]
fn definitions_read_what_each_law_is() {
    let definitions = counter_step_laws::<Good>().definitions();
    let read: Vec<_> = definitions.iter().map(|law| (law.suite, law.law, law.arg_names, law.input)).collect();
    assert_eq!(
        read,
        [
            ("CounterStepLaws", "bump_n_adds_n", &["n"][..], "u8"),
            ("CounterStepLaws", "get_is_stable", &["_n"][..], "u8")
        ]
    );
    assert!(definitions.iter().all(|law| law.subject.ends_with("Stepper")), "{definitions:?}");
}

/// A suite keeps the order its block was written in, so reports and catalogs read like the source.
#[test]
fn a_suite_lists_its_laws_in_block_order() {
    assert_eq!(
        counter_step_laws::<Good>().names(),
        [("CounterStepLaws", "bump_n_adds_n"), ("CounterStepLaws", "get_is_stable")]
    );
}

/// A composition root holding two counters, as a node holds the carriers it is made of.
#[derive(Debug, Clone, Default)]
struct Root {
    counter: Good,
    stepper: Good,
}

/// A root's check is one program: every suite projected onto its part of the root and joined, run
/// in order, and read by `names()` like any suite.
#[test]
fn a_join_over_a_root_runs_every_suite_it_holds() {
    // One join of suites over a root.
    let root_suites = || {
        let counter = counter_laws().over(|root: &Root| root.counter.clone());
        let stepper = counter_step_laws().over(|root: &Root| root.stepper.clone());
        counter.join(stepper)
    };
    Root::default.check().inputs(pinned).obeys(root_suites());
    let report = report_of(|| Root::default.check().inputs(pinned).caught_by(root_suites()))
        .expect("a reference root is caught by none of the suites it holds");
    assert!(report.contains("no law caught the subject: 4 held"), "{report}");
    assert_eq!(
        root_suites().names(),
        [
            ("CounterLaws", "bump_adds_one"),
            ("CounterLaws", "starts_at_zero"),
            ("CounterStepLaws", "bump_n_adds_n"),
            ("CounterStepLaws", "get_is_stable"),
        ]
    );
}

/// A suite with no laws, to show that checking nothing fails rather than passes.
#[ext(name = EmptyLaws, defunc(via = laws))]
impl<This> This where This: Counter {}

/// An empty check is a mistake, not a success: it fails.
#[test]
fn a_suite_with_no_laws_fails() {
    let report = report_of(|| Good::default.obeys(empty_laws::<Good>())).expect("an empty suite fails the check");
    assert!(report.contains("there are no laws to check"), "{report}");
}

/// A law whose input type the run never gave is reported, so a forgotten input cannot pass as a
/// held law.
#[test]
fn a_law_no_input_reaches_fails() {
    let report = report_of(|| Good::default.obeys(counter_step_laws())).expect("an unreached law fails");
    assert!(report.contains("no input reached CounterStepLaws::bump_n_adds_n"), "{report}");
}

/// A collection over keys `K`.
trait Col<K> {
    fn put(&mut self, key: K);
    fn has(&self, key: &K) -> bool;
}

/// A suite generic over a second parameter, the key, beside the subject.
#[ext(name = ColLaws, defunc(via = laws))]
impl<This, K> This
where
    This: Col<K>,
    K: Clone + Debug,
{
    fn put_then_has(&mut self, key: &K) -> Result<(), String> {
        self.put(key.clone());
        self.has(key).then_some(()).ok_or_else(|| format!("put {key:?}, then missed it"))
    }

    fn put_is_idempotent(&mut self, key: &K) -> Result<(), String> {
        self.put(key.clone());
        self.put(key.clone());
        self.has(key).then_some(()).ok_or_else(|| format!("put {key:?} twice, then missed it"))
    }
}

/// The reference collection.
#[derive(Debug, Default)]
struct VecCol(Vec<u8>);

impl Col<u8> for VecCol {
    fn put(&mut self, key: u8) {
        self.0.push(key);
    }

    fn has(&self, key: &u8) -> bool {
        self.0.contains(key)
    }
}

/// Mutant: forgets every key it is given.
#[derive(Debug, Default)]
struct DropsEveryKey;

impl Col<u8> for DropsEveryKey {
    fn put(&mut self, _: u8) {}

    fn has(&self, _: &u8) -> bool {
        false
    }
}

/// A suite with a parameter besides the subject runs once the parameter is chosen at the call.
#[test]
fn a_suite_generic_over_its_key() {
    let keys = || vec![1u8, 7];
    VecCol::default.check().inputs(keys).obeys(col_laws::<_, u8>());
    DropsEveryKey::default.check().inputs(keys).caught_by(col_laws::<_, u8>());
}

/// A case struct: the suite's subject is the corpus itself, not a carrier of some trait.
#[derive(Debug, Clone)]
struct Corpus<T>(Vec<T>);

/// Laws over a concrete generic type rather than over a trait's carriers.
#[ext(name = CorpusLaws, defunc(via = laws))]
impl<T> Corpus<T>
where
    T: PartialEq + Debug,
{
    fn every_case_equals_itself(&self) -> Result<(), String> {
        let unequal = self.0.iter().find(|case| PartialEq::ne(*case, *case));
        unequal.map_or(Ok(()), |case| Err(format!("{case:?} != itself")))
    }

    fn the_corpus_is_nonempty(&self) {
        assert!(!self.0.is_empty(), "an empty corpus checks nothing");
    }
}

/// A suite's subject can be any type: here a corpus, checked as a reference and as two mutants.
#[test]
fn a_suite_over_a_case_struct() {
    (|| Corpus(vec![1u8, 2])).obeys(corpus_laws());
    (|| Corpus(vec![1.0, f64::NAN])).caught_by(corpus_laws());
    (|| Corpus(Vec::<u8>::new())).caught_by(corpus_laws());
}

/// Boxed values of a possibly unsized type, as the input of a suite whose subject is `()`.
#[derive(Debug)]
struct Boxed<T: ?Sized>(Vec<Box<T>>);

/// A suite over a `?Sized` parameter: the generated operations and suite function keep it unsized.
#[ext(name = UnsizedLaws, defunc(via = laws))]
impl<T: ?Sized + PartialEq> () {
    fn every_value_equals_itself(&self, boxed: &Boxed<T>) {
        assert!(boxed.0.iter().zip(&boxed.0).all(|(a, b)| a == b), "a value != itself");
    }
}

/// An unsized parameter first, a sized one after it.
#[ext(name = UnsizedFirstLaws, defunc(via = laws))]
impl<T, U> ()
where
    T: ?Sized + PartialEq,
    U: PartialEq,
{
    fn both_sides_equal_themselves(&self, at: &(Boxed<T>, Vec<U>)) {
        assert!(at.0.0.iter().zip(&at.0.0).all(|(a, b)| a == b), "a value != itself");
        assert!(at.1.iter().zip(&at.1).all(|(a, b)| a == b), "a value != itself");
    }
}

/// Unsized parameters (`str`, `[u8]`) are accepted wherever the authored block accepts them.
#[test]
fn a_suite_over_an_unsized_parameter() {
    (|| ()).check().inputs(|| vec![Boxed::<str>(vec!["a".into(), "b".into()])]).obeys(unsized_laws::<str>());
    (|| ()).check().inputs(|| vec![Boxed::<[u8]>(vec![[1u8].into()])]).obeys(unsized_laws::<[u8]>());
    (|| ()).check().inputs(|| vec![(Boxed::<str>(vec!["a".into()]), vec![1])]).obeys(unsized_first_laws::<str, i32>());
}

/// Laws returning `Result` with errors of different types.
#[ext(name = ResultLaws, defunc(via = laws))]
impl<This> This
where
    This: Counter,
{
    /// Its error is the odd value it read.
    fn reads_even_after_two_bumps(&mut self) -> Result<(), u32> {
        self.bump();
        self.bump();
        let read = self.get();
        if read.is_multiple_of(2) { Ok(()) } else { Err(read) }
    }

    /// Its error is a fixed message.
    fn reads_below_a_hundred(&self) -> Result<(), &'static str> {
        (self.get() < 100).then_some(()).ok_or("a hundred or more")
    }
}

/// A law's `Err` is its failure, shown by its `Debug`, and only broken laws are reported.
#[test]
fn a_law_returning_a_result() {
    Good::default.obeys(result_laws());
    let report = report_of(|| starts_at_one.obeys(result_laws())).expect("an odd read breaks the law");
    // The error value is the outcome: `Debug` of the `E` the law returned.
    assert!(report.contains("broke ResultLaws::reads_even_after_two_bumps: 3"), "{report}");
    assert!(!report.contains("reads_below_a_hundred"), "{report}");
}

/// A store whose keys are an associated type.
trait Keyed {
    type Key;
    fn insert(&mut self, key: Self::Key);
    fn holds(&self, key: &Self::Key) -> bool;
}

/// The input names an associated type of the subject.
#[ext(name = KeyedLaws, defunc(via = laws))]
impl<This> This
where
    This: Keyed,
    This::Key: Clone,
{
    fn insert_then_holds(&mut self, key: &This::Key) -> Result<(), String> {
        self.insert(key.clone());
        self.holds(key).then_some(()).ok_or_else(|| "inserted, then missed".into())
    }

    fn a_new_store_holds_nothing(&self, key: &This::Key) -> Result<(), String> {
        (!self.holds(key)).then_some(()).ok_or_else(|| "a new store holds a key".into())
    }
}

/// The reference store.
#[derive(Debug, Default)]
struct KeyedVec(Vec<u8>);

impl Keyed for KeyedVec {
    type Key = u8;

    fn insert(&mut self, key: u8) {
        self.0.push(key);
    }

    fn holds(&self, key: &u8) -> bool {
        self.0.contains(key)
    }
}

/// Mutant: holds every key, inserted or not.
#[derive(Debug, Default)]
struct HoldsAll;

impl Keyed for HoldsAll {
    type Key = u8;

    fn insert(&mut self, _: u8) {}

    fn holds(&self, _: &u8) -> bool {
        true
    }
}

/// An input named through the subject's associated type is found by the type it resolves to.
#[test]
fn a_suite_whose_input_is_an_associated_type() {
    KeyedVec::default.check().inputs(pinned).obeys(keyed_laws());
    HoldsAll::default.check().inputs(pinned).caught_by(keyed_laws());
}

/// A second suite with a law named like one of `CounterLaws`. Its operations share a namespace with
/// that suite's, so it is declared in a module of its own.
mod twin {
    use super::Counter;
    use alux_ext::ext;
    use alux_laws::laws;

    #[ext(name = CounterTwinLaws, defunc(via = laws))]
    pub(super) impl<This> This
    where
        This: Counter,
    {
        fn starts_at_zero(&self) -> Result<(), String> {
            (self.get() == 0).then_some(()).ok_or_else(|| format!("it starts at {}", self.get()))
        }
    }
}

/// Laws named alike in two suites stay apart: both run, and each is reported under its own suite.
#[test]
fn two_suites_share_a_law_name() {
    let both = || counter_laws().join(twin::counter_twin_laws());
    Good::default.obeys(both());
    let report = report_of(|| starts_at_one.obeys(both())).expect("both laws named alike break");
    assert!(report.contains("broke CounterLaws::starts_at_zero: panicked at"), "{report}");
    assert!(report.contains("broke CounterTwinLaws::starts_at_zero: \"it starts at 1\""), "{report}");
    assert_eq!(
        counter_laws::<Good>().join(twin::counter_twin_laws::<Good>()).names(),
        [("CounterLaws", "bump_adds_one"), ("CounterLaws", "starts_at_zero"), ("CounterTwinLaws", "starts_at_zero"),]
    );
}

/// A set of subjects obeys only when every subject does, and a break names the subject it broke on.
#[test]
fn one_subject_breaking_fails_the_set_and_is_named() {
    let report = report_of(|| vec![(0, 1), (0, 2)].check_each(stepper).obeys(counter_laws()))
        .expect("a law broken on one subject fails the set");
    assert!(report.contains("broke CounterLaws::bump_adds_one: on subject (0, 2)"), "{report}");
    assert!(!report.contains("(0, 1)"), "{report}");
    assert!(!report.contains("starts_at_zero"), "{report}");
}

/// A set of mutants is caught when a law breaks on any one of them.
#[test]
fn a_set_is_caught_when_one_subject_breaks_a_law() {
    vec![(0, 1), (0, 2)].check_each(stepper).caught_by(counter_laws());

    let report = report_of(|| vec![(0, 1), (0, 1)].check_each(stepper).caught_by(counter_laws()))
        .expect("a set no law catches fails");
    assert!(report.contains("no law caught the subject"), "{report}");
}

/// A run with no subjects is a mistake, not a success: it fails.
#[test]
fn an_empty_set_fails() {
    let report = report_of(|| Vec::<(u32, u32)>::new().check_each(stepper).obeys(counter_laws()))
        .expect("a set with no subjects fails");
    assert!(report.contains("was given no subjects"), "{report}");
}

/// The input each `(at, step)` case is checked on: its own start, and only its own.
#[allow(clippy::trivially_copy_pass_by_ref, reason = "`inputs_each` lends each case")]
fn its_own_start(&(at, _): &(u32, u32)) -> Vec<u32> {
    vec![at]
}

/// Inputs chosen per case reach only that case's subject: the start a counter was built at checks
/// that counter and no other.
#[test]
fn a_per_case_input_catches_one_subject_while_the_rest_hold() {
    vec![(0, 1), (3, 2)].check_each(stepper).inputs_each(its_own_start).caught_by(start_laws());

    let report = report_of(|| vec![(0, 1), (3, 2)].check_each(stepper).inputs_each(its_own_start).obeys(start_laws()))
        .expect("an unlisted break on one subject fails the set");
    assert!(report.contains("broke StartLaws::one_bump_moves_one_past_it: on subject (3, 2)"), "{report}");
    assert!(!report.contains("reads_where_it_starts"), "{report}");
}

/// Sampling, behind the `proptest` feature: drawn inputs and subjects, and how the tier bounds them.
#[cfg(feature = "proptest")]
mod sampled {
    use super::*;
    use alux_laws_check::Tier;

    /// Each drawn input gets a fresh subject, like each pinned one.
    #[test]
    fn a_sampled_case_builds_one_subject() {
        MADE.with(|made| made.set(0));
        (|| stepper(&(0, 1))).check().sampled(0u8..10).inputs(pinned).tier(Tier::Dev).obeys(counter_step_laws());
        // Each law on each pinned input and on the one case `Tier::Dev` samples.
        assert_eq!(MADE.with(Cell::get), 2 * (pinned().len() + 1));
    }

    /// Drawn inputs reach every law over their type: no law opts in or out of sampling.
    #[test]
    fn sampled_inputs_reach_every_law_over_their_type() {
        STEP_CALLS.with(|calls| calls.set(0));
        STABLE_CALLS.with(|calls| calls.set(0));
        Good::default.check().sampled(1u8..=9).inputs(pinned).tier(Tier::Strict).obeys(counter_step_laws());
        assert_eq!(STEP_CALLS.with(Cell::get), pinned().len() + 64);
        assert_eq!(STABLE_CALLS.with(Cell::get), pinned().len() + 64);
    }

    /// Sampling no cases is refused when the run is set up, not discovered when it passes.
    #[test]
    #[should_panic(expected = "at least one case")]
    fn sampling_no_cases_is_refused() {
        let _ = Good::default.check().sampled_cases(1u8..=9, 0);
    }

    /// Drawn subjects join the pinned ones, as many as the tier allows, and a break names the drawn
    /// subject.
    #[test]
    fn sampled_subjects_follow_the_tier() {
        MADE.with(|made| made.set(0));
        vec![(0, 1)].check_each(stepper).sampled_subjects((0u32..1, 1u32..2), Tier::Dev).obeys(counter_laws());
        assert_eq!(MADE.with(Cell::get), 2 * 2, "two laws on the pinned and the drawn subject");

        let report = report_of(|| {
            vec![(0, 1)].check_each(stepper).sampled_subjects((0u32..1, 2u32..3), Tier::Dev).obeys(counter_laws());
        })
        .expect("a drawn subject that breaks a law fails the set");
        assert!(report.contains("broke CounterLaws::bump_adds_one: on subject (0, 2)"), "{report}");
    }
}
