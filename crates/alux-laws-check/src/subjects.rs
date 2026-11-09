//! Where a run starts: a maker of one subject, or cases each making their own.

use crate::{CaseCheck, Fresh, Run};
use alux_ext::ext;
use alux_laws::{LawsProgramAlg, Verdict};

/// Checks laws on the subjects a maker builds, a fresh one per check:
/// `Good::default.obeys(counter_laws())`.
#[ext(name = SubjectLawsExt)]
pub impl<This, S> This
where
    This: Fn() -> S,
{
    /// Starts a run on this maker's subjects, to give it inputs or a tier before it checks.
    fn check(self) -> Run<(), Fresh<This>> {
        Run::new(vec![()], Fresh(self), false)
    }

    /// Checks that every law of `laws` holds on this maker's subjects; see [`Run::obeys`].
    fn obeys<L>(self, laws: L)
    where
        L: for<'r> LawsProgramAlg<CaseCheck<'r, (), Fresh<This>>, Laws = Vec<Verdict>>,
    {
        self.check().obeys(laws);
    }

    /// Checks that `laws` catch this maker's subjects, built wrong on purpose; see
    /// [`Run::caught_by`].
    fn caught_by<L>(self, laws: L)
    where
        L: for<'r> LawsProgramAlg<CaseCheck<'r, (), Fresh<This>>, Laws = Vec<Verdict>>,
    {
        self.check().caught_by(laws);
    }

    /// Returns one verdict per law of `laws`; see [`Run::verdicts`].
    fn verdicts<L>(self, laws: &L) -> Vec<Verdict>
    where
        L: for<'r> LawsProgramAlg<CaseCheck<'r, (), Fresh<This>>, Laws = Vec<Verdict>>,
    {
        self.check().verdicts(laws)
    }
}

/// Checks laws on a subject built from each case: a law holds when it holds on every one, and a
/// break names the case it broke on.
#[ext(name = CaseLawsExt)]
pub impl<C> Vec<C> {
    /// Starts a run on a fresh subject per case, built by `make`.
    fn check_each<M, S>(self, make: M) -> Run<C, M>
    where
        M: Fn(&C) -> S,
    {
        Run::new(self, make, true)
    }
}
