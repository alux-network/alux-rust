//! Validates the bounded algebra channel laws without imposing a consumption loop.

use alux_sdk::AlgebraCall;
use alux_tokio::bounded_algebra_channel;
use futures::{FutureExt, StreamExt};
use std::num::NonZeroUsize;

fn one() -> NonZeroUsize {
    NonZeroUsize::MIN
}

#[tokio::test]
async fn sends_preserve_sender_order() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, ()>(one());

    assert_eq!(sender.send(1).await, Some(()));
    assert_eq!(operations.next().await.unwrap().operation(), &1);
    assert_eq!(sender.send(2).await, Some(()));
    assert_eq!(operations.next().await.unwrap().operation(), &2);
}

#[tokio::test]
async fn a_full_channel_applies_backpressure() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, ()>(one());
    assert_eq!(sender.send(1).await, Some(()));

    let mut second = Box::pin(sender.send(2));
    assert!(second.as_mut().now_or_never().is_none());
    assert_eq!(operations.next().await.unwrap().operation(), &1);
    second.await;
    assert_eq!(operations.next().await.unwrap().operation(), &2);
}

#[tokio::test]
async fn asking_states_the_consumers_answer() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, u16>(one());
    let asker = tokio::spawn(async move { sender.ask(4).await });

    let (operation, responder) = operations.next().await.unwrap().into_parts();
    assert!(responder.is_awaited());
    responder.respond(u16::from(operation) * 10);

    assert_eq!(asker.await.unwrap(), Some(40));
}

#[tokio::test]
async fn sending_where_nobody_reads_states_nothing() {
    let (sender, operations) = bounded_algebra_channel::<String, ()>(one());
    drop(operations);

    // What a lost reader means belongs to the feed, so the sender is told and nothing else.
    assert_eq!(sender.send("kept".to_owned()).await, None);
}

#[tokio::test]
async fn sending_states_that_something_was_there_to_take_it() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, ()>(one());

    // Somebody was there to take it; what it comes to, if anything, is not stated here.
    assert_eq!(sender.send(1).await, Some(()));
    assert_eq!(operations.next().await.unwrap().operation(), &1);
}

#[tokio::test]
async fn asking_where_nobody_answers_states_no_answer() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, u16>(one());
    let asker = tokio::spawn(async move { sender.ask(4).await });
    drop(operations.next().await.unwrap());

    assert_eq!(asker.await.unwrap(), None);
}

#[tokio::test]
async fn answering_an_asker_that_is_gone_is_not_a_failure() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, u16>(one());
    let asker = tokio::spawn(async move { sender.ask(4).await });
    let (_, responder) = operations.next().await.unwrap().into_parts();
    asker.abort();
    let _ended = asker.await;

    responder.respond(40);
}

#[tokio::test]
async fn an_operation_nobody_stayed_for_is_answered_all_the_same() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, u16>(one());
    assert_eq!(sender.send(4).await, Some(()));

    // Every operation carries the name it is answered on; the sender simply did not stay.
    let (_, responder) = operations.next().await.unwrap().into_parts();
    assert!(!responder.is_awaited());
    responder.respond(40);
}

#[tokio::test]
async fn stopping_ends_the_stream_where_it_is() {
    let (sender, mut operations) = bounded_algebra_channel::<u8, ()>(one());
    assert_eq!(sender.send(1).await, Some(()));
    sender.stop();

    // What the stream had not stated yet is never stated.
    assert!(operations.next().await.is_none());
}

#[tokio::test]
async fn a_stream_can_be_interpreted_until_it_ends() {
    let (sender, operations) = bounded_algebra_channel::<u8, ()>(one());
    let stated = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    let seen = std::sync::Arc::clone(&stated);
    let consumer = tokio::spawn(alux_tokio::interpret_stream(operations, move |message| {
        let seen = std::sync::Arc::clone(&seen);
        async move {
            seen.lock().unwrap().push(*message.operation());
        }
    }));

    assert_eq!(sender.send(1).await, Some(()));
    assert_eq!(sender.send(2).await, Some(()));
    // The stream ends when nothing can put anything else onto it, and what was put is stated.
    drop(sender);
    consumer.await.unwrap();

    assert_eq!(*stated.lock().unwrap(), vec![1, 2]);
}
