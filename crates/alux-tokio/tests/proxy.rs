//! A proxy states the trait itself, so awaiting one of its methods means the operation was
//! interpreted — including a method that states no value.

use alux_sdk::trait_algebra;
use alux_tokio::bounded_algebra_channel;
use futures::StreamExt;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[trait_algebra(derive(Debug), proxy)]
trait Notebook {
    async fn note(&self, value: u8);
}

/// Interprets noting slowly enough that carrying on early would be visible.
struct Notes(Arc<Mutex<Vec<u8>>>);

impl NotebookInterpreter for Notes {
    async fn note(&mut self, value: u8) {
        tokio::time::sleep(Duration::from_millis(20)).await;
        self.0.lock().unwrap().push(value);
    }
}

fn one() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

#[tokio::test]
async fn a_method_stating_no_value_waits_for_its_operation_to_be_interpreted() {
    let written = Arc::new(Mutex::new(Vec::new()));
    let (sender, mut operations) = bounded_algebra_channel::<NotebookOp, NotebookReply>(one());
    let notebook = NotebookProxy::new(sender);

    tokio::spawn({
        let written = Arc::clone(&written);
        async move {
            let mut notes = Notes(written);
            while let Some(message) = operations.next().await {
                let (operation, responder) = message.into_parts();
                responder.respond(operation.interpret(&mut notes).await);
            }
        }
    });

    notebook.note(1).await;

    // The await stated that noting happened, which is what it means where the interpreter is at
    // hand.
    assert_eq!(*written.lock().unwrap(), vec![1]);
}

#[tokio::test]
async fn a_method_stating_no_value_returns_where_the_interpreter_is_gone() {
    let (sender, operations) = bounded_algebra_channel::<NotebookOp, NotebookReply>(one());
    let notebook = NotebookProxy::new(sender);
    drop(operations);

    // Nothing was promised but the interpreting, and there is nothing to be told about its
    // absence.
    notebook.note(1).await;
}
