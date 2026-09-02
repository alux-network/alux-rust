use crate::AlgebraMessage;
use alux_sdk::{AlgebraCall, AlgebraSend};
use futures::stream::AbortHandle;
use std::fmt::{self, Debug, Formatter};
use tokio::sync::{mpsc, oneshot};

/// Sends operations into one bounded Tokio algebra channel.
pub struct BoundedAlgebraSender<Operation, Reply> {
    sender: mpsc::Sender<AlgebraMessage<Operation, Reply>>,
    stop: AbortHandle,
}

impl<Operation, Reply> BoundedAlgebraSender<Operation, Reply> {
    pub(crate) const fn from_sender(sender: mpsc::Sender<AlgebraMessage<Operation, Reply>>, stop: AbortHandle) -> Self {
        Self { sender, stop }
    }

    /// Puts one message onto the transport, and states whether anything was there to take it.
    ///
    /// Waiting here is backpressure: a bounded transport with no room holds the sender until
    /// there is room. A closed transport is an ordinary state and not a failure — it is what a
    /// finished interpreter leaves behind — and the message goes nowhere, which is what becomes
    /// of any message nobody is listening for. What was taken may still go unread, so this
    /// answers whether there was anybody to send to and not whether the message was heard.
    pub async fn put(&self, message: AlgebraMessage<Operation, Reply>) -> Option<()> {
        self.sender.send(message).await.ok()
    }

    /// Sends one operation without staying for its answer, and states whether anyone was there.
    ///
    /// Nothing an interpreter states comes back this way, not even that the operation was read.
    /// Carrying an operation nobody stays for is this transport's own doing and no part of
    /// [`AlgebraCall`], where stating an operation means receiving what the interpreter states.
    ///
    /// Nothing is stated where the stream has no reader left. What that means is the feed's own
    /// business, and is stated where the feed is: one whose reader is meant to outlive it says
    /// so, and one that may outlive its reader carries on.
    #[must_use = "a feed whose reader is gone states nothing, and only the caller knows what that means"]
    pub async fn send(&self, operation: Operation) -> Option<()> {
        self.put(AlgebraMessage::unheard(operation)).await
    }

    /// Ends the stream, so what it has not yet stated is never stated.
    ///
    /// The stream ends where it is, and operations already put onto the transport and not yet
    /// read are never read. Nothing else is interrupted: an interpreter part-way through one
    /// operation finishes it.
    pub fn stop(&self) {
        self.stop.abort();
    }
}

impl<Operation, Reply> Clone for BoundedAlgebraSender<Operation, Reply> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone(), stop: self.stop.clone() }
    }
}

impl<Operation, Reply> Debug for BoundedAlgebraSender<Operation, Reply> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("BoundedAlgebraSender").finish_non_exhaustive()
    }
}

impl<Operation, Reply> AlgebraSend<Operation> for BoundedAlgebraSender<Operation, Reply> {
    async fn send(&self, operation: Operation) -> Option<()> {
        self.put(AlgebraMessage::unheard(operation)).await
    }
}

impl<Operation, Reply> AlgebraCall<Operation, Reply> for BoundedAlgebraSender<Operation, Reply> {
    async fn ask(&self, operation: Operation) -> Option<Reply> {
        let (answering, answer) = oneshot::channel();
        self.put(AlgebraMessage::new(operation, answering)).await;
        answer.await.ok()
    }
}
