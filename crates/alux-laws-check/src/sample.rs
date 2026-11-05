//! Sampled inputs and subjects, drawn by proptest from a fixed seed.

use crate::run::Run;
use crate::tier::Tier;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use std::fmt::Debug;

/// How many cases a sampled source authors unless it says otherwise.
const AUTHORED: usize = 64;

impl<C: 'static, M> Run<C, M> {
    /// Every law over `I` also runs on inputs drawn from `strategy`: 64 of them under
    /// [`Tier::Strict`], one under [`Tier::Dev`].
    #[must_use]
    pub fn sampled<I: Debug + 'static>(self, strategy: impl Strategy<Value = I> + 'static) -> Self {
        self.sampled_cases(strategy, AUTHORED)
    }

    /// As [`sampled`](Self::sampled), with `cases` (at least one) authored for the strict tier.
    ///
    /// # Panics
    ///
    /// Panics when `cases` is zero.
    #[must_use]
    pub fn sampled_cases<I: Debug + 'static>(
        mut self,
        strategy: impl Strategy<Value = I> + 'static,
        cases: usize,
    ) -> Self {
        assert!(cases > 0, "a sampled source authors at least one case");
        self.add_sampled(cases, Box::new(move |tier: Tier| draw(&strategy, tier.samples(cases))));
        self
    }

    /// The run also takes subjects built from cases drawn from `strategy`.
    ///
    /// # Panics
    ///
    /// Panics when the strategy draws nothing.
    #[must_use]
    pub fn sampled_subjects(self, strategy: impl Strategy<Value = C>, tier: Tier) -> Self {
        self.sampled_subject_cases(strategy, AUTHORED, tier)
    }

    /// As [`sampled_subjects`](Self::sampled_subjects), with `cases` authored for the strict tier.
    ///
    /// Cases are drawn here, so the tier is given here and not read from the run.
    ///
    /// # Panics
    ///
    /// Panics when `cases` is zero or the strategy draws nothing.
    #[must_use]
    pub fn sampled_subject_cases(mut self, strategy: impl Strategy<Value = C>, cases: usize, tier: Tier) -> Self {
        assert!(cases > 0, "a subject set samples at least one case");
        let drawn = draw(&strategy, tier.samples(cases));
        self.add_cases(cases, drawn);
        self
    }
}

/// `n` values of `strategy` from a fixed seed. Panics when the strategy draws nothing.
fn draw<T>(strategy: &impl Strategy<Value = T>, n: usize) -> Vec<T> {
    let mut runner = TestRunner::deterministic();
    let one = |_| {
        let tree = strategy.new_tree(&mut runner);
        tree.unwrap_or_else(|why| panic!("the strategy drew nothing: {why}")).current()
    };
    (0..n).map(one).collect()
}
