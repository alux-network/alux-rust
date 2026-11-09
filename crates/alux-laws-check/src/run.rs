//! The [`Run`]: where subjects and inputs come from, and what is expected of the laws.

use crate::caught::caught;
use crate::report::{uncaught, unobeyed};
use crate::tier::Tier;
use alux_ext::OperationAlg;
use alux_laws::{Broke, LawAlg, LawOp, LawsAlg, LawsProgramAlg, Verdict};
#[cfg(feature = "proptest")]
use std::any::type_name;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

/// Makes the subject of a check from its case.
pub trait MakeSubject<C> {
    /// The subject made.
    type Subject;

    /// Returns a fresh subject for `case`.
    fn make_subject(&self, case: &C) -> Self::Subject;
}

impl<C, S, F> MakeSubject<C> for F
where
    F: Fn(&C) -> S,
{
    type Subject = S;

    fn make_subject(&self, case: &C) -> S {
        self(case)
    }
}

/// A maker of subjects that needs no case: the run of one subject maker.
#[derive(Debug, Clone, Copy)]
pub struct Fresh<F>(pub(crate) F);

impl<S, F> MakeSubject<()> for Fresh<F>
where
    F: Fn() -> S,
{
    type Subject = S;

    fn make_subject(&self, (): &()) -> S {
        (self.0)()
    }
}

/// A run's configuration: the subjects, the inputs by type and the tier.
pub struct Run<C, M> {
    cases: Vec<C>,
    make: M,
    /// Whether a verdict names the case it came from; not for a run over one subject maker.
    named: bool,
    sources: HashMap<TypeId, Box<dyn Any>>,
    /// `(what, authored cases)` per sampled source, noted once when the run starts.
    sampled: Vec<(String, usize)>,
    tier: Option<Tier>,
}

/// Where the inputs of one type come from.
pub(crate) struct Source<C, I> {
    pinned: Vec<Pinned<C, I>>,
    /// Draws as many inputs as a tier allows, from a fixed seed.
    sampled: Vec<Draw<I>>,
}

/// Draws as many inputs as a tier allows.
pub(crate) type Draw<I> = Box<dyn Fn(Tier) -> Vec<I>>;

/// Pinned inputs: the same for every case, or chosen per case.
pub(crate) enum Pinned<C, I> {
    All(Vec<I>),
    Each(PerCase<C, I>),
}

/// Chooses the inputs of one case.
pub(crate) type PerCase<C, I> = Box<dyn Fn(&C) -> Vec<I>>;

impl<C, M> Run<C, M> {
    pub(crate) fn new(cases: Vec<C>, make: M, named: bool) -> Self {
        Run { cases, make, named, sources: HashMap::new(), sampled: Vec::new(), tier: None }
    }

    /// Sampled sources draw as many cases as `tier` allows; without it, [`Tier::from_env`].
    #[must_use]
    pub fn tier(self, tier: Tier) -> Self {
        Self { tier: Some(tier), ..self }
    }

    fn the_tier(&self) -> Tier {
        self.tier.unwrap_or_else(Tier::from_env)
    }
}

impl<C: 'static, M> Run<C, M> {
    /// Every law over `I` also runs on `input`. A row lists its inputs this way, values of any
    /// input types in any order: `.input(Steps(7)).input(Bumps(3))`.
    #[must_use]
    pub fn input<I: 'static>(mut self, input: I) -> Self {
        self.source::<I>().pinned.push(Pinned::All(vec![input]));
        self
    }

    /// Every law over `I` runs on each of `inputs` (called once, here).
    #[must_use]
    pub fn inputs<I: 'static>(mut self, inputs: impl FnOnce() -> Vec<I>) -> Self {
        self.source::<I>().pinned.push(Pinned::All(inputs()));
        self
    }

    /// Every law over `I` runs, on each case's subject, on that case's inputs, and no case's
    /// input is fed to another's subject.
    #[must_use]
    pub fn inputs_each<I: 'static>(mut self, inputs: impl Fn(&C) -> Vec<I> + 'static) -> Self {
        self.source::<I>().pinned.push(Pinned::Each(Box::new(inputs)));
        self
    }

    pub(crate) fn source<I: 'static>(&mut self) -> &mut Source<C, I> {
        let empty = || Box::new(Source::<C, I> { pinned: Vec::new(), sampled: Vec::new() }) as Box<dyn Any>;
        let source = self.sources.entry(TypeId::of::<I>()).or_insert_with(empty);
        source.downcast_mut().expect("a source is stored under its own type")
    }

    #[cfg(feature = "proptest")]
    pub(crate) fn add_sampled<I: 'static>(&mut self, authored: usize, draw: Draw<I>) {
        self.sampled.push((type_name::<I>().to_owned(), authored));
        self.source::<I>().sampled.push(draw);
    }

    #[cfg(feature = "proptest")]
    pub(crate) fn add_cases(&mut self, authored: usize, cases: Vec<C>) {
        self.sampled.push(("subjects".to_owned(), authored));
        self.cases.extend(cases);
    }
}

