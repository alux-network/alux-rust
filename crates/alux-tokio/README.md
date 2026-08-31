# alux-tokio

This crate interprets algebra delivery with bounded Tokio channels. It owns **transport mechanics**, not domain operations and not the consumer's event loop. For the reasoning behind carrying operations rather than calls, read [Interpreters](https://alux-network.github.io/alux-programming/rust-dd/interpreters.html) and [First-order programs](https://alux-network.github.io/alux-programming/rust-dd/first-order-programs.html) in the ALUX programming guidelines.

A sequential program says all of this with one syntax. `service.add(2)` is at once the operation, its delivery, the waiting, and the effect, and nothing distinguishes them because nothing can come between them. Once the interpreter is elsewhere they come apart, and each is stated separately here: **the operation** is a value, **sending it** is one act, **waiting for an answer** is another, and **the loop that interprets** belongs to whoever owns the receiving end.

## Sending an operation

A channel is one name with two ends: operations are put on it at one end and stated at the other, **in the order they were put**. What may be put on it is one algebra's operations, and what comes back is that algebra's replies. The channel adds nothing to that vocabulary and understands none of it.

Every operation goes onto the channel carrying **the name it is answered on**. Interpreting one always states something back — the value the method states, or the unit reply of a method that states none — so an operation and its reply are the two halves of one act, which is the capability [`alux-sdk`](https://docs.rs/alux-sdk#trait-algebras) states and this crate interprets:

```rust
pub trait AlgebraCall<Operation, Reply> {
    /// Sends one operation and waits for the reply it is answered with.
    async fn ask(&self, operation: Operation) -> Option<Reply>;
}
```

A reply is the interpreter's own value, and only interpreting the operation produces one, so holding a reply is **evidence the operation ran** — including the unit reply of a method that states no value. Nothing is stated once the interpreter is gone, which is an ordinary ending and not a failure.

`BoundedAlgebraSender` also carries an operation **nobody stays for**: `send` puts it on the stream and returns, and nothing an interpreter states comes back. It states whether anybody was there to take it, and what a lost reader means is left to the feed — one whose reader is meant to outlive it says so at the call.

At the other end, each message carries the operation and the name it is answered on, which `into_parts` separates. An interpreter always has somewhere to state its reply. Answering where nobody stayed discards the answer, as does answering an asker who has since gone.

## Bounded channels

`bounded_algebra_channel` interprets that capability with a Tokio channel of a stated capacity, so backpressure is part of the value's construction: a full channel holds the sender until there is room. The receiving end is returned as a `ReceiverStream`, so the application retains control over **composition**, **scheduling**, **observation**, **recording**, and **termination**.

A program stated in a [trait algebra's](https://docs.rs/alux-sdk#trait-algebras) vocabulary can then run against an interpreter in another task, with the channel carrying every step across:

```rust
use alux_ext::ext;
use alux_sdk::trait_algebra;
use alux_tokio::bounded_algebra_channel;
use futures::StreamExt;
use std::num::NonZeroUsize;

#[trait_algebra(derive(Debug), proxy)]
trait Counter {
    async fn add(&self, amount: u64) -> u64;
    async fn reset(&self);
}

/// A program stated in that vocabulary: start from nothing, and count three amounts.
#[ext(name = CounterProgram)]
pub impl<This> This
where
    This: Counter,
{
    async fn tally(&self) -> u64 {
        self.add(20).await;
        self.reset().await;
        self.add(2).await;
        self.add(3).await;
        self.add(5).await
    }
}

/// One interpreter of it: a running total.
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (sender, mut operations) =
        bounded_algebra_channel::<CounterOp, CounterReply>(NonZeroUsize::new(8).unwrap());
    let counter = CounterProxy::new(sender);

    // Interpreting a stream is a fold: what the operations are interpreted into is one running
    // total, and what interpreting them leaves behind is the total after each one — which the
    // algebra never mentions and the program never asked for.
    let history = tokio::spawn(operations.fold(
        (Total::default(), Vec::new()),
        |(mut total, mut totals), message| async move {
            let (operation, responder) = message.into_parts();
            // Here is where the operation happens at all: the fold picks the matching `Total`
            // method, runs it, and states what it returned as the reply the caller waits on.
            let reply = operation.interpret(&mut total).await;

            totals.push(total.0);
            responder.respond(reply);

            (total, totals)
        },
    ));

    // The program states what it wants counted, and reads what it comes to.
    assert_eq!(counter.tally().await, 10);

    // Letting the last sender go ends the stream, and the fold states what it saw.
    drop(counter);
    let (_, totals) = history.await.unwrap();

    assert_eq!(totals, [20, 0, 2, 5, 10]);
}
```


Two readers, one run. `tally` is written against `Counter` and never learns that its calls cross a channel — it states four steps and reads one number, `10`. The interpreter sees the same four steps as values and folds them into something the program never asked for: `[0, 2, 5, 10]`, the total after each one. Neither knows what the other is doing, because the operation is a value in between them.

The loop that repeatedly receives and folds operations is intentionally absent. **A consuming loop is a terminal interpretation, not transport meaning.** Tests, debugging tools, examples, and simple leaf applications may write such a loop explicitly; reusable libraries should return the stream so their callers can merge or transform it. `interpret_stream` writes the loop for the common case where the stream is the only thing being waited on.

## Proxies

`CounterProxy` above *is* a `Counter`: an algebra asked for a [`proxy`](https://docs.rs/alux-sdk#proxies) states a value that holds one sender and answers the trait's own methods, so `tally` names no operation and no channel.

**Every call waits for the interpreter to run it**, which is what awaiting these signatures means where the interpreter is at hand — `add` is handed the total, and `reset` the evidence that clearing happened, which is why `tally` can read its result from the last call. An interpreter with work to leave running answers first and carries on, so how long a call waits is its to state.

The proxy is cloneable when its sender is, and `BoundedAlgebraSender` is, so many callers can state one vocabulary against one interpreter. It is generated only for an algebra whose carriers are all chosen, since a value stating the trait cannot leave a carrier open.

A proxy never carries an operation nobody stays for. A caller wanting one gone and forgotten holds `BoundedAlgebraSender` itself and calls `send`.

## Laws

- The bounded capacity applies backpressure: a send waits while the buffer is full.
- Operations accepted from one sender are observed in send order.
- Sending where nothing reads the stream any more states nothing, leaving it to the caller to say what that means.
- Sending states that something was there to take the operation, never that it was read or interpreted: one taken and then left unread is as unheard as one nothing took.
- Every operation carries the name it is answered on, so interpreting one always has somewhere to state its reply.
- Answering where the sender did not stay, or where the asker has gone, discards the answer. Neither is a failure of this program.
- Asking states no answer when none will ever come: the waiting is ended rather than never ending.
- The adapter never spawns a task and never consumes the receiver stream.
- A proxy waits for each of its operations to be interpreted, whether or not the method it states returns a value.
- A proxy holding a sender to a stopped or ended stream returns from a method stating no value, and panics from a method stating one: nothing was promised in the first case, and in the second there is no value to state and none to stand in for it.
