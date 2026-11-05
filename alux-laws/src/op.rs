//! One law as an operation: what checking it on a subject and an input means.

use crate::Failure;
use core::fmt::Debug;

/// Checks one law on a subject and an input.
///
/// A law reads its name and documentation from `OperationAlg`; this trait says only how it is
/// applied. Unlike `ApplyAlg` it is sync and borrows its subject, so a law runs on any subject,
/// `Send` or not. A `&self` law is handed `&*subject` by its generated `check`.
pub trait LawOp<S> {
    /// What the law takes beside its subject; `()` for a law over the subject alone.
    type Input;

    /// Checks the law on `subject` and `input`.
    ///
    /// # Errors
    ///
    /// Returns the [`Failure`] the law reports when it does not hold.
    fn check(&self, subject: &mut S, input: &Self::Input) -> Result<(), Failure>;
}

/// What a law method may return: `()`, or `Result<(), E: Debug>`.
pub trait IntoLawOutcome {
    /// Returns the outcome as a law states it.
    ///
    /// # Errors
    ///
    /// Returns a [`Failure`] when the law reported one.
    fn into_law_outcome(self) -> Result<(), Failure>;
}

impl IntoLawOutcome for () {
    fn into_law_outcome(self) -> Result<(), Failure> {
        Ok(())
    }
}

impl<E: Debug> IntoLawOutcome for Result<(), E> {
    fn into_law_outcome(self) -> Result<(), Failure> {
        self.map_err(|e| Failure::new(format!("{e:?}")))
    }
}
