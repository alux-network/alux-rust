//! What a defunctionalized operation is: its signature, its application, and the handle an
//! interpretation applies it with.

use core::future::Future;
use core::marker::PhantomData;

/// Applies a defunctionalized operation to a context and an argument product.
pub trait ApplyAlg<Context, Args> {
    /// The value produced by applying the operation.
    type Output;

    /// Interprets the operation using the supplied context and arguments.
    fn apply(&self, context: Context, args: Args) -> impl Future<Output = Self::Output> + Send;
}

/// Selects an owned runtime handle for a semantic context.
pub trait HandlerContextAlg<Context: ?Sized> {
    /// The owned carrier cloned into asynchronous operation invocations.
    type Handle: AsRef<Context> + Clone + Send + Sync + 'static;
}

/// Describes the semantic signature of a defunctionalized operation.
pub trait OperationAlg {
    /// The semantic context interpreted by the operation.
    type Context: ?Sized;
    /// The product of arguments accepted by the operation.
    type Args;

    /// The documentation the operation was declared with, as the author wrote it.
    ///
    /// An interpretation that describes a surface to a reader has nowhere else to read what an
    /// operation is for. The first line is its summary and the rest is its description, which is
    /// what a doc comment already means.
    const DOC: &'static str;

    /// The source-level name this operation was declared under.
    ///
    /// An interpretation that names operations to a reader, rather than only applying them, has no
    /// other place to read one from: a type name states where the operation lives, not what it was
    /// called.
    const NAME: &'static str;

    /// The source-level argument names, in declaration order.
    const ARG_NAMES: &'static [&'static str];
}

/// States an argument's type as the method writes it, known without any context.
///
/// A program can check a source against it where the program is declared.
pub struct StatedArg<Arg>(PhantomData<fn() -> Arg>);

impl<Arg> StatedArg<Arg> {
    /// States the argument type `Arg`.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Arg> Default for StatedArg<Arg> {
    fn default() -> Self {
        Self::new()
    }
}

