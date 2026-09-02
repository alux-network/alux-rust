//! A carrier stating an algebra need not be a transport at all.
//!
//! `transport = <Carrier>` names a type in the caller's own scope, and what the macro emits speaks
//! only [`AlgebraCall`] and [`AlgebraSend`]. So the carrier here holds its interpreter directly:
//! there is no channel, no runtime, and nothing named from `alux-tokio`.

use alux_sdk::{AlgebraCall, AlgebraSend, trait_algebra};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

#[trait_algebra(transport = AtHand)]
trait Counter {
    /// States a value, so a caller asks for it.
    async fn increment(&self, by: u32) -> u32;
    /// States none, so a caller sends and does not stay.
    async fn note(&self, text: String);
}

/// Interprets counting and noting.
#[derive(Default)]
struct Tally {
    sum: u32,
    notes: Vec<String>,
}

impl CounterInterpreter for Tally {
    async fn increment(&mut self, by: u32) -> u32 {
        self.sum += by;
        self.sum
    }

    async fn note(&mut self, text: String) {
        self.notes.push(text);
    }
}

/// A carrier that interprets where it stands, witnessing both capabilities without a transport.
struct AtHand<Operation, Reply> {
    tally: Arc<Mutex<Tally>>,
    stated: PhantomData<fn(Operation) -> Reply>,
}

impl AlgebraCall<CounterOp, CounterReply> for AtHand<CounterOp, CounterReply> {
    async fn ask(&self, operation: CounterOp) -> Option<CounterReply> {
        let mut tally = self.tally.lock().await;

        Some(operation.interpret(&mut *tally).await)
    }
}

impl AlgebraSend<CounterOp> for AtHand<CounterOp, CounterReply> {
    async fn send(&self, operation: CounterOp) -> Option<()> {
        self.ask(operation).await.map(|_stated| ())
    }
}

/// Reads the algebra, knowing nothing of what carries it.
async fn count_through(counter: &impl Counter) -> u32 {
    counter.note("counting".to_owned()).await;

    counter.increment(2).await
}

#[tokio::test]
async fn a_carrier_needs_no_transport_to_state_an_algebra() {
    let tally = Arc::new(Mutex::new(Tally { sum: 40, notes: Vec::new() }));
    let at_hand = AtHand { tally: Arc::clone(&tally), stated: PhantomData };

    assert_eq!(count_through(&at_hand).await, 42);
    assert_eq!(tally.lock().await.notes, ["counting".to_owned()]);
}
