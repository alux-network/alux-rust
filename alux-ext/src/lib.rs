//! Reifies extension methods as first-order operations with explicit application meaning.

#![allow(async_fn_in_trait)]

extern crate self as alux_ext;

/// Re-exports the procedural-macro implementation shared by ALUX program crates.
///
/// Program crates re-export their own backend attributes from this path so that authored code and
/// generated code refer to the same macro implementation without depending on it directly.
pub use alux_ext_macros as macros;
pub use alux_ext_macros::ext;
/// Re-exports the extension-method implementation referenced by generated code.
///
/// Expansion of [`ext`] names this path, so a crate using the attribute needs no separate
/// `extend` dependency.
pub use extend;

use core::future::Future;

/// Applies a defunctionalized operation to a context and an argument product.
pub trait ApplyAlg<Context, Args> {
    /// The value produced by applying the operation.
    type Output;

    /// Interprets the operation using the supplied context and arguments.
    fn apply(&self, context: Context, args: Args) -> impl Future<Output = Self::Output> + Send;
}

/// Selects an owned runtime handle for a semantic context.
pub trait HandlerContextAlg<Context> {
    /// The owned carrier cloned into asynchronous operation invocations.
    type Handle: AsRef<Context> + Clone + Send + Sync + 'static;
}

/// Describes the semantic signature of a defunctionalized operation.
pub trait OperationAlg {
    /// The semantic context interpreted by the operation.
    type Context;
    /// The product of arguments accepted by the operation.
    type Args;

    /// The source-level argument names, in declaration order.
    const ARG_NAMES: &'static [&'static str];
}

#[cfg(test)]
mod tests {
    use super::{ApplyAlg, OperationAlg, ext};
    use std::sync::Arc;

    trait ValueAlg {
        fn value(&self) -> u32;
    }

    struct Value(u32);

    impl ValueAlg for Value {
        fn value(&self) -> u32 {
            self.0
        }
    }

    #[ext(name = ValueExt, defunc)]
    impl<This> This
    where
        This: ValueAlg,
    {
        async fn value_plus(&self, increment: u32) -> u32 {
            self.value() + increment
        }
    }

    #[ext(name = DescribeExt, supertraits = ValueAlg + Sized)]
    impl<This> This
    where
        This: ValueAlg,
    {
        fn describe(&self) -> String {
            self.value().to_string()
        }
    }

    #[tokio::test]
    async fn preserves_extension_methods_and_defunctionalizes_their_application() {
        let value = Arc::new(Value(40));

        assert_eq!(value.value_plus(2).await, 42);
        assert_eq!(ValuePlusOperation::<Value>::default().apply(value, (2,)).await, 42);
        assert_eq!(<ValuePlusOperation<Value> as OperationAlg>::ARG_NAMES, &["increment"]);
    }

    #[test]
    fn remains_compatible_with_ordinary_extensions() {
        assert_eq!(Value(42).describe(), "42");
    }
}