/// States an argument whose type the context or a selected type decides, so only an
/// interpretation that names them can check it.
pub struct ContextArg;

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
        /// Adds `increment` to the value.
        async fn value_plus(&self, increment: u32) -> u32 {
            self.value() + increment
        }
    }

    #[ext(name = DescribeExt)]
    impl<This> This
    where
        This: ValueAlg,
    {
        fn describe(&self) -> String {
            self.value().to_string()
        }
    }

    trait ReadAlg {
        type Item;
        type Key;

        fn read(&self, key: Self::Key) -> Self::Item;
    }

    struct Reader;

    impl ReadAlg for Reader {
        type Item = String;
        type Key = u32;

        fn read(&self, key: u32) -> String {
            key.to_string()
        }
    }

    #[ext(name = ReadExt, defunc)]
    impl<This, Item, Key> This
    where
        This: ReadAlg<Item = Item, Key = Key>,
        Item: Send,
        Key: Send,
    {
        async fn read_suffixed(&self, key: Key, suffix: Item) -> (Item, Item) {
            (self.read(key), suffix)
        }
    }

    #[tokio::test]
    async fn preserves_named_parameters_bound_through_the_context() {
        let reader = Arc::new(Reader);
        let operation = ReadSuffixedOperation::<Reader>::default();

        assert_eq!(
            reader.read_suffixed(4, "!".to_owned()).await,
            operation.apply(reader.clone(), (4, "!".to_owned())).await,
        );
        assert_eq!(<ReadSuffixedOperation<Reader> as OperationAlg>::ARG_NAMES, &["key", "suffix"]);
    }

    #[tokio::test]
    async fn preserves_extension_methods_and_defunctionalizes_their_application() {
        let value = Arc::new(Value(40));

        assert_eq!(value.value_plus(2).await, 42);
        assert_eq!(ValuePlusOperation::<Value>::default().apply(value, (2,)).await, 42);
        assert_eq!(<ValuePlusOperation<Value> as OperationAlg>::NAME, "value_plus");
        assert_eq!(<ValuePlusOperation<Value> as OperationAlg>::DOC, "Adds `increment` to the value.");
        assert_eq!(<ValuePlusOperation<Value> as OperationAlg>::ARG_NAMES, &["increment"]);
    }

    trait BytesAlg {
        type Error;
        type Name;

        fn push(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
        fn leaf(&self, name: &Self::Name) -> Self::Name;
    }

    #[derive(Default)]
    struct Buffer(Vec<u8>);

    impl BytesAlg for Buffer {
        type Error = ();
        type Name = String;

        fn push(&mut self, bytes: &[u8]) -> Result<(), ()> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }

        fn leaf(&self, name: &String) -> String {
            name.rsplit('/').next().unwrap_or_default().to_owned()
        }
    }

    impl AsRef<[u8]> for Buffer {
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    impl AsMut<Buffer> for Buffer {
        fn as_mut(&mut self) -> &mut Buffer {
            self
        }
    }

    #[ext(name = BytesExt, defunc)]
    impl<This> This
    where
        This: BytesAlg,
    {
        /// Appends borrowed bytes.
        fn append(&mut self, bytes: &[u8]) -> Result<(), This::Error> {
            self.push(bytes)
        }

        /// Answers the last segment of each borrowed name.
        fn leaves(&self, names: &[This::Name]) -> Vec<This::Name> {
            names.iter().map(|name| self.leaf(name)).collect()
        }

        /// Answers the first `count` bytes, borrowed from the buffer.
        fn head(&self, count: usize) -> &[u8]
        where
            This: AsRef<[u8]>,
        {
            &self.as_ref()[..count]
        }

        /// Keeps the bytes a predicate the caller selects accepts.
        fn select<Predicate>(&self, bytes: Vec<u8>, predicate: Predicate) -> Vec<u8>
        where
            Predicate: Fn(u8) -> bool,
        {
            bytes.into_iter().filter(|byte| predicate(*byte)).collect()
        }
    }

    #[tokio::test]
    async fn applies_operations_to_borrowed_arguments() {
        let bytes = [1, 2, 3];
        let mut buffer = Buffer::default();
        AppendOperation::<Buffer>::default().apply(&mut buffer, (&bytes[..],)).await.unwrap();
        assert_eq!(buffer.0, [1, 2, 3]);
        assert_eq!(<AppendOperation<Buffer> as OperationAlg>::ARG_NAMES, &["bytes"]);

        let names = ["a/b".to_owned(), "c".to_owned()];
        let leaves = LeavesOperation::<Buffer>::default().apply(Arc::new(Buffer::default()), (&names[..],)).await;
        assert_eq!(leaves, ["b", "c"]);
    }

    #[tokio::test]
    async fn answers_a_borrowed_output_to_a_borrowed_handle() {
        let buffer = Buffer(vec![1, 2, 3]);
        let head = HeadOperation::<Buffer>::default().apply(&buffer, (2,)).await;
        assert_eq!(head, [1, 2]);
        assert_eq!(head, buffer.head(2));
    }

    #[ext(name = PrefixExt, defunc)]
    impl<This> This
    where
        This: AsRef<[u8]> + ?Sized,
    {
        /// Answers the first `count` bytes of any byte carrier, a slice included.
        fn prefix(&self, count: usize) -> &[u8] {
            &self.as_ref()[..count.min(self.as_ref().len())]
        }
    }

    #[tokio::test]
    async fn applies_an_operation_over_an_unsized_carrier() {
        let bytes: &[u8] = &[1, 2, 3];
        assert_eq!(PrefixOperation::<[u8]>::default().apply(bytes, (5,)).await, [1, 2, 3]);
        assert_eq!(PrefixOperation::<[u8]>::default().apply(bytes, (1,)).await, bytes.prefix(1));
    }

    trait Counter {
        fn count(&self) -> usize;
        fn bump(&mut self);
    }

    impl Counter for Buffer {
        fn count(&self) -> usize {
            self.0.len()
        }

        fn bump(&mut self) {
            self.0.push(0);
        }
    }

    #[ext(name = CounterExt, defunc)]
    impl<This> This
    where
        This: Counter + ?Sized,
    {
        /// Answers the count plus `more`.
        fn counted(&self, more: usize) -> usize {
            self.count() + more
        }

        /// Bumps the count and answers it.
        fn bumped(&mut self) -> usize {
            self.bump();
            self.count()
        }
    }

    #[tokio::test]
    async fn applies_an_operation_to_an_unsized_context_through_any_borrow() {
        let counted = CountedOperation::<dyn Counter + Send + Sync>::default();
        let shared: Arc<dyn Counter + Send + Sync> = Arc::new(Buffer(vec![1, 2]));
        assert_eq!(counted.apply(Arc::clone(&shared), (1,)).await, 3);
        assert_eq!(counted.apply(&*shared, (2,)).await, 4);

        let mut owned: Box<dyn Counter + Send + Sync> = Box::new(Buffer(vec![1]));
        let bumped = BumpedOperation::<dyn Counter + Send + Sync>::default();
        assert_eq!(bumped.apply(&mut *owned, ()).await, 2);
        assert_eq!(bumped.apply(owned, ()).await, 3);
    }

    #[tokio::test]
    async fn selects_the_type_parameters_a_method_declares() {
        fn odd(byte: u8) -> bool {
            byte % 2 == 1
        }
        let operation = SelectOperation::<Buffer, fn(u8) -> bool>::default();
        let selected = operation.apply(Arc::new(Buffer::default()), (vec![1, 2, 3], odd as fn(u8) -> bool)).await;
        assert_eq!(selected, [1, 3]);
        assert_eq!(<SelectOperation<Buffer, fn(u8) -> bool> as OperationAlg>::ARG_NAMES, &["bytes", "predicate"]);
    }

    #[test]
    fn remains_compatible_with_ordinary_extensions() {
        assert_eq!(Value(42).describe(), "42");
    }
}
