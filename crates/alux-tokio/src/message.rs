use tokio::sync::oneshot;

/// Carries one operation together with the name it is answered on.
#[derive(Debug)]
pub struct AlgebraMessage<Operation, Reply> {
    operation: Operation,
    responder: AlgebraResponder<Reply>,
}

/// Carries the name one operation is answered on.
///
/// Every operation is answered, because interpreting one always states something — the value the
/// method states, or the unit reply of a method that states none. Whether anybody was ever there
/// to hear it is the sender's business and not the message's: an operation stated to a feed
/// carries no name to answer on, and answering it costs nothing.
#[derive(Debug)]
pub struct AlgebraResponder<Reply>(Option<oneshot::Sender<Reply>>);

impl<Operation, Reply> AlgebraMessage<Operation, Reply> {
    /// Carries one operation together with the name it is answered on.
    pub const fn new(operation: Operation, answering: oneshot::Sender<Reply>) -> Self {
        Self { operation, responder: AlgebraResponder(Some(answering)) }
    }

    /// Carries one operation nobody stays for.
    pub(crate) const fn unheard(operation: Operation) -> Self {
        Self { operation, responder: AlgebraResponder(None) }
    }

    /// Borrows the operation without consuming its reply capability.
    pub const fn operation(&self) -> &Operation {
        &self.operation
    }

    /// Separates the pure operation from the capability that answers it.
    pub fn into_parts(self) -> (Operation, AlgebraResponder<Reply>) {
        (self.operation, self.responder)
    }
}

impl<Reply> AlgebraResponder<Reply> {
    /// Answers on the name the operation carried.
    ///
    /// An answer nobody waits for is never received, which is the ordinary end of an operation
    /// sent by someone who did not stay for it, and of one whose asker has since gone. Neither is
    /// a failure of this program.
    pub fn respond(self, reply: Reply) {
        if let Some(answering) = self.0 {
            let _unheard = answering.send(reply);
        }
    }

    /// States whether anybody is still waiting on this answer.
    pub fn is_awaited(&self) -> bool {
        self.0.as_ref().is_some_and(|answering| !answering.is_closed())
    }
}
