//! A carrier states the trait, so a caller holds the algebra rather than the channel.
//!
//! `transport = BoundedAlgebraSender` names the carrier in this file's own scope, so what
//! `trait_algebra` emits mentions no transport: it speaks `AlgebraCall` for a method that states a
//! value and `AlgebraSend` for one that states none.

use alux_sdk::trait_algebra;
use alux_tokio::{BoundedAlgebraSender, bounded_algebra_channel};
use futures::StreamExt;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

#[trait_algebra(derive(Debug), transport = BoundedAlgebraSender)]
trait Counter {
    /// States a value, so a caller asks and waits for it.
    async fn increment(&self, by: u32) -> u32;
    /// States none, so a caller sends and does not stay.
    async fn note(&self, text: String);
}

/// Interprets counting and noting.
struct Tally {
    sum: u32,
    notes: Arc<Mutex<Vec<String>>>,
}

impl CounterInterpreter for Tally {
    async fn increment(&mut self, by: u32) -> u32 {
        self.sum += by;
        self.sum
    }

    async fn note(&mut self, text: String) {
        self.notes.lock().unwrap().push(text);
    }
}

/// Reads the algebra, knowing nothing of how it reaches an interpreter.
async fn count_through(counter: &impl Counter) -> u32 {
    counter.note("counting".to_owned()).await;

    counter.increment(2).await
}

#[tokio::test]
async fn a_carrier_states_the_trait_it_carries() {
    let notes = Arc::new(Mutex::new(Vec::new()));
    let (sender, mut operations) = bounded_algebra_channel::<CounterOp, CounterReply>(NonZeroUsize::new(8).unwrap());

    let interpreting = tokio::spawn({
        let notes = Arc::clone(&notes);
        async move {
            let mut tally = Tally { sum: 40, notes };
            while let Some(message) = operations.next().await {
                let (operation, responder) = message.into_parts();
                responder.respond(operation.interpret(&mut tally).await);
            }
        }
    });

    // The sender is the algebra here. Nothing below names a channel, an operation, or a reply.
    assert_eq!(count_through(&sender).await, 42);

    sender.stop();
    interpreting.await.unwrap();
    assert_eq!(&*notes.lock().unwrap(), &["counting".to_owned()]);
}
