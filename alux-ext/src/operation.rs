//! What a defunctionalized operation is: its signature, its application, and the handle an
//! interpretation applies it with.

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
    use super::{ApplyAlg, OperationAlg};
    use crate::ext;
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
