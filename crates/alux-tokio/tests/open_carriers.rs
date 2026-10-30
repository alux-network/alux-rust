//! One channel carries one algebra, so a sender fixes the carriers an open algebra leaves open.
//!
//! `Registry` declares its peer as an associated type, and bare `transport` states it for whatever
//! carries `RegistryOp<Peer>` and `RegistryReply`. `BoundedAlgebraSender` names what it carries, so
//! the sender of a channel of `RegistryOp<u32>` is the registry of `u32` peers.

use alux_sdk::trait_algebra;
use alux_tokio::bounded_algebra_channel;
use futures::StreamExt;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

#[trait_algebra(derive(Debug), transport)]
trait Registry {
    /// Who is admitted.
    type Peer: Send;
    /// States a value, so a caller asks and waits for it.
    async fn admit(&self, peer: Self::Peer) -> usize;
    /// States none, so a caller sends and does not stay.
    async fn greet(&self, peer: Self::Peer);
}

/// Interprets admitting and greeting `u32` peers.
struct Admitted {
    peers: Vec<u32>,
    greeted: Arc<Mutex<Vec<u32>>>,
}

impl RegistryInterpreter for Admitted {
    type Peer = u32;

    async fn admit(&mut self, peer: u32) -> usize {
        self.peers.push(peer);
        self.peers.len()
    }

    async fn greet(&mut self, peer: u32) {
        self.greeted.lock().unwrap().push(peer);
    }
}

/// Reads the algebra, naming the peer it reads in and nothing of how it reaches an interpreter.
async fn admit_through(registry: &impl Registry<Peer = u32>) -> usize {
    registry.greet(3).await;

    registry.admit(3).await
}

#[tokio::test]
async fn a_sender_states_the_algebra_for_the_carriers_its_channel_carries() {
    let greeted = Arc::new(Mutex::new(Vec::new()));
    let (sender, mut operations) =
        bounded_algebra_channel::<RegistryOp<u32>, RegistryReply>(NonZeroUsize::new(8).unwrap());

    let interpreting = tokio::spawn({
        let greeted = Arc::clone(&greeted);
        async move {
            let mut admitted = Admitted { peers: vec![1, 2], greeted };
            while let Some(message) = operations.next().await {
                let (operation, responder) = message.into_parts();
                responder.respond(operation.interpret(&mut admitted).await);
            }
        }
    });

    assert_eq!(admit_through(&sender).await, 3);
    assert_eq!(admit_through(&Arc::new(sender.clone())).await, 4);

    sender.stop();
    interpreting.await.unwrap();
    assert_eq!(&*greeted.lock().unwrap(), &[3, 3]);
}