impl<C: Debug + 'static, M> Run<C, M> {
    /// Returns one verdict per law of `laws`, in order, over every subject of the run: the program
    /// compiled once per case, by a [`CaseCheck`] that checks each law as it is compiled.
    pub fn verdicts<L>(&self, laws: &L) -> Vec<Verdict>
    where
        L: for<'r> LawsProgramAlg<CaseCheck<'r, C, M>, Laws = Vec<Verdict>>,
    {
        let tier = self.the_tier();
        let mut all: Vec<Verdict> = Vec::new();
        for case in &self.cases {
            let one = laws.compile_laws(&CaseCheck { run: self, case, tier });
            if all.is_empty() {
                all = one;
            } else {
                all.iter_mut().zip(one).for_each(|(sum, one)| sum.merge(one));
            }
        }
        all
    }

    /// Checks that every law of `laws` holds on every subject and was reached by an input, so every
    /// suite of a joined program ran.
    ///
    /// # Panics
    ///
    /// Panics with one report when a law breaks, a law is reached by no input, the laws are none,
    /// or the run has no subjects.
    #[allow(clippy::needless_pass_by_value, reason = "a run takes the laws as its call site states them")]
    pub fn obeys<L>(self, laws: L)
    where
        L: for<'r> LawsProgramAlg<CaseCheck<'r, C, M>, Laws = Vec<Verdict>>,
    {
        let verdicts = self.verdicts_noted(&laws);
        let entries = unobeyed(&verdicts, self.cases.is_empty());
        fail_on(&entries);
    }

    /// Checks that `laws` catch the subjects, built wrong on purpose: a law of `laws` breaks on
    /// at least one of them.
    ///
    /// # Panics
    ///
    /// Panics with one report when every law held or was reached by no input, the laws are none,
    /// or the run has no subjects.
    #[allow(clippy::needless_pass_by_value, reason = "a run takes the laws as its call site states them")]
    pub fn caught_by<L>(self, laws: L)
    where
        L: for<'r> LawsProgramAlg<CaseCheck<'r, C, M>, Laws = Vec<Verdict>>,
    {
        let entries = uncaught(&self.verdicts_noted(&laws), self.cases.is_empty());
        fail_on(&entries);
    }

    /// The verdicts of `laws`, with a note for each sampled source that runs fewer cases than it
    /// authors.
    fn verdicts_noted<L>(&self, laws: &L) -> Vec<Verdict>
    where
        L: for<'r> LawsProgramAlg<CaseCheck<'r, C, M>, Laws = Vec<Verdict>>,
    {
        let tier = self.the_tier();
        for (what, authored) in &self.sampled {
            note_sampled(what, *authored, tier);
        }
        self.verdicts(laws)
    }
}

/// Panics with `entries` as one report, unless there are none.
fn fail_on(entries: &[String]) {
    // An entry's own line breaks (an assertion's `left:`/`right:`) sit deeper than the next entry,
    // so where one entry ends stays visible.
    let entries: Vec<String> = entries.iter().map(|entry| entry.replace('\n', "\n    ")).collect();
    assert!(entries.is_empty(), "the law check failed:\n  {}", entries.join("\n  "));
}

