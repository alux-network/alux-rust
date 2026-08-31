//! Calling an interpreter with one algebra operation.

/// Calls an interpreter with one algebra operation, and receives what it states back.
///
/// Interpreting an operation always states something — the value the method states, or the unit
/// reply of a method that states none — so a caller holding a reply knows the operation ran.
/// Implementations preserve order among operations called through one clone of a caller.
#[allow(async_fn_in_trait)]
pub trait AlgebraCall<Operation, Reply> {
    /// Calls one operation and waits for the reply it is answered with.
    ///
    /// How the waiting is done belongs to the caller: an interpreter at hand is awaited where it
    /// stands, and one across a transport is waited on through whatever that transport answers by.
    ///
    /// States no answer when none will ever come. Nothing went wrong: whoever would have answered
    /// is gone, so the waiting is ended rather than left to never end.
    async fn ask(&self, operation: Operation) -> Option<Reply>;
}
