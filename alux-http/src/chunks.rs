//! States a body produced over time, rather than one that is already in hand.

use alux_ext::ext;
use core::future::Future;
use futures::{Stream, stream};

/// States a body produced over time: the chunks it carries, and what taking the next one means.
///
/// A domain answering with a body it cannot hold all at once states this rather than naming a
/// stream type. What a chunk is, what a failure part-way through means, and how bytes actually move
/// are then the interpretation's to choose, which is the whole reason a program states a surface
/// rather than a framework's callbacks.
pub trait ChunksAlg {
    /// What one chunk carries.
    type Chunk;

    /// What a body failing part-way through means.
    type Error;

    /// Takes the next chunk, or nothing where the body has ended.
    fn next_chunk(&mut self) -> impl Future<Output = Option<Result<Self::Chunk, Self::Error>>> + Send;
}

/// The operations a body stated as chunks derives.
#[ext(name = ChunksExt, supertraits = ChunksAlg + Sized)]
pub impl<This> This
where
    This: ChunksAlg + Send + 'static,
    This::Chunk: Send,
{
    /// Reads this body as the sequence of chunks it produces.
    ///
    /// Nothing is converted and nothing is carried: a chunk stays what the domain said it was, and
    /// a failure stays what the domain said it meant. An interpretation states what to make of
    /// either, which is the only part of moving bytes that is its own.
    fn moving(self) -> impl Stream<Item = Result<Self::Chunk, Self::Error>> + Send {
        stream::unfold(self, |mut chunks| async move {
            let taken = chunks.next_chunk().await?;

            Some((taken, chunks))
        })
    }

    /// Takes every chunk this body produces, in the order it produces them.
    ///
    /// A sequence read whole is what anything states that cannot act on a piece at a time: a part's
    /// content read into a value, or an answer compared against what a caller would have received.
    // Written as a future rather than as `async fn`, which cannot state that it is `Send`.
    #[allow(clippy::manual_async_fn)]
    fn gathered(mut self) -> impl Future<Output = Result<Vec<Self::Chunk>, Self::Error>> + Send {
        async move {
            let mut taken = Vec::new();
            while let Some(chunk) = self.next_chunk().await {
                taken.push(chunk?);
            }

            Ok(taken)
        }
    }
}