/// Checks each law of a program on the subjects of one case as the program is compiled: the
/// checking interpreter of [`LawAlg`], whose law collection is the verdicts.
#[allow(missing_debug_implementations, reason = "it borrows a run, which holds closures")]
pub struct CaseCheck<'r, C, M> {
    run: &'r Run<C, M>,
    case: &'r C,
    tier: Tier,
}

impl<C, M> LawsAlg for CaseCheck<'_, C, M> {
    type Laws = Vec<Verdict>;

    fn laws_empty(&self) -> Self::Laws {
        Vec::new()
    }

    fn laws_merge(&self, mut left: Self::Laws, right: Self::Laws) -> Self::Laws {
        left.extend(right);
        left
    }
}

impl<C: Debug + 'static, M> CaseCheck<'_, C, M> {
    /// One law on a fresh subject and one input; the first break is kept.
    fn once<S, L>(&self, law: &L, input: &L::Input, sampled: bool, verdict: &mut Verdict)
    where
        M: MakeSubject<C, Subject = S>,
        L: LawOp<S, Input: Debug + 'static>,
    {
        verdict.reached += 1;
        let outcome = caught(|| Ok(self.run.make.make_subject(self.case)))
            .and_then(|mut subject| caught(|| law.check(&mut subject, input)));
        if let (Err(failure), None) = (outcome, &verdict.broke) {
            let unit = TypeId::of::<L::Input>() == TypeId::of::<()>();
            verdict.broke = Some(Broke {
                subject: self.run.named.then(|| format!("{:?}", self.case)),
                input: (!unit).then(|| shown(input)),
                sampled,
                failure,
            });
        }
    }
}

/// A checker shows an input (`Debug`) and finds it by type (`'static`), so it checks laws over
/// such inputs only.
impl<S, I, C, M> LawAlg<S, I> for CaseCheck<'_, C, M>
where
    I: Debug + 'static,
    C: Debug + 'static,
    M: MakeSubject<C, Subject = S>,
{
    fn finish_law<L>(&self, suite: &'static str, law: L) -> <Self as LawsAlg>::Laws
    where
        L: LawOp<S, Input = I> + OperationAlg,
    {
        let law = &law;
        let mut verdict = Verdict { suite, law: L::NAME, reached: 0, broke: None };
        let source = self.run.sources.get(&TypeId::of::<L::Input>());
        match source.and_then(|source| source.downcast_ref::<Source<C, L::Input>>()) {
            Some(source) => {
                for pinned in &source.pinned {
                    let each;
                    let inputs = match pinned {
                        Pinned::All(inputs) => inputs,
                        Pinned::Each(inputs) => {
                            each = inputs(self.case);
                            &each
                        }
                    };
                    for input in inputs {
                        if verdict.broke.is_some() {
                            break;
                        }
                        self.once(law, input, false, &mut verdict);
                    }
                }
                for draw in &source.sampled {
                    // A drawn case past the first break adds nothing the report shows.
                    for input in draw(self.tier) {
                        if verdict.broke.is_some() {
                            break;
                        }
                        self.once(law, &input, true, &mut verdict);
                    }
                }
            }
            // A law over the subject alone runs once, on `()`.
            None => {
                if let Some(unit) = (&() as &dyn Any).downcast_ref::<L::Input>() {
                    self.once(law, unit, false, &mut verdict);
                }
            }
        }
        vec![verdict]
    }
}

/// How many characters of an input a verdict shows.
const SHOWN: usize = 160;

/// An input as a verdict shows it: its `Debug` text, cut to [`SHOWN`] characters.
fn shown(input: &impl Debug) -> String {
    let text = format!("{input:?}");
    match text.char_indices().nth(SHOWN) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text,
    }
}

/// One stderr line when `what` runs fewer of its `authored` cases than were written.
fn note_sampled(what: &str, authored: usize, tier: Tier) {
    let n = tier.samples(authored);
    if n < authored {
        eprintln!("SAMPLED {what}: {n} of {authored} cases under {tier:?}; ALUX_TEST_TIER=strict runs them all");
    }
}
