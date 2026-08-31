//! A proxy states the trait itself, and sends its operations to an interpreter elsewhere.

use alux_sdk::{AlgebraCall, trait_algebra};
use tokio::sync::Mutex;

#[trait_algebra(derive(Debug, PartialEq), proxy)]
trait Counter {
    async fn add(&self, amount: u64) -> u64;
    async fn reset(&self);
}

/// Interprets counting by keeping a running total.
#[derive(Default)]
struct Total(u64);

impl CounterInterpreter for Total {
    async fn add(&mut self, amount: u64) -> u64 {
        self.0 += amount;
        self.0
    }

    async fn reset(&mut self) {
        self.0 = 0;
    }
}

/// Sends operations by interpreting them where they are stated, which is enough to be a sender.
#[derive(Default)]
struct Here(Mutex<Total>);

impl AlgebraCall<CounterOp, CounterReply> for Here {
    async fn ask(&self, operation: CounterOp) -> Option<CounterReply> {
        Some(operation.interpret(&mut *self.0.lock().await).await)
    }
}

/// Sends operations nowhere, which is what a proxy whose interpreter is gone talks to.
struct Nowhere;

impl AlgebraCall<CounterOp, CounterReply> for Nowhere {
    async fn ask(&self, _operation: CounterOp) -> Option<CounterReply> {
        None
    }
}

#[tokio::test]
async fn a_proxy_is_the_trait_it_states() {
    let counter = CounterProxy::new(Here::default());

    // The caller states the trait, and never states where it is interpreted.
    assert_eq!(counter.add(2).await, 2);
    assert_eq!(counter.add(3).await, 5);
    counter.reset().await;
    assert_eq!(counter.add(1).await, 1);
}

#[tokio::test]
async fn a_proxy_is_cloneable_when_its_sender_is() {
    #[derive(Clone)]
    struct Shared;

    impl AlgebraCall<CounterOp, CounterReply> for Shared {
        async fn ask(&self, _operation: CounterOp) -> Option<CounterReply> {
            Some(CounterReply::Add(7))
        }
    }

    let counter = CounterProxy::new(Shared);
    let another = counter.clone();

    assert_eq!(counter.add(1).await, 7);
    assert_eq!(another.add(1).await, 7);
}

#[tokio::test]
async fn a_method_stating_no_value_carries_on_where_nothing_answers() {
    let counter = CounterProxy::new(Nowhere);

    // Nobody was waiting on an outcome, so there is nothing to be told.
    counter.reset().await;
}
