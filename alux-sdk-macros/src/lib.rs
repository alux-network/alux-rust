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
/// Transport is a separate interpretation. `alux-tokio` can carry the generated operation and
/// reply types over a bounded channel while leaving stream consumption to the application.
#[proc_macro_attribute]
pub fn trait_algebra(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let definition = parse_macro_input!(item as ItemTrait);

    trait_algebra_internal(attribute.into(), &definition).into()
}
