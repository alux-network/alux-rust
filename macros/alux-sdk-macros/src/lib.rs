#![doc = include_str!("../README.md")]

mod trait_algebra;

use crate::trait_algebra::trait_algebra_internal;
use proc_macro::TokenStream;
use syn::{ItemTrait, parse_macro_input};

/// Reifies a trait as pure operation data and its fold.
///
/// In addition to re-emitting the annotated trait, the macro generates:
///
/// - `<Trait>Op`, with one inspectable variant per method and typed constructors;
/// - `<Trait>Reply`, with one variant per returning method and typed `into_<method>` accessors;
/// - `<Trait>Interpreter`, the mutable interpreter contract;
/// - `<Trait>Op::interpret`, which folds one operation into an interpreter and returns its reply.
///
/// The generated operation enum has one `call` signature shared by every variant. That signature is
/// synchronous only when every algebra method is synchronous. If any method is asynchronous, the
/// shared `call` function is asynchronous for all variants, although synchronous handler branches
/// still execute directly without awaiting.
///
/// Operations contain only method arguments. They do not contain an interpreter, return value, reply
/// channel, Tokio type, or any other transport decision.
///
/// Attribute arguments are copied to both generated enums. For example,
/// `#[trait_algebra(derive(Debug, Clone))]` derives `Debug` and `Clone` for both `<Trait>Op` and
/// `<Trait>Reply`.
///
/// Associated types used by method arguments become generic parameters of `<Trait>Op`. Associated
/// types used by return values become generic parameters of `<Trait>Reply`. The generated interpreter
/// redeclares those associated types, and `interpret` binds the syntax carriers to the handler carriers.
///
/// # Example
///
/// ```
/// # use alux_sdk_macros::trait_algebra;
/// #[trait_algebra(derive(Debug, PartialEq))]
/// trait Counter {
///     async fn add(&self, amount: u64) -> u64;
///     async fn reset(&self);
/// }
///
/// #[derive(Default)]
/// struct Total(u64);
///
/// impl CounterInterpreter for Total {
///     async fn add(&mut self, amount: u64) -> u64 {
///         self.0 += amount;
///         self.0
///     }
///
///     async fn reset(&mut self) {
///         self.0 = 0;
///     }
/// }
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let operation = CounterOp::add(2);
/// assert_eq!(operation, CounterOp::Add { amount: 2 });
///
/// let mut total = Total::default();
/// let reply = operation.interpret(&mut total).await;
/// assert_eq!(reply.into_add(), 2);
///
/// CounterOp::reset().interpret(&mut total).await;
/// assert_eq!(total.0, 0);
/// # }
/// ```
///
/// # Transports
///
/// `transport` states the trait itself for whatever carries its operations to an interpreter
/// elsewhere: a method stating a value asks and takes the value out of the reply, and a method
/// stating none sends and does not stay. Both spellings emit the same bodies, and differ in who is
/// allowed to name a type.
///
/// `transport = <Carrier>` names one carrier, resolved in the author's scope, and states
/// `impl Trait for Carrier<TraitOp, TraitReply>`. It is the right form where the crate that owns the
/// carrier states the impl.
///
/// Bare `transport`, or `transport = capability`, names none and states the impl for every witness
/// of the capabilities:
///
/// ```ignore
/// impl<Carrier> Counter for Carrier
/// where
///     Carrier: AlgebraCall<CounterOp, CounterReply> + AlgebraSend<CounterOp> + Send + Sync,
/// ```
///
/// It is the right form where the trait's own crate states the impl, which the orphan rule makes
/// every case where the trait is declared in a layer that must not know a transport. Only the
/// capabilities the algebra needs are asked for: `AlgebraCall` where any method states a value, and
/// `AlgebraSend` where any states none. `Send + Sync` is asked for because the impl reaches the
/// carrier through a reference, which is also what lets the trait state that its calls may be
/// awaited in another task, as `#[trait_variant::make(Send)]` does.
///
/// Since both capabilities forward through `&` and `Arc`, a borrow and a share of a carrier are the
/// algebra too, so the trait needs no `auto_impl` of its own, and carrying one would conflict with
/// the blanket impl.
///
/// Either way, transport is stated only for an algebra whose carriers are all chosen and whose
/// methods are all asynchronous. `alux-tokio` carries the generated operation and reply types over
/// a bounded channel while leaving stream consumption to the application.
#[proc_macro_attribute]
pub fn trait_algebra(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let definition = parse_macro_input!(item as ItemTrait);

    trait_algebra_internal(attribute.into(), &definition).into()
}
