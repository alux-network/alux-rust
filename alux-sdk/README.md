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

`CounterProxy<Calling>` holds one caller and states the trait's own methods, turning each call into the matching operation, `ask`ing it, and taking the value out of the reply. It is cloneable when its caller is. For an algebra whose carriers are still open, the proxy states the trait for whatever carriers its caller carries, read off [`AlgebraCarrier`](#open-carriers).

Where the operations go is stated by `AlgebraCall`: one operation stated, one reply received.

```rust
pub trait AlgebraCall<Operation, Reply> {
    /// Sends one operation and waits for the reply it is answered with.
    fn ask(&self, operation: Operation) -> impl Future<Output = Option<Reply>> + Send;
}
```

The waiting is stated as a `Send` future, because an interpreter reached this way is elsewhere, and what is elsewhere is another task. An `async fn` in a trait states a future nobody can name, so nobody outside the capability could ask for that: it is stated where the capability is, or not at all.

Interpreting an operation always states something — the value the method states, or the unit reply of a method that states none — so a caller holding a reply knows the operation ran. Nothing is stated where the interpreter is gone, which is an ordinary ending and not a failure: a method stating no value returns, and one stating a value panics, having promised a value that never came.

A transport may also carry an operation nobody stays for; `alux-tokio` states that on the channel itself, for producers like a progress stream that nobody waits on.

### Transports

A proxy is a value that *has* the trait. `transport` goes one step further and states that a carrier *is* the trait, so a caller holds the carrier and calls the vocabulary on it directly. A method stating a value asks and takes the value out of the reply; a method stating none sends and does not stay.

Two spellings, differing only in who is allowed to name a type:

```rust,ignore
// The crate that owns the carrier states the impl, and names it.
#[trait_algebra(transport = BoundedAlgebraSender)]
trait Counter { /* … */ }

// The crate that declares the trait states the impl, and names nothing.
#[trait_algebra(transport)]
trait Counter { /* … */ }
```

The first states `impl Counter for BoundedAlgebraSender<CounterOp, CounterReply>`, resolving the carrier in your own scope. The second names no type at all:

```rust,ignore
impl<Carrier> Counter for Carrier
where
    Carrier: AlgebraCall<CounterOp, CounterReply> + AlgebraSend<CounterOp> + Send + Sync,
```

This is what a layer whose job is *stating* capabilities needs, because the orphan rule puts `impl Counter for <carrier>` in the carrier's crate or the trait's, and a contract layer that must not know a transport has only the second option.

Only the capabilities the algebra asks for appear: `AlgebraCall` where a method states a value, `AlgebraSend` where one states none. `Send + Sync` appears because the impl calls the carrier through a reference from wherever the algebra is held, and an algebra whose futures are `Send` cannot be stated by a carrier that is not. That is also what makes this spelling work under `#[trait_variant::make(Send)]`, which is how a vocabulary says its calls may be awaited in another task.

Both capabilities forward through `&` and `Arc`, so a borrow and a share of a carrier are the algebra too. That is why the trait needs no `auto_impl` of its own — and why it must not carry one, since a blanket impl and `#[auto_impl(&, Arc)]` on the same trait conflict. A local type that forwards the capabilities is the algebra by the same impl; one that states the trait itself still may, because it witnesses no capability and nothing downstream can make it.

Either spelling requires an algebra whose methods are all asynchronous, since a carrier that *is* the trait reaches an interpreter elsewhere.

#### Open carriers

An algebra may leave its carriers open as associated types, and a transport still states it — for every choice of them at once. Each associated type becomes a parameter of the impl, bounded as the trait declares it (on the associated type, or in the trait's where clause), and the impl defines the associated type as that parameter. A caller names the carriers it reads in and nothing of what carries them:

```rust,ignore
#[trait_algebra(transport)]
trait Registry {
    type Peer: Send;
    async fn admit(&self, peer: Self::Peer) -> usize;
}

async fn admit_through(registry: &impl Registry<Peer = u32>) -> usize { registry.admit(3).await }
```

`transport = <Carrier>` reads the parameters off the carrier's own type, as `impl<Peer> Registry for Carrier<RegistryOp<Peer>, RegistryReply>`, and is conditioned on the capabilities that carrier witnesses: one witnessing them only for some choices, as `BoundedAlgebraSender` does only for `Send` ones, restricts the impl and leaves the algebra's bounds alone. Bare `transport` is headed by a type parameter, and a type parameter fixes nothing, since one carrier may witness `AlgebraCall` for several algebras. So it reads them off `AlgebraCarrier`, which names the one operation and reply type a carrier carries:

```rust,ignore
pub trait AlgebraCarrier {
    type Operation;
    type Reply;
}

impl<Carrier, Peer> Registry for Carrier
where
    Carrier: AlgebraCarrier<Operation = RegistryOp<Peer>, Reply = RegistryReply>
        + AlgebraCall<RegistryOp<Peer>, RegistryReply>
        + Send
        + Sync,
    Peer: Send,
{
    type Peer = Peer;
    // …
}
```

`AlgebraCarrier` forwards through `&` and `Arc` like the other capabilities, and `alux-tokio`'s sender states it for the algebra its channel carries. An algebra whose carriers are all chosen asks for none, so its impl is what it was. A carrier is refused where no impl could state it: one occurring in no argument and no return, one with parameters or a where clause of its own, and one named like what the impl is stated for (`Carrier`, the named carrier, or `Calling` for a proxy).
