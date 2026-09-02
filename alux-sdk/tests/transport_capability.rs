//! Bare `transport` names no carrier, so the crate declaring an algebra can state its transport.
//!
//! What the macro emits is headed by the capabilities rather than by a type, so every value that
//! witnesses [`AlgebraCall`] and [`AlgebraSend`] is the algebra: the carrier itself, a borrow of it,
//! a share of it, and a local type that forwards to one. The trait's own crate states this while
//! naming no transport at all, which is the shape a layer whose job is stating capabilities needs.
//!
//! Every call it states can be waited on in another task, which is what an algebra reached through
//! a transport is for, and what the capabilities' `Send` futures are stated for.

use alux_sdk::{AlgebraCall, AlgebraSend, trait_algebra};
use std::sync::Arc;
use tokio::sync::Mutex;

#[trait_algebra(transport)]
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

/// A carrier that interprets where it stands, witnessing both capabilities.
struct AtHand {
    tally: Arc<Mutex<Tally>>,
}

impl AlgebraCall<CounterOp, CounterReply> for AtHand {
    async fn ask(&self, operation: CounterOp) -> Option<CounterReply> {
        let mut tally = self.tally.lock().await;

        Some(operation.interpret(&mut *tally).await)
    }
}

impl AlgebraSend<CounterOp> for AtHand {
    async fn send(&self, operation: CounterOp) -> Option<()> {
        self.ask(operation).await.map(|_stated| ())
    }
}

/// A local type that carries nothing itself, and states the capabilities by forwarding.
struct Delegating {
    calling: Arc<AtHand>,
}

impl AlgebraCall<CounterOp, CounterReply> for Delegating {
    async fn ask(&self, operation: CounterOp) -> Option<CounterReply> {
        self.calling.ask(operation).await
    }
}

impl AlgebraSend<CounterOp> for Delegating {
    async fn send(&self, operation: CounterOp) -> Option<()> {
        self.calling.send(operation).await
    }
}

/// Reads the algebra, knowing nothing of what carries it.
async fn count_through(counter: impl Counter) -> u32 {
    counter.note("counting".to_owned()).await;

    counter.increment(2).await
}

#[tokio::test]
async fn every_witness_of_the_capabilities_is_the_algebra() {
    let tally = Arc::new(Mutex::new(Tally::default()));
    let at_hand = Arc::new(AtHand { tally: Arc::clone(&tally) });
    let delegating = Delegating { calling: Arc::clone(&at_hand) };

    assert_eq!(count_through(AtHand { tally: Arc::clone(&tally) }).await, 2);
    assert_eq!(count_through(&*at_hand).await, 4);
    assert_eq!(count_through(Arc::clone(&at_hand)).await, 6);
    assert_eq!(count_through(delegating).await, 8);
    assert_eq!(tally.lock().await.notes.len(), 4);
}

#[tokio::test]
async fn the_algebra_it_states_can_be_waited_on_elsewhere() {
    let tally = Arc::new(Mutex::new(Tally::default()));
    let at_hand = Arc::new(AtHand { tally: Arc::clone(&tally) });

    let counted = tokio::spawn(async move { count_through(at_hand).await });

    assert_eq!(counted.await.expect("the task counting is done"), 2);
}
