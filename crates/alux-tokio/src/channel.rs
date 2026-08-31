use crate::{AlgebraMessage, BoundedAlgebraSender};
use futures::stream::{AbortHandle, Abortable};
use futures::{Stream, StreamExt};
use std::future::Future;
use std::num::NonZeroUsize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// States every message one bounded algebra channel carries, until it is stopped or ends.
pub type AlgebraStream<Operation, Reply> = Abortable<ReceiverStream<AlgebraMessage<Operation, Reply>>>;

/// Constructs a cloneable sender and its single-owner receiver stream.
///
/// The non-zero capacity makes bounded backpressure part of the value's construction. This
/// function does not spawn a task; the caller decides where and how the returned stream is polled.
pub fn bounded_algebra_channel<Operation, Reply>(
    capacity: NonZeroUsize,
) -> (BoundedAlgebraSender<Operation, Reply>, AlgebraStream<Operation, Reply>) {
    let (sender, receiver) = mpsc::channel(capacity.get());
    let (stop, stopped) = AbortHandle::new_pair();
    (BoundedAlgebraSender::from_sender(sender, stop), Abortable::new(ReceiverStream::new(receiver), stopped))
}

/// Interprets every message one stream states, until the stream ends or is stopped.
///
/// The loop is a convenience, not transport meaning: it belongs to whoever owns the stream, and
/// exists here only for the common case where the stream is the one thing being waited on. A
/// consumer with more than one source drives the stream itself instead.
pub async fn interpret_stream<Message, Apply, Applied>(
    mut stream: impl Stream<Item = Message> + Unpin,
    mut apply: Apply,
) where
    Apply: FnMut(Message) -> Applied,
    Applied: Future<Output = ()>,
{
    while let Some(message) = stream.next().await {
        apply(message).await;
    }
}
