//! Calling an interpreter with one algebra operation.
//!
//! Every capability here forwards through `&` and `Arc`, because what a reference or a share stands
//! for is the caller itself. Nothing downstream could add that: naming any of them for a smart
//! pointer is a foreign trait on a foreign type, so it is stated once, here.
//!
//! The two that wait state their waiting as a future that is `Send`, because an interpreter reached
//! through a transport is elsewhere, and what is elsewhere is another task. An `async fn` in a trait
//! states a future nobody can name, so nobody outside could ask for that: it is stated where the
//! capability is, or not at all.

use core::future::Future;
use std::sync::Arc;

/// Calls an interpreter with one algebra operation, and receives what it states back.
///
/// Interpreting an operation always states something — the value the method states, or the unit
/// reply of a method that states none — so a caller holding a reply knows the operation ran.
/// Implementations preserve order among operations called through one clone of a caller.
pub trait AlgebraCall<Operation, Reply> {
    /// Calls one operation and waits for the reply it is answered with.
    ///
    /// How the waiting is done belongs to the caller: an interpreter at hand is awaited where it
    /// stands, and one across a transport is waited on through whatever that transport answers by.
    ///
    /// States no answer when none will ever come. Nothing went wrong: whoever would have answered
    /// is gone, so the waiting is ended rather than left to never end.
    fn ask(&self, operation: Operation) -> impl Future<Output = Option<Reply>> + Send;
}

/// Puts one algebra operation where an interpreter will read it, without staying for an answer.
///
/// A method that states no value has nothing to wait for, so stating it is one act and not two.
/// What comes back is only whether anybody was there to take the operation, never whether it was
/// read or interpreted: an interpreter that is gone states nothing, which is an ordinary ending.
pub trait AlgebraSend<Operation> {
    /// Puts one operation on its way, and states whether anybody was there to take it.
    ///
    /// # Errors
    ///
    /// States nothing when no interpreter remains. What that means belongs to whoever sends: a feed
    /// whose reader is meant to outlive it says so at the call, and one that may outlive its reader
    /// carries on.
    #[must_use = "a feed whose reader is gone states nothing, and only the caller knows what that means"]
    fn send(&self, operation: Operation) -> impl Future<Output = Option<()>> + Send;
}

/// Names the one algebra a carrier carries: its operations, and the replies they are answered with.
///
/// [`AlgebraCall`] and [`AlgebraSend`] are stated for an operation and a reply, so one carrier may
/// witness them for several algebras, and what it carries is read off the capability asked for. An
/// algebra whose carriers are still open has no capability to read them off: its operation and reply
/// are generic over them, and only the carrier knows which it was built for. This states it, so an
/// impl reaching a carrier through the capabilities can bind the algebra's carriers to the carrier's.
///
/// A carrier carries one algebra, so it states one of these; a carrier for several does not.
pub trait AlgebraCarrier {
    /// The operations this carrier puts where an interpreter reads them.
    type Operation;
    /// The replies those operations are answered with.
    type Reply;
}

/// Carries what the reference stands for.
impl<Carrying> AlgebraCarrier for &Carrying
where
    Carrying: AlgebraCarrier + ?Sized,
{
    type Operation = Carrying::Operation;
    type Reply = Carrying::Reply;
}

/// Carries what is shared.
impl<Carrying> AlgebraCarrier for Arc<Carrying>
where
    Carrying: AlgebraCarrier + ?Sized,
{
    type Operation = Carrying::Operation;
    type Reply = Carrying::Reply;
}

/// Calls whatever the reference stands for, so holding a borrow of a caller is holding the caller.
impl<Operation, Reply, Calling> AlgebraCall<Operation, Reply> for &Calling
where
    Calling: AlgebraCall<Operation, Reply> + ?Sized,
{
    fn ask(&self, operation: Operation) -> impl Future<Output = Option<Reply>> + Send {
        (**self).ask(operation)
    }
}

/// Calls whatever is shared, so handing out a share of a caller hands out the caller.
impl<Operation, Reply, Calling> AlgebraCall<Operation, Reply> for Arc<Calling>
where
    Calling: AlgebraCall<Operation, Reply> + ?Sized,
{
    fn ask(&self, operation: Operation) -> impl Future<Output = Option<Reply>> + Send {
        (**self).ask(operation)
    }
}

/// Sends through whatever the reference stands for.
impl<Operation, Sending> AlgebraSend<Operation> for &Sending
where
    Sending: AlgebraSend<Operation> + ?Sized,
{
    fn send(&self, operation: Operation) -> impl Future<Output = Option<()>> + Send {
        (**self).send(operation)
    }
}

/// Sends through whatever is shared.
impl<Operation, Sending> AlgebraSend<Operation> for Arc<Sending>
where
    Sending: AlgebraSend<Operation> + ?Sized,
{
    fn send(&self, operation: Operation) -> impl Future<Output = Option<()>> + Send {
        (**self).send(operation)
    }
}
