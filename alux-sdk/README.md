# alux-sdk

This crate keeps common Rust transformations short, typed, and composable. It is useful when an
operation should remain an expression—such as traversing fallible values, collecting an exact
shape, converting with an explicit target type, or reifying a trait as inspectable operations.

For the reasoning behind all of this, read [Capability algebras](https://alux-network.github.io/alux-programming/rust-dd/capability-algebras.html) and [First-order programs](https://alux-network.github.io/alux-programming/rust-dd/first-order-programs.html) in the ALUX programming guidelines.

Everything is exported from the crate root:

```rust
use alux_sdk::*;

let values: Result<Vec<_>, ()> = [1, 2, 3]
    .into_iter()
    .traverse(|value| Ok(value * 2));
assert_eq!(values, Ok(vec![2, 4, 6]));

let converted = 42_u8.to::<u64>();
assert_eq!(converted, 42_u64);
```

## Choose a helper

| When you need to | Use |
| --- | --- |
| Compose fallible transformations over an `Option` or iterator | [Traversable](#traversable) |
| Collect fallible values or pairs | `try_collect_vec`, `try_unzip` |
| Require an exact iterator cardinality | `collect_exact` |
| Borrow a slice prefix as an array | `try_to_const`, `try_to_const_mut` |
| Remove one iterator position or stop at an inclusive boundary | `skip_nth`, `stop_if` |
| State the target of an `Into` conversion inline | `to::<Target>()` |
| Define a finite mapping in both directions | `case_mapping!`, `case_mapping_partial!` |
| Reify trait operations as data | [`trait_algebra`](#trait-algebras) |
| Call a trait interpreted somewhere else | [`trait_algebra(proxy)`](#proxies) |

## Traversable

[`alux-traversable`](https://docs.rs/alux-traversable) states `traverse` and `sequence` over `Option` and iterators, sequencing the `Result` effect while preserving the shape and order of what was traversed. Everything it states is re-exported here:

```rust
use alux_sdk::*;

let doubled: Result<Vec<_>, ()> = [1, 2, 3].into_iter().traverse(|value| Ok(value * 2));
assert_eq!(doubled, Ok(vec![2, 4, 6]));

assert_eq!(Some(Ok::<_, ()>(42)).sequence(), Ok(Some(42)));
```

## Trait algebras

Use `trait_algebra` to turn operations into inspectable data with a generated interpreter contract:

```rust
use alux_sdk::trait_algebra;

#[trait_algebra(derive(Debug, PartialEq))]
trait Observation {
    fn observed(&self, value: u64);
}

assert_eq!(ObservationOp::observed(42), ObservationOp::Observed { value: 42 });
```

The trait states the vocabulary and nothing else. The macro re-emits it unchanged and adds four items beside it: the operations as data, the contract that interprets one, and the fold between them.

```rust
// What `#[trait_algebra(derive(Debug, PartialEq))] trait Observation` adds:

/// One `Observation` operation as data.
#[derive(Debug, PartialEq)]
enum ObservationOp {
    /// Holds the arguments of `observed`.
    Observed { value: u64 },
}

/// One reply to an `Observation` operation.
#[derive(Debug, PartialEq)]
enum ObservationReply {
    #[doc(hidden)]
    __Unit,
}

/// Interprets one `Observation` operation at a time.
trait ObservationInterpreter {
    /// Handles the `observed` operation.
    fn observed(&mut self, value: u64);
}

impl ObservationOp {
    /// Constructs the `observed` operation.
    fn observed(value: u64) -> Self {
        Self::Observed { value }
    }

    /// Folds this operation into one interpreter and returns its reply.
    fn interpret<Interpreter>(self, interpreter: &mut Interpreter) -> ObservationReply
    where
        Interpreter: ObservationInterpreter,
    {
        match self {
            Self::Observed { value } => {
                interpreter.observed(value);
                ObservationReply::__Unit
            }
        }
    }
}
```

- **`ObservationOp`** is the operation as a value: one variant per method, one field per argument, and nothing else. No receiver, no return, no reply channel, no transport — which is what makes an operation something you can hold, compare, write down, queue, send somewhere, and interpret later or twice.
- **`ObservationInterpreter`** is the interpreter contract: one method per operation, taking `&mut self` because interpreting one is what changes something, and returning the operation's own value directly.
- **`ObservationOp::interpret`** is the fold, and it is generated rather than written: adding a method to the trait adds a variant, an interpreter method, and the arm between them at once, so a vocabulary and its interpreters cannot drift apart.
- **`ObservationReply`** carries one variant per *returning* method, with `into_<method>` to take the value out. A method returning nothing still folds to something, which is what the hidden unit variant is for; here every method returns nothing, so that is all the reply has.

Associated types are lifted rather than chosen: each one used by an argument becomes a parameter of `<Trait>Op`, each one used by a return becomes a parameter of `<Trait>Reply`, and the interpreter redeclares them — so the vocabulary states its carriers without picking them. Attribute arguments are copied to both enums, and one asynchronous method makes `interpret` asynchronous for every variant.

### Proxies

Add `proxy` and the algebra states a value you can call the trait on, wherever the thing that interprets it happens to live:

```rust
use alux_sdk::{AlgebraCall, trait_algebra};
use tokio::sync::Mutex;

#[trait_algebra(derive(Debug), proxy)]
trait Counter {
    async fn add(&self, amount: u64) -> u64;
}

#[derive(Default)]
struct Total(u64);

impl CounterInterpreter for Total {
    async fn add(&mut self, amount: u64) -> u64 {
        self.0 += amount;
        self.0
    }
}

/// Interprets operations where it stands, which is all it takes to be called.
#[derive(Default)]
struct Here(Mutex<Total>);

impl AlgebraCall<CounterOp, CounterReply> for Here {
    async fn ask(&self, operation: CounterOp) -> Option<CounterReply> {
        Some(operation.interpret(&mut *self.0.lock().await).await)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let counter = CounterProxy::new(Here::default());

    assert_eq!(counter.add(2).await, 2);
    assert_eq!(counter.add(3).await, 5);
}
```

`counter.add(2).await` is the trait and says nothing else. Swap `Here` for one that carries operations to another task, another thread, or a queue, and the call is unchanged — `alux-tokio` states the bounded-channel one, and its README runs this same counter against an interpreter in a spawned task. The same swap is how a test replaces a live interpreter with one that records what it was asked.

What a caller never writes: a message type, a reply channel, a match on which operation came back, or an `unwrap` on any of it.

Every method waits for its operation to be interpreted, because that is what awaiting the trait's own method means where the interpreter is at hand. An interpreter with work to leave running answers first and carries on, so how long a call waits is its to state.

#### How it works

`CounterProxy<Calling>` holds one caller and states the trait's own methods, turning each call into the matching operation, `ask`ing it, and taking the value out of the reply. It is cloneable when its caller is, and is generated only for an algebra whose carriers are all chosen, since a value stating the trait cannot leave a carrier open.

Where the operations go is stated by `AlgebraCall`: one operation stated, one reply received.

```rust
pub trait AlgebraCall<Operation, Reply> {
    /// Sends one operation and waits for the reply it is answered with.
    async fn ask(&self, operation: Operation) -> Option<Reply>;
}
```

Interpreting an operation always states something — the value the method states, or the unit reply of a method that states none — so a caller holding a reply knows the operation ran. Nothing is stated where the interpreter is gone, which is an ordinary ending and not a failure: a method stating no value returns, and one stating a value panics, having promised a value that never came.

A transport may also carry an operation nobody stays for; `alux-tokio` states that on the channel itself, for producers like a progress stream that nobody waits on.
